use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AggregateOp {
    Sum,
    Count,
    CountAll,
    Avg,
    Min,
    Max,
    First,
    Last,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateSpec {
    pub result_field: String,
    pub subject_field: Option<String>,
    pub operation: AggregateOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGroupByConfig {
    pub group_fields: Vec<String>,
    pub aggregates: Vec<AggregateSpec>,
}

/// Accumulated state per aggregation per group
#[derive(Debug)]
enum Accumulator {
    Sum(f64),
    Count(u64),
    CountAll(u64),
    Avg { sum: f64, count: u64 },
    Min(Option<Value>),
    Max(Option<Value>),
    First(Option<Value>),
    Last(Option<Value>),
}

impl Accumulator {
    fn new(op: &AggregateOp) -> Self {
        match op {
            AggregateOp::Sum => Accumulator::Sum(0.0),
            AggregateOp::Count => Accumulator::Count(0),
            AggregateOp::CountAll => Accumulator::CountAll(0),
            AggregateOp::Avg => Accumulator::Avg { sum: 0.0, count: 0 },
            AggregateOp::Min => Accumulator::Min(None),
            AggregateOp::Max => Accumulator::Max(None),
            AggregateOp::First => Accumulator::First(None),
            AggregateOp::Last => Accumulator::Last(None),
        }
    }

    fn update(&mut self, val: Option<&Value>) {
        match self {
            Accumulator::Sum(s) => {
                if let Some(v) = val.and_then(|v| v.as_float()) {
                    *s += v;
                }
            }
            Accumulator::Count(c) => {
                if val.map(|v| !v.is_null()).unwrap_or(false) {
                    *c += 1;
                }
            }
            Accumulator::CountAll(c) => {
                *c += 1;
            }
            Accumulator::Avg { sum, count } => {
                if let Some(v) = val.and_then(|v| v.as_float()) {
                    *sum += v;
                    *count += 1;
                }
            }
            Accumulator::Min(m) => {
                if let Some(v) = val
                    && !v.is_null()
                {
                    *m = Some(match m.take() {
                        None => v.clone(),
                        Some(cur) => min_value(cur, v.clone()),
                    });
                }
            }
            Accumulator::Max(m) => {
                if let Some(v) = val
                    && !v.is_null()
                {
                    *m = Some(match m.take() {
                        None => v.clone(),
                        Some(cur) => max_value(cur, v.clone()),
                    });
                }
            }
            Accumulator::First(f) => {
                if f.is_none() {
                    *f = val.cloned();
                }
            }
            Accumulator::Last(l) => {
                *l = val.cloned();
            }
        }
    }

    fn result(&self) -> Value {
        match self {
            Accumulator::Sum(s) => Value::Float(*s),
            Accumulator::Count(c) | Accumulator::CountAll(c) => Value::Int(*c as i64),
            Accumulator::Avg { sum, count } => {
                if *count == 0 {
                    Value::Null
                } else {
                    Value::Float(sum / *count as f64)
                }
            }
            Accumulator::Min(v) | Accumulator::Max(v) | Accumulator::First(v) => {
                v.clone().unwrap_or(Value::Null)
            }
            Accumulator::Last(v) => v.clone().unwrap_or(Value::Null),
        }
    }
}

fn min_value(a: Value, b: Value) -> Value {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if x <= y {
                a
            } else {
                b
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if x <= y {
                a
            } else {
                b
            }
        }
        (Value::Str(x), Value::Str(y)) => {
            if x <= y {
                a
            } else {
                b
            }
        }
        _ => a,
    }
}

fn max_value(a: Value, b: Value) -> Value {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if x >= y {
                a
            } else {
                b
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if x >= y {
                a
            } else {
                b
            }
        }
        (Value::Str(x), Value::Str(y)) => {
            if x >= y {
                a
            } else {
                b
            }
        }
        _ => a,
    }
}

type GroupKey = Vec<String>;
type GroupAccumulators = Vec<Accumulator>;

pub struct MemoryGroupBy {
    config: MemoryGroupByConfig,
    groups: HashMap<GroupKey, GroupAccumulators>,
    /// Insertion order to keep output deterministic
    order: Vec<GroupKey>,
    output_schema: Option<Arc<RowSchema>>,
}

