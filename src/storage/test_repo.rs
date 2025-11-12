use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::analysis::{DetectedTest, TestFramework};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredTest {
    pub id: String,
    pub repository_id: String,
    pub name: String,
    pub test_framework: String,
    pub file_path: String,
    pub line_number: usize,
    pub language: String,
    pub test_type: String,
    pub suite_name: Option<String>,
    pub assertions: String, // JSON array
    pub setup_methods: String, // JSON array
    pub teardown_methods: String, // JSON array
    pub signature: Option<String>,
    pub doc_comment: Option<String>,
    pub parameters: String, // JSON array
    pub return_type: Option<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct TestRepository {
    db: Database,
}

impl TestRepository {
    pub fn new(db: Database) -> Self {
        TestRepository { db }
    }

    pub fn store_tests(&self, repository_id: &str, tests: &[DetectedTest]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let total = tests.len();
        if total > 0 {
            log::info!("Preparing to store {} test(s)...", total);
        }
        
        // Delete existing tests for this repository
        conn.execute(
            "DELETE FROM tests WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new tests with progress logging
        let now = Utc::now().to_rfc3339();
        let batch_size = 500; // Log every 500 tests
        let mut stored = 0;
        
        for test in tests {
            let framework_str = self.framework_to_string(&test.test_framework);
            let assertions_json = serde_json::to_string(&test.assertions)?;
            let setup_json = serde_json::to_string(&test.setup_methods)?;
            let teardown_json = serde_json::to_string(&test.teardown_methods)?;
            let params_json = serde_json::to_string(&test.parameters)?;
            
            conn.execute(
                "INSERT INTO tests (
                    id, repository_id, name, test_framework, file_path,
                    line_number, language, test_type, suite_name, assertions,
                    setup_methods, teardown_methods, signature, doc_comment,
                    parameters, return_type, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    test.id,
                    repository_id,
                    test.name,
                    framework_str,
                    test.file_path,
                    test.line_number as i32,
                    test.language,
                    test.test_type,
                    test.suite_name,
                    assertions_json,
                    setup_json,
                    teardown_json,
                    test.signature,
                    test.doc_comment,
                    params_json,
                    test.return_type,
                    now
                ],
            )?;
            
            stored += 1;
            if stored % batch_size == 0 || stored == total {
                let percent = (stored as f64 / total as f64 * 100.0) as u32;
                log::info!("  Stored {}/{} test(s) ({}%)...", stored, total, percent);
            }
        }
        
        if total > 0 {
            log::info!("✓ Successfully stored all {} test(s)", total);
        }
        
        Ok(())
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<StoredTest>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, test_framework, file_path, line_number,
                    language, test_type, suite_name, assertions, setup_methods,
                    teardown_methods, signature, doc_comment, parameters, return_type, created_at
             FROM tests WHERE repository_id = ?1 ORDER BY file_path, line_number"
        )?;
        
        let tests = stmt.query_map(params![repository_id], |row| {
            Ok(StoredTest {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                name: row.get(2)?,
                test_framework: row.get(3)?,
                file_path: row.get(4)?,
                line_number: row.get::<_, i32>(5)? as usize,
                language: row.get(6)?,
                test_type: row.get(7)?,
                suite_name: row.get(8)?,
                assertions: row.get(9)?,
                setup_methods: row.get(10)?,
                teardown_methods: row.get(11)?,
                signature: row.get(12)?,
                doc_comment: row.get(13)?,
                parameters: row.get(14)?,
                return_type: row.get(15)?,
                created_at: row.get(16)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(tests)
    }

    pub fn get_by_framework(&self, repository_id: &str, framework: &str) -> Result<Vec<StoredTest>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, test_framework, file_path, line_number,
                    language, test_type, suite_name, assertions, setup_methods,
                    teardown_methods, signature, doc_comment, parameters, return_type, created_at
             FROM tests WHERE repository_id = ?1 AND test_framework = ?2 ORDER BY file_path, line_number"
        )?;
        
        let tests = stmt.query_map(params![repository_id, framework], |row| {
            Ok(StoredTest {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                name: row.get(2)?,
                test_framework: row.get(3)?,
                file_path: row.get(4)?,
                line_number: row.get::<_, i32>(5)? as usize,
                language: row.get(6)?,
                test_type: row.get(7)?,
                suite_name: row.get(8)?,
                assertions: row.get(9)?,
                setup_methods: row.get(10)?,
                teardown_methods: row.get(11)?,
                signature: row.get(12)?,
                doc_comment: row.get(13)?,
                parameters: row.get(14)?,
                return_type: row.get(15)?,
                created_at: row.get(16)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(tests)
    }

    fn framework_to_string(&self, framework: &TestFramework) -> String {
        match framework {
            TestFramework::Jest => "jest",
            TestFramework::Mocha => "mocha",
            TestFramework::Vitest => "vitest",
            TestFramework::Jasmine => "jasmine",
            TestFramework::Ava => "ava",
            TestFramework::Tap => "tap",
            TestFramework::Pytest => "pytest",
            TestFramework::Unittest => "unittest",
            TestFramework::Nose => "nose",
            TestFramework::Doctest => "doctest",
            TestFramework::CargoTest => "cargo-test",
            TestFramework::GoTest => "go-test",
            TestFramework::JUnit => "junit",
            TestFramework::TestNG => "testng",
            TestFramework::XCTest => "xctest",
            TestFramework::Quick => "quick",
            TestFramework::Nimble => "nimble",
            TestFramework::RSpec => "rspec",
            TestFramework::Minitest => "minitest",
            TestFramework::Unknown => "unknown",
        }.to_string()
    }
}

