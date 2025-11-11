use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::analysis::{CodeElement, CodeCall, CodeElementType};

#[derive(Clone)]
pub struct CodeElementRepository {
    db: Database,
}

impl CodeElementRepository {
    pub fn new(db: Database) -> Self {
        CodeElementRepository { db }
    }

    pub fn store_elements(&self, repository_id: &str, elements: &[CodeElement]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing elements for this repository
        conn.execute(
            "DELETE FROM code_elements WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new elements
        let now = Utc::now();
        for element in elements {
            let element_type_str = self.element_type_to_string(&element.element_type);
            let parameters_json = serde_json::to_string(&element.parameters)?;
            
            conn.execute(
                "INSERT INTO code_elements 
                 (id, repository_id, name, element_type, file_path, line_number, language, signature, doc_comment, visibility, parameters, return_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    element.id,
                    repository_id,
                    element.name,
                    element_type_str,
                    element.file_path,
                    element.line_number as i32,
                    element.language,
                    element.signature,
                    element.doc_comment,
                    element.visibility,
                    parameters_json,
                    element.return_type,
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn store_calls(&self, repository_id: &str, calls: &[CodeCall]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing calls for this repository
        conn.execute(
            "DELETE FROM code_calls WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new calls
        let now = Utc::now();
        for call in calls {
            let id = Uuid::new_v4().to_string();
            
            conn.execute(
                "INSERT INTO code_calls 
                 (id, repository_id, caller_id, callee_id, call_type, line_number, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    repository_id,
                    call.caller_id,
                    call.callee_id,
                    call.call_type,
                    call.line_number as i32,
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<CodeElement>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, element_type, file_path, line_number, language, signature, doc_comment, visibility, parameters, return_type
             FROM code_elements WHERE repository_id = ?1 ORDER BY file_path, line_number"
        )?;
        
        let elements = stmt.query_map(params![repository_id], |row| {
            let element_type_str: String = row.get(2)?;
            let parameters_json: String = row.get(9)?;
            let parameters: Vec<String> = serde_json::from_str(&parameters_json).unwrap_or_default();

            Ok(CodeElement {
                id: row.get(0)?,
                name: row.get(1)?,
                element_type: self.string_to_element_type(&element_type_str),
                file_path: row.get(3)?,
                line_number: row.get::<_, i32>(4)? as usize,
                language: row.get(5)?,
                signature: row.get(6)?,
                doc_comment: row.get(7)?,
                visibility: row.get(8)?,
                parameters,
                return_type: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(elements)
    }

    pub fn get_by_type(&self, repository_id: &str, element_type: &str) -> Result<Vec<CodeElement>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, element_type, file_path, line_number, language, signature, doc_comment, visibility, parameters, return_type
             FROM code_elements WHERE repository_id = ?1 AND element_type = ?2 ORDER BY file_path, line_number"
        )?;
        
        let elements = stmt.query_map(params![repository_id, element_type], |row| {
            let element_type_str: String = row.get(2)?;
            let parameters_json: String = row.get(9)?;
            let parameters: Vec<String> = serde_json::from_str(&parameters_json).unwrap_or_default();

            Ok(CodeElement {
                id: row.get(0)?,
                name: row.get(1)?,
                element_type: self.string_to_element_type(&element_type_str),
                file_path: row.get(3)?,
                line_number: row.get::<_, i32>(4)? as usize,
                language: row.get(5)?,
                signature: row.get(6)?,
                doc_comment: row.get(7)?,
                visibility: row.get(8)?,
                parameters,
                return_type: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(elements)
    }

    pub fn get_calls(&self, repository_id: &str) -> Result<Vec<CodeCall>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT caller_id, callee_id, call_type, line_number
             FROM code_calls WHERE repository_id = ?1 ORDER BY line_number"
        )?;
        
        let calls = stmt.query_map(params![repository_id], |row| {
            Ok(CodeCall {
                caller_id: row.get(0)?,
                callee_id: row.get(1)?,
                call_type: row.get(2)?,
                line_number: row.get::<_, i32>(3)? as usize,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(calls)
    }

    fn element_type_to_string(&self, element_type: &CodeElementType) -> String {
        match element_type {
            CodeElementType::Function => "function",
            CodeElementType::Class => "class",
            CodeElementType::Module => "module",
            CodeElementType::Interface => "interface",
            CodeElementType::Struct => "struct",
            CodeElementType::Enum => "enum",
            CodeElementType::Method => "method",
            CodeElementType::Constant => "constant",
            CodeElementType::Variable => "variable",
        }.to_string()
    }

    fn string_to_element_type(&self, s: &str) -> CodeElementType {
        match s {
            "function" => CodeElementType::Function,
            "class" => CodeElementType::Class,
            "module" => CodeElementType::Module,
            "interface" => CodeElementType::Interface,
            "struct" => CodeElementType::Struct,
            "enum" => CodeElementType::Enum,
            "method" => CodeElementType::Method,
            "constant" => CodeElementType::Constant,
            "variable" => CodeElementType::Variable,
            _ => CodeElementType::Function,
        }
    }
}

