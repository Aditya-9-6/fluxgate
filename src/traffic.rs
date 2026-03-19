use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tracing::warn;
use redis::AsyncCommands;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Agent State
#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    Thinking,
    WaitingFor(String, u64), // Waiting for another agent's output, with timestamp
    Resolving,
}

impl AgentState {
    fn to_string(&self) -> String {
        match self {
            AgentState::Thinking => "thinking".to_string(),
            AgentState::WaitingFor(id, ts) => format!("waiting_for:{}:{}", id, ts),
            AgentState::Resolving => "resolving".to_string(),
        }
    }

    fn from_string(val: &str) -> Self {
        if val == "thinking" {
            AgentState::Thinking
        } else if val == "resolving" {
            AgentState::Resolving
        } else if val.starts_with("waiting_for:") {
            let parts: Vec<&str> = val.split(':').collect();
            if parts.len() >= 3 {
                let id = parts[1];
                let ts = parts[2].parse::<u64>().unwrap_or(0);
                AgentState::WaitingFor(id.to_string(), ts)
            } else {
                AgentState::WaitingFor("unknown".to_string(), 0)
            }
        } else {
            AgentState::Thinking // Default fallback
        }
    }
}

/// Implement "Multi-Agent Collision Avoidance" (The Traffic Controller):
/// Agentic Deadlock Detection via Tarjan's SCC. Acts as the "Air Traffic Controller".
/// Stores state in Redis, detects Wait-State Timeouts, and exposes execution DAG.
pub struct AgentTrafficController {
    pub redis: redis::Client,
    pub prefix: String,
    pub timeout_seconds: u64,
    pub hot_graph: Arc<Mutex<HashMap<String, AgentState>>>,
}

struct TarjanEnv {
    index: usize,
    indices: HashMap<String, usize>,
    lowlink: HashMap<String, usize>,
    on_stack: HashSet<String>,
    stack: Vec<String>,
    sccs: Vec<Vec<String>>,
}

impl AgentTrafficController {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let hot_graph = Arc::new(Mutex::new(HashMap::new()));
        
        let atc = Self {
            redis: client,
            prefix: "fluxgate:atc:state:".to_string(),
            timeout_seconds: 10,
            hot_graph: hot_graph.clone(),
        };

