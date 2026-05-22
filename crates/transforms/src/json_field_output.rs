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
pub struct JsonFieldOutputConfig {
    /// Name of the output field that will contain the JSON string
    pub target_field: String,
    /// Fields to include in the JSON object. Empty = include all input fields.
    #[serde(default)]
    pub include_fields: Vec<String>,
    /// If true, remove the included source fields from the output row
    #[serde(default)]
    pub remove_source_fields: bool,
}

pub struct JsonFieldOutput {
    config: JsonFieldOutputConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl JsonFieldOutput {
    pub fn new(config: JsonFieldOutputConfig) -> Self {
        Self {
            config,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: JsonFieldOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_output_schema(&self, input: &RowSchema) -> Arc<RowSchema> {
        let mut fields: Vec<Field> = if self.config.remove_source_fields {
            let include: std::collections::HashSet<&str> = self
                .config
                .include_fields
                .iter()
                .map(String::as_str)
                .collect();
            input
                .fields
                .iter()
                .filter(|f| {
                    self.config.include_fields.is_empty() || !include.contains(f.name.as_str())
                })
                .cloned()
                .collect()
        } else {
            input.fields.clone()
        };
        fields.push(Field::new(self.config.target_field.clone(), ValueType::String));
        Arc::new(RowSchema::new(fields))
    }

    fn value_to_json(v: &Value) -> serde_json::Value {
        match v {
            Value::Null => serde_json::Value::Null,
            Value::Str(s) => serde_json::Value::String(s.clone()),
            Value::Int(n) => serde_json::json!(*n),
            Value::Float(f) => serde_json::json!(*f),
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Date(d) => serde_json::json!(*d),
            Value::Timestamp(t) => serde_json::json!(*t),
            Value::Bytes(b) => serde_json::Value::String(base64_encode(b)),
        }
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as FmtWrite;
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        let _ = write!(
            out,
            "{}{}{}{}",
            TABLE[b0 >> 2] as char,
            TABLE[((b0 & 3) << 4) | (b1 >> 4)] as char,
            if chunk.len() > 1 { TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char } else { '=' },
            if chunk.len() > 2 { TABLE[b2 & 0x3f] as char } else { '=' },
        );
    }
    out
}

#[async_trait]
impl Transform for JsonFieldOutput {
    fn name(&self) -> &str {
        "JsonFieldOutput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(Arc::try_unwrap(self.build_output_schema(input))
            .unwrap_or_else(|arc| (*arc).clone()))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let out_schema = if let Some(s) = &self.output_schema {
            s.clone()
        } else {
            let s = self.build_output_schema(&row.schema);
            self.output_schema = Some(s.clone());
            s
        };

        // Build JSON object from selected fields
        let mut obj = serde_json::Map::new();
        let include_all = self.config.include_fields.is_empty();
        for field in row.schema.fields.iter() {
            let included = include_all || self.config.include_fields.contains(&field.name);
            if included {
                let jv = Self::value_to_json(row.get(&field.name).unwrap_or(&Value::Null));
                obj.insert(field.name.clone(), jv);
            }
        }
        let json_str = serde_json::to_string(&obj).unwrap_or_default();

        // Build output values
        let mut values: Vec<Value> = if self.config.remove_source_fields {
            let include_set: std::collections::HashSet<&str> = self
                .config
                .include_fields
                .iter()
                .map(String::as_str)
                .collect();
            row.schema
                .fields
                .iter()
                .enumerate()
                .filter(|(_, f)| include_all || !include_set.contains(f.name.as_str()))
                .map(|(i, _)| row.values[i].clone())
                .collect()
        } else {
            row.values.clone()
        };
        values.push(Value::Str(json_str));

        Ok(vec![Row::new(out_schema, values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
