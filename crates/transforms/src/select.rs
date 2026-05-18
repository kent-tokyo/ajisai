use ajisai_core::{
    context::ExecutionContext, error::Result, value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectField {
    /// Source field name
    pub name:      String,
    /// Renamed to this name (if different)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename:    Option<String>,
    /// Cast to this type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectValuesConfig {
    pub fields: Vec<SelectField>,
}

pub struct SelectValues {
    config:        SelectValuesConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl SelectValues {
    pub fn new(config: SelectValuesConfig) -> Self {
        Self { config, output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: SelectValuesConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let fields: Result<Vec<Field>> = self.config.fields.iter().map(|sf| {
            let src = input.field(&sf.name).ok_or_else(|| {
                AjisaiError::SchemaMismatch { field: sf.name.clone() }
            })?;
            let name = sf.rename.as_deref().unwrap_or(&sf.name).to_owned();
            let vt = if let Some(type_str) = &sf.value_type {
                match type_str.as_str() {
                    "Integer"   => ValueType::Integer,
                    "Float"     => ValueType::Float,
                    "Boolean"   => ValueType::Boolean,
                    "String"    => ValueType::String,
                    "Date"      => ValueType::Date,
                    "Timestamp" => ValueType::Timestamp,
                    other => return Err(AjisaiError::Config(
                        format!("Unknown type: {}", other)
                    )),
                }
            } else {
                src.value_type.clone()
            };
            Ok(Field::new(name, vt))
        }).collect();
        Ok(RowSchema::new(fields?))
    }

    fn cast(value: &Value, target: &ValueType) -> Value {
        match (value, target) {
            (Value::Str(s), ValueType::Integer) => {
                s.trim().parse::<i64>().map(Value::Int).unwrap_or(Value::Null)
            }
            (Value::Str(s), ValueType::Float) => {
                s.trim().parse::<f64>().map(Value::Float).unwrap_or(Value::Null)
            }
            (Value::Str(s), ValueType::Boolean) => {
                match s.trim().to_lowercase().as_str() {
                    "true" | "1" | "yes" => Value::Bool(true),
                    "false" | "0" | "no" => Value::Bool(false),
                    _ => Value::Null,
                }
            }
            (v, ValueType::String) => Value::Str(v.to_display_string()),
            (Value::Int(n), ValueType::Float) => Value::Float(*n as f64),
            _ => value.clone(),
        }
    }
}

#[async_trait]
impl Transform for SelectValues {
    fn name(&self) -> &str { "SelectValues" }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        self.build_output_schema(input)
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> { Ok(()) }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = if let Some(s) = &self.output_schema {
            s.clone()
        } else {
            let s = Arc::new(self.build_output_schema(&row.schema)?);
            self.output_schema = Some(s.clone());
            s
        };

        let values: Vec<Value> = self.config.fields.iter().zip(schema.fields.iter()).map(|(sf, out_field)| {
            let val = row.get(&sf.name).cloned().unwrap_or(Value::Null);
            Self::cast(&val, &out_field.value_type)
        }).collect();

        Ok(vec![Row::new(schema, values)])
    }

    async fn close(&mut self) -> Result<()> { Ok(()) }
}
