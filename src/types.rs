use serde::{Deserialize, Serialize};

// ============ LLMs ============

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

// ============ Media (TTS / Image / Video / etc.) ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AAMediaCreator {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AAMediaCategory {
    #[serde(default)]
    pub style_category: Option<String>,
    #[serde(default)]
    pub subject_matter_category: Option<String>,
    #[serde(default)]
    pub format_category: Option<String>,
    #[serde(default)]
    pub elo: Option<f64>,
    #[serde(default)]
    pub ci95: Option<String>,
    #[serde(default)]
    pub appearances: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AAMediaModel {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub model_creator: Option<AAMediaCreator>,
    #[serde(default)]
    pub elo: Option<f64>,
    #[serde(default)]
    pub rank: Option<u64>,
    #[serde(default)]
    pub ci95: Option<String>,
    #[serde(default)]
    pub appearances: Option<u64>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub categories: Option<Vec<AAMediaCategory>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCacheData {
    #[serde(rename = "fetchedAt")]
    pub fetched_at: String,
    pub models: Vec<AAMediaModel>,
}

// ============ Config ============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(rename = "envFile", skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
}

// ============ Display rows ============

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

// ============ Media kinds ============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    TextToSpeech,
    TextToImage,
    ImageEditing,
    TextToVideo,
    ImageToVideo,
}

impl MediaKind {
    pub fn all() -> &'static [MediaKind] {
        &[
            MediaKind::TextToSpeech,
            MediaKind::TextToImage,
            MediaKind::ImageEditing,
            MediaKind::TextToVideo,
            MediaKind::ImageToVideo,
        ]
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "tts" | "voice" | "speech" | "text-to-speech" => Ok(Self::TextToSpeech),
            "image" | "img" | "text-to-image" | "t2i" => Ok(Self::TextToImage),
            "image-edit" | "img-edit" | "imgedit" | "image-editing" | "edit" => {
                Ok(Self::ImageEditing)
            }
            "video" | "text-to-video" | "t2v" => Ok(Self::TextToVideo),
            "img2vid" | "image-to-video" | "i2v" => Ok(Self::ImageToVideo),
            _ => Err(format!(
                "Invalid media kind '{s}'. Valid: tts, image, image-edit, video, img2vid"
            )),
        }
    }

    pub fn path(&self) -> &'static str {
        match self {
            Self::TextToSpeech => "data/media/text-to-speech",
            Self::TextToImage => "data/media/text-to-image",
            Self::ImageEditing => "data/media/image-editing",
            Self::TextToVideo => "data/media/text-to-video",
            Self::ImageToVideo => "data/media/image-to-video",
        }
    }

    /// Filename-safe slug used for per-endpoint cache files.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::TextToSpeech => "tts",
            Self::TextToImage => "image",
            Self::ImageEditing => "image-edit",
            Self::TextToVideo => "video",
            Self::ImageToVideo => "img2vid",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::TextToSpeech => "Text-to-Speech",
            Self::TextToImage => "Text-to-Image",
            Self::ImageEditing => "Image Editing",
            Self::TextToVideo => "Text-to-Video",
            Self::ImageToVideo => "Image-to-Video",
        }
    }

    /// Whether this endpoint accepts `?include_categories=true`.
    pub fn supports_categories(&self) -> bool {
        matches!(
            self,
            Self::TextToImage | Self::TextToVideo | Self::ImageToVideo
        )
    }
}
