use ajisai_core::{
    context::ExecutionContext, error::Result, value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvFileInputConfig {
    pub filename:         String,
    #[serde(default = "default_delimiter")]
    pub delimiter:        char,
    #[serde(default = "default_true")]
    pub header_present:   bool,
    #[serde(default)]
    pub encoding:         String,
    /// Explicit field definitions; if empty, infer from header
    #[serde(default)]
    pub fields:           Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name:       String,
    #[serde(default = "default_type")]
    pub value_type: String,
}

fn default_delimiter() -> char { ',' }
fn default_true()      -> bool { true }
fn default_type()      -> String { "String".into() }

pub struct CsvFileInput {
    config: CsvFileInputConfig,
    schema: Option<Arc<RowSchema>>,
}

impl CsvFileInput {
    pub fn new(config: CsvFileInputConfig) -> Self {
        Self { config, schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: CsvFileInputConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for CsvFileInput {
    fn name(&self) -> &str { "CsvFileInput" }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        // Schema is determined at open() time after reading the header.
        // Return empty here; callers should use the schema from produce().
        Ok(RowSchema::default())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let filename = ctx.resolve(&self.config.filename);
        debug!("CsvFileInput opening '{}'", filename);

        if !self.config.fields.is_empty() {
            let fields: Vec<Field> = self.config.fields.iter().map(|f| {
                let vt = match f.value_type.as_str() {
                    "Integer"   => ValueType::Integer,
                    "Float"     => ValueType::Float,
                    "Boolean"   => ValueType::Boolean,
                    "Date"      => ValueType::Date,
                    "Timestamp" => ValueType::Timestamp,
                    _           => ValueType::String,
                };
                Field::new(f.name.clone(), vt)
            }).collect();
            self.schema = Some(Arc::new(RowSchema::new(fields)));
        }
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row]) // passthrough — this is a source
    }

    async fn close(&mut self) -> Result<()> { Ok(()) }

    fn is_source(&self) -> bool { true }

    async fn produce(&mut self, sender: mpsc::Sender<Row>) -> Result<()> {
        let filename = self.config.filename.clone();
        let delimiter = self.config.delimiter as u8;
        let header_present = self.config.header_present;

        let file = tokio::fs::File::open(&filename).await
            .map_err(|e| AjisaiError::Io(e))?;

        let std_file = file.into_std().await;

        let schema_ref = self.schema.clone();

        // CSV parsing is sync (csv crate) — run in spawn_blocking
        let result = tokio::task::spawn_blocking(move || {
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(delimiter)
                .has_headers(header_present)
                .from_reader(std_file);

            let schema = if let Some(s) = schema_ref {
                s
            } else {
                // Infer schema from header row
                let headers = rdr.headers()
                    .map_err(|e| AjisaiError::Parse(e.to_string()))?;
                let fields: Vec<Field> = headers
                    .iter()
                    .map(|h| Field::new(h.trim(), ValueType::String))
                    .collect();
                Arc::new(RowSchema::new(fields))
            };

            let rows: Vec<Row> = rdr.records()
                .map(|rec| {
                    let rec = rec.map_err(|e| AjisaiError::Parse(e.to_string()))?;
                    let values: Vec<Value> = rec.iter()
                        .enumerate()
                        .map(|(i, cell)| {
                            match schema.fields.get(i).map(|f| &f.value_type) {
                                Some(ValueType::Integer) => {
                                    cell.trim().parse::<i64>()
                                        .map(Value::Int)
                                        .unwrap_or(Value::Null)
                                }
                                Some(ValueType::Float) => {
                                    cell.trim().parse::<f64>()
                                        .map(Value::Float)
                                        .unwrap_or(Value::Null)
                                }
                                Some(ValueType::Boolean) => {
                                    match cell.trim().to_lowercase().as_str() {
                                        "true" | "1" | "yes" => Value::Bool(true),
                                        "false" | "0" | "no" => Value::Bool(false),
                                        _ => Value::Null,
                                    }
                                }
                                _ => Value::Str(cell.to_owned()),
                            }
                        })
                        .collect();
                    Ok(Row::new(schema.clone(), values))
                })
                .collect::<Result<Vec<Row>>>()?;

            Ok::<(Arc<RowSchema>, Vec<Row>), AjisaiError>((schema, rows))
        })
        .await
        .map_err(|e| AjisaiError::Pipeline(format!("spawn_blocking failed: {}", e)))??;

        let (schema, rows) = result;
        self.schema = Some(schema);

        for row in rows {
            sender.send(row).await
                .map_err(|_| AjisaiError::Pipeline("Downstream channel closed".into()))?;
        }

        Ok(())
    }
}
