use crate::utils::{resolve_context_path, resolve_safe_path};
use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
};
use async_trait::async_trait;
use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelFileOutputConfig {
    pub filename: String,
    #[serde(default = "default_sheet")]
    pub sheet_name: String,
    #[serde(default = "default_true")]
    pub header: bool,
}

fn default_sheet() -> String {
    "Sheet1".into()
}

fn default_true() -> bool {
    true
}

pub struct ExcelFileOutput {
    config: ExcelFileOutputConfig,
    resolved_path: String,
    buffer: Vec<Row>,
    temporary_path: Option<PathBuf>,
}

impl ExcelFileOutput {
    pub fn new(config: ExcelFileOutputConfig) -> Self {
        let resolved_path = config.filename.clone();
        Self {
            config,
            resolved_path,
            buffer: Vec::new(),
            temporary_path: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ExcelFileOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for ExcelFileOutput {
    fn name(&self) -> &str {
        "ExcelFileOutput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_path = resolve_context_path(ctx, &ctx.resolve(&self.config.filename))?
            .display()
            .to_string();
        self.temporary_path = None;
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

        let mut workbook = Workbook::new();
        let ws = workbook
            .add_worksheet()
            .set_name(&self.config.sheet_name)
            .map_err(|e| AjisaiError::Config(format!("Excel worksheet error: {}", e)))?;

        let first = &self.buffer[0];
        let mut excel_row: u32 = 0;

        if self.config.header {
            for (col, field) in first.schema.fields.iter().enumerate() {
                ws.write_string(excel_row, col as u16, &field.name)
                    .map_err(|e| AjisaiError::Config(e.to_string()))?;
            }
            excel_row += 1;
        }

        for row in &self.buffer {
            for (col, val) in row.values.iter().enumerate() {
                let c = col as u16;
                match val {
                    Value::Int(n) => {
                        ws.write_number(excel_row, c, *n as f64)
                            .map_err(|e| AjisaiError::Config(e.to_string()))?;
                    }
                    Value::Float(f) => {
                        ws.write_number(excel_row, c, *f)
                            .map_err(|e| AjisaiError::Config(e.to_string()))?;
                    }
                    Value::Bool(b) => {
                        ws.write_boolean(excel_row, c, *b)
                            .map_err(|e| AjisaiError::Config(e.to_string()))?;
                    }
                    Value::Null => {}
                    other => {
                        ws.write_string(excel_row, c, other.to_display_string())
                            .map_err(|e| AjisaiError::Config(e.to_string()))?;
                    }
                }
            }
            excel_row += 1;
        }

        let safe_path = resolve_safe_path(&self.resolved_path)?;
        let temporary = PathBuf::from(format!(
            "{}.ajisai-tmp-{}",
            safe_path.display(),
            std::process::id()
        ));
        self.temporary_path = Some(temporary.clone());
        workbook
            .save(&temporary)
            .map_err(|e| AjisaiError::Config(format!("Failed to save Excel file: {}", e)))?;
        std::fs::rename(&temporary, &safe_path).map_err(AjisaiError::Io)?;
        self.temporary_path = None;

        Ok(())
    }
}

impl Drop for ExcelFileOutput {
    fn drop(&mut self) {
        if let Some(path) = self.temporary_path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}
