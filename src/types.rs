use serde::{Deserialize, Serialize};

/// Raw API response types from Artificial Analysis v2 API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AAModelCreator {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AAEvaluations {
    pub artificial_analysis_intelligence_index: Option<f64>,
    pub artificial_analysis_coding_index: Option<f64>,
    pub artificial_analysis_math_index: Option<f64>,
    pub mmlu_pro: Option<f64>,
    pub gpqa: Option<f64>,
    pub hle: Option<f64>,
    pub livecodebench: Option<f64>,
    pub scicode: Option<f64>,
    pub math_500: Option<f64>,
    pub aime: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AAPricing {
    pub price_1m_blended_3_to_1: Option<f64>,
    pub price_1m_input_tokens: Option<f64>,
    pub price_1m_output_tokens: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AAModel {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub model_creator: Option<AAModelCreator>,
    #[serde(default)]
    pub evaluations: Option<AAEvaluations>,
    #[serde(default)]
    pub pricing: Option<AAPricing>,
    pub median_output_tokens_per_second: Option<f64>,
    pub median_time_to_first_token_seconds: Option<f64>,
    pub median_time_to_first_answer_token: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheData {
    #[serde(rename = "fetchedAt")]
    pub fetched_at: String,
    pub models: Vec<AAModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(rename = "envFile", skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
}

/// Processed model for display
#[derive(Debug, Clone)]
pub struct ModelRow {
    pub name: String,
    pub creator: String,
    pub slug: String,
    pub quality: Option<f64>,
    pub coding: Option<f64>,
    pub math: Option<f64>,
    pub input_price: Option<f64>,
    pub output_price: Option<f64>,
    pub blended_price: Option<f64>,
    pub speed: Option<f64>,
    pub ttft: Option<f64>,
}

impl ModelRow {
    pub fn from_api(m: &AAModel) -> Self {
        let evals = m.evaluations.as_ref();
        let pricing = m.pricing.as_ref();
        Self {
            name: m.name.clone(),
            creator: m
                .model_creator
                .as_ref()
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "?".to_string()),
            slug: m.slug.clone(),
            quality: evals.and_then(|e| e.artificial_analysis_intelligence_index),
            coding: evals.and_then(|e| e.artificial_analysis_coding_index),
            math: evals.and_then(|e| e.artificial_analysis_math_index),
            input_price: pricing.and_then(|p| p.price_1m_input_tokens),
            output_price: pricing.and_then(|p| p.price_1m_output_tokens),
            blended_price: pricing.and_then(|p| p.price_1m_blended_3_to_1),
            speed: m.median_output_tokens_per_second,
            ttft: m.median_time_to_first_token_seconds,
        }
    }
}
