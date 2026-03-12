use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "spark-insight",
    about = "Spark Eventlog intelligent analysis tool",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Event log file (shorthand: opens TUI directly)
    #[arg(global = false)]
    pub log_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Open TUI dashboard (default)
    Tui {
        /// Event log file path
        log_file: PathBuf,
    },

    /// Run built-in diagnostics (non-interactive)
    Analyze {
        /// Event log file path
        log_file: PathBuf,

        /// Comma-separated focus areas: skew,spill,shuffle,gc,summary
        #[arg(long, value_delimiter = ',')]
        focus: Vec<String>,

        /// Show top N results
        #[arg(long, default_value = "20")]
        top: usize,

        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        output: String,
    },

    /// Execute a single SQL query
    Sql {
        /// Event log file path
        log_file: PathBuf,

        /// SQL query to execute
        query: String,

        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        output: String,
    },

    /// Cache management
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum CacheAction {
    /// List all cached event logs
    List,
    /// Clear cache for a specific event log
    Clear {
        /// Event log file path or cache key
        log_file: PathBuf,
    },
    /// Clear all caches
    ClearAll,
}
