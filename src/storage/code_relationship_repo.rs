use anyhow::Result;
use chrono::Utc;
use crate::storage::Database;
use crate::analysis::{CodeRelationship, RelationshipTargetType};
use rusqlite::params;

#[derive(Clone)]
pub struct CodeRelationshipRepository {
    db: Database,
}

impl CodeRelationshipRepository {
    pub fn new(db: Database) -> Self {
        CodeRelationshipRepository { db }
    }

    pub fn store_relationships(&self, repository_id: &str, relationships: &[CodeRelationship]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();

        let total = relationships.len();
        if total > 0 {
            log::info!("Preparing to store {} code relationships...", total);
        }

        // Delete existing relationships for this repository
        conn.execute(
            "DELETE FROM code_relationships WHERE repository_id = ?1",
            params![repository_id],
        )?;

        // Insert new relationships with progress logging
        let now = Utc::now();
        let batch_size = 1000; // Log every 1000 relationships
        let mut stored = 0;
        
        for rel in relationships {
            let target_type_str = match rel.target_type {
                RelationshipTargetType::Service => "service",
                RelationshipTargetType::Dependency => "dependency",
            };

            conn.execute(
                "INSERT INTO code_relationships 
                 (id, repository_id, code_element_id, target_type, target_id, relationship_type, confidence, evidence, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    rel.id,
                    repository_id,
                    rel.code_element_id,
                    target_type_str,
                    rel.target_id,
                    rel.relationship_type,
                    rel.confidence,
                    rel.evidence,
                    now.to_rfc3339()
                ],
            )?;
            
            stored += 1;
            if stored % batch_size == 0 || stored == total {
                let percent = (stored as f64 / total as f64 * 100.0) as u32;
                log::info!("  Stored {}/{} relationships ({}%)...", stored, total, percent);
            }
        }

        if total > 0 {
            log::info!("✓ Successfully stored all {} code relationships", total);
        }
        Ok(())
    }

    pub fn get_by_code_element(&self, repository_id: &str, code_element_id: &str) -> Result<Vec<CodeRelationship>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, code_element_id, target_type, target_id, relationship_type, confidence, evidence
             FROM code_relationships WHERE repository_id = ?1 AND code_element_id = ?2"
        )?;
        
        let relationships = stmt.query_map(params![repository_id, code_element_id], |row| {
            let target_type_str: String = row.get(2)?;
            let target_type = match target_type_str.as_str() {
                "service" => RelationshipTargetType::Service,
                "dependency" => RelationshipTargetType::Dependency,
                _ => RelationshipTargetType::Service, // Default
            };

            Ok(CodeRelationship {
                id: row.get(0)?,
                code_element_id: row.get(1)?,
                target_type,
                target_id: row.get(3)?,
                relationship_type: row.get(4)?,
                confidence: row.get(5)?,
                evidence: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(relationships)
    }

    pub fn get_by_target(&self, repository_id: &str, target_type: &str, target_id: &str) -> Result<Vec<CodeRelationship>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, code_element_id, target_type, target_id, relationship_type, confidence, evidence
             FROM code_relationships WHERE repository_id = ?1 AND target_type = ?2 AND target_id = ?3"
        )?;
        
        let relationships = stmt.query_map(params![repository_id, target_type, target_id], |row| {
            let target_type_str: String = row.get(2)?;
            let target_type_enum = match target_type_str.as_str() {
                "service" => RelationshipTargetType::Service,
                "dependency" => RelationshipTargetType::Dependency,
                _ => RelationshipTargetType::Service,
            };

            Ok(CodeRelationship {
                id: row.get(0)?,
                code_element_id: row.get(1)?,
                target_type: target_type_enum,
                target_id: row.get(3)?,
                relationship_type: row.get(4)?,
                confidence: row.get(5)?,
                evidence: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(relationships)
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<CodeRelationship>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, code_element_id, target_type, target_id, relationship_type, confidence, evidence
             FROM code_relationships WHERE repository_id = ?1"
        )?;

        let relationships = stmt.query_map(params![repository_id], |row| {
            let target_type_str: String = row.get(2)?;
            let target_type_enum = match target_type_str.as_str() {
                "service" => RelationshipTargetType::Service,
                "dependency" => RelationshipTargetType::Dependency,
                _ => RelationshipTargetType::Service,
            };

            Ok(CodeRelationship {
                id: row.get(0)?,
                code_element_id: row.get(1)?,
                target_type: target_type_enum,
                target_id: row.get(3)?,
                relationship_type: row.get(4)?,
                confidence: row.get(5)?,
                evidence: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(relationships)
    }
}

