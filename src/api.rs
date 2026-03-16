use crate::cache::{get_cached_models, set_cached_models};
use crate::config::get_api_key;
use crate::types::AAModel;

const BASE_URL: &str = "https://artificialanalysis.ai/api/v2";

pub fn fetch_models(force_refresh: bool) -> Result<Vec<AAModel>, String> {
    if !force_refresh {
        if let Some(cached) = get_cached_models() {
            return Ok(cached);
        }
    }

    let api_key = get_api_key().ok_or_else(|| {
        "No API key configured. Run `aa auth <key>` or set AA_API_KEY environment variable.\n\
         Get a free key at https://artificialanalysis.ai/account/api"
            .to_string()
    })?;

    let client = reqwest::blocking::Client::new();
    let res = client
        .get(format!("{BASE_URL}/data/llms/models"))
        .header("x-api-key", &api_key)
        .send()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = res.status();
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

    let body: serde_json::Value = res.json().map_err(|e| format!("Failed to parse JSON: {e}"))?;

    let models: Vec<AAModel> = if body.is_array() {
        serde_json::from_value(body).map_err(|e| format!("Failed to parse models: {e}"))?
    } else if let Some(data) = body.get("data") {
        serde_json::from_value(data.clone()).map_err(|e| format!("Failed to parse models: {e}"))?
    } else {
        Vec::new()
    };

    set_cached_models(&models);
    Ok(models)
}
