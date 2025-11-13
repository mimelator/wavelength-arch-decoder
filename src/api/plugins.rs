use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub patterns_count: PluginPatternCounts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginPatternCounts {
    pub environment_variables: usize,
    pub sdk_patterns: usize,
    pub api_endpoints: usize,
    pub database_patterns: usize,
    pub aws_infrastructure: usize,
    pub aws_sdk_v2_services: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginsResponse {
    pub plugins: Vec<PluginInfo>,
    pub total: usize,
}

/// Get list of loaded plugins
pub async fn get_plugins() -> impl Responder {
    let plugin_dir = Path::new("config/plugins");
    let mut plugins = Vec::new();

    if plugin_dir.exists() && plugin_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(plugin_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        let plugin_name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        // Try to load plugin config to get details
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(plugin_config) = serde_json::from_str::<crate::security::pattern_config::PatternConfig>(&content) {
                                let patterns_count = PluginPatternCounts {
                                    environment_variables: plugin_config.patterns.environment_variables.len(),
                                    sdk_patterns: plugin_config.patterns.sdk_patterns.len(),
                                    api_endpoints: plugin_config.patterns.api_endpoints.len(),
                                    database_patterns: plugin_config.patterns.database_patterns.len(),
                                    aws_infrastructure: plugin_config.patterns.aws_infrastructure.len(),
                                    aws_sdk_v2_services: plugin_config.patterns.aws_sdk_v2_services.len(),
                                };

                                plugins.push(PluginInfo {
                                    name: plugin_name,
                                    path: path.to_string_lossy().to_string(),
                                    version: Some(plugin_config.version),
                                    patterns_count,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    plugins.sort_by(|a, b| a.name.cmp(&b.name));

    HttpResponse::Ok().json(PluginsResponse {
        total: plugins.len(),
        plugins,
    })
}

