use crate::error::Result;
use crate::parser::types::{ExecutorInfo, JobInfo, StageInfo, TaskMetrics};
use arrow::array::{
    BooleanBuilder, Int32Builder, Int64Builder, StringBuilder,
};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

use super::schema::{executors_schema, jobs_schema, stages_schema, tasks_schema};

pub fn build_tasks_batch(tasks: &[TaskMetrics]) -> Result<RecordBatch> {
    let schema = tasks_schema();
    let n = tasks.len();

    let mut stage_id = Int32Builder::with_capacity(n);
    let mut task_id = Int64Builder::with_capacity(n);
    let mut task_attempt_id = Int32Builder::with_capacity(n);
    let mut executor_id = StringBuilder::with_capacity(n, n * 8);
    let mut host = StringBuilder::with_capacity(n, n * 16);
    let mut status = StringBuilder::with_capacity(n, n * 8);
    let mut launch_time = Int64Builder::with_capacity(n);
    let mut finish_time = Int64Builder::with_capacity(n);
    let mut duration_ms = Int64Builder::with_capacity(n);
    let mut failed = BooleanBuilder::with_capacity(n);
    let mut input_bytes = Int64Builder::with_capacity(n);
    let mut input_records = Int64Builder::with_capacity(n);
    let mut output_bytes = Int64Builder::with_capacity(n);
    let mut output_records = Int64Builder::with_capacity(n);
    let mut shuffle_read_bytes = Int64Builder::with_capacity(n);
    let mut shuffle_read_records = Int64Builder::with_capacity(n);
    let mut shuffle_write_bytes = Int64Builder::with_capacity(n);
    let mut shuffle_write_records = Int64Builder::with_capacity(n);
    let mut memory_spill_bytes = Int64Builder::with_capacity(n);
    let mut disk_spill_bytes = Int64Builder::with_capacity(n);
    let mut gc_time_ms = Int64Builder::with_capacity(n);
    let mut peak_exec_memory = Int64Builder::with_capacity(n);
    let mut executor_run_time_ms = Int64Builder::with_capacity(n);

    for t in tasks {
        stage_id.append_value(t.stage_id);
        task_id.append_value(t.task_id);
        task_attempt_id.append_value(t.task_attempt_id);
        executor_id.append_value(&t.executor_id);
        host.append_value(&t.host);
        status.append_value(&t.status);
        launch_time.append_value(t.launch_time);
        finish_time.append_value(t.finish_time);
        duration_ms.append_value(t.duration_ms);
        failed.append_value(t.failed);
        input_bytes.append_value(t.input_bytes);
        input_records.append_value(t.input_records);
        output_bytes.append_value(t.output_bytes);
        output_records.append_value(t.output_records);
        shuffle_read_bytes.append_value(t.shuffle_read_bytes);
        shuffle_read_records.append_value(t.shuffle_read_records);
        shuffle_write_bytes.append_value(t.shuffle_write_bytes);
        shuffle_write_records.append_value(t.shuffle_write_records);
        memory_spill_bytes.append_value(t.memory_spill_bytes);
        disk_spill_bytes.append_value(t.disk_spill_bytes);
        gc_time_ms.append_value(t.gc_time_ms);
        peak_exec_memory.append_value(t.peak_exec_memory);
        executor_run_time_ms.append_value(t.executor_run_time_ms);
    }

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(stage_id.finish()),
            Arc::new(task_id.finish()),
            Arc::new(task_attempt_id.finish()),
            Arc::new(executor_id.finish()),
            Arc::new(host.finish()),
            Arc::new(status.finish()),
            Arc::new(launch_time.finish()),
            Arc::new(finish_time.finish()),
            Arc::new(duration_ms.finish()),
            Arc::new(failed.finish()),
            Arc::new(input_bytes.finish()),
            Arc::new(input_records.finish()),
            Arc::new(output_bytes.finish()),
            Arc::new(output_records.finish()),
            Arc::new(shuffle_read_bytes.finish()),
            Arc::new(shuffle_read_records.finish()),
            Arc::new(shuffle_write_bytes.finish()),
            Arc::new(shuffle_write_records.finish()),
            Arc::new(memory_spill_bytes.finish()),
            Arc::new(disk_spill_bytes.finish()),
            Arc::new(gc_time_ms.finish()),
            Arc::new(peak_exec_memory.finish()),
            Arc::new(executor_run_time_ms.finish()),
        ],
    )?;
    Ok(batch)
}

