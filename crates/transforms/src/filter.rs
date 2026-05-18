use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum Condition {
    Eq { field: String, value: String },
    Ne { field: String, value: String },
    Gt { field: String, value: String },
    Lt { field: String, value: String },
    Gte { field: String, value: String },
    Lte { field: String, value: String },
    IsNull { field: String },
    NotNull { field: String },
    Contains { field: String, value: String },
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
    Not { condition: Box<Condition> },
}

impl Condition {
    pub fn evaluate(&self, row: &Row) -> bool {
        match self {
            Condition::IsNull { field } => row.get(field).map(|v| v.is_null()).unwrap_or(true),
            Condition::NotNull { field } => row.get(field).map(|v| !v.is_null()).unwrap_or(false),
            Condition::Eq { field, value } => row
                .get(field)
                .map(|v| v.to_display_string() == *value)
                .unwrap_or(false),
            Condition::Ne { field, value } => row
                .get(field)
                .map(|v| v.to_display_string() != *value)
                .unwrap_or(false),
            Condition::Gt { field, value } => compare_numeric(row, field, value, |a, b| a > b),
            Condition::Lt { field, value } => compare_numeric(row, field, value, |a, b| a < b),
            Condition::Gte { field, value } => compare_numeric(row, field, value, |a, b| a >= b),
            Condition::Lte { field, value } => compare_numeric(row, field, value, |a, b| a <= b),
            Condition::Contains { field, value } => row
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.contains(value.as_str()))
                .unwrap_or(false),
            Condition::And { conditions } => conditions.iter().all(|c| c.evaluate(row)),
            Condition::Or { conditions } => conditions.iter().any(|c| c.evaluate(row)),
            Condition::Not { condition } => !condition.evaluate(row),
        }
    }
}

fn compare_numeric(row: &Row, field: &str, value: &str, cmp: impl Fn(f64, f64) -> bool) -> bool {
    let row_val = row.get(field).and_then(|v| match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(f) => Some(*f),
        Value::Str(s) => s.parse::<f64>().ok(),
        _ => None,
    });
    let cmp_val = value.parse::<f64>().ok();
    match (row_val, cmp_val) {
        (Some(a), Some(b)) => cmp(a, b),
        _ => false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRowsConfig {
    /// If omitted, all rows pass through (useful as a passthrough/tee node)
    pub condition: Option<Condition>,
}

pub struct FilterRows {
    config: FilterRowsConfig,
}

impl FilterRows {
    pub fn new(config: FilterRowsConfig) -> Self {
        Self { config }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: FilterRowsConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for FilterRows {
    fn name(&self) -> &str {
        "FilterRows"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let passes = match &self.config.condition {
            Some(cond) => cond.evaluate(&row),
            None => true, // no condition = pass all rows
        };
        if passes {
            Ok(vec![row])
        } else {
            Ok(vec![])
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

    #[test]
    fn filter_eq() {
        let row = make_row("Alice", 30);
        let cond = Condition::Eq {
            field: "name".into(),
            value: "Alice".into(),
        };
        assert!(cond.evaluate(&row));
        let cond2 = Condition::Eq {
            field: "name".into(),
            value: "Bob".into(),
        };
        assert!(!cond2.evaluate(&row));
    }

    #[test]
    fn filter_gt() {
        let row = make_row("Alice", 30);
        let cond = Condition::Gt {
            field: "age".into(),
            value: "20".into(),
        };
        assert!(cond.evaluate(&row));
        let cond2 = Condition::Gt {
            field: "age".into(),
            value: "40".into(),
        };
        assert!(!cond2.evaluate(&row));
    }
}
