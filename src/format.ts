import type { AAModel, ModelRow } from "./types.js";

// ANSI color helpers (no dependencies)
const c = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  dim: "\x1b[2m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  cyan: "\x1b[36m",
  white: "\x1b[37m",
  gray: "\x1b[90m",
  bgBlue: "\x1b[44m",
};

export function toRow(m: AAModel): ModelRow {
  return {
    name: m.name,
    creator: m.model_creator?.name ?? "?",
    slug: m.slug,
    quality: m.evaluations?.artificial_analysis_intelligence_index ?? null,
    coding: m.evaluations?.artificial_analysis_coding_index ?? null,
    math: m.evaluations?.artificial_analysis_math_index ?? null,
    inputPrice: m.pricing?.price_1m_input_tokens ?? null,
    outputPrice: m.pricing?.price_1m_output_tokens ?? null,
    blendedPrice: m.pricing?.price_1m_blended_3_to_1 ?? null,
    speed: m.median_output_tokens_per_second ?? null,
    ttft: m.median_time_to_first_token_seconds ?? null,
  };
}

function fmtNum(n: number | null, decimals = 1): string {
  if (n === null || n === undefined) return c.dim + "-" + c.reset;
  return n.toFixed(decimals);
}

function fmtPrice(n: number | null): string {
  if (n === null || n === undefined) return c.dim + "-" + c.reset;
  if (n < 0.1) return c.green + "$" + n.toFixed(3) + c.reset;
  if (n < 1) return c.green + "$" + n.toFixed(2) + c.reset;
  if (n < 5) return c.yellow + "$" + n.toFixed(2) + c.reset;
  return "$" + n.toFixed(2);
}

