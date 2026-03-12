pub mod builder;
pub mod cache;
pub mod schema;

use crate::error::Result;
use crate::parser::{parse_eventlog, ParsedEventLog};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::info;

use builder::{build_executors_batch, build_jobs_batch, build_stages_batch, build_tasks_batch};
use cache::{cache_dir, cache_key, ensure_cache_dir, is_cache_fresh};

pub struct EventStore {
    pub cache_dir: PathBuf,
    pub app_info: crate::parser::types::AppInfo,
    pub from_cache: bool,
}

impl EventStore {
    /// Load from event log, using cache if available
    pub fn load(log_path: &Path) -> Result<Self> {
        let key = cache_key(log_path)?;
        let dir = cache_dir(&key)?;

        if is_cache_fresh(&dir) {
            info!("Cache hit: {}", dir.display());
            // Read app info from a small metadata file if present
            let app_info = load_app_meta(&dir).unwrap_or_default();
            return Ok(EventStore {
                cache_dir: dir,
                app_info,
                from_cache: true,
            });
        }

        info!("Cache miss, parsing: {}", log_path.display());
        let parsed = parse_eventlog(log_path)?;
        ensure_cache_dir(&dir)?;

        // Write parquet files
        write_parquet(&dir.join("tasks.parquet"), build_tasks_batch(&parsed.tasks)?)?;
        write_parquet(&dir.join("stages.parquet"), build_stages_batch(&parsed.stages)?)?;
        write_parquet(&dir.join("jobs.parquet"), build_jobs_batch(&parsed.jobs)?)?;
        write_parquet(
            &dir.join("executors.parquet"),
            build_executors_batch(&parsed.executors)?,
        )?;

        // Save app meta
        save_app_meta(&dir, &parsed.app_info)?;

        Ok(EventStore {
            cache_dir: dir,
            app_info: parsed.app_info,
            from_cache: false,
        })
    }

    pub fn tasks_path(&self) -> PathBuf {
        self.cache_dir.join("tasks.parquet")
    }

    pub fn stages_path(&self) -> PathBuf {
        self.cache_dir.join("stages.parquet")
    }

    pub fn jobs_path(&self) -> PathBuf {
        self.cache_dir.join("jobs.parquet")
    }

    pub fn executors_path(&self) -> PathBuf {
        self.cache_dir.join("executors.parquet")
    }
}

fn write_parquet(path: &Path, batch: RecordBatch) -> Result<()> {
    let file = File::create(path)?;
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}

fn save_app_meta(dir: &Path, info: &crate::parser::types::AppInfo) -> Result<()> {
    let path = dir.join("app_meta.json");
    let json = serde_json::to_string(info)
        .map_err(|e| crate::error::InsightError::JsonParse { line: 0, source: e })?;
    std::fs::write(path, json)?;
    Ok(())
}

fn load_app_meta(dir: &Path) -> Option<crate::parser::types::AppInfo> {
    let path = dir.join("app_meta.json");
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}
