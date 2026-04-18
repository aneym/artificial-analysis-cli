use std::fs;
use std::path::PathBuf;

use crate::types::Config;

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".config")
        .join("artificial-analysis")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> Config {
    let path = config_path();
    if !path.exists() {
        return Config::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &Config) {
    let dir = config_dir();
    fs::create_dir_all(&dir).expect("failed to create config directory");
    let json = serde_json::to_string_pretty(config).expect("failed to serialize config");
    fs::write(config_path(), format!("{json}\n")).expect("failed to write config file");
}

/// Read a key from a .env file by name.
/// Supports KEY=value and KEY="value" formats, skips comments and blank lines.
fn read_key_from_env_file(path: &str, key_name: &str) -> Option<String> {
    let expanded = if path.starts_with('~') {
        dirs::home_dir()
            .map(|h| path.replacen('~', &h.to_string_lossy(), 1))
            .unwrap_or_else(|| path.to_string())
    } else {
        path.to_string()
    };

    let contents = fs::read_to_string(&expanded).ok()?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=')
            && k.trim() == key_name {
                let val = v.trim();
                // Strip surrounding quotes
                let unquoted = if (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''))
                {
                    &val[1..val.len() - 1]
                } else {
                    val
                };
                if !unquoted.is_empty() {
                    return Some(unquoted.to_string());
                }
            }
    }
    None
}

/// Resolve API key with precedence:
/// 1. AA_API_KEY env var
/// 2. Config file's stored api_key
/// 3. Config file's env_file path → read AA_API_KEY from it
pub fn get_api_key() -> Option<String> {
    // 1. Environment variable
    if let Ok(val) = std::env::var("AA_API_KEY")
        && !val.is_empty() {
            return Some(val);
        }

    let cfg = load_config();

    // 2. Direct key in config
    if let Some(ref key) = cfg.api_key
        && !key.is_empty() {
            return Some(key.clone());
        }

    // 3. env_file path in config
    if let Some(ref env_path) = cfg.env_file
        && let Some(key) = read_key_from_env_file(env_path, "AA_API_KEY") {
            return Some(key);
        }

    None
}

/// Describe where the API key is coming from (for display)
pub fn get_api_key_source() -> &'static str {
    if std::env::var("AA_API_KEY")
        .ok()
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return "AA_API_KEY env var";
    }

    let cfg = load_config();

    if cfg.api_key.as_ref().is_some_and(|k| !k.is_empty()) {
        return "config file (direct key)";
    }

    if cfg.env_file.is_some() {
        return "config file (env-file reference)";
    }

    "not configured"
}
