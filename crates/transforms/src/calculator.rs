use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A single calculated field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calculation {
    /// Output field name
    pub field: String,
    /// Output field type
    #[serde(default = "default_type")]
    pub value_type: String,
    /// The operation to perform
    pub operation: Operation,
}

fn default_type() -> String {
    "String".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum Operation {
    // Arithmetic
    Add {
        a: Operand,
        b: Operand,
    },
    Subtract {
        a: Operand,
        b: Operand,
    },
    Multiply {
        a: Operand,
        b: Operand,
    },
    Divide {
        a: Operand,
        b: Operand,
    },
    Modulo {
        a: Operand,
        b: Operand,
    },
    // String
    Concat {
        parts: Vec<Operand>,
    },
    Upper {
        field: String,
    },
    Lower {
        field: String,
    },
    Trim {
        field: String,
    },
    Length {
        field: String,
    },
    Substring {
        field: String,
        start: usize,
        length: Option<usize>,
    },
    Replace {
        field: String,
        from: String,
        to: String,
    },
    // Type conversion
    ToString {
        field: String,
    },
    ToInteger {
        field: String,
    },
    ToFloat {
        field: String,
    },
    // Conditional
    IfNull {
        field: String,
        default: Operand,
    },
    Coalesce {
        fields: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum Operand {
    /// Reference to a field in the current row
    Field { field: String },
    /// Literal integer
    LitInt(i64),
    /// Literal float
    LitFloat(f64),
    /// Literal string
    LitStr(String),
}

impl Operand {
    pub fn resolve(&self, row: &Row) -> Value {
        match self {
            Operand::Field { field } => row.get(field).cloned().unwrap_or(Value::Null),
            Operand::LitInt(n) => Value::Int(*n),
            Operand::LitFloat(f) => Value::Float(*f),
            Operand::LitStr(s) => Value::Str(s.clone()),
        }
    }
}

fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(f) => Some(*f),
        Value::Str(s) => s.parse().ok(),
        _ => None,
    }
}

impl Operation {
    pub fn evaluate(&self, row: &Row) -> Value {
        match self {
            Operation::Add { a, b } => {
                let av = a.resolve(row);
                let bv = b.resolve(row);
                match (&av, &bv) {
                    (Value::Int(x), Value::Int(y)) => {
                        x.checked_add(*y).map(Value::Int).unwrap_or(Value::Null)
                    }
                    _ => match (to_f64(&av), to_f64(&bv)) {
                        (Some(x), Some(y)) => Value::Float(x + y),
                        _ => Value::Null,
                    },
                }
            }
            Operation::Subtract { a, b } => {
                let av = a.resolve(row);
                let bv = b.resolve(row);
                match (&av, &bv) {
                    (Value::Int(x), Value::Int(y)) => {
                        x.checked_sub(*y).map(Value::Int).unwrap_or(Value::Null)
                    }
                    _ => match (to_f64(&av), to_f64(&bv)) {
                        (Some(x), Some(y)) => Value::Float(x - y),
                        _ => Value::Null,
                    },
                }
            }
            Operation::Multiply { a, b } => {
                let av = a.resolve(row);
                let bv = b.resolve(row);
                match (&av, &bv) {
                    (Value::Int(x), Value::Int(y)) => {
                        x.checked_mul(*y).map(Value::Int).unwrap_or(Value::Null)
                    }
                    _ => match (to_f64(&av), to_f64(&bv)) {
                        (Some(x), Some(y)) => Value::Float(x * y),
                        _ => Value::Null,
                    },
                }
            }
            Operation::Divide { a, b } => {
                match (to_f64(&a.resolve(row)), to_f64(&b.resolve(row))) {
                    (Some(x), Some(y)) if y != 0.0 => Value::Float(x / y),
                    _ => Value::Null,
                }
            }
            Operation::Modulo { a, b } => match (&a.resolve(row), &b.resolve(row)) {
                (Value::Int(x), Value::Int(y)) if *y != 0 => Value::Int(x % y),
                _ => Value::Null,
            },
            Operation::Concat { parts } => {
                let s: String = parts
                    .iter()
                    .map(|p| p.resolve(row).to_display_string())
                    .collect();
                Value::Str(s)
            }
            Operation::Upper { field } => row
                .get(field)
                .and_then(|v| v.as_str().map(|s| Value::Str(s.to_uppercase())))
                .unwrap_or(Value::Null),
            Operation::Lower { field } => row
                .get(field)
                .and_then(|v| v.as_str().map(|s| Value::Str(s.to_lowercase())))
                .unwrap_or(Value::Null),
            Operation::Trim { field } => row
                .get(field)
                .and_then(|v| v.as_str().map(|s| Value::Str(s.trim().to_owned())))
                .unwrap_or(Value::Null),
            Operation::Length { field } => row
                .get(field)
                .and_then(|v| v.as_str().map(|s| Value::Int(s.len() as i64)))
                .unwrap_or(Value::Null),
            Operation::Substring {
                field,
                start,
                length,
            } => row
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| {
                    let chars: Vec<char> = s.chars().collect();
                    let from = (*start).min(chars.len());
                    let slice: String = match length {
                        Some(len) => chars[from..].iter().take(*len).collect(),
                        None => chars[from..].iter().collect(),
                    };
                    Value::Str(slice)
                })
                .unwrap_or(Value::Null),
            Operation::Replace { field, from, to } => row
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| Value::Str(s.replace(from.as_str(), to.as_str())))
                .unwrap_or(Value::Null),
            Operation::ToString { field } => row
                .get(field)
                .map(|v| Value::Str(v.to_display_string()))
                .unwrap_or(Value::Null),
            Operation::ToInteger { field } => row
                .get(field)
                .and_then(|v| to_f64(v))
                .map(|f| Value::Int(f as i64))
                .unwrap_or(Value::Null),
            Operation::ToFloat { field } => row
                .get(field)
                .and_then(|v| to_f64(v))
                .map(Value::Float)
                .unwrap_or(Value::Null),
            Operation::IfNull { field, default } => match row.get(field) {
                Some(v) if !v.is_null() => v.clone(),
                _ => default.resolve(row),
            },
            Operation::Coalesce { fields } => fields
                .iter()
                .find_map(|f| row.get(f).filter(|v| !v.is_null()).cloned())
                .unwrap_or(Value::Null),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatorConfig {
    pub calculations: Vec<Calculation>,
}

pub struct CalculatorStep {
    config: CalculatorConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl CalculatorStep {
    pub fn new(config: CalculatorConfig) -> Self {
        Self {
            config,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: CalculatorConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_schema(&self, input: &RowSchema) -> RowSchema {
        let mut fields = input.fields.clone();
        for calc in &self.config.calculations {
            let vt = match calc.value_type.as_str() {
                "Integer" => ValueType::Integer,
                "Float" => ValueType::Float,
                "Boolean" => ValueType::Boolean,
                _ => ValueType::String,
            };
            // Replace existing field or append new one
            if let Some(idx) = fields.iter().position(|f| f.name == calc.field) {
                fields[idx] = Field::new(calc.field.clone(), vt);
            } else {
                fields.push(Field::new(calc.field.clone(), vt));
            }
        }
        RowSchema::new(fields)
    }
}

#[async_trait]
impl Transform for CalculatorStep {
    fn name(&self) -> &str {
        "CalculatorStep"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(self.build_schema(input))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = if let Some(s) = &self.output_schema {
            s.clone()
        } else {
            let s = Arc::new(self.build_schema(&row.schema));
            self.output_schema = Some(s.clone());
            s
        };

        let mut values = row.values.clone();

        for calc in &self.config.calculations {
            let result = calc.operation.evaluate(&row);
            if let Some(idx) = schema.field_index(&calc.field) {
                if idx < values.len() {
                    values[idx] = result;
                } else {
                    values.push(result);
                }
            } else {
                values.push(result);
            }
        }

        Ok(vec![Row::new(schema, values)])
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

    fn make_row() -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("price", ValueType::Float),
            Field::new("quantity", ValueType::Integer),
            Field::new("name", ValueType::String),
        ]));
        Row::new(
            schema,
            vec![
                Value::Float(9.99),
                Value::Int(3),
                Value::Str("  hello  ".into()),
            ],
        )
    }

    #[tokio::test]
    async fn multiply_fields() {
        let config = CalculatorConfig {
            calculations: vec![Calculation {
                field: "total".into(),
                value_type: "Float".into(),
                operation: Operation::Multiply {
                    a: Operand::Field {
                        field: "price".into(),
                    },
                    b: Operand::Field {
                        field: "quantity".into(),
                    },
                },
            }],
        };
        let mut calc = CalculatorStep::new(config);
        let ctx = ExecutionContext::new();
        calc.open(&ctx).await.unwrap();
        let out = calc.process(make_row()).await.unwrap();
        let total = out[0].get("total").unwrap();
        assert!((total.as_float().unwrap() - 29.97).abs() < 0.001);
    }

    #[tokio::test]
    async fn trim_and_upper() {
        let config = CalculatorConfig {
            calculations: vec![
                Calculation {
                    field: "trimmed".into(),
                    value_type: "String".into(),
                    operation: Operation::Trim {
                        field: "name".into(),
                    },
                },
                Calculation {
                    field: "uppername".into(),
                    value_type: "String".into(),
                    operation: Operation::Upper {
                        field: "name".into(),
                    },
                },
            ],
        };
        let mut calc = CalculatorStep::new(config);
        let ctx = ExecutionContext::new();
        calc.open(&ctx).await.unwrap();
        let out = calc.process(make_row()).await.unwrap();
        assert_eq!(out[0].get("trimmed"), Some(&Value::Str("hello".into())));
        assert_eq!(
            out[0].get("uppername"),
            Some(&Value::Str("  HELLO  ".into()))
        );
    }
}
