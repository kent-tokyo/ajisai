use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::Write as IoWrite;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum JsonOutputFormat {
    /// Write all rows as a JSON array
    #[default]
    Array,
    /// One JSON object per line (JSONL / NDJSON)
    Lines,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFileOutputConfig {
    pub filename: String,
    #[serde(default)]
    pub format: JsonOutputFormat,
    #[serde(default = "default_true")]
    pub pretty: bool,
}

fn default_true() -> bool {
    true
}

pub struct JsonFileOutput {
    config: JsonFileOutputConfig,
    buffer: Vec<serde_json::Map<String, serde_json::Value>>,
    file: Option<std::fs::File>,
    started: bool,
    resolved_filename: Option<String>,
    target_path: Option<PathBuf>,
    temporary_path: Option<PathBuf>,
}

impl JsonFileOutput {
    pub fn new(config: JsonFileOutputConfig) -> Self {
        Self {
            config,
            buffer: Vec::new(),
            file: None,
            started: false,
            resolved_filename: None,
            target_path: None,
            temporary_path: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: JsonFileOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn row_to_object(row: &Row) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (field, value) in row.schema.fields.iter().zip(row.values.iter()) {
            let jv = match value {
                Value::Str(s) => serde_json::Value::String(s.clone()),
                Value::Int(n) => serde_json::Value::Number((*n).into()),
                Value::Float(f) => serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
                Value::Bool(b) => serde_json::Value::Bool(*b),
                Value::Null => serde_json::Value::Null,
                other => serde_json::Value::String(other.to_display_string()),
            };
            map.insert(field.name.clone(), jv);
        }
        map
    }
}

#[async_trait]
impl Transform for JsonFileOutput {
    fn name(&self) -> &str {
        "JsonFileOutput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let filename = ctx.resolve(&self.config.filename);

        // Reject path traversal attempts
        if std::path::Path::new(&filename)
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(AjisaiError::Config("Path traversal not allowed".into()));
        }

        self.resolved_filename = Some(filename.clone());
        let target = PathBuf::from(&filename);
        let temporary = PathBuf::from(format!("{}.ajisai-tmp-{}", filename, std::process::id()));
        let f = std::fs::File::create(&temporary).map_err(AjisaiError::Io)?;
        self.file = Some(f);
        self.target_path = Some(target);
        self.temporary_path = Some(temporary);
        self.started = false;

        if matches!(self.config.format, JsonOutputFormat::Array) {
            let f = self.file.as_mut().unwrap();
            write!(f, "[").map_err(AjisaiError::Io)?;
        }
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let obj = Self::row_to_object(&row);

        match self.config.format {
            JsonOutputFormat::Lines => {
                let f = self
                    .file
                    .as_mut()
                    .ok_or_else(|| AjisaiError::Pipeline("JsonFileOutput not opened".into()))?;
                let line = serde_json::to_string(&obj)
                    .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;
                writeln!(f, "{}", line).map_err(AjisaiError::Io)?;
            }
            JsonOutputFormat::Array => {
                // Buffer all rows; write array in close()
                self.buffer.push(obj);
            }
        }
        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(ref mut f) = self.file {
            if matches!(self.config.format, JsonOutputFormat::Array) {
                let arr = serde_json::Value::Array(
                    self.buffer
                        .drain(..)
                        .map(serde_json::Value::Object)
                        .collect(),
                );
                let json = if self.config.pretty {
                    serde_json::to_string_pretty(&arr)
                } else {
                    serde_json::to_string(&arr)
                }
                .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;

                let temporary = self.temporary_path.as_ref().ok_or_else(|| {
                    AjisaiError::Pipeline("JsonFileOutput temporary path missing".into())
                })?;
                drop(self.file.take());
                std::fs::write(temporary, json).map_err(AjisaiError::Io)?;
            } else {
                f.flush().map_err(AjisaiError::Io)?;
                self.file = None;
            }
            let target = self.target_path.as_ref().ok_or_else(|| {
                AjisaiError::Pipeline("JsonFileOutput target path missing".into())
            })?;
            let temporary = self.temporary_path.as_ref().ok_or_else(|| {
                AjisaiError::Pipeline("JsonFileOutput temporary path missing".into())
            })?;
            std::fs::rename(temporary, target).map_err(AjisaiError::Io)?;
            self.temporary_path = None;
        }
        Ok(())
    }
}

impl Drop for JsonFileOutput {
    fn drop(&mut self) {
        if let Some(path) = self.temporary_path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, ValueType};
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn row() -> Row {
        Row::new(
            Arc::new(RowSchema::new(vec![Field::new("id", ValueType::Integer)])),
            vec![Value::Int(7)],
        )
    }

    #[tokio::test]
    async fn commits_json_array_atomically() {
        let target = NamedTempFile::new().unwrap();
        let path = target.path().display().to_string();
        let temporary = format!("{path}.ajisai-tmp-{}", std::process::id());
        let mut output = JsonFileOutput::new(JsonFileOutputConfig {
            filename: path.clone(),
            format: JsonOutputFormat::Array,
            pretty: false,
        });
        output.open(&ExecutionContext::new()).await.unwrap();
        output.process(row()).await.unwrap();
        output.close().await.unwrap();
        assert!(!std::path::Path::new(&temporary).exists());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "[{\"id\":7}]");
    }

    #[tokio::test]
    async fn drop_removes_uncommitted_json_output() {
        let target = NamedTempFile::new().unwrap();
        let path = target.path().display().to_string();
        let temporary = format!("{path}.ajisai-tmp-{}", std::process::id());
        let mut output = JsonFileOutput::new(JsonFileOutputConfig {
            filename: path,
            format: JsonOutputFormat::Lines,
            pretty: false,
        });
        output.open(&ExecutionContext::new()).await.unwrap();
        output.process(row()).await.unwrap();
        assert!(std::path::Path::new(&temporary).exists());
        drop(output);
        assert!(!std::path::Path::new(&temporary).exists());
    }
}
