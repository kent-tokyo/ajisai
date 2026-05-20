use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneRowConfig {
    /// Number of times to clone each row (total output = input * clone_count)
    #[serde(default = "default_clone_count")]
    pub clone_count: usize,
}

fn default_clone_count() -> usize {
    1
}

pub struct CloneRow {
    config: CloneRowConfig,
}

impl CloneRow {
    pub fn new(config: CloneRowConfig) -> Self {
        Self { config }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: CloneRowConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for CloneRow {
    fn name(&self) -> &str {
        "CloneRow"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let n = self.config.clone_count.max(1);
        Ok(std::iter::repeat_with(|| row.clone()).take(n).collect())
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

    fn make_row(id: i64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("id", ValueType::Integer)]));
        Row::new(schema, vec![Value::Int(id)])
    }

    #[tokio::test]
    async fn clones_row_n_times() {
        let mut t = CloneRow::new(CloneRowConfig { clone_count: 3 });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(make_row(1)).await.unwrap();
        assert_eq!(out.len(), 3);
        for r in &out {
            assert_eq!(r.get("id"), Some(&Value::Int(1)));
        }
    }

    #[tokio::test]
    async fn clone_count_zero_emits_one() {
        let mut t = CloneRow::new(CloneRowConfig { clone_count: 0 });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(make_row(1)).await.unwrap();
        assert_eq!(out.len(), 1);
    }
}