        // Periodic Background Sync: Redis -> Hot Graph
        let redis_clone = atc.redis.clone();
        let prefix_clone = atc.prefix.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                if let Ok(mut conn) = redis_clone.get_multiplexed_async_connection().await {
                    let match_pattern = format!("{}*", prefix_clone);
                    if let Ok(keys) = conn.keys::<_, Vec<String>>(&match_pattern).await {
                        let mut new_states = HashMap::new();
                        for key in keys {
                            if let Ok(Some(val)) = conn.get::<_, Option<String>>(&key).await {
                                let agent_id = key.trim_start_matches(&prefix_clone).to_string();
                                new_states.insert(agent_id, AgentState::from_string(&val));
                            }
                        }
                        {
                            let mut graph = hot_graph.lock().await;
                            *graph = new_states;
                        }
                    }
                }
            }
        });

        Ok(atc)
    }

    pub async fn update_agent(&self, agent_id: &str, state_type: &str, waiting_for: Option<&str>) {
        let state = match state_type {
            "waiting" => {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                AgentState::WaitingFor(waiting_for.unwrap_or("unknown").to_string(), current_time)
            },
            "resolving" => AgentState::Resolving,
            _ => AgentState::Thinking,
        };
        
        {
            let mut graph = self.hot_graph.lock().await;
            graph.insert(agent_id.to_string(), state.clone());
        }

        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let key = format!("{}{}", self.prefix, agent_id);
            let _: Result<(), _> = conn.set_ex(key, state.to_string(), 3600).await;
        }
    }

    pub async fn update_agent_state(&self, agent_id: &str, state: AgentState) {
        let state = match state {
            AgentState::WaitingFor(id, _) => {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                AgentState::WaitingFor(id, current_time)
            },
            s => s
        };
        {
            let mut graph = self.hot_graph.lock().await;
            graph.insert(agent_id.to_string(), state.clone());
        }
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let key = format!("{}{}", self.prefix, agent_id);
            let _: Result<(), _> = conn.set_ex(key, state.to_string(), 3600).await;
        }
    }

    /// Run Tarjan's SCC algorithm on the current wait-graph
    fn tarjan_scc(adj_list: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
        let mut env = TarjanEnv {
            index: 0,
            indices: HashMap::new(),
            lowlink: HashMap::new(),
            on_stack: HashSet::new(),
            stack: Vec::new(),
            sccs: Vec::new(),
        };

        for v in adj_list.keys() {
            if !env.indices.contains_key(v) {
                Self::strongconnect(v, adj_list, &mut env);
            }
        }
        env.sccs
    }

    fn strongconnect(v: &str, adj_list: &HashMap<String, Vec<String>>, env: &mut TarjanEnv) {
        env.indices.insert(v.to_string(), env.index);
        env.lowlink.insert(v.to_string(), env.index);
        env.index += 1;
        env.stack.push(v.to_string());
        env.on_stack.insert(v.to_string());

        if let Some(neighbors) = adj_list.get(v) {
            for w in neighbors {
                if !env.indices.contains_key(w) {
                    Self::strongconnect(w, adj_list, env);
                    let low_v = *env.lowlink.get(v).unwrap();
                    let low_w = *env.lowlink.get(w).unwrap();
                    env.lowlink.insert(v.to_string(), std::cmp::min(low_v, low_w));
                } else if env.on_stack.contains(w) {
                    let low_v = *env.lowlink.get(v).unwrap();
                    let index_w = *env.indices.get(w).unwrap();
                    env.lowlink.insert(v.to_string(), std::cmp::min(low_v, index_w));
                }
            }
        }

        if env.lowlink.get(v) == env.indices.get(v) {
            let mut scc = Vec::new();
            loop {
                if let Some(w) = env.stack.pop() {
                    env.on_stack.remove(&w);
                    scc.push(w.clone());
                    if w == v {
                        break;
                    }
                } else {
                    break;
                }
            }
            env.sccs.push(scc);
        }
    }

    pub async fn detect_deadlocks(&self) -> bool {
        let mut loop_detected = false;
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // S2.1 Optimize: Read from Hot Graph
        let current_states = {
            let graph = self.hot_graph.lock().await;
            graph.clone()
        };

        if current_states.is_empty() { return false; }

        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut agents_to_resolve: Vec<String> = Vec::new();

        for (agent_a, state_a) in &current_states {
            if let AgentState::WaitingFor(agent_b, ts) = state_a {
                if current_time > *ts && (current_time - *ts) > self.timeout_seconds {
                    warn!("🛑 TIMEOUT: Agent '{}' waiting for '{}' >10s.", agent_a, agent_b);
                    agents_to_resolve.push(agent_a.clone());
                    loop_detected = true;
                } else {
                    adj_list.entry(agent_a.clone()).or_insert_with(Vec::new).push(agent_b.clone());
                }
            }
        }

        let sccs = Self::tarjan_scc(&adj_list);
        for component in sccs {
            if component.len() > 1 {
                warn!("🛑 DEADLOCK DETECTED: Cycle in agents {:?}", component);
                for agent in component { agents_to_resolve.push(agent); }
                loop_detected = true;
            }
        }

        if !agents_to_resolve.is_empty() {
            if let Ok(mut graph) = self.hot_graph.try_lock() {
                if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
                    for agent in agents_to_resolve {
                        graph.insert(agent.clone(), AgentState::Resolving);
                        let key = format!("{}{}", self.prefix, agent);
                        let _: Result<(), _> = conn.set_ex(key, AgentState::Resolving.to_string(), 3600).await;
                    }
                }
            }
        }

        loop_detected
    }
}
