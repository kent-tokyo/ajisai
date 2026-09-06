use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum JoinType {
    #[default]
    Inner,
    LeftOuter,
    RightOuter,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeJoinConfig {
    /// Key field from the left (main) stream
    pub left_key: String,
    /// Key field from the right (lookup) stream
    pub right_key: String,
    #[serde(default)]
    pub join_type: JoinType,
    /// Prefix to add to right-side field names to avoid collisions
    #[serde(default = "default_prefix")]
    pub right_prefix: String,
}

fn default_prefix() -> String {
    "right_".into()
}

/// MergeJoin performs a sort-merge join on two pre-sorted input streams.
///
/// Both streams MUST be sorted ascending on their respective key fields.
pub struct MergeJoin {
    config: MergeJoinConfig,
    right_rows: Vec<Row>,
    right_pos: usize,
    output_schema: Option<Arc<RowSchema>>,
    /// Left-side schema, captured from the first processed row.
    left_schema: Option<Arc<RowSchema>>,
    /// One-past-end of the last matched right group (for N:M join support).
    right_match_end: usize,
    /// The key of the last processed left row (for N:M join support).
    last_left_key: Option<Value>,
    /// Index where the current group of matching right rows begins.
    group_start: usize,
}

impl MergeJoin {
    pub fn new(config: MergeJoinConfig) -> Self {
        Self {
            config,
            right_rows: Vec::new(),
            right_pos: 0,
            output_schema: None,
            left_schema: None,
            right_match_end: 0,
            last_left_key: None,
            group_start: 0,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: MergeJoinConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    pub fn load_right(&mut self, rows: impl IntoIterator<Item = Row>) {
        self.right_rows = rows.into_iter().collect();
        self.right_pos = 0;
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

    /// Produce a row with actual left values and actual right values.
    fn merge_rows(&self, left: &Row, right: Option<&Row>, schema: Arc<RowSchema>) -> Row {
        let mut values = left.values.clone();
        if let Some(r) = right {
            values.extend(r.values.iter().cloned());
        } else {
            let right_len = schema.fields.len() - left.values.len();
            values.extend(std::iter::repeat_n(Value::Null, right_len));
        }
        Row::new(schema, values)
    }

    /// Produce a row with null-filled left side and actual right values.
    fn right_only_row(
        left_field_count: usize,
        right_values: Vec<Value>,
        schema: Arc<RowSchema>,
    ) -> Row {
        let mut values: Vec<Value> = std::iter::repeat_n(Value::Null, left_field_count).collect();
        values.extend(right_values);
        Row::new(schema, values)
    }

    fn key_ord(left_key: &Value, right_key: &Value) -> Ordering {
        left_key.compare(right_key)
    }

    /// Ensure output_schema is built and return a clone.
    fn ensure_schema(
        &mut self,
        left_schema: &RowSchema,
        right_schema: &RowSchema,
    ) -> Arc<RowSchema> {
        if self.output_schema.is_none() {
            self.output_schema = Some(Arc::new(self.build_schema(left_schema, right_schema)));
        }
        self.output_schema.as_ref().unwrap().clone()
    }
}

#[async_trait]
impl Transform for MergeJoin {
    fn name(&self) -> &str {
        "MergeJoin"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    fn side_input_count(&self) -> usize {
        1
    }

    async fn load_side_input(&mut self, _idx: usize, rows: Vec<Row>) -> Result<()> {
        self.load_right(rows);
        Ok(())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.right_pos = 0;
        self.left_schema = None;
        self.output_schema = None;
        self.right_match_end = 0;
        self.last_left_key = None;
        self.group_start = 0;
        Ok(())
    }

    async fn process(&mut self, left_row: Row) -> Result<Vec<Row>> {
        // Capture left schema on first call
        if self.left_schema.is_none() {
            self.left_schema = Some(left_row.schema.clone());
        }
        let left_schema = self.left_schema.as_ref().unwrap().clone();

        let left_key = left_row
            .get(&self.config.left_key)
            .cloned()
            .unwrap_or(Value::Null);

        let mut output: Vec<Row> = Vec::new();

        if Some(&left_key) != self.last_left_key.as_ref() {
            // New left key: advance right_pos past the previous matched group,
            // then run the advance loop to skip rows < left_key (emitting
            // unmatched right rows for RightOuter / Full joins).
            self.right_pos = self.right_match_end;

            while self.right_pos < self.right_rows.len() {
                let rkey = self.right_rows[self.right_pos]
                    .get(&self.config.right_key)
                    .cloned()
                    .unwrap_or(Value::Null);
                if Self::key_ord(&rkey, &left_key) == Ordering::Less {
                    if matches!(self.config.join_type, JoinType::RightOuter | JoinType::Full) {
                        let right_schema = self.right_rows[self.right_pos].schema.clone();
                        let right_values = self.right_rows[self.right_pos].values.clone();
                        let schema = self.ensure_schema(&left_schema, &right_schema);
                        output.push(Self::right_only_row(
                            left_schema.fields.len(),
                            right_values,
                            schema,
                        ));
                    }
                    self.right_pos += 1;
                } else {
                    break;
                }
            }

            // Mark the start of the potential match group for this new key
            self.group_start = self.right_pos;
        }
        // If Same key as previous left row: re-scan from group_start (no advance loop needed)

        // Collect all right rows matching left_key, scanning from group_start
        let mut matched = false;
        let mut scan = self.group_start;
        while scan < self.right_rows.len() {
            let right_row = &self.right_rows[scan];
            let rkey = right_row
                .get(&self.config.right_key)
                .cloned()
                .unwrap_or(Value::Null);

            match Self::key_ord(&left_key, &rkey) {
                Ordering::Equal => {
                    matched = true;
                    let right_schema = right_row.schema.clone();
                    let schema = self.ensure_schema(&left_schema, &right_schema);
                    // Re-borrow after ensure_schema to satisfy borrow checker
                    let right_row = &self.right_rows[scan];
                    output.push(self.merge_rows(&left_row, Some(right_row), schema));
                    scan += 1;
                }
                Ordering::Less => break,
                Ordering::Greater => {
                    scan += 1;
                }
            }
        }

        // Track the end of this group and remember the key for next call
        self.right_match_end = scan;
        self.last_left_key = Some(left_key.clone());

        // Left / Full outer: emit left row with null right side if no match
        if !matched && matches!(self.config.join_type, JoinType::LeftOuter | JoinType::Full) {
            let schema = self
                .output_schema
                .clone()
                .unwrap_or_else(|| Arc::new(left_row.schema.as_ref().clone()));
            output.push(self.merge_rows(&left_row, None, schema));
        }

        Ok(output)
    }

    /// Emit any remaining unmatched right rows for RightOuter / Full joins.
    async fn flush(&mut self) -> Result<Vec<Row>> {
        if !matches!(self.config.join_type, JoinType::RightOuter | JoinType::Full) {
            return Ok(vec![]);
        }
        let Some(left_schema) = self.left_schema.clone() else {
            return Ok(vec![]);
        };
        // Start from the furthest position we've advanced to (account for N:M joins)
        let mut pos = self.right_pos.max(self.right_match_end);
        let mut output = Vec::new();
        while pos < self.right_rows.len() {
            let right_schema = self.right_rows[pos].schema.clone();
            let right_values = self.right_rows[pos].values.clone();
            let schema = self.ensure_schema(&left_schema, &right_schema);
            output.push(Self::right_only_row(
                left_schema.fields.len(),
                right_values,
                schema,
            ));
            pos += 1;
        }
        Ok(output)
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};

    fn left_schema() -> Arc<RowSchema> {
        Arc::new(RowSchema::new(vec![
            Field::new("id", ValueType::Integer),
            Field::new("name", ValueType::String),
        ]))
    }

    fn right_schema() -> Arc<RowSchema> {
        Arc::new(RowSchema::new(vec![
            Field::new("id", ValueType::Integer),
            Field::new("label", ValueType::String),
        ]))
    }

    fn lrow(id: i64, name: &str) -> Row {
        Row::new(left_schema(), vec![Value::Int(id), Value::Str(name.into())])
    }

    fn rrow(id: i64, label: &str) -> Row {
        Row::new(
            right_schema(),
            vec![Value::Int(id), Value::Str(label.into())],
        )
    }

    async fn run_join(join_type: JoinType, left: Vec<Row>, right: Vec<Row>) -> Vec<Row> {
        let config = MergeJoinConfig {
            left_key: "id".into(),
            right_key: "id".into(),
            join_type,
            right_prefix: "r_".into(),
        };
        let mut mj = MergeJoin::new(config);
        mj.load_right(right);
        let ctx = ExecutionContext::new();
        mj.open(&ctx).await.unwrap();
        let mut out = Vec::new();
        for row in left {
            out.extend(mj.process(row).await.unwrap());
        }
        out.extend(mj.flush().await.unwrap());
        out
    }

    #[tokio::test]
    async fn inner_join() {
        let out = run_join(
            JoinType::Inner,
            vec![lrow(1, "Alice"), lrow(2, "Bob"), lrow(3, "Carol")],
            vec![rrow(1, "One"), rrow(2, "Two"), rrow(4, "Four")],
        )
        .await;
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].get("r_label"), Some(&Value::Str("One".into())));
        assert_eq!(out[1].get("r_label"), Some(&Value::Str("Two".into())));
    }

    #[tokio::test]
    async fn left_outer_join() {
        let out = run_join(
            JoinType::LeftOuter,
            vec![lrow(1, "Alice"), lrow(3, "Carol")],
            vec![rrow(1, "One"), rrow(2, "Two")],
        )
        .await;
        // Alice matches; Carol has no right match → null-filled right
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].get("r_label"), Some(&Value::Str("One".into())));
        assert_eq!(out[1].get("name"), Some(&Value::Str("Carol".into())));
        assert_eq!(out[1].get("r_label"), Some(&Value::Null));
    }

    #[tokio::test]
    async fn right_outer_join() {
        // Left: 1, 3  Right: 1, 2, 4
        // Matches: id=1
        // Unmatched right: id=2 (passed during advance), id=4 (remaining after left ends)
        let out = run_join(
            JoinType::RightOuter,
            vec![lrow(1, "Alice"), lrow(3, "Carol")],
            vec![rrow(1, "One"), rrow(2, "Two"), rrow(4, "Four")],
        )
        .await;
        // Expected: (Alice,One), (null,Two), (null,Four)
        assert_eq!(out.len(), 3, "got: {:?}", out);
        assert_eq!(out[0].get("name"), Some(&Value::Str("Alice".into())));
        assert_eq!(out[0].get("r_label"), Some(&Value::Str("One".into())));
        assert_eq!(out[1].get("name"), Some(&Value::Null));
        assert_eq!(out[1].get("r_label"), Some(&Value::Str("Two".into())));
        assert_eq!(out[2].get("name"), Some(&Value::Null));
        assert_eq!(out[2].get("r_label"), Some(&Value::Str("Four".into())));
    }

    #[tokio::test]
    async fn duplicate_left_keys() {
        // N:M join: 2 left rows with id=1, 2 right rows with id=1
        // should produce 4 output rows (2x2 cross product for matching key)
        let out = run_join(
            JoinType::Inner,
            vec![lrow(1, "Alice"), lrow(1, "Alex"), lrow(2, "Bob")],
            vec![rrow(1, "One"), rrow(1, "Uno"), rrow(2, "Two")],
        )
        .await;
        // Expected: (Alice,One), (Alice,Uno), (Alex,One), (Alex,Uno), (Bob,Two)
        assert_eq!(out.len(), 5, "got: {:?}", out);
        assert_eq!(out[0].get("name"), Some(&Value::Str("Alice".into())));
        assert_eq!(out[0].get("r_label"), Some(&Value::Str("One".into())));
        assert_eq!(out[1].get("name"), Some(&Value::Str("Alice".into())));
        assert_eq!(out[1].get("r_label"), Some(&Value::Str("Uno".into())));
        assert_eq!(out[2].get("name"), Some(&Value::Str("Alex".into())));
        assert_eq!(out[2].get("r_label"), Some(&Value::Str("One".into())));
        assert_eq!(out[3].get("name"), Some(&Value::Str("Alex".into())));
        assert_eq!(out[3].get("r_label"), Some(&Value::Str("Uno".into())));
        assert_eq!(out[4].get("name"), Some(&Value::Str("Bob".into())));
        assert_eq!(out[4].get("r_label"), Some(&Value::Str("Two".into())));
    }

    #[tokio::test]
    async fn full_outer_join() {
        // Left: 1, 3  Right: 1, 2, 4
        // Matches: id=1
        // Unmatched left: id=3 → null right
        // Unmatched right: id=2 (advance), id=4 (flush)
        let out = run_join(
            JoinType::Full,
            vec![lrow(1, "Alice"), lrow(3, "Carol")],
            vec![rrow(1, "One"), rrow(2, "Two"), rrow(4, "Four")],
        )
        .await;
        // Expected: (Alice,One), (null,Two), (Carol,null), (null,Four)
        assert_eq!(out.len(), 4, "got: {:?}", out);
        // id=1 match
        assert_eq!(out[0].get("name"), Some(&Value::Str("Alice".into())));
        assert_eq!(out[0].get("r_label"), Some(&Value::Str("One".into())));
        // id=2 unmatched right (emitted during advance when left=3 > right=2)
        assert_eq!(out[1].get("name"), Some(&Value::Null));
        assert_eq!(out[1].get("r_label"), Some(&Value::Str("Two".into())));
        // id=3 unmatched left
        assert_eq!(out[2].get("name"), Some(&Value::Str("Carol".into())));
        assert_eq!(out[2].get("r_label"), Some(&Value::Null));
        // id=4 unmatched right (emitted by flush)
        assert_eq!(out[3].get("name"), Some(&Value::Null));
        assert_eq!(out[3].get("r_label"), Some(&Value::Str("Four".into())));
    }
}