pub fn build_stages_batch(stages: &[StageInfo]) -> Result<RecordBatch> {
    let schema = stages_schema();
    let n = stages.len();

    let mut stage_id = Int32Builder::with_capacity(n);
    let mut stage_attempt_id = Int32Builder::with_capacity(n);
    let mut job_id = Int32Builder::with_capacity(n);
    let mut name = StringBuilder::with_capacity(n, n * 16);
    let mut num_tasks = Int32Builder::with_capacity(n);
    let mut num_failed_tasks = Int32Builder::with_capacity(n);
    let mut submission_time = Int64Builder::with_capacity(n);
    let mut completion_time = Int64Builder::with_capacity(n);
    let mut duration_ms = Int64Builder::with_capacity(n);

    for s in stages {
        stage_id.append_value(s.stage_id);
        stage_attempt_id.append_value(s.stage_attempt_id);
        job_id.append_value(s.job_id);
        name.append_value(&s.name);
        num_tasks.append_value(s.num_tasks);
        num_failed_tasks.append_value(s.num_failed_tasks);
        submission_time.append_value(s.submission_time);
        completion_time.append_value(s.completion_time);
        duration_ms.append_value(s.duration_ms);
    }

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(stage_id.finish()),
            Arc::new(stage_attempt_id.finish()),
            Arc::new(job_id.finish()),
            Arc::new(name.finish()),
            Arc::new(num_tasks.finish()),
            Arc::new(num_failed_tasks.finish()),
            Arc::new(submission_time.finish()),
            Arc::new(completion_time.finish()),
            Arc::new(duration_ms.finish()),
        ],
    )?;
    Ok(batch)
}

pub fn build_jobs_batch(jobs: &[JobInfo]) -> Result<RecordBatch> {
    let schema = jobs_schema();
    let n = jobs.len();

    let mut job_id = Int32Builder::with_capacity(n);
    let mut submission_time = Int64Builder::with_capacity(n);
    let mut completion_time = Int64Builder::with_capacity(n);
    let mut status = StringBuilder::with_capacity(n, n * 8);
    let mut num_tasks = Int32Builder::with_capacity(n);
    let mut num_failed_tasks = Int32Builder::with_capacity(n);
    let mut failure_reason = StringBuilder::with_capacity(n, n * 8);

    for j in jobs {
        job_id.append_value(j.job_id);
        submission_time.append_value(j.submission_time);
        completion_time.append_value(j.completion_time);
        status.append_value(&j.status);
        num_tasks.append_value(j.num_tasks);
        num_failed_tasks.append_value(j.num_failed_tasks);
        match &j.failure_reason {
            Some(r) => failure_reason.append_value(r),
            None => failure_reason.append_null(),
        }
    }

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(job_id.finish()),
            Arc::new(submission_time.finish()),
            Arc::new(completion_time.finish()),
            Arc::new(status.finish()),
            Arc::new(num_tasks.finish()),
            Arc::new(num_failed_tasks.finish()),
            Arc::new(failure_reason.finish()),
        ],
    )?;
    Ok(batch)
}

pub fn build_executors_batch(executors: &[ExecutorInfo]) -> Result<RecordBatch> {
    let schema = executors_schema();
    let n = executors.len();

    let mut executor_id = StringBuilder::with_capacity(n, n * 8);
    let mut add_time = Int64Builder::with_capacity(n);
    let mut remove_time = Int64Builder::with_capacity(n);
    let mut remove_reason = StringBuilder::with_capacity(n, n * 16);
    let mut max_memory = Int64Builder::with_capacity(n);
    let mut total_cores = Int32Builder::with_capacity(n);

    for e in executors {
        executor_id.append_value(&e.executor_id);
        add_time.append_value(e.add_time);
        remove_time.append_value(e.remove_time);
        match &e.remove_reason {
            Some(r) => remove_reason.append_value(r),
            None => remove_reason.append_null(),
        }
        max_memory.append_value(e.max_memory);
        total_cores.append_value(e.total_cores);
    }

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(executor_id.finish()),
            Arc::new(add_time.finish()),
            Arc::new(remove_time.finish()),
            Arc::new(remove_reason.finish()),
            Arc::new(max_memory.finish()),
            Arc::new(total_cores.finish()),
        ],
    )?;
    Ok(batch)
}
