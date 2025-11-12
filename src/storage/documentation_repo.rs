use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use crate::storage::Database;
use crate::analysis::documentation::DocumentationFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDocumentation {
    pub id: String,
    pub repository_id: String,
    pub file_path: String,
    pub file_name: String,
    pub doc_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content_preview: String,
    pub word_count: usize,
    pub line_count: usize,
    pub has_code_examples: bool,
    pub has_api_references: bool,
    pub has_diagrams: bool,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Clone)]
pub struct DocumentationRepository {
    db: Database,
}

impl DocumentationRepository {
    pub fn new(db: Database) -> Self {
        DocumentationRepository { db }
    }

    pub fn store_documentation(&self, docs: &[DocumentationFile]) -> Result<()> {
        let conn = self.db.conn.lock().unwrap();
        let total = docs.len();
        
        if total > 0 {
            log::info!("Preparing to store {} documentation files...", total);
        }
        
        // Delete existing documentation for this repository first to prevent duplicates
        if let Some(first_doc) = docs.first() {
            conn.execute(
                "DELETE FROM documentation WHERE repository_id = ?1",
                params![first_doc.repository_id],
            )?;
        }
        
        let mut stored = 0;
        let batch_size = 100;
        
        for doc in docs {
            conn.execute(
                "INSERT INTO documentation (
                    id, repository_id, file_path, file_name, doc_type, title, description,
                    content_preview, word_count, line_count, has_code_examples,
                    has_api_references, has_diagrams, metadata, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    doc.id,
                    doc.repository_id,
                    doc.file_path,
                    doc.file_name,
                    format!("{:?}", doc.doc_type),
                    doc.title,
                    doc.description,
                    doc.content_preview,
                    doc.word_count as i64,
                    doc.line_count as i64,
                    doc.has_code_examples as i32,
                    doc.has_api_references as i32,
                    doc.has_diagrams as i32,
                    serde_json::to_string(&doc.metadata)?,
                    chrono::Utc::now().to_rfc3339(),
                ],
            )?;
            
            stored += 1;
            if stored % batch_size == 0 || stored == total {
                let percent = (stored as f64 / total as f64 * 100.0) as u32;
                log::info!("  Stored {}/{} documentation files ({}%)...", stored, total, percent);
            }
        }
        
        if total > 0 {
            log::info!("✓ Successfully stored all {} documentation files", total);
        }
        
        Ok(())
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<StoredDocumentation>> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, file_path, file_name, doc_type, title, description,
             content_preview, word_count, line_count, has_code_examples,
             has_api_references, has_diagrams, metadata, created_at
             FROM documentation
             WHERE repository_id = ?1
             ORDER BY file_path"
        )?;
        
        let docs = stmt.query_map(params![repository_id], |row| {
            Ok(StoredDocumentation {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                file_path: row.get(2)?,
                file_name: row.get(3)?,
                doc_type: row.get(4)?,
                title: row.get(5)?,
                description: row.get(6)?,
                content_preview: row.get(7)?,
                word_count: row.get::<_, i64>(8)? as usize,
                line_count: row.get::<_, i64>(9)? as usize,
                has_code_examples: row.get::<_, i32>(10)? != 0,
                has_api_references: row.get::<_, i32>(11)? != 0,
                has_diagrams: row.get::<_, i32>(12)? != 0,
                metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or(serde_json::json!({})),
                created_at: row.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(docs)
    }

    pub fn get_by_type(&self, repository_id: &str, doc_type: &str) -> Result<Vec<StoredDocumentation>> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, file_path, file_name, doc_type, title, description,
             content_preview, word_count, line_count, has_code_examples,
             has_api_references, has_diagrams, metadata, created_at
             FROM documentation
             WHERE repository_id = ?1 AND doc_type = ?2
             ORDER BY file_path"
        )?;
        
        let docs = stmt.query_map(params![repository_id, doc_type], |row| {
            Ok(StoredDocumentation {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                file_path: row.get(2)?,
                file_name: row.get(3)?,
                doc_type: row.get(4)?,
                title: row.get(5)?,
                description: row.get(6)?,
                content_preview: row.get(7)?,
                word_count: row.get::<_, i64>(8)? as usize,
                line_count: row.get::<_, i64>(9)? as usize,
                has_code_examples: row.get::<_, i32>(10)? != 0,
                has_api_references: row.get::<_, i32>(11)? != 0,
                has_diagrams: row.get::<_, i32>(12)? != 0,
                metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or(serde_json::json!({})),
                created_at: row.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(docs)
    }

    pub fn search(&self, repository_id: &str, query: &str) -> Result<Vec<StoredDocumentation>> {
        let conn = self.db.conn.lock().unwrap();
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, file_path, file_name, doc_type, title, description,
             content_preview, word_count, line_count, has_code_examples,
             has_api_references, has_diagrams, metadata, created_at
             FROM documentation
             WHERE repository_id = ?1 AND (
                 file_name LIKE ?2 OR
                 title LIKE ?2 OR
                 description LIKE ?2 OR
                 content_preview LIKE ?2
             )
             ORDER BY file_path"
        )?;
        
        let docs = stmt.query_map(params![repository_id, search_pattern], |row| {
            Ok(StoredDocumentation {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                file_path: row.get(2)?,
                file_name: row.get(3)?,
                doc_type: row.get(4)?,
                title: row.get(5)?,
                description: row.get(6)?,
                content_preview: row.get(7)?,
                word_count: row.get::<_, i64>(8)? as usize,
                line_count: row.get::<_, i64>(9)? as usize,
                has_code_examples: row.get::<_, i32>(10)? != 0,
                has_api_references: row.get::<_, i32>(11)? != 0,
                has_diagrams: row.get::<_, i32>(12)? != 0,
                metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or(serde_json::json!({})),
                created_at: row.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(docs)
    }

    pub fn delete_by_repository(&self, repository_id: &str) -> Result<()> {
        let conn = self.db.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM documentation WHERE repository_id = ?1",
            params![repository_id],
        )?;
        Ok(())
    }
}

