use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    Code,
    Reasoning,
    Summary,
    General,
}

pub struct ModelOptimizer {
    pub history: Arc<RwLock<HashMap<TaskType, String>>>,
}

impl ModelOptimizer {
    pub fn new() -> Self {
        let mut initial_history = HashMap::new();
        initial_history.insert(TaskType::Code, "claude-3-5-sonnet".to_string());
        initial_history.insert(TaskType::Reasoning, "gpt-4o-2024-05-13".to_string());
        initial_history.insert(TaskType::Summary, "llama-3-8b".to_string());
        initial_history.insert(TaskType::General, "gpt-4o-mini".to_string());

        Self {
            history: Arc::new(RwLock::new(initial_history)),
        }
    }

    /// Classifies the task type of a given prompt.
    pub fn classify_task(&self, prompt: &str) -> TaskType {
        let p = prompt.to_lowercase();
        if p.contains("function") || p.contains("code") || p.contains("rust") {
            TaskType::Code
        } else if p.contains("solve") || p.contains("reason") || p.contains("math") {
            TaskType::Reasoning
        } else if p.contains("summarize") || p.contains("shorten") {
            TaskType::Summary
        } else {
            TaskType::General
        }
    }

    /// Selects the optimal model for the detected task.
    pub async fn route_optimal(&self, task: TaskType) -> String {
        let history = self.history.read().await;
        let model = history.get(&task).cloned().unwrap_or_else(|| "gpt-4o-mini".to_string());
        info!("🎯 [OPTIMIZER] Optimal model selected for task {:?}: {}", task, model);
        model
    }
}
