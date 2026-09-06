use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteToLogConfig {
    #[serde(default = "default_level")]
    pub log_level: String,
    #[serde(default)]
    pub log_label: String,
    pub fields: Option<Vec<String>>,
    #[serde(default)]
    pub print_header: bool,
}

fn default_level() -> String {
    "info".into()
}

pub struct WriteToLog {
    config: WriteToLogConfig,
    header_printed: bool,
}

impl WriteToLog {
    pub fn new(config: WriteToLogConfig) -> Self {
        Self {
            config,
            header_printed: false,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: WriteToLogConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for WriteToLog {
    fn name(&self) -> &str {
        "WriteToLog"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let label = &self.config.log_label;

        let field_names: Vec<&str> = match &self.config.fields {
            Some(fs) => fs.iter().map(|s| s.as_str()).collect(),
            None => row.schema.fields.iter().map(|f| f.name.as_str()).collect(),
        };

        if self.config.print_header && !self.header_printed {
            let header = field_names.join(", ");
            log_at(&self.config.log_level, label, &header);
            self.header_printed = true;
        }

        let values: Vec<String> = field_names
            .iter()
            .map(|name| {
                row.get(name)
                    .map(|v| v.to_display_string())
                    .unwrap_or_default()
            })
            .collect();
        log_at(&self.config.log_level, label, &values.join(", "));

        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

fn log_at(level: &str, label: &str, msg: &str) {
    let prefix = if label.is_empty() {
        msg.to_owned()
    } else {
        format!("[{}] {}", label, msg)
    };
    match level {
        "debug" => tracing::debug!("{}", prefix),
        "warn" => tracing::warn!("{}", prefix),
        "error" => tracing::error!("{}", prefix),
        _ => tracing::info!("{}", prefix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};
    use std::sync::Arc;

    #[tokio::test]
    async fn passes_rows_through() {
        let mut t = WriteToLog::new(WriteToLogConfig {
            log_level: "info".into(),
            log_label: "test".into(),
            fields: None,
            print_header: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let schema = Arc::new(RowSchema::new(vec![Field::new("name", ValueType::String)]));
        let row = Row::new(schema, vec![Value::Str("Alice".into())]);
        let out = t.process(row).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("name"), Some(&Value::Str("Alice".into())));
    }
}
