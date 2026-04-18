use std::fs;
use std::path::PathBuf;

use chrono::Utc;

use crate::config::config_dir;
use crate::types::{AAMediaModel, AAModel, CacheData, MediaCacheData, MediaKind};

const CACHE_TTL_MS: i64 = 24 * 60 * 60 * 1000; // 24 hours

fn llm_cache_path() -> PathBuf {
    config_dir().join("cache.json")
}

fn media_cache_path(kind: MediaKind, include_categories: bool) -> PathBuf {
    let suffix = if include_categories { "-cats" } else { "" };
    config_dir().join(format!("cache-{}{}.json", kind.slug(), suffix))
}

fn read_age_ms(path: &PathBuf) -> Option<i64> {
    let content = fs::read_to_string(path).ok()?;
    let fetched_at = serde_json::from_str::<serde_json::Value>(&content)
        .ok()?
        .get("fetchedAt")?
        .as_str()?
        .to_string();
    let fetched = chrono::DateTime::parse_from_rfc3339(&fetched_at)
        .ok()?
        .with_timezone(&Utc);
    Some(Utc::now().signed_duration_since(fetched).num_milliseconds())
}

fn age_fresh(path: &PathBuf) -> bool {
    read_age_ms(path).is_some_and(|ms| ms <= CACHE_TTL_MS)
}

fn format_age(ms: i64) -> String {
    let hours = ms / (1000 * 60 * 60);
    let mins = (ms % (1000 * 60 * 60)) / (1000 * 60);
    if hours > 0 {
        format!("{hours}h {mins}m ago")
    } else {
        format!("{mins}m ago")
    }
}

pub fn get_cached_models() -> Option<Vec<AAModel>> {
    let path = llm_cache_path();
    if !path.exists() || !age_fresh(&path) {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let data: CacheData = serde_json::from_str(&content).ok()?;
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
        fs::write(llm_cache_path(), json).ok();
    }
}

pub fn get_cached_media(kind: MediaKind, include_categories: bool) -> Option<Vec<AAMediaModel>> {
    let path = media_cache_path(kind, include_categories);
    if !path.exists() || !age_fresh(&path) {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let data: MediaCacheData = serde_json::from_str(&content).ok()?;
    Some(data.models)
}

pub fn set_cached_media(kind: MediaKind, include_categories: bool, models: &[AAMediaModel]) {
    let dir = config_dir();
    fs::create_dir_all(&dir).ok();
    let data = MediaCacheData {
        fetched_at: Utc::now().to_rfc3339(),
        models: models.to_vec(),
    };
    if let Ok(json) = serde_json::to_string(&data) {
        fs::write(media_cache_path(kind, include_categories), json).ok();
    }
}

/// LLM cache age — preserved for the existing `aa models` footer.
pub fn get_cache_age() -> Option<String> {
    read_age_ms(&llm_cache_path()).map(format_age)
}

pub fn get_media_cache_age(kind: MediaKind, include_categories: bool) -> Option<String> {
    read_age_ms(&media_cache_path(kind, include_categories)).map(format_age)
}

/// Clear only the LLM cache.
pub fn clear_cache() -> bool {
    let path = llm_cache_path();
    path.exists() && fs::remove_file(&path).is_ok()
}

/// Clear every cache file (LLM + all media variants). Returns count removed.
pub fn clear_all_caches() -> usize {
    let dir = config_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return 0;
    };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if (name == "cache.json" || (name.starts_with("cache-") && name.ends_with(".json")))
            && fs::remove_file(entry.path()).is_ok() {
                removed += 1;
            }
    }
    removed
}

/// Report the age of every existing cache file (LLM + media variants).
pub fn list_cache_ages() -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    if llm_cache_path().exists() {
        out.push(("LLMs".to_string(), get_cache_age()));
    }
    for kind in MediaKind::all() {
        for cats in [false, true] {
            if cats && !kind.supports_categories() {
                continue;
            }
            let path = media_cache_path(*kind, cats);
            if path.exists() {
                let label = if cats {
                    format!("{} (+ categories)", kind.label())
                } else {
                    kind.label().to_string()
                };
                out.push((label, read_age_ms(&path).map(format_age)));
            }
        }
    }
    out
}
