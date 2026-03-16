mod api;
mod cache;
mod config;
mod format;
mod types;

use clap::{Parser, Subcommand};

use api::fetch_models;
use cache::{clear_cache, get_cache_age};
use config::{get_api_key, load_config, save_config};
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

        Commands::Auth { key } => {
            if let Some(key) = key {
                let mut cfg = load_config();
                cfg.api_key = Some(key);
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
                        let source = if std::env::var("AA_API_KEY").is_ok() {
                            "AA_API_KEY env var"
                        } else {
                            "config file"
                        };
                        println!("Source: {source}");
                    }
                    None => {
                        println!("No API key configured.");
                        println!("Run: aa auth <your-key>");
                        println!("Or set: export AA_API_KEY=<your-key>");
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
