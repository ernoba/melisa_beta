use futures;
use std::fs;

use crate::mcore::errors::e_node::NodeError;
use crate::mcore::melisad::services::mconf::NODE_FILE;
use crate::mcore::melisad::services::node::{NodeManager, NodeProcess, NodeStatus};

// Fungsi baru untuk mencari node yang cocok berdasarkan domain dan path request
impl NodeManager {
    pub fn find_node_by_route(&self, domain: &str, path: &str) -> Option<NodeProcess> {
        let processes_lock = self.processes.read().unwrap();
        
        processes_lock.values()
            // Pastikan hanya memilih node yang aktif
            .filter(|node| node.status == NodeStatus::Active)
            // Cari yang domain-nya cocok DAN path-nya diawali dengan route_path si node
            .find(|node| {
                let domain_match = node.domain == domain || domain.starts_with(&format!("{}:", node.domain));
                let path_match = path.starts_with(&node.route_path);
                domain_match && path_match
            })
            .cloned()
    }
}