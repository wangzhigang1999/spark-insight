use crate::error::{InsightError, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Returns the cache directory for a given app ID (or file path hash)
pub fn cache_dir(app_id: &str) -> Result<PathBuf> {
    let base = dirs::home_dir()
        .ok_or_else(|| InsightError::Cache("Cannot find home directory".to_string()))?;
    Ok(base.join(".spark-insight").join("cache").join(app_id))
}

/// Derive a cache key from the event log path + file metadata
pub fn cache_key(log_path: &Path) -> Result<String> {
    let meta = std::fs::metadata(log_path)?;
    let modified = meta
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let size = meta.len();
    let name = log_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    Ok(format!("{}_{}_{}", name, size, modified))
}

/// Check if a cache entry is fresh (parquet files exist)
pub fn is_cache_fresh(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }
    // Check that all 4 parquet files exist
    for table in &["tasks", "stages", "jobs", "executors"] {
        if !dir.join(format!("{}.parquet", table)).exists() {
            return false;
        }
    }
    true
}

/// Create cache directory
pub fn ensure_cache_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}

/// List all cached app IDs
pub fn list_caches() -> Result<Vec<String>> {
    let base = dirs::home_dir()
        .ok_or_else(|| InsightError::Cache("Cannot find home directory".to_string()))?;
    let cache_base = base.join(".spark-insight").join("cache");

    if !cache_base.exists() {
        return Ok(vec![]);
    }

    let mut entries = vec![];
    for entry in std::fs::read_dir(&cache_base)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
    }
    Ok(entries)
}

/// Clear a specific cache
pub fn clear_cache(app_id: &str) -> Result<()> {
    let dir = cache_dir(app_id)?;
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}
