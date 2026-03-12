use crate::query::{builtin, format::batch_to_json, QueryEngine};
use crate::store::EventStore;
use anyhow::Result;
use serde_json::{json, Value};

pub async fn run_json_report(store: &EventStore) -> Result<Value> {
    let engine = QueryEngine::new(store).await?;

    let app = &store.app_info;

    let jobs_batch = engine.execute_sql_single(builtin::JOB_SUMMARY).await?;
    let stages_batch = engine.execute_sql_single(builtin::STAGE_SUMMARY).await?;
    let skew_batch = engine.execute_sql_single(builtin::SKEW_ANALYSIS).await?;
    let spill_batch = engine.execute_sql_single(builtin::SPILL_ANALYSIS).await?;
    let shuffle_batch = engine.execute_sql_single(builtin::SHUFFLE_ANALYSIS).await?;
    let gc_batch = engine.execute_sql_single(builtin::GC_ANALYSIS).await?;
    let slow_tasks_batch = engine.execute_sql_single(builtin::TOP_SLOW_TASKS).await?;
    let failed_batch = engine.execute_sql_single(builtin::FAILED_TASKS).await?;

    let report = json!({
        "app": {
            "name": app.app_name,
            "id": app.app_id,
            "start_time": app.start_time,
            "end_time": app.end_time,
            "duration_ms": app.end_time - app.start_time,
        },
        "jobs": batch_to_json(&jobs_batch),
        "stages": batch_to_json(&stages_batch),
        "diagnostics": {
            "skew": batch_to_json(&skew_batch),
            "spill": batch_to_json(&spill_batch),
            "shuffle": batch_to_json(&shuffle_batch),
            "gc": batch_to_json(&gc_batch),
        },
        "top_slow_tasks": batch_to_json(&slow_tasks_batch),
        "failed_tasks": batch_to_json(&failed_batch),
    });

    Ok(report)
}
