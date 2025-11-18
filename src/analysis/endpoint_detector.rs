use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
    Any, // Catch-all routes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedEndpoint {
    pub path: String, // e.g., "/api/users", "/users/:id"
    pub method: HttpMethod,
    pub handler: Option<String>, // Function/method name that handles this endpoint
    pub file_path: String,
    pub line_number: Option<usize>,
    pub framework: Option<String>, // e.g., "express", "flask", "actix"
    pub middleware: Vec<String>, // Middleware/guards applied to this endpoint
    pub parameters: Vec<String>, // Route parameters like :id, {id}
}

pub struct EndpointDetector;

impl EndpointDetector {
    pub fn new() -> Self {
        EndpointDetector
    }

    /// Detect API endpoints in a repository
    pub fn detect_endpoints(&self, repo_path: &Path) -> Result<Vec<DetectedEndpoint>> {
        let mut endpoints = Vec::new();

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
                // Detect endpoints in code files
                if let Some(lang) = &ext {
                    match lang.as_str() {
                        "js" | "jsx" | "ts" | "tsx" => {
                            endpoints.extend(self.detect_endpoints_js(&content, path)?);
                        }
                        "py" => {
                            endpoints.extend(self.detect_endpoints_python(&content, path)?);
                        }
                        "rs" => {
                            endpoints.extend(self.detect_endpoints_rust(&content, path)?);
                        }
                        "go" => {
                            endpoints.extend(self.detect_endpoints_go(&content, path)?);
                        }
                        "java" => {
                            endpoints.extend(self.detect_endpoints_java(&content, path)?);
                        }
                        _ => {}
                    }
                }

                // Detect endpoints in config files (OpenAPI, API Gateway, etc.)
                if file_name.ends_with(".json") || 
                   file_name.ends_with(".yaml") || 
                   file_name.ends_with(".yml") ||
                   file_name.contains("openapi") ||
                   file_name.contains("swagger") ||
                   file_name.contains("api-gateway") {
                    endpoints.extend(self.detect_endpoints_config(&content, path, &file_name)?);
                }
            }
        }

        Ok(endpoints)
    }

    fn detect_endpoints_js(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedEndpoint>> {
        let mut endpoints = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Express: app.get('/api/users', handler)
        let express_get = Regex::new(r#"(?:app|router|express)\.(get|post|put|delete|patch|all)\s*\(\s*['"]([^'"]+)['"]"#)?;
        // Express Router: router.get('/users', handler)
        let router_pattern = Regex::new(r#"router\.(get|post|put|delete|patch|all)\s*\(\s*['"]([^'"]+)['"]"#)?;
        // Fastify: fastify.get('/api/users', handler)
        let fastify_pattern = Regex::new(r#"fastify\.(get|post|put|delete|patch)\s*\(\s*['"]([^'"]+)['"]"#)?;
        // NestJS: @Get('/users'), @Post('/users')
        let nestjs_decorator = Regex::new(r#"@(Get|Post|Put|Delete|Patch|All)\s*\(\s*['"]([^'"]+)['"]"#)?;
        // Next.js API routes: export default function handler(req, res) in /api/*.js
        let nextjs_api = Regex::new(r"export\s+(?:default\s+)?(?:async\s+)?function\s+(\w+)")?;

        for (line_num, line) in lines.iter().enumerate() {
            // Express routes
            for cap in express_get.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler: self.extract_handler_name(line, line_num, &lines),
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("express".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }

            // Router routes
            for cap in router_pattern.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler: self.extract_handler_name(line, line_num, &lines),
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("express".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }

            // Fastify routes
            for cap in fastify_pattern.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler: self.extract_handler_name(line, line_num, &lines),
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("fastify".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }

            // NestJS decorators
            for cap in nestjs_decorator.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        // Find the method name on the next few lines
                        let handler = self.find_nestjs_handler(line_num, &lines);
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler,
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("nestjs".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }

            // Next.js API routes
            if file_path.to_string_lossy().contains("/api/") {
                if let Some(cap) = nextjs_api.captures(line) {
                    if let Some(handler_name) = cap.get(1) {
                        let api_path = self.extract_nextjs_path(file_path);
                        endpoints.push(DetectedEndpoint {
                            path: api_path,
                            method: HttpMethod::Any,
                            handler: Some(handler_name.as_str().to_string()),
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("nextjs".to_string()),
                            middleware: Vec::new(),
                            parameters: Vec::new(),
                        });
                    }
                }
            }
        }

        Ok(endpoints)
    }

    fn detect_endpoints_python(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedEndpoint>> {
        let mut endpoints = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Flask: @app.route('/api/users', methods=['GET'])
        let flask_route = Regex::new(r#"@(?:app|bp|blueprint)\.route\s*\(\s*['"]([^'"]+)['"]"#)?;
        // Flask methods: methods=['GET', 'POST']
        let flask_methods = Regex::new(r"methods\s*=\s*\[([^\]]+)\]")?;
        // FastAPI: @app.get('/users'), @app.post('/users')
        let fastapi_route = Regex::new(r#"@(?:app|router)\.(get|post|put|delete|patch)\s*\(\s*['"]([^'"]+)['"]"#)?;
        // Django: path('users/', views.users_list)
        let django_path = Regex::new(r#"path\s*\(\s*['"]([^'"]+)['"]"#)?;
        // Django URL: url(r'^users/$', views.users_list)
        let django_url = Regex::new(r#"url\s*\(\s*r['"]([^'"]+)['"]"#)?;

        for (line_num, line) in lines.iter().enumerate() {
            // Flask routes
            if let Some(cap) = flask_route.captures(line) {
                if let Some(path_str) = cap.get(1) {
                    let method = if let Some(methods_cap) = flask_methods.captures(line) {
                        self.parse_methods_list(methods_cap.get(1).unwrap().as_str())
                    } else {
                        HttpMethod::Get // Default to GET
                    };

                    let handler = self.extract_python_handler(line_num, &lines);
                    endpoints.push(DetectedEndpoint {
                        path: path_str.as_str().to_string(),
                        method,
                        handler,
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        framework: Some("flask".to_string()),
                        middleware: Vec::new(),
                        parameters: self.extract_route_params(path_str.as_str()),
                    });
                }
            }

            // FastAPI routes
            for cap in fastapi_route.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        let handler = self.extract_python_handler(line_num, &lines);
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler,
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("fastapi".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }

            // Django paths
            for pattern in &[&django_path, &django_url] {
                for cap in pattern.captures_iter(line) {
                    if let Some(path_str) = cap.get(1) {
                        let handler = self.extract_django_handler(line);
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method: HttpMethod::Any, // Django doesn't specify method in URL config
                            handler,
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("django".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }
        }

        Ok(endpoints)
    }

    fn detect_endpoints_rust(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedEndpoint>> {
        let mut endpoints = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Actix: #[get("/api/users")]
        let actix_get = Regex::new(r#"#\[(get|post|put|delete|patch)\s*\(\s*"([^"]+)""#)?;
        // Actix: .route("/users", web::get().to(handler))
        let actix_route = Regex::new(r#"\.route\s*\(\s*"([^"]+)""#)?;
        // Axum: Router::new().route("/users", get(handler))
        let _axum_route = Regex::new(r#"\.route\s*\(\s*"([^"]+)""#)?;
        // Rocket: #[get("/users")]
        let rocket_get = Regex::new(r#"#\[(get|post|put|delete|patch)\s*\(\s*"([^"]+)""#)?;

        for (line_num, line) in lines.iter().enumerate() {
            // Actix attributes
            for cap in actix_get.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        let handler = self.find_rust_handler(line_num, &lines);
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler,
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("actix".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }

            // Actix route method
            if let Some(cap) = actix_route.captures(line) {
                if let Some(path_str) = cap.get(1) {
                    let method = self.extract_actix_method(line);
                    let handler = self.extract_actix_handler(line);
                    endpoints.push(DetectedEndpoint {
                        path: path_str.as_str().to_string(),
                        method,
                        handler,
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        framework: Some("actix".to_string()),
                        middleware: Vec::new(),
                        parameters: self.extract_route_params(path_str.as_str()),
                    });
                }
            }

            // Rocket attributes
            for cap in rocket_get.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        let handler = self.find_rust_handler(line_num, &lines);
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler,
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("rocket".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }
        }

        Ok(endpoints)
    }

    fn detect_endpoints_go(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedEndpoint>> {
        let mut endpoints = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Gin: router.GET("/users", handler)
        let gin_route = Regex::new(r#"router\.(GET|POST|PUT|DELETE|PATCH)\s*\(\s*"([^"]+)""#)?;
        // Echo: e.GET("/users", handler)
        let echo_route = Regex::new(r#"e\.(GET|POST|PUT|DELETE|PATCH)\s*\(\s*"([^"]+)""#)?;
        // Chi: r.Get("/users", handler)
        let chi_route = Regex::new(r#"r\.(Get|Post|Put|Delete|Patch)\s*\(\s*"([^"]+)""#)?;

        for (line_num, line) in lines.iter().enumerate() {
            for pattern in &[&gin_route, &echo_route, &chi_route] {
                for cap in pattern.captures_iter(line) {
                    if let Some(method_str) = cap.get(1) {
                        if let Some(path_str) = cap.get(2) {
                            let method = self.parse_method(method_str.as_str());
                            let handler = self.extract_go_handler(line);
                            let framework = if gin_route.is_match(line) {
                                Some("gin".to_string())
                            } else if echo_route.is_match(line) {
                                Some("echo".to_string())
                            } else if chi_route.is_match(line) {
                                Some("chi".to_string())
                            } else {
                                None
                            };

                            endpoints.push(DetectedEndpoint {
                                path: path_str.as_str().to_string(),
                                method,
                                handler,
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework,
                                middleware: Vec::new(),
                                parameters: self.extract_route_params(path_str.as_str()),
                            });
                        }
                    }
                }
            }
        }

        Ok(endpoints)
    }

    fn detect_endpoints_java(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedEndpoint>> {
        let mut endpoints = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Spring: @GetMapping("/users"), @PostMapping("/users")
        let spring_mapping = Regex::new(r#"@(Get|Post|Put|Delete|Patch)Mapping\s*\(\s*"([^"]+)""#)?;
        // Spring: @RequestMapping(value = "/users", method = RequestMethod.GET)
        let spring_request = Regex::new(r#"@RequestMapping\s*\([^)]*value\s*=\s*"([^"]+)""#)?;
        // JAX-RS: @GET @Path("/users")
        let jaxrs_path = Regex::new(r#"@Path\s*\(\s*"([^"]+)""#)?;

        for (line_num, line) in lines.iter().enumerate() {
            // Spring mappings
            for cap in spring_mapping.captures_iter(line) {
                if let Some(method_str) = cap.get(1) {
                    if let Some(path_str) = cap.get(2) {
                        let method = self.parse_method(method_str.as_str());
                        let handler = self.find_java_handler(line_num, &lines);
                        endpoints.push(DetectedEndpoint {
                            path: path_str.as_str().to_string(),
                            method,
                            handler,
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: Some(line_num + 1),
                            framework: Some("spring".to_string()),
                            middleware: Vec::new(),
                            parameters: self.extract_route_params(path_str.as_str()),
                        });
                    }
                }
            }

            // Spring RequestMapping
            if let Some(cap) = spring_request.captures(line) {
                if let Some(path_str) = cap.get(1) {
                    let method = self.extract_spring_method(line);
                    let handler = self.find_java_handler(line_num, &lines);
                    endpoints.push(DetectedEndpoint {
                        path: path_str.as_str().to_string(),
                        method,
                        handler,
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        framework: Some("spring".to_string()),
                        middleware: Vec::new(),
                        parameters: self.extract_route_params(path_str.as_str()),
                    });
                }
            }

            // JAX-RS
            if let Some(cap) = jaxrs_path.captures(line) {
                if let Some(path_str) = cap.get(1) {
                    let method = self.extract_jaxrs_method(line_num, &lines);
                    let handler = self.find_java_handler(line_num, &lines);
                    endpoints.push(DetectedEndpoint {
                        path: path_str.as_str().to_string(),
                        method,
                        handler,
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        framework: Some("jaxrs".to_string()),
                        middleware: Vec::new(),
                        parameters: self.extract_route_params(path_str.as_str()),
                    });
                }
            }
        }

        Ok(endpoints)
    }

    // Helper methods
    fn parse_method(&self, method_str: &str) -> HttpMethod {
        match method_str.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "OPTIONS" => HttpMethod::Options,
            "HEAD" => HttpMethod::Head,
            "ALL" => HttpMethod::Any,
            _ => HttpMethod::Get,
        }
    }

    fn parse_methods_list(&self, methods_str: &str) -> HttpMethod {
        // For Flask routes with multiple methods, default to Any
        if methods_str.contains(',') {
            HttpMethod::Any
        } else {
            self.parse_method(methods_str.trim().trim_matches('\'').trim_matches('"'))
        }
    }

    fn extract_route_params(&self, path: &str) -> Vec<String> {
        let mut params = Vec::new();
        // Express: /users/:id
        let express_param = Regex::new(r":(\w+)").unwrap();
        for cap in express_param.captures_iter(path) {
            if let Some(param) = cap.get(1) {
                params.push(param.as_str().to_string());
            }
        }
        // FastAPI/Flask: /users/{id}
        let curly_param = Regex::new(r"\{(\w+)\}").unwrap();
        for cap in curly_param.captures_iter(path) {
            if let Some(param) = cap.get(1) {
                params.push(param.as_str().to_string());
            }
        }
        params
    }

    fn extract_handler_name(&self, line: &str, line_num: usize, lines: &[&str]) -> Option<String> {
        // Try to extract function name from the same line or next line
        let func_pattern = Regex::new(r"(?:function|const|let|var)\s+(\w+)\s*[=(]").unwrap();
        if let Some(cap) = func_pattern.captures(line) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }
        // Check next few lines
        for i in (line_num + 1)..(line_num + 5).min(lines.len()) {
            if let Some(cap) = func_pattern.captures(lines[i]) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_python_handler(&self, line_num: usize, lines: &[&str]) -> Option<String> {
        // Look for function definition on next few lines
        let func_pattern = Regex::new(r"def\s+(\w+)\s*\(").unwrap();
        for i in (line_num + 1)..(line_num + 5).min(lines.len()) {
            if let Some(cap) = func_pattern.captures(lines[i]) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_django_handler(&self, line: &str) -> Option<String> {
        // Extract handler from views.users_list or views.UserListView.as_view()
        let handler_pattern = Regex::new(r"views\.(\w+)").unwrap();
        handler_pattern.captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn find_nestjs_handler(&self, line_num: usize, lines: &[&str]) -> Option<String> {
        // Look for method definition after decorator
        let method_pattern = Regex::new(r"(\w+)\s*\([^)]*\)\s*[:{]").unwrap();
        for i in (line_num + 1)..(line_num + 10).min(lines.len()) {
            if let Some(cap) = method_pattern.captures(lines[i]) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn find_rust_handler(&self, line_num: usize, lines: &[&str]) -> Option<String> {
        // Look for async fn or fn definition
        let fn_pattern = Regex::new(r"(?:async\s+)?fn\s+(\w+)\s*\(").unwrap();
        for i in (line_num + 1)..(line_num + 10).min(lines.len()) {
            if let Some(cap) = fn_pattern.captures(lines[i]) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_actix_method(&self, line: &str) -> HttpMethod {
        if line.contains("web::get()") {
            HttpMethod::Get
        } else if line.contains("web::post()") {
            HttpMethod::Post
        } else if line.contains("web::put()") {
            HttpMethod::Put
        } else if line.contains("web::delete()") {
            HttpMethod::Delete
        } else {
            HttpMethod::Any
        }
    }

    fn extract_actix_handler(&self, line: &str) -> Option<String> {
        let handler_pattern = Regex::new(r"\.to\((\w+)\)").unwrap();
        handler_pattern.captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_go_handler(&self, line: &str) -> Option<String> {
        // Extract handler function name
        let handler_pattern = Regex::new(r",\s*(\w+)\)").unwrap();
        handler_pattern.captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn find_java_handler(&self, line_num: usize, lines: &[&str]) -> Option<String> {
        // Look for method definition
        let method_pattern = Regex::new(r"(?:public|private|protected)?\s*(?:@\w+\s*)*\s*(\w+)\s*\([^)]*\)").unwrap();
        for i in (line_num + 1)..(line_num + 10).min(lines.len()) {
            if let Some(cap) = method_pattern.captures(lines[i]) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_spring_method(&self, line: &str) -> HttpMethod {
        if line.contains("RequestMethod.GET") || line.contains("method = GET") {
            HttpMethod::Get
        } else if line.contains("RequestMethod.POST") || line.contains("method = POST") {
            HttpMethod::Post
        } else if line.contains("RequestMethod.PUT") || line.contains("method = PUT") {
            HttpMethod::Put
        } else if line.contains("RequestMethod.DELETE") || line.contains("method = DELETE") {
            HttpMethod::Delete
        } else {
            HttpMethod::Any
        }
    }

    fn extract_jaxrs_method(&self, line_num: usize, lines: &[&str]) -> HttpMethod {
        // Look for @GET, @POST, etc. before @Path
        for i in (line_num.saturating_sub(5))..=line_num {
            if lines[i].contains("@GET") {
                return HttpMethod::Get;
            } else if lines[i].contains("@POST") {
                return HttpMethod::Post;
            } else if lines[i].contains("@PUT") {
                return HttpMethod::Put;
            } else if lines[i].contains("@DELETE") {
                return HttpMethod::Delete;
            }
        }
        HttpMethod::Any
    }

    fn extract_nextjs_path(&self, file_path: &Path) -> String {
        // Convert file path to API path
        // e.g., /api/users/index.js -> /api/users
        let path_str = file_path.to_string_lossy();
        if let Some(api_pos) = path_str.find("/api/") {
            let after_api = &path_str[api_pos + 5..];
            let without_ext = after_api.split('.').next().unwrap_or(after_api);
            format!("/api/{}", without_ext)
        } else {
            "/api".to_string()
        }
    }

    fn detect_endpoints_config(&self, content: &str, file_path: &Path, file_name: &str) -> Result<Vec<DetectedEndpoint>> {
        let mut endpoints = Vec::new();
        
        // Try to parse as JSON first (OpenAPI, API Gateway configs)
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            // OpenAPI/Swagger format
            if json.get("openapi").is_some() || json.get("swagger").is_some() {
                if let Some(paths) = json.get("paths").and_then(|p| p.as_object()) {
                    for (path, path_item) in paths {
                        if let Some(path_obj) = path_item.as_object() {
                            for (method, _) in path_obj {
                                let http_method = match method.to_uppercase().as_str() {
                                    "GET" => HttpMethod::Get,
                                    "POST" => HttpMethod::Post,
                                    "PUT" => HttpMethod::Put,
                                    "DELETE" => HttpMethod::Delete,
                                    "PATCH" => HttpMethod::Patch,
                                    "OPTIONS" => HttpMethod::Options,
                                    "HEAD" => HttpMethod::Head,
                                    _ => HttpMethod::Any,
                                };
                                
                                endpoints.push(DetectedEndpoint {
                                    path: path.clone(),
                                    method: http_method,
                                    handler: None,
                                    file_path: file_path.to_string_lossy().to_string(),
                                    line_number: None,
                                    framework: Some("openapi".to_string()),
                                    middleware: Vec::new(),
                                    parameters: self.extract_route_params(path),
                                });
                            }
                        }
                    }
                }
            }
            
            // AWS API Gateway format
            if json.get("paths").is_some() && json.get("x-amazon-apigateway-integration").is_some() {
                if let Some(paths) = json.get("paths").and_then(|p| p.as_object()) {
                    for (path, path_item) in paths {
                        if let Some(path_obj) = path_item.as_object() {
                            for (method, _) in path_obj {
                                let http_method = match method.to_uppercase().as_str() {
                                    "GET" => HttpMethod::Get,
                                    "POST" => HttpMethod::Post,
                                    "PUT" => HttpMethod::Put,
                                    "DELETE" => HttpMethod::Delete,
                                    "PATCH" => HttpMethod::Patch,
                                    _ => HttpMethod::Any,
                                };
                                
                                endpoints.push(DetectedEndpoint {
                                    path: path.clone(),
                                    method: http_method,
                                    handler: None,
                                    file_path: file_path.to_string_lossy().to_string(),
                                    line_number: None,
                                    framework: Some("api-gateway".to_string()),
                                    middleware: Vec::new(),
                                    parameters: self.extract_route_params(path),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Try YAML parsing (OpenAPI YAML format) - simple regex-based detection
        if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
            let yaml_path_pattern = Regex::new(r#"(?m)^\s+/([^\s:]+):\s*$"#)?;
            let yaml_method_pattern = Regex::new(r#"(?m)^\s+(get|post|put|delete|patch):\s*$"#)?;
            
            let lines: Vec<&str> = content.lines().collect();
            let mut current_path = String::new();
            
            for (line_num, line) in lines.iter().enumerate() {
                if let Some(cap) = yaml_path_pattern.captures(line) {
                    if let Some(path_match) = cap.get(1) {
                        current_path = format!("/{}", path_match.as_str());
                    }
                }
                
                if let Some(cap) = yaml_method_pattern.captures(line) {
                    if let Some(method_match) = cap.get(1) {
                        if !current_path.is_empty() {
                            let method = self.parse_method(method_match.as_str());
                            endpoints.push(DetectedEndpoint {
                                path: current_path.clone(),
                                method,
                                handler: None,
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: Some(line_num + 1),
                                framework: Some("openapi".to_string()),
                                middleware: Vec::new(),
                                parameters: self.extract_route_params(&current_path),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(endpoints)
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
                             file_name.ends_with(".properties") ||
                             file_name.contains("openapi") ||
                             file_name.contains("swagger") ||
                             file_name.contains("api-gateway");
        
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

