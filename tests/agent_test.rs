use fluxgate::ghost::GhostProtocolGateway;
use fluxgate::agents::mcp::McpHub;
use fluxgate::agents::session::SessionManager;
use fluxgate::agents::memory_graph::SharedMemoryGraph;
use fluxgate::energy::EnergyRouter;
use fluxgate::stats::StatsTracker;
use std::sync::Arc;
use sqlx::PgPool;
use serde_json::json;

async fn setup_test_gateway() -> GhostProtocolGateway {
    let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
    let stats = Arc::new(StatsTracker::new(pool));
    GhostProtocolGateway::new(
        100,
        Arc::new(EnergyRouter::new()),
        stats,
        Arc::new(McpHub::new()),
        Arc::new(SessionManager::new()),
        Arc::new(SharedMemoryGraph::new()),
    )
}

#[tokio::test]
async fn test_session_history_persistence() {
    let gateway = setup_test_gateway().await;
    let session_id = "test_session_1";
    
    // First interaction
    let msg1 = json!({"role": "user", "content": "My name is Alice."});
    
    let history = gateway.session_manager.append_and_reconstruct(session_id, msg1);
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["content"], "My name is Alice.");
    
    // Second interaction
    let msg2 = json!({"role": "user", "content": "What is my name?"});
    let history2 = gateway.session_manager.append_and_reconstruct(session_id, msg2);
    assert_eq!(history2.len(), 2);
}

#[tokio::test]
async fn test_memory_fact_retrieval() {
    let gateway = setup_test_gateway().await;
    let session_id = "test_session_2";
    
    // Store a fact
    gateway.memory_graph.publish_fact(session_id, "user_preference", "Likes coffee", None).await;
    
    // Retrieve specific fact
    let fact = gateway.memory_graph.subscribe_fact("user_preference").await;
    assert!(fact.is_some());
    assert_eq!(fact.unwrap().value, "Likes coffee");
    
    // Consolidated context check
    let context = gateway.memory_graph.get_consolidated_context().await;
    assert!(context.contains("user_preference: Likes coffee"));
}

#[tokio::test]
async fn test_mcp_tool_execution_routing() {
    let gateway = setup_test_gateway().await;
    let session_id = "test_session_3";
    
    // In a real test, we'd register a tool and call it.
    let args = json!({});
    let result = gateway.mcp_hub.handle_tool_call(session_id, "unknown_tool", &args);
    
    match result {
        Ok(val) => {
            assert_eq!(val["status"], "success");
            assert!(val["result"].as_str().unwrap().contains("Executed unknown_tool"));
        },
        Err(e) => panic!("Should have succeeded with mock: {}", e),
    }
}
