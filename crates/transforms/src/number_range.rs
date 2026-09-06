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
pub struct RangeEntry {
    /// Inclusive lower bound (None = -∞)
    pub lower: Option<f64>,
    /// Inclusive upper bound (None = +∞)
    pub upper: Option<f64>,
    /// Label to emit when the value falls in this range
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberRangeConfig {
    /// Source numeric field
    pub field: String,
    /// Output field name for the category string
    pub output_field: String,
    /// Ordered list of ranges (first match wins)
    #[serde(default)]
    pub ranges: Vec<RangeEntry>,
    /// Emitted when no range matches
    #[serde(default)]
    pub default_value: String,
}

pub struct NumberRange {
    config: NumberRangeConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl NumberRange {
    pub fn new(config: NumberRangeConfig) -> Self {
        Self {
            config,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: NumberRangeConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn classify(&self, num: f64) -> &str {
        for r in &self.config.ranges {
            let above_lower = r.lower.is_none_or(|lo| num >= lo);
            let below_upper = r.upper.is_none_or(|hi| num <= hi);
            if above_lower && below_upper {
                return &r.result;
            }
        }
        &self.config.default_value
    }
}

#[async_trait]
impl Transform for NumberRange {
    fn name(&self) -> &str {
        "NumberRange"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        fields.push(Field::new(&self.config.output_field, ValueType::String));
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
                fields.push(Field::new(&self.config.output_field, ValueType::String));
                Arc::new(RowSchema::new(fields))
            })
            .clone();

        let num = match row.get(&self.config.field) {
            Some(Value::Int(i)) => *i as f64,
            Some(Value::Float(f)) => *f,
            Some(Value::Str(s)) => s.parse::<f64>().unwrap_or(f64::NAN),
            _ => f64::NAN,
        };

        let category = self.classify(num).to_owned();
        let mut values = row.values.clone();
        values.push(Value::Str(category));
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

    fn make_row(score: f64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("score", ValueType::Float)]));
        Row::new(schema, vec![Value::Float(score)])
    }

    fn make_transform() -> NumberRange {
        NumberRange::new(NumberRangeConfig {
            field: "score".into(),
            output_field: "grade".into(),
            ranges: vec![
                RangeEntry {
                    lower: None,
                    upper: Some(59.9),
                    result: "F".into(),
                },
                RangeEntry {
                    lower: Some(60.0),
                    upper: Some(74.9),
                    result: "C".into(),
                },
                RangeEntry {
                    lower: Some(75.0),
                    upper: Some(89.9),
                    result: "B".into(),
                },
                RangeEntry {
                    lower: Some(90.0),
                    upper: None,
                    result: "A".into(),
                },
            ],
            default_value: "?".into(),
        })
    }

    #[tokio::test]
    async fn classifies_ranges() {
        let mut t = make_transform();
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row(95.0)).await.unwrap();
        assert_eq!(out[0].get("grade"), Some(&Value::Str("A".into())));

        let out = t.process(make_row(80.0)).await.unwrap();
        assert_eq!(out[0].get("grade"), Some(&Value::Str("B".into())));

        let out = t.process(make_row(45.0)).await.unwrap();
        assert_eq!(out[0].get("grade"), Some(&Value::Str("F".into())));
    }

    #[tokio::test]
    async fn uses_default_when_no_match() {
        let mut t = NumberRange::new(NumberRangeConfig {
            field: "score".into(),
            output_field: "grade".into(),
            ranges: vec![RangeEntry {
                lower: Some(0.0),
                upper: Some(100.0),
                result: "ok".into(),
            }],
            default_value: "out-of-range".into(),
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(make_row(150.0)).await.unwrap();
        assert_eq!(
            out[0].get("grade"),
            Some(&Value::Str("out-of-range".into()))
        );
    }
}
