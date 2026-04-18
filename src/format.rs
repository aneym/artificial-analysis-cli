use crate::types::{AAMediaModel, MediaKind, ModelRow};

// ANSI color codes (no dependencies)
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const GRAY: &str = "\x1b[90m";

// ============ Sorting (LLMs) ============

#[derive(Debug, Clone, Copy)]
pub enum SortKey {
    Quality,
    Cost,
    Speed,
    Coding,
    Math,
    Value,
}

impl SortKey {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "quality" => Ok(Self::Quality),
            "cost" => Ok(Self::Cost),
            "speed" => Ok(Self::Speed),
            "coding" => Ok(Self::Coding),
            "math" => Ok(Self::Math),
            "value" => Ok(Self::Value),
            _ => Err(format!(
                "Invalid sort key '{s}'. Valid: quality, cost, speed, coding, math, value"
            )),
        }
    }
}

pub fn sort_rows(rows: &mut [ModelRow], key: SortKey) {
    match key {
        SortKey::Quality => rows.sort_by(|a, b| {
            b.quality
                .unwrap_or(-1.0)
                .partial_cmp(&a.quality.unwrap_or(-1.0))
                .unwrap()
        }),
        SortKey::Cost => rows.sort_by(|a, b| {
            a.output_price
                .unwrap_or(999.0)
                .partial_cmp(&b.output_price.unwrap_or(999.0))
                .unwrap()
        }),
        SortKey::Speed => rows.sort_by(|a, b| {
            b.speed
                .unwrap_or(-1.0)
                .partial_cmp(&a.speed.unwrap_or(-1.0))
                .unwrap()
        }),
        SortKey::Coding => rows.sort_by(|a, b| {
            b.coding
                .unwrap_or(-1.0)
                .partial_cmp(&a.coding.unwrap_or(-1.0))
                .unwrap()
        }),
        SortKey::Math => rows.sort_by(|a, b| {
            b.math
                .unwrap_or(-1.0)
                .partial_cmp(&a.math.unwrap_or(-1.0))
                .unwrap()
        }),
        SortKey::Value => rows.sort_by(|a, b| {
            let a_val = match (a.quality, a.blended_price) {
                (Some(q), Some(p)) if p > 0.0 => q / p,
                _ => -1.0,
            };
            let b_val = match (b.quality, b.blended_price) {
                (Some(q), Some(p)) if p > 0.0 => q / p,
                _ => -1.0,
            };
            b_val.partial_cmp(&a_val).unwrap()
        }),
    }
}

// ============ Sorting (Media) ============

#[derive(Debug, Clone, Copy)]
pub enum MediaSortKey {
    Rank,
    Elo,
    Appearances,
    Recent,
}

impl MediaSortKey {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "rank" => Ok(Self::Rank),
            "elo" => Ok(Self::Elo),
            "appearances" | "popularity" | "votes" => Ok(Self::Appearances),
            "recent" | "release" | "newest" => Ok(Self::Recent),
            _ => Err(format!(
                "Invalid media sort key '{s}'. Valid: rank, elo, appearances, recent"
            )),
        }
    }
}

pub fn sort_media(rows: &mut [AAMediaModel], key: MediaSortKey) {
    match key {
        MediaSortKey::Rank => rows.sort_by(|a, b| {
            a.rank
                .unwrap_or(u64::MAX)
                .cmp(&b.rank.unwrap_or(u64::MAX))
        }),
        MediaSortKey::Elo => rows.sort_by(|a, b| {
            b.elo
                .unwrap_or(-1.0)
                .partial_cmp(&a.elo.unwrap_or(-1.0))
                .unwrap()
        }),
        MediaSortKey::Appearances => rows.sort_by(|a, b| {
            b.appearances.unwrap_or(0).cmp(&a.appearances.unwrap_or(0))
        }),
        MediaSortKey::Recent => rows.sort_by(|a, b| {
            b.release_date
                .clone()
                .unwrap_or_default()
                .cmp(&a.release_date.clone().unwrap_or_default())
        }),
    }
}

// ============ Shared helpers ============

fn fmt_num(n: Option<f64>, decimals: usize) -> String {
    match n {
        Some(v) => format!("{v:.decimals$}"),
        None => format!("{DIM}-{RESET}"),
    }
}

fn fmt_price(n: Option<f64>) -> String {
    match n {
        Some(v) if v < 0.1 => format!("{GREEN}${v:.3}{RESET}"),
        Some(v) if v < 1.0 => format!("{GREEN}${v:.2}{RESET}"),
        Some(v) if v < 5.0 => format!("{YELLOW}${v:.2}{RESET}"),
        Some(v) => format!("${v:.2}"),
        None => format!("{DIM}-{RESET}"),
    }
}

