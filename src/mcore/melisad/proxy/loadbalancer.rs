use std::sync::Arc;
/// Load balancing strategies untuk node selection
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::mcore::melisad::services::node::{NodeManager, NodeProcess};

#[derive(Debug, Clone, Copy)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,

    /// Least connections
    LeastConnections,

    /// Random selection
    Random,
}

#[derive(Clone)]
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    round_robin_index: Arc<AtomicUsize>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        LoadBalancer {
            strategy,
            round_robin_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Select node berdasarkan domain, path, dan strategy
    pub fn select_node(
        &self,
        domain: &str,
        path: &str,
        node_manager: &NodeManager,
    ) -> Option<NodeProcess> {
        let mut matching_nodes = node_manager.find_matching_nodes_by_route(domain, path);

        if matching_nodes.is_empty() {
            return None;
        }

        // Select based on strategy
        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let idx =
                    self.round_robin_index.fetch_add(1, Ordering::Relaxed) % matching_nodes.len();
                Some(matching_nodes[idx].clone())
            }
            LoadBalancingStrategy::LeastConnections => {
                // Simplified: sort by PID
                matching_nodes.sort_by_key(|n| n.pid);
                Some(matching_nodes[0].clone())
            }
            LoadBalancingStrategy::Random => {
                let idx = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as usize
                    % matching_nodes.len();
                Some(matching_nodes[idx].clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancer_creation() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
        assert_eq!(lb.round_robin_index.load(Ordering::Relaxed), 0);
    }
}
