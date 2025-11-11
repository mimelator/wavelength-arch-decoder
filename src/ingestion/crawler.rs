use anyhow::Result;
use git2::{Repository, FetchOptions, RemoteCallbacks};
use std::path::{Path, PathBuf};
use std::fs;
use crate::config::StorageConfig;

pub struct RepositoryCrawler {
    cache_path: PathBuf,
}

impl RepositoryCrawler {
    pub fn new(config: &StorageConfig) -> Result<Self> {
        let cache_path = PathBuf::from(&config.repository_cache_path);
        
        // Ensure cache directory exists
        fs::create_dir_all(&cache_path)?;
        
        Ok(RepositoryCrawler {
            cache_path,
        })
    }

    /// Clone or update a repository from a URL
    pub fn clone_or_update(&self, url: &str, branch: Option<&str>) -> Result<PathBuf> {
        let repo_name = self.extract_repo_name(url);
        let repo_path = self.cache_path.join(&repo_name);
        let branch = branch.unwrap_or("main");

        if repo_path.exists() {
            // Update existing repository
            self.update_repository(&repo_path, branch)?;
        } else {
            // Clone new repository
            self.clone_repository(url, &repo_path, branch)?;
        }

        Ok(repo_path)
    }

    /// Clone a repository
    fn clone_repository(&self, url: &str, path: &Path, branch: &str) -> Result<()> {
        log::info!("Cloning repository: {} to {}", url, path.display());
        
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);
        
        // Set up callbacks for authentication if needed
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Try to use SSH key or token from environment
            if let Ok(ssh_key_path) = std::env::var("SSH_KEY_PATH") {
                git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    Path::new(&ssh_key_path),
                    None,
                )
            } else if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                git2::Cred::userpass_plaintext(&token, "")
            } else {
                git2::Cred::default()
            }
        });

        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.branch(branch);
        
        builder.clone(url, path)?;
        
        log::info!("Successfully cloned repository to {}", path.display());
        Ok(())
    }

    /// Update an existing repository
    fn update_repository(&self, path: &Path, branch: &str) -> Result<()> {
        log::info!("Updating repository at {}", path.display());
        
        let repo = Repository::open(path)?;
        
        // Fetch latest changes
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", "origin"))?;
        
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            if let Ok(ssh_key_path) = std::env::var("SSH_KEY_PATH") {
                git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    Path::new(&ssh_key_path),
                    None,
                )
            } else if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                git2::Cred::userpass_plaintext(&token, "")
            } else {
                git2::Cred::default()
            }
        });
        fetch_options.remote_callbacks(callbacks);
        
        remote.fetch(&[branch], Some(&mut fetch_options), None)?;
        
        // Checkout the branch
        let reference = format!("refs/remotes/origin/{}", branch);
        let oid = repo.refname_to_id(&reference)?;
        let object = repo.find_object(oid, None)?;
        repo.checkout_tree(&object, None)?;
        
        // Update HEAD
        repo.set_head(&reference)?;
        
        log::info!("Successfully updated repository");
        Ok(())
    }

    /// Extract repository name from URL
    fn extract_repo_name(&self, url: &str) -> String {
        url.split('/')
            .last()
            .unwrap_or("repository")
            .trim_end_matches(".git")
            .to_string()
    }

    /// Get the repository path for a given URL
    pub fn get_repo_path(&self, url: &str) -> PathBuf {
        let repo_name = self.extract_repo_name(url);
        self.cache_path.join(&repo_name)
    }

    /// Check if repository exists in cache
    pub fn repository_exists(&self, url: &str) -> bool {
        self.get_repo_path(url).exists()
    }

    /// Remove a repository from cache
    pub fn remove_repository(&self, url: &str) -> Result<()> {
        let repo_path = self.get_repo_path(url);
        if repo_path.exists() {
            fs::remove_dir_all(&repo_path)?;
            log::info!("Removed repository cache: {}", repo_path.display());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_repo_name() {
        let config = StorageConfig {
            repository_cache_path: "./cache".to_string(),
            max_cache_size: "10GB".to_string(),
        };
        let crawler = RepositoryCrawler::new(&config).unwrap();
        
        assert_eq!(
            crawler.extract_repo_name("https://github.com/user/repo.git"),
            "repo"
        );
        assert_eq!(
            crawler.extract_repo_name("https://github.com/user/repo"),
            "repo"
        );
    }
}