/// Strip ANSI escape sequences for visible length calculation.
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

fn pad(s: &str, width: usize, right: bool) -> String {
    let vis = visible_len(s);
    if vis >= width {
        return s.to_string();
    }
    let padding = " ".repeat(width - vis);
    if right {
        format!("{padding}{s}")
    } else {
        format!("{s}{padding}")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let taken: String = s.chars().take(max - 1).collect();
        format!("{taken}\u{2026}")
    }
}

// ============ LLM formatters ============

pub fn print_models_table(rows: &[ModelRow], limit: Option<usize>) {
    let display: &[ModelRow] = match limit {
        Some(n) if n < rows.len() => &rows[..n],
        _ => rows,
    };

    let header = [
        pad("#", 4, false),
        pad("Model", 36, false),
        pad("Creator", 12, false),
        pad("Quality", 8, true),
        pad("Coding", 8, true),
        pad("In $/M", 9, true),
        pad("Out $/M", 9, true),
        pad("Speed", 9, true),
        pad("TTFT", 6, true),
    ]
    .join("  ");

    println!();
    println!("{BOLD}{BLUE}{header}{RESET}");
    println!("{DIM}{}{RESET}", "\u{2500}".repeat(header.len()));

    for (i, r) in display.iter().enumerate() {
        let rank = format!("{DIM}{}{RESET}", i + 1);
        let name_str = format!("{BOLD}{}{RESET}", truncate(&r.name, 35));
        let creator_str = format!("{GRAY}{}{RESET}", truncate(&r.creator, 11));
        let speed_str = match r.speed {
            Some(v) => format!("{v:.0} t/s"),
            None => format!("{DIM}-{RESET}"),
        };
        let ttft_str = match r.ttft {
            Some(v) => format!("{v:.1}s"),
            None => format!("{DIM}-{RESET}"),
        };

        let line = [
            pad(&rank, 4, false),
            pad(&name_str, 36, false),
            pad(&creator_str, 12, false),
            pad(&fmt_num(r.quality, 1), 8, true),
            pad(&fmt_num(r.coding, 1), 8, true),
            pad(&fmt_price(r.input_price), 9, true),
            pad(&fmt_price(r.output_price), 9, true),
            pad(&speed_str, 9, true),
            pad(&ttft_str, 6, true),
        ]
        .join("  ");
        println!("{line}");
    }

    println!();
    let shown_msg = if limit.is_some_and(|n| rows.len() > n) {
        format!(
            "{} of {} models shown (use --all to show all)",
            display.len(),
            rows.len()
        )
    } else {
        format!("{} of {} models shown", display.len(), rows.len())
    };
    println!("{DIM}{shown_msg}{RESET}");
    println!("{DIM}Data from artificialanalysis.ai{RESET}");
    println!();
}

pub fn print_compare_table(rows: &[ModelRow]) {
    if rows.is_empty() {
        println!("No models found matching those names.");
        return;
    }

    struct Metric {
        label: &'static str,
        get: fn(&ModelRow) -> String,
    }

    let metrics: Vec<Metric> = vec![
        Metric {
            label: "Intelligence",
            get: |r| fmt_num(r.quality, 1),
        },
        Metric {
            label: "Coding",
            get: |r| fmt_num(r.coding, 1),
        },
        Metric {
            label: "Math",
            get: |r| fmt_num(r.math, 1),
        },
        Metric {
            label: "Input $/M",
            get: |r| fmt_price(r.input_price),
        },
        Metric {
            label: "Output $/M",
            get: |r| fmt_price(r.output_price),
        },
        Metric {
            label: "Blended $/M",
            get: |r| fmt_price(r.blended_price),
        },
        Metric {
            label: "Speed (t/s)",
            get: |r| fmt_num(r.speed, 0),
        },
        Metric {
            label: "TTFT (s)",
            get: |r| fmt_num(r.ttft, 2),
        },
        Metric {
            label: "Creator",
            get: |r| r.creator.clone(),
        },
    ];

    let col_w = 18;
    let label_w = 14;

    println!();
    let names: Vec<String> = rows
        .iter()
        .map(|r| pad(&format!("{BOLD}{}{RESET}", truncate(&r.name, col_w - 1)), col_w, false))
        .collect();
    let hdr = format!("{}  {}", pad("", label_w, false), names.join("  "));
    println!("{hdr}");
    println!(
        "{DIM}{}{RESET}",
        "\u{2500}".repeat(label_w + 2 + rows.len() * (col_w + 2))
    );

    for m in &metrics {
        let values: Vec<String> = rows.iter().map(|r| pad(&(m.get)(r), col_w, false)).collect();
        let line = format!(
            "{}  {}",
            pad(&format!("{CYAN}{}{RESET}", m.label), label_w, false),
            values.join("  ")
        );
        println!("{line}");
    }

    println!();
    println!("{DIM}Data from artificialanalysis.ai{RESET}");
    println!();
}

