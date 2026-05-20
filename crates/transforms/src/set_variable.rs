use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableSpec {
    pub variable_name: String,
    pub field_name: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetVariableConfig {
    pub variables: Vec<VariableSpec>,
}

pub struct SetVariable {
    config: SetVariableConfig,
    ctx: ExecutionContext,
}

impl SetVariable {
    pub fn new(config: SetVariableConfig) -> Self {
        Self { config, ctx: ExecutionContext::new() }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: SetVariableConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for SetVariable {
    fn name(&self) -> &str {
        "SetVariable"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.ctx = ctx.clone();
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        for spec in &self.config.variables {
            let val = row
                .get(&spec.field_name)
                .map(|v| v.to_display_string())
                .or_else(|| spec.default_value.clone())
                .unwrap_or_default();
            self.ctx.set_var(&spec.variable_name, val);
        }
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
    async fn sets_variable_from_field() {
        let mut t = SetVariable::new(SetVariableConfig {
            variables: vec![VariableSpec {
                variable_name: "MY_VAR".into(),
                field_name: "val".into(),
                default_value: None,
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let schema = Arc::new(RowSchema::new(vec![Field::new("val", ValueType::String)]));
        let row = Row::new(schema, vec![Value::Str("hello".into())]);
        let out = t.process(row).await.unwrap();

        assert_eq!(out.len(), 1);
        assert_eq!(t.ctx.get_var("MY_VAR"), Some("hello"));
    }
}
