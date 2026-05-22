use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderField {
    pub field: String,
    #[serde(default = "default_ascending")]
    pub ascending: bool,
}

fn default_ascending() -> bool {
    true
}

fn default_offset() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnalyticFunction {
    RowNumber,
    Rank,
    DenseRank,
    Lag,
    Lead,
    FirstValue,
    LastValue,
    SumOver,
    AvgOver,
    MinOver,
    MaxOver,
    CountOver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticSpec {
    /// Name of the output field that will hold the computed value
    pub result_field: String,
    pub function: AnalyticFunction,
    /// Source field for LAG / LEAD / FIRST_VALUE / LAST_VALUE / SUM_OVER / AVG_OVER / MIN_OVER / MAX_OVER
    #[serde(default)]
    pub subject: Option<String>,
    /// Number of rows to look back (LAG) or forward (LEAD). Default: 1.
    #[serde(default = "default_offset")]
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticQueryConfig {
    /// Fields that define the partition boundary. Empty = single global partition.
    #[serde(default)]
    pub partition_fields: Vec<String>,
    /// Fields that define the ordering within each partition
    #[serde(default)]
    pub order_fields: Vec<OrderField>,
    /// Window function specifications to compute
    pub analytics: Vec<AnalyticSpec>,
}

pub struct AnalyticQuery {
    config: AnalyticQueryConfig,
    buffer: Vec<Row>,
    output_schema: Option<Arc<RowSchema>>,
}

impl AnalyticQuery {
    pub fn new(config: AnalyticQueryConfig) -> Self {
        Self {
            config,
            buffer: Vec::new(),
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: AnalyticQueryConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_output_schema(input: &RowSchema, analytics: &[AnalyticSpec]) -> Arc<RowSchema> {
        let mut fields = input.fields.clone();
        for spec in analytics {
            let vt = match spec.function {
                AnalyticFunction::RowNumber
                | AnalyticFunction::Rank
                | AnalyticFunction::DenseRank
                | AnalyticFunction::CountOver => ValueType::Integer,
                AnalyticFunction::AvgOver | AnalyticFunction::SumOver => ValueType::Float,
                _ => ValueType::String, // will carry the type of subject at runtime
            };
            fields.push(Field::new(spec.result_field.clone(), vt));
        }
        Arc::new(RowSchema::new(fields))
    }

    fn partition_key(row: &Row, partition_fields: &[String]) -> String {
        if partition_fields.is_empty() {
            return String::new();
        }
        partition_fields
            .iter()
            .map(|f| row.get(f).map(|v| v.to_display_string()).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\x00")
    }

    fn compare_rows(a: &Row, b: &Row, order_fields: &[OrderField]) -> Ordering {
        for of in order_fields {
            let va = a.get(&of.field).unwrap_or(&Value::Null);
            let vb = b.get(&of.field).unwrap_or(&Value::Null);
            let ord = va.compare(vb);
            if ord != Ordering::Equal {
                return if of.ascending { ord } else { ord.reverse() };
            }
        }
        Ordering::Equal
    }

    fn numeric_value(v: &Value) -> Option<f64> {
        match v {
            Value::Int(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    fn compute_partition(
        &self,
        partition: &[Row],
        out_schema: Arc<RowSchema>,
    ) -> Vec<Row> {
        let n = partition.len();
        let mut result = Vec::with_capacity(n);

        // Precompute rank-related arrays (rank values may differ from row_number on ties)
        // rank_values[i] = rank (1-based, with gaps on ties)
        let mut rank_values = vec![0usize; n];
        let mut dense_rank_values = vec![0usize; n];

        {
            let mut dense_rank = 1usize;
            let mut prev_rank_start = 0usize;

            for i in 0..n {
                if i == 0 {
                    rank_values[i] = 1;
                    dense_rank_values[i] = 1;
                } else {
                    let tied = Self::compare_rows(
                        &partition[i],
                        &partition[i - 1],
                        &self.config.order_fields,
                    ) == Ordering::Equal;
                    if tied {
                        rank_values[i] = rank_values[prev_rank_start];
                        dense_rank_values[i] = dense_rank;
                    } else {
                        rank_values[i] = i + 1;
                        dense_rank += 1;
                        dense_rank_values[i] = dense_rank;
                        prev_rank_start = i;
                    }
                }
            }
        }

        for (i, row) in partition.iter().enumerate() {
            let mut values = row.values.clone();

            for spec in &self.config.analytics {
                let v = match spec.function {
                    AnalyticFunction::RowNumber => Value::Int((i + 1) as i64),

                    AnalyticFunction::Rank => Value::Int(rank_values[i] as i64),

                    AnalyticFunction::DenseRank => Value::Int(dense_rank_values[i] as i64),

                    AnalyticFunction::Lag => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        let back = spec.offset;
                        if i >= back {
                            partition[i - back].get(src).cloned().unwrap_or(Value::Null)
                        } else {
                            Value::Null
                        }
                    }

                    AnalyticFunction::Lead => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        let fwd = spec.offset;
                        if i + fwd < n {
                            partition[i + fwd].get(src).cloned().unwrap_or(Value::Null)
                        } else {
                            Value::Null
                        }
                    }

                    AnalyticFunction::FirstValue => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        partition[0].get(src).cloned().unwrap_or(Value::Null)
                    }

                    AnalyticFunction::LastValue => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        partition[n - 1].get(src).cloned().unwrap_or(Value::Null)
                    }

                    AnalyticFunction::SumOver => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        let sum: f64 = partition
                            .iter()
                            .filter_map(|r| r.get(src).and_then(Self::numeric_value))
                            .sum();
                        Value::Float(sum)
                    }

                    AnalyticFunction::AvgOver => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        let vals: Vec<f64> = partition
                            .iter()
                            .filter_map(|r| r.get(src).and_then(Self::numeric_value))
                            .collect();
                        if vals.is_empty() {
                            Value::Null
                        } else {
                            Value::Float(vals.iter().sum::<f64>() / vals.len() as f64)
                        }
                    }

                    AnalyticFunction::MinOver => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        partition
                            .iter()
                            .filter_map(|r| r.get(src))
                            .filter(|v| !v.is_null())
                            .min_by(|a, b| a.compare(b))
                            .cloned()
                            .unwrap_or(Value::Null)
                    }

                    AnalyticFunction::MaxOver => {
                        let src = spec.subject.as_deref().unwrap_or("");
                        partition
                            .iter()
                            .filter_map(|r| r.get(src))
                            .filter(|v| !v.is_null())
                            .max_by(|a, b| a.compare(b))
                            .cloned()
                            .unwrap_or(Value::Null)
                    }

                    AnalyticFunction::CountOver => {
                        let src = spec.subject.as_deref();
                        let count = match src {
                            Some(field) => partition
                                .iter()
                                .filter(|r| !r.get(field).unwrap_or(&Value::Null).is_null())
                                .count(),
                            None => n,
                        };
                        Value::Int(count as i64)
                    }
                };
                values.push(v);
            }

            result.push(Row::new(out_schema.clone(), values));
        }

        result
    }
}

#[async_trait]
impl Transform for AnalyticQuery {
    fn name(&self) -> &str {
        "AnalyticQuery"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(Arc::try_unwrap(Self::build_output_schema(input, &self.config.analytics))
            .unwrap_or_else(|arc| (*arc).clone()))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.buffer.clear();
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        self.buffer.push(row);
        Ok(vec![])
    }

    async fn flush(&mut self) -> Result<Vec<Row>> {
        if self.buffer.is_empty() {
            return Ok(vec![]);
        }

        let out_schema = self
            .output_schema
            .get_or_insert_with(|| {
                Self::build_output_schema(&self.buffer[0].schema, &self.config.analytics)
            })
            .clone();

        // Group rows by partition key, preserving order within each group
        let mut partition_map: HashMap<String, Vec<Row>> = HashMap::new();
        let mut partition_order: Vec<String> = Vec::new();

        for row in self.buffer.drain(..) {
            let key = Self::partition_key(&row, &self.config.partition_fields);
            let entry = partition_map.entry(key.clone()).or_insert_with(|| {
                partition_order.push(key);
                Vec::new()
            });
            entry.push(row);
        }

        let order_fields = self.config.order_fields.clone();
        let mut out_rows: Vec<Row> = Vec::new();

        for key in &partition_order {
            let partition = partition_map.get_mut(key).unwrap();
            // Sort within partition by order_fields
            partition.sort_by(|a, b| Self::compare_rows(a, b, &order_fields));
            let computed = self.compute_partition(partition, out_schema.clone());
            out_rows.extend(computed);
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
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};

    fn make_row(dept: &str, salary: i64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("dept", ValueType::String),
            Field::new("salary", ValueType::Integer),
        ]));
        Row::new(
            schema,
            vec![Value::Str(dept.into()), Value::Int(salary)],
        )
    }

    #[tokio::test]
    async fn row_number_within_partition() {
        let config = AnalyticQueryConfig {
            partition_fields: vec!["dept".into()],
            order_fields: vec![OrderField {
                field: "salary".into(),
                ascending: false,
            }],
            analytics: vec![AnalyticSpec {
                result_field: "rn".into(),
                function: AnalyticFunction::RowNumber,
                subject: None,
                offset: 1,
            }],
        };
        let mut aq = AnalyticQuery::new(config);

        for (dept, sal) in [("A", 100), ("B", 200), ("A", 300), ("B", 150)] {
            aq.process(make_row(dept, sal)).await.unwrap();
        }
        let out = aq.flush().await.unwrap();
        assert_eq!(out.len(), 4);
        // Dept A: [300, 100] → row_numbers [1, 2]
        // Dept B: [200, 150] → row_numbers [1, 2]
        for row in &out {
            let rn = row.get("rn").and_then(|v| v.as_int()).unwrap();
            assert!(rn == 1 || rn == 2, "unexpected row_number {rn}");
        }
    }

    #[tokio::test]
    async fn lag_across_rows() {
        let config = AnalyticQueryConfig {
            partition_fields: vec![],
            order_fields: vec![OrderField {
                field: "salary".into(),
                ascending: true,
            }],
            analytics: vec![AnalyticSpec {
                result_field: "prev_sal".into(),
                function: AnalyticFunction::Lag,
                subject: Some("salary".into()),
                offset: 1,
            }],
        };
        let mut aq = AnalyticQuery::new(config);
        for sal in [10i64, 20, 30] {
            let schema = Arc::new(RowSchema::new(vec![Field::new("salary", ValueType::Integer)]));
            aq.process(Row::new(schema, vec![Value::Int(sal)]))
                .await
                .unwrap();
        }
        let out = aq.flush().await.unwrap();
        assert_eq!(out.len(), 3);
        assert!(out[0].get("prev_sal").unwrap().is_null());
        assert_eq!(out[1].get("prev_sal"), Some(&Value::Int(10)));
        assert_eq!(out[2].get("prev_sal"), Some(&Value::Int(20)));
    }
}
