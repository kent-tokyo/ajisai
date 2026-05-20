use crate::utils::resolve_safe_path;
use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, ValueType},
    AjisaiError, Transform,
};
use arrow_array::{
    ArrayRef, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray,
};
use arrow_schema::{Field as ArrowField, Schema};
use async_trait::async_trait;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParquetFileOutputConfig {
    pub filename: String,
    #[serde(default = "default_compression")]
    pub compression: String,
}

fn default_compression() -> String {
    "snappy".into()
}

pub struct ParquetFileOutput {
    config: ParquetFileOutputConfig,
    resolved_path: String,
    buffer: Vec<Row>,
}

impl ParquetFileOutput {
    pub fn new(config: ParquetFileOutputConfig) -> Self {
        let resolved_path = config.filename.clone();
        Self { config, resolved_path, buffer: Vec::new() }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ParquetFileOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_arrow_schema(schema: &RowSchema) -> Schema {
        let fields: Vec<ArrowField> = schema
            .fields
            .iter()
            .map(|f| ArrowField::new(f.name.as_str(), crate::parquet_utils::value_type_to_arrow(&f.value_type), true))
            .collect();
        Schema::new(fields)
    }

    fn rows_to_batch(rows: &[Row], arrow_schema: &Arc<Schema>) -> Result<RecordBatch> {
        if rows.is_empty() {
            return Ok(RecordBatch::new_empty(arrow_schema.clone()));
        }
        let n = rows.len();
        let row_schema = &rows[0].schema;

        let columns: Vec<ArrayRef> = row_schema
            .fields
            .iter()
            .enumerate()
            .map(|(col_idx, field)| -> ArrayRef {
                match field.value_type {
                    ValueType::Integer => {
                        let vals: Vec<Option<i64>> = (0..n)
                            .map(|r| rows[r].get_by_index(col_idx).and_then(|v| v.as_int()))
                            .collect();
                        Arc::new(Int64Array::from(vals))
                    }
                    ValueType::Float => {
                        let vals: Vec<Option<f64>> = (0..n)
                            .map(|r| rows[r].get_by_index(col_idx).and_then(|v| v.as_float()))
                            .collect();
                        Arc::new(Float64Array::from(vals))
                    }
                    ValueType::Boolean => {
                        let vals: Vec<Option<bool>> = (0..n)
                            .map(|r| rows[r].get_by_index(col_idx).and_then(|v| v.as_bool()))
                            .collect();
                        Arc::new(BooleanArray::from(vals))
                    }
                    _ => {
                        let vals: Vec<Option<String>> = (0..n)
                            .map(|r| {
                                rows[r].get_by_index(col_idx).map(|v| {
                                    if v.is_null() { None } else { Some(v.to_display_string()) }
                                }).flatten()
                            })
                            .collect();
                        Arc::new(StringArray::from(vals))
                    }
                }
            })
            .collect();

        RecordBatch::try_new(arrow_schema.clone(), columns)
            .map_err(|e| AjisaiError::Config(format!("Failed to build record batch: {}", e)))
    }
}

#[async_trait]
impl Transform for ParquetFileOutput {
    fn name(&self) -> &str {
        "ParquetFileOutput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_path = ctx.resolve(&self.config.filename);
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        self.buffer.push(row);
        Ok(vec![])
    }

    async fn close(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let row_schema = self.buffer[0].schema.clone();
        let arrow_schema = Arc::new(Self::build_arrow_schema(&row_schema));
        let batch = Self::rows_to_batch(&self.buffer, &arrow_schema)?;

        let compression = match self.config.compression.to_lowercase().as_str() {
            "gzip" => Compression::GZIP(Default::default()),
            "none" | "uncompressed" => Compression::UNCOMPRESSED,
            _ => Compression::SNAPPY,
        };

        let props = WriterProperties::builder()
            .set_compression(compression)
            .build();

        let safe_path = resolve_safe_path(&self.resolved_path)?;
        let path_str = self.resolved_path.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::File::create(&safe_path)
                .map_err(|e| AjisaiError::Config(format!("Cannot create Parquet file '{}': {}", path_str, e)))?;
            let mut writer = ArrowWriter::try_new(file, arrow_schema, Some(props))
                .map_err(|e| AjisaiError::Config(format!("Cannot create Parquet writer: {}", e)))?;
            writer
                .write(&batch)
                .map_err(|e| AjisaiError::Config(format!("Parquet write error: {}", e)))?;
            writer
                .close()
                .map_err(|e| AjisaiError::Config(format!("Parquet close error: {}", e)))?;
            Ok(())
        })
        .await
        .map_err(|e| AjisaiError::Config(format!("Spawn blocking failed: {}", e)))??;

        Ok(())
    }
}
