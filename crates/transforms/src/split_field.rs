use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Splits a string field into multiple rows — one row per token.
///
/// The source field is removed from the output schema and replaced with
/// `output_field` (same name allowed). NULL or empty tokens are dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitFieldToRowsConfig {
    /// Source field to split
    pub field: String,
    /// Delimiter string (e.g. ",", "|", " ")
    pub delimiter: String,
    /// Output field name for each token
    pub output_field: String,
    /// When true, trim whitespace from each token
    #[serde(default)]
    pub trim: bool,
}

pub struct SplitFieldToRows {
    config: SplitFieldToRowsConfig,
    output_schema: Option<Arc<RowSchema>>,
    /// Index of the source field in the input schema
    src_idx: Option<usize>,
}

impl SplitFieldToRows {
    pub fn new(config: SplitFieldToRowsConfig) -> Self {
        Self {
            config,
            output_schema: None,
            src_idx: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: SplitFieldToRowsConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for SplitFieldToRows {
    fn name(&self) -> &str {
        "SplitFieldToRows"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        // Remove source field, add output_field (String)
        let mut fields: Vec<Field> = input
            .fields
            .iter()
            .filter(|f| f.name != self.config.field)
            .cloned()
            .collect();
        fields.push(Field::new(
            self.config.output_field.clone(),
            ValueType::String,
        ));
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.output_schema = None;
        self.src_idx = None;
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let src_idx = match self.src_idx {
            Some(i) => i,
            None => {
                let i = row
                    .schema
                    .fields
                    .iter()
                    .position(|f| f.name == self.config.field)
                    .ok_or_else(|| {
                        AjisaiError::Config(format!(
                            "SplitFieldToRows: field '{}' not found",
                            self.config.field
                        ))
                    })?;
                self.src_idx = Some(i);
                i
            }
        };

        let schema = match &self.output_schema {
            Some(s) => s.clone(),
            None => {
                let s = Arc::new(self.output_schema(&row.schema)?);
                self.output_schema = Some(s.clone());
                s
            }
        };

        let text = match &row.values[src_idx] {
            Value::Str(s) => s.clone(),
            Value::Null => return Ok(vec![]),
            other => other.to_display_string(),
        };

        let delim = if self.config.delimiter.is_empty() {
            " "
        } else {
            &self.config.delimiter
        };

        // Build base values without the source field
        let base_values: Vec<Value> = row
            .values
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != src_idx)
            .map(|(_, v)| v.clone())
            .collect();

        let out: Vec<Row> = text
            .split(delim)
            .filter_map(|token| {
                let t = if self.config.trim {
                    token.trim()
                } else {
                    token
                };
                if t.is_empty() {
                    return None;
                }
                let mut values = base_values.clone();
                values.push(Value::Str(t.to_owned()));
                Some(Row::new(schema.clone(), values))
            })
            .collect();

        Ok(out)
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

    fn make_row(id: i64, tags: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("id", ValueType::Integer),
            Field::new("tags", ValueType::String),
        ]));
        Row::new(schema, vec![Value::Int(id), Value::Str(tags.into())])
    }

    #[tokio::test]
    async fn splits_into_rows() {
        let mut t = SplitFieldToRows::new(SplitFieldToRowsConfig {
            field: "tags".into(),
            delimiter: ",".into(),
            output_field: "tag".into(),
            trim: true,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row(1, "a, b, c")).await.unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].get("id"), Some(&Value::Int(1)));
        assert_eq!(out[0].get("tag"), Some(&Value::Str("a".into())));
        assert_eq!(out[1].get("tag"), Some(&Value::Str("b".into())));
        assert_eq!(out[2].get("tag"), Some(&Value::Str("c".into())));
        // original "tags" field must be gone
        assert!(out[0].get("tags").is_none());
    }

    #[tokio::test]
    async fn null_field_emits_no_rows() {
        let mut t = SplitFieldToRows::new(SplitFieldToRowsConfig {
            field: "tags".into(),
            delimiter: ",".into(),
            output_field: "tag".into(),
            trim: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let schema = Arc::new(RowSchema::new(vec![
            Field::new("id", ValueType::Integer),
            Field::new("tags", ValueType::String),
        ]));
        let row = Row::new(schema, vec![Value::Int(1), Value::Null]);
        let out = t.process(row).await.unwrap();
        assert!(out.is_empty());
    }
}
