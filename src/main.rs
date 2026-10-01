mod cgi;
mod config;
mod error;
mod router;
mod server;

pub mod utils {
    pub mod cookie;
    pub mod session;
}

use server::Server;

fn main() {
    match config::Config::load("config.yaml") {
        Ok(cfg) => {
            println!("Configuration loaded successfully!");
            let mut server = Server::new(cfg);
            if let Err(e) = server.run() {
                eprintln!("Server error: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to load configuration: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::error::get_error_response;
    use crate::router::Router;

    #[test]
    fn test_config_loading() {
        // Test that config loads successfully from the actual config.yaml file
        let config = Config::load("config.yaml").expect("Failed to load config.yaml");
        assert!(!config.server.host.is_empty());
        assert!(!config.server.ports.is_empty());
        assert!(config.server.client_max_body_size > 0);
    }

    #[test]
    fn test_route_matching() {
        // Load actual config to test router matching
        let config = Config::load("config.yaml").expect("Failed to load config.yaml");
        let router = Router::new(config);

        // Verify correct route match (assuming / exists in your config.yaml)
        let matched = router.match_route("/");
        assert!(matched.is_some());

        // Verify fallback route behavior for unmatched paths
        let fallback = router.match_route("/nonexistent");
        assert!(fallback.is_some());
        assert_eq!(fallback.unwrap().path, "/");
    }

    #[test]
    fn test_status_code_generation() {
        // Verify error response string formatting
        let response_404 = get_error_response(404, "Not Found");
        let response_str = String::from_utf8_lossy(response_404.as_bytes());

        assert!(response_str.contains("HTTP/1.1 404 Not Found"));
        assert!(response_str.contains("Content-Length:"));

        let response_500 = get_error_response(500, "Internal Server Error");
        let response_500_str = String::from_utf8_lossy(response_500.as_bytes());
        assert!(response_500_str.contains("HTTP/1.1 500 Internal Server Error"));
    }
}