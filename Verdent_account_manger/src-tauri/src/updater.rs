use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const GITHUB_API_URL: &str = "https://api.github.com/repos/chaogei/verdent-account-manager/releases/latest";
const UPDATE_CONFIG_FILE: &str = "update_config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub skipped_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_name: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub published_at: Option<String>,
}

pub async fn check_for_updates(current_version: &str) -> Result<UpdateInfo, String> {
    let client = reqwest::Client::builder()
        .user_agent("Verdent-Account-Manager")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    
    let skipped_version = load_update_config()
        .ok()
        .and_then(|config| config.skipped_version);

    if let Some(ref skipped) = skipped_version {
        if skipped == &latest_version {
            return Ok(UpdateInfo {
                has_update: false,
                current_version: current_version.to_string(),
                latest_version: Some(latest_version),
                release_name: None,
                release_notes: None,
                download_url: None,
                published_at: None,
            });
        }
    }

    let has_update = compare_versions(current_version, &latest_version);

    Ok(UpdateInfo {
        has_update,
        current_version: current_version.to_string(),
        latest_version: Some(latest_version),
        release_name: Some(release.name),
        release_notes: release.body,
        download_url: Some(release.html_url),
        published_at: Some(release.published_at),
    })
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let current_part = current_parts.get(i).unwrap_or(&0);
        let latest_part = latest_parts.get(i).unwrap_or(&0);

        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false;
        }
    }

    false
}

fn get_update_config_path() -> Result<PathBuf, String> {
    let app_data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Failed to get app data directory".to_string())?;
    
    let config_dir = app_data_dir.join("verdent_accounts");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    Ok(config_dir.join(UPDATE_CONFIG_FILE))
}

fn load_update_config() -> Result<UpdateConfig, String> {
    let config_path = get_update_config_path()?;
    
    if !config_path.exists() {
        return Ok(UpdateConfig {
            skipped_version: None,
        });
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read update config: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse update config: {}", e))
}

pub fn save_skipped_version(version: String) -> Result<(), String> {
    let config = UpdateConfig {
        skipped_version: Some(version),
    };

    let config_path = get_update_config_path()?;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write update config: {}", e))?;

    Ok(())
}

pub fn clear_skipped_version() -> Result<(), String> {
    let config = UpdateConfig {
        skipped_version: None,
    };

    let config_path = get_update_config_path()?;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write update config: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(compare_versions("1.0.0", "1.0.1"));
        assert!(compare_versions("1.0.0", "1.1.0"));
        assert!(compare_versions("1.0.0", "2.0.0"));
        assert!(!compare_versions("1.0.1", "1.0.0"));
        assert!(!compare_versions("1.1.0", "1.0.0"));
        assert!(!compare_versions("2.0.0", "1.0.0"));
        assert!(!compare_versions("1.0.0", "1.0.0"));
        assert!(compare_versions("1.5.2", "1.6.0"));
        assert!(!compare_versions("1.6.0", "1.5.2"));
    }
}
