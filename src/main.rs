mod api;
mod cache;
mod config;
mod format;
mod types;

use clap::{Parser, Subcommand};

use api::{fetch_media, fetch_models, fetch_raw};
use cache::{
    clear_all_caches, clear_cache, get_cache_age, get_media_cache_age, list_cache_ages,
};
use config::{get_api_key, get_api_key_source, load_config, save_config};
use format::{
    print_compare_table, print_media_compare_table, print_media_detail, print_media_table,
    print_model_detail, print_models_table, sort_media, sort_rows, MediaSortKey, SortKey,
};
use types::{AAMediaModel, MediaKind, ModelRow};

#[derive(Parser)]
#[command(
    name = "aa",
    about = "Query AI model benchmarks, pricing, and performance from Artificial Analysis",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Shared filter/sort options for every `aa <media-kind>` subcommand.
#[derive(clap::Args, Clone)]
struct MediaArgs {
    /// Sort by: rank, elo, appearances, recent
    #[arg(short, long, default_value = "rank")]
    sort: String,

    /// Filter models by name or slug (case-insensitive substring)
    #[arg(short, long)]
    filter: Option<String>,

    /// Filter by creator/provider name
    #[arg(long)]
    creator: Option<String>,

    /// Number of models to show (default: 30)
    #[arg(short = 'n', long)]
    limit: Option<usize>,

    /// Show all models
    #[arg(short, long)]
    all: bool,

    /// Force refresh from API (ignores cache)
    #[arg(long)]
    refresh: bool,

    /// Include per-category ELOs (only text-to-image, text-to-video, image-to-video)
    #[arg(long)]
    categories: bool,

    /// Emit raw JSON instead of a formatted table
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List LLMs with intelligence scores, pricing, and speed
    Models {
        /// Sort by: quality, cost, speed, coding, math, value
        #[arg(short, long, default_value = "quality")]
        sort: String,

        /// Filter models by name (case-insensitive substring)
        #[arg(short, long)]
        filter: Option<String>,

        /// Only show models with output price < $1/M tokens
        #[arg(short, long)]
        cheap: bool,

        /// Number of models to show (default: 30)
        #[arg(short = 'n', long)]
        limit: Option<usize>,

        /// Show all models
        #[arg(short, long)]
        all: bool,

        /// Force refresh from API (ignores cache)
        #[arg(long)]
        refresh: bool,

        /// Minimum intelligence index score
        #[arg(long)]
        min_quality: Option<f64>,

        /// Maximum output price per million tokens
        #[arg(long)]
        max_cost: Option<f64>,

        /// Filter by creator/provider name
        #[arg(long)]
        creator: Option<String>,

        /// Emit raw JSON instead of a formatted table
        #[arg(long)]
        json: bool,
    },

    /// Compare models side-by-side (use slug or partial name)
    Compare {
        /// Model names or slugs to compare
        models: Vec<String>,

        /// Kind of model: llm (default), tts, image, image-edit, video, img2vid
        #[arg(long, default_value = "llm")]
        kind: String,

        /// Include per-category ELOs (only applies to media kinds)
        #[arg(long)]
        categories: bool,

        /// Force refresh from API
        #[arg(long)]
        refresh: bool,
    },

    /// Show detailed info for a single model
    Show {
        /// Model name or slug
        model: String,

        /// Kind of model: llm (default), tts, image, image-edit, video, img2vid
        #[arg(long, default_value = "llm")]
        kind: String,

        /// Include per-category ELOs (only applies to media kinds)
        #[arg(long)]
        categories: bool,

        /// Force refresh from API
        #[arg(long)]
        refresh: bool,

        /// Emit raw JSON instead of a formatted table
        #[arg(long)]
        json: bool,
    },

    /// Text-to-speech model rankings (aliases: voice, speech)
    #[command(alias = "voice", alias = "speech")]
    Tts(MediaArgs),

    /// Text-to-image model rankings (alias: img)
    #[command(alias = "img")]
    Image(MediaArgs),

    /// Image-editing model rankings
    #[command(alias = "imgedit", alias = "edit")]
    ImageEdit(MediaArgs),

    /// Text-to-video model rankings
    Video(MediaArgs),

    /// Image-to-video model rankings (alias: i2v)
    #[command(alias = "i2v")]
    Img2vid(MediaArgs),

    /// Generic media lookup: aa media <kind> [options]
    Media {
        /// Media kind: tts, image, image-edit, video, img2vid
        kind: String,

        #[command(flatten)]
        args: MediaArgs,
    },