impl MemoryGroupBy {
    pub fn new(config: MemoryGroupByConfig) -> Self {
        Self {
            config,
            groups: HashMap::new(),
            order: Vec::new(),
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: MemoryGroupByConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_output_schema(&self) -> Arc<RowSchema> {
        let mut fields: Vec<Field> = self
            .config
            .group_fields
            .iter()
            .map(|n| Field::new(n.as_str(), ValueType::String))
            .collect();
        for agg in &self.config.aggregates {
            let vt = match agg.operation {
                AggregateOp::Count | AggregateOp::CountAll => ValueType::Integer,
                AggregateOp::Sum | AggregateOp::Avg | AggregateOp::Min | AggregateOp::Max => {
                    ValueType::Float
                }
                AggregateOp::First | AggregateOp::Last => ValueType::String,
            };
            fields.push(Field::new(agg.result_field.as_str(), vt));
        }
        Arc::new(RowSchema::new(fields))
    }
}

#[async_trait]
impl Transform for MemoryGroupBy {
    fn name(&self) -> &str {
        "MemoryGroupBy"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        Ok((*self.build_output_schema()).clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        // Extract group key
        let key: GroupKey = self
            .config
            .group_fields
            .iter()
            .map(|f| {
                row.get(f)
                    .map(|v| v.to_display_string())
                    .unwrap_or_default()
            })
            .collect();

        // Get or insert accumulator row for this group
        let accumulators = self.groups.entry(key.clone()).or_insert_with(|| {
            self.order.push(key.clone());
            self.config
                .aggregates
                .iter()
                .map(|agg| Accumulator::new(&agg.operation))
                .collect()
        });

        // Update each accumulator
        for (acc, agg) in accumulators.iter_mut().zip(&self.config.aggregates) {
            let val = agg.subject_field.as_ref().and_then(|f| row.get(f));
            acc.update(val);
        }

        Ok(vec![])
    }

    async fn flush(&mut self) -> Result<Vec<Row>> {
        let schema = self.build_output_schema();
        self.output_schema = Some(schema.clone());

        let mut rows = Vec::with_capacity(self.order.len());
        for key in &self.order {
            let accs = &self.groups[key];
            let mut values: Vec<Value> = key.iter().map(|s| Value::Str(s.clone())).collect();
            for acc in accs {
                values.push(acc.result());
            }
            rows.push(Row::new(schema.clone(), values));
        }
        Ok(rows)
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, ValueType};

    fn make_row(dept: &str, salary: f64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("dept", ValueType::String),
            Field::new("salary", ValueType::Float),
        ]));
        Row::new(schema, vec![Value::Str(dept.into()), Value::Float(salary)])
    }

    #[tokio::test]
    async fn group_by_sum_count() {
        let mut t = MemoryGroupBy::new(MemoryGroupByConfig {
            group_fields: vec!["dept".into()],
            aggregates: vec![
                AggregateSpec {
                    result_field: "total_salary".into(),
                    subject_field: Some("salary".into()),
                    operation: AggregateOp::Sum,
                },
                AggregateSpec {
                    result_field: "headcount".into(),
                    subject_field: Some("salary".into()),
                    operation: AggregateOp::Count,
                },
            ],
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        t.process(make_row("eng", 100.0)).await.unwrap();
        t.process(make_row("eng", 200.0)).await.unwrap();
        t.process(make_row("sales", 150.0)).await.unwrap();
        let rows = t.flush().await.unwrap();

        assert_eq!(rows.len(), 2);
        let eng = rows
            .iter()
            .find(|r| r.get("dept") == Some(&Value::Str("eng".into())))
            .unwrap();
        assert_eq!(eng.get("total_salary"), Some(&Value::Float(300.0)));
        assert_eq!(eng.get("headcount"), Some(&Value::Int(2)));
    }

    #[tokio::test]
    async fn count_all() {
        let mut t = MemoryGroupBy::new(MemoryGroupByConfig {
            group_fields: vec!["dept".into()],
            aggregates: vec![AggregateSpec {
                result_field: "cnt".into(),
                subject_field: None,
                operation: AggregateOp::CountAll,
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        t.process(make_row("eng", 0.0)).await.unwrap();
        t.process(make_row("eng", 0.0)).await.unwrap();
        let rows = t.flush().await.unwrap();
        assert_eq!(rows[0].get("cnt"), Some(&Value::Int(2)));
    }
}
