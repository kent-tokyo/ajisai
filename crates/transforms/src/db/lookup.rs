use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Column, Row as SqlxRow};
use std::sync::Arc;
use tokio::sync::mpsc;

/// For each input row, execute a parameterised SQL query keyed by `key_field`
/// and append the specified `return_fields` from the first matching DB row.
/// Non-matching rows pass through with Null for the return fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseLookupConfig {
    pub connection_url: String,
    /// SQL with one `?` placeholder (e.g. "SELECT * FROM t WHERE id = ?")
    pub sql: String,
    /// Field in the input row used as the lookup key
    pub key_field: String,
    /// DB columns to append to the input row; empty = append all columns
    #[serde(default)]
    pub return_fields: Vec<String>,
}

pub struct DatabaseLookup {
    config: DatabaseLookupConfig,
    pool: Option<AnyPool>,
}

impl DatabaseLookup {
    pub fn new(config: DatabaseLookupConfig) -> Self {
        Self { config, pool: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: DatabaseLookupConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn sqlx_to_value(row: &sqlx::any::AnyRow, col_idx: usize) -> Value {
        if let Ok(v) = row.try_get::<i64, _>(col_idx) {
            return Value::Int(v);
        }
        if let Ok(v) = row.try_get::<f64, _>(col_idx) {
            return Value::Float(v);
        }
        if let Ok(v) = row.try_get::<bool, _>(col_idx) {
            return Value::Bool(v);
        }
        if let Ok(v) = row.try_get::<String, _>(col_idx) {
            return Value::Str(v);
        }
        Value::Null
    }
}

#[async_trait]
impl Transform for DatabaseLookup {
    fn name(&self) -> &str {
        "DatabaseLookup"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        // Return fields are unknown at schema-time without a DB connection;
        // append the declared return_fields as String columns.
        let mut fields = input.fields.clone();
        for rf in &self.config.return_fields {
            if fields.iter().all(|f| &f.name != rf) {
                fields.push(Field::new(rf, ValueType::String));
            }
        }
        Ok(RowSchema::new(fields))
    }

    fn is_source(&self) -> bool {
        false
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect(&self.config.connection_url)
            .await
            .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;
        self.pool = Some(pool);
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| AjisaiError::Pipeline("DatabaseLookup pool not initialised".into()))?;

        // Extract key value as string for binding
        let key_val = match row.schema.field_index(&self.config.key_field) {
            Some(idx) => row.values[idx].to_display_string(),
            None => {
                return Err(AjisaiError::Config(format!(
                    "DatabaseLookup: key_field '{}' not found in input schema",
                    self.config.key_field
                )));
            }
        };

        // Execute parameterised query
        let db_row_opt = sqlx::query(&self.config.sql)
            .bind(key_val)
            .fetch_optional(pool)
            .await
            .map_err(|e| AjisaiError::Io(std::io::Error::other(e.to_string())))?;

        let Some(db_row) = db_row_opt else {
            // No match — emit row with Null for return fields
            return Ok(vec![append_nulls(row, &self.config.return_fields)]);
        };

        // Determine which DB columns to return
        let col_names: Vec<String> = db_row
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        let selected: Vec<(usize, String)> = if self.config.return_fields.is_empty() {
            col_names.iter().cloned().enumerate().collect()
        } else {
            self.config
                .return_fields
                .iter()
                .filter_map(|rf| {
                    col_names
                        .iter()
                        .position(|c| c == rf)
                        .map(|i| (i, rf.clone()))
                })
                .collect()
        };

        // Build output schema = input schema + new fields
        let mut out_fields = row.schema.fields.clone();
        for (_, name) in &selected {
            if out_fields.iter().all(|f| &f.name != name) {
                out_fields.push(Field::new(name, ValueType::String));
            }
        }
        let out_schema = Arc::new(RowSchema::new(out_fields));

        let mut out_values = row.values.clone();
        for (col_idx, name) in &selected {
            let val = Self::sqlx_to_value(&db_row, *col_idx);
            match row.schema.field_index(name) {
                Some(i) => out_values[i] = val,
                None => out_values.push(val),
            }
        }

        Ok(vec![Row {
            schema: out_schema,
            values: out_values,
        }])
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }

    async fn produce(&mut self, _tx: mpsc::Sender<Row>) -> Result<()> {
        unreachable!("DatabaseLookup is not a source")
    }
}

fn append_nulls(row: Row, return_fields: &[String]) -> Row {
    if return_fields.is_empty() {
        return row;
    }
    let mut out_fields = row.schema.fields.clone();
    for name in return_fields {
        if out_fields.iter().all(|f| &f.name != name) {
            out_fields.push(Field::new(name, ValueType::String));
        }
    }
    let extra_count = out_fields.len() - row.values.len();
    let mut out_values = row.values.clone();
    for _ in 0..extra_count {
        out_values.push(Value::Null);
    }
    Row {
        schema: Arc::new(RowSchema::new(out_fields)),
        values: out_values,
    }
}
