use crate::utils::coerce;
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
pub struct GetVariableSpec {
    pub field_name: String,
    /// Variable name, optionally using ${VAR} syntax
    pub variable: String,
    pub field_type: ValueType,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableConfig {
    pub variables: Vec<GetVariableSpec>,
}

pub struct GetVariable {
    config: GetVariableConfig,
    /// Resolved values cached at open() time
    cached: Vec<Value>,
    output_schema: Option<Arc<RowSchema>>,
}

impl GetVariable {
    pub fn new(config: GetVariableConfig) -> Self {
        Self { config, cached: Vec::new(), output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: GetVariableConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

}

#[async_trait]
impl Transform for GetVariable {
    fn name(&self) -> &str {
        "GetVariable"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        for spec in &self.config.variables {
            fields.push(Field::new(&spec.field_name, spec.field_type.clone()));
        }
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.cached = self
            .config
            .variables
            .iter()
            .map(|spec| {
                let resolved = ctx.resolve(&spec.variable);
                // If ${VAR} was not resolved (variable not found), use the actual env var name
                let raw = if resolved == spec.variable {
                    spec.default_value.as_deref().unwrap_or("").to_owned()
                } else {
                    resolved
                };
                coerce(&raw, &spec.field_type)
            })
            .collect();
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = self.output_schema.get_or_insert_with(|| {
            let mut fields = row.schema.fields.clone();
            for spec in &self.config.variables {
                fields.push(Field::new(&spec.field_name, spec.field_type.clone()));
            }
            Arc::new(RowSchema::new(fields))
        });

        let mut values = row.values.clone();
        values.extend(self.cached.clone());
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

    #[tokio::test]
    async fn adds_variable_as_field() {
        let mut ctx = ExecutionContext::new();
        ctx.set_var("ENV", "production");

        let mut t = GetVariable::new(GetVariableConfig {
            variables: vec![GetVariableSpec {
                field_name: "env".into(),
                variable: "${ENV}".into(),
                field_type: ValueType::String,
                default_value: None,
            }],
        });
        t.open(&ctx).await.unwrap();

        let schema = Arc::new(RowSchema::new(vec![Field::new("id", ValueType::Integer)]));
        let row = Row::new(schema, vec![Value::Int(1)]);
        let out = t.process(row).await.unwrap();

        assert_eq!(out[0].get("env"), Some(&Value::Str("production".into())));
    }

    #[tokio::test]
    async fn uses_default_when_missing() {
        let mut t = GetVariable::new(GetVariableConfig {
            variables: vec![GetVariableSpec {
                field_name: "env".into(),
                variable: "${MISSING}".into(),
                field_type: ValueType::String,
                default_value: Some("dev".into()),
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let schema = Arc::new(RowSchema::new(vec![Field::new("id", ValueType::Integer)]));
        let row = Row::new(schema, vec![Value::Int(1)]);
        let out = t.process(row).await.unwrap();

        assert_eq!(out[0].get("env"), Some(&Value::Str("dev".into())));
    }
}
