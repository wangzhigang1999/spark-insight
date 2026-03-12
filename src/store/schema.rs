use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;

pub fn tasks_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("stage_id", DataType::Int32, false),
        Field::new("task_id", DataType::Int64, false),
        Field::new("task_attempt_id", DataType::Int32, false),
        Field::new("executor_id", DataType::Utf8, false),
        Field::new("host", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("launch_time", DataType::Int64, false),
        Field::new("finish_time", DataType::Int64, false),
        Field::new("duration_ms", DataType::Int64, false),
        Field::new("failed", DataType::Boolean, false),
        Field::new("input_bytes", DataType::Int64, false),
        Field::new("input_records", DataType::Int64, false),
        Field::new("output_bytes", DataType::Int64, false),
        Field::new("output_records", DataType::Int64, false),
        Field::new("shuffle_read_bytes", DataType::Int64, false),
        Field::new("shuffle_read_records", DataType::Int64, false),
        Field::new("shuffle_write_bytes", DataType::Int64, false),
        Field::new("shuffle_write_records", DataType::Int64, false),
        Field::new("memory_spill_bytes", DataType::Int64, false),
        Field::new("disk_spill_bytes", DataType::Int64, false),
        Field::new("gc_time_ms", DataType::Int64, false),
        Field::new("peak_exec_memory", DataType::Int64, false),
        Field::new("executor_run_time_ms", DataType::Int64, false),
    ]))
}

pub fn stages_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("stage_id", DataType::Int32, false),
        Field::new("stage_attempt_id", DataType::Int32, false),
        Field::new("job_id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("num_tasks", DataType::Int32, false),
        Field::new("num_failed_tasks", DataType::Int32, false),
        Field::new("submission_time", DataType::Int64, false),
        Field::new("completion_time", DataType::Int64, false),
        Field::new("duration_ms", DataType::Int64, false),
    ]))
}

pub fn jobs_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("job_id", DataType::Int32, false),
        Field::new("submission_time", DataType::Int64, false),
        Field::new("completion_time", DataType::Int64, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("num_tasks", DataType::Int32, false),
        Field::new("num_failed_tasks", DataType::Int32, false),
        Field::new("failure_reason", DataType::Utf8, true),
    ]))
}

pub fn executors_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("executor_id", DataType::Utf8, false),
        Field::new("add_time", DataType::Int64, false),
        Field::new("remove_time", DataType::Int64, false),
        Field::new("remove_reason", DataType::Utf8, true),
        Field::new("max_memory", DataType::Int64, false),
        Field::new("total_cores", DataType::Int32, false),
    ]))
}
