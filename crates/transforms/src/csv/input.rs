use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvFileInputConfig {
    pub filename: String,
    #[serde(default = "default_delimiter")]
    pub delimiter: char,
    #[serde(default = "default_true", alias = "has_header")]
    pub header_present: bool,
    #[serde(default)]
    pub encoding: String,
    /// Explicit field definitions; if empty, infer from header
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    #[serde(default = "default_type")]
    pub value_type: String,
}

fn default_delimiter() -> char {
    ','
}
fn default_true() -> bool {
    true
}
fn default_type() -> String {
    "String".into()
}

pub struct CsvFileInput {
    config: CsvFileInputConfig,
    schema: Option<Arc<RowSchema>>,
    resolved_filename: Option<String>,
}

impl CsvFileInput {
    pub fn new(config: CsvFileInputConfig) -> Self {
        Self {
            config,
            schema: None,
            resolved_filename: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: CsvFileInputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for CsvFileInput {
    fn name(&self) -> &str {
        "CsvFileInput"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        // Schema is determined at open() time after reading the header.
        // Return empty here; callers should use the schema from produce().
        Ok(RowSchema::default())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let filename = ctx.resolve(&self.config.filename);
        let filename = if let Some(root) = ctx.project_root() {
            crate::utils::resolve_path_in_root(root, &filename)?
                .to_string_lossy()
                .into_owned()
        } else {
            filename
        };
        debug!("CsvFileInput opening '{}'", filename);

        if !self.config.encoding.is_empty()
            && !self.config.encoding.eq_ignore_ascii_case("utf-8")
            && !self.config.encoding.eq_ignore_ascii_case("utf8")
        {
            return Err(AjisaiError::Config(format!(
                "Unsupported CSV encoding '{}'; only UTF-8 is supported",
                self.config.encoding
            )));
        }

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
                        "Date" => ValueType::Date,
                        "Timestamp" => ValueType::Timestamp,
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
        Ok(vec![row]) // passthrough — this is a source
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
        let delimiter = self.config.delimiter as u8;
        let header_present = self.config.header_present;

        let file = tokio::fs::File::open(&filename)
            .await
            .map_err(AjisaiError::Io)?;

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
                let headers = rdr
                    .headers()
                    .map_err(|e| AjisaiError::Parse(e.to_string()))?;
                let fields: Vec<Field> = headers
                    .iter()
                    .map(|h| Field::new(h.trim(), ValueType::String))
                    .collect();
                Arc::new(RowSchema::new(fields))
            };

            for rec in rdr.records() {
                let rec = rec.map_err(|e| AjisaiError::Parse(e.to_string()))?;
                let values: Vec<Value> = rec
                    .iter()
                    .enumerate()
                    .map(
                        |(i, cell)| match schema.fields.get(i).map(|f| &f.value_type) {
                            Some(ValueType::Integer) => cell
                                .trim()
                                .parse::<i64>()
                                .map(Value::Int)
                                .unwrap_or(Value::Null),
                            Some(ValueType::Float) => cell
                                .trim()
                                .parse::<f64>()
                                .map(Value::Float)
                                .unwrap_or(Value::Null),
                            Some(ValueType::Boolean) => match cell.trim().to_lowercase().as_str() {
                                "true" | "1" | "yes" => Value::Bool(true),
                                "false" | "0" | "no" => Value::Bool(false),
                                _ => Value::Null,
                            },
                            _ => Value::Str(cell.to_owned()),
                        },
                    )
                    .collect();
                sender
                    .blocking_send(Row::new(schema.clone(), values))
                    .map_err(|_| AjisaiError::Pipeline("Downstream channel closed".into()))?;
            }

            Ok::<Arc<RowSchema>, AjisaiError>(schema)
        })
        .await
        .map_err(|e| AjisaiError::Pipeline(format!("spawn_blocking failed: {}", e)))??;

        self.schema = Some(result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn config(path: &std::path::Path) -> CsvFileInputConfig {
        CsvFileInputConfig {
            filename: path.display().to_string(),
            delimiter: ',',
            header_present: true,
            encoding: String::new(),
            fields: vec![
                FieldDef {
                    name: "name".into(),
                    value_type: "String".into(),
                },
                FieldDef {
                    name: "age".into(),
                    value_type: "Integer".into(),
                },
                FieldDef {
                    name: "active".into(),
                    value_type: "Boolean".into(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn streams_rows_and_applies_explicit_types() {
        let file = NamedTempFile::new().unwrap();
        tokio::fs::write(
            file.path(),
            "name,age,active\nAlice,30,true\nBob,bad,nope\n",
        )
        .await
        .unwrap();
        let mut input = CsvFileInput::new(config(file.path()));
        input.open(&ExecutionContext::new()).await.unwrap();
        let (sender, mut receiver) = mpsc::channel(1);
        let producer = tokio::spawn(async move {
            input.produce(sender).await.unwrap();
            input
        });

        let first = receiver.recv().await.unwrap();
        assert_eq!(first.values[0], Value::Str("Alice".into()));
        assert_eq!(first.values[1], Value::Int(30));
        assert_eq!(first.values[2], Value::Bool(true));
        let second = receiver.recv().await.unwrap();
        assert_eq!(second.values[1], Value::Null);
        assert_eq!(second.values[2], Value::Null);
        assert!(receiver.recv().await.is_none());
        assert!(producer.await.unwrap().schema.is_some());
    }

    #[tokio::test]
    async fn reports_downstream_disconnect() {
        let file = NamedTempFile::new().unwrap();
        tokio::fs::write(file.path(), "name\nAlice\n")
            .await
            .unwrap();
        let mut input = CsvFileInput::new(CsvFileInputConfig {
            fields: vec![],
            ..config(file.path())
        });
        input.open(&ExecutionContext::new()).await.unwrap();
        let (sender, receiver) = mpsc::channel(1);
        drop(receiver);
        assert!(input.produce(sender).await.is_err());
    }

    #[tokio::test]
    async fn preserves_quoted_delimiters_and_maps_empty_values_to_null() {
        let file = NamedTempFile::new().unwrap();
        tokio::fs::write(file.path(), "name,age,active\n\"Doe, Jane\",,false\n")
            .await
            .unwrap();
        let mut input = CsvFileInput::new(config(file.path()));
        input.open(&ExecutionContext::new()).await.unwrap();
        let (sender, mut receiver) = mpsc::channel(1);
        input.produce(sender).await.unwrap();
        let row = receiver.recv().await.unwrap();
        assert_eq!(row.values[0], Value::Str("Doe, Jane".into()));
        assert_eq!(row.values[1], Value::Null);
        assert_eq!(row.values[2], Value::Bool(false));
    }

    #[tokio::test]
    async fn rejects_unsupported_encoding_before_opening_file() {
        let file = NamedTempFile::new().unwrap();
        let mut cfg = config(file.path());
        cfg.encoding = "Shift_JIS".into();
        let mut input = CsvFileInput::new(cfg);
        let error = input.open(&ExecutionContext::new()).await.unwrap_err();
        assert!(error.to_string().contains("only UTF-8 is supported"));
    }

    #[tokio::test]
    async fn project_root_scopes_relative_input() {
        let root = tempfile::tempdir().unwrap();
        tokio::fs::write(root.path().join("input.csv"), "name\nAlice\n")
            .await
            .unwrap();
        let mut input = CsvFileInput::new(CsvFileInputConfig {
            filename: "input.csv".into(),
            delimiter: ',',
            header_present: true,
            encoding: String::new(),
            fields: vec![],
        });
        let mut context = ExecutionContext::new();
        context.set_project_root(root.path());
        input.open(&context).await.unwrap();
        let (sender, mut receiver) = mpsc::channel(1);
        input.produce(sender).await.unwrap();
        assert_eq!(
            receiver.recv().await.unwrap().values[0],
            Value::Str("Alice".into())
        );
    }

    #[tokio::test]
    async fn reports_missing_columns_as_parse_error() {
        let file = NamedTempFile::new().unwrap();
        tokio::fs::write(file.path(), "name,age,active\nAlice,30\n")
            .await
            .unwrap();
        let mut input = CsvFileInput::new(config(file.path()));
        input.open(&ExecutionContext::new()).await.unwrap();
        let (sender, _receiver) = mpsc::channel(1);
        let error = input.produce(sender).await.unwrap_err();
        assert!(error.to_string().contains("record") || error.to_string().contains("field"));
    }
}
