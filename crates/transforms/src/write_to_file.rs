use crate::utils::resolve_safe_path;
use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write as IoWrite};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteToFileConfig {
    /// Output file path
    pub filename: String,
    /// Field whose value to write on each row
    pub field: String,
    /// Append to existing file instead of overwriting
    #[serde(default)]
    pub append: bool,
    /// Write a newline after each value
    #[serde(default = "default_true")]
    pub newline: bool,
}

fn default_true() -> bool {
    true
}

pub struct WriteToFile {
    config: WriteToFileConfig,
    writer: Option<Mutex<BufWriter<std::fs::File>>>,
}

impl WriteToFile {
    pub fn new(config: WriteToFileConfig) -> Self {
        Self { config, writer: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: WriteToFileConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for WriteToFile {
    fn name(&self) -> &str {
        "WriteToFile"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let filename = ctx.resolve(&self.config.filename);
        let safe_path = resolve_safe_path(&filename)?;

        let file = if self.config.append {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&safe_path)
        } else {
            std::fs::File::create(&safe_path).map(|f| f)
        }
        .map_err(AjisaiError::Io)?;

        self.writer = Some(Mutex::new(BufWriter::new(file)));
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let writer_mutex = self
            .writer
            .as_ref()
            .ok_or_else(|| AjisaiError::Pipeline("WriteToFile not opened".into()))?;

        let mut w = writer_mutex
            .lock()
            .map_err(|_| AjisaiError::Pipeline("Writer lock poisoned".into()))?;

        let value = row
            .get(&self.config.field)
            .map(|v| v.to_display_string())
            .unwrap_or_default();

        if self.config.newline {
            writeln!(w, "{}", value).map_err(AjisaiError::Io)?;
        } else {
            write!(w, "{}", value).map_err(AjisaiError::Io)?;
        }

        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(mutex) = self.writer.take() {
            let mut w = mutex.into_inner().unwrap();
            w.flush().map_err(AjisaiError::Io)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn make_row(msg: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("msg", ValueType::String)]));
        Row::new(schema, vec![Value::Str(msg.into())])
    }

    #[tokio::test]
    async fn writes_field_values_to_file() {
        let f = NamedTempFile::new().unwrap();
        let path = f.path().to_str().unwrap().to_owned();

        let mut t = WriteToFile::new(WriteToFileConfig {
            filename: path.clone(),
            field: "msg".into(),
            append: false,
            newline: true,
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        t.process(make_row("hello")).await.unwrap();
        t.process(make_row("world")).await.unwrap();
        t.close().await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "hello\nworld\n");
    }

    #[tokio::test]
    async fn appends_to_existing_file() {
        let f = NamedTempFile::new().unwrap();
        std::fs::write(f.path(), "existing\n").unwrap();
        let path = f.path().to_str().unwrap().to_owned();

        let mut t = WriteToFile::new(WriteToFileConfig {
            filename: path.clone(),
            field: "msg".into(),
            append: true,
            newline: true,
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        t.process(make_row("new")).await.unwrap();
        t.close().await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "existing\nnew\n");
    }
}
