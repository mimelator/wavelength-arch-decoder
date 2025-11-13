use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolType {
    BuildTool,
    TestFramework,
    Linter,
    Formatter,
    DevServer,
    CodeGenerator,
    Debugger,
    TaskRunner,
    ShellScript,
    DevEnvironment,
    Sdk,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    // Build Tools
    Webpack,
    Vite,
    Rollup,
    Esbuild,
    Tsc,
    Cargo,
    GoBuild,
    Maven,
    Gradle,
    
    // Testing
    Jest,
    Mocha,
    Pytest,
    Unittest,
    CargoTest,
    GoTest,
    
    // Linting
    Eslint,
    Pylint,
    Rustfmt,
    Gofmt,
    
    // Formatting
    Prettier,
    Black,
    
    // Dev Servers
    WebpackDevServer,
    ViteDev,
    Nodemon,
    
    // Task Runners
    NpmScripts,
    Make,
    Just,
    Task,
    
    // Shell Scripts
    Bash,
    Zsh,
    Fish,
    
    // Dev Environments
    Venv,
    Conda,
    DockerDev,
    DevContainer,
    
    // SDKs
    AwsCdk,
    Terraform,
    Serverless,
    FirebaseCli,
    
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolScript {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub file_path: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTool {
    pub name: String,
    pub tool_type: ToolType,
    pub category: ToolCategory,
    pub version: Option<String>,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub detection_method: String, // "package_script", "config_file", "dependency", etc.
    pub configuration: HashMap<String, String>,
    pub scripts: Vec<ToolScript>,
    pub confidence: f64,
}

pub struct ToolDetector;

impl ToolDetector {
    pub fn new() -> Self {
        ToolDetector
    }

    /// Detect tools in a repository
    pub fn detect_tools(&self, repo_path: &Path) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        
        // Detect from package scripts
        tools.extend(self.detect_from_package_scripts(repo_path)?);
        
        // Detect from config files
        tools.extend(self.detect_from_config_files(repo_path)?);
        
        // Detect from dependencies
        tools.extend(self.detect_from_dependencies(repo_path)?);
        
        // Detect script files
        tools.extend(self.detect_script_files(repo_path)?);
        
        // Detect development environments
        tools.extend(self.detect_dev_environments(repo_path)?);
        
        // Deduplicate tools
        Ok(self.deduplicate_tools(tools))
    }

    /// Detect tools from package.json scripts
    fn detect_from_package_scripts(&self, repo_path: &Path) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        let package_json = repo_path.join("package.json");
        
        if !package_json.exists() {
            return Ok(tools);
        }
        
        let content = std::fs::read_to_string(&package_json)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(scripts) = json.get("scripts").and_then(|v| v.as_object()) {
            let mut tool_scripts = Vec::new();
            
            for (script_name, script_command) in scripts {
                let command = script_command.as_str().unwrap_or("");
                tool_scripts.push(ToolScript {
                    name: script_name.clone(),
                    command: command.to_string(),
                    description: None,
                    file_path: "package.json".to_string(),
                    line_number: None,
                });
                
                // Detect tools from command
                if let Some(tool) = self.detect_tool_from_command(command, &package_json) {
                    tools.push(tool);
                }
            }
            
            // Add npm scripts as a tool itself
            if !tool_scripts.is_empty() {
                tools.push(DetectedTool {
                    name: "npm scripts".to_string(),
                    tool_type: ToolType::TaskRunner,
                    category: ToolCategory::NpmScripts,
                    version: None,
                    file_path: "package.json".to_string(),
                    line_number: None,
                    detection_method: "package_script".to_string(),
                    configuration: HashMap::new(),
                    scripts: tool_scripts,
                    confidence: 0.9,
                });
            }
        }
        
        Ok(tools)
    }

    /// Detect tool from a command string
    fn detect_tool_from_command(&self, command: &str, file_path: &Path) -> Option<DetectedTool> {
        let command_lower = command.to_lowercase();
        
        // Build tools
        if command_lower.contains("webpack") {
            return Some(DetectedTool {
                name: "webpack".to_string(),
                tool_type: ToolType::BuildTool,
                category: ToolCategory::Webpack,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        if command_lower.contains("vite") {
            return Some(DetectedTool {
                name: "vite".to_string(),
                tool_type: ToolType::BuildTool,
                category: ToolCategory::Vite,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        if command_lower.contains("tsc") || command_lower.contains("typescript") {
            return Some(DetectedTool {
                name: "TypeScript Compiler".to_string(),
                tool_type: ToolType::BuildTool,
                category: ToolCategory::Tsc,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.85,
            });
        }
        
        // Testing frameworks
        if command_lower.contains("jest") {
            return Some(DetectedTool {
                name: "jest".to_string(),
                tool_type: ToolType::TestFramework,
                category: ToolCategory::Jest,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        if command_lower.contains("mocha") {
            return Some(DetectedTool {
                name: "mocha".to_string(),
                tool_type: ToolType::TestFramework,
                category: ToolCategory::Mocha,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        // Linters
        if command_lower.contains("eslint") {
            return Some(DetectedTool {
                name: "eslint".to_string(),
                tool_type: ToolType::Linter,
                category: ToolCategory::Eslint,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        // Formatters
        if command_lower.contains("prettier") {
            return Some(DetectedTool {
                name: "prettier".to_string(),
                tool_type: ToolType::Formatter,
                category: ToolCategory::Prettier,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        // Dev servers
        if command_lower.contains("webpack-dev-server") || command_lower.contains("webpack serve") {
            return Some(DetectedTool {
                name: "webpack-dev-server".to_string(),
                tool_type: ToolType::DevServer,
                category: ToolCategory::WebpackDevServer,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        if command_lower.contains("vite") && (command_lower.contains("dev") || command_lower.contains("serve")) {
            return Some(DetectedTool {
                name: "vite dev server".to_string(),
                tool_type: ToolType::DevServer,
                category: ToolCategory::ViteDev,
                version: None,
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                detection_method: "package_script".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.9,
            });
        }
        
        None
    }

    /// Detect tools from config files
    fn detect_from_config_files(&self, repo_path: &Path) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        
        use walkdir::WalkDir;
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
            if file_name.starts_with('.') && !file_name.starts_with(".eslintrc") &&
               !file_name.starts_with(".prettierrc") && !file_name.starts_with(".editorconfig") {
                continue;
            }
            
            if path.to_string_lossy().contains("node_modules") ||
               path.to_string_lossy().contains("target") ||
               path.to_string_lossy().contains(".git") {
                continue;
            }
            
            // ESLint config
            if file_name.contains("eslintrc") || file_name == ".eslintrc" ||
               file_name == ".eslintrc.js" || file_name == ".eslintrc.json" ||
               file_name == "eslint.config.js" {
                tools.push(DetectedTool {
                    name: "eslint".to_string(),
                    tool_type: ToolType::Linter,
                    category: ToolCategory::Eslint,
                    version: None,
                    file_path: path.strip_prefix(repo_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                    line_number: None,
                    detection_method: "config_file".to_string(),
                    configuration: HashMap::new(),
                    scripts: Vec::new(),
                    confidence: 0.95,
                });
            }
            
            // Prettier config
            if file_name.contains("prettierrc") || file_name == ".prettierrc" ||
               file_name == ".prettierrc.js" || file_name == ".prettierrc.json" ||
               file_name == "prettier.config.js" {
                tools.push(DetectedTool {
                    name: "prettier".to_string(),
                    tool_type: ToolType::Formatter,
                    category: ToolCategory::Prettier,
                    version: None,
                    file_path: path.strip_prefix(repo_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                    line_number: None,
                    detection_method: "config_file".to_string(),
                    configuration: HashMap::new(),
                    scripts: Vec::new(),
                    confidence: 0.95,
                });
            }
            
            // Webpack config
            if file_name.contains("webpack.config") {
                tools.push(DetectedTool {
                    name: "webpack".to_string(),
                    tool_type: ToolType::BuildTool,
                    category: ToolCategory::Webpack,
                    version: None,
                    file_path: path.strip_prefix(repo_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                    line_number: None,
                    detection_method: "config_file".to_string(),
                    configuration: HashMap::new(),
                    scripts: Vec::new(),
                    confidence: 0.9,
                });
            }
            
            // Vite config
            if file_name.contains("vite.config") {
                tools.push(DetectedTool {
                    name: "vite".to_string(),
                    tool_type: ToolType::BuildTool,
                    category: ToolCategory::Vite,
                    version: None,
                    file_path: path.strip_prefix(repo_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                    line_number: None,
                    detection_method: "config_file".to_string(),
                    configuration: HashMap::new(),
                    scripts: Vec::new(),
                    confidence: 0.9,
                });
            }
            
            // Jest config
            if file_name.contains("jest.config") || file_name == "jest.config.js" ||
               file_name == "jest.config.json" {
                tools.push(DetectedTool {
                    name: "jest".to_string(),
                    tool_type: ToolType::TestFramework,
                    category: ToolCategory::Jest,
                    version: None,
                    file_path: path.strip_prefix(repo_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                    line_number: None,
                    detection_method: "config_file".to_string(),
                    configuration: HashMap::new(),
                    scripts: Vec::new(),
                    confidence: 0.9,
                });
            }
            
            // Makefile
            if file_name == "makefile" || file_name == "makefile.am" || file_name == "makefile.in" {
                tools.push(DetectedTool {
                    name: "make".to_string(),
                    tool_type: ToolType::TaskRunner,
                    category: ToolCategory::Make,
                    version: None,
                    file_path: path.strip_prefix(repo_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                    line_number: None,
                    detection_method: "config_file".to_string(),
                    configuration: HashMap::new(),
                    scripts: Vec::new(),
                    confidence: 0.9,
                });
            }
        }
        
        Ok(tools)
    }

    /// Detect tools from dependencies
    fn detect_from_dependencies(&self, repo_path: &Path) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        let package_json = repo_path.join("package.json");
        
        if !package_json.exists() {
            return Ok(tools);
        }
        
        let content = std::fs::read_to_string(&package_json)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        
        // Check devDependencies
        if let Some(dev_deps) = json.get("devDependencies").and_then(|v| v.as_object()) {
            for (name, version) in dev_deps {
                let name_lower = name.to_lowercase();
                let version_str = version.as_str().unwrap_or("unknown");
                
                // Known tool packages
                if name_lower == "eslint" || name_lower.starts_with("eslint-") {
                    tools.push(DetectedTool {
                        name: name.clone(),
                        tool_type: ToolType::Linter,
                        category: ToolCategory::Eslint,
                        version: Some(version_str.to_string()),
                        file_path: "package.json".to_string(),
                        line_number: None,
                        detection_method: "dependency".to_string(),
                        configuration: HashMap::new(),
                        scripts: Vec::new(),
                        confidence: 0.9,
                    });
                }
                
                if name_lower == "prettier" {
                    tools.push(DetectedTool {
                        name: name.clone(),
                        tool_type: ToolType::Formatter,
                        category: ToolCategory::Prettier,
                        version: Some(version_str.to_string()),
                        file_path: "package.json".to_string(),
                        line_number: None,
                        detection_method: "dependency".to_string(),
                        configuration: HashMap::new(),
                        scripts: Vec::new(),
                        confidence: 0.9,
                    });
                }
                
                if name_lower == "jest" || name_lower.starts_with("jest-") {
                    tools.push(DetectedTool {
                        name: name.clone(),
                        tool_type: ToolType::TestFramework,
                        category: ToolCategory::Jest,
                        version: Some(version_str.to_string()),
                        file_path: "package.json".to_string(),
                        line_number: None,
                        detection_method: "dependency".to_string(),
                        configuration: HashMap::new(),
                        scripts: Vec::new(),
                        confidence: 0.9,
                    });
                }
                
                if name_lower == "webpack" || name_lower.starts_with("webpack-") {
                    tools.push(DetectedTool {
                        name: name.clone(),
                        tool_type: ToolType::BuildTool,
                        category: ToolCategory::Webpack,
                        version: Some(version_str.to_string()),
                        file_path: "package.json".to_string(),
                        line_number: None,
                        detection_method: "dependency".to_string(),
                        configuration: HashMap::new(),
                        scripts: Vec::new(),
                        confidence: 0.9,
                    });
                }
                
                if name_lower == "vite" {
                    tools.push(DetectedTool {
                        name: name.clone(),
                        tool_type: ToolType::BuildTool,
                        category: ToolCategory::Vite,
                        version: Some(version_str.to_string()),
                        file_path: "package.json".to_string(),
                        line_number: None,
                        detection_method: "dependency".to_string(),
                        configuration: HashMap::new(),
                        scripts: Vec::new(),
                        confidence: 0.9,
                    });
                }
            }
        }
        
        Ok(tools)
    }

    /// Detect script files
    fn detect_script_files(&self, repo_path: &Path) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        
        use walkdir::WalkDir;
        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Skip common ignore patterns
            if path.to_string_lossy().contains("node_modules") ||
               path.to_string_lossy().contains("target") ||
               path.to_string_lossy().contains(".git") {
                continue;
            }
            
            // Shell scripts
            if file_name.ends_with(".sh") || file_name.ends_with(".bash") {
                // Check shebang
                if let Ok(content) = std::fs::read_to_string(path) {
                    let first_line = content.lines().next().unwrap_or("");
                    let tool = if first_line.contains("bash") {
                        DetectedTool {
                            name: format!("{} (bash script)", file_name),
                            tool_type: ToolType::ShellScript,
                            category: ToolCategory::Bash,
                            version: None,
                            file_path: path.strip_prefix(repo_path)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                            line_number: None,
                            detection_method: "script_file".to_string(),
                            configuration: HashMap::new(),
                            scripts: Vec::new(),
                            confidence: 0.8,
                        }
                    } else {
                        DetectedTool {
                            name: format!("{} (shell script)", file_name),
                            tool_type: ToolType::ShellScript,
                            category: ToolCategory::Bash,
                            version: None,
                            file_path: path.strip_prefix(repo_path)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                            line_number: None,
                            detection_method: "script_file".to_string(),
                            configuration: HashMap::new(),
                            scripts: Vec::new(),
                            confidence: 0.7,
                        }
                    };
                    tools.push(tool);
                }
            }
        }
        
        Ok(tools)
    }

    /// Detect development environments
    fn detect_dev_environments(&self, repo_path: &Path) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        
        use walkdir::WalkDir;
        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let path = entry.path();
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            // Python virtual environments
            if dir_name == ".venv" || dir_name == "venv" || dir_name == "env" ||
               dir_name == ".virtualenv" {
                tools.push(DetectedTool {
                    name: "Python Virtual Environment".to_string(),
                    tool_type: ToolType::DevEnvironment,
                    category: ToolCategory::Venv,
                    version: None,
                    file_path: path.strip_prefix(repo_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                    line_number: None,
                    detection_method: "directory_pattern".to_string(),
                    configuration: HashMap::new(),
                    scripts: Vec::new(),
                    confidence: 0.9,
                });
            }
        }
        
        // Check for devcontainer
        let devcontainer = repo_path.join(".devcontainer").join("devcontainer.json");
        if devcontainer.exists() {
            tools.push(DetectedTool {
                name: "Dev Container".to_string(),
                tool_type: ToolType::DevEnvironment,
                category: ToolCategory::DevContainer,
                version: None,
                file_path: ".devcontainer/devcontainer.json".to_string(),
                line_number: None,
                detection_method: "config_file".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.95,
            });
        }
        
        // Check for version files
        let nvmrc = repo_path.join(".nvmrc");
        if nvmrc.exists() {
            tools.push(DetectedTool {
                name: "Node Version Manager".to_string(),
                tool_type: ToolType::DevEnvironment,
                category: ToolCategory::Unknown,
                version: None,
                file_path: ".nvmrc".to_string(),
                line_number: None,
                detection_method: "version_file".to_string(),
                configuration: HashMap::new(),
                scripts: Vec::new(),
                confidence: 0.8,
            });
        }
        
        Ok(tools)
    }

    /// Deduplicate tools (same name, category, type)
    fn deduplicate_tools(&self, tools: Vec<DetectedTool>) -> Vec<DetectedTool> {
        let mut seen: std::collections::HashMap<(String, ToolCategory, ToolType), usize> = 
            std::collections::HashMap::new();
        let mut deduplicated: Vec<DetectedTool> = Vec::new();
        
        for tool in tools {
            let key = (tool.name.clone(), tool.category.clone(), tool.tool_type.clone());
            
            if let Some(&existing_idx) = seen.get(&key) {
                // Tool already exists, merge if this one has higher confidence
                let existing: &mut DetectedTool = &mut deduplicated[existing_idx];
                if tool.confidence > existing.confidence {
                    existing.confidence = tool.confidence;
                    existing.line_number = tool.line_number;
                }
                // Merge scripts
                for script in tool.scripts {
                    if !existing.scripts.iter().any(|s| s.name == script.name) {
                        existing.scripts.push(script);
                    }
                }
                // Merge configuration
                for (k, v) in tool.configuration {
                    existing.configuration.insert(k, v);
                }
            } else {
                // New tool, add it
                let idx = deduplicated.len();
                seen.insert(key, idx);
                deduplicated.push(tool);
            }
        }
        
        deduplicated
    }
}

