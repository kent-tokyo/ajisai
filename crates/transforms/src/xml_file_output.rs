use crate::utils::resolve_safe_path;
use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use serde::{Deserialize, Serialize};
use std::io::BufWriter;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlFileOutputConfig {
    pub filename: String,
    #[serde(default = "default_root")]
    pub root_element: String,
    #[serde(default = "default_row")]
    pub row_element: String,
    #[serde(default)]
    pub encoding: String,
}

fn default_root() -> String {
    "rows".into()
}
fn default_row() -> String {
    "row".into()
}

pub struct XmlFileOutput {
    config: XmlFileOutputConfig,
    rows: Vec<Row>,
}

impl XmlFileOutput {
    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let mut config: XmlFileOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        if config.encoding.is_empty() {
            config.encoding = "UTF-8".into();
        }
        Ok(Box::new(Self { config, rows: Vec::new() }))
    }
}

#[async_trait]
impl Transform for XmlFileOutput {
    fn name(&self) -> &str {
        "XmlFileOutput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.rows.clear();
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        self.rows.push(row);
        Ok(Vec::new())
    }

    async fn flush(&mut self) -> Result<Vec<Row>> {
        let path = resolve_safe_path(&self.config.filename)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AjisaiError::Io)?;
        }

        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut writer = Writer::new_with_indent(BufWriter::new(cursor), b' ', 2);

            writer
                .write_event(Event::Decl(BytesDecl::new("1.0", Some(&self.config.encoding), None)))
                .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;

            let root_start = BytesStart::new(&self.config.root_element);
            writer
                .write_event(Event::Start(root_start))
                .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;

            for row in &self.rows {
                let row_start = BytesStart::new(&self.config.row_element);
                writer
                    .write_event(Event::Start(row_start))
                    .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;

                for (field, value) in row.schema.fields.iter().zip(row.values.iter()) {
                    let field_start = BytesStart::new(field.name.as_str());
                    writer
                        .write_event(Event::Start(field_start))
                        .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;

                    let text = match value {
                        Value::Null => String::new(),
                        Value::Bool(b) => b.to_string(),
                        Value::Int(i) => i.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Str(s) => s.clone(),
                        Value::Date(d) => d.to_string(),
                        Value::Timestamp(ts) => ts.to_string(),
                        Value::Bytes(b) => base64_encode(b),
                    };
                    writer
                        .write_event(Event::Text(BytesText::new(&text)))
                        .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;

                    writer
                        .write_event(Event::End(BytesEnd::new(field.name.as_str())))
                        .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;
                }

                writer
                    .write_event(Event::End(BytesEnd::new(&self.config.row_element)))
                    .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;
            }

            writer
                .write_event(Event::End(BytesEnd::new(&self.config.root_element)))
                .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;
        }

        let mut file = File::create(&path).await.map_err(AjisaiError::Io)?;
        file.write_all(&buf).await.map_err(AjisaiError::Io)?;

        Ok(Vec::new())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let _ = write!(out, "{}", TABLE[b0 >> 2] as char);
        let _ = write!(out, "{}", TABLE[((b0 & 3) << 4) | (b1 >> 4)] as char);
        let _ = write!(out, "{}", if chunk.len() > 1 { TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char } else { '=' });
        let _ = write!(out, "{}", if chunk.len() > 2 { TABLE[b2 & 0x3f] as char } else { '=' });
    }
    out
}
