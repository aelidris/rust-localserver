use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub host: String,
    pub ports: Vec<u16>,
    pub client_max_body_size: usize,
    pub error_pages: HashMap<u16, String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RouteConfig {
    pub path: String,
    pub root: Option<String>,
    pub methods: Option<Vec<String>>,
    pub default_file: Option<String>,
    pub directory_listing: Option<bool>,
    pub cgi_extensions: Option<Vec<String>>,
    pub redirect: Option<String>,
}

impl Config {
    // Function to load and parse the config file using standard library only
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Default values initialization
        let mut host = String::from("127.0.0.1");
        let mut ports = vec![8080];
        let mut client_max_body_size = 1048576;
        let mut error_pages = HashMap::new();
        let mut routes = Vec::new();

        // Simple line-by-line parsing for standard keys
        let mut current_route: Option<RouteConfig> = None;
        let mut in_routes = false;
        let mut in_error_pages = false;

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Detect sections
            if trimmed.starts_with("routes:") {
                if let Some(r) = current_route.take() {
                    routes.push(r);
                }
                in_routes = true;
                in_error_pages = false;
                continue;
            } else if trimmed.starts_with("error_pages:") {
                in_error_pages = true;
                in_routes = false;
                continue;
            }

            if in_error_pages && trimmed.contains(':') {
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    if let Ok(code) = parts[0].trim().parse::<u16>() {
                        let page = parts[1].trim().trim_matches('"').to_string();
                        error_pages.insert(code, page);
                    }
                }
            } else if in_routes {
                if trimmed.starts_with("- path:") || trimmed.starts_with("path:") {
                    if let Some(r) = current_route.take() {
                        routes.push(r);
                    }
                    let path_val = trimmed.split(':').nth(1).unwrap_or("").trim().trim_matches('"').to_string();
                    current_route = Some(RouteConfig {
                        path: path_val,
                        root: None,
                        methods: None,
                        default_file: None,
                        directory_listing: None,
                        cgi_extensions: None,
                        redirect: None,
                    });
                } else if let Some(ref mut r) = current_route {
                    if trimmed.starts_with("root:") {
                        r.root = Some(trimmed.split(':').nth(1).unwrap_or("").trim().trim_matches('"').to_string());
                    } else if trimmed.starts_with("directory_listing:") {
                        let val = trimmed.split(':').nth(1).unwrap_or("").trim();
                        r.directory_listing = Some(val == "true");
                    } else if trimmed.starts_with("default_file:") {
                        r.default_file = Some(trimmed.split(':').nth(1).unwrap_or("").trim().trim_matches('"').to_string());
                    } else if trimmed.starts_with("redirect:") {
                        r.redirect = Some(trimmed.split(':').nth(1).unwrap_or("").trim().trim_matches('"').to_string());
                    } else if trimmed.starts_with("methods:") {
                        let cleaned = trimmed.split(':').nth(1).unwrap_or("").trim();
                        let cleaned = cleaned.trim_matches(|c| c == '[' || c == ']');
                        let methods: Vec<String> = cleaned
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        r.methods = Some(methods);
                    } else if trimmed.starts_with("cgi_extensions:") {
                        let cleaned = trimmed.split(':').nth(1).unwrap_or("").trim();
                        let cleaned = cleaned.trim_matches(|c| c == '[' || c == ']');
                        let exts: Vec<String> = cleaned
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        r.cgi_extensions = Some(exts);
                    }
                }
            } else {
                // Server configuration parsing
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim();
                    match key {
                        "host" => host = val.trim_matches('"').to_string(),
                        "client_max_body_size" => {
                            if let Ok(sz) = val.parse::<usize>() {
                                client_max_body_size = sz;
                            }
                        }
                        "ports" => {
                            // Basic parsing for ports like [8080, 8081]
                            let cleaned = val.trim_matches(|c| c == '[' || c == ']');
                            ports = cleaned
                                .split(',')
                                .filter_map(|s| s.trim().parse::<u16>().ok())
                                .collect();
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(r) = current_route.take() {
            routes.push(r);
        }

        Ok(Config {
            server: ServerConfig {
                host,
                ports,
                client_max_body_size,
                error_pages,
            },
            routes,
        })
    }
}