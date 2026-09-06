use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvFileOutputConfig {
    pub filename: String,
    #[serde(default = "default_delimiter")]
    pub delimiter: char,
    #[serde(default = "default_true")]
    pub header_present: bool,
    #[serde(default)]
    pub append: bool,
}

fn default_delimiter() -> char {
    ','
}
fn default_true() -> bool {
    true
}

pub struct CsvFileOutput {
    config: CsvFileOutputConfig,
    writer: Option<Mutex<csv::Writer<std::fs::File>>>,
    headers_written: bool,
    target_path: Option<std::path::PathBuf>,
    temporary_path: Option<std::path::PathBuf>,
}

impl CsvFileOutput {
    pub fn new(config: CsvFileOutputConfig) -> Self {
        Self {
            config,
            writer: None,
            headers_written: false,
            target_path: None,
            temporary_path: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: CsvFileOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

impl Drop for CsvFileOutput {
    fn drop(&mut self) {
        if let Some(temp_path) = self.temporary_path.take() {
            let _ = std::fs::remove_file(temp_path);
        }
    }
}

#[async_trait]
impl Transform for CsvFileOutput {
    fn name(&self) -> &str {
        "CsvFileOutput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
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
        debug!("CsvFileOutput opening '{}'", filename);

        // Reject path traversal attempts
        if std::path::Path::new(&filename)
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(AjisaiError::Config("Path traversal not allowed".into()));
        }

        let target_path = std::path::PathBuf::from(&filename);
        let temporary_path = if self.config.append {
            None
        } else {
            Some(std::path::PathBuf::from(format!(
                "{}.ajisai-tmp-{}",
                filename,
                std::process::id()
            )))
        };
        let write_path = temporary_path.as_ref().unwrap_or(&target_path);
        let file = if self.config.append {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(write_path)
        } else {
            std::fs::File::create(write_path)
        }
        .map_err(AjisaiError::Io)?;

        let writer = csv::WriterBuilder::new()
            .delimiter(self.config.delimiter as u8)
            .from_writer(file);

        self.writer = Some(Mutex::new(writer));
        self.target_path = Some(target_path);
        self.temporary_path = temporary_path;
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let writer_mutex = self
            .writer
            .as_ref()
            .ok_or_else(|| AjisaiError::Pipeline("CsvFileOutput not opened".into()))?;

        let mut writer = writer_mutex
            .lock()
            .map_err(|_| AjisaiError::Pipeline("Writer lock poisoned".into()))?;

        if self.config.header_present && !self.headers_written {
            let headers: Vec<&str> = row.schema.fields.iter().map(|f| f.name.as_str()).collect();
            writer
                .write_record(&headers)
                .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;
            self.headers_written = true;
        }

        let record: Vec<String> = row.values.iter().map(|v| v.to_display_string()).collect();
        writer
            .write_record(&record)
            .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;

        Ok(vec![row]) // pass through for chaining
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(writer_mutex) = self.writer.take() {
            let mut writer = writer_mutex.into_inner().unwrap();
            writer.flush().map_err(AjisaiError::Io)?;
        }
        if let Some(temp_path) = self.temporary_path.take() {
            let target_path = self
                .target_path
                .take()
                .ok_or_else(|| AjisaiError::Pipeline("CSV output target missing".into()))?;
            std::fs::rename(temp_path, target_path).map_err(AjisaiError::Io)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, Value, ValueType};
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn row() -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("value", ValueType::String)]));
        Row::new(schema, vec![Value::Str("pending".into())])
    }

    #[tokio::test]
    async fn drop_removes_uncommitted_temporary_file() {
        let target = NamedTempFile::new().unwrap();
        let path = target.path().display().to_string();
        let temporary = format!("{path}.ajisai-tmp-{}", std::process::id());
        let mut output = CsvFileOutput::new(CsvFileOutputConfig {
            filename: path,
            delimiter: ',',
            header_present: true,
            append: false,
        });
        output.open(&ExecutionContext::new()).await.unwrap();
        output.process(row()).await.unwrap();
        assert!(std::path::Path::new(&temporary).exists());
        drop(output);
        assert!(!std::path::Path::new(&temporary).exists());
    }
}
