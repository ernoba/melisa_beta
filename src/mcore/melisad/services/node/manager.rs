use once_cell::sync::Lazy;
/// Core NodeManager - manages collection of backend nodes
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock, atomic::AtomicUsize};

use crate::mcore::config::load_config::CONFIG;
use crate::mcore::melisad::services::node::models::NodeProcess;

/// NodeManager manages all registered backend nodes
/// Uses Arc<RwLock> + Copy-on-Write semantics untuk thread-safe updates
pub struct NodeManager {
    /// HashMap of nodes, wrapped in Arc untuk zero-copy sharing
    pub processes: RwLock<Arc<HashMap<String, NodeProcess>>>,

    /// Track cumulative bytes untuk trigger flush
    pub accumulated_bytes: AtomicUsize,

    /// Path ke storage file (dari config)
    pub storage_path: String,
}

/// Global singleton instance
pub static NODE_MANAGER: Lazy<NodeManager> = Lazy::new(|| {
    let storage_path = CONFIG.nodes.storage_file.clone();
    NodeManager::new(&storage_path)
});

impl NodeManager {
    /// Inisialisasi NodeManager dengan membaca dari file
    pub fn new(path: &str) -> Self {
        let processes: HashMap<String, NodeProcess> = match fs::read_to_string(path) {
            Ok(content) if !content.trim().is_empty() => {
                let mut loaded: HashMap<String, NodeProcess> =
                    serde_json::from_str(&content).unwrap_or_default();

                for (hash, node) in loaded.iter_mut() {
                    node.hash = hash.clone();
                }

                loaded
            }
            _ => HashMap::new(),
        };

        NodeManager {
            processes: RwLock::new(Arc::new(processes)),
            accumulated_bytes: AtomicUsize::new(0),
            storage_path: path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_node_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let node_file = temp_dir.path().join("test-nodes.json");

        // Create empty node file
        fs::write(&node_file, "{}").unwrap();

        let manager = NodeManager::new(node_file.to_str().unwrap());
        assert_eq!(manager.storage_path, node_file.to_str().unwrap());
    }

    #[test]
    fn test_node_manager_load_existing() {
        let temp_dir = TempDir::new().unwrap();
        let node_file = temp_dir.path().join("test-nodes.json");

        // Create file dengan existing nodes
        let test_data = r#"{
            "abc123": {
                "name": "test-node",
                "pid": 100000,
                "url": "http://localhost:3000",
                "domain": "test.local",
                "route_path": "/api",
                "status": "Active"
            }
        }"#;
        fs::write(&node_file, test_data).unwrap();

        let manager = NodeManager::new(node_file.to_str().unwrap());
        let nodes_lock = manager.processes.read().unwrap();
        assert_eq!(nodes_lock.len(), 1);
    }
}
