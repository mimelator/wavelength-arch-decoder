use wavelength_arch_decoder::analysis::{DependencyExtractor, PackageDependency, PackageManager, PortDetector, EndpointDetector};
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[test]
fn test_npm_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let package_json = temp_dir.path().join("package.json");
    
    fs::write(&package_json, r#"{
        "name": "test-project",
        "version": "1.0.0",
        "dependencies": {
            "express": "^4.18.0",
            "lodash": "4.17.21"
        },
        "devDependencies": {
            "jest": "^29.0.0",
            "typescript": "5.0.0"
        },
        "optionalDependencies": {
            "fsevents": "^2.3.0"
        }
    }"#).unwrap();

    let extractor = DependencyExtractor::new();
    let manifests = extractor.extract_from_repository(temp_dir.path()).unwrap();
    
    assert_eq!(manifests.len(), 1);
    let manifest = &manifests[0];
    assert_eq!(manifest.package_manager, PackageManager::Npm);
    assert_eq!(manifest.dependencies.len(), 5);
    
    // Check dependencies
    assert!(manifest.dependencies.iter().any(|d| d.name == "express" && !d.is_dev));
    assert!(manifest.dependencies.iter().any(|d| d.name == "lodash" && !d.is_dev));
    assert!(manifest.dependencies.iter().any(|d| d.name == "jest" && d.is_dev));
    assert!(manifest.dependencies.iter().any(|d| d.name == "typescript" && d.is_dev));
    assert!(manifest.dependencies.iter().any(|d| d.name == "fsevents" && d.is_optional));
}

#[test]
fn test_pip_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let requirements_txt = temp_dir.path().join("requirements.txt");
    
    fs::write(&requirements_txt, r#"
# Production dependencies
flask==2.3.0
requests>=2.28.0
numpy~=1.24.0
pandas>1.5.0
scipy<2.0.0

# Development
pytest>=7.0.0
    "#).unwrap();

    let extractor = DependencyExtractor::new();
    let manifests = extractor.extract_from_repository(temp_dir.path()).unwrap();
    
    assert_eq!(manifests.len(), 1);
    let manifest = &manifests[0];
    assert_eq!(manifest.package_manager, PackageManager::Pip);
    assert_eq!(manifest.dependencies.len(), 6);
    
    // Check specific dependencies
    let flask = manifest.dependencies.iter().find(|d| d.name == "flask").unwrap();
    assert_eq!(flask.version, "2.3.0");
    
    let requests = manifest.dependencies.iter().find(|d| d.name == "requests").unwrap();
    assert!(requests.version.starts_with(">="));
}

#[test]
fn test_cargo_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    
    fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
actix-web = "4.5"

[dev-dependencies]
mockito = "1.2"
tempfile = "3.9"
    "#).unwrap();

    let extractor = DependencyExtractor::new();
    let manifests = extractor.extract_from_repository(temp_dir.path()).unwrap();
    
    assert_eq!(manifests.len(), 1);
    let manifest = &manifests[0];
    assert_eq!(manifest.package_manager, PackageManager::Cargo);
    assert!(manifest.dependencies.len() >= 3);
    
    // Check dependencies
    assert!(manifest.dependencies.iter().any(|d| d.name == "serde" && !d.is_dev));
    assert!(manifest.dependencies.iter().any(|d| d.name == "tokio" && !d.is_dev));
    assert!(manifest.dependencies.iter().any(|d| d.name == "mockito" && d.is_dev));
}

#[test]
fn test_go_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let go_mod = temp_dir.path().join("go.mod");
    
    fs::write(&go_mod, r#"
module github.com/example/test

go 1.21

require (
    github.com/gin-gonic/gin v1.9.1
    github.com/stretchr/testify v1.8.4
)
    "#).unwrap();

    let extractor = DependencyExtractor::new();
    let manifests = extractor.extract_from_repository(temp_dir.path()).unwrap();
    
    assert_eq!(manifests.len(), 1);
    let manifest = &manifests[0];
    assert_eq!(manifest.package_manager, PackageManager::Go);
    assert!(manifest.dependencies.len() >= 2);
}

