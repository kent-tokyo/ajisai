use crate::utils::resolve_safe_path;
use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema},
    AjisaiError, Transform,
};
use arrow_array::RecordBatch;
use async_trait::async_trait;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParquetFileInputConfig {
    pub filename: String,
}

pub struct ParquetFileInput {
    config: ParquetFileInputConfig,
    resolved_path: String,
}

impl ParquetFileInput {
    pub fn new(config: ParquetFileInputConfig) -> Self {
        let resolved_path = config.filename.clone();
        Self { config, resolved_path }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ParquetFileInputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn batch_to_rows(batch: &RecordBatch, schema: &Arc<RowSchema>) -> Vec<Row> {
        let n = batch.num_rows();
        (0..n)
            .map(|r| {
                let values = (0..batch.num_columns())
                    .map(|c| crate::parquet_utils::array_value_at(batch.column(c).as_ref(), r))
                    .collect();
                Row::new(schema.clone(), values)
            })
            .collect()
    }
}

#[async_trait]
impl Transform for ParquetFileInput {
    fn name(&self) -> &str {
        "ParquetFileInput"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        Ok(RowSchema::new(vec![]))
    }

    fn is_source(&self) -> bool {
        true
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_path = ctx.resolve(&self.config.filename);
        Ok(())
    }

    async fn produce(&mut self, sender: Sender<Row>) -> Result<()> {
        let safe_path = resolve_safe_path(&self.resolved_path)?;
        let path_str = self.resolved_path.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<Vec<(Arc<RowSchema>, Vec<Row>)>> {
            let file = std::fs::File::open(&safe_path)
                .map_err(|e| AjisaiError::Config(format!("Cannot open Parquet file '{}': {}", path_str, e)))?;

            let builder = ParquetRecordBatchReaderBuilder::try_new(file)
                .map_err(|e| AjisaiError::Config(format!("Invalid Parquet file: {}", e)))?;

            let arrow_schema = builder.schema().clone();
            let fields: Vec<Field> = arrow_schema
                .fields()
                .iter()
                .map(|f| Field::new(f.name().as_str(), crate::parquet_utils::arrow_to_value_type(f.data_type())))
                .collect();
            let schema = Arc::new(RowSchema::new(fields));

            let reader = builder
                .build()
                .map_err(|e| AjisaiError::Config(format!("Cannot build Parquet reader: {}", e)))?;

            let mut batches = Vec::new();
            for batch in reader {
                let batch = batch.map_err(|e| AjisaiError::Config(format!("Parquet read error: {}", e)))?;
                let rows = Self::batch_to_rows(&batch, &schema);
                batches.push((schema.clone(), rows));
            }
            Ok(batches)
        })
        .await
        .map_err(|e| AjisaiError::Config(format!("Spawn blocking failed: {}", e)))??;

        for (_, rows) in result {
            for row in rows {
                if sender.send(row).await.is_err() {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
