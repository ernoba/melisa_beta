use crate::mcore::melisad::services::node::{NodeManager, NodeProcess, NodeStatus};

/// Fungsi untuk mencari node yang cocok berdasarkan domain dan path request.
impl NodeManager {
    pub fn find_node_by_route(&self, domain: &str, path: &str) -> Option<NodeProcess> {
        self.find_matching_nodes_by_route(domain, path)
            .into_iter()
            .next()
    }

    pub fn find_matching_nodes_by_route(&self, domain: &str, path: &str) -> Vec<NodeProcess> {
        let processes_lock = self.processes.read().unwrap();
        let request_host = normalize_host(domain);
        let request_path = normalize_request_path(path);

        let mut matching_nodes: Vec<NodeProcess> = processes_lock
            .values()
            .filter(|node| node.status == NodeStatus::Active)
            .filter(|node| {
                domain_matches(&node.domain, &request_host)
                    && route_matches(&node.route_path, &request_path)
            })
            .cloned()
            .collect();

        if let Some(max_specificity) = matching_nodes
            .iter()
            .map(|node| route_specificity(&node.route_path))
            .max()
        {
            matching_nodes.retain(|node| route_specificity(&node.route_path) == max_specificity);
        }

        matching_nodes.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.url.cmp(&b.url)));
        matching_nodes
    }
}

pub(crate) fn route_specificity(route_path: &str) -> usize {
    normalize_route_path(route_path).len()
}

fn domain_matches(node_domain: &str, request_host: &str) -> bool {
    normalize_host(node_domain) == normalize_host(request_host)
}

fn route_matches(route_path: &str, request_path: &str) -> bool {
    let route_path = normalize_route_path(route_path);

    route_path == "/"
        || request_path == route_path
        || request_path
            .strip_prefix(&route_path)
            .is_some_and(|remaining| remaining.starts_with('/'))
}

fn normalize_host(host: &str) -> String {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();

    if let Some(without_brackets) = host.strip_prefix('[') {
        if let Some((ipv6, _)) = without_brackets.split_once(']') {
            return ipv6.to_string();
        }
    }

    host.split_once(':')
        .map(|(host_without_port, _)| host_without_port.to_string())
        .unwrap_or(host)
}

fn normalize_request_path(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() || !path.starts_with('/') {
        "/".to_string()
    } else {
        path.to_string()
    }
}

fn normalize_route_path(route_path: &str) -> String {
    let route_path = route_path.trim();

    if route_path.is_empty() || route_path == "/" {
        return "/".to_string();
    }

    let route_path = route_path.trim_end_matches('/');
    if route_path.starts_with('/') {
        route_path.to_string()
    } else {
        format!("/{}", route_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_match_respects_path_boundaries() {
        assert!(route_matches("/api", "/api"));
        assert!(route_matches("/api", "/api/users"));
        assert!(!route_matches("/api", "/api-v2"));
        assert!(!route_matches("/api", "/api2"));
    }

    #[test]
    fn host_match_ignores_port_and_case() {
        assert!(domain_matches("Example.COM", "example.com:8080"));
        assert!(!domain_matches("api.example.com", "example.com:8080"));
    }
}
