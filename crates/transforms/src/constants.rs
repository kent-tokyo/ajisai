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
pub struct ConstantField {
    pub name: String,
    pub value_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConstantsConfig {
    pub fields: Vec<ConstantField>,
}

pub struct AddConstants {
    config: AddConstantsConfig,
    output_schema: Option<Arc<RowSchema>>,
    const_values: Vec<Value>,
}

impl AddConstants {
    pub fn new(config: AddConstantsConfig) -> Self {
        Self {
            config,
            output_schema: None,
            const_values: Vec::new(),
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: AddConstantsConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn parse_constant(cf: &ConstantField) -> Result<(Field, Value)> {
        let (vt, value) =
            match cf.value_type.as_str() {
                "Integer" => {
                    let n = cf.value.parse::<i64>().map_err(|_| {
                        AjisaiError::Parse(format!("'{}' is not an Integer", cf.value))
                    })?;
                    (ValueType::Integer, Value::Int(n))
                }
                "Float" => {
                    let f = cf.value.parse::<f64>().map_err(|_| {
                        AjisaiError::Parse(format!("'{}' is not a Float", cf.value))
                    })?;
                    (ValueType::Float, Value::Float(f))
                }
                "Boolean" => {
                    let b = match cf.value.to_lowercase().as_str() {
                        "true" | "1" | "yes" => true,
                        _ => false,
                    };
                    (ValueType::Boolean, Value::Bool(b))
                }
                _ => (ValueType::String, Value::Str(cf.value.clone())),
            };
        Ok((Field::new(cf.name.clone(), vt), value))
    }
}

#[async_trait]
impl Transform for AddConstants {
    fn name(&self) -> &str {
        "AddConstants"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        for cf in &self.config.fields {
            let (field, _) = Self::parse_constant(cf)?;
            fields.push(field);
        }
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        let mut values = Vec::new();
        for cf in &self.config.fields {
            let (_, v) = Self::parse_constant(cf)?;
            values.push(v);
        }
        self.const_values = values;
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = self.output_schema.get_or_insert_with(|| {
            let mut fields = row.schema.fields.clone();
            for cf in &self.config.fields {
                if let Ok((field, _)) = Self::parse_constant(cf) {
                    fields.push(field);
                }
            }
            Arc::new(RowSchema::new(fields))
        }).clone();

        let mut values = row.values.clone();
        values.extend(self.const_values.iter().cloned());
        Ok(vec![Row::new(schema, values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
