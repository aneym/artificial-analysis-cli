use crate::cache::{get_cached_media, get_cached_models, set_cached_media, set_cached_models};
use crate::config::get_api_key;
use crate::types::{AAMediaModel, AAModel, MediaKind};

const BASE_URL: &str = "https://artificialanalysis.ai/api/v2";

fn require_api_key() -> Result<String, String> {
    get_api_key().ok_or_else(|| {
        "No API key configured. Run `aa auth <key>` or set AA_API_KEY environment variable.\n\
         Get a free key at https://artificialanalysis.ai/account/api"
            .to_string()
    })
}

fn check_status(status: reqwest::StatusCode) -> Result<(), String> {
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(
            "Invalid API key. Check your key at https://artificialanalysis.ai/account/api"
                .to_string(),
        );
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(
            "Rate limit exceeded (25 requests/day on free tier). Try again tomorrow or use cached data."
                .to_string(),
        );
    }
    if !status.is_success() {
        return Err(format!("API error: {status}"));
    }
    Ok(())
}

fn http_get(path: &str, query: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    let api_key = require_api_key()?;
    let url = format!("{BASE_URL}/{}", path.trim_start_matches('/'));
    let client = reqwest::blocking::Client::new();
    let res = client
        .get(&url)
        .header("x-api-key", &api_key)
        .query(query)
        .send()
        .map_err(|e| format!("HTTP request failed: {e}"))?;
    check_status(res.status())?;
    res.json().map_err(|e| format!("Failed to parse JSON: {e}"))
}

fn unwrap_data(body: serde_json::Value) -> serde_json::Value {
    if body.is_array() {
        body
    } else if let Some(data) = body.get("data") {
        data.clone()
    } else {
        serde_json::Value::Array(Vec::new())
    }
}

pub fn fetch_models(force_refresh: bool) -> Result<Vec<AAModel>, String> {
    if !force_refresh
        && let Some(cached) = get_cached_models() {
            return Ok(cached);
        }
    let body = http_get("data/llms/models", &[])?;
    let data = unwrap_data(body);
    let models: Vec<AAModel> =
        serde_json::from_value(data).map_err(|e| format!("Failed to parse models: {e}"))?;
    set_cached_models(&models);
    Ok(models)
}

pub fn fetch_media(
    kind: MediaKind,
    include_categories: bool,
    force_refresh: bool,
) -> Result<Vec<AAMediaModel>, String> {
    if !force_refresh
        && let Some(cached) = get_cached_media(kind, include_categories) {
            return Ok(cached);
        }
    let query: Vec<(&str, &str)> = if include_categories && kind.supports_categories() {
        vec![("include_categories", "true")]
    } else {
        vec![]
    };
    let body = http_get(kind.path(), &query)?;
    let data = unwrap_data(body);
    let models: Vec<AAMediaModel> = serde_json::from_value(data)
        .map_err(|e| format!("Failed to parse {} models: {e}", kind.label()))?;
    set_cached_media(kind, include_categories, &models);
    Ok(models)
}

/// Raw passthrough: call any AA v2 path with arbitrary query parameters.
/// Used by `aa raw <path>` to stay forward-compatible with unwrapped endpoints.
pub fn fetch_raw(path: &str, query: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    http_get(path, query)
}
