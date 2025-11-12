use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use uuid::Uuid;
use crate::analysis::utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TestFramework {
    Jest,
    Mocha,
    Vitest,
    Jasmine,
    Ava,
    Tap,
    Pytest,
    Unittest,
    Nose,
    Doctest,
    CargoTest,
    GoTest,
    JUnit,
    TestNG,
    XCTest,
    Quick,
    Nimble,
    RSpec,
    Minitest,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTest {
    pub id: String,
    pub name: String,
    pub test_framework: TestFramework,
    pub file_path: String,
    pub line_number: usize,
    pub language: String,
    pub test_type: String, // "unit", "integration", "e2e", "snapshot", etc.
    pub suite_name: Option<String>, // describe/it block name, class name, etc.
    pub assertions: Vec<String>, // expect, assert, etc. calls found
    pub setup_methods: Vec<String>, // beforeEach, setUp, etc.
    pub teardown_methods: Vec<String>, // afterEach, tearDown, etc.
    pub signature: Option<String>,
    pub doc_comment: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
}

pub struct TestDetector;

impl TestDetector {
    pub fn new() -> Self {
        TestDetector
    }

    /// Detect tests in a repository
    pub fn detect_tests(&self, repo_path: &Path) -> Result<Vec<DetectedTest>> {
        let mut tests = Vec::new();

        // Walk through code files looking for test files
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
            let path_str = path.to_string_lossy().to_lowercase();
            if utils::should_skip_file(&file_name, &path_str) {
                continue;
            }

            // Check if file is a test file
            let is_test_file = self.is_test_file(&file_name, &path_str);
            if !is_test_file {
                continue;
            }

            // Normalize path
            let normalized_path = path.strip_prefix(repo_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());

            // Determine language
            let language = utils::detect_language(path);
            if language.is_none() {
                continue;
            }

            // Analyze test file
            if let Ok(content) = std::fs::read_to_string(path) {
                // Skip minified/compiled code
                if utils::is_minified_or_compiled(&content, &normalized_path) {
                    continue;
                }

                match language.as_deref() {
                    Some("javascript") | Some("typescript") => {
                        tests.extend(self.analyze_js_ts_tests(&content, &normalized_path, &language.unwrap())?);
                    }
                    Some("python") => {
                        tests.extend(self.analyze_python_tests(&content, &normalized_path)?);
                    }
                    Some("rust") => {
                        tests.extend(self.analyze_rust_tests(&content, &normalized_path)?);
                    }
                    Some("go") => {
                        tests.extend(self.analyze_go_tests(&content, &normalized_path)?);
                    }
                    Some("java") => {
                        tests.extend(self.analyze_java_tests(&content, &normalized_path)?);
                    }
                    Some("swift") => {
                        tests.extend(self.analyze_swift_tests(&content, &normalized_path)?);
                    }
                    _ => {}
                }
            }
        }

        Ok(tests)
    }

    /// Check if a file is likely a test file
    fn is_test_file(&self, file_name: &str, path_str: &str) -> bool {
        // Common test file patterns
        file_name.contains("test") ||
        file_name.contains("spec") ||
        file_name.ends_with(".test.js") ||
        file_name.ends_with(".test.ts") ||
        file_name.ends_with(".test.jsx") ||
        file_name.ends_with(".test.tsx") ||
        file_name.ends_with(".spec.js") ||
        file_name.ends_with(".spec.ts") ||
        file_name.ends_with(".spec.jsx") ||
        file_name.ends_with(".spec.tsx") ||
        file_name.ends_with("_test.py") ||
        file_name.ends_with("_test.go") ||
        file_name.ends_with("_test.rs") ||
        file_name.ends_with("Test.java") ||
        file_name.ends_with("Tests.swift") ||
        path_str.contains("/test/") ||
        path_str.contains("/tests/") ||
        path_str.contains("/__tests__/") ||
        path_str.contains("/spec/") ||
        path_str.contains("/specs/")
    }


    /// Analyze JavaScript/TypeScript test files
    fn analyze_js_ts_tests(&self, content: &str, file_path: &str, language: &str) -> Result<Vec<DetectedTest>> {
        let mut tests = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_suite: Option<String> = None;
        let mut framework = TestFramework::Unknown;

        // Detect framework
        if content.contains("jest") || content.contains("from 'jest'") || content.contains("from \"jest\"") {
            framework = TestFramework::Jest;
        } else if content.contains("mocha") || content.contains("from 'mocha'") || content.contains("from \"mocha\"") {
            framework = TestFramework::Mocha;
        } else if content.contains("vitest") || content.contains("from 'vitest'") || content.contains("from \"vitest\"") {
            framework = TestFramework::Vitest;
        } else if content.contains("describe") || content.contains("it(") || content.contains("test(") {
            // Generic test framework detection
            framework = TestFramework::Jest; // Default to Jest for describe/it patterns
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_num = line_idx + 1;

            // Detect describe blocks (test suites)
            if line.starts_with("describe(") || line.starts_with("describe.skip(") || line.starts_with("describe.only(") {
                if let Some(suite_name) = self.extract_describe_name(line) {
                    current_suite = Some(suite_name.clone());
                }
            }

            // Detect test/it blocks
            if line.starts_with("it(") || line.starts_with("test(") || 
               line.starts_with("it.skip(") || line.starts_with("test.skip(") ||
               line.starts_with("it.only(") || line.starts_with("test.only(") {
                if let Some(test_name) = self.extract_test_name(line) {
                    let id = Uuid::new_v4().to_string();
                    let mut assertions = Vec::new();
                    let mut setup_methods = Vec::new();
                    let mut teardown_methods = Vec::new();

                    // Look ahead for assertions and setup/teardown
                    for next_line in lines.iter().skip(line_idx).take(20) {
                        if next_line.contains("expect(") || next_line.contains("assert(") {
                            assertions.push(next_line.trim().to_string());
                        }
                        if next_line.contains("beforeEach") || next_line.contains("beforeAll") {
                            setup_methods.push("beforeEach".to_string());
                        }
                        if next_line.contains("afterEach") || next_line.contains("afterAll") {
                            teardown_methods.push("afterEach".to_string());
                        }
                    }

                    tests.push(DetectedTest {
                        id,
                        name: test_name.clone(),
                        test_framework: framework.clone(),
                        file_path: file_path.to_string(),
                        line_number: line_num,
                        language: language.to_string(),
                        test_type: "unit".to_string(),
                        suite_name: current_suite.clone(),
                        assertions,
                        setup_methods,
                        teardown_methods,
                        signature: Some(line.to_string()),
                        doc_comment: None,
                        parameters: self.extract_parameters_js(line),
                        return_type: None,
                    });
                }
            }
        }

        Ok(tests)
    }

    /// Analyze Python test files
    fn analyze_python_tests(&self, content: &str, file_path: &str) -> Result<Vec<DetectedTest>> {
        let mut tests = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut framework = TestFramework::Unknown;
        let mut current_class: Option<String> = None;

        // Detect framework
        if content.contains("import unittest") || content.contains("from unittest") {
            framework = TestFramework::Unittest;
        } else if content.contains("import pytest") || content.contains("from pytest") {
            framework = TestFramework::Pytest;
        } else if content.contains("import nose") || content.contains("from nose") {
            framework = TestFramework::Nose;
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_num = line_idx + 1;

            // Detect test classes
            if line.starts_with("class ") && (line.contains("Test") || line.contains("TestCase")) {
                if let Some(class_name) = self.extract_class_name_python(line) {
                    current_class = Some(class_name.clone());
                }
            }

            // Detect test methods
            if line.starts_with("def test_") || line.starts_with("def test") {
                if let Some(test_name) = self.extract_test_name_python(line) {
                    let id = Uuid::new_v4().to_string();
                    let mut assertions = Vec::new();

                    // Look ahead for assertions
                    for next_line in lines.iter().skip(line_idx).take(30) {
                        if next_line.contains("assert ") || next_line.contains("self.assert") {
                            assertions.push(next_line.trim().to_string());
                        }
                    }

                    tests.push(DetectedTest {
                        id,
                        name: test_name.clone(),
                        test_framework: framework.clone(),
                        file_path: file_path.to_string(),
                        line_number: line_num,
                        language: "python".to_string(),
                        test_type: "unit".to_string(),
                        suite_name: current_class.clone(),
                        assertions,
                        setup_methods: vec!["setUp".to_string()], // Common in unittest
                        teardown_methods: vec!["tearDown".to_string()],
                        signature: Some(line.to_string()),
                        doc_comment: None,
                        parameters: self.extract_parameters_python(line),
                        return_type: None,
                    });
                }
            }
        }

        Ok(tests)
    }

    /// Analyze Rust test files
    fn analyze_rust_tests(&self, content: &str, file_path: &str) -> Result<Vec<DetectedTest>> {
        let mut tests = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let framework = TestFramework::CargoTest;

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_num = line_idx + 1;

            // Detect #[test] attributes
            if line.starts_with("#[test]") || line.contains("#[test]") {
                // Look for the function definition on the next few lines
                for next_line in lines.iter().skip(line_idx).take(5) {
                    if next_line.contains("fn ") {
                        if let Some(test_name) = self.extract_function_name_rust(next_line) {
                            let id = Uuid::new_v4().to_string();
                            let mut assertions = Vec::new();

                            // Look ahead for assertions
                            for assert_line in lines.iter().skip(line_idx).take(50) {
                                if assert_line.contains("assert!") || assert_line.contains("assert_eq!") {
                                    assertions.push(assert_line.trim().to_string());
                                }
                            }

                            tests.push(DetectedTest {
                                id,
                                name: test_name.clone(),
                                test_framework: framework.clone(),
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                language: "rust".to_string(),
                                test_type: "unit".to_string(),
                                suite_name: None,
                                assertions,
                                setup_methods: Vec::new(),
                                teardown_methods: Vec::new(),
                                signature: Some(next_line.to_string()),
                                doc_comment: None,
                                parameters: self.extract_parameters_rust(next_line),
                                return_type: None,
                            });
                        }
                        break;
                    }
                }
            }
        }

        Ok(tests)
    }

    /// Analyze Go test files
    fn analyze_go_tests(&self, content: &str, file_path: &str) -> Result<Vec<DetectedTest>> {
        let mut tests = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let framework = TestFramework::GoTest;

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_num = line_idx + 1;

            // Detect Test functions (must start with "Test")
            if line.starts_with("func Test") {
                if let Some(test_name) = self.extract_test_name_go(line) {
                    let id = Uuid::new_v4().to_string();
                    let mut assertions = Vec::new();

                    // Look ahead for assertions
                    for next_line in lines.iter().skip(line_idx).take(50) {
                        if next_line.contains("t.Error") || next_line.contains("t.Fatal") ||
                           next_line.contains("assert.") || next_line.contains("require.") {
                            assertions.push(next_line.trim().to_string());
                        }
                    }

                    tests.push(DetectedTest {
                        id,
                        name: test_name.clone(),
                        test_framework: framework.clone(),
                        file_path: file_path.to_string(),
                        line_number: line_num,
                        language: "go".to_string(),
                        test_type: "unit".to_string(),
                        suite_name: None,
                        assertions,
                        setup_methods: Vec::new(),
                        teardown_methods: Vec::new(),
                        signature: Some(line.to_string()),
                        doc_comment: None,
                        parameters: self.extract_parameters_go(line),
                        return_type: None,
                    });
                }
            }
        }

        Ok(tests)
    }

    /// Analyze Java test files
    fn analyze_java_tests(&self, content: &str, file_path: &str) -> Result<Vec<DetectedTest>> {
        let mut tests = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut framework = TestFramework::Unknown;
        let mut current_class: Option<String> = None;

        // Detect framework
        if content.contains("@Test") && content.contains("org.junit") {
            framework = TestFramework::JUnit;
        } else if content.contains("@Test") && content.contains("org.testng") {
            framework = TestFramework::TestNG;
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_num = line_idx + 1;

            // Detect test classes
            if line.contains("class ") && (line.contains("Test") || line.contains("Tests")) {
                if let Some(class_name) = self.extract_class_name_java(line) {
                    current_class = Some(class_name.clone());
                }
            }

            // Detect @Test annotations
            if line.contains("@Test") {
                // Look for method definition on next few lines
                for next_line in lines.iter().skip(line_idx).take(5) {
                    if next_line.contains("public void") || next_line.contains("public static void") {
                        if let Some(test_name) = self.extract_test_name_java(next_line) {
                            let id = Uuid::new_v4().to_string();
                            let mut assertions = Vec::new();

                            // Look ahead for assertions
                            for assert_line in lines.iter().skip(line_idx).take(50) {
                                if assert_line.contains("assert") || assert_line.contains("Assert.") {
                                    assertions.push(assert_line.trim().to_string());
                                }
                            }

                            tests.push(DetectedTest {
                                id,
                                name: test_name.clone(),
                                test_framework: framework.clone(),
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                language: "java".to_string(),
                                test_type: "unit".to_string(),
                                suite_name: current_class.clone(),
                                assertions,
                                setup_methods: vec!["@BeforeEach".to_string(), "@BeforeAll".to_string()],
                                teardown_methods: vec!["@AfterEach".to_string(), "@AfterAll".to_string()],
                                signature: Some(next_line.to_string()),
                                doc_comment: None,
                                parameters: Vec::new(),
                                return_type: Some("void".to_string()),
                            });
                        }
                        break;
                    }
                }
            }
        }

        Ok(tests)
    }

    /// Analyze Swift test files
    fn analyze_swift_tests(&self, content: &str, file_path: &str) -> Result<Vec<DetectedTest>> {
        let mut tests = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut framework = TestFramework::Unknown;
        let mut current_class: Option<String> = None;

        // Detect framework
        if content.contains("import XCTest") {
            framework = TestFramework::XCTest;
        } else if content.contains("import Quick") || content.contains("import Nimble") {
            framework = TestFramework::Quick;
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_num = line_idx + 1;

            // Detect test classes
            if line.contains("class ") && (line.contains("Tests") || line.contains("TestCase")) {
                if let Some(class_name) = self.extract_class_name_swift(line) {
                    current_class = Some(class_name.clone());
                }
            }

            // Detect test methods (func test...)
            if line.starts_with("func test") {
                if let Some(test_name) = self.extract_test_name_swift(line) {
                    let id = Uuid::new_v4().to_string();
                    let mut assertions = Vec::new();

                    // Look ahead for assertions
                    for next_line in lines.iter().skip(line_idx).take(50) {
                        if next_line.contains("XCTAssert") || next_line.contains("expect(") {
                            assertions.push(next_line.trim().to_string());
                        }
                    }

                    tests.push(DetectedTest {
                        id,
                        name: test_name.clone(),
                        test_framework: framework.clone(),
                        file_path: file_path.to_string(),
                        line_number: line_num,
                        language: "swift".to_string(),
                        test_type: "unit".to_string(),
                        suite_name: current_class.clone(),
                        assertions,
                        setup_methods: vec!["setUp()".to_string()],
                        teardown_methods: vec!["tearDown()".to_string()],
                        signature: Some(line.to_string()),
                        doc_comment: None,
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }
        }

        Ok(tests)
    }

    // Helper methods for extracting names and parameters
    fn extract_describe_name(&self, line: &str) -> Option<String> {
        // Extract name from describe("name", ...)
        if let Some(start) = line.find('(') {
            let after_paren = &line[start + 1..];
            if let Some(quote_start) = after_paren.find('"') {
                let after_quote = &after_paren[quote_start + 1..];
                if let Some(quote_end) = after_quote.find('"') {
                    return Some(after_quote[..quote_end].to_string());
                }
            }
        }
        None
    }

    fn extract_test_name(&self, line: &str) -> Option<String> {
        // Extract name from it("name", ...) or test("name", ...)
        self.extract_describe_name(line)
    }

    fn extract_test_name_python(&self, line: &str) -> Option<String> {
        // Extract from def test_name(...)
        if let Some(start) = line.find("def test") {
            let after_def = &line[start + 4..];
            if let Some(paren) = after_def.find('(') {
                return Some(after_def[..paren].trim().to_string());
            }
        }
        None
    }

    fn extract_function_name_rust(&self, line: &str) -> Option<String> {
        // Extract from fn name(...)
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(space_or_paren) = after_fn.find(|c: char| c.is_whitespace() || c == '(') {
                return Some(after_fn[..space_or_paren].trim().to_string());
            }
        }
        None
    }

    fn extract_test_name_go(&self, line: &str) -> Option<String> {
        // Extract from func TestName(...)
        if let Some(start) = line.find("func Test") {
            let after_func = &line[start + 5..];
            if let Some(paren) = after_func.find('(') {
                return Some(after_func[..paren].trim().to_string());
            }
        }
        None
    }

    fn extract_test_name_java(&self, line: &str) -> Option<String> {
        // Extract from public void testName(...)
        if let Some(start) = line.find("void ") {
            let after_void = &line[start + 5..];
            if let Some(paren) = after_void.find('(') {
                return Some(after_void[..paren].trim().to_string());
            }
        }
        None
    }

    fn extract_test_name_swift(&self, line: &str) -> Option<String> {
        // Extract from func testName(...)
        if let Some(start) = line.find("func test") {
            let after_func = &line[start + 5..];
            if let Some(paren) = after_func.find('(') {
                return Some(after_func[..paren].trim().to_string());
            }
        }
        None
    }

    fn extract_class_name_python(&self, line: &str) -> Option<String> {
        // Extract from class Name(...)
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            if let Some(colon_or_paren) = after_class.find(|c: char| c == ':' || c == '(') {
                return Some(after_class[..colon_or_paren].trim().to_string());
            }
        }
        None
    }

    fn extract_class_name_java(&self, line: &str) -> Option<String> {
        // Extract from public class Name {...}
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            if let Some(space_or_brace) = after_class.find(|c: char| c.is_whitespace() || c == '{') {
                return Some(after_class[..space_or_brace].trim().to_string());
            }
        }
        None
    }

    fn extract_class_name_swift(&self, line: &str) -> Option<String> {
        // Extract from class Name: ...
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            if let Some(colon) = after_class.find(':') {
                return Some(after_class[..colon].trim().to_string());
            }
        }
        None
    }

    fn extract_parameters_js(&self, line: &str) -> Vec<String> {
        // Extract parameters from function/test signature
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start + 1..].find(')') {
                let params_str = &line[start + 1..start + 1 + end];
                return params_str.split(',').map(|p| p.trim().to_string()).collect();
            }
        }
        Vec::new()
    }

    fn extract_parameters_python(&self, line: &str) -> Vec<String> {
        // Extract parameters from def test_name(param1, param2):
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start + 1..].find(')') {
                let params_str = &line[start + 1..start + 1 + end];
                return params_str.split(',').map(|p| p.trim().to_string()).collect();
            }
        }
        Vec::new()
    }

    fn extract_parameters_rust(&self, line: &str) -> Vec<String> {
        // Extract parameters from fn name(param1: Type, param2: Type)
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start + 1..].find(')') {
                let params_str = &line[start + 1..start + 1 + end];
                return params_str.split(',').map(|p| p.trim().to_string()).collect();
            }
        }
        Vec::new()
    }

    fn extract_parameters_go(&self, line: &str) -> Vec<String> {
        // Extract parameters from func TestName(t *testing.T)
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start + 1..].find(')') {
                let params_str = &line[start + 1..start + 1 + end];
                return params_str.split(',').map(|p| p.trim().to_string()).collect();
            }
        }
        Vec::new()
    }
}

