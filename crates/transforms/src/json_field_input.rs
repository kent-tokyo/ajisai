use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn default_type() -> String {
    "String".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonExtractField {
    /// Output field name
    pub name: String,
    /// Dot-notation path into the JSON object, e.g. "user.address.city".
    /// Empty string or "$" means the root value itself.
    #[serde(default)]
    pub path: String,
    #[serde(default = "default_type")]
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFieldInputConfig {
    /// Name of the input field that contains the JSON string
    pub source_field: String,
    /// Fields to extract from the parsed JSON
    pub fields: Vec<JsonExtractField>,
    /// If true and the source is a JSON array, emit one row per element
    #[serde(default)]
    pub expand_array: bool,
}

pub struct JsonFieldInput {
    config: JsonFieldInputConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl JsonFieldInput {
    pub fn new(config: JsonFieldInputConfig) -> Self {
        Self {
            config,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: JsonFieldInputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_output_schema(input: &RowSchema, fields: &[JsonExtractField]) -> Arc<RowSchema> {
        let mut schema_fields = input.fields.clone();
        for f in fields {
            let vt = match f.field_type.as_str() {
                "Integer" => ValueType::Integer,
                "Float" => ValueType::Float,
                "Boolean" => ValueType::Boolean,
                _ => ValueType::String,
            };
            schema_fields.push(Field::new(f.name.clone(), vt));
        }
        Arc::new(RowSchema::new(schema_fields))
    }

    /// Walk a dot-notation path into a JSON value.
    /// "a.b.0.c" → value["a"]["b"][0]["c"]
    fn traverse<'v>(value: &'v serde_json::Value, path: &str) -> &'v serde_json::Value {
        static NULL: serde_json::Value = serde_json::Value::Null;
        if path.is_empty() || path == "$" {
            return value;
        }
        let mut cur = value;
        for segment in path.split('.') {
            cur = if let Ok(idx) = segment.parse::<usize>() {
                cur.get(idx).unwrap_or(&NULL)
            } else {
                cur.get(segment).unwrap_or(&NULL)
            };
        }
        cur
    }

    fn json_to_value(v: &serde_json::Value, field_type: &str) -> Value {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => match field_type {
                "String" => Value::Str(b.to_string()),
                _ => Value::Bool(*b),
            },
            serde_json::Value::Number(n) => match field_type {
                "Integer" => n.as_i64().map(Value::Int).unwrap_or(Value::Null),
                "Float" => n.as_f64().map(Value::Float).unwrap_or(Value::Null),
                "Boolean" => Value::Bool(n.as_i64().unwrap_or(0) != 0),
                _ => n
                    .as_i64()
                    .map(Value::Int)
                    .or_else(|| n.as_f64().map(Value::Float))
                    .unwrap_or(Value::Null),
            },
            serde_json::Value::String(s) => match field_type {
                "Integer" => s.parse::<i64>().map(Value::Int).unwrap_or(Value::Null),
                "Float" => s.parse::<f64>().map(Value::Float).unwrap_or(Value::Null),
                "Boolean" => Value::Bool(matches!(s.to_lowercase().as_str(), "true" | "1" | "yes")),
                _ => Value::Str(s.clone()),
            },
            other => Value::Str(other.to_string()),
        }
    }

    fn apply_extract(
        &self,
        row: &Row,
        json_val: &serde_json::Value,
        schema: Arc<RowSchema>,
    ) -> Row {
        let mut values = row.values.clone();
        for f in &self.config.fields {
            let extracted = Self::traverse(json_val, &f.path);
            values.push(Self::json_to_value(extracted, &f.field_type));
        }
        Row::new(schema, values)
    }
}

#[async_trait]
impl Transform for JsonFieldInput {
    fn name(&self) -> &str {
        "JsonFieldInput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(
            Arc::try_unwrap(Self::build_output_schema(input, &self.config.fields))
                .unwrap_or_else(|arc| (*arc).clone()),
        )
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = self
            .output_schema
            .get_or_insert_with(|| Self::build_output_schema(&row.schema, &self.config.fields))
            .clone();

        let json_str = match row.get(&self.config.source_field) {
            Some(Value::Str(s)) => s.clone(),
            Some(other) => other.to_display_string(),
            None => return Ok(vec![row]),
        };

        let json_val: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(_) => {
                // Unparseable JSON → pass through with null extract fields
                let mut values = row.values.clone();
                for _ in &self.config.fields {
                    values.push(Value::Null);
                }
                return Ok(vec![Row::new(schema, values)]);
            }
        };

        if self.config.expand_array {
            if let Some(arr) = json_val.as_array() {
                let rows: Vec<Row> = arr
                    .iter()
                    .map(|elem| self.apply_extract(&row, elem, schema.clone()))
                    .collect();
                return Ok(rows);
            }
        }

        Ok(vec![self.apply_extract(&row, &json_val, schema)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
