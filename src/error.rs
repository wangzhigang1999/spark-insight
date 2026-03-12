use thiserror::Error;

#[derive(Error, Debug)]
pub enum InsightError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error at line {line}: {source}")]
    JsonParse {
        line: usize,
        source: serde_json::Error,
    },

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    #[error("DataFusion error: {0}")]
    DataFusion(#[from] datafusion::error::DataFusionError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Event log file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid event log format: {0}")]
    InvalidFormat(String),
}

pub type Result<T> = std::result::Result<T, InsightError>;
