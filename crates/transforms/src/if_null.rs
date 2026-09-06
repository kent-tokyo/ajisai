use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value, ValueType},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfNullReplacement {
    pub field: String,
    pub default_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfNullConfig {
    pub replacements: Vec<IfNullReplacement>,
}

pub struct IfNull {
    config: IfNullConfig,
}

impl IfNull {
    pub fn new(config: IfNullConfig) -> Self {
        Self { config }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: IfNullConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn coerce(s: &str, vt: &ValueType) -> Value {
        match vt {
            ValueType::Integer => s
                .parse::<i64>()
                .map(Value::Int)
                .unwrap_or_else(|_| Value::Str(s.to_owned())),
            ValueType::Float => s
                .parse::<f64>()
                .map(Value::Float)
                .unwrap_or_else(|_| Value::Str(s.to_owned())),
            ValueType::Boolean => {
                Value::Bool(matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
            }
            _ => Value::Str(s.to_owned()),
        }
    }
}

#[async_trait]
impl Transform for IfNull {
    fn name(&self) -> &str {
        "IfNull"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let mut values = row.values.clone();
        for rep in &self.config.replacements {
            if let Some(idx) = row.schema.fields.iter().position(|f| f.name == rep.field)
                && values[idx].is_null()
            {
                values[idx] = Self::coerce(&rep.default_value, &row.schema.fields[idx].value_type);
            }
        }
        Ok(vec![Row::new(row.schema.clone(), values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, ValueType};
    use std::sync::Arc;

    fn make_row(name: Option<&str>, age: Option<i64>) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("name", ValueType::String),
            Field::new("age", ValueType::Integer),
        ]));
        Row::new(
            schema,
            vec![
                name.map(|s| Value::Str(s.into())).unwrap_or(Value::Null),
                age.map(Value::Int).unwrap_or(Value::Null),
            ],
        )
    }

    #[tokio::test]
    async fn replaces_null_string() {
        let mut t = IfNull::new(IfNullConfig {
            replacements: vec![IfNullReplacement {
                field: "name".into(),
                default_value: "Unknown".into(),
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let row = make_row(None, Some(42));
        let out = t.process(row).await.unwrap();
        assert_eq!(out[0].get("name"), Some(&Value::Str("Unknown".into())));
        assert_eq!(out[0].get("age"), Some(&Value::Int(42)));
    }

    #[tokio::test]
    async fn replaces_null_int_with_coercion() {
        let mut t = IfNull::new(IfNullConfig {
            replacements: vec![IfNullReplacement {
                field: "age".into(),
                default_value: "0".into(),
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let row = make_row(Some("Alice"), None);
        let out = t.process(row).await.unwrap();
        assert_eq!(out[0].get("age"), Some(&Value::Int(0)));
    }

    #[tokio::test]
    async fn leaves_non_null_unchanged() {
        let mut t = IfNull::new(IfNullConfig {
            replacements: vec![IfNullReplacement {
                field: "name".into(),
                default_value: "Unknown".into(),
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let row = make_row(Some("Alice"), Some(30));
        let out = t.process(row).await.unwrap();
        assert_eq!(out[0].get("name"), Some(&Value::Str("Alice".into())));
    }
}
