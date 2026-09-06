use crate::utils::{resolve_context_path, resolve_safe_path};
use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
};
use async_trait::async_trait;
use calamine::{Data, Reader, open_workbook_auto};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelFileInputConfig {
    pub filename: String,
    pub sheet_name: Option<String>,
    #[serde(default = "default_true")]
    pub header_present: bool,
}

fn default_true() -> bool {
    true
}

pub struct ExcelFileInput {
    config: ExcelFileInputConfig,
    resolved_path: String,
}

impl ExcelFileInput {
    pub fn new(config: ExcelFileInputConfig) -> Self {
        let resolved_path = config.filename.clone();
        Self {
            config,
            resolved_path,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ExcelFileInputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn data_to_value(cell: &Data) -> Value {
        match cell {
            Data::Int(n) => Value::Int(*n),
            Data::Float(f) => Value::Float(*f),
            Data::String(s) => Value::Str(s.clone()),
            Data::Bool(b) => Value::Bool(*b),
            Data::Empty => Value::Null,
            Data::Error(_) => Value::Null,
            Data::DateTime(dt) => Value::Str(dt.to_string()),
            Data::DateTimeIso(s) | Data::DurationIso(s) => Value::Str(s.clone()),
        }
    }
}

#[async_trait]
impl Transform for ExcelFileInput {
    fn name(&self) -> &str {
        "ExcelFileInput"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        // Schema is determined at produce() time; return empty placeholder
        Ok(RowSchema::new(vec![]))
    }

    fn is_source(&self) -> bool {
        true
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_path = resolve_context_path(ctx, &ctx.resolve(&self.config.filename))?
            .display()
            .to_string();
        Ok(())
    }

    async fn produce(&mut self, sender: Sender<Row>) -> Result<()> {
        let safe_path = resolve_safe_path(&self.resolved_path)?;
        let mut workbook = open_workbook_auto(&safe_path).map_err(|e| {
            AjisaiError::Config(format!(
                "Cannot open Excel file '{}': {}",
                self.resolved_path, e
            ))
        })?;

        let sheet_name = match &self.config.sheet_name {
            Some(n) => n.clone(),
            None => workbook
                .sheet_names()
                .first()
                .cloned()
                .ok_or_else(|| AjisaiError::Config("Excel workbook has no sheets".into()))?,
        };

        let range = workbook.worksheet_range(&sheet_name).map_err(|e| {
            AjisaiError::Config(format!("Cannot read sheet '{}': {}", sheet_name, e))
        })?;

        let mut rows_iter = range.rows();

        // Determine column names
        let headers: Vec<String> = if self.config.header_present {
            match rows_iter.next() {
                Some(header_row) => header_row
                    .iter()
                    .map(|c| match c {
                        Data::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect(),
                None => return Ok(()),
            }
        } else {
            // Generate col0, col1, ... based on first row width
            let first: Vec<_> = rows_iter.next().map(|r| r.to_vec()).unwrap_or_default();
            let names: Vec<String> = (0..first.len()).map(|i| format!("col{}", i)).collect();
            // Process the first row we already consumed
            if !first.is_empty() {
                let schema = Arc::new(RowSchema::new(
                    names
                        .iter()
                        .map(|n| Field::new(n.as_str(), ValueType::String))
                        .collect(),
                ));
                let values = first.iter().map(Self::data_to_value).collect();
                if sender.send(Row::new(schema, values)).await.is_err() {
                    return Ok(());
                }
            }
            names
        };

        let schema = Arc::new(RowSchema::new(
            headers
                .iter()
                .map(|n| Field::new(n.as_str(), ValueType::String))
                .collect(),
        ));

        for row in rows_iter {
            let mut values: Vec<Value> = row.iter().map(Self::data_to_value).collect();
            // Pad or truncate to match header count
            values.resize(headers.len(), Value::Null);
            if sender.send(Row::new(schema.clone(), values)).await.is_err() {
                break;
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
