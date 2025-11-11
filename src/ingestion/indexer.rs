use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::ingestion::crawler::RepositoryCrawler;

pub struct FileIndexer {
    crawler: RepositoryCrawler,
}

#[derive(Debug, Clone)]
pub struct IndexedFile {
    pub path: PathBuf,
    pub relative_path: String,
    pub file_type: FileType,
    pub language: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Code,
    Config,
    Infrastructure,
    CiCd,
    Documentation,
    Other,
}

impl FileIndexer {
    pub fn new(crawler: RepositoryCrawler) -> Self {
        FileIndexer { crawler }
    }

    /// Index all files in a repository
    pub fn index_repository(&self, repo_url: &str) -> Result<Vec<IndexedFile>> {
        let repo_path = self.crawler.get_repo_path(repo_url);
        
        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Repository not found: {}", repo_url));
        }

        let mut files = Vec::new();
        
        for entry in WalkDir::new(&repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(&repo_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Skip hidden files and common ignore patterns
            if self.should_skip(&relative_path) {
                continue;
            }

            let file_type = self.detect_file_type(&relative_path);
            let language = self.detect_language(&relative_path);
            let size = entry.metadata()?.len();

            files.push(IndexedFile {
                path: path.to_path_buf(),
                relative_path,
                file_type,
                language,
                size,
            });
        }

        Ok(files)
    }

    /// Determine if a file should be skipped
    fn should_skip(&self, path: &str) -> bool {
        // Skip hidden files and directories
        if path.contains("/.") || path.starts_with(".") {
            return true;
        }

        // Skip common ignore patterns
        let ignore_patterns = [
            "node_modules/",
            ".git/",
            "target/",
            "dist/",
            "build/",
            ".next/",
            ".venv/",
            "__pycache__/",
            ".DS_Store",
            "package-lock.json",
            "yarn.lock",
            "Cargo.lock",
        ];

        ignore_patterns.iter().any(|pattern| path.contains(pattern))
    }

    /// Detect file type based on path and extension
    fn detect_file_type(&self, path: &str) -> FileType {
        let path_lower = path.to_lowercase();

        // Infrastructure as Code
        if path_lower.contains("terraform") || 
           path_lower.contains("cloudformation") ||
           path_lower.ends_with(".tf") ||
           path_lower.ends_with(".tfvars") ||
           path_lower.contains("pulumi") ||
           path_lower.contains("serverless.yml") ||
           path_lower.contains("serverless.yaml") {
            return FileType::Infrastructure;
        }

        // CI/CD
        if path_lower.contains(".github/workflows") ||
           path_lower.contains(".gitlab-ci") ||
           path_lower.contains("jenkinsfile") ||
           path_lower.contains("circleci") ||
           path_lower.contains(".travis.yml") ||
           path_lower.contains("azure-pipelines") {
            return FileType::CiCd;
        }

        // Configuration files
        if path_lower.ends_with(".json") ||
           path_lower.ends_with(".yaml") ||
           path_lower.ends_with(".yml") ||
           path_lower.ends_with(".toml") ||
           path_lower.ends_with(".ini") ||
           path_lower.ends_with(".conf") ||
           path_lower.ends_with(".config") ||
           path_lower.contains("package.json") ||
           path_lower.contains("requirements.txt") ||
           path_lower.contains("cargo.toml") ||
           path_lower.contains("pom.xml") ||
           path_lower.contains("go.mod") {
            return FileType::Config;
        }

        // Documentation
        if path_lower.ends_with(".md") ||
           path_lower.ends_with(".rst") ||
           path_lower.ends_with(".txt") ||
           path_lower.contains("readme") ||
           path_lower.contains("license") ||
           path_lower.contains("changelog") {
            return FileType::Documentation;
        }

        // Code files (common extensions)
        if path_lower.ends_with(".rs") ||
           path_lower.ends_with(".go") ||
           path_lower.ends_with(".py") ||
           path_lower.ends_with(".js") ||
           path_lower.ends_with(".ts") ||
           path_lower.ends_with(".jsx") ||
           path_lower.ends_with(".tsx") ||
           path_lower.ends_with(".java") ||
           path_lower.ends_with(".cpp") ||
           path_lower.ends_with(".c") ||
           path_lower.ends_with(".h") ||
           path_lower.ends_with(".cs") ||
           path_lower.ends_with(".php") ||
           path_lower.ends_with(".rb") ||
           path_lower.ends_with(".swift") ||
           path_lower.ends_with(".kt") ||
           path_lower.ends_with(".scala") {
            return FileType::Code;
        }

        FileType::Other
    }

    /// Detect programming language from file extension
    fn detect_language(&self, path: &str) -> Option<String> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())?
            .to_lowercase();

        match ext.as_str() {
            "rs" => Some("rust".to_string()),
            "go" => Some("go".to_string()),
            "py" => Some("python".to_string()),
            "js" | "jsx" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "java" => Some("java".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "c" => Some("c".to_string()),
            "h" | "hpp" => Some("c".to_string()),
            "cs" => Some("csharp".to_string()),
            "php" => Some("php".to_string()),
            "rb" => Some("ruby".to_string()),
            "swift" => Some("swift".to_string()),
            "kt" => Some("kotlin".to_string()),
            "scala" => Some("scala".to_string()),
            "sh" => Some("shell".to_string()),
            "bash" => Some("bash".to_string()),
            "zsh" => Some("zsh".to_string()),
            "fish" => Some("fish".to_string()),
            "ps1" => Some("powershell".to_string()),
            "sql" => Some("sql".to_string()),
            "html" | "htm" => Some("html".to_string()),
            "css" => Some("css".to_string()),
            "scss" | "sass" => Some("scss".to_string()),
            "json" => Some("json".to_string()),
            "yaml" | "yml" => Some("yaml".to_string()),
            "toml" => Some("toml".to_string()),
            "xml" => Some("xml".to_string()),
            "md" => Some("markdown".to_string()),
            "tf" => Some("hcl".to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        let config = crate::config::StorageConfig {
            repository_cache_path: "./cache".to_string(),
            max_cache_size: "10GB".to_string(),
        };
        let crawler = RepositoryCrawler::new(&config).unwrap();
        let indexer = FileIndexer::new(crawler);

        assert_eq!(
            indexer.detect_file_type("src/main.rs"),
            FileType::Code
        );
        assert_eq!(
            indexer.detect_file_type("package.json"),
            FileType::Config
        );
        assert_eq!(
            indexer.detect_file_type(".github/workflows/ci.yml"),
            FileType::CiCd
        );
        assert_eq!(
            indexer.detect_file_type("terraform/main.tf"),
            FileType::Infrastructure
        );
        assert_eq!(
            indexer.detect_file_type("README.md"),
            FileType::Documentation
        );
    }

    #[test]
    fn test_detect_language() {
        let config = crate::config::StorageConfig {
            repository_cache_path: "./cache".to_string(),
            max_cache_size: "10GB".to_string(),
        };
        let crawler = RepositoryCrawler::new(&config).unwrap();
        let indexer = FileIndexer::new(crawler);

        assert_eq!(indexer.detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(indexer.detect_language("app.py"), Some("python".to_string()));
        assert_eq!(indexer.detect_language("index.js"), Some("javascript".to_string()));
        assert_eq!(indexer.detect_language("unknown.xyz"), None);
    }
}

