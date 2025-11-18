#[cfg(test)]
mod tests {
    use std::path::Path;
    use tempfile::TempDir;
    use std::fs;
    use wavelength_arch_decoder::analysis::{PortDetector, EndpointDetector};

    #[test]
    fn test_port_detection_express() {
        let temp_dir = TempDir::new().unwrap();
        let server_file = temp_dir.path().join("server.js");
        
        fs::write(&server_file, r#"
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

app.get('/api/users', (req, res) => {
  res.json({ users: [] });
});

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
"#).unwrap();

        let detector = PortDetector::new();
        let ports = detector.detect_ports(temp_dir.path()).unwrap();
        
        // Should detect port 3000 from app.listen(PORT)
        assert!(ports.iter().any(|p| p.port == 3000), "Should detect port 3000");
        println!("✓ Detected ports: {:?}", ports.iter().map(|p| p.port).collect::<Vec<_>>());
    }

    #[test]
    fn test_port_detection_env_file() {
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");
        
        fs::write(&env_file, r#"
PORT=8080
DATABASE_URL=postgresql://localhost:5432/mydb
REDIS_URL=redis://localhost:6379
"#).unwrap();

        let detector = PortDetector::new();
        let ports = detector.detect_ports(temp_dir.path()).unwrap();
        
        // Should detect ports from .env file
        assert!(ports.iter().any(|p| p.port == 8080), "Should detect PORT=8080");
        assert!(ports.iter().any(|p| p.port == 5432), "Should detect postgres port");
        assert!(ports.iter().any(|p| p.port == 6379), "Should detect redis port");
        println!("✓ Detected ports from .env: {:?}", ports.iter().map(|p| (p.port, &p.port_type)).collect::<Vec<_>>());
    }

    #[test]
    fn test_endpoint_detection_express() {
        let temp_dir = TempDir::new().unwrap();
        let server_file = temp_dir.path().join("server.js");
        
        fs::write(&server_file, r#"
const express = require('express');
const app = express();

app.get('/api/users', (req, res) => {
  res.json({ users: [] });
});

app.post('/api/users', (req, res) => {
  res.json({ id: 1 });
});

app.get('/api/users/:id', (req, res) => {
  res.json({ id: req.params.id });
});
"#).unwrap();

        let detector = EndpointDetector::new();
        let endpoints = detector.detect_endpoints(temp_dir.path()).unwrap();
        
        // Should detect at least 3 endpoints
        assert!(endpoints.len() >= 3, "Should detect at least 3 endpoints");
        
        // Check for specific endpoints
        assert!(endpoints.iter().any(|e| e.path == "/api/users" && e.method.to_string() == "GET"), 
                "Should detect GET /api/users");
        assert!(endpoints.iter().any(|e| e.path == "/api/users" && e.method.to_string() == "POST"), 
                "Should detect POST /api/users");
        assert!(endpoints.iter().any(|e| e.path == "/api/users/:id" && e.method.to_string() == "GET"), 
                "Should detect GET /api/users/:id");
        
        println!("✓ Detected endpoints:");
        for ep in &endpoints {
            println!("  {} {} (framework: {:?})", ep.method, ep.path, ep.framework);
        }
    }

    #[test]
    fn test_endpoint_detection_flask() {
        let temp_dir = TempDir::new().unwrap();
        let app_file = temp_dir.path().join("app.py");
        
        fs::write(&app_file, r#"
from flask import Flask, jsonify
app = Flask(__name__)

@app.route('/api/posts', methods=['GET'])
def get_posts():
    return jsonify([])

@app.route('/api/posts', methods=['POST'])
def create_post():
    return jsonify({'id': 1})

@app.route('/api/posts/<int:post_id>', methods=['GET'])
def get_post(post_id):
    return jsonify({'id': post_id})
"#).unwrap();

        let detector = EndpointDetector::new();
        let endpoints = detector.detect_endpoints(temp_dir.path()).unwrap();
        
        // Should detect Flask endpoints
        assert!(endpoints.len() >= 2, "Should detect Flask endpoints");
        assert!(endpoints.iter().any(|e| e.path == "/api/posts"), "Should detect /api/posts");
        assert!(endpoints.iter().any(|e| e.framework.as_ref().map(|s| s.as_str()) == Some("flask")), 
                "Should identify Flask framework");
        
        println!("✓ Detected Flask endpoints:");
        for ep in &endpoints {
            println!("  {} {} (framework: {:?}, handler: {:?})", 
                ep.method, ep.path, ep.framework, ep.handler);
        }
    }
}

