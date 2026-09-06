use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// A field to retrieve from the lookup stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupReturnField {
    /// Field name in the lookup stream
    pub name: String,
    /// Rename in the output row (defaults to `name`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamLookupConfig {
    /// The name of the lookup transform node (pre-loaded as a source)
    pub lookup_transform: String,
    /// Key field in the main stream
    pub key_field: String,
    /// Key field in the lookup stream (must match type with `key_field`)
    pub lookup_key_field: String,
    /// Fields to copy from the lookup row into the output row
    pub return_fields: Vec<LookupReturnField>,
    /// Value to emit for missing keys (null by default)
    #[serde(default)]
    pub no_match_value: Option<String>,
}

/// StreamLookup loads a side input (lookup table) into a HashMap keyed by
/// `lookup_key_field`, then for each main-stream row it enriches the row
/// with fields from the matching lookup row.
///
/// The lookup table must be provided via `load_lookup()` before execution.
pub struct StreamLookup {
    config: StreamLookupConfig,
    /// key → lookup Row
    lookup_map: HashMap<String, Row>,
    output_schema: Option<Arc<RowSchema>>,
}

impl StreamLookup {
    pub fn new(config: StreamLookupConfig) -> Self {
        Self {
            config,
            lookup_map: HashMap::new(),
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: StreamLookupConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    /// Pre-load the lookup table from an iterable of rows.
    /// Called by the engine before pipeline execution starts.
    pub fn load_lookup(&mut self, rows: impl IntoIterator<Item = Row>) {
        self.lookup_map.clear();
        let key_field = &self.config.lookup_key_field;
        for row in rows {
            if let Some(key) = row.get(key_field) {
                self.lookup_map.insert(key.to_display_string(), row);
            }
        }
    }

    fn build_output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        for rf in &self.config.return_fields {
            let name = rf.rename.as_deref().unwrap_or(&rf.name).to_owned();
            // Type is String unless lookup schema is known at build time
            fields.push(Field::new(name, ajisai_core::ValueType::String));
        }
        Ok(RowSchema::new(fields))
    }
}

#[async_trait]
impl Transform for StreamLookup {
    fn name(&self) -> &str {
        "StreamLookup"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        self.build_output_schema(input)
    }

    fn side_input_count(&self) -> usize {
        1
    }

    async fn load_side_input(&mut self, _idx: usize, rows: Vec<Row>) -> Result<()> {
        self.load_lookup(rows);
        Ok(())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = if let Some(s) = &self.output_schema {
            s.clone()
        } else {
            let s = Arc::new(self.build_output_schema(&row.schema)?);
            self.output_schema = Some(s.clone());
            s
        };

        let key = row
            .get(&self.config.key_field)
            .map(|v| v.to_display_string())
            .unwrap_or_default();

        let lookup_row = self.lookup_map.get(&key);

        let no_match = self
            .config
            .no_match_value
            .as_deref()
            .map(|s| Value::Str(s.to_owned()))
            .unwrap_or(Value::Null);

        let mut values = row.values.clone();
        for rf in &self.config.return_fields {
            let v = lookup_row
                .and_then(|lr| lr.get(&rf.name))
                .cloned()
                .unwrap_or_else(|| no_match.clone());
            values.push(v);
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

    fn make_row(fields: &[(&str, ValueType)], values: Vec<Value>) -> Row {
        let schema = Arc::new(RowSchema::new(
            fields
                .iter()
                .map(|(n, t)| Field::new(*n, t.clone()))
                .collect(),
        ));
        Row::new(schema, values)
    }

    #[tokio::test]
    async fn lookup_hit_and_miss() {
        let config = StreamLookupConfig {
            lookup_transform: "lookup".into(),
            key_field: "id".into(),
            lookup_key_field: "id".into(),
            return_fields: vec![LookupReturnField {
                name: "label".into(),
                rename: None,
            }],
            no_match_value: Some("N/A".into()),
        };
        let mut sl = StreamLookup::new(config);

        let lookup_rows = vec![
            make_row(
                &[("id", ValueType::String), ("label", ValueType::String)],
                vec![Value::Str("1".into()), Value::Str("One".into())],
            ),
            make_row(
                &[("id", ValueType::String), ("label", ValueType::String)],
                vec![Value::Str("2".into()), Value::Str("Two".into())],
            ),
        ];
        sl.load_lookup(lookup_rows);

        let ctx = ExecutionContext::new();
        sl.open(&ctx).await.unwrap();

        let hit = make_row(&[("id", ValueType::String)], vec![Value::Str("1".into())]);
        let out = sl.process(hit).await.unwrap();
        assert_eq!(out[0].get("label"), Some(&Value::Str("One".into())));

        let miss = make_row(&[("id", ValueType::String)], vec![Value::Str("99".into())]);
        let out2 = sl.process(miss).await.unwrap();
        assert_eq!(out2[0].get("label"), Some(&Value::Str("N/A".into())));
    }
}
