/// Processed task metrics (after parsing TaskEnd events)
#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    pub stage_id: i32,
    pub task_id: i64,
    pub task_attempt_id: i32,
    pub executor_id: String,
    pub host: String,
    pub status: String,
    pub launch_time: i64,
    pub finish_time: i64,
    pub duration_ms: i64,
    pub failed: bool,
    pub input_bytes: i64,
    pub input_records: i64,
    pub output_bytes: i64,
    pub output_records: i64,
    pub shuffle_read_bytes: i64,
    pub shuffle_read_records: i64,
    pub shuffle_write_bytes: i64,
    pub shuffle_write_records: i64,
    pub memory_spill_bytes: i64,
    pub disk_spill_bytes: i64,
    pub gc_time_ms: i64,
    pub peak_exec_memory: i64,
    pub executor_run_time_ms: i64,
}

/// Processed stage info
#[derive(Debug, Clone)]
pub struct StageInfo {
    pub stage_id: i32,
    pub stage_attempt_id: i32,
    pub job_id: i32,
    pub name: String,
    pub num_tasks: i32,
    pub num_failed_tasks: i32,
    pub submission_time: i64,
    pub completion_time: i64,
    pub duration_ms: i64,
}

/// Processed job info
#[derive(Debug, Clone)]
pub struct JobInfo {
    pub job_id: i32,
    pub submission_time: i64,
    pub completion_time: i64,
    pub status: String,
    pub num_tasks: i32,
    pub num_failed_tasks: i32,
    pub failure_reason: Option<String>,
    pub stage_ids: Vec<i32>,
}

/// Processed executor info
#[derive(Debug, Clone)]
pub struct ExecutorInfo {
    pub executor_id: String,
    pub add_time: i64,
    pub remove_time: i64,
    pub remove_reason: Option<String>,
    pub max_memory: i64,
    pub total_cores: i32,
}

/// Application-level info
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppInfo {
    pub app_name: String,
    pub app_id: String,
    pub start_time: i64,
    pub end_time: i64,
}
