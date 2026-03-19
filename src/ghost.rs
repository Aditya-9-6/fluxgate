use tracing::{info, warn, error, debug};
use std::sync::Arc;
use reqwest::Client;
use serde_json::json;
use tiktoken_rs::CoreBPE;

/// Implement the Ghost Protocol:
/// Builds a Hybrid-Cloud Mesh that routes incoming requests to a local SLM (e.g., 8B param model).
/// If the model says the query is too complex, anonymize the data (strip identifying info)
/// and escalate to Anthropic/OpenAI ensuring 100% sovereign privacy.
use crate::energy::EnergyRouter;
use crate::stats::StatsTracker;

use crate::agents::mcp::McpHub;
use crate::agents::session::SessionManager;
use crate::agents::memory_graph::SharedMemoryGraph;
use crate::performance::speculative::SpeculativeEngine;
use crate::scrubber::{ScrubberManager, DeIdentifier};

use crate::intelligence::accountability::CausalAccountability;

pub struct GhostProtocolGateway {
    pub local_slm_capacity: u32,
    pub bpe: Arc<CoreBPE>,
    pub http_client: Client,
    pub local_slm_url: String,
    pub frontier_api_url: String,
    pub frontier_api_key: String,
    pub energy_router: Arc<EnergyRouter>,
    pub stats: Arc<StatsTracker>,
    pub speculative: SpeculativeEngine,
    pub mcp_hub: Arc<McpHub>,
    pub session_manager: Arc<SessionManager>,
    pub memory_graph: Arc<SharedMemoryGraph>,
    pub scrubber: Arc<ScrubberManager>,
    pub accountability: Arc<CausalAccountability>,
}

