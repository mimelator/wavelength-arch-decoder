use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PackageManager {
    Npm,
    Pip,
    Cargo,
    Maven,
    Gradle,
    Go,
    Composer,
    NuGet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version: String,
    pub package_manager: PackageManager,
    pub is_dev: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyManifest {
    pub package_manager: PackageManager,
    pub dependencies: Vec<PackageDependency>,
    pub file_path: String,
}

pub struct DependencyExtractor;

impl DependencyExtractor {
    pub fn new() -> Self {
        DependencyExtractor
    }

    /// Extract dependencies from a repository
    pub fn extract_from_repository(&self, repo_path: &Path) -> Result<Vec<DependencyManifest>> {
        let mut manifests = Vec::new();

        // Look for package.json (npm)
        if let Some(manifest) = self.extract_npm(repo_path)? {
            manifests.push(manifest);
        }

        // Look for requirements.txt or setup.py (pip)
        if let Some(manifest) = self.extract_pip(repo_path)? {
            manifests.push(manifest);
        }

        // Look for Cargo.toml (cargo)
        if let Some(manifest) = self.extract_cargo(repo_path)? {
            manifests.push(manifest);
        }

        // Look for pom.xml (maven)
        if let Some(manifest) = self.extract_maven(repo_path)? {
            manifests.push(manifest);
        }

        // Look for go.mod (go)
        if let Some(manifest) = self.extract_go(repo_path)? {
            manifests.push(manifest);
        }

        Ok(manifests)
    }

    /// Extract npm dependencies from package.json
    fn extract_npm(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        let package_json = repo_path.join("package.json");
        if !package_json.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&package_json)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let mut dependencies = Vec::new();

        // Extract dependencies
        if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(PackageDependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("unknown").to_string(),
                    package_manager: PackageManager::Npm,
                    is_dev: false,
                    is_optional: false,
                });
            }
        }

        // Extract devDependencies
        if let Some(deps) = json.get("devDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(PackageDependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("unknown").to_string(),
                    package_manager: PackageManager::Npm,
                    is_dev: true,
                    is_optional: false,
                });
            }
        }

        // Extract optionalDependencies
        if let Some(deps) = json.get("optionalDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(PackageDependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("unknown").to_string(),
                    package_manager: PackageManager::Npm,
                    is_dev: false,
                    is_optional: true,
                });
            }
        }

        if dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::Npm,
            dependencies,
            file_path: "package.json".to_string(),
        }))
    }

    /// Extract pip dependencies from requirements.txt
    fn extract_pip(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        let requirements_txt = repo_path.join("requirements.txt");
        if !requirements_txt.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&requirements_txt)?;
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse format: package==version or package>=version, etc.
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(package_spec) = parts.first() {
                let package_spec = *package_spec; // Dereference to get &str
                // Handle formats like: package==1.0.0, package>=1.0.0, package~=1.0.0
                let (name, version) = if package_spec.contains("==") {
                    if let Some(eq_pos) = package_spec.find("==") {
                        let name = &package_spec[..eq_pos];
                        let version = &package_spec[eq_pos + 2..];
                        (name, version.to_string())
                    } else {
                        (package_spec, "latest".to_string())
                    }
                } else if package_spec.contains(">=") {
                    if let Some(ge_pos) = package_spec.find(">=") {
                        let name = &package_spec[..ge_pos];
                        let version = format!(">={}", &package_spec[ge_pos + 2..]);
                        (name, version)
                    } else {
                        (package_spec, "latest".to_string())
                    }
                } else if package_spec.contains("<=") {
                    if let Some(le_pos) = package_spec.find("<=") {
                        let name = &package_spec[..le_pos];
                        let version = format!("<={}", &package_spec[le_pos + 2..]);
                        (name, version)
                    } else {
                        (package_spec, "latest".to_string())
                    }
                } else if package_spec.contains("~=") {
                    if let Some(tilde_pos) = package_spec.find("~=") {
                        let name = &package_spec[..tilde_pos];
                        let version = format!("~={}", &package_spec[tilde_pos + 2..]);
                        (name, version)
                    } else {
                        (package_spec, "latest".to_string())
                    }
                } else if package_spec.contains('>') && !package_spec.contains(">=") {
                    if let Some(gt_pos) = package_spec.find('>') {
                        let name = &package_spec[..gt_pos];
                        let version = format!(">{}", &package_spec[gt_pos + 1..]);
                        (name, version)
                    } else {
                        (package_spec, "latest".to_string())
                    }
                } else if package_spec.contains('<') && !package_spec.contains("<=") {
                    if let Some(lt_pos) = package_spec.find('<') {
                        let name = &package_spec[..lt_pos];
                        let version = format!("<{}", &package_spec[lt_pos + 1..]);
                        (name, version)
                    } else {
                        (package_spec, "latest".to_string())
                    }
                } else {
                    (package_spec, "latest".to_string())
                };

                dependencies.push(PackageDependency {
                    name: name.to_string(),
                    version,
                    package_manager: PackageManager::Pip,
                    is_dev: false,
                    is_optional: false,
                });
            }
        }

        if dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::Pip,
            dependencies,
            file_path: "requirements.txt".to_string(),
        }))
    }

    /// Extract Cargo dependencies from Cargo.toml
    fn extract_cargo(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        let cargo_toml = repo_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&cargo_toml)?;
        let toml: toml::Value = toml::from_str(&content)?;

        let mut dependencies = Vec::new();

        // Extract [dependencies]
        if let Some(deps) = toml.get("dependencies").and_then(|v| v.as_table()) {
            for (name, value) in deps {
                let version = if let Some(version_str) = value.as_str() {
                    version_str.to_string()
                } else if let Some(table) = value.as_table() {
                    table.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "unknown".to_string()
                };

                dependencies.push(PackageDependency {
                    name: name.clone(),
                    version,
                    package_manager: PackageManager::Cargo,
                    is_dev: false,
                    is_optional: false,
                });
            }
        }

        // Extract [dev-dependencies]
        if let Some(deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
            for (name, value) in deps {
                let version = if let Some(version_str) = value.as_str() {
                    version_str.to_string()
                } else if let Some(table) = value.as_table() {
                    table.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "unknown".to_string()
                };

                dependencies.push(PackageDependency {
                    name: name.clone(),
                    version,
                    package_manager: PackageManager::Cargo,
                    is_dev: true,
                    is_optional: false,
                });
            }
        }

        if dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::Cargo,
            dependencies,
            file_path: "Cargo.toml".to_string(),
        }))
    }

    /// Extract Maven dependencies from pom.xml
    fn extract_maven(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        let pom_xml = repo_path.join("pom.xml");
        if !pom_xml.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&pom_xml)?;
        
        // Simple XML parsing for dependencies
        // For production, consider using a proper XML parser
        let mut dependencies = Vec::new();
        let mut in_dependency = false;
        let mut current_name = String::new();
        let mut current_version = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.contains("<dependency>") {
                in_dependency = true;
                current_name.clear();
                current_version.clear();
            } else if line.contains("</dependency>") {
                if !current_name.is_empty() {
                    dependencies.push(PackageDependency {
                        name: current_name.clone(),
                        version: if current_version.is_empty() { "unknown".to_string() } else { current_version.clone() },
                        package_manager: PackageManager::Maven,
                        is_dev: false,
                        is_optional: false,
                    });
                }
                in_dependency = false;
            } else if in_dependency {
                if line.contains("<groupId>") {
                    // Extract groupId
                } else if line.contains("<artifactId>") {
                    if let Some(start) = line.find(">") {
                        if let Some(end) = line.find("</artifactId>") {
                            current_name = line[start + 1..end].trim().to_string();
                        }
                    }
                } else if line.contains("<version>") {
                    if let Some(start) = line.find(">") {
                        if let Some(end) = line.find("</version>") {
                            current_version = line[start + 1..end].trim().to_string();
                        }
                    }
                }
            }
        }

        if dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::Maven,
            dependencies,
            file_path: "pom.xml".to_string(),
        }))
    }

    /// Extract Go dependencies from go.mod
    fn extract_go(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        let go_mod = repo_path.join("go.mod");
        if !go_mod.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&go_mod)?;
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("require (") {
                // Multi-line require block
                continue;
            } else if line.starts_with("require ") && !line.contains("(") {
                // Single-line require
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1];
                    let version = if parts.len() >= 3 {
                        parts[2].to_string()
                    } else {
                        "latest".to_string()
                    };
                    dependencies.push(PackageDependency {
                        name: name.to_string(),
                        version,
                        package_manager: PackageManager::Go,
                        is_dev: false,
                        is_optional: false,
                    });
                }
            } else if !line.starts_with("module ") && 
                      !line.starts_with("go ") && 
                      !line.is_empty() &&
                      !line.starts_with("//") &&
                      !line.contains("require") &&
                      !line.contains("(") &&
                      !line.contains(")") {
                // Continuation of require block
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 1 {
                    let name = parts[0];
                    let version = if parts.len() >= 2 {
                        parts[1].to_string()
                    } else {
                        "latest".to_string()
                    };
                    dependencies.push(PackageDependency {
                        name: name.to_string(),
                        version,
                        package_manager: PackageManager::Go,
                        is_dev: false,
                        is_optional: false,
                    });
                }
            }
        }

        if dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::Go,
            dependencies,
            file_path: "go.mod".to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_extract_npm() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");
        fs::write(&package_json, r#"{
            "name": "test",
            "dependencies": {
                "express": "^4.18.0",
                "lodash": "4.17.21"
            },
            "devDependencies": {
                "jest": "^29.0.0"
            }
        }"#).unwrap();

        let extractor = DependencyExtractor::new();
        let manifest = extractor.extract_npm(temp_dir.path()).unwrap().unwrap();
        
        assert_eq!(manifest.package_manager, PackageManager::Npm);
        assert_eq!(manifest.dependencies.len(), 3);
        assert!(manifest.dependencies.iter().any(|d| d.name == "express"));
        assert!(manifest.dependencies.iter().any(|d| d.name == "jest" && d.is_dev));
    }

    #[test]
    fn test_extract_cargo() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, r#"
[package]
name = "test"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
mockito = "1.0"
        "#).unwrap();

        let extractor = DependencyExtractor::new();
        let manifest = extractor.extract_cargo(temp_dir.path()).unwrap().unwrap();
        
        assert_eq!(manifest.package_manager, PackageManager::Cargo);
        assert!(manifest.dependencies.len() >= 2);
    }
}