    /// Call any AA v2 endpoint directly and dump the raw JSON
    Raw {
        /// Endpoint path (e.g. "data/media/text-to-speech")
        path: String,

        /// Query params as key=value (repeatable): --query include_categories=true
        #[arg(short = 'q', long = "query", value_name = "KEY=VAL")]
        query: Vec<String>,

        /// Pretty-print instead of one-line JSON
        #[arg(long)]
        pretty: bool,
    },

    /// List every AA v2 endpoint this CLI knows about
    Endpoints,

    /// Set or show API key
    Auth {
        /// API key to save (omit to show current key)
        key: Option<String>,

        /// Path to a .env file containing AA_API_KEY
        #[arg(long)]
        env_file: Option<String>,

        /// Remove stored key and env-file reference
        #[arg(long)]
        clear: bool,
    },

    /// Show cache status or clear it
    Cache {
        /// Clear the cache (LLM only by default; combine with --all for every cache)
        #[arg(long)]
        clear: bool,

        /// When used with --clear, remove every per-endpoint cache
        #[arg(long)]
        all: bool,
    },
}

// ============ Matching helpers ============

fn find_model<'a>(rows: &'a [ModelRow], query: &str) -> Option<&'a ModelRow> {
    let q = query.to_lowercase();
    rows.iter()
        .find(|r| r.slug.to_lowercase() == q)
        .or_else(|| rows.iter().find(|r| r.name.to_lowercase() == q))
        .or_else(|| rows.iter().find(|r| r.slug.to_lowercase().contains(&q)))
        .or_else(|| rows.iter().find(|r| r.name.to_lowercase().contains(&q)))
}

fn find_media<'a>(rows: &'a [AAMediaModel], query: &str) -> Option<&'a AAMediaModel> {
    let q = query.to_lowercase();
    rows.iter()
        .find(|r| r.slug.to_lowercase() == q)
        .or_else(|| rows.iter().find(|r| r.name.to_lowercase() == q))
        .or_else(|| rows.iter().find(|r| r.slug.to_lowercase().contains(&q)))
        .or_else(|| rows.iter().find(|r| r.name.to_lowercase().contains(&q)))
}

fn apply_media_filters(rows: &mut Vec<AAMediaModel>, args: &MediaArgs) {
    if let Some(ref term) = args.filter {
        let t = term.to_lowercase();
        rows.retain(|r| r.name.to_lowercase().contains(&t) || r.slug.to_lowercase().contains(&t));
    }
    if let Some(ref term) = args.creator {
        let t = term.to_lowercase();
        rows.retain(|r| {
            r.model_creator
                .as_ref()
                .map(|c| c.name.to_lowercase().contains(&t))
                .unwrap_or(false)
        });
    }
}

fn run_media(kind: MediaKind, args: MediaArgs) -> Result<(), String> {
    let sort_key = MediaSortKey::from_str(&args.sort)?;
    let include_cats = args.categories && kind.supports_categories();
    let api_models = fetch_media(kind, include_cats, args.refresh)?;

    let mut rows: Vec<AAMediaModel> = api_models;
    apply_media_filters(&mut rows, &args);
    sort_media(&mut rows, sort_key);

    let limit = if args.all { None } else { Some(args.limit.unwrap_or(30)) };

    if args.json {
        let out: Vec<&AAMediaModel> = match limit {
            Some(n) if n < rows.len() => rows.iter().take(n).collect(),
            _ => rows.iter().collect(),
        };
        let json = serde_json::to_string_pretty(&out)
            .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
        println!("{json}");
        return Ok(());
    }

    print_media_table(&rows, limit, kind);

    if let Some(age) = get_media_cache_age(kind, include_cats) {
        println!("  \x1b[90mCache: {age}\x1b[0m\n");
    }
    Ok(())
}

fn parse_raw_query(pairs: &[String]) -> Result<Vec<(String, String)>, String> {
    pairs
        .iter()
        .map(|p| {
            p.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .ok_or_else(|| format!("Invalid --query entry '{p}' (expected KEY=VAL)"))
        })
        .collect()
}

