use anyhow::Result;
use git2::{Repository, FetchOptions, RemoteCallbacks, Cred};
use std::path::{Path, PathBuf};
use std::fs;
use crate::config::StorageConfig;

pub struct RepositoryCrawler {
    cache_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RepositoryCredentials {
    pub auth_type: AuthType,
}

#[derive(Debug, Clone)]
pub enum AuthType {
    SshKey(String),           // Path to SSH key
    Token(String),            // Personal access token
    UsernamePassword(String, String), // Username and password
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
    pub fn clone_or_update(&self, url: &str, branch: Option<&str>, credentials: Option<&RepositoryCredentials>) -> Result<PathBuf> {
        let repo_name = self.extract_repo_name(url);
        let repo_path = self.cache_path.join(&repo_name);
        let branch = branch.unwrap_or("main");

        if repo_path.exists() {
            // Update existing repository
            self.update_repository(&repo_path, branch, credentials)?;
        } else {
            // Clone new repository
            self.clone_repository(url, &repo_path, branch, credentials)?;
        }

        Ok(repo_path)
    }

    /// Clone a repository
    fn clone_repository(&self, url: &str, path: &Path, branch: &str, credentials: Option<&RepositoryCredentials>) -> Result<()> {
        log::info!("Cloning repository: {} to {}", url, path.display());
        
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);
        
        // Set up callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        let creds = credentials.cloned();
        callbacks.credentials(move |url_str, username_from_url, allowed_types| {
            Self::get_credentials(url_str, username_from_url, allowed_types, creds.as_ref())
        });

        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.branch(branch);
        
        // Handle HTTPS URLs with tokens by embedding them in the URL
        let final_url = if let Some(creds) = credentials {
            Self::embed_credentials_in_url(url, creds)?
        } else {
            url.to_string()
        };
        
        builder.clone(&final_url, path)?;
        
        log::info!("Successfully cloned repository to {}", path.display());
        Ok(())
    }

    /// Update an existing repository
    fn update_repository(&self, path: &Path, branch: &str, credentials: Option<&RepositoryCredentials>) -> Result<()> {
        log::info!("Updating repository at {}", path.display());
        
        let repo = Repository::open(path)?;
        
        // Fetch latest changes
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.remote("origin", "origin"))?;
        
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        let creds = credentials.cloned();
        callbacks.credentials(move |url_str, username_from_url, allowed_types| {
            Self::get_credentials(url_str, username_from_url, allowed_types, creds.as_ref())
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

    /// Get credentials for authentication
    fn get_credentials(
        url_str: &str,
        username_from_url: Option<&str>,
        allowed_types: git2::CredentialType,
        credentials: Option<&RepositoryCredentials>,
    ) -> Result<Cred, git2::Error> {
        // First try repository-specific credentials
        if let Some(creds) = credentials {
            match &creds.auth_type {
                AuthType::SshKey(key_path) => {
                    if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                        return Cred::ssh_key(
                            username_from_url.unwrap_or("git"),
                            None,
                            Path::new(key_path),
                            None,
                        );
                    }
                }
                AuthType::Token(token) => {
                    if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                        // For GitHub/GitLab, use token as username
                        return Cred::userpass_plaintext(token, "");
                    }
                }
                AuthType::UsernamePassword(username, password) => {
                    if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                        return Cred::userpass_plaintext(username, password);
                    }
                }
            }
        }

        // Fall back to environment variables
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            if let Ok(ssh_key_path) = std::env::var("SSH_KEY_PATH") {
                return Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    Path::new(&ssh_key_path),
                    None,
                );
            }
        }

        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            // Try provider-specific tokens
            if url_str.contains("github.com") {
                if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                    return Cred::userpass_plaintext(&token, "");
                }
            } else if url_str.contains("gitlab.com") {
                if let Ok(token) = std::env::var("GITLAB_TOKEN") {
                    return Cred::userpass_plaintext(&token, "");
                }
            } else if url_str.contains("bitbucket.org") {
                if let Ok(token) = std::env::var("BITBUCKET_TOKEN") {
                    return Cred::userpass_plaintext(&token, "");
                }
            }
        }

        // Default credentials
        Cred::default()
    }

    /// Embed credentials in URL for HTTPS cloning
    fn embed_credentials_in_url(url: &str, credentials: &RepositoryCredentials) -> Result<String> {
        if url.starts_with("https://") {
            match &credentials.auth_type {
                AuthType::Token(token) => {
                    // For GitHub: https://token@github.com/user/repo.git
                    // For GitLab: https://oauth2:token@gitlab.com/user/repo.git
                    if url.contains("github.com") {
                        Ok(url.replacen("https://", &format!("https://{}@", token), 1))
                    } else if url.contains("gitlab.com") {
                        Ok(url.replacen("https://", &format!("https://oauth2:{}@", token), 1))
                    } else if url.contains("bitbucket.org") {
                        Ok(url.replacen("https://", &format!("https://x-token-auth:{}@", token), 1))
                    } else {
                        Ok(url.to_string())
                    }
                }
                AuthType::UsernamePassword(username, password) => {
                    Ok(url.replacen("https://", &format!("https://{}:{}@", username, password), 1))
                }
                _ => Ok(url.to_string()),
            }
        } else {
            Ok(url.to_string())
        }
    }

    /// Extract repository name from URL
    fn extract_repo_name(&self, url: &str) -> String {
        // Remove credentials from URL if present
        let clean_url = url.split('@').last().unwrap_or(url);
        clean_url.split('/')
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
        assert_eq!(
            crawler.extract_repo_name("git@github.com:user/repo.git"),
            "repo"
        );
    }
}
