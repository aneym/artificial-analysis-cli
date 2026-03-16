use std::fs;
use std::path::PathBuf;

use chrono::Utc;

use crate::config::config_dir;
use crate::types::{AAModel, CacheData};

const CACHE_TTL_MS: i64 = 24 * 60 * 60 * 1000; // 24 hours

fn cache_path() -> PathBuf {
    config_dir().join("cache.json")
}

pub fn get_cached_models() -> Option<Vec<AAModel>> {
    let path = cache_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let data: CacheData = serde_json::from_str(&content).ok()?;
    let fetched = chrono::DateTime::parse_from_rfc3339(&data.fetched_at)
        .ok()?
        .with_timezone(&Utc);
    let age_ms = Utc::now().signed_duration_since(fetched).num_milliseconds();
    if age_ms > CACHE_TTL_MS {
        return None;
    }
    Some(data.models)
}

pub fn set_cached_models(models: &[AAModel]) {
    let dir = config_dir();
    fs::create_dir_all(&dir).ok();
    let data = CacheData {
        fetched_at: Utc::now().to_rfc3339(),
        models: models.to_vec(),
    };
    if let Ok(json) = serde_json::to_string(&data) {
        fs::write(cache_path(), json).ok();
    }
}

pub fn clear_cache() -> bool {
    let path = cache_path();
    if path.exists() {
        fs::remove_file(&path).is_ok()
    } else {
        false
    }
}

pub fn get_cache_age() -> Option<String> {
    let path = cache_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let data: CacheData = serde_json::from_str(&content).ok()?;
    let fetched = chrono::DateTime::parse_from_rfc3339(&data.fetched_at)
        .ok()?
        .with_timezone(&Utc);
    let age_ms = Utc::now().signed_duration_since(fetched).num_milliseconds();
    let hours = age_ms / (1000 * 60 * 60);
    let mins = (age_ms % (1000 * 60 * 60)) / (1000 * 60);
    if hours > 0 {
        Some(format!("{hours}h {mins}m ago"))
    } else {
        Some(format!("{mins}m ago"))
    }
}
