/** Raw API response types from Artificial Analysis v2 API */

export interface AAModelCreator {
  id: string;
  name: string;
  slug: string;
}

export interface AAEvaluations {
  artificial_analysis_intelligence_index?: number | null;
  artificial_analysis_coding_index?: number | null;
  artificial_analysis_math_index?: number | null;
  mmlu_pro?: number | null;
  gpqa?: number | null;
  hle?: number | null;
  livecodebench?: number | null;
  scicode?: number | null;
  math_500?: number | null;
  aime?: number | null;
}

export interface AAPricing {
  price_1m_blended_3_to_1?: number | null;
  price_1m_input_tokens?: number | null;
  price_1m_output_tokens?: number | null;
}

export interface AAModel {
  id: string;
  name: string;
  slug: string;
  model_creator: AAModelCreator;
  evaluations: AAEvaluations;
  pricing: AAPricing;
  median_output_tokens_per_second?: number | null;
  median_time_to_first_token_seconds?: number | null;
  median_time_to_first_answer_token?: number | null;
}

export interface AAApiResponse {
  data: AAModel[];
}

/** Processed model for display */
export interface ModelRow {
  name: string;
  creator: string;
  slug: string;
  quality: number | null;
  coding: number | null;
  math: number | null;
  inputPrice: number | null;
  outputPrice: number | null;
  blendedPrice: number | null;
  speed: number | null;
  ttft: number | null;
}

export interface CacheData {
  fetchedAt: string;
  models: AAModel[];
}

export interface Config {
  apiKey?: string;
}
