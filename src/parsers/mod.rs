use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;
use crate::ingestion::FileType;

pub struct FileParser;

impl FileParser {
    /// Parse a file based on its type
    pub fn parse_file(&self, path: &Path, file_type: &FileType) -> Result<ParsedFile> {
        let content = fs::read_to_string(path)?;
        
        match file_type {
            FileType::Config => self.parse_config_file(path, &content),
            FileType::Code => Ok(ParsedFile::Code { content }),
            FileType::Infrastructure => self.parse_infrastructure_file(path, &content),
            FileType::CiCd => self.parse_cicd_file(path, &content),
            FileType::Documentation => Ok(ParsedFile::Documentation { content }),
            FileType::Other => Ok(ParsedFile::Other { content }),
        }
    }

    /// Parse configuration files (JSON, YAML, TOML)
    fn parse_config_file(&self, path: &Path, content: &str) -> Result<ParsedFile> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "json" => {
                let json: Value = serde_json::from_str(content)?;
                Ok(ParsedFile::Config {
                    format: "json".to_string(),
                    data: json,
                })
            }
            "yaml" | "yml" => {
                let yaml: Value = serde_yaml::from_str(content)?;
                Ok(ParsedFile::Config {
                    format: "yaml".to_string(),
                    data: yaml,
                })
            }
            "toml" => {
                let toml: Value = toml::from_str(content)?;
                Ok(ParsedFile::Config {
                    format: "toml".to_string(),
                    data: toml,
                })
            }
            _ => Ok(ParsedFile::Config {
                format: "text".to_string(),
                data: Value::String(content.to_string()),
            }),
        }
    }

    /// Parse infrastructure as code files
    fn parse_infrastructure_file(&self, path: &Path, content: &str) -> Result<ParsedFile> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "tf" | "tfvars" => {
                // Terraform files - basic parsing
                Ok(ParsedFile::Infrastructure {
                    provider: self.detect_provider(content),
                    resources: self.extract_resources(content),
                })
            }
            "yaml" | "yml" => {
                // Could be CloudFormation, Kubernetes, etc.
                let yaml: Value = serde_yaml::from_str(content)?;
                Ok(ParsedFile::Infrastructure {
                    provider: self.detect_provider_from_yaml(&yaml),
                    resources: self.extract_resources_from_yaml(&yaml),
                })
            }
            _ => Ok(ParsedFile::Infrastructure {
                provider: "unknown".to_string(),
                resources: vec![],
            }),
        }
    }

    /// Parse CI/CD configuration files
    fn parse_cicd_file(&self, path: &Path, content: &str) -> Result<ParsedFile> {
        let path_str = path.to_string_lossy().to_lowercase();
        
        let platform = if path_str.contains("github") {
            "github"
        } else if path_str.contains("gitlab") {
            "gitlab"
        } else if path_str.contains("jenkins") {
            "jenkins"
        } else if path_str.contains("circleci") {
            "circleci"
        } else if path_str.contains("travis") {
            "travis"
        } else if path_str.contains("azure") {
            "azure"
        } else {
            "unknown"
        };

        let yaml: Value = serde_yaml::from_str(content).unwrap_or(Value::Null);
        
        Ok(ParsedFile::CiCd {
            platform: platform.to_string(),
            config: yaml,
        })
    }

    /// Detect cloud provider from content
    fn detect_provider(&self, content: &str) -> String {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("aws") || content_lower.contains("amazon") {
            "aws"
        } else if content_lower.contains("azurerm") || content_lower.contains("azure") {
            "azure"
        } else if content_lower.contains("google") || content_lower.contains("gcp") {
            "gcp"
        } else if content_lower.contains("kubernetes") || content_lower.contains("k8s") {
            "kubernetes"
        } else {
            "unknown"
        }.to_string()
    }

    /// Detect provider from YAML structure
    fn detect_provider_from_yaml(&self, yaml: &Value) -> String {
        if let Some(obj) = yaml.as_object() {
            if obj.contains_key("AWSTemplateFormatVersion") {
                return "aws".to_string();
            }
            if obj.contains_key("apiVersion") && obj.contains_key("kind") {
                return "kubernetes".to_string();
            }
        }
        "unknown".to_string()
    }

    /// Extract resource names from Terraform content
    fn extract_resources(&self, content: &str) -> Vec<String> {
        let mut resources = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("resource \"") {
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start+1..].find('"') {
                        let resource_type = &trimmed[start+1..start+1+end];
                        resources.push(resource_type.to_string());
                    }
                }
            }
        }
        
        resources
    }

    /// Extract resources from YAML
    fn extract_resources_from_yaml(&self, yaml: &Value) -> Vec<String> {
        let mut resources = Vec::new();
        
        if let Some(obj) = yaml.as_object() {
            if let Some(resources_obj) = obj.get("Resources") {
                if let Some(resources_map) = resources_obj.as_object() {
                    for key in resources_map.keys() {
                        resources.push(key.clone());
                    }
                }
            }
        }
        
        resources
    }
}

#[derive(Debug, Clone)]
pub enum ParsedFile {
    Config {
        format: String,
        data: Value,
    },
    Code {
        content: String,
    },
    Infrastructure {
        provider: String,
        resources: Vec<String>,
    },
    CiCd {
        platform: String,
        config: Value,
    },
    Documentation {
        content: String,
    },
    Other {
        content: String,
    },
}

