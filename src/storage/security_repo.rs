use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::security::{SecurityEntity, SecurityRelationship, SecurityVulnerability, SecurityEntityType, VulnerabilitySeverity};
use serde_json::Value;

#[derive(Clone)]
pub struct SecurityRepository {
    db: Database,
}

impl SecurityRepository {
    pub fn new(db: Database) -> Self {
        SecurityRepository { db }
    }

    pub fn store_entities(&self, repository_id: &str, entities: &[SecurityEntity]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing entities for this repository
        conn.execute(
            "DELETE FROM security_entities WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new entities
        let now = Utc::now();
        for entity in entities {
            let entity_type_str = self.entity_type_to_string(&entity.entity_type);
            let config_json = serde_json::to_string(&entity.configuration)?;
            
            conn.execute(
                "INSERT INTO security_entities 
                 (id, repository_id, entity_type, name, provider, configuration, file_path, line_number, arn, region, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    entity.id,
                    repository_id,
                    entity_type_str,
                    entity.name,
                    entity.provider,
                    config_json,
                    entity.file_path,
                    entity.line_number.map(|n| n as i32),
                    entity.arn,
                    entity.region,
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn store_relationships(&self, repository_id: &str, relationships: &[SecurityRelationship]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing relationships for this repository
        conn.execute(
            "DELETE FROM security_relationships WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new relationships
        let now = Utc::now();
        for relationship in relationships {
            let id = Uuid::new_v4().to_string();
            let permissions_json = serde_json::to_string(&relationship.permissions)?;
            
            conn.execute(
                "INSERT INTO security_relationships 
                 (id, repository_id, source_entity_id, target_entity_id, relationship_type, permissions, condition, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    repository_id,
                    relationship.source_entity_id,
                    relationship.target_entity_id,
                    relationship.relationship_type,
                    permissions_json,
                    relationship.condition,
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn store_vulnerabilities(&self, repository_id: &str, vulnerabilities: &[SecurityVulnerability]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing vulnerabilities for this repository
        conn.execute(
            "DELETE FROM security_vulnerabilities WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new vulnerabilities
        let now = Utc::now();
        for vuln in vulnerabilities {
            let severity_str = self.severity_to_string(&vuln.severity);
            
            conn.execute(
                "INSERT INTO security_vulnerabilities 
                 (id, repository_id, entity_id, vulnerability_type, severity, description, recommendation, file_path, line_number, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    vuln.id,
                    repository_id,
                    vuln.entity_id,
                    vuln.vulnerability_type,
                    severity_str,
                    vuln.description,
                    vuln.recommendation,
                    vuln.file_path,
                    vuln.line_number.map(|n| n as i32),
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn get_entities(&self, repository_id: &str) -> Result<Vec<SecurityEntity>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, entity_type, name, provider, configuration, file_path, line_number, arn, region
             FROM security_entities WHERE repository_id = ?1 ORDER BY entity_type, name"
        )?;
        
        let entities = stmt.query_map(params![repository_id], |row| {
            let entity_type_str: String = row.get(1)?;
            let config_json: String = row.get(4)?;
            let configuration: HashMap<String, Value> = serde_json::from_str(&config_json).unwrap_or_default();

            Ok(SecurityEntity {
                id: row.get(0)?,
                entity_type: self.string_to_entity_type(&entity_type_str),
                name: row.get(2)?,
                provider: row.get(3)?,
                configuration,
                file_path: row.get(5)?,
                line_number: match row.get::<_, Option<i32>>(6)? {
                    Some(n) => Some(n as usize),
                    None => None,
                },
                arn: row.get(7)?,
                region: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    pub fn get_by_type(&self, repository_id: &str, entity_type: &str) -> Result<Vec<SecurityEntity>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, entity_type, name, provider, configuration, file_path, line_number, arn, region
             FROM security_entities WHERE repository_id = ?1 AND entity_type = ?2 ORDER BY name"
        )?;
        
        let entities = stmt.query_map(params![repository_id, entity_type], |row| {
            let entity_type_str: String = row.get(1)?;
            let config_json: String = row.get(4)?;
            let configuration: HashMap<String, Value> = serde_json::from_str(&config_json).unwrap_or_default();

            Ok(SecurityEntity {
                id: row.get(0)?,
                entity_type: self.string_to_entity_type(&entity_type_str),
                name: row.get(2)?,
                provider: row.get(3)?,
                configuration,
                file_path: row.get(5)?,
                line_number: match row.get::<_, Option<i32>>(6)? {
                    Some(n) => Some(n as usize),
                    None => None,
                },
                arn: row.get(7)?,
                region: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    pub fn get_relationships(&self, repository_id: &str) -> Result<Vec<SecurityRelationship>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT source_entity_id, target_entity_id, relationship_type, permissions, condition
             FROM security_relationships WHERE repository_id = ?1 ORDER BY relationship_type"
        )?;
        
        let relationships = stmt.query_map(params![repository_id], |row| {
            let permissions_json: String = row.get(3)?;
            let permissions: Vec<String> = serde_json::from_str(&permissions_json).unwrap_or_default();

            Ok(SecurityRelationship {
                source_entity_id: row.get(0)?,
                target_entity_id: row.get(1)?,
                relationship_type: row.get(2)?,
                permissions,
                condition: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(relationships)
    }

    pub fn get_vulnerabilities(&self, repository_id: &str) -> Result<Vec<SecurityVulnerability>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, entity_id, vulnerability_type, severity, description, recommendation, file_path, line_number
             FROM security_vulnerabilities WHERE repository_id = ?1 ORDER BY 
             CASE severity 
                 WHEN 'Critical' THEN 1
                 WHEN 'High' THEN 2
                 WHEN 'Medium' THEN 3
                 WHEN 'Low' THEN 4
                 ELSE 5
             END"
        )?;
        
        let vulnerabilities = stmt.query_map(params![repository_id], |row| {
            let severity_str: String = row.get(3)?;

            Ok(SecurityVulnerability {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                vulnerability_type: row.get(2)?,
                severity: self.string_to_severity(&severity_str),
                description: row.get(4)?,
                recommendation: row.get(5)?,
                file_path: row.get(6)?,
                line_number: match row.get::<_, Option<i32>>(7)? {
                    Some(n) => Some(n as usize),
                    None => None,
                },
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(vulnerabilities)
    }

    pub fn get_vulnerabilities_by_severity(&self, repository_id: &str, severity: &str) -> Result<Vec<SecurityVulnerability>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, entity_id, vulnerability_type, severity, description, recommendation, file_path, line_number
             FROM security_vulnerabilities WHERE repository_id = ?1 AND severity = ?2 ORDER BY vulnerability_type"
        )?;
        
        let vulnerabilities = stmt.query_map(params![repository_id, severity], |row| {
            let severity_str: String = row.get(3)?;

            Ok(SecurityVulnerability {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                vulnerability_type: row.get(2)?,
                severity: self.string_to_severity(&severity_str),
                description: row.get(4)?,
                recommendation: row.get(5)?,
                file_path: row.get(6)?,
                line_number: row.get::<_, Option<i32>>(7).map(|n| n as usize),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(vulnerabilities)
    }

    fn entity_type_to_string(&self, entity_type: &SecurityEntityType) -> String {
        match entity_type {
            SecurityEntityType::IamRole => "iam_role",
            SecurityEntityType::IamPolicy => "iam_policy",
            SecurityEntityType::LambdaFunction => "lambda_function",
            SecurityEntityType::S3Bucket => "s3_bucket",
            SecurityEntityType::SecurityGroup => "security_group",
            SecurityEntityType::Vpc => "vpc",
            SecurityEntityType::Subnet => "subnet",
            SecurityEntityType::Ec2Instance => "ec2_instance",
            SecurityEntityType::RdsInstance => "rds_instance",
            SecurityEntityType::ApiGateway => "api_gateway",
        }.to_string()
    }

    fn string_to_entity_type(&self, s: &str) -> SecurityEntityType {
        match s {
            "iam_role" => SecurityEntityType::IamRole,
            "iam_policy" => SecurityEntityType::IamPolicy,
            "lambda_function" => SecurityEntityType::LambdaFunction,
            "s3_bucket" => SecurityEntityType::S3Bucket,
            "security_group" => SecurityEntityType::SecurityGroup,
            "vpc" => SecurityEntityType::Vpc,
            "subnet" => SecurityEntityType::Subnet,
            "ec2_instance" => SecurityEntityType::Ec2Instance,
            "rds_instance" => SecurityEntityType::RdsInstance,
            "api_gateway" => SecurityEntityType::ApiGateway,
            _ => SecurityEntityType::IamRole,
        }
    }

    fn severity_to_string(&self, severity: &VulnerabilitySeverity) -> String {
        match severity {
            VulnerabilitySeverity::Critical => "Critical",
            VulnerabilitySeverity::High => "High",
            VulnerabilitySeverity::Medium => "Medium",
            VulnerabilitySeverity::Low => "Low",
            VulnerabilitySeverity::Info => "Info",
        }.to_string()
    }

    fn string_to_severity(&self, s: &str) -> VulnerabilitySeverity {
        match s {
            "Critical" => VulnerabilitySeverity::Critical,
            "High" => VulnerabilitySeverity::High,
            "Medium" => VulnerabilitySeverity::Medium,
            "Low" => VulnerabilitySeverity::Low,
            "Info" => VulnerabilitySeverity::Info,
            _ => VulnerabilitySeverity::Info,
        }
    }
}

use std::collections::HashMap;

