use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortType {
    HttpServer,
    Database,
    MessageQueue,
    Cache,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPort {
    pub port: u16,
    pub port_type: PortType,
    pub context: String, // e.g., "app.listen(3000)", "PORT=8080", "postgresql://localhost:5432"
    pub file_path: String,
    pub line_number: Option<usize>,
    pub framework: Option<String>, // e.g., "express", "flask", "actix"
    pub environment: Option<String>, // e.g., "development", "production"
    pub is_config: bool, // true if from config file, false if from code
}

pub struct PortDetector;

impl PortDetector {
    pub fn new() -> Self {
        PortDetector
    }

    /// Detect ports in a repository
    pub fn detect_ports(&self, repo_path: &Path) -> Result<Vec<DetectedPort>> {
        let mut ports = Vec::new();

        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Skip hidden files and common ignore patterns
            let path_str = path.to_string_lossy().to_lowercase();
            if self.should_skip_path(&path_str, &file_name) {
                continue;
            }

            // Determine language/framework from extension
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());

            if let Ok(content) = std::fs::read_to_string(path) {
                // Detect ports in code files
                if let Some(lang) = &ext {
                    match lang.as_str() {
                        "js" | "jsx" | "ts" | "tsx" => {
                            ports.extend(self.detect_ports_js(&content, path)?);
                        }
                        "py" => {
                            ports.extend(self.detect_ports_python(&content, path)?);
                        }
                        "rs" => {
                            ports.extend(self.detect_ports_rust(&content, path)?);
                        }
                        "go" => {
                            ports.extend(self.detect_ports_go(&content, path)?);
                        }
                        "java" => {
                            ports.extend(self.detect_ports_java(&content, path)?);
                        }
                        _ => {}
                    }
                }

                // Detect ports in config files
                if file_name.ends_with(".env") || 
                   file_name.ends_with(".config") ||
                   file_name.contains("config") ||
                   file_name.ends_with(".json") ||
                   file_name.ends_with(".yaml") ||
                   file_name.ends_with(".yml") ||
                   file_name.ends_with(".toml") {
                    ports.extend(self.detect_ports_config(&content, path, &file_name)?);
                }
            }
        }

        // Deduplicate ports (same port, file, line)
        let mut deduplicated = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for port in ports {
            let key = (port.port, port.file_path.clone(), port.line_number);
            if !seen.contains(&key) {
                seen.insert(key);
                deduplicated.push(port);
            }
        }

        Ok(deduplicated)
    }

    fn detect_ports_js(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedPort>> {
        let mut ports = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Express: app.listen(3000), server.listen(8080)
        let express_pattern = Regex::new(r"(?:app|server|express)\.listen\s*\(\s*(\d{4,5})\s*[,)]")?;
        // Fastify: fastify.listen({ port: 3000 })
        let fastify_pattern = Regex::new(r"\.listen\s*\(\s*\{\s*port\s*:\s*(\d{4,5})\s*")?;
        // NestJS: await app.listen(3000)
        let nestjs_pattern = Regex::new(r"await\s+(?:app|server)\.listen\s*\(\s*(\d{4,5})\s*")?;
        // Next.js: port: 3000 in next.config.js
        let nextjs_pattern = Regex::new(r"port\s*:\s*(\d{4,5})")?;
        // Environment variable: process.env.PORT || 3000
        let env_port_pattern = Regex::new(r"process\.env\.PORT\s*\|\|\s*(\d{4,5})")?;

        for (line_num, line) in lines.iter().enumerate() {
            for pattern in &[&express_pattern, &fastify_pattern, &nestjs_pattern, &nextjs_pattern, &env_port_pattern] {
                if let Some(cap) = pattern.captures(line) {
                    if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                        if self.is_valid_port(port_num) {
                            let framework = if express_pattern.is_match(line) {
                                Some("express".to_string())
                            } else if fastify_pattern.is_match(line) {
                                Some("fastify".to_string())
                            } else if nestjs_pattern.is_match(line) {
                                Some("nestjs".to_string())
                            } else if nextjs_pattern.is_match(line) {
                                Some("nextjs".to_string())
                            } else {
                                None
                            };

                            ports.push(DetectedPort {
                                port: port_num,
                                port_type: PortType::HttpServer,
                                context: line.trim().to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework,
                                environment: None,
                                is_config: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(ports)
    }

    fn detect_ports_python(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedPort>> {
        let mut ports = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Flask: app.run(port=5000)
        let flask_pattern = Regex::new(r"app\.run\s*\([^)]*port\s*=\s*(\d{4,5})")?;
        // FastAPI: uvicorn.run(app, port=8000)
        let fastapi_pattern = Regex::new(r"uvicorn\.run\s*\([^)]*port\s*=\s*(\d{4,5})")?;
        // Django: runserver 8000
        let django_pattern = Regex::new(r"runserver\s+(\d{4,5})")?;
        // Generic: .run(port=PORT)
        let run_pattern = Regex::new(r"\.run\s*\([^)]*port\s*=\s*(\d{4,5})")?;
        // Socket: socket.bind(('localhost', 8080))
        let socket_pattern = Regex::new(r"\.bind\s*\([^)]*,\s*(\d{4,5})\s*\)")?;

        for (line_num, line) in lines.iter().enumerate() {
            for pattern in &[&flask_pattern, &fastapi_pattern, &django_pattern, &run_pattern, &socket_pattern] {
                if let Some(cap) = pattern.captures(line) {
                    if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                        if self.is_valid_port(port_num) {
                            let framework = if flask_pattern.is_match(line) {
                                Some("flask".to_string())
                            } else if fastapi_pattern.is_match(line) {
                                Some("fastapi".to_string())
                            } else if django_pattern.is_match(line) {
                                Some("django".to_string())
                            } else {
                                None
                            };

                            ports.push(DetectedPort {
                                port: port_num,
                                port_type: PortType::HttpServer,
                                context: line.trim().to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework,
                                environment: None,
                                is_config: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(ports)
    }

    fn detect_ports_rust(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedPort>> {
        let mut ports = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Actix: HttpServer::new().bind("127.0.0.1:8080")
        let actix_bind_pattern = Regex::new(r#"\.bind\s*\(\s*"[^"]*:(\d{4,5})""#)?;
        // Actix: .bind(("127.0.0.1", 8080))
        let actix_tuple_pattern = Regex::new(r#"\.bind\s*\(\s*\(\s*"[^"]*"\s*,\s*(\d{4,5})\s*\)"#)?;
        // Rocket: rocket::build().mount("/", routes)
        // Note: Rocket uses config file, but we can detect port in code
        let rocket_pattern = Regex::new(r"rocket::build\s*\(\)")?;
        // Axum: axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        let axum_pattern = Regex::new(r"\.bind\s*\([^)]*:(\d{4,5})")?;

        for (line_num, line) in lines.iter().enumerate() {
            for pattern in &[&actix_bind_pattern, &actix_tuple_pattern, &axum_pattern] {
                if let Some(cap) = pattern.captures(line) {
                    if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                        if self.is_valid_port(port_num) {
                            let framework = if actix_bind_pattern.is_match(line) || actix_tuple_pattern.is_match(line) {
                                Some("actix".to_string())
                            } else if rocket_pattern.is_match(line) {
                                Some("rocket".to_string())
                            } else if axum_pattern.is_match(line) {
                                Some("axum".to_string())
                            } else {
                                None
                            };

                            ports.push(DetectedPort {
                                port: port_num,
                                port_type: PortType::HttpServer,
                                context: line.trim().to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework,
                                environment: None,
                                is_config: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(ports)
    }

    fn detect_ports_go(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedPort>> {
        let mut ports = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Gin: router.Run(":8080")
        let gin_pattern = Regex::new(r#"\.Run\s*\(\s*":(\d{4,5})""#)?;
        // Echo: e.Start(":8080")
        let echo_pattern = Regex::new(r#"\.Start\s*\(\s*":(\d{4,5})""#)?;
        // net/http: http.ListenAndServe(":8080", nil)
        let http_pattern = Regex::new(r#"http\.ListenAndServe\s*\(\s*":(\d{4,5})""#)?;
        // net/http: Listen("tcp", ":8080")
        let listen_pattern = Regex::new(r#"Listen\s*\(\s*"tcp"\s*,\s*":(\d{4,5})""#)?;

        for (line_num, line) in lines.iter().enumerate() {
            for pattern in &[&gin_pattern, &echo_pattern, &http_pattern, &listen_pattern] {
                if let Some(cap) = pattern.captures(line) {
                    if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                        if self.is_valid_port(port_num) {
                            let framework = if gin_pattern.is_match(line) {
                                Some("gin".to_string())
                            } else if echo_pattern.is_match(line) {
                                Some("echo".to_string())
                            } else {
                                None
                            };

                            ports.push(DetectedPort {
                                port: port_num,
                                port_type: PortType::HttpServer,
                                context: line.trim().to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework,
                                environment: None,
                                is_config: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(ports)
    }

    fn detect_ports_java(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedPort>> {
        let mut ports = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Spring Boot: server.port=8080
        let spring_pattern = Regex::new(r"server\.port\s*=\s*(\d{4,5})")?;
        // Spring Boot: @Value("${server.port:8080}")
        let spring_value_pattern = Regex::new(r"server\.port[:\s]*(\d{4,5})")?;
        // Jetty: Server server = new Server(8080)
        let jetty_pattern = Regex::new(r"new\s+Server\s*\(\s*(\d{4,5})\s*\)")?;
        // Tomcat: connector.setPort(8080)
        let tomcat_pattern = Regex::new(r"\.setPort\s*\(\s*(\d{4,5})\s*\)")?;

        for (line_num, line) in lines.iter().enumerate() {
            for pattern in &[&spring_pattern, &spring_value_pattern, &jetty_pattern, &tomcat_pattern] {
                if let Some(cap) = pattern.captures(line) {
                    if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                        if self.is_valid_port(port_num) {
                            let framework = if spring_pattern.is_match(line) || spring_value_pattern.is_match(line) {
                                Some("spring".to_string())
                            } else if jetty_pattern.is_match(line) {
                                Some("jetty".to_string())
                            } else if tomcat_pattern.is_match(line) {
                                Some("tomcat".to_string())
                            } else {
                                None
                            };

                            ports.push(DetectedPort {
                                port: port_num,
                                port_type: PortType::HttpServer,
                                context: line.trim().to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework,
                                environment: None,
                                is_config: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(ports)
    }

    fn detect_ports_config(&self, content: &str, file_path: &Path, file_name: &str) -> Result<Vec<DetectedPort>> {
        let mut ports = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Environment variables: PORT=8080, SERVER_PORT=3000
        let env_port_pattern = Regex::new(r"(?i)^\s*(?:PORT|SERVER_PORT|APP_PORT|HTTP_PORT)\s*=\s*(\d{4,5})")?;
        // Database URLs: postgresql://localhost:5432, mysql://localhost:3306
        let db_url_pattern = Regex::new(r"://[^:]+:(\d{4,5})")?;
        // JSON/YAML: "port": 8080, port: 3000
        let json_port_pattern = Regex::new(r#"(?i)"port"\s*:\s*(\d{4,5})"#)?;
        let yaml_port_pattern = Regex::new(r"(?i)^\s*port\s*:\s*(\d{4,5})")?;
        // Docker Compose: "8080:8080" or ports: - "8080:8080"
        let docker_port_pattern = Regex::new(r#""(\d{4,5}):\d{4,5}""#)?;

        for (line_num, line) in lines.iter().enumerate() {
            // Environment variables
            if let Some(cap) = env_port_pattern.captures(line) {
                if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                    if self.is_valid_port(port_num) {
                        ports.push(DetectedPort {
                            port: port_num,
                            port_type: PortType::HttpServer,
                            context: line.trim().to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: None,
                            environment: self.extract_environment(file_name),
                            is_config: true,
                        });
                    }
                }
            }

            // Database URLs
            if let Some(cap) = db_url_pattern.captures(line) {
                if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                    if self.is_valid_port(port_num) {
                        let port_type = if line.contains("postgres") || line.contains("postgresql") {
                            PortType::Database
                        } else if line.contains("mysql") {
                            PortType::Database
                        } else if line.contains("mongodb") {
                            PortType::Database
                        } else if line.contains("redis") {
                            PortType::Cache
                        } else {
                            PortType::Other
                        };

                        ports.push(DetectedPort {
                            port: port_num,
                            port_type,
                            context: line.trim().to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: None,
                            environment: self.extract_environment(file_name),
                            is_config: true,
                        });
                    }
                }
            }

            // JSON/YAML ports
            for pattern in &[&json_port_pattern, &yaml_port_pattern] {
                if let Some(cap) = pattern.captures(line) {
                    if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                        if self.is_valid_port(port_num) {
                            ports.push(DetectedPort {
                                port: port_num,
                                port_type: PortType::HttpServer,
                                context: line.trim().to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework: None,
                                environment: self.extract_environment(file_name),
                                is_config: true,
                            });
                        }
                    }
                }
            }

            // Docker ports
            if let Some(cap) = docker_port_pattern.captures(line) {
                if let Ok(port_num) = cap.get(1).unwrap().as_str().parse::<u16>() {
                    if self.is_valid_port(port_num) {
                        ports.push(DetectedPort {
                            port: port_num,
                            port_type: PortType::HttpServer,
                            context: line.trim().to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: None,
                            environment: None,
                            is_config: true,
                        });
                    }
                }
            }
        }

        Ok(ports)
    }

    fn is_valid_port(&self, port: u16) -> bool {
        // Exclude system ports (0-1023) and common ephemeral ports (49152-65535)
        // Focus on application ports (1024-49151)
        port >= 1024 && port <= 49151
    }

    fn extract_environment(&self, file_name: &str) -> Option<String> {
        let file_lower = file_name.to_lowercase();
        if file_lower.contains(".prod") || file_lower.contains("production") {
            Some("production".to_string())
        } else if file_lower.contains(".dev") || file_lower.contains("development") {
            Some("development".to_string())
        } else if file_lower.contains(".test") || file_lower.contains("testing") {
            Some("testing".to_string())
        } else if file_lower.contains(".staging") || file_lower.contains("staging") {
            Some("staging".to_string())
        } else {
            None
        }
    }

    fn should_skip_path(&self, path_str: &str, file_name: &str) -> bool {
        // Allow config files even if they start with .
        let is_config_file = file_name.ends_with(".env") || 
                             file_name.ends_with(".config") ||
                             file_name.contains("config") ||
                             file_name.ends_with(".json") ||
                             file_name.ends_with(".yaml") ||
                             file_name.ends_with(".yml") ||
                             file_name.ends_with(".toml") ||
                             file_name.ends_with(".properties");
        
        // Skip hidden files EXCEPT config files
        if file_name.starts_with('.') && !is_config_file {
            return true;
        }

        path_str.contains("node_modules") ||
        path_str.contains("target") ||
        path_str.contains(".git") ||
        path_str.contains("venv/") ||
        path_str.contains("venv\\") ||
        path_str.contains("/venv/") ||
        path_str.contains("\\venv\\") ||
        path_str.contains("site-packages") ||
        path_str.contains(".venv/") ||
        path_str.contains(".venv\\")
    }
}