#[test]
fn test_multiple_package_managers() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple package files
    fs::write(temp_dir.path().join("package.json"), r#"{
        "dependencies": {"express": "^4.18.0"}
    }"#).unwrap();
    
    fs::write(temp_dir.path().join("requirements.txt"), "flask==2.3.0\n").unwrap();
    
    fs::write(temp_dir.path().join("Cargo.toml"), r#"
[dependencies]
serde = "1.0"
    "#).unwrap();

    let extractor = DependencyExtractor::new();
    let manifests = extractor.extract_from_repository(temp_dir.path()).unwrap();
    
    assert_eq!(manifests.len(), 3);
    
    let package_managers: Vec<_> = manifests.iter()
        .map(|m| &m.package_manager)
        .collect();
    
    assert!(package_managers.contains(&&PackageManager::Npm));
    assert!(package_managers.contains(&&PackageManager::Pip));
    assert!(package_managers.contains(&&PackageManager::Cargo));
}

#[test]
fn test_dependency_graph() {
    let mut graph = DependencyGraph::new();
    
    // Add root dependencies
    graph.add_node(
        PackageDependency {
            name: "express".to_string(),
            version: "4.18.0".to_string(),
            package_manager: PackageManager::Npm,
            is_dev: false,
            is_optional: false,
        },
        vec!["body-parser".to_string(), "cookie-parser".to_string()],
    );
    
    graph.add_node(
        PackageDependency {
            name: "body-parser".to_string(),
            version: "1.20.0".to_string(),
            package_manager: PackageManager::Npm,
            is_dev: false,
            is_optional: false,
        },
        vec!["bytes".to_string()],
    );
    
    graph.add_node(
        PackageDependency {
            name: "cookie-parser".to_string(),
            version: "1.4.6".to_string(),
            package_manager: PackageManager::Npm,
            is_dev: false,
            is_optional: false,
        },
        vec![],
    );
    
    graph.add_node(
        PackageDependency {
            name: "bytes".to_string(),
            version: "3.1.2".to_string(),
            package_manager: PackageManager::Npm,
            is_dev: false,
            is_optional: false,
        },
        vec![],
    );
    
    // Test transitive dependencies
    let transitive = graph.resolve_transitive("express").unwrap();
    assert!(transitive.contains("body-parser"));
    assert!(transitive.contains("cookie-parser"));
    assert!(transitive.contains("bytes"));
    
    // Test root dependencies
    let roots = graph.get_root_dependencies();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].name, "express");
}

#[test]
fn test_version_conflict_detection() {
    let mut graph = DependencyGraph::new();
    
    // Add same package with different versions
    graph.add_node(
        PackageDependency {
            name: "lodash".to_string(),
            version: "4.17.21".to_string(),
            package_manager: PackageManager::Npm,
            is_dev: false,
            is_optional: false,
        },
        vec![],
    );
    
    graph.add_node(
        PackageDependency {
            name: "lodash".to_string(),
            version: "4.17.20".to_string(),
            package_manager: PackageManager::Npm,
            is_dev: false,
            is_optional: false,
        },
        vec![],
    );
    
    let conflicts = graph.detect_conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].package_name, "lodash");
    assert_eq!(conflicts[0].versions.len(), 2);
    assert!(conflicts[0].versions.contains(&"4.17.21".to_string()));
    assert!(conflicts[0].versions.contains(&"4.17.20".to_string()));
}

#[test]
fn test_dependency_statistics() {
    let mut graph = DependencyGraph::new();
    
    graph.add_node(
        PackageDependency {
            name: "express".to_string(),
            version: "4.18.0".to_string(),
            package_manager: PackageManager::Npm,
            is_dev: false,
            is_optional: false,
        },
        vec![],
    );
    
    graph.add_node(
        PackageDependency {
            name: "flask".to_string(),
            version: "2.3.0".to_string(),
            package_manager: PackageManager::Pip,
            is_dev: false,
            is_optional: false,
        },
        vec![],
    );
    
    let stats = graph.get_statistics();
    assert_eq!(stats.total_dependencies, 2);
    assert_eq!(stats.unique_packages, 2);
    assert_eq!(stats.conflicts, 0);
    assert_eq!(stats.package_managers.len(), 2);
}

#[test]
fn test_empty_repository() {
    let temp_dir = TempDir::new().unwrap();
    
    let extractor = DependencyExtractor::new();
    let manifests = extractor.extract_from_repository(temp_dir.path()).unwrap();
    
    assert_eq!(manifests.len(), 0);
}

