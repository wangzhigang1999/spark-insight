pub mod app;
pub mod events;
pub mod ui;
pub mod widgets;

use crate::error::Result;
use crate::query::builtin;
use crate::query::format::batch_to_table_data;
use crate::query::QueryEngine;
use crate::store::EventStore;
use crate::tui::app::{AppState, MetricsDisplay, SkewBarItem, TreeItem};
use arrow::array::{Array, Float64Array, Int32Array, Int64Array};
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::Path;
use tokio::time::Duration;

pub async fn run_tui(log_path: &Path) -> Result<()> {
    // Load data
    let store = EventStore::load(log_path)?;
    let engine = QueryEngine::new(&store).await?;

    let file_name = log_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut state = AppState::new(store.app_info.clone(), file_name, store.from_cache);

    // Load initial data into state
    load_stage_tree(&engine, &mut state).await;
    load_skew_bars(&engine, &mut state).await;
    update_metrics_for_selection(&engine, &mut state).await;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = run_loop(&mut terminal, &mut state, &engine).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    engine: &QueryEngine,
) -> Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui::render(f, state))?;

        let prev_selected = state.tree_selected;

        if event::poll(tick_rate)? {
            let ev = event::read()?;
            let execute_sql = events::handle_event(ev, state);

            if state.should_quit {
                break;
            }

            if execute_sql {
                let sql = state.sql_history.last().cloned().unwrap_or_default();
                state.sql_executing = true;
                state.sql_error = None;
                state.sql_result = None;

                match engine.execute_sql_single(&sql).await {
                    Ok(batch) => {
                        state.sql_result = Some(batch_to_table_data(&batch));
                        state.sql_executing = false;
                    }
                    Err(e) => {
                        state.sql_error = Some(e.to_string());
                        state.sql_executing = false;
                    }
                }
            }

            // Update metrics if selection changed
            if state.tree_selected != prev_selected {
                update_metrics_for_selection(engine, state).await;
            }
        }
    }

    Ok(())
}

async fn load_stage_tree(engine: &QueryEngine, state: &mut AppState) {
    let Ok(batch) = engine.execute_sql_single(builtin::STAGE_SUMMARY).await else {
        return;
    };
    let Ok(jobs_batch) = engine.execute_sql_single(builtin::JOB_SUMMARY).await else {
        return;
    };

    // Parse jobs
    let mut job_items: Vec<(i32, i64, String)> = vec![];
    if let (Some(job_id_col), Some(dur_col), Some(status_col)) = (
        jobs_batch.column_by_name("job_id"),
        jobs_batch.column_by_name("duration_ms"),
        jobs_batch.column_by_name("status"),
    ) {
        let job_ids = job_id_col.as_any().downcast_ref::<Int32Array>();
        let durations = dur_col.as_any().downcast_ref::<Int64Array>();
        let statuses = status_col
            .as_any()
            .downcast_ref::<arrow::array::StringArray>();

        if let (Some(job_ids), Some(durations), Some(statuses)) = (job_ids, durations, statuses) {
            for i in 0..job_ids.len() {
                job_items.push((job_ids.value(i), durations.value(i), statuses.value(i).to_string()));
            }
        }
    }

    // Parse stages
    struct StageRow {
        stage_id: i32,
        job_id: i32,
        name: String,
        duration_ms: i64,
        num_tasks: i32,
        spill: i64,
        skew: f64,
    }

    let mut stage_rows: Vec<StageRow> = vec![];
    if let (
        Some(sid_col),
        Some(jid_col),
        Some(name_col),
        Some(dur_col),
        Some(ntasks_col),
        Some(spill_col),
    ) = (
        batch.column_by_name("stage_id"),
        batch.column_by_name("job_id"),
        batch.column_by_name("name"),
        batch.column_by_name("duration_ms"),
        batch.column_by_name("num_tasks"),
        batch.column_by_name("total_spill"),
    ) {
        let stage_ids = sid_col.as_any().downcast_ref::<Int32Array>();
        let job_ids = jid_col.as_any().downcast_ref::<Int32Array>();
        let names = name_col
            .as_any()
            .downcast_ref::<arrow::array::StringArray>();
        let durations = dur_col.as_any().downcast_ref::<Int64Array>();
        let ntasks = ntasks_col.as_any().downcast_ref::<Int32Array>();
        let spills = spill_col.as_any().downcast_ref::<Int64Array>();

        if let (Some(sids), Some(jids), Some(names), Some(durs), Some(nts), Some(spills)) =
            (stage_ids, job_ids, names, durations, ntasks, spills)
        {
            for i in 0..sids.len() {
                stage_rows.push(StageRow {
                    stage_id: sids.value(i),
                    job_id: jids.value(i),
                    name: names.value(i).to_string(),
                    duration_ms: durs.value(i),
                    num_tasks: nts.value(i),
                    spill: if spills.is_null(i) { 0 } else { spills.value(i) },
                    skew: 0.0,
                });
            }
        }
    }

    // Build tree items
    let mut items: Vec<TreeItem> = vec![];
    for (job_id, duration_ms, status) in &job_items {
        let job_spill: i64 = stage_rows
            .iter()
            .filter(|s| s.job_id == *job_id)
            .map(|s| s.spill)
            .sum();
        items.push(TreeItem {
            job_id: *job_id,
            stage_id: None,
            label: format!("Job {}", job_id),
            duration_ms: *duration_ms,
            is_expanded: true,
            is_job: true,
            num_tasks: 0,
            status: status.clone(),
            spill_bytes: job_spill,
            skew_ratio: 0.0,
        });

        for sr in stage_rows.iter().filter(|s| s.job_id == *job_id) {
            items.push(TreeItem {
                job_id: *job_id,
                stage_id: Some(sr.stage_id),
                label: sr.name.clone(),
                duration_ms: sr.duration_ms,
                is_expanded: false,
                is_job: false,
                num_tasks: sr.num_tasks,
                status: "Completed".to_string(),
                spill_bytes: sr.spill,
                skew_ratio: sr.skew,
            });
        }
    }

    // If no jobs (e.g. partial log), just add stages
    if items.is_empty() {
        for sr in &stage_rows {
            items.push(TreeItem {
                job_id: sr.job_id,
                stage_id: Some(sr.stage_id),
                label: sr.name.clone(),
                duration_ms: sr.duration_ms,
                is_expanded: false,
                is_job: false,
                num_tasks: sr.num_tasks,
                status: "Completed".to_string(),
                spill_bytes: sr.spill,
                skew_ratio: sr.skew,
            });
        }
    }

    state.tree_items = items;
}

