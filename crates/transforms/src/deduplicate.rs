use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicateConfig {
    /// Fields that form the deduplication key. All fields if empty.
    #[serde(default)]
    pub key_fields: Vec<String>,
}

/// Deduplicate removes rows with duplicate key values, keeping the first occurrence.
/// Internally maintains a HashSet of seen keys (in-memory).
pub struct Deduplicate {
    config: DeduplicateConfig,
    seen: HashSet<String>,
}

impl Deduplicate {
    pub fn new(config: DeduplicateConfig) -> Self {
        Self {
            config,
            seen: HashSet::new(),
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: DeduplicateConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn row_key(&self, row: &Row) -> String {
        if self.config.key_fields.is_empty() {
            // All fields form the key
            row.values
                .iter()
                .map(|v| v.to_display_string())
                .collect::<Vec<_>>()
                .join("\x00")
        } else {
            self.config
                .key_fields
                .iter()
                .map(|f| {
                    row.get(f)
                        .map(|v| v.to_display_string())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join("\x00")
        }
    }
}

#[async_trait]
impl Transform for Deduplicate {
    fn name(&self) -> &str {
        "Deduplicate"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.seen.clear();
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let key = self.row_key(&row);
        if self.seen.insert(key) {
            Ok(vec![row]) // first occurrence — pass through
        } else {
            Ok(vec![]) // duplicate — drop
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

    fn make_row(name: &str, age: i64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("name", ValueType::String),
            Field::new("age", ValueType::Integer),
        ]));
        Row::new(schema, vec![Value::Str(name.into()), Value::Int(age)])
    }

    #[tokio::test]
    async fn deduplicates_by_all_fields() {
        let config = DeduplicateConfig { key_fields: vec![] };
        let mut dedup = Deduplicate::new(config);
        let ctx = ExecutionContext::new();
        dedup.open(&ctx).await.unwrap();

        let r1 = make_row("Alice", 30);
        let r2 = make_row("Alice", 30); // duplicate
        let r3 = make_row("Bob", 25);

        assert_eq!(dedup.process(r1).await.unwrap().len(), 1);
        assert_eq!(dedup.process(r2).await.unwrap().len(), 0); // dropped
        assert_eq!(dedup.process(r3).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn deduplicates_by_key_field() {
        let config = DeduplicateConfig {
            key_fields: vec!["name".into()],
        };
        let mut dedup = Deduplicate::new(config);
        let ctx = ExecutionContext::new();
        dedup.open(&ctx).await.unwrap();

        let r1 = make_row("Alice", 30);
        let r2 = make_row("Alice", 99); // same name, different age → still duplicate by key
        let r3 = make_row("Bob", 25);

        assert_eq!(dedup.process(r1).await.unwrap().len(), 1);
        assert_eq!(dedup.process(r2).await.unwrap().len(), 0);
        assert_eq!(dedup.process(r3).await.unwrap().len(), 1);
    }
}