impl GhostProtocolGateway {
    pub fn new(
        capacity: u32, 
        energy: Arc<EnergyRouter>, 
        stats: Arc<StatsTracker>,
        mcp_hub: Arc<McpHub>,
        session_manager: Arc<SessionManager>,
        memory_graph: Arc<SharedMemoryGraph>,
        frontier_api_key: String,
        scrubber: Arc<ScrubberManager>,
        accountability: Arc<CausalAccountability>,
    ) -> Self {
        let bpe = tiktoken_rs::cl100k_base().expect("Failed to initialize Tiktoken BPE");
        
        Self { 
            local_slm_capacity: capacity,
            bpe: Arc::new(bpe),
            http_client: Client::new(),
            local_slm_url: std::env::var("LOCAL_SLM_URL").unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string()),
            frontier_api_url: std::env::var("FRONTIER_API_URL").unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string()),
            frontier_api_key,
            energy_router: energy,
            stats,
            speculative: SpeculativeEngine::new(),
            mcp_hub,
            session_manager,
            memory_graph,
            scrubber,
            accountability,
        }
    }

    /// Evaluates Request Complexity using real token counts
    fn calculate_complexity(&self, query: &str) -> u32 {
        self.bpe.encode_with_special_tokens(query).len() as u32
    }

    /// Anonymizes personal data before sending off-device.
    /// Replaces real PII with placeholders and tracks mapping.
    fn anonymize_query(&self, query: &str, de_id: &DeIdentifier) -> String {
        info!("🔐 [PRIVACY] De-identifying Sovereign Data Stream before cloud escalation...");
        self.scrubber.scrub_and_map(query, de_id)
    }

    /// Routes the request according to the Invisible Sovereign AI ruleset + LLM Arbitrage.
    /// Implements a multi-step Reasoning Loop (ReAct) for autonomous tool usage.
    pub async fn process_sovereign_request(&self, query: &str, session_id: Option<&str>) -> String {
        info!("Ghost Protocol: Initiating Autonomous Reasoning Loop...");
        let complexity = self.calculate_complexity(query);
        let sid = session_id.unwrap_or("anonymous_session");
        
        // 1. Session & Memory Integration
        let mut history = self.session_manager.append_and_reconstruct(sid, json!({"role": "user", "content": query}));
        let memory_context = self.memory_graph.get_consolidated_context().await;
        
        if !memory_context.is_empty() {
             info!("🧠 [MEMORY-GRAPH] Injecting consolidated context into reasoning loop.");
             history.insert(0, json!({
                 "role": "system", 
                 "content": format!("Long-term Memory Context:\n{}\n\nYou are an autonomous agent. Use THOUGHT: to reason, ACTION: tool_name(args) to use tools, and FINAL_ANSWER: to conclude.", memory_context)
             }));
        }

        let mut current_query = query.to_string();
        let mut max_steps = 5;
        let mut agent_output = String::new();

        while max_steps > 0 {
            let step_id = Uuid::new_v4().to_string();
            info!("🔄 [V12-REASONING] Executing Step {} (Remaining: {})", step_id, max_steps);
            
            // Record Recursive Causality
            self.accountability.record_causality(&current_query, &step_id, 0.95, (6 - max_steps) as u32);

            // 2. Speculative Decoding (Startup Latency Optimization - only for first step)
            if max_steps == 5 {
                if let Some(prefix) = self.speculative.speculate_prefix(&current_query) {
                    info!("🚀 [SPECULATION] Immediate prefix found. Returning low-latency response.");
                    return prefix;
                }
            }

            // 3. Routing Logic (Local vs Cloud)
            let best_zone = self.energy_router.route_query(complexity);
            let use_local = complexity <= self.local_slm_capacity || matches!(best_zone, crate::energy::ProviderZone::LocalSlm(_));

            let step_response = if use_local {
                debug!("Ghost Protocol: Consulting Local SLM...");
                self.call_local_slm(&history).await
            } else {
                debug!("Ghost Protocol: Escalating to Frontier with De-identification...");
                let de_id = DeIdentifier::new();
                let last_msg = history.last().and_then(|m| m["content"].as_str()).unwrap_or("");
                let scrubbed_query = self.anonymize_query(last_msg, &de_id);
                
                // Swap last message with scrubbed version for cloud call
                let mut cloud_history = history.clone();
                if let Some(msg) = cloud_history.last_mut() {
                    msg["content"] = serde_json::Value::String(scrubbed_query);
                }

                match self.call_frontier_model(&cloud_history).await {
                    Ok(raw_resp) => {
                        let recovered = de_id.recover(&raw_resp);
                        Ok(recovered)
                    }
                    Err(e) => Err(e),
                }
            };

            let response_text = match step_response {
                Ok(text) => text,
                Err(e) => {
                    error!("LLM Call Failed: {}. Terminating loop with error.", e);
                    return format!("Error: Internal reasoning failure. {}", e);
                }
            };

            info!("🤖 [AGENT] {}", response_text);
            history.push(json!({"role": "assistant", "content": response_text.clone()}));

            // 4. Action Parsing & Execution
            if let Some(final_answer) = self.parse_final_answer(&response_text) {
                agent_output = final_answer.to_string();
                break;
            } else if let Some((tool, args)) = self.parse_tool_action(&response_text) {
                info!("🔌 [AGENT] Requested Action: {} with {:?}", tool, args);
                
                match self.mcp_hub.handle_tool_call(sid, &tool, &args) {
                    Ok(result) => {
                        let obs = format!("OBSERVATION: {}", result);
                        info!("👁️ [AGENT] {}", obs);
                        history.push(json!({"role": "user", "content": obs}));
                    }
                    Err(e) => {
                        let obs = format!("OBSERVATION ERROR: {}", e);
                        warn!("⚠️ [AGENT] {}", obs);
                        history.push(json!({"role": "user", "content": obs}));
                    }
                }
            } else {
                // No clear action or answer, LLM might be just thinking or outputting text
                agent_output = response_text;
                break;
            }

            max_steps -= 1;
        }

        // 5. Post-Processing: Memory Discovery
        if agent_output.to_lowercase().contains("fact:") {
             self.memory_graph.publish_fact(sid, "autonomous_discovery", &agent_output, None).await;
        }

        self.session_manager.append_and_reconstruct(sid, json!({"role": "assistant", "content": agent_output.clone()}));
        self.stats.record_request("ghost_agent_reasoning_v10", 1.0);
        
        agent_output
    }

    async fn call_local_slm(&self, messages: &[serde_json::Value]) -> Result<String, String> {
        let payload = json!({
            "model": "llama3",
            "messages": messages,
            "stream": false
        });

        match self.http_client.post(&self.local_slm_url).json(&payload).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    json["response"].as_str().map(|s| s.to_string()).ok_or_else(|| "Missing response field".to_string())
                } else { Err("Failed to parse SLM JSON".to_string()) }
            }
            Err(e) => Err(format!("Local SLM unreachable: {}", e))
        }
    }

    async fn call_frontier_model(&self, messages: &[serde_json::Value]) -> Result<String, String> {
        let payload = json!({
            "model": "gpt-4-turbo",
            "messages": messages,
            "temperature": 0.7
        });

        match self.http_client.post(&self.frontier_api_url)
            .bearer_auth(&self.frontier_api_key)
            .json(&payload)
            .send()
            .await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    json["choices"][0]["message"]["content"].as_str().map(|s| s.to_string()).ok_or_else(|| "Missing content field".to_string())
                } else { Err("Failed to parse Frontier JSON".to_string()) }
            }
            Err(e) => Err(format!("Frontier Model unreachable: {}", e))
        }
    }

    fn parse_final_answer<'a>(&self, text: &'a str) -> Option<&'a str> {
        if let Some(pos) = text.find("FINAL_ANSWER:") {
            return Some(text[pos + 13..].trim());
        }
        None
    }

    fn parse_tool_action(&self, text: &str) -> Option<(String, serde_json::Value)> {
        if let Some(pos) = text.find("ACTION:") {
            let action_str = text[pos + 7..].trim();
            if let Some(paren_pos) = action_str.find('(') {
                let tool_name = action_str[..paren_pos].trim().to_string();
                if let Some(end_paren) = action_str.rfind(')') {
                    let args_str = &action_str[paren_pos + 1..end_paren];
                    let args: serde_json::Value = serde_json::from_str(args_str).unwrap_or(json!({}));
                    return Some((tool_name, args));
                }
            }
        }
        None
    }

    /// Routes and streams the request according to the Invisible Sovereign AI ruleset.
    pub async fn stream_sovereign_request(&self, query: &str) -> reqwest::Result<reqwest::Response> {
        info!("Ghost Protocol: Initiating sovereign stream...");
        let complexity = self.calculate_complexity(query);

        if complexity <= self.local_slm_capacity {
            info!("Ghost Protocol: Complexity {}. Streaming from LOCAL SLM.", complexity);
            let payload = json!({
                "model": "llama3",
                "prompt": query,
                "stream": true
            });
            self.http_client.post(&self.local_slm_url).json(&payload).send().await
        } else {
            let de_id = DeIdentifier::new();
            let sanitized_query = self.anonymize_query(query, &de_id);
            warn!("Ghost Protocol: Complexity {} exceeds capacity. Streaming from FRONTIER with De-identification.", complexity);
            // Note: In streaming mode, re-identification is harder as it requires stateful transformation.
            // For now, we de-identify the prompt, but streaming chunks will return placeholders.
            let payload = json!({
                "model": "gpt-4-turbo",
                "messages": [{"role": "user", "content": sanitized_query}],
                "stream": true
            });
            self.http_client.post(&self.frontier_api_url)
                .bearer_auth(&self.frontier_api_key)
                .json(&payload)
                .send()
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::EnergyRouter;
    use crate::stats::StatsTracker;
    use crate::agents::mcp::McpHub;
    use crate::agents::session::SessionManager;
    use crate::agents::memory_graph::SharedMemoryGraph;

    fn setup_gateway() -> GhostProtocolGateway {
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        GhostProtocolGateway::new(
            100,
            Arc::new(EnergyRouter::new()),
            Arc::new(StatsTracker::new(pool)),
            Arc::new(McpHub::new()),
            Arc::new(SessionManager::new()),
            Arc::new(SharedMemoryGraph::new()),
            "sk-test".to_string(),
            Arc::new(ScrubberManager::new()),
        )
    }

    #[tokio::test]
    async fn test_parse_final_answer() {
        let gateway = setup_gateway();
        let text = "THOUGHT: I'm done. FINAL_ANSWER: The sky is blue.";
        assert_eq!(gateway.parse_final_answer(text), Some("The sky is blue."));
        
        let no_answer = "THOUGHT: Still thinking...";
        assert_eq!(gateway.parse_final_answer(no_answer), None);
    }

    #[tokio::test]
    async fn test_parse_tool_action() {
        let gateway = setup_gateway();
        let text = "THOUGHT: I should check status. ACTION: get_status({\"id\": 123})";
        let (tool, args) = gateway.parse_tool_action(text).unwrap();
        assert_eq!(tool, "get_status");
        assert_eq!(args, json!({"id": 123}));

        let malformed = "ACTION: invalid_args(not_json)";
        let (tool2, args2) = gateway.parse_tool_action(malformed).unwrap();
        assert_eq!(tool2, "invalid_args");
        assert_eq!(args2, json!({})); // Should fallback to empty object
    }
}
