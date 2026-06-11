// src/mcore/melisad/proxy/mod.rs

use std::net::SocketAddr;
use std::sync::Arc;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;
use http_body_util::Full;
use hyper::body::Bytes;

use crate::mcore::melisad::services::node::NodeManager;
use crate::mcore::config::load_config::CONFIG;

/// Fungsi utama untuk menjalankan server proxy Melisa
pub async fn run_proxy_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Mengambil konfigurasi host dan port utama dari config.toml (ex: 127.0.0.1:8080)
    let addr: SocketAddr = format!("{}:{}", CONFIG.host, CONFIG.port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    
    println!("🚀 Melisa Proxy Gateway berjalan di http://{}", addr);

    // Reusable HTTP Client dari reqwest untuk meneruskan request ke upstream node
    let client = Arc::new(reqwest::Client::new());
    let node_manager = NodeManager::get_instance();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        
        let client_clone = Arc::clone(&client);
        
        // Menangani koneksi secara asinkronus
        tokio::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                let client = Arc::clone(&client_clone);
                handle_proxy_request(req, client)
            });

            if let Err(err) = auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, service)
                .await
            {
                eprintln!("Error saat melayani koneksi proxy: {:?}", err);
            }
        });
    }
}

/// Logika inti penentuan arah (Routing Layer)
async fn handle_proxy_request(
    req: Request<Incoming>,
    client: Arc<reqwest::Client>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let node_manager = NodeManager::get_instance();

    // 1. Ekstrak Domain/Host dari Header
    let host = req.headers()
        .get(hyper::header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // 2. Ekstrak Path dari URI
    let path = req.uri().path();

    println!("[PROXY LOG] Request Masuk -> Host: {}, Path: {}", host, path);

    // 3. Tanya ke NodeManager: "Ada node yang megang rute ini?"
    if let Some(target_node) = node_manager.find_node_by_route(host, path) {
        println!("➔ Rute cocok! Mengarahkan ke Node: {} ({})", target_node.name, target_node.url);

        // 4. Konstruksi URL tujuan ke backend node
        // Misal: target_node.url adalah "http://127.0.0.1:3000" dan path adalah "/transaksi"
        let upstream_url = format!("{}{}", target_node.url, path);

        // 5. Teruskan request menggunakan reqwest
        // Catatan: Untuk simplisitas versi beta, kita asumsikan GET. 
        // Anda bisa meng-extend ini untuk mem-forward method, header, dan body secara utuh.
        match client.get(&upstream_url).send().await {
            Ok(upstream_res) => {
                let status = upstream_res.status();
                let body_bytes = upstream_res.bytes().await.unwrap_or_default();
                
                let mut response = Response::new(Full::new(body_bytes));
                *response.status_mut() = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK);
                
                Ok(response)
            }
            Err(_) => {
                let mut res = Response::new(Full::new(Bytes::from("Bad Gateway: Upstream Node Unreachable")));
                *res.status_mut() = StatusCode::BAD_GATEWAY;
                Ok(res)
            }
        }
    } else {
        // Jika tidak ada domain & path yang cocok di nodes.json
        println!("❌ Tidak ada node yang cocok untuk Rute: {}/{}", host, path);
        let mut res = Response::new(Full::new(Bytes::from("404 Not Found: No node registered for this route")));
        *res.status_mut() = StatusCode::NOT_FOUND;
        Ok(res)
    }
}