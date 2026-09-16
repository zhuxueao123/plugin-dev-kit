use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

const CONFIG_DIR_NAME: &str = "asapflow";
const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StoredConfig {
    pub base_url: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub base_url: String,
    pub token: String,
}

pub fn config_path() -> Result<PathBuf> {
    if let Some(path) = executable_config_path() {
        return Ok(path);
    }

    user_config_path()
}

fn user_config_path() -> Result<PathBuf> {
    let base =
        dirs::config_dir().ok_or_else(|| anyhow!("Unable to resolve local config directory."))?;
    Ok(base.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
}

fn executable_config_path() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let parent = executable.parent()?;
    Some(parent.join(CONFIG_FILE_NAME))
}

pub fn load_config() -> Result<StoredConfig> {
    if let Some(path) = executable_config_path() {
        if path.exists() {
            return load_config_from_path(&path);
        }
    }

    let path = user_config_path()?;
    if path.exists() {
        return load_config_from_path(&path);
    }

    Ok(StoredConfig::default())
}

pub fn save_config(config: &StoredConfig) -> Result<PathBuf> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let raw = serde_json::to_string_pretty(config)?;
    fs::write(&path, raw)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    Ok(path)
}

fn load_config_from_path(path: &PathBuf) -> Result<StoredConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let config = serde_json::from_str::<StoredConfig>(strip_utf8_bom(&raw))
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    Ok(config)
}

pub fn resolve(base_url_arg: Option<&str>, token_arg: Option<&str>) -> Result<ResolvedConfig> {
    let stored = load_config()?;
    let env_base_url = std::env::var("ASAPFLOW_BASE_URL").ok();
    let env_token = std::env::var("ASAPFLOW_TOKEN").ok();

    let base_url = base_url_arg
        .map(str::to_string)
        .or(env_base_url)
        .or(stored.base_url)
        .ok_or_else(|| {
            anyhow!("Missing base URL. Use --base-url, config file, or ASAPFLOW_BASE_URL.")
        })?;

    let token = token_arg
        .map(str::to_string)
        .or(env_token)
        .or(stored.token)
        .ok_or_else(|| anyhow!("Missing token. Use --token, config file, or ASAPFLOW_TOKEN."))?;

    Ok(ResolvedConfig {
        base_url: normalize_base_url(&base_url),
        token,
    })
}

pub fn normalize_base_url(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

fn strip_utf8_bom(value: &str) -> &str {
    value.strip_prefix('\u{feff}').unwrap_or(value)
}
