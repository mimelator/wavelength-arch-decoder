use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::analysis::{DetectedTool, ToolType, ToolCategory, ToolScript};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredTool {
    pub id: String,
    pub repository_id: String,
    pub name: String,
    pub tool_type: String,
    pub category: String,
    pub version: Option<String>,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub detection_method: String,
    pub configuration: String,
    pub confidence: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredToolScript {
    pub id: String,
    pub tool_id: String,
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct ToolRepository {
    db: Database,
}

impl ToolRepository {
    pub fn new(db: Database) -> Self {
        ToolRepository { db }
    }

    pub fn store_tools(&self, repository_id: &str, tools: &[DetectedTool]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing tools for this repository
        conn.execute(
            "DELETE FROM tools WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new tools
        let now = Utc::now().to_rfc3339();
        for tool in tools {
            let id = Uuid::new_v4().to_string();
            let tool_type_str = self.tool_type_to_string(&tool.tool_type);
            let category_str = self.category_to_string(&tool.category);
            let config_json = serde_json::to_string(&tool.configuration)?;
            
            conn.execute(
                "INSERT INTO tools (
                    id, repository_id, name, tool_type, category, version,
                    file_path, line_number, detection_method, configuration,
                    confidence, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    id,
                    repository_id,
                    tool.name,
                    tool_type_str,
                    category_str,
                    tool.version,
                    tool.file_path,
                    tool.line_number,
                    tool.detection_method,
                    config_json,
                    tool.confidence,
                    now
                ],
            )?;
            
            // Store tool scripts
            for script in &tool.scripts {
                let script_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO tool_scripts (
                        id, tool_id, name, command, description,
                        file_path, line_number, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        script_id,
                        id,
                        script.name,
                        script.command,
                        script.description,
                        script.file_path,
                        script.line_number,
                        now
                    ],
                )?;
            }
        }
        
        Ok(())
    }

    pub fn get_tools_by_repository(&self, repository_id: &str) -> Result<Vec<StoredTool>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, tool_type, category, version,
                    file_path, line_number, detection_method, configuration,
                    confidence, created_at
             FROM tools
             WHERE repository_id = ?1
             ORDER BY name"
        )?;
        
        let tools = stmt.query_map(params![repository_id], |row| {
            Ok(StoredTool {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                name: row.get(2)?,
                tool_type: row.get(3)?,
                category: row.get(4)?,
                version: row.get(5)?,
                file_path: row.get(6)?,
                line_number: row.get(7)?,
                detection_method: row.get(8)?,
                configuration: row.get(9)?,
                confidence: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(tools)
    }

    pub fn get_tool_scripts(&self, repository_id: &str, tool_id: &str) -> Result<Vec<StoredToolScript>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Join with tools table to ensure tool belongs to repository
        let mut stmt = conn.prepare(
            "SELECT ts.id, ts.tool_id, ts.name, ts.command, ts.description,
                    ts.file_path, ts.line_number, ts.created_at
             FROM tool_scripts ts
             INNER JOIN tools t ON ts.tool_id = t.id
             WHERE t.repository_id = ?1 AND ts.tool_id = ?2
             ORDER BY ts.name"
        )?;
        
        let scripts = stmt.query_map(params![repository_id, tool_id], |row| {
            Ok(StoredToolScript {
                id: row.get(0)?,
                tool_id: row.get(1)?,
                name: row.get(2)?,
                command: row.get(3)?,
                description: row.get(4)?,
                file_path: row.get(5)?,
                line_number: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(scripts)
    }

    fn tool_type_to_string(&self, tool_type: &ToolType) -> String {
        match tool_type {
            ToolType::BuildTool => "BuildTool".to_string(),
            ToolType::TestFramework => "TestFramework".to_string(),
            ToolType::Linter => "Linter".to_string(),
            ToolType::Formatter => "Formatter".to_string(),
            ToolType::DevServer => "DevServer".to_string(),
            ToolType::CodeGenerator => "CodeGenerator".to_string(),
            ToolType::Debugger => "Debugger".to_string(),
            ToolType::TaskRunner => "TaskRunner".to_string(),
            ToolType::ShellScript => "ShellScript".to_string(),
            ToolType::DevEnvironment => "DevEnvironment".to_string(),
            ToolType::Sdk => "Sdk".to_string(),
            ToolType::Other => "Other".to_string(),
        }
    }

    fn category_to_string(&self, category: &ToolCategory) -> String {
        match category {
            ToolCategory::Webpack => "Webpack".to_string(),
            ToolCategory::Vite => "Vite".to_string(),
            ToolCategory::Rollup => "Rollup".to_string(),
            ToolCategory::Esbuild => "Esbuild".to_string(),
            ToolCategory::Tsc => "Tsc".to_string(),
            ToolCategory::Cargo => "Cargo".to_string(),
            ToolCategory::GoBuild => "GoBuild".to_string(),
            ToolCategory::Maven => "Maven".to_string(),
            ToolCategory::Gradle => "Gradle".to_string(),
            ToolCategory::Jest => "Jest".to_string(),
            ToolCategory::Mocha => "Mocha".to_string(),
            ToolCategory::Pytest => "Pytest".to_string(),
            ToolCategory::Unittest => "Unittest".to_string(),
            ToolCategory::CargoTest => "CargoTest".to_string(),
            ToolCategory::GoTest => "GoTest".to_string(),
            ToolCategory::Eslint => "Eslint".to_string(),
            ToolCategory::Pylint => "Pylint".to_string(),
            ToolCategory::Rustfmt => "Rustfmt".to_string(),
            ToolCategory::Gofmt => "Gofmt".to_string(),
            ToolCategory::Prettier => "Prettier".to_string(),
            ToolCategory::Black => "Black".to_string(),
            ToolCategory::WebpackDevServer => "WebpackDevServer".to_string(),
            ToolCategory::ViteDev => "ViteDev".to_string(),
            ToolCategory::Nodemon => "Nodemon".to_string(),
            ToolCategory::NpmScripts => "NpmScripts".to_string(),
            ToolCategory::Make => "Make".to_string(),
            ToolCategory::Just => "Just".to_string(),
            ToolCategory::Task => "Task".to_string(),
            ToolCategory::Bash => "Bash".to_string(),
            ToolCategory::Zsh => "Zsh".to_string(),
            ToolCategory::Fish => "Fish".to_string(),
            ToolCategory::Venv => "Venv".to_string(),
            ToolCategory::Conda => "Conda".to_string(),
            ToolCategory::DockerDev => "DockerDev".to_string(),
            ToolCategory::DevContainer => "DevContainer".to_string(),
            ToolCategory::AwsCdk => "AwsCdk".to_string(),
            ToolCategory::Terraform => "Terraform".to_string(),
            ToolCategory::Serverless => "Serverless".to_string(),
            ToolCategory::FirebaseCli => "FirebaseCli".to_string(),
            ToolCategory::Unknown => "Unknown".to_string(),
        }
    }
}

