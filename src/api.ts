import { getApiKey } from "./config.js";
import { getCachedModels, setCachedModels } from "./cache.js";
import type { AAModel } from "./types.js";

const BASE_URL = "https://artificialanalysis.ai/api/v2";

export async function fetchModels(forceRefresh = false): Promise<AAModel[]> {
  if (!forceRefresh) {
    const cached = getCachedModels();
    if (cached) return cached;
  }

  const apiKey = getApiKey();
  if (!apiKey) {
    throw new Error(
      "No API key configured. Run `aa auth <key>` or set AA_API_KEY environment variable.\n" +
        "Get a free key at https://artificialanalysis.ai/account/api",
    );
  }

  const res = await fetch(`${BASE_URL}/data/llms/models`, {
    headers: { "x-api-key": apiKey },
  });

  if (res.status === 401) {
    throw new Error(
      "Invalid API key. Check your key at https://artificialanalysis.ai/account/api",
    );
  }
  if (res.status === 429) {
    throw new Error(
      "Rate limit exceeded (25 requests/day on free tier). Try again tomorrow or use cached data.",
    );
  }
  if (!res.ok) {
    throw new Error(`API error: ${res.status} ${res.statusText}`);
  }

  const body = await res.json();
  const models: AAModel[] = Array.isArray(body) ? body : (body.data ?? []);

  setCachedModels(models);
  return models;
}
