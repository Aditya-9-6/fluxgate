use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tracing::{warn, debug};

pub struct LoopProtector {
    agent_graph: Mutex<HashMap<String, Vec<String>>>,
}

impl LoopProtector {
    pub fn new() -> Self {
        LoopProtector {
            agent_graph: Mutex::new(HashMap::new()),
        }
    }

    /// Records an edge in the agentic dependency graph and checks for SCC (Strongly Connected Components).
    pub fn register_dependency(&self, agent_from: &str, agent_to: &str) -> bool {
        let mut graph = self.agent_graph.lock().unwrap();
        graph.entry(agent_from.to_string()).or_default().push(agent_to.to_string());

        if self.find_deadlocks(&graph) {
            warn!("🔄 [TARJAN-SCC] Deadlock detected in agent graph! Cyclic dependency involving {} found.", agent_from);
            return false;
        }
        true
    }

    /// Implements Tarjan's SCC to detect cycles in the graph.
    fn find_deadlocks(&self, graph: &HashMap<String, Vec<String>>) -> bool {
        let mut indices: HashMap<String, i32> = HashMap::new();
        let mut lowlink: HashMap<String, i32> = HashMap::new();
        let mut stack: Vec<String> = Vec::new();
        let mut on_stack: HashSet<String> = HashSet::new();
        let mut index = 0;
        let mut cycle_found = false;

        for node in graph.keys() {
            if !indices.contains_key(node) {
                self.strongconnect(node, graph, &mut index, &mut indices, &mut lowlink, &mut stack, &mut on_stack, &mut cycle_found);
            }
            if cycle_found { break; }
        }
        cycle_found
    }

    fn strongconnect(
        &self,
        v: &str,
        graph: &HashMap<String, Vec<String>>,
        index: &mut i32,
        indices: &mut HashMap<String, i32>,
        lowlink: &mut HashMap<String, i32>,
        stack: &mut Vec<String>,
        on_stack: &mut HashSet<String>,
        cycle_found: &mut bool,
    ) {
        indices.insert(v.to_string(), *index);
        lowlink.insert(v.to_string(), *index);
        *index += 1;
        stack.push(v.to_string());
        on_stack.insert(v.to_string());

        if let Some(edges) = graph.get(v) {
            for w in edges {
                if !indices.contains_key(w) {
                    self.strongconnect(w, graph, index, indices, lowlink, stack, on_stack, cycle_found);
                    if let (Some(&v_low), Some(&w_low)) = (lowlink.get(v), lowlink.get(w)) {
                        lowlink.insert(v.to_string(), v_low.min(w_low));
                    }
                } else if on_stack.contains(w) {
                    if let (Some(&v_low), Some(&w_idx)) = (lowlink.get(v), indices.get(w)) {
                        lowlink.insert(v.to_string(), v_low.min(w_idx));
                    }
                }
            }
        }

        if lowlink.get(v) == indices.get(v) {
            let mut component_count = 0;
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                component_count += 1;
                if w == v { break; }
            }
            if component_count > 1 || (component_count == 1 && graph.get(v).map_or(false, |e| e.contains(&v.to_string()))) {
                *cycle_found = true;
            }
        }
    }
}
