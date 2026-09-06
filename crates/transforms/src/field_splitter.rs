use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSplitterConfig {
    /// Source field to split
    pub field: String,
    /// Delimiter string (e.g. ",", " ", "|")
    pub delimiter: String,
    /// Output field names for each split part (positional)
    pub output_fields: Vec<String>,
    /// Whether to trim whitespace from each token
    #[serde(default)]
    pub trim: bool,
}

pub struct FieldSplitter {
    config: FieldSplitterConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl FieldSplitter {
    pub fn new(config: FieldSplitterConfig) -> Self {
        Self {
            config,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: FieldSplitterConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for FieldSplitter {
    fn name(&self) -> &str {
        "FieldSplitter"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        for name in &self.config.output_fields {
            fields.push(Field::new(name.as_str(), ValueType::String));
        }
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = self
            .output_schema
            .get_or_insert_with(|| {
                let mut fields = row.schema.fields.clone();
                for name in &self.config.output_fields {
                    fields.push(Field::new(name.as_str(), ValueType::String));
                }
                Arc::new(RowSchema::new(fields))
            })
            .clone();

        let raw = row
            .get(&self.config.field)
            .map(|v| v.to_display_string())
            .unwrap_or_default();

        let parts: Vec<&str> = raw.split(&*self.config.delimiter).collect();

        let mut values = row.values.clone();
        for (i, out_name) in self.config.output_fields.iter().enumerate() {
            let _ = out_name; // used only for schema; value by position
            let token = parts.get(i).copied().unwrap_or("");
            let token = if self.config.trim {
                token.trim()
            } else {
                token
            };
            values.push(Value::Str(token.to_owned()));
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

    fn make_row(csv: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("data", ValueType::String)]));
        Row::new(schema, vec![Value::Str(csv.into())])
    }

    #[tokio::test]
    async fn splits_comma_separated() {
        let mut t = FieldSplitter::new(FieldSplitterConfig {
            field: "data".into(),
            delimiter: ",".into(),
            output_fields: vec!["a".into(), "b".into(), "c".into()],
            trim: true,
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(make_row("hello, world, foo")).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("a"), Some(&Value::Str("hello".into())));
        assert_eq!(out[0].get("b"), Some(&Value::Str("world".into())));
        assert_eq!(out[0].get("c"), Some(&Value::Str("foo".into())));
    }

    #[tokio::test]
    async fn fewer_parts_than_fields_gives_empty_strings() {
        let mut t = FieldSplitter::new(FieldSplitterConfig {
            field: "data".into(),
            delimiter: ",".into(),
            output_fields: vec!["a".into(), "b".into(), "c".into()],
            trim: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(make_row("x,y")).await.unwrap();
        assert_eq!(out[0].get("c"), Some(&Value::Str("".into())));
    }
}
