use ajisai_core::{
    context::ExecutionContext, error::Result, value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvFileOutputConfig {
    pub filename:          String,
    #[serde(default = "default_delimiter")]
    pub delimiter:         char,
    #[serde(default = "default_true")]
    pub header_present:    bool,
    #[serde(default)]
    pub append:            bool,
}

fn default_delimiter() -> char { ',' }
fn default_true()      -> bool { true }

pub struct CsvFileOutput {
    config:  CsvFileOutputConfig,
    writer:  Option<Mutex<csv::Writer<std::fs::File>>>,
    headers_written: bool,
}

impl CsvFileOutput {
    pub fn new(config: CsvFileOutputConfig) -> Self {
        Self { config, writer: None, headers_written: false }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: CsvFileOutputConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for CsvFileOutput {
    fn name(&self) -> &str { "CsvFileOutput" }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let filename = ctx.resolve(&self.config.filename);
        debug!("CsvFileOutput opening '{}'", filename);

        let file = if self.config.append {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filename)
        } else {
            std::fs::File::create(&filename).map(|f| f)
        }
        .map_err(AjisaiError::Io)?;

        let writer = csv::WriterBuilder::new()
            .delimiter(self.config.delimiter as u8)
            .from_writer(file);

        self.writer = Some(Mutex::new(writer));
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let writer_mutex = self.writer.as_ref()
            .ok_or_else(|| AjisaiError::Pipeline("CsvFileOutput not opened".into()))?;

        let mut writer = writer_mutex.lock().unwrap();

        if self.config.header_present && !self.headers_written {
            let headers: Vec<&str> = row.schema.fields.iter().map(|f| f.name.as_str()).collect();
            writer.write_record(&headers)
                .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;
            self.headers_written = true;
        }

        let record: Vec<String> = row.values.iter().map(|v| v.to_display_string()).collect();
        writer.write_record(&record)
            .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;

        Ok(vec![row]) // pass through for chaining
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(writer_mutex) = self.writer.take() {
            let mut writer = writer_mutex.into_inner().unwrap();
            writer.flush().map_err(AjisaiError::Io)?;
        }
        Ok(())
    }
}
