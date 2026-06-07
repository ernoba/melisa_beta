mod mcore;

use mcore::services::node::{NodeManager, NodeError};

// Di mulai untuk umat manusia
// Juni 2026
// Kita ke ijen kan?
// Kamu masih ingetkan ...f
fn main() {
    let name = "Example Pfsfdsfrocsffff";
    let pid = 124346665;
    
    // Ambil instance manager sekali saja agar kode lebih bersih
    let manager = NodeManager::get_instance();

    // Tangani proses pembuatan secara elegan
    match manager.create(name, pid) {
        Ok(_) => println!("Node '{}' berhasil dibuat.", name),
        Err(NodeError::AlreadyExists) => println!("Info: Node '{}' sudah aktif sebelumnya.", name),
        Err(e) => println!("Error tidak terduga saat membuat node: {:?}", e),
    }

    // Ambil daftar hash (Pastikan nama method di NodeManager kamu adalah `list`)
    let hashes = match manager.list() {
        Some(v) => v,
        None => {
            println!("Tidak ada proses node yang ditemukan.");
            return;
        }
    };

    println!("\n--- Daftar Hash Node Aktif ---");
    for (index, hash) in hashes.iter().enumerate() {
        println!("{}. {}", index + 1, hash);
    }
}
