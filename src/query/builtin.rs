/// Built-in diagnostic SQL queries

pub const SKEW_ANALYSIS: &str = r#"
SELECT
    stage_id,
    COUNT(*) AS task_count,
    MAX(duration_ms) AS max_ms,
    AVG(duration_ms) AS avg_ms,
    CAST(MAX(duration_ms) AS DOUBLE) / NULLIF(AVG(duration_ms), 0) AS skew_ratio,
    SUM(memory_spill_bytes + disk_spill_bytes) AS total_spill_bytes
FROM tasks
GROUP BY stage_id
HAVING task_count > 1
ORDER BY skew_ratio DESC NULLS LAST
LIMIT 20
"#;

pub const SPILL_ANALYSIS: &str = r#"
SELECT
    stage_id,
    SUM(memory_spill_bytes) AS memory_spill_bytes,
    SUM(disk_spill_bytes) AS disk_spill_bytes,
    SUM(memory_spill_bytes + disk_spill_bytes) AS total_spill_bytes,
    COUNT(*) AS task_count
FROM tasks
WHERE memory_spill_bytes > 0 OR disk_spill_bytes > 0
GROUP BY stage_id
ORDER BY total_spill_bytes DESC
LIMIT 20
"#;

pub const SHUFFLE_ANALYSIS: &str = r#"
SELECT * FROM (
    SELECT
        stage_id,
        SUM(shuffle_read_bytes) AS shuffle_read_bytes,
        SUM(shuffle_write_bytes) AS shuffle_write_bytes,
        COUNT(*) AS task_count
    FROM tasks
    GROUP BY stage_id
    HAVING SUM(shuffle_read_bytes) > 0 OR SUM(shuffle_write_bytes) > 0
) __s
ORDER BY (shuffle_read_bytes + shuffle_write_bytes) DESC
LIMIT 20
"#;

pub const GC_ANALYSIS: &str = r#"
SELECT
    stage_id,
    AVG(CAST(gc_time_ms AS DOUBLE) / NULLIF(duration_ms, 0)) AS avg_gc_ratio,
    MAX(CAST(gc_time_ms AS DOUBLE) / NULLIF(duration_ms, 0)) AS max_gc_ratio,
    SUM(gc_time_ms) AS total_gc_ms,
    COUNT(*) AS task_count
FROM tasks
WHERE duration_ms > 0
GROUP BY stage_id
ORDER BY avg_gc_ratio DESC NULLS LAST
LIMIT 20
"#;

pub const STAGE_SUMMARY: &str = r#"
SELECT
    s.stage_id,
    s.job_id,
    s.name,
    s.num_tasks,
    s.duration_ms,
    COALESCE(t.max_task_ms, 0) AS max_task_ms,
    COALESCE(t.avg_task_ms, 0) AS avg_task_ms,
    COALESCE(t.total_input_bytes, 0) AS total_input_bytes,
    COALESCE(t.total_shuffle_read, 0) AS total_shuffle_read,
    COALESCE(t.total_shuffle_write, 0) AS total_shuffle_write,
    COALESCE(t.total_spill, 0) AS total_spill
FROM stages s
LEFT JOIN (
    SELECT
        stage_id,
        MAX(duration_ms) AS max_task_ms,
        AVG(duration_ms) AS avg_task_ms,
        SUM(input_bytes) AS total_input_bytes,
        SUM(shuffle_read_bytes) AS total_shuffle_read,
        SUM(shuffle_write_bytes) AS total_shuffle_write,
        SUM(memory_spill_bytes + disk_spill_bytes) AS total_spill
    FROM tasks
    GROUP BY stage_id
) t ON s.stage_id = t.stage_id
ORDER BY s.duration_ms DESC
"#;

pub const JOB_SUMMARY: &str = r#"
SELECT
    job_id,
    status,
    (completion_time - submission_time) AS duration_ms,
    num_tasks,
    num_failed_tasks
FROM jobs
ORDER BY job_id
"#;

pub const TOP_SLOW_TASKS: &str = r#"
SELECT
    stage_id,
    task_id,
    executor_id,
    host,
    duration_ms,
    gc_time_ms,
    memory_spill_bytes,
    disk_spill_bytes
FROM tasks
ORDER BY duration_ms DESC
LIMIT 20
"#;

pub const FAILED_TASKS: &str = r#"
SELECT
    stage_id,
    task_id,
    executor_id,
    host,
    status,
    duration_ms
FROM tasks
WHERE failed = true
ORDER BY stage_id, task_id
LIMIT 50
"#;

pub const EXECUTOR_SUMMARY: &str = r#"
SELECT
    executor_id,
    total_cores,
    (remove_time - add_time) AS active_ms,
    remove_reason
FROM executors
ORDER BY executor_id
"#;
