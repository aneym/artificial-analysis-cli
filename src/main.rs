mod api;
mod cache;
mod config;
mod format;
mod types;

use clap::{Parser, Subcommand};

use api::fetch_models;
use cache::{clear_cache, get_cache_age};
use config::{get_api_key, get_api_key_source, load_config, save_config};
use format::{print_compare_table, print_model_detail, print_models_table, sort_rows, SortKey};
use types::ModelRow;

#[derive(Parser)]
#[command(
    name = "aa",
    about = "Query AI model benchmarks, pricing, and performance from Artificial Analysis",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List AI models with intelligence scores, pricing, and speed
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
    },

    /// Compare models side-by-side (use slug or partial name)
    Compare {
        /// Model names or slugs to compare
        models: Vec<String>,

        /// Force refresh from API
        #[arg(long)]
        refresh: bool,
    },

    /// Show detailed info for a single model
    Show {
        /// Model name or slug
        model: String,

        /// Force refresh from API
        #[arg(long)]
        refresh: bool,
    },

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
        /// Clear the cache
        #[arg(long)]
        clear: bool,
    },
}

fn find_model<'a>(rows: &'a [ModelRow], query: &str) -> Option<&'a ModelRow> {
    let q = query.to_lowercase();
    // Exact slug match first, then exact name, then substring
    rows.iter()
        .find(|r| r.slug.to_lowercase() == q)
        .or_else(|| rows.iter().find(|r| r.name.to_lowercase() == q))
        .or_else(|| rows.iter().find(|r| r.slug.to_lowercase().contains(&q)))
        .or_else(|| rows.iter().find(|r| r.name.to_lowercase().contains(&q)))
}

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
        } => {
            let sort_key = SortKey::from_str(&sort)?;
            let models = fetch_models(refresh)?;
            let mut rows: Vec<ModelRow> = models.iter().map(ModelRow::from_api).collect();

            // Filters
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

            // Sort
            sort_rows(&mut rows, sort_key);

            // Display
            let display_limit = if all { None } else { Some(limit.unwrap_or(30)) };
            print_models_table(&rows, display_limit);

            if let Some(age) = get_cache_age() {
                println!("  \x1b[90mCache: {age}\x1b[0m\n");
            }
        }

        Commands::Compare { models, refresh } => {
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
        }

        Commands::Show { model, refresh } => {
            let api_models = fetch_models(refresh)?;
            let all_rows: Vec<ModelRow> = api_models.iter().map(ModelRow::from_api).collect();

            match find_model(&all_rows, &model) {
                Some(found) => print_model_detail(found),
                None => return Err(format!("No model found matching \"{model}\"")),
            }
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
                // Verify the file exists and contains the key before saving
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
                cfg.api_key = None; // Clear direct key when using env-file
                save_config(&cfg);
                // Verify it resolves
                match get_api_key() {
                    Some(_) => println!(
                        "\x1b[32mEnv file saved. AA_API_KEY found in {path}\x1b[0m"
                    ),
                    None => println!(
                        "\x1b[33mWarning: env file saved but AA_API_KEY not found in {path}\x1b[0m"
                    ),
                }
            } else if let Some(key) = key {
                let mut cfg = load_config();
                cfg.api_key = Some(key);
                cfg.env_file = None; // Clear env-file when setting direct key
                save_config(&cfg);
                println!("\x1b[32mAPI key saved.\x1b[0m");
            } else {
                // Show current status
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
                        println!(
                            "Get a free key at https://artificialanalysis.ai/account/api"
                        );
                    }
                }
            }
        }

        Commands::Cache { clear } => {
            if clear {
                let cleared = clear_cache();
                println!(
                    "{}",
                    if cleared {
                        "Cache cleared."
                    } else {
                        "No cache to clear."
                    }
                );
            } else {
                match get_cache_age() {
                    Some(age) => println!("Cache age: {age}"),
                    None => println!("No cache (will fetch on next request)"),
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
