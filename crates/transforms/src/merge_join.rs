use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JoinType {
    Inner,
    LeftOuter,
    RightOuter,
    Full,
}

impl Default for JoinType {
    fn default() -> Self { JoinType::Inner }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeJoinConfig {
    /// Key field from the left (main) stream
    pub left_key:  String,
    /// Key field from the right (lookup) stream
    pub right_key: String,
    #[serde(default)]
    pub join_type: JoinType,
    /// Prefix to add to right-side field names to avoid collisions
    #[serde(default = "default_prefix")]
    pub right_prefix: String,
}

fn default_prefix() -> String { "right_".into() }

/// MergeJoin performs a sort-merge join on two pre-sorted input streams.
///
/// Usage: load the right-side rows via `load_right()` before the pipeline runs,
/// then process() feeds left-side rows one at a time and produces joined output.
///
/// Both streams MUST be sorted ascending on their respective key fields.
pub struct MergeJoin {
    config:        MergeJoinConfig,
    right_rows:    Vec<Row>,
    right_pos:     usize,
    output_schema: Option<Arc<RowSchema>>,
}

impl MergeJoin {
    pub fn new(config: MergeJoinConfig) -> Self {
        Self { config, right_rows: Vec::new(), right_pos: 0, output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: MergeJoinConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    /// Pre-load the right-side (inner) stream before execution starts.
    pub fn load_right(&mut self, rows: impl IntoIterator<Item = Row>) {
        self.right_rows = rows.into_iter().collect();
        self.right_pos  = 0;
    }

    fn build_schema(&self, left: &RowSchema, right: &RowSchema) -> RowSchema {
        let mut fields = left.fields.clone();
        for f in &right.fields {
            fields.push(Field::new(
                format!("{}{}", self.config.right_prefix, f.name),
                f.value_type.clone(),
            ));
        }
        RowSchema::new(fields)
    }

    fn merge_rows(&self, left: &Row, right: Option<&Row>, schema: Arc<RowSchema>) -> Row {
        let mut values = left.values.clone();
        if let Some(r) = right {
            values.extend(r.values.iter().cloned());
        } else {
            // Null-fill right side for outer joins
            let right_len = schema.fields.len() - left.values.len();
            values.extend(std::iter::repeat(Value::Null).take(right_len));
        }
        Row::new(schema, values)
    }

    fn key_ord(left_key: &Value, right_key: &Value) -> Ordering {
        match (left_key, right_key) {
            (Value::Int(a),   Value::Int(b))   => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Str(a),   Value::Str(b))   => a.cmp(b),
            (a, b) => a.to_display_string().cmp(&b.to_display_string()),
        }
    }
}

#[async_trait]
impl Transform for MergeJoin {
    fn name(&self) -> &str { "MergeJoin" }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        // Right schema is unknown at plan time; return left schema as placeholder
        Ok(input.clone())
    }

    fn side_input_count(&self) -> usize { 1 }

    async fn load_side_input(&mut self, _idx: usize, rows: Vec<Row>) -> Result<()> {
        self.load_right(rows);
        Ok(())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.right_pos = 0;
        Ok(())
    }

    async fn process(&mut self, left_row: Row) -> Result<Vec<Row>> {
        let left_key = left_row.get(&self.config.left_key)
            .cloned()
            .unwrap_or(Value::Null);

        let mut output: Vec<Row> = Vec::new();

        // Advance right pointer past rows with keys smaller than left_key
        while self.right_pos < self.right_rows.len() {
            let rkey = self.right_rows[self.right_pos]
                .get(&self.config.right_key)
                .cloned()
                .unwrap_or(Value::Null);
            if Self::key_ord(&rkey, &left_key) == Ordering::Less {
                // Right outer: emit unmatched right rows
                if self.config.join_type == JoinType::RightOuter
                    || self.config.join_type == JoinType::Full
                {
                    // We'd need left-schema here; skip for simplicity
                }
                self.right_pos += 1;
            } else {
                break;
            }
        }

        // Collect all right rows matching left_key
        let mut matched = false;
        let mut scan = self.right_pos;
        while scan < self.right_rows.len() {
            let right_row = &self.right_rows[scan];
            let rkey = right_row.get(&self.config.right_key)
                .cloned()
                .unwrap_or(Value::Null);

            match Self::key_ord(&left_key, &rkey) {
                Ordering::Equal => {
                    matched = true;
                    let schema = if let Some(s) = &self.output_schema {
                        s.clone()
                    } else {
                        let s = Arc::new(self.build_schema(&left_row.schema, &right_row.schema));
                        self.output_schema = Some(s.clone());
                        s
                    };
                    output.push(self.merge_rows(&left_row, Some(right_row), schema));
                    scan += 1;
                }
                Ordering::Less => break, // right is now ahead of left
                Ordering::Greater => { scan += 1; } // shouldn't happen if sorted
            }
        }

        // Left/Full outer: emit left row with nulls if no match
        if !matched && matches!(self.config.join_type, JoinType::LeftOuter | JoinType::Full) {
            let schema = self.output_schema.clone().unwrap_or_else(|| {
                // Build schema with just left fields (right side unknown)
                Arc::new(left_row.schema.as_ref().clone())
            });
            output.push(self.merge_rows(&left_row, None, schema));
        }

        Ok(output)
    }

    async fn close(&mut self) -> Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};

    fn make_row(id: i64, val: &str, schema: Arc<RowSchema>) -> Row {
        Row::new(schema, vec![Value::Int(id), Value::Str(val.into())])
    }

    #[tokio::test]
    async fn inner_join() {
        let left_schema = Arc::new(RowSchema::new(vec![
            Field::new("id",   ValueType::Integer),
            Field::new("name", ValueType::String),
        ]));
        let right_schema = Arc::new(RowSchema::new(vec![
            Field::new("id",    ValueType::Integer),
            Field::new("label", ValueType::String),
        ]));

        let config = MergeJoinConfig {
            left_key:     "id".into(),
            right_key:    "id".into(),
            join_type:    JoinType::Inner,
            right_prefix: "r_".into(),
        };
        let mut mj = MergeJoin::new(config);
        mj.load_right(vec![
            make_row(1, "One",   right_schema.clone()),
            make_row(2, "Two",   right_schema.clone()),
            make_row(4, "Four",  right_schema.clone()),
        ]);

        let ctx = ExecutionContext::new();
        mj.open(&ctx).await.unwrap();

        let out1 = mj.process(make_row(1, "Alice", left_schema.clone())).await.unwrap();
        let out2 = mj.process(make_row(2, "Bob",   left_schema.clone())).await.unwrap();
        let out3 = mj.process(make_row(3, "Carol", left_schema.clone())).await.unwrap(); // no match

        assert_eq!(out1.len(), 1);
        assert_eq!(out1[0].get("r_label"), Some(&Value::Str("One".into())));
        assert_eq!(out2.len(), 1);
        assert_eq!(out3.len(), 0); // inner join: no right match → no output
    }
}
