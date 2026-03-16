import {
  readFileSync,
  writeFileSync,
  mkdirSync,
  existsSync,
  unlinkSync,
} from "node:fs";
import { join } from "node:path";
import { getConfigDir } from "./config.js";
import type { AAModel, CacheData } from "./types.js";

const CACHE_FILE = join(getConfigDir(), "cache.json");
const CACHE_TTL_MS = 24 * 60 * 60 * 1000; // 24 hours

export function getCachedModels(): AAModel[] | null {
  if (!existsSync(CACHE_FILE)) return null;
  try {
    const data: CacheData = JSON.parse(readFileSync(CACHE_FILE, "utf-8"));
    const age = Date.now() - new Date(data.fetchedAt).getTime();
    if (age > CACHE_TTL_MS) return null;
    return data.models;
  } catch {
    return null;
  }
}

export function setCachedModels(models: AAModel[]): void {
  mkdirSync(getConfigDir(), { recursive: true });
  const data: CacheData = { fetchedAt: new Date().toISOString(), models };
  writeFileSync(CACHE_FILE, JSON.stringify(data));
}

export function clearCache(): boolean {
  if (existsSync(CACHE_FILE)) {
    unlinkSync(CACHE_FILE);
    return true;
  }
  return false;
}

export function getCacheAge(): string | null {
  if (!existsSync(CACHE_FILE)) return null;
  try {
    const data: CacheData = JSON.parse(readFileSync(CACHE_FILE, "utf-8"));
    const ageMs = Date.now() - new Date(data.fetchedAt).getTime();
    const hours = Math.floor(ageMs / (1000 * 60 * 60));
    const mins = Math.floor((ageMs % (1000 * 60 * 60)) / (1000 * 60));
    if (hours > 0) return `${hours}h ${mins}m ago`;
    return `${mins}m ago`;
  } catch {
    return null;
  }
}
