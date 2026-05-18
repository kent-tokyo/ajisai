use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::Write as IoWrite;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonOutputFormat {
    /// Write all rows as a JSON array
    Array,
    /// One JSON object per line (JSONL / NDJSON)
    Lines,
}

impl Default for JsonOutputFormat {
    fn default() -> Self { JsonOutputFormat::Array }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFileOutputConfig {
    pub filename: String,
    #[serde(default)]
    pub format:   JsonOutputFormat,
    #[serde(default = "default_true")]
    pub pretty:   bool,
}

fn default_true() -> bool { true }

pub struct JsonFileOutput {
    config:  JsonFileOutputConfig,
    buffer:  Vec<serde_json::Map<String, serde_json::Value>>,
    file:    Option<std::fs::File>,
    started: bool,
}

impl JsonFileOutput {
    pub fn new(config: JsonFileOutputConfig) -> Self {
        Self { config, buffer: Vec::new(), file: None, started: false }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: JsonFileOutputConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn row_to_object(row: &Row) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (field, value) in row.schema.fields.iter().zip(row.values.iter()) {
            let jv = match value {
                Value::Str(s)       => serde_json::Value::String(s.clone()),
                Value::Int(n)       => serde_json::Value::Number((*n).into()),
                Value::Float(f)     => {
                    serde_json::Number::from_f64(*f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                }
                Value::Bool(b)      => serde_json::Value::Bool(*b),
                Value::Null         => serde_json::Value::Null,
                other               => serde_json::Value::String(other.to_display_string()),
            };
            map.insert(field.name.clone(), jv);
        }
        map
    }
}

#[async_trait]
impl Transform for JsonFileOutput {
    fn name(&self) -> &str { "JsonFileOutput" }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let filename = ctx.resolve(&self.config.filename);
        let f = std::fs::File::create(&filename).map_err(AjisaiError::Io)?;
        self.file = Some(f);
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
                let f = self.file.as_mut()
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
                    self.buffer.drain(..)
                        .map(serde_json::Value::Object)
                        .collect()
                );
                let json = if self.config.pretty {
                    serde_json::to_string_pretty(&arr)
                } else {
                    serde_json::to_string(&arr)
                }
                .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;

                // Overwrite with the full array (we wrote "[" in open())
                // Re-open and write the complete JSON
                let filename = self.config.filename.clone();
                drop(self.file.take());
                std::fs::write(&filename, json).map_err(AjisaiError::Io)?;
            } else {
                f.flush().map_err(AjisaiError::Io)?;
                self.file = None;
            }
        }
        Ok(())
    }
}
