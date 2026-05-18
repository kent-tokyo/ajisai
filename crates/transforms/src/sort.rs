use ajisai_core::{
    context::ExecutionContext, error::Result, value::{Row, RowSchema, Value},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortKey {
    pub field:      String,
    #[serde(default = "default_ascending")]
    pub ascending:  bool,
}

fn default_ascending() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortRowsConfig {
    pub keys: Vec<SortKey>,
}

/// SortRows buffers all rows, sorts with rayon, then emits them.
/// This is a blocking (batch) transform by nature.
pub struct SortRows {
    config: SortRowsConfig,
    buffer: Vec<Row>,
}

impl SortRows {
    pub fn new(config: SortRowsConfig) -> Self {
        Self { config, buffer: Vec::new() }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: SortRowsConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn compare_values(a: &Value, b: &Value) -> Ordering {
        match (a, b) {
            (Value::Int(x),   Value::Int(y))   => x.cmp(y),
            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
            (Value::Int(x),   Value::Float(y)) => (*x as f64).partial_cmp(y).unwrap_or(Ordering::Equal),
            (Value::Float(x), Value::Int(y))   => x.partial_cmp(&(*y as f64)).unwrap_or(Ordering::Equal),
            (Value::Str(x),   Value::Str(y))   => x.cmp(y),
            (Value::Bool(x),  Value::Bool(y))  => x.cmp(y),
            (Value::Null,     Value::Null)      => Ordering::Equal,
            (Value::Null,     _)               => Ordering::Less,
            (_,               Value::Null)     => Ordering::Greater,
            (a, b) => a.to_display_string().cmp(&b.to_display_string()),
        }
    }
}

#[async_trait]
impl Transform for SortRows {
    fn name(&self) -> &str { "SortRows" }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.buffer.clear();
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        self.buffer.push(row);
        Ok(vec![]) // emit nothing until close()
    }

    async fn flush(&mut self) -> Result<Vec<Row>> {
        Ok(self.flush_sorted())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

impl SortRows {
    /// Called by the engine after all process() calls to get sorted output.
    /// NOTE: The standard Transform trait doesn't have this — the engine handles
    /// SortRows specially via the `flush_sorted` extension pattern.
    pub fn flush_sorted(&mut self) -> Vec<Row> {
        let keys = self.config.keys.clone();

        // Use rayon for parallel sort
        use rayon::prelude::*;
        self.buffer.par_sort_unstable_by(|a, b| {
            for key in &keys {
                let va = a.get(&key.field).unwrap_or(&Value::Null);
                let vb = b.get(&key.field).unwrap_or(&Value::Null);
                let ord = Self::compare_values(va, vb);
                if ord != Ordering::Equal {
                    return if key.ascending { ord } else { ord.reverse() };
                }
            }
            Ordering::Equal
        });

        std::mem::take(&mut self.buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};
    use std::sync::Arc;

    fn make_row(name: &str, age: i64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("name", ValueType::String),
            Field::new("age",  ValueType::Integer),
        ]));
        Row::new(schema, vec![Value::Str(name.into()), Value::Int(age)])
    }

    #[test]
    fn sort_by_age_asc() {
        let config = SortRowsConfig {
            keys: vec![SortKey { field: "age".into(), ascending: true }],
        };
        let mut sorter = SortRows::new(config);
        sorter.buffer = vec![make_row("C", 30), make_row("A", 10), make_row("B", 20)];
        let sorted = sorter.flush_sorted();
        let ages: Vec<i64> = sorted.iter()
            .map(|r| r.get("age").and_then(|v| v.as_int()).unwrap())
            .collect();
        assert_eq!(ages, vec![10, 20, 30]);
    }
}