async fn load_skew_bars(engine: &QueryEngine, state: &mut AppState) {
    let Ok(batch) = engine.execute_sql_single(builtin::SKEW_ANALYSIS).await else {
        return;
    };

    let mut bars: Vec<SkewBarItem> = vec![];

    let sid_col = batch.column_by_name("stage_id");
    let cnt_col = batch.column_by_name("task_count");
    let max_col = batch.column_by_name("max_ms");
    let skew_col = batch.column_by_name("skew_ratio");

    if let (Some(sids), Some(cnts), Some(maxes), Some(skews)) = (sid_col, cnt_col, max_col, skew_col) {
        let sids = sids.as_any().downcast_ref::<Int32Array>();
        let cnts = cnts.as_any().downcast_ref::<Int64Array>();
        let maxes = maxes.as_any().downcast_ref::<Int64Array>();
        let skews = skews.as_any().downcast_ref::<Float64Array>();

        if let (Some(sids), Some(cnts), Some(maxes), Some(skews)) = (sids, cnts, maxes, skews) {
            for i in 0..sids.len() {
                let skew = if skews.is_null(i) { 1.0 } else { skews.value(i) };
                let max_ms = maxes.value(i);
                let median_ms = if skew > 0.0 { (max_ms as f64 / skew) as i64 } else { max_ms };
                bars.push(SkewBarItem {
                    stage_id: sids.value(i),
                    task_count: cnts.value(i) as i32,
                    max_ms,
                    median_ms,
                    skew_ratio: skew,
                });
            }
        }
    }

    state.skew_bars = bars;
}

async fn update_metrics_for_selection(engine: &QueryEngine, state: &mut AppState) {
    let stage_filter = match state.selected_stage_id() {
        Some(sid) => format!("WHERE stage_id = {}", sid),
        None => "".to_string(),
    };

    let sql = format!(
        r#"SELECT
            SUM(input_bytes) AS input_bytes,
            SUM(memory_spill_bytes + disk_spill_bytes) AS spill_bytes,
            SUM(shuffle_read_bytes) AS shuffle_read_bytes,
            SUM(shuffle_write_bytes) AS shuffle_write_bytes,
            AVG(CAST(gc_time_ms AS DOUBLE) / NULLIF(duration_ms, 0)) AS gc_ratio,
            MAX(duration_ms) AS max_task_ms,
            AVG(duration_ms) AS avg_task_ms,
            COUNT(*) AS total_tasks,
            SUM(CAST(failed AS INT)) AS failed_tasks
        FROM tasks
        {}"#,
        stage_filter
    );

    let Ok(batch) = engine.execute_sql_single(&sql).await else {
        return;
    };

    if batch.num_rows() == 0 {
        return;
    }

    fn get_i64(batch: &arrow::record_batch::RecordBatch, col: &str) -> i64 {
        batch
            .column_by_name(col)
            .and_then(|c| c.as_any().downcast_ref::<Int64Array>())
            .and_then(|a| if a.is_null(0) { None } else { Some(a.value(0)) })
            .unwrap_or(0)
    }
    fn get_f64(batch: &arrow::record_batch::RecordBatch, col: &str) -> f64 {
        batch
            .column_by_name(col)
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .and_then(|a| if a.is_null(0) { None } else { Some(a.value(0)) })
            .unwrap_or(0.0)
    }

    // Count skew stages
    let skew_count = state.skew_bars.iter().filter(|b| b.skew_ratio > 10.0).count();

    state.metrics = MetricsDisplay {
        input_bytes: get_i64(&batch, "input_bytes"),
        spill_bytes: get_i64(&batch, "spill_bytes"),
        shuffle_read_bytes: get_i64(&batch, "shuffle_read_bytes"),
        shuffle_write_bytes: get_i64(&batch, "shuffle_write_bytes"),
        gc_ratio: get_f64(&batch, "gc_ratio"),
        skew_stages: skew_count,
        max_task_ms: get_i64(&batch, "max_task_ms"),
        avg_task_ms: get_f64(&batch, "avg_task_ms"),
        total_tasks: get_i64(&batch, "total_tasks") as i32,
        failed_tasks: get_i64(&batch, "failed_tasks") as i32,
    };
}