pub fn print_model_detail(row: &ModelRow) {
    println!();
    println!("{BOLD}{BLUE}{}{RESET}{DIM} by {}{RESET}", row.name, row.creator);
    println!("{DIM}{}{RESET}", "\u{2500}".repeat(50));

    let speed_str = match row.speed {
        Some(v) => format!("{v:.0} tokens/sec"),
        None => "-".to_string(),
    };
    let ttft_str = match row.ttft {
        Some(v) => format!("{v:.2}s"),
        None => "-".to_string(),
    };

    let lines: Vec<(&str, String)> = vec![
        ("Intelligence", fmt_num(row.quality, 1)),
        ("Coding", fmt_num(row.coding, 1)),
        ("Math", fmt_num(row.math, 1)),
        (
            "Input Price",
            format!("{}/M tokens", fmt_price(row.input_price)),
        ),
        (
            "Output Price",
            format!("{}/M tokens", fmt_price(row.output_price)),
        ),
        (
            "Blended Price",
            format!("{}/M tokens", fmt_price(row.blended_price)),
        ),
        ("Output Speed", speed_str),
        ("Time to First Token", ttft_str),
        ("Slug", format!("{DIM}{}{RESET}", row.slug)),
    ];

    for (label, value) in &lines {
        println!("  {CYAN}{}{RESET}  {value}", pad(label, 20, false));
    }
    println!();
}

// ============ Media formatters ============

fn creator_name(m: &AAMediaModel) -> String {
    m.model_creator
        .as_ref()
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "?".to_string())
}

pub fn print_media_table(rows: &[AAMediaModel], limit: Option<usize>, kind: MediaKind) {
    let display: &[AAMediaModel] = match limit {
        Some(n) if n < rows.len() => &rows[..n],
        _ => rows,
    };

    let header = [
        pad("Rank", 5, false),
        pad("Model", 36, false),
        pad("Creator", 16, false),
        pad("ELO", 6, true),
        pad("CI95", 9, true),
        pad("Votes", 8, true),
        pad("Released", 9, true),
    ]
    .join("  ");

    println!();
    println!("{BOLD}{BLUE}{} models{RESET}", kind.label());
    println!("{BOLD}{BLUE}{header}{RESET}");
    println!("{DIM}{}{RESET}", "\u{2500}".repeat(header.len()));

    for (i, m) in display.iter().enumerate() {
        let rank_display = m
            .rank
            .map(|r| r.to_string())
            .unwrap_or_else(|| format!("{}", i + 1));
        let rank_str = format!("{DIM}{rank_display}{RESET}");
        let name_str = format!("{BOLD}{}{RESET}", truncate(&m.name, 35));
        let creator_str = format!("{GRAY}{}{RESET}", truncate(&creator_name(m), 15));
        let elo_str = match m.elo {
            Some(v) => format!("{v:.0}"),
            None => format!("{DIM}-{RESET}"),
        };
        let ci_str = m.ci95.clone().unwrap_or_else(|| format!("{DIM}-{RESET}"));
        let votes_str = match m.appearances {
            Some(v) => format!("{v}"),
            None => format!("{DIM}-{RESET}"),
        };
        let released_str = m
            .release_date
            .clone()
            .unwrap_or_else(|| format!("{DIM}-{RESET}"));

        let line = [
            pad(&rank_str, 5, false),
            pad(&name_str, 36, false),
            pad(&creator_str, 16, false),
            pad(&elo_str, 6, true),
            pad(&ci_str, 9, true),
            pad(&votes_str, 8, true),
            pad(&released_str, 9, true),
        ]
        .join("  ");
        println!("{line}");
    }

    println!();
    let shown_msg = if limit.is_some_and(|n| rows.len() > n) {
        format!(
            "{} of {} models shown (use --all to show all)",
            display.len(),
            rows.len()
        )
    } else {
        format!("{} of {} models shown", display.len(), rows.len())
    };
    println!("{DIM}{shown_msg}{RESET}");
    println!("{DIM}Data from artificialanalysis.ai{RESET}");
    println!();
}

