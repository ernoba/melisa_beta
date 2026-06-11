/// Node data models untuk Melisa
use serde::{Deserialize, Serialize};

/// Representasi dari satu backend node/upstream server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeProcess {
    /// Unique hash identifier untuk node (skip dari serialization)
    #[serde(skip)]
    pub hash: String,

    /// Nama deskriptif node
    pub name: String,

    /// Process ID atau identifier unik
    pub pid: u32,

    /// URL upstream server (misal: http://127.0.0.1:3000)
    pub url: String,

    /// Domain/hostname yang handle node ini
    pub domain: String,

    /// Route path prefix yang di-handle node ini (misal: /api/users)
    pub route_path: String,

    /// Status operasional node
    pub status: NodeStatus,
}

/// Status operasional dari satu node
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeStatus {
    /// Node aktif dan siap menerima requests
    Active,

    /// Node stopped/maintenance
    Stopped,

    /// Status tidak diketahui (health check pending)
    Unknown,
}
