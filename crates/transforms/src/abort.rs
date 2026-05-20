use crate::filter::Condition;
use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Abort halts the pipeline with an error when a row matches the condition.
/// Rows that do NOT match the condition are forwarded unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortConfig {
    /// Condition that triggers the abort. If omitted, every row triggers abort.
    pub condition: Option<Condition>,
    /// Error message included in the pipeline error
    #[serde(default = "default_message")]
    pub message: String,
}

fn default_message() -> String {
    "Abort: condition met".to_owned()
}

pub struct Abort {
    config: AbortConfig,
}

impl Abort {
    pub fn new(config: AbortConfig) -> Self {
        Self { config }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: AbortConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for Abort {
    fn name(&self) -> &str {
        "Abort"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let triggered = match &self.config.condition {
            Some(cond) => cond.evaluate(&row),
            None => true,
        };
        if triggered {
            Err(AjisaiError::Pipeline(self.config.message.clone()))
        } else {
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

    fn make_row(status: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("status", ValueType::String)]));
        Row::new(schema, vec![Value::Str(status.into())])
    }

    #[tokio::test]
    async fn aborts_on_matching_row() {
        let mut t = Abort::new(AbortConfig {
            condition: Some(Condition::Eq {
                field: "status".into(),
                value: "ERROR".into(),
            }),
            message: "got error row".into(),
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        assert!(t.process(make_row("OK")).await.unwrap().len() == 1);
        let err = t.process(make_row("ERROR")).await.unwrap_err();
        assert!(err.to_string().contains("got error row"));
    }

    #[tokio::test]
    async fn aborts_unconditionally_when_no_condition() {
        let mut t = Abort::new(AbortConfig { condition: None, message: "always".into() });
        t.open(&ExecutionContext::new()).await.unwrap();
        assert!(t.process(make_row("anything")).await.is_err());
    }
}
