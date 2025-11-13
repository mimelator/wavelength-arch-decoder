use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: Option<String>,
    pub update_available: bool,
    pub last_checked: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    published_at: String,
}

// Cache the version check result for 24 hours
static VERSION_CACHE: Lazy<Mutex<Option<(VersionInfo, u64)>>> = Lazy::new(|| Mutex::new(None));

const CACHE_DURATION_SECONDS: u64 = 24 * 60 * 60; // 24 hours
const GITHUB_API_URL: &str = "https://api.github.com/repos/mimelator/wavelength-arch-decoder/releases/latest";

/// Get current version from VERSION file or env
pub fn get_current_version() -> String {
    match std::fs::read_to_string("VERSION") {
        Ok(v) => v.trim().to_string(),
        Err(_) => {
            match std::fs::read_to_string("../VERSION") {
                Ok(v) => v.trim().to_string(),
                Err(_) => {
                    std::env::var("WAVELENGTH_VERSION")
                        .unwrap_or_else(|_| "0.7.3".to_string())
                }
            }
        }
    }
}

/// Compare two semantic version strings
/// Returns true if latest > current
fn is_newer_version(current: &str, latest: &str) -> bool {
    // Remove 'v' prefix if present
    let current = current.trim_start_matches('v').trim();
    let latest = latest.trim_start_matches('v').trim();
    
    let current_parts: Vec<&str> = current.split('.').collect();
    let latest_parts: Vec<&str> = latest.split('.').collect();
    
    // Compare major, minor, patch
    for i in 0..3 {
        let current_num = current_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let latest_num = latest_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        
        if latest_num > current_num {
            return true;
        } else if latest_num < current_num {
            return false;
        }
    }
    
    false // Versions are equal
}

/// Check GitHub for latest release version
async fn fetch_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("wavelength-arch-decoder")
        .timeout(Duration::from_secs(5))
        .build()?;
    
    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()).into());
    }
    
    let release: GitHubRelease = response.json().await?;
    Ok(release.tag_name)
}

/// Check for updates (with caching)
pub async fn check_for_updates(force: bool) -> VersionInfo {
    let current = get_current_version();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Check cache first (unless forced)
    if !force {
        if let Ok(cache) = VERSION_CACHE.lock() {
            if let Some((cached_info, cached_time)) = cache.as_ref() {
                if now - cached_time < CACHE_DURATION_SECONDS {
                    return cached_info.clone();
                }
            }
        }
    }
    
    // Check if version checking is disabled
    let check_enabled = std::env::var("CHECK_VERSION_UPDATES")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase() == "true";
    
    if !check_enabled {
        return VersionInfo {
            current: current.clone(),
            latest: None,
            update_available: false,
            last_checked: Some(now),
        };
    }
    
    // Fetch latest version from GitHub
    let latest_result = fetch_latest_version().await;
    
    let (latest, update_available) = match latest_result {
        Ok(latest_version) => {
            let update_available = is_newer_version(&current, &latest_version);
            (Some(latest_version), update_available)
        }
        Err(e) => {
            log::debug!("Failed to check for updates: {}", e);
            (None, false)
        }
    };
    
    let version_info = VersionInfo {
        current: current.clone(),
        latest,
        update_available,
        last_checked: Some(now),
    };
    
    // Update cache
    if let Ok(mut cache) = VERSION_CACHE.lock() {
        *cache = Some((version_info.clone(), now));
    }
    
    version_info
}

