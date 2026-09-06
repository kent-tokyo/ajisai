use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Dummy is a no-op pass-through transform — all rows are forwarded unchanged.
/// Useful as a pipeline placeholder, tee point, or testing aid.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DummyConfig {}

pub struct Dummy;

impl Dummy {
    pub fn new(_config: DummyConfig) -> Self {
        Self
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: DummyConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for Dummy {
    fn name(&self) -> &str {
        "Dummy"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row])
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

    #[tokio::test]
    async fn passes_rows_unchanged() {
        let mut t = Dummy::new(DummyConfig {});
        t.open(&ExecutionContext::new()).await.unwrap();

        let schema = Arc::new(RowSchema::new(vec![Field::new("id", ValueType::Integer)]));
        let row = Row::new(schema, vec![Value::Int(42)]);
        let out = t.process(row).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("id"), Some(&Value::Int(42)));
    }
}
