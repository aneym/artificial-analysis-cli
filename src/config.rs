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

pub fn get_api_key() -> Option<String> {
    std::env::var("AA_API_KEY")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| load_config().api_key)
}
