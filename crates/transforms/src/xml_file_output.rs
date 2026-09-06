use crate::utils::{resolve_context_path, resolve_safe_path};
use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
};
use async_trait::async_trait;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use serde::{Deserialize, Serialize};
use std::io::BufWriter;
use std::path::PathBuf;
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
    temporary_path: Option<PathBuf>,
}

impl XmlFileOutput {
    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let mut config: XmlFileOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        if config.encoding.is_empty() {
            config.encoding = "UTF-8".into();
        }
        Ok(Box::new(Self {
            config,
            rows: Vec::new(),
            temporary_path: None,
        }))
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

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.rows.clear();
        let resolved = ctx.resolve(&self.config.filename);
        let safe = resolve_context_path(ctx, &resolved)?;
        self.config.filename = safe.display().to_string();
        self.temporary_path = None;
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
                .write_event(Event::Decl(BytesDecl::new(
                    "1.0",
                    Some(&self.config.encoding),
                    None,
                )))
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

        let temporary = PathBuf::from(format!(
            "{}.ajisai-tmp-{}",
            path.display(),
            std::process::id()
        ));
        self.temporary_path = Some(temporary.clone());
        let mut file = File::create(&temporary).await.map_err(AjisaiError::Io)?;
        file.write_all(&buf).await.map_err(AjisaiError::Io)?;
        file.flush().await.map_err(AjisaiError::Io)?;
        drop(file);
        tokio::fs::rename(&temporary, &path)
            .await
            .map_err(AjisaiError::Io)?;
        self.temporary_path = None;

        Ok(Vec::new())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Drop for XmlFileOutput {
    fn drop(&mut self) {
        if let Some(path) = self.temporary_path.take() {
            let _ = std::fs::remove_file(path);
        }
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
        let _ = write!(
            out,
            "{}",
            if chunk.len() > 1 {
                TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char
            } else {
                '='
            }
        );
        let _ = write!(
            out,
            "{}",
            if chunk.len() > 2 {
                TABLE[b2 & 0x3f] as char
            } else {
                '='
            }
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, ValueType};
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn writes_xml_via_atomic_temporary_file() {
        let target = NamedTempFile::new().unwrap();
        let path = target.path().display().to_string();
        let mut output = XmlFileOutput {
            config: XmlFileOutputConfig {
                filename: path.clone(),
                root_element: "rows".into(),
                row_element: "row".into(),
                encoding: "UTF-8".into(),
            },
            rows: Vec::new(),
            temporary_path: None,
        };
        let schema = Arc::new(RowSchema::new(vec![Field::new("id", ValueType::Integer)]));
        output.open(&ExecutionContext::new()).await.unwrap();
        output
            .process(Row::new(schema, vec![Value::Int(1)]))
            .await
            .unwrap();
        output.flush().await.unwrap();
        assert!(
            std::fs::read_to_string(path)
                .unwrap()
                .contains("<id>1</id>")
        );
    }
}
