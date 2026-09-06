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
#[derive(Default)]
pub enum DenormAggregate {
    #[default]
    First,
    Last,
    Sum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetField {
    pub key_value: String,
    pub result_field: String,
    #[serde(default)]
    pub aggregate: DenormAggregate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowDenormaliserConfig {
    pub group_fields: Vec<String>,
    /// The field that acts as pivot key (its value is matched against target_fields[i].key_value)
    pub key_field: String,
    /// Which field's value is used as the data value to pivot into result_field
    pub value_field: String,
    pub target_fields: Vec<TargetField>,
}

type GroupKey = Vec<String>;

struct GroupState {
    group_values: Vec<Value>,
    pivoted: HashMap<String, Value>,
}

pub struct RowDenormaliser {
    config: RowDenormaliserConfig,
    groups: HashMap<GroupKey, GroupState>,
    order: Vec<GroupKey>,
    output_schema: Option<Arc<RowSchema>>,
}

impl RowDenormaliser {
    pub fn new(config: RowDenormaliserConfig) -> Self {
        Self {
            config,
            groups: HashMap::new(),
            order: Vec::new(),
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: RowDenormaliserConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_output_schema(config: &RowDenormaliserConfig, input: &RowSchema) -> Arc<RowSchema> {
        let mut fields: Vec<Field> = config
            .group_fields
            .iter()
            .filter_map(|n| input.field(n))
            .cloned()
            .collect();
        for tf in &config.target_fields {
            fields.push(Field::new(&tf.result_field, ValueType::String));
        }
        Arc::new(RowSchema::new(fields))
    }
}

#[async_trait]
impl Transform for RowDenormaliser {
    fn name(&self) -> &str {
        "RowDenormaliser"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok((*Self::build_output_schema(&self.config, input)).clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
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

        let group = self.groups.entry(key.clone()).or_insert_with(|| {
            self.order.push(key.clone());
            let group_values: Vec<Value> = self
                .config
                .group_fields
                .iter()
                .map(|f| row.get(f).cloned().unwrap_or(Value::Null))
                .collect();
            GroupState {
                group_values,
                pivoted: HashMap::new(),
            }
        });

        let pivot_key = row
            .get(&self.config.key_field)
            .map(|v| v.to_display_string())
            .unwrap_or_default();
        let data_val = row
            .get(&self.config.value_field)
            .cloned()
            .unwrap_or(Value::Null);

        // Find matching target field(s)
        for tf in &self.config.target_fields {
            if tf.key_value == pivot_key {
                let entry = group.pivoted.entry(tf.result_field.clone());
                match tf.aggregate {
                    DenormAggregate::First => {
                        entry.or_insert(data_val.clone());
                    }
                    DenormAggregate::Last => {
                        *group
                            .pivoted
                            .entry(tf.result_field.clone())
                            .or_insert(Value::Null) = data_val.clone();
                    }
                    DenormAggregate::Sum => {
                        let cur = group
                            .pivoted
                            .entry(tf.result_field.clone())
                            .or_insert(Value::Float(0.0));
                        let a = cur.as_float().unwrap_or(0.0);
                        let b = data_val.as_float().unwrap_or(0.0);
                        *cur = Value::Float(a + b);
                    }
                }
            }
        }

        Ok(vec![])
    }

    async fn flush(&mut self) -> Result<Vec<Row>> {
        let schema = match &self.output_schema {
            Some(s) => s.clone(),
            None => {
                // Build schema from first group
                if self.order.is_empty() {
                    return Ok(vec![]);
                }
                let mut fields: Vec<Field> = self
                    .config
                    .group_fields
                    .iter()
                    .map(|n| Field::new(n.as_str(), ValueType::String))
                    .collect();
                for tf in &self.config.target_fields {
                    fields.push(Field::new(&tf.result_field, ValueType::String));
                }
                let s = Arc::new(RowSchema::new(fields));
                self.output_schema = Some(s.clone());
                s
            }
        };

        let mut rows = Vec::with_capacity(self.order.len());
        for key in &self.order {
            let gs = &self.groups[key];
            let mut values = gs.group_values.clone();
            for tf in &self.config.target_fields {
                let v = gs
                    .pivoted
                    .get(&tf.result_field)
                    .cloned()
                    .unwrap_or(Value::Null);
                values.push(match v {
                    Value::Null => Value::Null,
                    other => Value::Str(other.to_display_string()),
                });
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

    fn make_row(dept: &str, quarter: &str, sales: f64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("dept", ValueType::String),
            Field::new("quarter", ValueType::String),
            Field::new("sales", ValueType::Float),
        ]));
        Row::new(
            schema,
            vec![
                Value::Str(dept.into()),
                Value::Str(quarter.into()),
                Value::Float(sales),
            ],
        )
    }

    #[tokio::test]
    async fn pivots_quarter_data() {
        let mut t = RowDenormaliser::new(RowDenormaliserConfig {
            group_fields: vec!["dept".into()],
            key_field: "quarter".into(),
            value_field: "sales".into(),
            target_fields: vec![
                TargetField {
                    key_value: "Q1".into(),
                    result_field: "q1_sales".into(),
                    aggregate: DenormAggregate::First,
                },
                TargetField {
                    key_value: "Q2".into(),
                    result_field: "q2_sales".into(),
                    aggregate: DenormAggregate::First,
                },
            ],
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        t.process(make_row("eng", "Q1", 100.0)).await.unwrap();
        t.process(make_row("eng", "Q2", 200.0)).await.unwrap();
        t.process(make_row("sales", "Q1", 300.0)).await.unwrap();

        let rows = t.flush().await.unwrap();
        assert_eq!(rows.len(), 2);
        let eng = rows
            .iter()
            .find(|r| r.get("dept") == Some(&Value::Str("eng".into())))
            .unwrap();
        assert_eq!(eng.get("q1_sales"), Some(&Value::Str("100".into())));
        assert_eq!(eng.get("q2_sales"), Some(&Value::Str("200".into())));
    }
}
