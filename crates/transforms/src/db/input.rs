use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Column, Row as SqlxRow, TypeInfo};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInputConfig {
    /// JDBC-style URL: sqlite://path.db, postgres://user:pass@host/db, mysql://...
    pub connection_url: String,
    pub sql:            String,
    /// Max rows to fetch (0 = unlimited)
    #[serde(default)]
    pub limit:          u64,
}

pub struct TableInput {
    config: TableInputConfig,
}

impl TableInput {
    pub fn new(config: TableInputConfig) -> Self { Self { config } }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: TableInputConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn sqlx_to_value(row: &sqlx::any::AnyRow, col_idx: usize) -> Value {
        let _type_name = row.column(col_idx).type_info().name().to_lowercase();

        // Try integer first
        if let Ok(v) = row.try_get::<i64, _>(col_idx) {
            return Value::Int(v);
        }
        if let Ok(v) = row.try_get::<i32, _>(col_idx) {
            return Value::Int(v as i64);
        }
        // Float
        if let Ok(v) = row.try_get::<f64, _>(col_idx) {
            return Value::Float(v);
        }
        // Bool
        if let Ok(v) = row.try_get::<bool, _>(col_idx) {
            return Value::Bool(v);
        }
        // String fallback
        if let Ok(v) = row.try_get::<String, _>(col_idx) {
            return Value::Str(v);
        }
        // Optional string (nullable)
        if let Ok(Some(v)) = row.try_get::<Option<String>, _>(col_idx) {
            return Value::Str(v);
        }

        Value::Null
    }
}

#[async_trait]
impl Transform for TableInput {
    fn name(&self) -> &str { "TableInput" }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        Ok(RowSchema::default())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> { Ok(()) }
    async fn process(&mut self, row: Row) -> Result<Vec<Row>> { Ok(vec![row]) }
    async fn close(&mut self) -> Result<()> { Ok(()) }
    fn is_source(&self) -> bool { true }

    async fn produce(&mut self, sender: mpsc::Sender<Row>) -> Result<()> {
        let url = &self.config.connection_url;
        debug!("TableInput connecting to '{}'", url);

        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect(url).await
            .map_err(|e| AjisaiError::Config(format!("DB connect failed: {}", e)))?;

        let sql = &self.config.sql;
        let rows = sqlx::query(sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| AjisaiError::Pipeline(format!("Query failed: {}", e)))?;

        if rows.is_empty() {
            return Ok(());
        }

        // Build schema from first row
        let first = &rows[0];
        let fields: Vec<Field> = first.columns().iter().map(|col| {
            let type_name = col.type_info().name().to_lowercase();
            let vt = if type_name.contains("int") {
                ValueType::Integer
            } else if type_name.contains("real") || type_name.contains("float") || type_name.contains("double") || type_name.contains("numeric") {
                ValueType::Float
            } else if type_name.contains("bool") {
                ValueType::Boolean
            } else {
                ValueType::String
            };
            Field::new(col.name(), vt)
        }).collect();
        let schema = Arc::new(RowSchema::new(fields));

        let limit = self.config.limit;
        for (i, db_row) in rows.iter().enumerate() {
            if limit > 0 && i as u64 >= limit { break; }
            let values: Vec<Value> = (0..db_row.len())
                .map(|idx| Self::sqlx_to_value(db_row, idx))
                .collect();
            let row = Row::new(schema.clone(), values);
            sender.send(row).await
                .map_err(|_| AjisaiError::Pipeline("Downstream closed".into()))?;
        }

        pool.close().await;
        Ok(())
    }
}
