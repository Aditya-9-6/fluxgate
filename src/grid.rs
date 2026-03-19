pub struct GridManager;

impl GridManager {
    pub fn new() -> Self { Self }

    pub fn select_best_worker(&self, ram_req: u64, gpu_req: f32) -> Option<WorkerNode> {
        Some(WorkerNode { node_id: "worker-01".to_string() })
    }
}

pub struct WorkerNode {
    pub node_id: String,
}
