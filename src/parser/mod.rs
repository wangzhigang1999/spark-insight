pub mod events;
pub mod types;

use crate::error::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::warn;

use events::SparkEvent;
use types::{AppInfo, ExecutorInfo, JobInfo, StageInfo, TaskMetrics};

/// Parsed data from a Spark event log
#[derive(Debug, Default)]
pub struct ParsedEventLog {
    pub app_info: AppInfo,
    pub jobs: Vec<JobInfo>,
    pub stages: Vec<StageInfo>,
    pub tasks: Vec<TaskMetrics>,
    pub executors: Vec<ExecutorInfo>,
}

/// Parse a Spark event log file (JSON Lines format)
pub fn parse_eventlog(path: &Path) -> Result<ParsedEventLog> {
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} Parsing...")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );

    let reader = BufReader::new(pb.wrap_read(file));
    let mut result = ParsedEventLog::default();

    // Track pending state
    let mut job_stages: HashMap<i32, Vec<i32>> = HashMap::new();
    let mut stage_job_map: HashMap<i32, i32> = HashMap::new();
    let mut pending_jobs: HashMap<i32, (i64, Vec<i32>)> = HashMap::new();
    let mut pending_stages: HashMap<(i32, i32), types::StageInfo> = HashMap::new();
    let mut stage_task_counts: HashMap<i32, i32> = HashMap::new();
    let mut stage_failed_tasks: HashMap<i32, i32> = HashMap::new();
    let mut executor_map: HashMap<String, ExecutorInfo> = HashMap::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let event: SparkEvent = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(e) => {
                warn!("Line {}: failed to parse event: {}", line_num + 1, e);
                SparkEvent::Unknown
            }
        };

        match event {
            SparkEvent::ApplicationStart(e) => {
                result.app_info.app_name = e.app_name;
                result.app_info.app_id = e.app_id.unwrap_or_default();
                result.app_info.start_time = e.timestamp;
            }
            SparkEvent::ApplicationEnd(e) => {
                result.app_info.end_time = e.timestamp;
            }
            SparkEvent::JobStart(e) => {
                let stage_ids = e.stage_ids.clone();
                pending_jobs.insert(e.job_id, (e.submission_time, stage_ids.clone()));
                job_stages.insert(e.job_id, stage_ids.clone());
                for sid in &stage_ids {
                    stage_job_map.insert(*sid, e.job_id);
                }
            }
            SparkEvent::JobEnd(e) => {
                if let Some((submission_time, stage_ids)) = pending_jobs.remove(&e.job_id) {
                    let status = e.job_result.result.clone();
                    let failure_reason = e
                        .job_result
                        .exception
                        .as_ref()
                        .and_then(|ex| ex.message.clone());
                    let num_tasks: i32 = stage_ids
                        .iter()
                        .map(|sid| stage_task_counts.get(sid).copied().unwrap_or(0))
                        .sum();
                    let num_failed_tasks: i32 = stage_ids
                        .iter()
                        .map(|sid| stage_failed_tasks.get(sid).copied().unwrap_or(0))
                        .sum();
                    result.jobs.push(JobInfo {
                        job_id: e.job_id,
                        submission_time,
                        completion_time: e.completion_time,
                        status,
                        num_tasks,
                        num_failed_tasks,
                        failure_reason,
                        stage_ids,
                    });
                }
            }
            SparkEvent::StageSubmitted(e) => {
                let si = &e.stage_info;
                let job_id = stage_job_map.get(&si.stage_id).copied().unwrap_or(-1);
                let stage = StageInfo {
                    stage_id: si.stage_id,
                    stage_attempt_id: si.stage_attempt_id,
                    job_id,
                    name: si.stage_name.clone(),
                    num_tasks: si.num_tasks,
                    num_failed_tasks: 0,
                    submission_time: si.submission_time.unwrap_or(0),
                    completion_time: 0,
                    duration_ms: 0,
                };
                pending_stages.insert((si.stage_id, si.stage_attempt_id), stage);
                stage_task_counts.insert(si.stage_id, si.num_tasks);
            }
            SparkEvent::StageCompleted(e) => {
                let si = &e.stage_info;
                let key = (si.stage_id, si.stage_attempt_id);
                if let Some(mut stage) = pending_stages.remove(&key) {
                    let completion = si.completion_time.unwrap_or(0);
                    stage.completion_time = completion;
                    if completion > 0 && stage.submission_time > 0 {
                        stage.duration_ms = completion - stage.submission_time;
                    }
                    stage.num_failed_tasks =
                        stage_failed_tasks.get(&si.stage_id).copied().unwrap_or(0);
                    result.stages.push(stage);
                }
            }
            SparkEvent::TaskEnd(e) => {
                let ti = &e.task_info;
                let failed = ti.failed.unwrap_or(false);
                if failed {
                    *stage_failed_tasks.entry(e.stage_id).or_insert(0) += 1;
                }

                let finish_time = ti.finish_time.unwrap_or(0);
                let launch_time = ti.launch_time;
                let duration_ms = if finish_time > launch_time {
                    finish_time - launch_time
                } else {
                    0
                };

                let mut task = TaskMetrics {
                    stage_id: e.stage_id,
                    task_id: ti.task_id,
                    task_attempt_id: ti.attempt,
                    executor_id: ti.executor_id.clone(),
                    host: ti.host.clone(),
                    status: e.task_end_reason.reason.clone(),
                    launch_time,
                    finish_time,
                    duration_ms,
                    failed,
                    ..Default::default()
                };

                if let Some(metrics) = &e.task_metrics {
                    if let Some(im) = &metrics.input_metrics {
                        task.input_bytes = im.bytes_read.unwrap_or(0);
                        task.input_records = im.records_read.unwrap_or(0);
                    }
                    if let Some(om) = &metrics.output_metrics {
                        task.output_bytes = om.bytes_written.unwrap_or(0);
                        task.output_records = om.records_written.unwrap_or(0);
                    }
                    if let Some(srm) = &metrics.shuffle_read_metrics {
                        task.shuffle_read_bytes = srm
                            .remote_bytes_read
                            .unwrap_or(0)
                            .saturating_add(srm.local_bytes_read.unwrap_or(0));
                        task.shuffle_read_records = srm.total_records_read.unwrap_or(0);
                    }
                    if let Some(swm) = &metrics.shuffle_write_metrics {
                        task.shuffle_write_bytes = swm.shuffle_bytes_written.unwrap_or(0);
                        task.shuffle_write_records = swm.shuffle_records_written.unwrap_or(0);
                    }
                    task.memory_spill_bytes = metrics.memory_bytes_spilled.unwrap_or(0);
                    task.disk_spill_bytes = metrics.disk_bytes_spilled.unwrap_or(0);
                    task.gc_time_ms = metrics.jvm_gc_time.unwrap_or(0);
                    task.peak_exec_memory = metrics.peak_execution_memory.unwrap_or(0);
                    task.executor_run_time_ms = metrics.executor_run_time.unwrap_or(0);
                }

                result.tasks.push(task);
            }
            SparkEvent::ExecutorAdded(e) => {
                executor_map.insert(
                    e.executor_id.clone(),
                    ExecutorInfo {
                        executor_id: e.executor_id,
                        add_time: e.timestamp,
                        remove_time: 0,
                        remove_reason: None,
                        max_memory: 0,
                        total_cores: e.executor_info.total_cores,
                    },
                );
            }
            SparkEvent::ExecutorRemoved(e) => {
                if let Some(ex) = executor_map.get_mut(&e.executor_id) {
                    ex.remove_time = e.timestamp;
                    ex.remove_reason = Some(e.removed_reason);
                }
            }
            SparkEvent::TaskStart(_) | SparkEvent::Unknown => {}
        }
    }

    pb.finish_with_message("Parsed");

    // Flush any pending stages (incomplete logs)
    for (_, stage) in pending_stages {
        result.stages.push(stage);
    }
    // Flush executors
    for (_, ex) in executor_map {
        result.executors.push(ex);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_fixture(lines: &[&str]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        f
    }

    #[test]
    fn test_parse_app_start_end() {
        let f = make_fixture(&[
            r#"{"Event":"SparkListenerApplicationStart","App Name":"test-app","App ID":"app-001","Timestamp":1000}"#,
            r#"{"Event":"SparkListenerApplicationEnd","Timestamp":5000}"#,
        ]);
        let result = parse_eventlog(f.path()).unwrap();
        assert_eq!(result.app_info.app_name, "test-app");
        assert_eq!(result.app_info.app_id, "app-001");
        assert_eq!(result.app_info.start_time, 1000);
        assert_eq!(result.app_info.end_time, 5000);
    }

    #[test]
    fn test_parse_job_lifecycle() {
        let f = make_fixture(&[
            r#"{"Event":"SparkListenerJobStart","Job ID":0,"Submission Time":1000,"Stage IDs":[0,1],"Properties":{}}"#,
            r#"{"Event":"SparkListenerJobEnd","Job ID":0,"Completion Time":5000,"Job Result":{"Result":"JobSucceeded"}}"#,
        ]);
        let result = parse_eventlog(f.path()).unwrap();
        assert_eq!(result.jobs.len(), 1);
        assert_eq!(result.jobs[0].job_id, 0);
        assert_eq!(result.jobs[0].status, "JobSucceeded");
    }

    #[test]
    fn test_parse_stage_lifecycle() {
        let f = make_fixture(&[
            r#"{"Event":"SparkListenerStageSubmitted","Stage Info":{"Stage ID":0,"Stage Attempt ID":0,"Stage Name":"count","Number of Tasks":4,"Submission Time":1000}}"#,
            r#"{"Event":"SparkListenerStageCompleted","Stage Info":{"Stage ID":0,"Stage Attempt ID":0,"Stage Name":"count","Number of Tasks":4,"Submission Time":1000,"Completion Time":3000}}"#,
        ]);
        let result = parse_eventlog(f.path()).unwrap();
        assert_eq!(result.stages.len(), 1);
        assert_eq!(result.stages[0].stage_id, 0);
        assert_eq!(result.stages[0].duration_ms, 2000);
    }

    #[test]
    fn test_unknown_events_ignored() {
        let f = make_fixture(&[
            r#"{"Event":"SparkListenerEnvironmentUpdate","JVM Information":{}}"#,
            r#"{"Event":"SparkListenerBlockManagerAdded","Block Manager ID":{}}"#,
        ]);
        let result = parse_eventlog(f.path()).unwrap();
        assert!(result.jobs.is_empty());
        assert!(result.stages.is_empty());
    }
}
