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
            let path_str = path.to_string_lossy().to_lowercase();
            if file_name.starts_with('.') || 
               path_str.contains("node_modules") ||
               path_str.contains("target") ||
               path_str.contains(".git") ||
               path_str.contains("/dist/") ||
               path_str.contains("/build/") ||
               path_str.contains("/.next/") ||
               path_str.contains("\\.next\\") ||  // Windows path separator
               path_str.contains("/out/") ||
               path_str.contains("/.nuxt/") ||
               path_str.contains("/.cache/") ||
               path_str.contains("/coverage/") ||
               path_str.contains("/.next/static/") ||  // Next.js build artifacts
               path_str.contains("/.next/server/") ||  // Next.js server chunks
               file_name.ends_with(".min.js") ||
               file_name.ends_with(".min.css") ||
               file_name.ends_with(".bundle.js") ||
               file_name.ends_with(".chunk.js") ||
               file_name.ends_with(".class") ||
               file_name.ends_with(".pyc") ||
               file_name.ends_with(".pyo") ||
               file_name.ends_with(".so") ||
               file_name.ends_with(".dll") ||
               file_name.ends_with(".dylib") ||
               file_name.ends_with(".a") ||
               file_name.ends_with(".o") ||
               file_name.ends_with(".rlib") {
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
                // Skip minified/compiled code
                if self.is_minified_or_compiled(&content, &normalized_path) {
                    continue;
                }
                
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
                    Some("swift") => {
                        let (file_elements, file_calls) = self.analyze_swift(&content, &normalized_path)?;
                        elements.extend(file_elements);
                        calls.extend(file_calls);
                    }
                    Some("objective-c") => {
                        let (file_elements, file_calls) = self.analyze_objective_c(&content, &normalized_path)?;
                        elements.extend(file_elements);
                        calls.extend(file_calls);
                    }
                    Some("java") => {
                        let (file_elements, file_calls) = self.analyze_java(&content, &normalized_path)?;
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
                    "swift" => Some("swift".to_string()),
                    "m" | "mm" => Some("objective-c".to_string()),
                    "java" => Some("java".to_string()),
                    "h" => {
                        // Header files could be C, C++, or Objective-C
                        // Check if it's likely Objective-C by looking at the directory structure
                        let path_str = path.to_string_lossy().to_lowercase();
                        if path_str.contains(".xcodeproj") || path_str.contains("ios") || path_str.contains("iphone") || path_str.contains("macos") {
                            Some("objective-c".to_string())
                        } else {
                            None // Skip C/C++ headers for now
                        }
                    },
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

    /// Analyze Swift files
    fn analyze_swift(&self, content: &str, normalized_path: &str) -> Result<(Vec<CodeElement>, Vec<CodeCall>)> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();
        let mut element_map: HashMap<String, String> = HashMap::new();

        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_idx = line_num + 1;

            // Detect function declarations: func functionName(...)
            if line.starts_with("func ") {
                if let Some(name) = self.extract_function_name_swift(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Function,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "swift".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_swift(line),
                        parameters: self.extract_parameters_swift(line),
                        return_type: self.extract_return_type_swift(line),
                    });
                }
            }

            // Detect class declarations: class ClassName
            if line.starts_with("class ") || line.starts_with("final class ") {
                if let Some(name) = self.extract_class_name_swift(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Class,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "swift".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_swift(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect struct declarations: struct StructName
            if line.starts_with("struct ") {
                if let Some(name) = self.extract_struct_name_swift(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Struct,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "swift".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_swift(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect enum declarations: enum EnumName
            if line.starts_with("enum ") {
                if let Some(name) = self.extract_enum_name_swift(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Enum,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "swift".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_swift(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect imports: import ModuleName
            if line.starts_with("import ") {
                if let Some(module) = self.extract_import_swift(line) {
                    let module_id = Uuid::new_v4().to_string();
                    if !element_map.contains_key(&module_id) {
                        elements.push(CodeElement {
                            id: module_id.clone(),
                            name: module.clone(),
                            element_type: CodeElementType::Module,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "swift".to_string(),
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

    /// Analyze Objective-C files
    fn analyze_objective_c(&self, content: &str, normalized_path: &str) -> Result<(Vec<CodeElement>, Vec<CodeCall>)> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();
        let mut element_map: HashMap<String, String> = HashMap::new();

        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_idx = line_num + 1;

            // Detect method declarations: - (returnType)methodName or + (returnType)methodName
            if (line.starts_with("- ") || line.starts_with("+ ")) && line.contains('(') {
                if let Some(name) = self.extract_method_name_objc(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Method,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "objective-c".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: if line.starts_with("+") { Some("public".to_string()) } else { Some("private".to_string()) },
                        parameters: Vec::new(),
                        return_type: self.extract_return_type_objc(line),
                    });
                }
            }

            // Detect interface declarations: @interface ClassName
            if line.starts_with("@interface ") {
                if let Some(name) = self.extract_class_name_objc(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Class,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "objective-c".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: Some("public".to_string()),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect implementation: @implementation ClassName
            if line.starts_with("@implementation ") {
                if let Some(name) = self.extract_class_name_objc(line) {
                    // Implementation is usually already tracked as a class
                    if !element_map.contains_key(&name) {
                        let id = uuid::Uuid::new_v4().to_string();
                        element_map.insert(name.clone(), id.clone());
                        
                        elements.push(CodeElement {
                            id: id.clone(),
                            name,
                            element_type: CodeElementType::Class,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "objective-c".to_string(),
                            signature: Some(line.to_string()),
                            doc_comment: self.extract_doc_comment(&lines, line_num),
                            visibility: Some("public".to_string()),
                            parameters: Vec::new(),
                            return_type: None,
                        });
                    }
                }
            }

            // Detect imports: #import "file.h" or #import <Module/file.h>
            if line.starts_with("#import ") {
                if let Some(module) = self.extract_import_objc(line) {
                    let module_id = Uuid::new_v4().to_string();
                    if !element_map.contains_key(&module_id) {
                        elements.push(CodeElement {
                            id: module_id.clone(),
                            name: module.clone(),
                            element_type: CodeElementType::Module,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "objective-c".to_string(),
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

    // Swift helper functions
    fn extract_function_name_swift(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("func ") {
            let after_func = &line[start + 5..];
            if let Some(end) = after_func.find('(') {
                let name_part = &after_func[..end].trim();
                // Handle async, throws, etc.
                let name = name_part.split_whitespace().next().unwrap_or("").to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    fn extract_class_name_swift(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            if let Some(end) = after_class.find(|c: char| c.is_whitespace()) {
                return Some(after_class[..end].trim().to_string());
            } else if let Some(end) = after_class.find(':') {
                return Some(after_class[..end].trim().to_string());
            } else if let Some(end) = after_class.find('{') {
                return Some(after_class[..end].trim().to_string());
            } else {
                return Some(after_class.trim().to_string());
            }
        }
        None
    }

    fn extract_struct_name_swift(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("struct ") {
            let after_struct = &line[start + 7..];
            if let Some(end) = after_struct.find(|c: char| c.is_whitespace()) {
                return Some(after_struct[..end].trim().to_string());
            } else if let Some(end) = after_struct.find(':') {
                return Some(after_struct[..end].trim().to_string());
            } else if let Some(end) = after_struct.find('{') {
                return Some(after_struct[..end].trim().to_string());
            } else {
                return Some(after_struct.trim().to_string());
            }
        }
        None
    }

    fn extract_enum_name_swift(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("enum ") {
            let after_enum = &line[start + 5..];
            if let Some(end) = after_enum.find(|c: char| c.is_whitespace()) {
                return Some(after_enum[..end].trim().to_string());
            } else if let Some(end) = after_enum.find(':') {
                return Some(after_enum[..end].trim().to_string());
            } else if let Some(end) = after_enum.find('{') {
                return Some(after_enum[..end].trim().to_string());
            } else {
                return Some(after_enum.trim().to_string());
            }
        }
        None
    }

    fn extract_import_swift(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("import ") {
            let after_import = &line[start + 7..];
            let module = after_import.trim();
            if !module.is_empty() {
                return Some(module.to_string());
            }
        }
        None
    }

    fn extract_visibility_swift(&self, line: &str) -> Option<String> {
        if line.contains("private ") || line.contains("fileprivate ") {
            Some("private".to_string())
        } else if line.contains("public ") || line.contains("open ") {
            Some("public".to_string())
        } else if line.contains("internal ") {
            Some("internal".to_string())
        } else {
            Some("internal".to_string()) // Default in Swift
        }
    }

    fn extract_parameters_swift(&self, line: &str) -> Vec<String> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let params = &line[start+1..start+1+end];
                return params.split(',')
                    .map(|p| {
                        let param = p.trim();
                        // Extract parameter name (before colon in Swift)
                        if let Some(colon) = param.find(':') {
                            param[..colon].trim().to_string()
                        } else {
                            param.to_string()
                        }
                    })
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    fn extract_return_type_swift(&self, line: &str) -> Option<String> {
        // Swift return types come after -> : func name() -> ReturnType
        if let Some(arrow) = line.find("->") {
            let after_arrow = &line[arrow + 2..];
            let return_type = after_arrow.trim();
            if let Some(end) = return_type.find('{') {
                return Some(return_type[..end].trim().to_string());
            } else if let Some(end) = return_type.find(|c: char| c.is_whitespace()) {
                return Some(return_type[..end].trim().to_string());
            } else {
                return Some(return_type.to_string());
            }
        }
        None
    }

    // Objective-C helper functions
    fn extract_method_name_objc(&self, line: &str) -> Option<String> {
        // Format: - (returnType)methodName:param1:param2
        if let Some(start) = line.find(')') {
            let after_return = &line[start + 1..];
            if let Some(end) = after_return.find(':') {
                return Some(after_return[..end].trim().to_string());
            } else if let Some(end) = after_return.find('{') {
                return Some(after_return[..end].trim().to_string());
            } else {
                return Some(after_return.trim().to_string());
            }
        }
        None
    }

    fn extract_class_name_objc(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("@interface ") {
            let after_interface = &line[start + 11..];
            if let Some(end) = after_interface.find(|c: char| c.is_whitespace()) {
                return Some(after_interface[..end].trim().to_string());
            } else if let Some(end) = after_interface.find(':') {
                return Some(after_interface[..end].trim().to_string());
            } else {
                return Some(after_interface.trim().to_string());
            }
        } else if let Some(start) = line.find("@implementation ") {
            let after_impl = &line[start + 16..];
            if let Some(end) = after_impl.find(|c: char| c.is_whitespace()) {
                return Some(after_impl[..end].trim().to_string());
            } else if let Some(end) = after_impl.find('{') {
                return Some(after_impl[..end].trim().to_string());
            } else {
                return Some(after_impl.trim().to_string());
            }
        }
        None
    }

    fn extract_import_objc(&self, line: &str) -> Option<String> {
        // #import "file.h" or #import <Module/file.h>
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start+1..].find('"') {
                return Some(line[start+1..start+1+end].to_string());
            }
        } else if let Some(start) = line.find('<') {
            if let Some(end) = line[start+1..].find('>') {
                return Some(line[start+1..start+1+end].to_string());
            }
        }
        None
    }

    fn extract_return_type_objc(&self, line: &str) -> Option<String> {
        // Format: - (returnType)methodName
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                return Some(line[start+1..start+1+end].trim().to_string());
            }
        }
        None
    }

    /// Analyze Java files
    fn analyze_java(&self, content: &str, normalized_path: &str) -> Result<(Vec<CodeElement>, Vec<CodeCall>)> {
        let mut elements = Vec::new();
        let mut calls = Vec::new();
        let mut element_map: HashMap<String, String> = HashMap::new();

        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            let line_idx = line_num + 1;

            // Detect class declarations: public class ClassName or class ClassName
            if line.contains("class ") && (line.starts_with("public ") || line.starts_with("private ") || 
                line.starts_with("protected ") || line.starts_with("abstract ") || 
                line.starts_with("final ") || line.starts_with("class ")) {
                if let Some(name) = self.extract_class_name_java(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Class,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "java".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_java(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect interface declarations: public interface InterfaceName
            if line.contains("interface ") && (line.starts_with("public ") || line.starts_with("private ") || 
                line.starts_with("protected ") || line.starts_with("interface ")) {
                if let Some(name) = self.extract_interface_name_java(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Interface,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "java".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_java(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect enum declarations: public enum EnumName
            if line.contains("enum ") && (line.starts_with("public ") || line.starts_with("private ") || 
                line.starts_with("protected ") || line.starts_with("enum ")) {
                if let Some(name) = self.extract_enum_name_java(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Enum,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "java".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_java(line),
                        parameters: Vec::new(),
                        return_type: None,
                    });
                }
            }

            // Detect method declarations: public ReturnType methodName(...) or private void methodName(...)
            if (line.contains("public ") || line.contains("private ") || line.contains("protected ") || 
                line.contains("static ") || line.contains("final ")) && 
                line.contains('(') && line.contains(')') && 
                !line.contains("class ") && !line.contains("interface ") && !line.contains("enum ") {
                if let Some(name) = self.extract_method_name_java(line) {
                    let id = uuid::Uuid::new_v4().to_string();
                    element_map.insert(name.clone(), id.clone());
                    
                    elements.push(CodeElement {
                        id: id.clone(),
                        name,
                        element_type: CodeElementType::Method,
                        file_path: normalized_path.to_string(),
                        line_number: line_idx,
                        language: "java".to_string(),
                        signature: Some(line.to_string()),
                        doc_comment: self.extract_doc_comment(&lines, line_num),
                        visibility: self.extract_visibility_java(line),
                        parameters: self.extract_parameters_java(line),
                        return_type: self.extract_return_type_java(line),
                    });
                }
            }

            // Detect imports: import package.ClassName; or import static package.ClassName.*;
            if line.starts_with("import ") {
                if let Some(module) = self.extract_import_java(line) {
                    let module_id = Uuid::new_v4().to_string();
                    if !element_map.contains_key(&module_id) {
                        elements.push(CodeElement {
                            id: module_id.clone(),
                            name: module.clone(),
                            element_type: CodeElementType::Module,
                            file_path: normalized_path.to_string(),
                            line_number: line_idx,
                            language: "java".to_string(),
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

    // Java helper functions
    fn extract_class_name_java(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            // Skip generic type parameters: class MyClass<T>
            let name_part = if let Some(generic_start) = after_class.find('<') {
                &after_class[..generic_start]
            } else if let Some(space_end) = after_class.find(|c: char| c.is_whitespace()) {
                &after_class[..space_end]
            } else if let Some(extends_start) = after_class.find("extends") {
                &after_class[..extends_start]
            } else if let Some(implements_start) = after_class.find("implements") {
                &after_class[..implements_start]
            } else if let Some(brace_start) = after_class.find('{') {
                &after_class[..brace_start]
            } else {
                after_class
            };
            let name = name_part.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }

    fn extract_interface_name_java(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("interface ") {
            let after_interface = &line[start + 9..];
            let name_part = if let Some(generic_start) = after_interface.find('<') {
                &after_interface[..generic_start]
            } else if let Some(space_end) = after_interface.find(|c: char| c.is_whitespace()) {
                &after_interface[..space_end]
            } else if let Some(extends_start) = after_interface.find("extends") {
                &after_interface[..extends_start]
            } else if let Some(brace_start) = after_interface.find('{') {
                &after_interface[..brace_start]
            } else {
                after_interface
            };
            let name = name_part.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }

    fn extract_enum_name_java(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("enum ") {
            let after_enum = &line[start + 5..];
            let name_part = if let Some(space_end) = after_enum.find(|c: char| c.is_whitespace()) {
                &after_enum[..space_end]
            } else if let Some(implements_start) = after_enum.find("implements") {
                &after_enum[..implements_start]
            } else if let Some(brace_start) = after_enum.find('{') {
                &after_enum[..brace_start]
            } else {
                after_enum
            };
            let name = name_part.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }

    fn extract_method_name_java(&self, line: &str) -> Option<String> {
        // Format: [modifiers] ReturnType methodName(params)
        // Find the opening parenthesis and work backwards
        if let Some(paren_start) = line.find('(') {
            let before_paren = &line[..paren_start];
            // Split by whitespace and get the last token before (
            let parts: Vec<&str> = before_paren.split_whitespace().collect();
            if let Some(last_part) = parts.last() {
                // Skip if it's a modifier or type keyword
                let modifiers = ["public", "private", "protected", "static", "final", "abstract",
                                 "synchronized", "native", "strictfp", "void", "int", "String",
                                 "boolean", "long", "double", "float", "char", "byte", "short"];
                if !last_part.is_empty() && 
                   !modifiers.contains(last_part) &&
                   !last_part.contains('.') {
                    return Some(last_part.to_string());
                }
            }
        }
        None
    }

    fn extract_import_java(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("import ") {
            let after_import = &line[start + 7..];
            // Remove "static " if present
            let after_static = if after_import.starts_with("static ") {
                &after_import[7..]
            } else {
                after_import
            };
            // Remove semicolon
            let module = after_static.trim_end_matches(';').trim();
            if !module.is_empty() {
                return Some(module.to_string());
            }
        }
        None
    }

    fn extract_visibility_java(&self, line: &str) -> Option<String> {
        if line.contains("public ") {
            Some("public".to_string())
        } else if line.contains("private ") {
            Some("private".to_string())
        } else if line.contains("protected ") {
            Some("protected".to_string())
        } else {
            Some("package-private".to_string()) // Default in Java
        }
    }

    fn extract_parameters_java(&self, line: &str) -> Vec<String> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start+1..].find(')') {
                let params = &line[start+1..start+1+end];
                return params.split(',')
                    .map(|p| {
                        let param = p.trim();
                        // Extract parameter name (last word before any = or after last space)
                        if let Some(equals) = param.find('=') {
                            param[..equals].trim().to_string()
                        } else {
                            // Get the last word (parameter name)
                            let parts: Vec<&str> = param.split_whitespace().collect();
                            if let Some(name) = parts.last() {
                                name.to_string()
                            } else {
                                param.to_string()
                            }
                        }
                    })
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    fn extract_return_type_java(&self, line: &str) -> Option<String> {
        // Format: [modifiers] ReturnType methodName(params)
        // Find the opening parenthesis and work backwards
        if let Some(paren_start) = line.find('(') {
            let before_paren = &line[..paren_start];
            let parts: Vec<&str> = before_paren.split_whitespace().collect();
            // Return type is usually the second-to-last or last part (before method name)
            // Skip modifiers: public, private, static, final, etc.
            let modifiers = ["public", "private", "protected", "static", "final", "abstract", 
                             "synchronized", "native", "strictfp"];
            for part in parts.iter().rev() {
                if !modifiers.contains(part) && !part.is_empty() {
                    return Some(part.to_string());
                }
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

    /// Check if code appears to be minified or compiled
    fn is_minified_or_compiled(&self, content: &str, file_path: &str) -> bool {
        // Skip empty or very small files
        if content.len() < 100 {
            return false;
        }

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return false;
        }

        // Check for common compiled/minified indicators:
        // 1. Very long average line length (minified code has long lines)
        let avg_line_length = content.len() / lines.len().max(1);
        if avg_line_length > 200 {
            return true;
        }

        // 2. Very few line breaks relative to content size (minified code is mostly on one line)
        if lines.len() < content.len() / 500 {
            return true;
        }

        // 3. High ratio of non-whitespace characters (minified code has no spaces)
        let non_whitespace: usize = content.chars().filter(|c| !c.is_whitespace()).count();
        let whitespace_ratio = if content.len() > 0 {
            non_whitespace as f64 / content.len() as f64
        } else {
            0.0
        };
        if whitespace_ratio > 0.95 && content.len() > 1000 {
            return true;
        }

        // 4. Check for common minified patterns
        if content.contains("!function") && content.contains("(function") && whitespace_ratio > 0.9 {
            return true;
        }

        // 5. Check for source map comments (indicates compiled/minified code)
        if content.contains("//# sourceMappingURL=") || content.contains("//@ sourceMappingURL=") {
            return true;
        }

        // 6. Check for webpack/rollup bundle patterns
        if file_path.contains("bundle") || file_path.contains("chunk") {
            if content.contains("webpackChunkName") || content.contains("__webpack_require__") {
                return true;
            }
        }

        false
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

