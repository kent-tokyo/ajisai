use ajisai_core::{
    Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
};
use async_trait::async_trait;

pub struct AppendStreams;

impl AppendStreams {
    pub fn from_json(_value: serde_json::Value) -> Result<Box<dyn Transform>> {
        Ok(Box::new(AppendStreams))
    }
}

#[async_trait]
impl Transform for AppendStreams {
    fn name(&self) -> &str {
        "AppendStreams"
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
    async fn passes_rows_through() {
        let mut t = AppendStreams;
        t.open(&ExecutionContext::new()).await.unwrap();
        let schema = Arc::new(RowSchema::new(vec![Field::new("x", ValueType::Integer)]));
        let row = Row::new(schema, vec![Value::Int(42)]);
        let out = t.process(row.clone()).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("x"), Some(&Value::Int(42)));
    }
}
