/// HTTP request handling - routing dan forwarding
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode};
use std::sync::Arc;
use std::time::Instant;

use crate::mcore::config::load_config::CONFIG;
use crate::mcore::melisad::proxy::forwarder::forward_request_with_retry;
use crate::mcore::melisad::proxy::loadbalancer::LoadBalancer;
use crate::mcore::melisad::proxy::metrics::ProxyMetrics;
use crate::mcore::melisad::services::node::NODE_MANAGER;
use crate::mcore::mlog::LOGGER;

pub async fn handle_proxy_request(
    req: Request<Incoming>,
    client: Arc<reqwest::Client>,
    load_balancer: Arc<LoadBalancer>,
    metrics: Arc<ProxyMetrics>,
    peer_addr: String,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let start = Instant::now();
    let request_id = format!("REQ-{}", uuid::Uuid::new_v4().simple());
    let (parts, body) = req.into_parts();

    // Extract metadata
    let method = parts.method.clone();
    let uri = parts.uri.clone();
    let headers = parts.headers;
    let host = headers
        .get(hyper::header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let path = uri.path().to_string();
    let path_and_query = uri
        .path_and_query()
        .map(|path_and_query| path_and_query.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());
    let body_bytes = body.collect().await?.to_bytes();

    // Try to select node via load balancer
    if let Some(target_node) = load_balancer.select_node(&host, &path, &NODE_MANAGER) {
        let upstream_node_name = format!("{} ({})", target_node.name, target_node.url);
        let _ = LOGGER.log_debug(&format!(
            "[{}] Route matched -> {}",
            request_id, upstream_node_name
        ));

        // Construct upstream URL
        let upstream_url = format!(
            "{}{}",
            target_node.url.trim_end_matches('/'),
            path_and_query
        );

        // Forward request dengan retry
        let response = forward_request_with_retry(
            &client,
            &method,
            &upstream_url,
            &headers,
            body_bytes,
            &request_id,
            CONFIG.proxy.max_retries,
            CONFIG.proxy.retry_backoff_ms,
        )
        .await;

        let duration_ms = start.elapsed().as_millis();

        match response {
            Ok(forwarded) => {
                let bytes_len = forwarded.body.len();
                metrics.record_request(bytes_len, false);

                let _ = LOGGER.log_access(
                    &peer_addr,
                    method.as_str(),
                    &path_and_query,
                    forwarded.status.as_u16(),
                    bytes_len,
                    duration_ms,
                    Some(&target_node.name),
                );

                let mut proxy_response = Response::new(Full::new(forwarded.body));
                *proxy_response.status_mut() = forwarded.status;
                proxy_response.headers_mut().extend(forwarded.headers);
                Ok(proxy_response)
            }
            Err(err) => {
                metrics.record_request(0, true);
                let _ = LOGGER.log_error(&format!(
                    "[{}] Failed to reach upstream ({}): {:?}",
                    request_id, upstream_url, err
                ));

                let _ = LOGGER.log_access(
                    &peer_addr,
                    method.as_str(),
                    &path_and_query,
                    502,
                    0,
                    duration_ms,
                    Some("error"),
                );

                let error_body = format!(
                    "{{\"error\": \"Bad Gateway\", \"request_id\": \"{}\"}}",
                    request_id
                );
                let mut error_response = Response::new(Full::new(Bytes::from(error_body)));
                *error_response.status_mut() = StatusCode::BAD_GATEWAY;
                Ok(error_response)
            }
        }
    } else {
        metrics.record_request(0, true);

        let _ = LOGGER.log_error(&format!(
            "[{}] No route found for {}{}",
            request_id, host, path
        ));

        let duration_ms = start.elapsed().as_millis();
        let _ = LOGGER.log_access(
            &peer_addr,
            method.as_str(),
            &path_and_query,
            404,
            0,
            duration_ms,
            None,
        );

        let error_body = format!(
            "{{\"error\": \"Not Found\", \"path\": \"{}\", \"request_id\": \"{}\"}}",
            path, request_id
        );
        let mut not_found = Response::new(Full::new(Bytes::from(error_body)));
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        Ok(not_found)
    }
}