#[test]
fn test_maven_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let pom_xml = temp_dir.path().join("pom.xml");
    
    fs::write(&pom_xml, r#"
<project>
    <dependencies>
        <dependency>
            <groupId>org.springframework</groupId>
            <artifactId>spring-core</artifactId>
            <version>5.3.21</version>
        </dependency>
        <dependency>
            <groupId>com.fasterxml.jackson.core</groupId>
            <artifactId>jackson-databind</artifactId>
            <version>2.13.3</version>
        </dependency>
    </dependencies>
</project>
    "#).unwrap();

    let extractor = DependencyExtractor::new();
    let manifests = extractor.extract_from_repository(temp_dir.path()).unwrap();
    
    assert_eq!(manifests.len(), 1);
    let manifest = &manifests[0];
    assert_eq!(manifest.package_manager, PackageManager::Maven);
    assert_eq!(manifest.dependencies.len(), 2);
    
    assert!(manifest.dependencies.iter().any(|d| d.name == "spring-core"));
    assert!(manifest.dependencies.iter().any(|d| d.name == "jackson-databind"));
}

#[test]
fn test_port_and_endpoint_detection_spring_petclinic() {
    let repo_path = Path::new("./cache/repos/spring-petclinic");
    
    if !repo_path.exists() {
        println!("⚠️  Repository not found at {:?}, skipping test", repo_path);
        return;
    }
    
    println!("🔍 Testing port detection on Spring PetClinic...");
    let port_detector = PortDetector::new();
    
    match port_detector.detect_ports(repo_path) {
        Ok(ports) => {
            println!("✓ Found {} port(s)", ports.len());
            for port in ports.iter().take(10) {
                println!("  - Port {} ({:?}) in {}:{}", 
                    port.port, 
                    port.port_type,
                    port.file_path,
                    port.line_number.unwrap_or(0)
                );
            }
            // Spring Boot typically uses port 8080, but may not be explicitly set
            // So we just check that detection runs without error
            println!("Port detection completed successfully");
        }
        Err(e) => {
            eprintln!("❌ Error detecting ports: {}", e);
            panic!("Port detection failed: {}", e);
        }
    }
    
    println!("\n🔍 Testing endpoint detection on Spring PetClinic...");
    let endpoint_detector = EndpointDetector::new();
    
    match endpoint_detector.detect_endpoints(repo_path) {
        Ok(endpoints) => {
            println!("✓ Found {} endpoint(s)", endpoints.len());
            for endpoint in endpoints.iter().take(10) {
                println!("  - {:?} {} ({:?}) in {}:{}", 
                    endpoint.method,
                    endpoint.path,
                    endpoint.framework,
                    endpoint.file_path,
                    endpoint.line_number.unwrap_or(0)
                );
            }
            // Spring PetClinic should have endpoints like /owners, /vets, etc.
            assert!(endpoints.len() > 0, "Should detect at least one endpoint");
            assert!(endpoints.iter().any(|e| e.path.contains("/owners") || e.path.contains("/vets")), 
                "Should detect Spring Boot endpoints");
        }
        Err(e) => {
            eprintln!("❌ Error detecting endpoints: {}", e);
            panic!("Endpoint detection failed: {}", e);
        }
    }
}


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
    use wavelength_arch_decoder::analysis::HttpMethod;
    assert!(endpoints.iter().any(|e| e.path == "/api/users" && matches!(e.method, HttpMethod::Get)), 
            "Should detect GET /api/users");
    assert!(endpoints.iter().any(|e| e.path == "/api/users" && matches!(e.method, HttpMethod::Post)), 
            "Should detect POST /api/users");
    assert!(endpoints.iter().any(|e| e.path == "/api/users/:id" && matches!(e.method, HttpMethod::Get)), 
            "Should detect GET /api/users/:id");
    
    println!("✓ Detected endpoints:");
    for ep in &endpoints {
        println!("  {:?} {} (framework: {:?})", ep.method, ep.path, ep.framework);
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
        println!("  {:?} {} (framework: {:?}, handler: {:?})", 
            ep.method, ep.path, ep.framework, ep.handler);
    }
}
