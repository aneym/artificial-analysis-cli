#!/usr/bin/env node

import { Command } from "commander";
import { fetchModels } from "./api.js";
import { clearCache, getCacheAge } from "./cache.js";
import { getApiKey, loadConfig, saveConfig } from "./config.js";
import {
  toRow,
  sortRows,
  printModelsTable,
  printCompareTable,
  printModelDetail,
  type SortKey,
} from "./format.js";
import type { ModelRow } from "./types.js";

const program = new Command();

program
  .name("aa")
  .description(
    "Query AI model benchmarks, pricing, and performance from Artificial Analysis",
  )
  .version("0.1.0");

// ── aa models ──────────────────────────────────────────────────────────
program
  .command("models")
  .description("List AI models with intelligence scores, pricing, and speed")
  .option(
    "-s, --sort <key>",
    "Sort by: quality, cost, speed, coding, math, value",
    "quality",
  )
  .option(
    "-f, --filter <name>",
    "Filter models by name (case-insensitive substring)",
  )
  .option("-c, --cheap", "Only show models with output price < $1/M tokens")
  .option("-n, --limit <n>", "Number of models to show (default: 30)")
  .option("-a, --all", "Show all models")
  .option("--refresh", "Force refresh from API (ignores cache)")
  .option("--min-quality <n>", "Minimum intelligence index score")
  .option("--max-cost <n>", "Maximum output price per million tokens")
  .option("--creator <name>", "Filter by creator/provider name")
  .action(async (opts) => {
    try {
      const models = await fetchModels(opts.refresh);
      let rows = models.map(toRow);

      // Filters
      if (opts.filter) {
        const term = opts.filter.toLowerCase();
        rows = rows.filter(
          (r) =>
            r.name.toLowerCase().includes(term) ||
            r.slug.toLowerCase().includes(term),
        );
      }
      if (opts.creator) {
        const term = opts.creator.toLowerCase();
        rows = rows.filter((r) => r.creator.toLowerCase().includes(term));
      }
      if (opts.cheap) {
        rows = rows.filter((r) => r.outputPrice !== null && r.outputPrice < 1);
      }
      if (opts.minQuality) {
        const min = parseFloat(opts.minQuality);
        rows = rows.filter((r) => r.quality !== null && r.quality >= min);
      }
      if (opts.maxCost) {
        const max = parseFloat(opts.maxCost);
        rows = rows.filter(
          (r) => r.outputPrice !== null && r.outputPrice <= max,
        );
      }

      // Sort
      rows = sortRows(rows, opts.sort as SortKey);

      // Display
      const limit = opts.all ? undefined : parseInt(opts.limit ?? "30", 10);
      printModelsTable(rows, { limit });

      const age = getCacheAge();
      if (age) console.log(`  \x1b[90mCache: ${age}\x1b[0m\n`);
    } catch (e) {
      console.error(`\x1b[31mError: ${(e as Error).message}\x1b[0m`);
      process.exit(1);
    }
  });

// ── aa compare ─────────────────────────────────────────────────────────
program
  .command("compare <models...>")
  .description("Compare models side-by-side (use slug or partial name)")
  .option("--refresh", "Force refresh from API")
  .action(async (modelNames: string[], opts) => {
    try {
      const models = await fetchModels(opts.refresh);
      const allRows = models.map(toRow);

      const matched: ModelRow[] = [];
      for (const query of modelNames) {
        const q = query.toLowerCase();
        const found = allRows.find(
          (r) =>
            r.slug.toLowerCase() === q ||
            r.name.toLowerCase() === q ||
            r.slug.toLowerCase().includes(q) ||
            r.name.toLowerCase().includes(q),
        );
        if (found) {
          matched.push(found);
        } else {
          console.warn(
            `\x1b[33mWarning: No model found matching "${query}"\x1b[0m`,
          );
        }
      }

      printCompareTable(matched);
    } catch (e) {
      console.error(`\x1b[31mError: ${(e as Error).message}\x1b[0m`);
      process.exit(1);
    }
  });

// ── aa show ────────────────────────────────────────────────────────────
program
  .command("show <model>")
  .description("Show detailed info for a single model")
  .option("--refresh", "Force refresh from API")
  .action(async (model: string, opts) => {
    try {
      const models = await fetchModels(opts.refresh);
      const allRows = models.map(toRow);
      const q = model.toLowerCase();

      const found = allRows.find(
        (r) =>
          r.slug.toLowerCase() === q ||
          r.name.toLowerCase() === q ||
          r.slug.toLowerCase().includes(q) ||
          r.name.toLowerCase().includes(q),
      );

      if (!found) {
        console.error(`No model found matching "${model}"`);
        process.exit(1);
      }

      printModelDetail(found);
    } catch (e) {
      console.error(`\x1b[31mError: ${(e as Error).message}\x1b[0m`);
      process.exit(1);
    }
  });

// ── aa auth ────────────────────────────────────────────────────────────
program
  .command("auth [key]")
  .description("Set or show API key")
  .action((key?: string) => {
    if (key) {
      const config = loadConfig();
      config.apiKey = key;
      saveConfig(config);
      console.log("\x1b[32mAPI key saved.\x1b[0m");
    } else {
      const current = getApiKey();
      if (current) {
        const masked = current.slice(0, 6) + "..." + current.slice(-4);
        console.log(`Current API key: ${masked}`);
        console.log(
          `Source: ${process.env.AA_API_KEY ? "AA_API_KEY env var" : "config file"}`,
        );
      } else {
        console.log("No API key configured.");
        console.log("Run: aa auth <your-key>");
        console.log("Or set: export AA_API_KEY=<your-key>");
        console.log(
          "Get a free key at https://artificialanalysis.ai/account/api",
        );
      }
    }
  });

// ── aa cache ───────────────────────────────────────────────────────────
program
  .command("cache")
  .description("Show cache status or clear it")
  .option("--clear", "Clear the cache")
  .action((opts) => {
    if (opts.clear) {
      const cleared = clearCache();
      console.log(cleared ? "Cache cleared." : "No cache to clear.");
    } else {
      const age = getCacheAge();
      console.log(
        age ? `Cache age: ${age}` : "No cache (will fetch on next request)",
      );
    }
  });

program.parse();
