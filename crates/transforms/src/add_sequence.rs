use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSequenceConfig {
    pub field_name: String,
    #[serde(default = "default_one")]
    pub start: i64,
    #[serde(default = "default_one")]
    pub increment: i64,
}

fn default_one() -> i64 {
    1
}

pub struct AddSequence {
    config: AddSequenceConfig,
    counter: i64,
    output_schema: Option<Arc<RowSchema>>,
}

impl AddSequence {
    pub fn new(config: AddSequenceConfig) -> Self {
        let counter = config.start - config.increment;
        Self {
            config,
            counter,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: AddSequenceConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for AddSequence {
    fn name(&self) -> &str {
        "AddSequence"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        fields.push(Field::new(&self.config.field_name, ValueType::Integer));
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.counter = self.config.start - self.config.increment;
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        self.counter += self.config.increment;

        let schema = self.output_schema.get_or_insert_with(|| {
            let mut fields = row.schema.fields.clone();
            fields.push(Field::new(&self.config.field_name, ValueType::Integer));
            Arc::new(RowSchema::new(fields))
        });

        let mut values = row.values.clone();
        values.push(Value::Int(self.counter));
        Ok(vec![Row::new(schema.clone(), values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, ValueType};

    fn make_row(name: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("name", ValueType::String)]));
        Row::new(schema, vec![Value::Str(name.into())])
    }

    #[tokio::test]
    async fn adds_sequence_field() {
        let mut t = AddSequence::new(AddSequenceConfig {
            field_name: "id".into(),
            start: 1,
            increment: 1,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let r1 = t.process(make_row("a")).await.unwrap();
        let r2 = t.process(make_row("b")).await.unwrap();
        let r3 = t.process(make_row("c")).await.unwrap();

        assert_eq!(r1[0].get("id"), Some(&Value::Int(1)));
        assert_eq!(r2[0].get("id"), Some(&Value::Int(2)));
        assert_eq!(r3[0].get("id"), Some(&Value::Int(3)));
    }

    #[tokio::test]
    async fn custom_start_and_increment() {
        let mut t = AddSequence::new(AddSequenceConfig {
            field_name: "seq".into(),
            start: 10,
            increment: 5,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let r1 = t.process(make_row("a")).await.unwrap();
        let r2 = t.process(make_row("b")).await.unwrap();

        assert_eq!(r1[0].get("seq"), Some(&Value::Int(10)));
        assert_eq!(r2[0].get("seq"), Some(&Value::Int(15)));
    }
}