function pad(s: string, len: number, align: "left" | "right" = "left"): string {
  // Strip ANSI codes for length calculation
  const visible = s.replace(/\x1b\[[0-9;]*m/g, "");
  const diff = len - visible.length;
  if (diff <= 0) return s;
  const padding = " ".repeat(diff);
  return align === "right" ? padding + s : s + padding;
}

function truncate(s: string, max: number): string {
  if (s.length <= max) return s;
  return s.slice(0, max - 1) + "\u2026";
}

export type SortKey =
  | "quality"
  | "cost"
  | "speed"
  | "coding"
  | "math"
  | "value";

export function sortRows(rows: ModelRow[], key: SortKey): ModelRow[] {
  const sorted = [...rows];
  switch (key) {
    case "quality":
      return sorted.sort((a, b) => (b.quality ?? -1) - (a.quality ?? -1));
    case "cost":
      return sorted.sort(
        (a, b) => (a.outputPrice ?? 999) - (b.outputPrice ?? 999),
      );
    case "speed":
      return sorted.sort((a, b) => (b.speed ?? -1) - (a.speed ?? -1));
    case "coding":
      return sorted.sort((a, b) => (b.coding ?? -1) - (a.coding ?? -1));
    case "math":
      return sorted.sort((a, b) => (b.math ?? -1) - (a.math ?? -1));
    case "value":
      // Quality per dollar (blended price)
      return sorted.sort((a, b) => {
        const aVal =
          a.quality && a.blendedPrice ? a.quality / a.blendedPrice : -1;
        const bVal =
          b.quality && b.blendedPrice ? b.quality / b.blendedPrice : -1;
        return bVal - aVal;
      });
    default:
      return sorted;
  }
}

export function printModelsTable(
  rows: ModelRow[],
  opts: { limit?: number } = {},
): void {
  const display = opts.limit ? rows.slice(0, opts.limit) : rows;

  // Header
  const header = [
    pad("#", 4),
    pad("Model", 36),
    pad("Creator", 12),
    pad("Quality", 8, "right"),
    pad("Coding", 8, "right"),
    pad("In $/M", 9, "right"),
    pad("Out $/M", 9, "right"),
    pad("Speed", 9, "right"),
    pad("TTFT", 6, "right"),
  ].join("  ");

  console.log();
  console.log(c.bold + c.blue + header + c.reset);
  console.log(c.dim + "\u2500".repeat(header.length) + c.reset);

  display.forEach((r, i) => {
    const line = [
      pad(c.dim + String(i + 1) + c.reset, 4),
      pad(c.bold + truncate(r.name, 35) + c.reset, 36),
      pad(c.gray + truncate(r.creator, 11) + c.reset, 12),
      pad(fmtNum(r.quality), 8, "right"),
      pad(fmtNum(r.coding), 8, "right"),
      pad(fmtPrice(r.inputPrice), 9, "right"),
      pad(fmtPrice(r.outputPrice), 9, "right"),
      pad(
        r.speed ? fmtNum(r.speed, 0) + " t/s" : c.dim + "-" + c.reset,
        9,
        "right",
      ),
      pad(r.ttft ? fmtNum(r.ttft, 1) + "s" : c.dim + "-" + c.reset, 6, "right"),
    ].join("  ");
    console.log(line);
  });

  console.log();
  console.log(
    c.dim +
      `${display.length} of ${rows.length} models shown` +
      (opts.limit && rows.length > opts.limit
        ? ` (use --all to show all)`
        : "") +
      c.reset,
  );
  console.log(c.dim + "Data from artificialanalysis.ai" + c.reset);
  console.log();
}

export function printCompareTable(rows: ModelRow[]): void {
  if (rows.length === 0) {
    console.log("No models found matching those names.");
    return;
  }

  const metrics: Array<{ label: string; get: (r: ModelRow) => string }> = [
    { label: "Intelligence", get: (r) => fmtNum(r.quality) },
    { label: "Coding", get: (r) => fmtNum(r.coding) },
    { label: "Math", get: (r) => fmtNum(r.math) },
    { label: "Input $/M", get: (r) => fmtPrice(r.inputPrice) },
    { label: "Output $/M", get: (r) => fmtPrice(r.outputPrice) },
    { label: "Blended $/M", get: (r) => fmtPrice(r.blendedPrice) },
    { label: "Speed (t/s)", get: (r) => fmtNum(r.speed, 0) },
    { label: "TTFT (s)", get: (r) => fmtNum(r.ttft, 2) },
    { label: "Creator", get: (r) => r.creator },
  ];

  const colW = 18;
  const labelW = 14;

  // Header
  console.log();
  const hdr =
    pad("", labelW) +
    "  " +
    rows
      .map((r) => pad(c.bold + truncate(r.name, colW - 1) + c.reset, colW))
      .join("  ");
  console.log(hdr);
  console.log(
    c.dim + "\u2500".repeat(labelW + 2 + rows.length * (colW + 2)) + c.reset,
  );

  // Rows
  for (const m of metrics) {
    const line =
      pad(c.cyan + m.label + c.reset, labelW) +
      "  " +
      rows.map((r) => pad(m.get(r), colW)).join("  ");
    console.log(line);
  }

  console.log();
  console.log(c.dim + "Data from artificialanalysis.ai" + c.reset);
  console.log();
}

export function printModelDetail(row: ModelRow): void {
  console.log();
  console.log(
    c.bold +
      c.blue +
      row.name +
      c.reset +
      c.dim +
      ` by ${row.creator}` +
      c.reset,
  );
  console.log(c.dim + "\u2500".repeat(50) + c.reset);

  const lines: Array<[string, string]> = [
    ["Intelligence", fmtNum(row.quality)],
    ["Coding", fmtNum(row.coding)],
    ["Math", fmtNum(row.math)],
    ["Input Price", fmtPrice(row.inputPrice) + "/M tokens"],
    ["Output Price", fmtPrice(row.outputPrice) + "/M tokens"],
    ["Blended Price", fmtPrice(row.blendedPrice) + "/M tokens"],
    ["Output Speed", row.speed ? fmtNum(row.speed, 0) + " tokens/sec" : "-"],
    ["Time to First Token", row.ttft ? fmtNum(row.ttft, 2) + "s" : "-"],
    ["Slug", c.dim + row.slug + c.reset],
  ];

  for (const [label, value] of lines) {
    console.log(`  ${c.cyan}${pad(label, 20)}${c.reset}  ${value}`);
  }
  console.log();
}
