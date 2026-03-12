use crate::query::{builtin, format::batch_to_table_string, QueryEngine};
use crate::store::EventStore;
use anyhow::Result;

/// Wrap a SQL query to apply a custom top-N limit (overrides any existing LIMIT)
fn with_limit(sql: &str, top: usize) -> String {
    format!("SELECT * FROM ({}) __q LIMIT {}", sql.trim(), top)
}

pub async fn run_analyze(store: &EventStore, focus: &[String], top: usize) -> Result<()> {
    let engine = QueryEngine::new(store).await?;

    let app = &store.app_info;
    println!("App: {}  ID: {}", app.app_name, app.app_id);
    if app.end_time > app.start_time {
        let dur = app.end_time - app.start_time;
        println!("Duration: {}ms", dur);
    }
    println!();

    let run_all = focus.is_empty();

    if run_all || focus.iter().any(|f| f == "summary") {
        println!("=== Job Summary ===");
        let batch = engine.execute_sql_single(builtin::JOB_SUMMARY).await?;
        println!("{}", batch_to_table_string(&batch));
        println!();

        println!("=== Stage Summary (top by duration) ===");
        let batch = engine
            .execute_sql_single(&with_limit(builtin::STAGE_SUMMARY, top))
            .await?;
        println!("{}", batch_to_table_string(&batch));
        println!();
    }

    if run_all || focus.iter().any(|f| f == "skew") {
        println!("=== Skew Analysis ===");
        let batch = engine
            .execute_sql_single(&with_limit(builtin::SKEW_ANALYSIS, top))
            .await?;
        println!("{}", batch_to_table_string(&batch));
        println!();
    }

    if run_all || focus.iter().any(|f| f == "spill") {
        println!("=== Spill Analysis ===");
        let batch = engine
            .execute_sql_single(&with_limit(builtin::SPILL_ANALYSIS, top))
            .await?;
        println!("{}", batch_to_table_string(&batch));
        println!();
    }

    if run_all || focus.iter().any(|f| f == "shuffle") {
        println!("=== Shuffle Analysis ===");
        let batch = engine
            .execute_sql_single(&with_limit(builtin::SHUFFLE_ANALYSIS, top))
            .await?;
        println!("{}", batch_to_table_string(&batch));
        println!();
    }

    if run_all || focus.iter().any(|f| f == "gc") {
        println!("=== GC Analysis ===");
        let batch = engine
            .execute_sql_single(&with_limit(builtin::GC_ANALYSIS, top))
            .await?;
        println!("{}", batch_to_table_string(&batch));
        println!();
    }

    Ok(())
}
