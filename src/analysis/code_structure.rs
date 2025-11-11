use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CodeElementType {
    Function,
    Class,
    Module,
    Interface,
    Struct,
    Enum,
    Method,
    Constant,
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    pub id: String,
    pub name: String,
    pub element_type: CodeElementType,
    pub file_path: String,
    pub line_number: usize,
    pub language: String,
    pub signature: Option<String>,
    pub doc_comment: Option<String>,
    pub visibility: Option<String>, // public, private, protected
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCall {
    pub caller_id: String,
    pub callee_id: String,
    pub call_type: String, // function_call, method_call, import, etc.
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeStructure {
    pub elements: Vec<CodeElement>,
    pub calls: Vec<CodeCall>,
}

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn new() -> Self {
        CodeAnalyzer
    }

    /// Analyze code structure in a repository
    pub fn analyze_repository(&self, repo_path: &Path) -> Result<CodeStructure> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();

        // Walk through code files
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
            if file_name.starts_with('.') || 
               path.to_string_lossy().contains("node_modules") ||
               path.to_string_lossy().contains("target") ||
               path.to_string_lossy().contains(".git") {
                continue;
            }

            // Normalize path: make it relative to repo_path
            let normalized_path = path.strip_prefix(repo_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());

            // Determine language from extension
            let language = self.detect_language(path);
            if language.is_none() {
                continue;
            }

            // Analyze file based on language
            if let Ok(content) = std::fs::read_to_string(path) {
                match language.as_deref() {
                    Some("javascript") | Some("typescript") => {
                        let (file_elements, file_calls) = self.analyze_js_ts(&content, &normalized_path)?;
                        elements.extend(file_elements);
                        calls.extend(file_calls);
                    }
                    Some("python") => {
                        let (file_elements, file_calls) = self.analyze_python(&content, &normalized_path)?;
                        elements.extend(file_elements);
                        calls.extend(file_calls);
                    }
                    Some("rust") => {
                        let (file_elements, file_calls) = self.analyze_rust(&content, &normalized_path)?;
                        elements.extend(file_elements);
                        calls.extend(file_calls);
                    }
                    Some("go") => {
                        let (file_elements, file_calls) = self.analyze_go(&content, &normalized_path)?;
                        elements.extend(file_elements);
                        calls.extend(file_calls);
                    }
                    _ => {}
                }
            }
        }

        Ok(CodeStructure { elements, calls })
    }

    fn detect_language(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| {
                match ext.to_lowercase().as_str() {
                    "js" | "jsx" => Some("javascript".to_string()),
                    "ts" | "tsx" => Some("typescript".to_string()),
                    "py" => Some("python".to_string()),
                    "rs" => Some("rust".to_string()),
                    "go" => Some("go".to_string()),
                    _ => None,
                }
            })
            .flatten()
    }

    /// Analyze JavaScript/TypeScript files
    fn analyze_js_ts(&self, content: &str, normalized_path: &str) -> Result<(Vec<CodeElement>, Vec<CodeCall>)> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();
        let mut element_map: HashMap<String, String> = HashMap::new(); // name -> id

        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_idx = line_num + 1;

            // Detect function declarations
            if line.contains("function ") || line.contains("const ") && line.contains("=") && line.contains("=>") {
                if let Some(name) = self.extract_function_name_js(line) {
                    // Use UUID for ID to ensure uniqueness
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Function,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "javascript".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: None,
                        parameters: self.extract_parameters_js(line),
                        return_type: None,
                    });
                }
            }

            // Detect class declarations
            if line.contains("class ") {
                if let Some(name) = self.extract_class_name_js(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Class,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "javascript".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: None,
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect imports
            if line.starts_with("import ") || line.starts_with("require(") {
                if let Some(module) = self.extract_import_js(line) {
                    // Create a module element if it doesn't exist
                    let module_id = Uuid::new_v4().to_string();
                    if !element_map.contains_key(&module_id) {
                        elements.push(CodeElement {
                            id: module_id.clone(),
                            name: module.clone(),
                            element_type: CodeElementType::Module,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "javascript".to_string(),
                            signature: None,
                            doc_comment: None,
                            visibility: None,
                            parameters: Vec::new(),
                            return_type: None,
                        });
                        element_map.insert(module_id.clone(), module_id.clone());
                    }
                }
            }
        }

        Ok((elements, calls))
    }

    /// Analyze Python files
    fn analyze_python(&self, content: &str, normalized_path: &str) -> Result<(Vec<CodeElement>, Vec<CodeCall>)> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();
        let mut element_map: HashMap<String, String> = HashMap::new();

        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_idx = line_num + 1;

            // Detect function definitions
            if line.starts_with("def ") {
                if let Some(name) = self.extract_function_name_python(line) {
                    let id = Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Function,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "python".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: None,
                        parameters: self.extract_parameters_python(line),
                        return_type: None,
                    });
                }
            }

            // Detect class definitions
            if line.starts_with("class ") {
                if let Some(name) = self.extract_class_name_python(line) {
                    let id = Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Class,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "python".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: None,
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect imports
            if line.starts_with("import ") || line.starts_with("from ") {
                if let Some(module) = self.extract_import_python(line) {
                    let module_id = Uuid::new_v4().to_string();
                    if !element_map.contains_key(&module_id) {
                        elements.push(CodeElement {
                            id: module_id.clone(),
                            name: module.clone(),
                            element_type: CodeElementType::Module,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "python".to_string(),
                            signature: None,
                            doc_comment: None,
                            visibility: None,
                            parameters: Vec::new(),
                            return_type: None,
                        });
                        element_map.insert(module_id.clone(), module_id.clone());
                    }
                }
            }
        }

        Ok((elements, calls))
    }

    /// Analyze Rust files
    fn analyze_rust(&self, content: &str, normalized_path: &str) -> Result<(Vec<CodeElement>, Vec<CodeCall>)> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();
        let mut element_map: HashMap<String, String> = HashMap::new();

        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_idx = line_num + 1;

            // Detect function definitions
            if line.starts_with("fn ") || line.contains(" fn ") {
                if let Some(name) = self.extract_function_name_rust(line) {
                    let id = Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Function,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "rust".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_rust(line),
                        parameters: self.extract_parameters_rust(line),
                        return_type: self.extract_return_type_rust(line),
                    });
                }
            }

            // Detect struct definitions
            if line.starts_with("struct ") || line.contains(" struct ") {
                if let Some(name) = self.extract_struct_name_rust(line) {
                    let id = Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Struct,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "rust".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_rust(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect enum definitions
            if line.starts_with("enum ") || line.contains(" enum ") {
                if let Some(name) = self.extract_enum_name_rust(line) {
                    let id = Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Enum,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "rust".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_rust(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect use statements (imports)
            if line.starts_with("use ") {
                if let Some(module) = self.extract_use_rust(line) {
                    let module_id = Uuid::new_v4().to_string();
                    if !element_map.contains_key(&module_id) {
                        elements.push(CodeElement {
                            id: module_id.clone(),
                            name: module.clone(),
                            element_type: CodeElementType::Module,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "rust".to_string(),
                            signature: None,
                            doc_comment: None,
                            visibility: None,
                            parameters: Vec::new(),
                            return_type: None,
                        });
                        element_map.insert(module_id.clone(), module_id.clone());
                    }
                }
            }
        }

        Ok((elements, calls))
    }

    /// Analyze Go files
    fn analyze_go(&self, content: &str, normalized_path: &str) -> Result<(Vec<CodeElement>, Vec<CodeCall>)> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();
        let mut element_map: HashMap<String, String> = HashMap::new();

        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_idx = line_num + 1;

            // Detect function definitions
            if line.starts_with("func ") {
                if let Some(name) = self.extract_function_name_go(line) {
                    let id = Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Function,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "go".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_go(line),
                        parameters: self.extract_parameters_go(line),
                        return_type: self.extract_return_type_go(line),
                    });
                }
            }

            // Detect type definitions (structs, interfaces)
            if line.starts_with("type ") {
                if let Some((name, element_type)) = self.extract_type_go(line) {
                    let id = Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "go".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_go(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect imports
            if line.starts_with("import ") {
                if let Some(module) = self.extract_import_go(line) {
                    let module_id = Uuid::new_v4().to_string();
                    if !element_map.contains_key(&module_id) {
                        elements.push(CodeElement {
                            id: module_id.clone(),
                            name: module.clone(),
                            element_type: CodeElementType::Module,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "go".to_string(),
                            signature: None,
                            doc_comment: None,
                            visibility: None,
                            parameters: Vec::new(),
                            return_type: None,
                        });
                        element_map.insert(module_id.clone(), module_id.clone());
                    }
                }
            }
        }

        Ok((elements, calls))
    }

    // Helper functions for extracting names and signatures
    fn extract_function_name_js(&self, line: &str) -> Option<String> {
        // Match: function name(...) or const name = (...) =>
        if let Some(start) = line.find("function ") {
            let after_fn = &line[start + 9..];
            if let Some(end) = after_fn.find('(') {
                return Some(after_fn[..end].trim().to_string());
            }
        }
        if line.contains("const ") && line.contains("=") && line.contains("=>") {
            if let Some(start) = line.find("const ") {
                let after_const = &line[start + 6..];
                if let Some(end) = after_const.find('=') {
                    return Some(after_const[..end].trim().to_string());
                }
            }
        }
        None
    }

    fn extract_class_name_js(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            if let Some(end) = after_class.find(|c: char| c == ' ' || c == '{') {
                return Some(after_class[..end].trim().to_string());
            }
            // If no delimiter found, take until end of line
            return Some(after_class.trim().to_string());
        }
        None
    }

    fn extract_import_js(&self, line: &str) -> Option<String> {
        if line.starts_with("import ") {
            // Extract from: import name from "module" or require("module")
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start+1..].find('"') {
                    return Some(line[start+1..start+1+end].to_string());
                }
            }
        }
        if line.contains("require(") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start+1..].find('"') {
                    return Some(line[start+1..start+1+end].to_string());
                }
            }
        }
        None
    }

    fn extract_parameters_js(&self, line: &str) -> Vec<String> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let params = &line[start+1..start+1+end];
                return params.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    fn extract_function_name_python(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("def ") {
            let after_def = &line[start + 4..];
            if let Some(end) = after_def.find('(') {
                return Some(after_def[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_class_name_python(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            if let Some(end) = after_class.find(|c: char| c == ':' || c == '(') {
                return Some(after_class[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_import_python(&self, line: &str) -> Option<String> {
        if line.starts_with("import ") {
            let after_import = &line[7..];
            if let Some(end) = after_import.find(' ') {
                return Some(after_import[..end].trim().to_string());
            }
            return Some(after_import.trim().to_string());
        }
        if line.starts_with("from ") {
            if let Some(start) = line.find("from ") {
                let after_from = &line[start + 5..];
                if let Some(end) = after_from.find(" import") {
                    return Some(after_from[..end].trim().to_string());
                }
            }
        }
        None
    }

    fn extract_parameters_python(&self, line: &str) -> Vec<String> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let params = &line[start+1..start+1+end];
                return params.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    fn extract_function_name_rust(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(end) = after_fn.find(|c: char| c == '<' || c == '(') {
                return Some(after_fn[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_struct_name_rust(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("struct ") {
            let after_struct = &line[start + 7..];
            if let Some(end) = after_struct.find(|c: char| c == ' ' || c == '{' || c == '<') {
                return Some(after_struct[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_enum_name_rust(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("enum ") {
            let after_enum = &line[start + 5..];
            if let Some(end) = after_enum.find(|c: char| c == ' ' || c == '{') {
                return Some(after_enum[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_use_rust(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("use ") {
            let after_use = &line[start + 4..];
            if let Some(end) = after_use.find(|c: char| c == ';' || c == '{' || c == ':') {
                return Some(after_use[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_visibility_rust(&self, line: &str) -> Option<String> {
        if line.starts_with("pub ") {
            Some("public".to_string())
        } else {
            Some("private".to_string())
        }
    }

    fn extract_parameters_rust(&self, line: &str) -> Vec<String> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let params = &line[start+1..start+1+end];
                return params.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    fn extract_return_type_rust(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("-> ") {
            let after_arrow = &line[start + 3..];
            if let Some(end) = after_arrow.find(|c: char| c == '{' || c == ';') {
                return Some(after_arrow[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_function_name_go(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("func ") {
            let after_func = &line[start + 5..];
            // Handle receiver methods: func (r *Receiver) Method(...)
            let name_start = if after_func.starts_with('(') {
                if let Some(end) = after_func.find(')') {
                    after_func[end + 1..].trim()
                } else {
                    after_func
                }
            } else {
                after_func
            };
            if let Some(end) = name_start.find('(') {
                return Some(name_start[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_type_go(&self, line: &str) -> Option<(String, CodeElementType)> {
        if let Some(start) = line.find("type ") {
            let after_type = &line[start + 5..];
            if let Some(end) = after_type.find(' ') {
                let name = after_type[..end].trim().to_string();
                let rest = &after_type[end..];
                let element_type = if rest.contains("struct") {
                    CodeElementType::Struct
                } else if rest.contains("interface") {
                    CodeElementType::Interface
                } else {
                    CodeElementType::Struct // Default
                };
                return Some((name, element_type));
            }
        }
        None
    }

    fn extract_import_go(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start+1..].find('"') {
                return Some(line[start+1..start+1+end].to_string());
            }
        }
        None
    }

    fn extract_visibility_go(&self, line: &str) -> Option<String> {
        // In Go, exported names start with uppercase
        if let Some(name_start) = line.find("func ") {
            let after_func = &line[name_start + 5..];
            if let Some(name_end) = after_func.find('(') {
                let name = &after_func[..name_end].trim();
                if !name.is_empty() && name.chars().next().unwrap().is_uppercase() {
                    return Some("public".to_string());
                }
            }
        }
        Some("private".to_string())
    }

    fn extract_parameters_go(&self, line: &str) -> Vec<String> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let params = &line[start+1..start+1+end];
                return params.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    fn extract_return_type_go(&self, line: &str) -> Option<String> {
        // Go return types come after the parameters: func name(params) returnType
        if let Some(start) = line.find(')') {
            let after_params = &line[start + 1..];
            if let Some(end) = after_params.find('{') {
                return Some(after_params[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_doc_comment(&self, lines: &[&str], line_num: usize) -> Option<String> {
        // Look for doc comments above the function/class
        let mut doc_lines = Vec::new();
        let mut check_line = line_num;
        
        while check_line > 0 {
            check_line -= 1;
            let line = lines[check_line].trim();
            if line.starts_with("//") || line.starts_with("#") || line.starts_with("///") {
                doc_lines.insert(0, line.to_string());
            } else if line.is_empty() {
                continue;
            } else {
                break;
            }
        }
        
        if !doc_lines.is_empty() {
            Some(doc_lines.join("\n"))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_name_js() {
        let analyzer = CodeAnalyzer::new();
        assert_eq!(analyzer.extract_function_name_js("function test() {}"), Some("test".to_string()));
        assert_eq!(analyzer.extract_function_name_js("const arrow = () => {}"), Some("arrow".to_string()));
    }

    #[test]
    fn test_extract_function_name_python() {
        let analyzer = CodeAnalyzer::new();
        assert_eq!(analyzer.extract_function_name_python("def test_function():"), Some("test_function".to_string()));
    }
}

