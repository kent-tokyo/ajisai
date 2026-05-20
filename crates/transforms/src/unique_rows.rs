use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// UniqueRows is a streaming deduplication transform that assumes the input is
/// already sorted on the key fields. It emits a row only when the key changes.
/// For unsorted input, use `Deduplicate` instead (which buffers everything).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueRowsConfig {
    /// Key fields to compare. All fields are used if empty.
    #[serde(default)]
    pub key_fields: Vec<String>,
}

pub struct UniqueRows {
    config: UniqueRowsConfig,
    last_key: Option<String>,
}

impl UniqueRows {
    pub fn new(config: UniqueRowsConfig) -> Self {
        Self { config, last_key: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: UniqueRowsConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn row_key(&self, row: &Row) -> String {
        if self.config.key_fields.is_empty() {
            row.values
                .iter()
                .map(|v| v.to_display_string())
                .collect::<Vec<_>>()
                .join("\x00")
        } else {
            self.config
                .key_fields
                .iter()
                .map(|f| row.get(f).map(|v| v.to_display_string()).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\x00")
        }
    }
}

#[async_trait]
impl Transform for UniqueRows {
    fn name(&self) -> &str {
        "UniqueRows"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.last_key = None;
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let key = self.row_key(&row);
        if self.last_key.as_deref() == Some(&key) {
            Ok(vec![])
        } else {
            self.last_key = Some(key);
            Ok(vec![row])
        }
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};
    use std::sync::Arc;

    fn make_row(name: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("name", ValueType::String)]));
        Row::new(schema, vec![Value::Str(name.into())])
    }

    #[tokio::test]
    async fn deduplicates_consecutive_duplicates() {
        let mut t = UniqueRows::new(UniqueRowsConfig { key_fields: vec![] });
        t.open(&ExecutionContext::new()).await.unwrap();

        assert_eq!(t.process(make_row("Alice")).await.unwrap().len(), 1);
        assert_eq!(t.process(make_row("Alice")).await.unwrap().len(), 0);
        assert_eq!(t.process(make_row("Bob")).await.unwrap().len(), 1);
        assert_eq!(t.process(make_row("Alice")).await.unwrap().len(), 1); // non-consecutive, emitted
    }

    #[tokio::test]
    async fn passes_all_when_no_duplicates() {
        let mut t = UniqueRows::new(UniqueRowsConfig { key_fields: vec![] });
        t.open(&ExecutionContext::new()).await.unwrap();

        assert_eq!(t.process(make_row("A")).await.unwrap().len(), 1);
        assert_eq!(t.process(make_row("B")).await.unwrap().len(), 1);
        assert_eq!(t.process(make_row("C")).await.unwrap().len(), 1);
    }
}
