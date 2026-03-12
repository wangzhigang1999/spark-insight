use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level Spark event types we care about
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "Event")]
pub enum SparkEvent {
    #[serde(rename = "SparkListenerApplicationStart")]
    ApplicationStart(ApplicationStartEvent),

    #[serde(rename = "SparkListenerApplicationEnd")]
    ApplicationEnd(ApplicationEndEvent),

    #[serde(rename = "SparkListenerJobStart")]
    JobStart(JobStartEvent),

    #[serde(rename = "SparkListenerJobEnd")]
    JobEnd(JobEndEvent),

    #[serde(rename = "SparkListenerStageSubmitted")]
    StageSubmitted(StageSubmittedEvent),

    #[serde(rename = "SparkListenerStageCompleted")]
    StageCompleted(StageCompletedEvent),

    #[serde(rename = "SparkListenerTaskStart")]
    TaskStart(TaskStartEvent),

    #[serde(rename = "SparkListenerTaskEnd")]
    TaskEnd(TaskEndEvent),

    #[serde(rename = "SparkListenerExecutorAdded")]
    ExecutorAdded(ExecutorAddedEvent),

    #[serde(rename = "SparkListenerExecutorRemoved")]
    ExecutorRemoved(ExecutorRemovedEvent),

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApplicationStartEvent {
    #[serde(rename = "App Name")]
    pub app_name: String,
    #[serde(rename = "App ID")]
    pub app_id: Option<String>,
    #[serde(rename = "Timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApplicationEndEvent {
    #[serde(rename = "Timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobStartEvent {
    #[serde(rename = "Job ID")]
    pub job_id: i32,
    #[serde(rename = "Submission Time")]
    pub submission_time: i64,
    #[serde(rename = "Stage IDs")]
    pub stage_ids: Vec<i32>,
    #[serde(rename = "Properties")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobEndEvent {
    #[serde(rename = "Job ID")]
    pub job_id: i32,
    #[serde(rename = "Completion Time")]
    pub completion_time: i64,
    #[serde(rename = "Job Result")]
    pub job_result: JobResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobResult {
    #[serde(rename = "Result")]
    pub result: String,
    #[serde(rename = "Exception")]
    pub exception: Option<ExceptionInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExceptionInfo {
    #[serde(rename = "Message")]
    pub message: Option<String>,
    #[serde(rename = "Stack Trace")]
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StageSubmittedEvent {
    #[serde(rename = "Stage Info")]
    pub stage_info: StageInfoRaw,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StageCompletedEvent {
    #[serde(rename = "Stage Info")]
    pub stage_info: StageInfoRaw,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StageInfoRaw {
    #[serde(rename = "Stage ID")]
    pub stage_id: i32,
    #[serde(rename = "Stage Attempt ID")]
    pub stage_attempt_id: i32,
    #[serde(rename = "Stage Name")]
    pub stage_name: String,
    #[serde(rename = "Number of Tasks")]
    pub num_tasks: i32,
    #[serde(rename = "Submission Time")]
    pub submission_time: Option<i64>,
    #[serde(rename = "Completion Time")]
    pub completion_time: Option<i64>,
    #[serde(rename = "Failure Reason")]
    pub failure_reason: Option<String>,
    #[serde(rename = "Accumulables")]
    pub accumulables: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskStartEvent {
    #[serde(rename = "Stage ID")]
    pub stage_id: i32,
    #[serde(rename = "Stage Attempt ID")]
    pub stage_attempt_id: i32,
    #[serde(rename = "Task Info")]
    pub task_info: TaskInfoRaw,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskEndEvent {
    #[serde(rename = "Stage ID")]
    pub stage_id: i32,
    #[serde(rename = "Stage Attempt ID")]
    pub stage_attempt_id: i32,
    #[serde(rename = "Task Type")]
    pub task_type: String,
    #[serde(rename = "Task End Reason")]
    pub task_end_reason: TaskEndReason,
    #[serde(rename = "Task Info")]
    pub task_info: TaskInfoRaw,
    #[serde(rename = "Task Executor Metrics")]
    pub task_executor_metrics: Option<TaskExecutorMetrics>,
    #[serde(rename = "Task Metrics")]
    pub task_metrics: Option<TaskMetricsRaw>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskEndReason {
    #[serde(rename = "Reason")]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskInfoRaw {
    #[serde(rename = "Task ID")]
    pub task_id: i64,
    #[serde(rename = "Index")]
    pub index: i32,
    #[serde(rename = "Attempt")]
    pub attempt: i32,
    #[serde(rename = "Partition ID")]
    pub partition_id: Option<i32>,
    #[serde(rename = "Launch Time")]
    pub launch_time: i64,
    #[serde(rename = "Executor ID")]
    pub executor_id: String,
    #[serde(rename = "Host")]
    pub host: String,
    #[serde(rename = "Locality")]
    pub locality: String,
    #[serde(rename = "Speculative")]
    pub speculative: bool,
    #[serde(rename = "Finish Time")]
    pub finish_time: Option<i64>,
    #[serde(rename = "Failed")]
    pub failed: Option<bool>,
    #[serde(rename = "Killed")]
    pub killed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskExecutorMetrics {
    #[serde(rename = "JVMHeapMemory")]
    pub jvm_heap_memory: Option<i64>,
    #[serde(rename = "JVMOffHeapMemory")]
    pub jvm_off_heap_memory: Option<i64>,
    #[serde(rename = "OnHeapExecutionMemory")]
    pub on_heap_execution_memory: Option<i64>,
    #[serde(rename = "OffHeapExecutionMemory")]
    pub off_heap_execution_memory: Option<i64>,
    #[serde(rename = "OnHeapStorageMemory")]
    pub on_heap_storage_memory: Option<i64>,
    #[serde(rename = "OffHeapStorageMemory")]
    pub off_heap_storage_memory: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskMetricsRaw {
    #[serde(rename = "Executor Deserialize Time")]
    pub executor_deserialize_time: Option<i64>,
    #[serde(rename = "Executor Deserialize CPU Time")]
    pub executor_deserialize_cpu_time: Option<i64>,
    #[serde(rename = "Executor Run Time")]
    pub executor_run_time: Option<i64>,
    #[serde(rename = "Executor CPU Time")]
    pub executor_cpu_time: Option<i64>,
    #[serde(rename = "Peak Execution Memory")]
    pub peak_execution_memory: Option<i64>,
    #[serde(rename = "Result Size")]
    pub result_size: Option<i64>,
    #[serde(rename = "JVM GC Time")]
    pub jvm_gc_time: Option<i64>,
    #[serde(rename = "Result Serialization Time")]
    pub result_serialization_time: Option<i64>,
    #[serde(rename = "Memory Bytes Spilled")]
    pub memory_bytes_spilled: Option<i64>,
    #[serde(rename = "Disk Bytes Spilled")]
    pub disk_bytes_spilled: Option<i64>,
    #[serde(rename = "Input Metrics")]
    pub input_metrics: Option<InputMetrics>,
    #[serde(rename = "Output Metrics")]
    pub output_metrics: Option<OutputMetrics>,
    #[serde(rename = "Shuffle Read Metrics")]
    pub shuffle_read_metrics: Option<ShuffleReadMetrics>,
    #[serde(rename = "Shuffle Write Metrics")]
    pub shuffle_write_metrics: Option<ShuffleWriteMetrics>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputMetrics {
    #[serde(rename = "Bytes Read")]
    pub bytes_read: Option<i64>,
    #[serde(rename = "Records Read")]
    pub records_read: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputMetrics {
    #[serde(rename = "Bytes Written")]
    pub bytes_written: Option<i64>,
    #[serde(rename = "Records Written")]
    pub records_written: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShuffleReadMetrics {
    #[serde(rename = "Remote Blocks Fetched")]
    pub remote_blocks_fetched: Option<i64>,
    #[serde(rename = "Local Blocks Fetched")]
    pub local_blocks_fetched: Option<i64>,
    #[serde(rename = "Fetch Wait Time")]
    pub fetch_wait_time: Option<i64>,
    #[serde(rename = "Remote Bytes Read")]
    pub remote_bytes_read: Option<i64>,
    #[serde(rename = "Remote Bytes Read To Disk")]
    pub remote_bytes_read_to_disk: Option<i64>,
    #[serde(rename = "Local Bytes Read")]
    pub local_bytes_read: Option<i64>,
    #[serde(rename = "Total Records Read")]
    pub total_records_read: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShuffleWriteMetrics {
    #[serde(rename = "Shuffle Bytes Written")]
    pub shuffle_bytes_written: Option<i64>,
    #[serde(rename = "Shuffle Write Time")]
    pub shuffle_write_time: Option<i64>,
    #[serde(rename = "Shuffle Records Written")]
    pub shuffle_records_written: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutorAddedEvent {
    #[serde(rename = "Executor ID")]
    pub executor_id: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: i64,
    #[serde(rename = "Executor Info")]
    pub executor_info: ExecutorInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutorRemovedEvent {
    #[serde(rename = "Executor ID")]
    pub executor_id: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: i64,
    #[serde(rename = "Removed Reason")]
    pub removed_reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutorInfo {
    #[serde(rename = "Host")]
    pub host: String,
    #[serde(rename = "Total Cores")]
    pub total_cores: i32,
    #[serde(rename = "Log Urls")]
    pub log_urls: Option<HashMap<String, String>>,
    #[serde(rename = "Attributes")]
    pub attributes: Option<HashMap<String, String>>,
    #[serde(rename = "Resources")]
    pub resources: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "Resource Profile Id")]
    pub resource_profile_id: Option<i32>,
}
