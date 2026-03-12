mod cli;
mod error;
mod parser;
mod query;
mod reporter;
mod store;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{CacheAction, Cli, Commands};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (only visible with RUST_LOG env var)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        // Default: open TUI if log file provided as positional arg
        None => {
            if let Some(log_file) = cli.log_file {
                tui::run_tui(&log_file).await?;
            } else {
                eprintln!("Usage: spark-insight <eventlog-file>");
                eprintln!("       spark-insight --help");
                std::process::exit(1);
            }
        }

        Some(Commands::Tui { log_file }) => {
            tui::run_tui(&log_file).await?;
        }

        Some(Commands::Analyze {
            log_file,
            focus,
            top,
            output,
        }) => {
            let store = store::EventStore::load(&log_file)
                .with_context(|| format!("Failed to load {}", log_file.display()))?;

            if output == "json" {
                let report = reporter::json::run_json_report(&store).await?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                reporter::terminal::run_analyze(&store, &focus, top).await?;
            }
        }

        Some(Commands::Sql {
            log_file,
            query,
            output,
        }) => {
            let store = store::EventStore::load(&log_file)
                .with_context(|| format!("Failed to load {}", log_file.display()))?;
            let engine = query::QueryEngine::new(&store).await?;
            let batch = engine.execute_sql_single(&query).await?;

            if output == "json" {
                let rows = query::format::batch_to_json(&batch);
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                println!("{}", query::format::batch_to_table_string(&batch));
            }
        }

        Some(Commands::Cache { action }) => match action {
            CacheAction::List => {
                let caches = store::cache::list_caches()?;
                if caches.is_empty() {
                    println!("No cached event logs found.");
                } else {
                    println!("Cached event logs:");
                    for c in caches {
                        println!("  {}", c);
                    }
                }
            }
            CacheAction::Clear { log_file } => {
                let key = store::cache::cache_key(&log_file)?;
                store::cache::clear_cache(&key)?;
                println!("Cache cleared for: {}", log_file.display());
            }
            CacheAction::ClearAll => {
                let caches = store::cache::list_caches()?;
                for key in &caches {
                    store::cache::clear_cache(key)?;
                }
                println!("Cleared {} cache(s).", caches.len());
            }
        },
    }

    Ok(())
}
