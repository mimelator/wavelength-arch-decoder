use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    pub version: String,
    pub patterns: PatternSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSet {
    pub environment_variables: Vec<PatternRule>,
    pub sdk_patterns: Vec<PatternRule>,
    pub api_endpoints: Vec<PatternRule>,
    pub database_patterns: Vec<PatternRule>,
    pub aws_infrastructure: Vec<AwsInfrastructureRule>,
    pub aws_sdk_v2_services: Vec<AwsSdkV2Rule>,
    #[serde(default)]
    pub aws_sdk_v3_service_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRule {
    pub pattern: String,
    pub provider: String,
    pub service_type: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub service_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsInfrastructureRule {
    pub pattern: String,
    pub provider: String,
    pub service_name: String,
    pub service_type: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsSdkV2Rule {
    pub pattern: String,
    pub display_name: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.7
}

pub struct PatternLoader;

impl PatternLoader {
    /// Load patterns from the default config file
    pub fn load_default() -> Result<PatternConfig> {
        let config_path = Path::new("config/service_patterns.json");
        Self::load_from_file(config_path)
    }

    /// Load patterns from a specific file
    pub fn load_from_file(path: &Path) -> Result<PatternConfig> {
        let content = fs::read_to_string(path)?;
        let config: PatternConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Load patterns from multiple files (for plugin system)
    pub fn load_with_plugins(base_path: &Path, plugin_dir: Option<&Path>) -> Result<PatternConfig> {
        let mut config = Self::load_from_file(base_path)?;

        // Load custom pattern files from plugin directory
        if let Some(plugin_path) = plugin_dir {
            if plugin_path.exists() && plugin_path.is_dir() {
                for entry in fs::read_dir(plugin_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(plugin_config) = Self::load_from_file(&path) {
                            // Merge plugin patterns into base config
                            config.patterns.environment_variables.extend(plugin_config.patterns.environment_variables);
                            config.patterns.sdk_patterns.extend(plugin_config.patterns.sdk_patterns);
                            config.patterns.api_endpoints.extend(plugin_config.patterns.api_endpoints);
                            config.patterns.database_patterns.extend(plugin_config.patterns.database_patterns);
                            config.patterns.aws_infrastructure.extend(plugin_config.patterns.aws_infrastructure);
                            config.patterns.aws_sdk_v2_services.extend(plugin_config.patterns.aws_sdk_v2_services);
                            // Merge service maps
                            for (k, v) in plugin_config.patterns.aws_sdk_v3_service_map {
                                config.patterns.aws_sdk_v3_service_map.insert(k, v);
                            }
                        }
                    }
                }
            }
        }

        Ok(config)
    }
}

