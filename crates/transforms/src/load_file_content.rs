use crate::utils::resolve_safe_path;
use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    Utf8,
    Base64,
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Utf8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadFileContentConfig {
    /// Field containing the file path
    pub path_field: String,
    /// Output field name for the file content
    pub content_field: String,
    /// How to encode the file bytes into the content field
    #[serde(default)]
    pub encoding: Encoding,
}

pub struct LoadFileContent {
    config: LoadFileContentConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl LoadFileContent {
    pub fn new(config: LoadFileContentConfig) -> Self {
        Self {
            config,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: LoadFileContentConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for LoadFileContent {
    fn name(&self) -> &str {
        "LoadFileContent"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        fields.push(Field::new(&self.config.content_field, ValueType::String));
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = self
            .output_schema
            .get_or_insert_with(|| {
                let mut fields = row.schema.fields.clone();
                fields.push(Field::new(&self.config.content_field, ValueType::String));
                Arc::new(RowSchema::new(fields))
            })
            .clone();

        let path_raw = row
            .get(&self.config.path_field)
            .map(|v| v.to_display_string())
            .unwrap_or_default();

        let safe_path = resolve_safe_path(&path_raw)?;

        let content = match self.config.encoding {
            Encoding::Utf8 => std::fs::read_to_string(&safe_path).map_err(AjisaiError::Io)?,
            Encoding::Base64 => {
                let bytes = std::fs::read(&safe_path).map_err(AjisaiError::Io)?;
                let mut out = String::with_capacity(bytes.len() * 4 / 3 + 4);
                const CHARS: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                for chunk in bytes.chunks(3) {
                    let b0 = chunk[0];
                    let b1 = *chunk.get(1).unwrap_or(&0);
                    let b2 = *chunk.get(2).unwrap_or(&0);
                    out.push(CHARS[(b0 >> 2) as usize] as char);
                    out.push(CHARS[((b0 & 0x3) << 4 | b1 >> 4) as usize] as char);
                    if chunk.len() > 1 {
                        out.push(CHARS[((b1 & 0xf) << 2 | b2 >> 6) as usize] as char);
                    } else {
                        out.push('=');
                    }
                    if chunk.len() > 2 {
                        out.push(CHARS[(b2 & 0x3f) as usize] as char);
                    } else {
                        out.push('=');
                    }
                }
                out
            }
        };

        let mut values = row.values.clone();
        values.push(Value::Str(content));
        Ok(vec![Row::new(schema, values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};
    use std::io::Write as _;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn make_row(path: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("path", ValueType::String)]));
        Row::new(schema, vec![Value::Str(path.into())])
    }

    #[tokio::test]
    async fn reads_utf8_file() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "hello world").unwrap();

        let mut t = LoadFileContent::new(LoadFileContentConfig {
            path_field: "path".into(),
            content_field: "content".into(),
            encoding: Encoding::Utf8,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t
            .process(make_row(f.path().to_str().unwrap()))
            .await
            .unwrap();
        assert_eq!(
            out[0].get("content"),
            Some(&Value::Str("hello world".into()))
        );
    }
}
