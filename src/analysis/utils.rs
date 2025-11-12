use std::path::Path;

/// Detect programming language from file extension
/// 
/// Returns the language name as a string, or None if the language is not supported
pub fn detect_language(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .and_then(|ext| {
            match ext.to_lowercase().as_str() {
                "js" | "jsx" | "mjs" => Some("javascript".to_string()),
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
                    if path_str.contains(".xcodeproj") || path_str.contains("ios") || 
                       path_str.contains("iphone") || path_str.contains("macos") {
                        Some("objective-c".to_string())
                    } else {
                        None // Skip C/C++ headers for now
                    }
                },
                _ => None,
            }
        })
}

/// Check if content appears to be minified or compiled code
/// 
/// This is a heuristic check that looks for common indicators of minified code:
/// - Very long average line length (>200 chars)
/// - Very few line breaks relative to content size
/// - High ratio of non-whitespace characters (>95%)
/// - Common minified patterns (!function, webpack patterns)
/// - Source map comments
pub fn is_minified_or_compiled(content: &str, file_path: &str) -> bool {
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

/// Check if a file path should be skipped during analysis
/// 
/// Returns true if the file matches common ignore patterns (node_modules, build artifacts, etc.)
pub fn should_skip_file(file_name: &str, path_str: &str) -> bool {
    file_name.starts_with('.') || 
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
    path_str.contains("/.next/static/") ||
    path_str.contains("/.next/server/") ||
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
    file_name.ends_with(".rlib")
}

