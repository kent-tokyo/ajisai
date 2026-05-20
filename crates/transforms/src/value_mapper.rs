use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingEntry {
    pub source_value: String,
    pub target_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueMapperConfig {
    /// Source field to map
    pub field: String,
    /// Output field name (defaults to `field` if omitted — in-place replacement)
    #[serde(default)]
    pub output_field: Option<String>,
    /// Lookup table: source_value → target_value
    #[serde(default)]
    pub mappings: Vec<MappingEntry>,
    /// Emitted when source_value is not found in mappings
    #[serde(default)]
    pub default_value: Option<String>,
}

pub struct ValueMapper {
    config: ValueMapperConfig,
    lookup: HashMap<String, String>,
}

impl ValueMapper {
    pub fn new(config: ValueMapperConfig) -> Self {
        let lookup: HashMap<String, String> = config
            .mappings
            .iter()
            .map(|m| (m.source_value.clone(), m.target_value.clone()))
            .collect();
        Self { config, lookup }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ValueMapperConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for ValueMapper {
    fn name(&self) -> &str {
        "ValueMapper"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, mut row: Row) -> Result<Vec<Row>> {
        let out_field = self
            .config
            .output_field
            .as_deref()
            .unwrap_or(&self.config.field);

        let raw = row
            .get(&self.config.field)
            .map(|v| v.to_display_string())
            .unwrap_or_default();

        let mapped = if let Some(target) = self.lookup.get(&raw) {
            Value::Str(target.clone())
        } else if let Some(def) = &self.config.default_value {
            Value::Str(def.clone())
        } else {
            // No match and no default: leave the field value unchanged
            return Ok(vec![row]);
        };

        if let Some(idx) = row.schema.fields.iter().position(|f| f.name == out_field) {
            row.values[idx] = mapped;
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

    fn make_row(status: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new(
            "status",
            ValueType::String,
        )]));
        Row::new(schema, vec![Value::Str(status.into())])
    }

    fn make_transform() -> ValueMapper {
        ValueMapper::new(ValueMapperConfig {
            field: "status".into(),
            output_field: None,
            mappings: vec![
                MappingEntry {
                    source_value: "Y".into(),
                    target_value: "Yes".into(),
                },
                MappingEntry {
                    source_value: "N".into(),
                    target_value: "No".into(),
                },
            ],
            default_value: Some("Unknown".into()),
        })
    }

    #[tokio::test]
    async fn maps_known_values() {
        let mut t = make_transform();
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row("Y")).await.unwrap();
        assert_eq!(out[0].get("status"), Some(&Value::Str("Yes".into())));

        let out = t.process(make_row("N")).await.unwrap();
        assert_eq!(out[0].get("status"), Some(&Value::Str("No".into())));
    }

    #[tokio::test]
    async fn uses_default_for_unknown() {
        let mut t = make_transform();
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row("?")).await.unwrap();
        assert_eq!(out[0].get("status"), Some(&Value::Str("Unknown".into())));
    }

    #[tokio::test]
    async fn passthrough_when_no_default_and_no_match() {
        let mut t = ValueMapper::new(ValueMapperConfig {
            field: "status".into(),
            output_field: None,
            mappings: vec![],
            default_value: None,
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(make_row("original")).await.unwrap();
        assert_eq!(out[0].get("status"), Some(&Value::Str("original".into())));
    }
}
