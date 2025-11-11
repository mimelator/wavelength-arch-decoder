use wavelength_arch_decoder::analysis::{DependencyExtractor, DependencyGraph, PackageDependency, PackageManager};
use tempfile::TempDir;
use std::fs;

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