pub fn print_media_detail(model: &AAMediaModel, kind: MediaKind) {
    let creator = creator_name(model);
    println!();
    println!(
        "{BOLD}{BLUE}{}{RESET}{DIM} by {}  ({}){RESET}",
        model.name,
        creator,
        kind.label()
    );
    println!("{DIM}{}{RESET}", "\u{2500}".repeat(60));

    let rank_str = model
        .rank
        .map(|r| r.to_string())
        .unwrap_or_else(|| "-".to_string());
    let elo_str = model
        .elo
        .map(|v| format!("{v:.0}"))
        .unwrap_or_else(|| "-".to_string());
    let ci_str = model.ci95.clone().unwrap_or_else(|| "-".to_string());
    let votes_str = model
        .appearances
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let release_str = model.release_date.clone().unwrap_or_else(|| "-".to_string());

    let lines: Vec<(&str, String)> = vec![
        ("Rank", rank_str),
        ("ELO", elo_str),
        ("95% CI", ci_str),
        ("Votes", votes_str),
        ("Released", release_str),
        ("Slug", format!("{DIM}{}{RESET}", model.slug)),
    ];

    for (label, value) in &lines {
        println!("  {CYAN}{}{RESET}  {value}", pad(label, 14, false));
    }

    if let Some(cats) = &model.categories
        && !cats.is_empty() {
            println!();
            println!("{BOLD}Category breakdown{RESET}");
            println!(
                "  {CYAN}{}{RESET}  {CYAN}{}{RESET}  {CYAN}{}{RESET}  {CYAN}{}{RESET}",
                pad("Category", 28, false),
                pad("ELO", 6, true),
                pad("CI95", 9, true),
                pad("Votes", 7, true),
            );
            println!("  {DIM}{}{RESET}", "\u{2500}".repeat(28 + 6 + 9 + 7 + 6));

            let mut sorted: Vec<_> = cats.iter().collect();
            sorted.sort_by(|a, b| {
                b.elo
                    .unwrap_or(-1.0)
                    .partial_cmp(&a.elo.unwrap_or(-1.0))
                    .unwrap()
            });

            for c in sorted {
                let cat_label = c
                    .style_category
                    .clone()
                    .or_else(|| c.subject_matter_category.clone())
                    .or_else(|| c.format_category.clone())
                    .unwrap_or_else(|| "-".to_string());
                let elo = c
                    .elo
                    .map(|v| format!("{v:.0}"))
                    .unwrap_or_else(|| "-".to_string());
                let ci = c.ci95.clone().unwrap_or_else(|| "-".to_string());
                let votes = c
                    .appearances
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string());
                println!(
                    "  {}  {}  {}  {}",
                    pad(&truncate(&cat_label, 28), 28, false),
                    pad(&elo, 6, true),
                    pad(&ci, 9, true),
                    pad(&votes, 7, true),
                );
            }
        }

    println!();
}

pub fn print_media_compare_table(rows: &[AAMediaModel], kind: MediaKind) {
    if rows.is_empty() {
        println!("No models found matching those names.");
        return;
    }

    struct Metric {
        label: &'static str,
        get: fn(&AAMediaModel) -> String,
    }

    let metrics: Vec<Metric> = vec![
        Metric {
            label: "Rank",
            get: |m| {
                m.rank
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "-".to_string())
            },
        },
        Metric {
            label: "ELO",
            get: |m| {
                m.elo
                    .map(|v| format!("{v:.0}"))
                    .unwrap_or_else(|| "-".to_string())
            },
        },
        Metric {
            label: "95% CI",
            get: |m| m.ci95.clone().unwrap_or_else(|| "-".to_string()),
        },
        Metric {
            label: "Votes",
            get: |m| {
                m.appearances
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string())
            },
        },
        Metric {
            label: "Released",
            get: |m| m.release_date.clone().unwrap_or_else(|| "-".to_string()),
        },
        Metric {
            label: "Creator",
            get: creator_name,
        },
    ];

    let col_w = 18;
    let label_w = 14;

    println!();
    println!("{BOLD}{BLUE}{} compare{RESET}", kind.label());
    let names: Vec<String> = rows
        .iter()
        .map(|r| {
            pad(
                &format!("{BOLD}{}{RESET}", truncate(&r.name, col_w - 1)),
                col_w,
                false,
            )
        })
        .collect();
    let hdr = format!("{}  {}", pad("", label_w, false), names.join("  "));
    println!("{hdr}");
    println!(
        "{DIM}{}{RESET}",
        "\u{2500}".repeat(label_w + 2 + rows.len() * (col_w + 2))
    );

    for m in &metrics {
        let values: Vec<String> = rows.iter().map(|r| pad(&(m.get)(r), col_w, false)).collect();
        let line = format!(
            "{}  {}",
            pad(&format!("{CYAN}{}{RESET}", m.label), label_w, false),
            values.join("  ")
        );
        println!("{line}");
    }

    println!();
    println!("{DIM}Data from artificialanalysis.ai{RESET}");
    println!();
}
