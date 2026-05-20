use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// One unpivot specification: a set of source fields that map to one type_value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizeSpec {
    /// Value written into type_field for this group (e.g. "Q1_Sales")
    pub type_value: String,
    /// Source field names whose values become value_field (one row per field)
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowNormaliserConfig {
    /// Output field that holds the "type" label
    pub type_field: String,
    /// Output field that holds the original value
    pub value_field: String,
    /// Columns that are kept as-is on every output row
    pub non_pivot_fields: Vec<String>,
    pub normalize_specs: Vec<NormalizeSpec>,
}

pub struct RowNormaliser {
    config: RowNormaliserConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl RowNormaliser {
    pub fn new(config: RowNormaliserConfig) -> Self {
        Self { config, output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: RowNormaliserConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_schema(config: &RowNormaliserConfig, input: &RowSchema) -> Arc<RowSchema> {
        let mut fields: Vec<Field> = config
            .non_pivot_fields
            .iter()
            .filter_map(|n| input.field(n))
            .cloned()
            .collect();
        fields.push(Field::new(&config.type_field, ValueType::String));
        fields.push(Field::new(&config.value_field, ValueType::String));
        Arc::new(RowSchema::new(fields))
    }
}

#[async_trait]
impl Transform for RowNormaliser {
    fn name(&self) -> &str {
        "RowNormaliser"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok((*Self::build_schema(&self.config, input)).clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = self.output_schema.get_or_insert_with(|| {
            Self::build_schema(&self.config, &row.schema)
        });

        // Build the non-pivot portion once
        let non_pivot_values: Vec<Value> = self
            .config
            .non_pivot_fields
            .iter()
            .map(|n| row.get(n).cloned().unwrap_or(Value::Null))
            .collect();

        let mut out_rows = Vec::new();

        for spec in &self.config.normalize_specs {
            for src_field in &spec.fields {
                let val = row.get(src_field).cloned().unwrap_or(Value::Null);
                let mut values = non_pivot_values.clone();
                values.push(Value::Str(spec.type_value.clone()));
                values.push(match val {
                    Value::Null => Value::Null,
                    other => Value::Str(other.to_display_string()),
                });
                out_rows.push(Row::new(schema.clone(), values));
            }
        }

        Ok(out_rows)
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, ValueType};

    fn make_row() -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("dept", ValueType::String),
            Field::new("q1", ValueType::Float),
            Field::new("q2", ValueType::Float),
        ]));
        Row::new(schema, vec![
            Value::Str("eng".into()),
            Value::Float(100.0),
            Value::Float(200.0),
        ])
    }

    #[tokio::test]
    async fn unpivots_two_columns() {
        let mut t = RowNormaliser::new(RowNormaliserConfig {
            type_field: "quarter".into(),
            value_field: "sales".into(),
            non_pivot_fields: vec!["dept".into()],
            normalize_specs: vec![
                NormalizeSpec { type_value: "Q1".into(), fields: vec!["q1".into()] },
                NormalizeSpec { type_value: "Q2".into(), fields: vec!["q2".into()] },
            ],
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let rows = t.process(make_row()).await.unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("quarter"), Some(&Value::Str("Q1".into())));
        assert_eq!(rows[0].get("sales"), Some(&Value::Str("100".into())));
        assert_eq!(rows[1].get("quarter"), Some(&Value::Str("Q2".into())));
        assert_eq!(rows[1].get("dept"), Some(&Value::Str("eng".into())));
    }
}