// ============ Entry points ============

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Models {
            sort,
            filter,
            cheap,
            limit,
            all,
            refresh,
            min_quality,
            max_cost,
            creator,
            json,
        } => {
            let sort_key = SortKey::from_str(&sort)?;
            let models = fetch_models(refresh)?;
            let mut rows: Vec<ModelRow> = models.iter().map(ModelRow::from_api).collect();

            if let Some(ref term) = filter {
                let t = term.to_lowercase();
                rows.retain(|r| {
                    r.name.to_lowercase().contains(&t) || r.slug.to_lowercase().contains(&t)
                });
            }
            if let Some(ref term) = creator {
                let t = term.to_lowercase();
                rows.retain(|r| r.creator.to_lowercase().contains(&t));
            }
            if cheap {
                rows.retain(|r| r.output_price.is_some_and(|p| p < 1.0));
            }
            if let Some(min) = min_quality {
                rows.retain(|r| r.quality.is_some_and(|q| q >= min));
            }
            if let Some(max) = max_cost {
                rows.retain(|r| r.output_price.is_some_and(|p| p <= max));
            }

            sort_rows(&mut rows, sort_key);

            let display_limit = if all { None } else { Some(limit.unwrap_or(30)) };

            if json {
                // Emit raw AAModel records (preserves full evaluations + pricing).
                let slugs: std::collections::HashSet<String> =
                    rows.iter().map(|r| r.slug.clone()).collect();
                let filtered: Vec<_> = models.iter().filter(|m| slugs.contains(&m.slug)).collect();
                let limited: Vec<_> = match display_limit {
                    Some(n) if n < filtered.len() => filtered[..n].to_vec(),
                    _ => filtered,
                };
                let out = serde_json::to_string_pretty(&limited)
                    .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
                println!("{out}");
                return Ok(());
            }

            print_models_table(&rows, display_limit);

            if let Some(age) = get_cache_age() {
                println!("  \x1b[90mCache: {age}\x1b[0m\n");
            }
        }

        Commands::Compare {
            models,
            kind,
            categories,
            refresh,
        } => {
            if kind.eq_ignore_ascii_case("llm") || kind.eq_ignore_ascii_case("llms") {
                let api_models = fetch_models(refresh)?;
                let all_rows: Vec<ModelRow> = api_models.iter().map(ModelRow::from_api).collect();

                let mut matched = Vec::new();
                for query in &models {
                    if let Some(found) = find_model(&all_rows, query) {
                        matched.push(found.clone());
                    } else {
                        eprintln!("\x1b[33mWarning: No model found matching \"{query}\"\x1b[0m");
                    }
                }
                print_compare_table(&matched);
            } else {
                let media_kind = MediaKind::from_str(&kind)?;
                let include_cats = categories && media_kind.supports_categories();
                let api_models = fetch_media(media_kind, include_cats, refresh)?;
                let mut matched = Vec::new();
                for query in &models {
                    if let Some(found) = find_media(&api_models, query) {
                        matched.push(found.clone());
                    } else {
                        eprintln!("\x1b[33mWarning: No model found matching \"{query}\"\x1b[0m");
                    }
                }
                print_media_compare_table(&matched, media_kind);
            }
        }

        Commands::Show {
            model,
            kind,
            categories,
            refresh,
            json,
        } => {
            if kind.eq_ignore_ascii_case("llm") || kind.eq_ignore_ascii_case("llms") {
                let api_models = fetch_models(refresh)?;
                let all_rows: Vec<ModelRow> = api_models.iter().map(ModelRow::from_api).collect();
                match find_model(&all_rows, &model) {
                    Some(found) => {
                        if json {
                            let raw = api_models.iter().find(|m| m.slug == found.slug);
                            let out = serde_json::to_string_pretty(&raw)
                                .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
                            println!("{out}");
                        } else {
                            print_model_detail(found);
                        }
                    }
                    None => return Err(format!("No model found matching \"{model}\"")),
                }
            } else {
                let media_kind = MediaKind::from_str(&kind)?;
                let include_cats = categories && media_kind.supports_categories();
                let api_models = fetch_media(media_kind, include_cats, refresh)?;
                match find_media(&api_models, &model) {
                    Some(found) => {
                        if json {
                            let out = serde_json::to_string_pretty(&found)
                                .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
                            println!("{out}");
                        } else {
                            print_media_detail(found, media_kind);
                        }
                    }
                    None => return Err(format!("No model found matching \"{model}\"")),
                }
            }
        }

        Commands::Tts(args) => run_media(MediaKind::TextToSpeech, args)?,
        Commands::Image(args) => run_media(MediaKind::TextToImage, args)?,
        Commands::ImageEdit(args) => run_media(MediaKind::ImageEditing, args)?,
        Commands::Video(args) => run_media(MediaKind::TextToVideo, args)?,
        Commands::Img2vid(args) => run_media(MediaKind::ImageToVideo, args)?,
        Commands::Media { kind, args } => {
            let mk = MediaKind::from_str(&kind)?;
            run_media(mk, args)?;
        }

        Commands::Raw {
            path,
            query,
            pretty,
        } => {
            let parsed = parse_raw_query(&query)?;
            let tuples: Vec<(&str, &str)> =
                parsed.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let value = fetch_raw(&path, &tuples)?;
            let out = if pretty {
                serde_json::to_string_pretty(&value)
            } else {
                serde_json::to_string(&value)
            }
            .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
            println!("{out}");
        }

        Commands::Endpoints => {
            println!("\x1b[1mAA v2 endpoints covered by this CLI\x1b[0m\n");
            let rows: &[(&str, &str, &str)] = &[
                ("aa models",     "data/llms/models",           "LLM benchmarks, pricing, speed"),
                ("aa tts",        "data/media/text-to-speech",  "Text-to-speech rankings"),
                ("aa image",      "data/media/text-to-image",   "Text-to-image rankings"),
                ("aa image-edit", "data/media/image-editing",   "Image-editing rankings"),
                ("aa video",      "data/media/text-to-video",   "Text-to-video rankings"),
                ("aa img2vid",    "data/media/image-to-video",  "Image-to-video rankings"),
                ("aa raw <path>", "(any)",                      "Raw passthrough for any endpoint"),
            ];
            for (cmd, path, desc) in rows {
                println!(
                    "  \x1b[36m{:<16}\x1b[0m  \x1b[90m{:<32}\x1b[0m  {}",
                    cmd, path, desc
                );
            }
            println!(
                "\n\x1b[2mUse --json on any list command for programmatic output.\nUse `aa show <model> --kind <kind>` for a single model's detail.\nUse `aa compare a b c --kind <kind>` for side-by-side comparison.\x1b[0m"
            );
        }

        Commands::Auth {
            key,
            env_file,
            clear,
        } => {
            if clear {
                let mut cfg = load_config();
                cfg.api_key = None;
                cfg.env_file = None;
                save_config(&cfg);
                println!("\x1b[32mConfig cleared.\x1b[0m");
            } else if let Some(path) = env_file {
                let expanded = if path.starts_with('~') {
                    dirs::home_dir()
                        .map(|h| path.replacen('~', &h.to_string_lossy(), 1))
                        .unwrap_or_else(|| path.clone())
                } else {
                    path.clone()
                };
                if !std::path::Path::new(&expanded).exists() {
                    return Err(format!("File not found: {path}"));
                }
                let mut cfg = load_config();
                cfg.env_file = Some(path.clone());
                cfg.api_key = None;
                save_config(&cfg);
                match get_api_key() {
                    Some(_) => {
                        println!("\x1b[32mEnv file saved. AA_API_KEY found in {path}\x1b[0m")
                    }
                    None => println!(
                        "\x1b[33mWarning: env file saved but AA_API_KEY not found in {path}\x1b[0m"
                    ),
                }
            } else if let Some(key) = key {
                let mut cfg = load_config();
                cfg.api_key = Some(key);
                cfg.env_file = None;
                save_config(&cfg);
                println!("\x1b[32mAPI key saved.\x1b[0m");
            } else {
                match get_api_key() {
                    Some(current) => {
                        let masked = format!(
                            "{}...{}",
                            &current[..current.len().min(6)],
                            &current[current.len().saturating_sub(4)..]
                        );
                        println!("Current API key: {masked}");
                        println!("Source: {}", get_api_key_source());
                        let cfg = load_config();
                        if let Some(ref path) = cfg.env_file {
                            println!("Env file: {path}");
                        }
                    }
                    None => {
                        println!("No API key configured.\n");
                        println!("Options:");
                        println!("  aa auth <key>                     Save key directly");
                        println!(
                            "  aa auth --env-file ~/.openclaw/.env   Read from .env file"
                        );
                        println!("  export AA_API_KEY=<key>           Environment variable\n");
                        println!("Get a free key at https://artificialanalysis.ai/account/api");
                    }
                }
            }
        }

        Commands::Cache { clear, all } => {
            if clear {
                if all {
                    let n = clear_all_caches();
                    println!("Removed {n} cache file(s).");
                } else {
                    let cleared = clear_cache();
                    println!(
                        "{}",
                        if cleared {
                            "LLM cache cleared. (Use --all to clear every cache.)"
                        } else {
                            "No LLM cache to clear."
                        }
                    );
                }
            } else {
                let ages = list_cache_ages();
                if ages.is_empty() {
                    println!("No caches yet (will fetch on next request).");
                } else {
                    for (label, age) in ages {
                        match age {
                            Some(a) => println!("  {label:<28}  {a}"),
                            None => println!("  {label:<28}  (unreadable)"),
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\x1b[31mError: {e}\x1b[0m");
        std::process::exit(1);
    }
}
