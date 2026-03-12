pub mod builtin;
pub mod format;

use crate::error::Result;
use crate::store::EventStore;
use arrow::record_batch::RecordBatch;
use datafusion::prelude::*;
use datafusion::execution::context::SessionConfig;
use std::path::PathBuf;

pub struct QueryEngine {
    ctx: SessionContext,
}

impl QueryEngine {
    pub async fn new(store: &EventStore) -> Result<Self> {
        let config = SessionConfig::new()
            .with_information_schema(true);
        let ctx = SessionContext::new_with_config(config);

        // Register parquet tables
        register_parquet(&ctx, "tasks", &store.tasks_path()).await?;
        register_parquet(&ctx, "stages", &store.stages_path()).await?;
        register_parquet(&ctx, "jobs", &store.jobs_path()).await?;
        register_parquet(&ctx, "executors", &store.executors_path()).await?;

        Ok(QueryEngine { ctx })
    }

    /// Execute a SQL query and return RecordBatches
    pub async fn execute_sql(&self, sql: &str) -> Result<Vec<RecordBatch>> {
        let df = self.ctx.sql(sql).await?;
        let batches = df.collect().await?;
        Ok(batches)
    }

    /// Execute SQL and return as a single concatenated batch (for display)
    pub async fn execute_sql_single(&self, sql: &str) -> Result<RecordBatch> {
        let batches = self.execute_sql(sql).await?;
        if batches.is_empty() {
            return Ok(RecordBatch::new_empty(
                arrow::datatypes::Schema::empty().into(),
            ));
        }
        let schema = batches[0].schema();
        let batch = arrow::compute::concat_batches(&schema, &batches)?;
        Ok(batch)
    }
}

async fn register_parquet(ctx: &SessionContext, name: &str, path: &PathBuf) -> Result<()> {
    ctx.register_parquet(name, path.to_str().unwrap(), ParquetReadOptions::default())
        .await?;
    Ok(())
}
