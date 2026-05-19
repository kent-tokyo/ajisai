use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonFormat {
    /// Array of objects: [{"a":1},{"a":2}]
    Array,
    /// One JSON object per line (JSONL / NDJSON)
    Lines,
}

impl Default for JsonFormat {
    fn default() -> Self {
        JsonFormat::Array
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFileInputConfig {
    pub filename: String,
    #[serde(default)]
    pub format: JsonFormat,
    /// Explicit field definitions; inferred from first record if empty
    #[serde(default)]
    pub fields: Vec<JsonFieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFieldDef {
    pub name: String,
    #[serde(default = "default_type")]
    pub value_type: String,
}

fn default_type() -> String {
    "String".into()
}

pub struct JsonFileInput {
    config: JsonFileInputConfig,
    schema: Option<Arc<RowSchema>>,
    resolved_filename: Option<String>,
}

impl JsonFileInput {
    pub fn new(config: JsonFileInputConfig) -> Self {
        Self {
            config,
            schema: None,
            resolved_filename: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: JsonFileInputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn json_to_value(v: &serde_json::Value, target_type: &ValueType) -> Value {
        match (v, target_type) {
            (serde_json::Value::Null, _) => Value::Null,
            (serde_json::Value::Bool(b), _) => Value::Bool(*b),
            (serde_json::Value::Number(n), ValueType::Integer) => {
                n.as_i64().map(Value::Int).unwrap_or(Value::Null)
            }
            (serde_json::Value::Number(n), ValueType::Float) => {
                n.as_f64().map(Value::Float).unwrap_or(Value::Null)
            }
            (serde_json::Value::Number(n), _) => n
                .as_i64()
                .map(Value::Int)
                .or_else(|| n.as_f64().map(Value::Float))
                .unwrap_or(Value::Null),
            (serde_json::Value::String(s), ValueType::Integer) => {
                s.parse::<i64>().map(Value::Int).unwrap_or(Value::Null)
            }
            (serde_json::Value::String(s), ValueType::Float) => {
                s.parse::<f64>().map(Value::Float).unwrap_or(Value::Null)
            }
            (serde_json::Value::String(s), _) => Value::Str(s.clone()),
            (serde_json::Value::Array(arr), _) => {
                Value::Str(serde_json::to_string(arr).unwrap_or_default())
            }
            (serde_json::Value::Object(obj), _) => {
                Value::Str(serde_json::to_string(obj).unwrap_or_default())
            }
        }
    }

    fn infer_schema(obj: &serde_json::Map<String, serde_json::Value>) -> RowSchema {
        let fields: Vec<Field> = obj
            .iter()
            .map(|(k, v)| {
                let vt = match v {
                    serde_json::Value::Bool(_) => ValueType::Boolean,
                    serde_json::Value::Number(n) => {
                        if n.is_f64() {
                            ValueType::Float
                        } else {
                            ValueType::Integer
                        }
                    }
                    _ => ValueType::String,
                };
                Field::new(k.clone(), vt)
            })
            .collect();
        RowSchema::new(fields)
    }

    fn object_to_row(
        obj: &serde_json::Map<String, serde_json::Value>,
        schema: Arc<RowSchema>,
    ) -> Row {
        let values: Vec<Value> = schema
            .fields
            .iter()
            .map(|f| {
                obj.get(&f.name)
                    .map(|v| Self::json_to_value(v, &f.value_type))
                    .unwrap_or(Value::Null)
            })
            .collect();
        Row::new(schema, values)
    }
}

#[async_trait]
impl Transform for JsonFileInput {
    fn name(&self) -> &str {
        "JsonFileInput"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        Ok(RowSchema::default())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let filename = ctx.resolve(&self.config.filename);
        debug!("JsonFileInput opening '{}'", filename);

        // Reject path traversal attempts
        if std::path::Path::new(&filename)
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(AjisaiError::Config("Path traversal not allowed".into()));
        }

        self.resolved_filename = Some(filename);

        if !self.config.fields.is_empty() {
            let fields: Vec<Field> = self
                .config
                .fields
                .iter()
                .map(|f| {
                    let vt = match f.value_type.as_str() {
                        "Integer" => ValueType::Integer,
                        "Float" => ValueType::Float,
                        "Boolean" => ValueType::Boolean,
                        _ => ValueType::String,
                    };
                    Field::new(f.name.clone(), vt)
                })
                .collect();
            self.schema = Some(Arc::new(RowSchema::new(fields)));
        }
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row])
    }
    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
    fn is_source(&self) -> bool {
        true
    }

    async fn produce(&mut self, sender: mpsc::Sender<Row>) -> Result<()> {
        let filename = self
            .resolved_filename
            .as_deref()
            .unwrap_or(&self.config.filename)
            .to_owned();
        let format = self.config.format.clone();
        let schema_hint = self.schema.clone();

        let text = tokio::fs::read_to_string(&filename)
            .await
            .map_err(AjisaiError::Io)?;

        let result = tokio::task::spawn_blocking(move || {
            let objects: Vec<serde_json::Map<String, serde_json::Value>> = match format {
                JsonFormat::Array => {
                    let arr: serde_json::Value = serde_json::from_str(&text)
                        .map_err(|e| AjisaiError::Parse(e.to_string()))?;
                    arr.as_array()
                        .ok_or_else(|| AjisaiError::Parse("Expected a JSON array".into()))?
                        .iter()
                        .filter_map(|v| v.as_object().cloned())
                        .collect()
                }
                JsonFormat::Lines => text
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| {
                        let v: serde_json::Value = serde_json::from_str(l)
                            .map_err(|e| AjisaiError::Parse(e.to_string()))?;
                        v.as_object().cloned().ok_or_else(|| {
                            AjisaiError::Parse("Each line must be a JSON object".into())
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
            };

            let schema = schema_hint.unwrap_or_else(|| {
                let s = objects
                    .first()
                    .map(|o| JsonFileInput::infer_schema(o))
                    .unwrap_or_default();
                Arc::new(s)
            });

            let rows: Vec<Row> = objects
                .iter()
                .map(|obj| JsonFileInput::object_to_row(obj, schema.clone()))
                .collect();

            Ok::<(Arc<RowSchema>, Vec<Row>), AjisaiError>((schema, rows))
        })
        .await
        .map_err(|e| AjisaiError::Pipeline(format!("spawn_blocking: {}", e)))??;

        let (schema, rows) = result;
        self.schema = Some(schema);

        for row in rows {
            sender
                .send(row)
                .await
                .map_err(|_| AjisaiError::Pipeline("Downstream closed".into()))?;
        }
        Ok(())
    }
}
