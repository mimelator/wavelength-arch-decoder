use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

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
    SwiftPackageManager,
    CocoaPods,
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

        // Look for Package.swift (Swift Package Manager)
        if let Some(manifest) = self.extract_swift_package_manager(repo_path)? {
            manifests.push(manifest);
        }

        // Look for Podfile (CocoaPods)
        if let Some(manifest) = self.extract_cocoapods(repo_path)? {
            manifests.push(manifest);
        }

        // Look for Xcode project files (.xcodeproj/project.pbxproj) for Swift Package Manager dependencies
        if let Some(manifest) = self.extract_xcode_packages(repo_path)? {
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

    /// Extract Swift Package Manager dependencies from Package.swift
    fn extract_swift_package_manager(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        // Look for Package.swift in the root or any subdirectory
        let mut package_swift = repo_path.join("Package.swift");
        if !package_swift.exists() {
            // Try to find it in subdirectories (SPM allows nested Package.swift)
            let mut found = None;
            for entry in std::fs::read_dir(repo_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let nested_package = path.join("Package.swift");
                    if nested_package.exists() {
                        found = Some(nested_package);
                        break;
                    }
                }
            }
            if let Some(found_path) = found {
                package_swift = found_path;
            } else {
                return Ok(None);
            }
        }
        let content = std::fs::read_to_string(&package_swift)?;
        let mut dependencies = Vec::new();

        // Parse Package.swift - look for .package(url:from:) or .package(url:exact:) patterns
        let lines: Vec<&str> = content.lines().collect();
        let mut in_dependencies = false;
        
        for line in lines {
            let line = line.trim();
            
            // Detect dependencies section
            if line.contains("dependencies:") || line.contains("dependencies =") {
                in_dependencies = true;
                continue;
            }
            
            if in_dependencies {
                // Look for .package(url:from:) or .package(url:exact:)
                if line.contains(".package(") {
                    // Extract URL and version
                    if let Some(url_start) = line.find("url:") {
                        let after_url = &line[url_start + 4..];
                        let url = if let Some(url_end) = after_url.find(',') {
                            after_url[..url_end].trim_matches(|c| c == '"' || c == ' ' || c == '\'')
                        } else if let Some(url_end) = after_url.find(')') {
                            after_url[..url_end].trim_matches(|c| c == '"' || c == ' ' || c == '\'')
                        } else {
                            continue;
                        };
                        
                        // Extract package name from URL (last component)
                        let name = url.split('/').last().unwrap_or("unknown").trim_end_matches(".git");
                        
                        // Extract version
                        let version = if line.contains("from:") {
                            if let Some(version_start) = line.find("from:") {
                                let after_from = &line[version_start + 5..];
                                if let Some(version_end) = after_from.find(',') {
                                    after_from[..version_end].trim_matches(|c| c == '"' || c == ' ' || c == '\'')
                                } else if let Some(version_end) = after_from.find(')') {
                                    after_from[..version_end].trim_matches(|c| c == '"' || c == ' ' || c == '\'')
                                } else {
                                    "unknown"
                                }
                            } else {
                                "unknown"
                            }
                        } else if line.contains("exact:") {
                            if let Some(version_start) = line.find("exact:") {
                                let after_exact = &line[version_start + 6..];
                                if let Some(version_end) = after_exact.find(',') {
                                    after_exact[..version_end].trim_matches(|c| c == '"' || c == ' ' || c == '\'')
                                } else if let Some(version_end) = after_exact.find(')') {
                                    after_exact[..version_end].trim_matches(|c| c == '"' || c == ' ' || c == '\'')
                                } else {
                                    "unknown"
                                }
                            } else {
                                "unknown"
                            }
                        } else {
                            "latest"
                        };
                        
                        dependencies.push(PackageDependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            package_manager: PackageManager::SwiftPackageManager,
                            is_dev: false,
                            is_optional: false,
                        });
                    }
                }
                
                // Stop if we hit another top-level declaration
                if line.starts_with("targets:") || line.starts_with("products:") || line.starts_with("name:") {
                    break;
                }
            }
        }

        if dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::SwiftPackageManager,
            dependencies,
            file_path: "Package.swift".to_string(),
        }))
    }

    /// Extract CocoaPods dependencies from Podfile
    fn extract_cocoapods(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        let podfile = repo_path.join("Podfile");
        if !podfile.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&podfile)?;
        let mut dependencies = Vec::new();

        // Parse Podfile - look for pod 'Name', 'version' patterns
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
            // Look for pod declarations: pod 'Name', '~> version' or pod 'Name'
            if line.starts_with("pod ") || line.starts_with("pod '") || line.starts_with("pod \"") {
                let pod_line = line.strip_prefix("pod ").unwrap_or(line);
                let pod_line = pod_line.trim();
                
                // Extract pod name (first quoted string)
                let name = if let Some(start) = pod_line.find('\'') {
                    if let Some(end) = pod_line[start+1..].find('\'') {
                        pod_line[start+1..start+1+end].to_string()
                    } else if let Some(start) = pod_line.find('"') {
                        if let Some(end) = pod_line[start+1..].find('"') {
                            pod_line[start+1..start+1+end].to_string()
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else if let Some(start) = pod_line.find('"') {
                    if let Some(end) = pod_line[start+1..].find('"') {
                        pod_line[start+1..start+1+end].to_string()
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                
                // Extract version (look for '~>', '>=', '<=', '==', or quoted version)
                let version = if let Some(version_op) = pod_line.find("~>") {
                    let after_op = &pod_line[version_op + 2..];
                    if let Some(end) = after_op.find('\'') {
                        after_op[..end].trim().to_string()
                    } else if let Some(end) = after_op.find('"') {
                        after_op[..end].trim().to_string()
                    } else if let Some(end) = after_op.find(',') {
                        after_op[..end].trim().to_string()
                    } else {
                        after_op.trim().to_string()
                    }
                } else if let Some(version_start) = pod_line.find(',') {
                    let after_comma = &pod_line[version_start + 1..];
                    let version_str = after_comma.trim();
                    if let Some(start) = version_str.find('\'') {
                        if let Some(end) = version_str[start+1..].find('\'') {
                            version_str[start+1..start+1+end].to_string()
                        } else {
                            "latest".to_string()
                        }
                    } else if let Some(start) = version_str.find('"') {
                        if let Some(end) = version_str[start+1..].find('"') {
                            version_str[start+1..start+1+end].to_string()
                        } else {
                            "latest".to_string()
                        }
                    } else {
                        "latest".to_string()
                    }
                } else {
                    "latest".to_string()
                };
                
                dependencies.push(PackageDependency {
                    name,
                    version,
                    package_manager: PackageManager::CocoaPods,
                    is_dev: false,
                    is_optional: false,
                });
            }
        }

        if dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::CocoaPods,
            dependencies,
            file_path: "Podfile".to_string(),
        }))
    }

    /// Extract Swift Package Manager dependencies from Xcode project files
    fn extract_xcode_packages(&self, repo_path: &Path) -> Result<Option<DependencyManifest>> {
        // Find all .xcodeproj directories
        let mut xcode_projects = Vec::new();
        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let path = entry.path();
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".xcodeproj"))
                .unwrap_or(false)
            {
                let project_pbxproj = path.join("project.pbxproj");
                if project_pbxproj.exists() {
                    xcode_projects.push(project_pbxproj);
                }
            }
        }

        if xcode_projects.is_empty() {
            return Ok(None);
        }

        let mut all_dependencies = Vec::new();
        let mut seen_packages: std::collections::HashSet<String> = std::collections::HashSet::new();

        for project_file in xcode_projects {
            let content = std::fs::read_to_string(&project_file)?;
            
            // Parse XCRemoteSwiftPackageReference sections
            // Format: XCRemoteSwiftPackageReference "package-name" = { isa = XCRemoteSwiftPackageReference; repositoryURL = "https://..."; ... }
            let lines: Vec<&str> = content.lines().collect();
            let mut in_package_ref = false;
            let mut current_name: Option<String> = None;
            let mut current_url: Option<String> = None;

            for line in lines {
                let line = line.trim();
                
                // Detect start of XCRemoteSwiftPackageReference
                if line.contains("XCRemoteSwiftPackageReference") && line.contains('"') {
                    // Extract package name from: XCRemoteSwiftPackageReference "package-name"
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start+1..].find('"') {
                            current_name = Some(line[start+1..start+1+end].to_string());
                            in_package_ref = true;
                            current_url = None;
                        }
                    }
                }
                
                // Extract repositoryURL
                if in_package_ref && line.contains("repositoryURL") {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start+1..].find('"') {
                            current_url = Some(line[start+1..start+1+end].to_string());
                        }
                    } else if let Some(start) = line.find('=') {
                        // Sometimes URL is on next line or without quotes
                        let url_part = line[start+1..].trim();
                        if !url_part.is_empty() && (url_part.starts_with("http") || url_part.starts_with("git@")) {
                            current_url = Some(url_part.trim_matches(|c| c == '"' || c == ';' || c == ' ').to_string());
                        }
                    }
                }
                
                // Detect end of package reference block
                if in_package_ref && line == "};" {
                    if let (Some(name), Some(url)) = (current_name.take(), current_url.take()) {
                        // Create a unique key from URL to avoid duplicates
                        let url_key = url.clone();
                        if !seen_packages.contains(&url_key) {
                            seen_packages.insert(url_key.clone());
                            
                            // Extract version/revision if available (look for requirementKind, branch, version, etc.)
                            let version = if let Some(url_pos) = content.find(&format!("repositoryURL = \"{}\"", url)) {
                                // Try to find version requirement after this URL
                                let after_url = &content[url_pos..];
                                // Look for requirementKind, branch, version, or revision
                                if let Some(branch_start) = after_url.find("branch =") {
                                    if let Some(branch_quote) = after_url[branch_start..].find('"') {
                                        if let Some(branch_end) = after_url[branch_start+branch_quote+1..].find('"') {
                                            let branch = &after_url[branch_start+branch_quote+1..branch_start+branch_quote+1+branch_end];
                                            format!("branch:{}", branch)
                                        } else {
                                            "latest".to_string()
                                        }
                                    } else {
                                        "latest".to_string()
                                    }
                                } else if let Some(version_start) = after_url.find("version =") {
                                    if let Some(version_quote) = after_url[version_start..].find('"') {
                                        if let Some(version_end) = after_url[version_start+version_quote+1..].find('"') {
                                            let version_str = &after_url[version_start+version_quote+1..version_start+version_quote+1+version_end];
                                            format!("version:{}", version_str)
                                        } else {
                                            "latest".to_string()
                                        }
                                    } else {
                                        "latest".to_string()
                                    }
                                } else {
                                    "latest".to_string()
                                }
                            } else {
                                "latest".to_string()
                            };
                            
                            all_dependencies.push(PackageDependency {
                                name: name.clone(),
                                version,
                                package_manager: PackageManager::SwiftPackageManager,
                                is_dev: false,
                                is_optional: false,
                            });
                        }
                    }
                    in_package_ref = false;
                    current_name = None;
                    current_url = None;
                }
            }
        }

        if all_dependencies.is_empty() {
            return Ok(None);
        }

        Ok(Some(DependencyManifest {
            package_manager: PackageManager::SwiftPackageManager,
            dependencies: all_dependencies,
            file_path: "project.pbxproj".to_string(),
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

