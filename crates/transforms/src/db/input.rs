use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
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
    pub sql: String,
    /// Max rows to fetch (0 = unlimited)
    #[serde(default)]
    pub limit: u64,
}

pub struct TableInput {
    config: TableInputConfig,
    resolved_url: Option<String>,
}

impl TableInput {
    pub fn new(config: TableInputConfig) -> Self {
        Self {
            config,
            resolved_url: None,
        }
    }

    /// Mask the password in a connection URL for safe logging.
    /// Replaces `://user:password@` patterns with `://user:***@`.
    fn mask_url_password(url: &str) -> String {
        // Find "://" then look for user:pass@host pattern
        if let Some(scheme_end) = url.find("://") {
            let after_scheme = &url[scheme_end + 3..];
            if let Some(at_pos) = after_scheme.find('@') {
                let credentials = &after_scheme[..at_pos];
                if let Some(colon_pos) = credentials.find(':') {
                    let user = &credentials[..colon_pos];
                    let masked = format!(
                        "{}://{}:***@{}",
                        &url[..scheme_end],
                        user,
                        &after_scheme[at_pos + 1..]
                    );
                    return masked;
                }
            }
        }
        url.to_string()
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: TableInputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
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
    fn name(&self) -> &str {
        "TableInput"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        Ok(RowSchema::default())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_url = Some(ctx.resolve(&self.config.connection_url));
        Ok(())
    }
    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row])
    }
    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
    fn is_source(&self) -> bool {
        true
    }

    async fn produce(&mut self, sender: mpsc::Sender<Row>) -> Result<()> {
        let url = self
            .resolved_url
            .as_deref()
            .unwrap_or(&self.config.connection_url)
            .to_owned();
        debug!(
            "TableInput connecting to '{}'",
            Self::mask_url_password(&url)
        );

        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect(&url)
            .await
            .map_err(|e| AjisaiError::Config(format!("DB connect failed: {}", e)))?;

        let sql = &self.config.sql;
        // The SQL is an explicit project configuration string; SQLx 0.9
        // requires an explicit audit marker for dynamic statements.
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql.clone()))
            .fetch_all(&pool)
            .await
            .map_err(|e| AjisaiError::Pipeline(format!("Query failed: {}", e)))?;

        if rows.is_empty() {
            return Ok(());
        }

        // Build schema from first row
        let first = &rows[0];
        let fields: Vec<Field> = first
            .columns()
            .iter()
            .map(|col| {
                let type_name = col.type_info().name().to_lowercase();
                let vt = if type_name.contains("int") {
                    ValueType::Integer
                } else if type_name.contains("real")
                    || type_name.contains("float")
                    || type_name.contains("double")
                    || type_name.contains("numeric")
                {
                    ValueType::Float
                } else if type_name.contains("bool") {
                    ValueType::Boolean
                } else {
                    ValueType::String
                };
                Field::new(col.name(), vt)
            })
            .collect();
        let schema = Arc::new(RowSchema::new(fields));

        let limit = self.config.limit;
        for (i, db_row) in rows.iter().enumerate() {
            if limit > 0 && i as u64 >= limit {
                break;
            }
            let values: Vec<Value> = (0..db_row.len())
                .map(|idx| Self::sqlx_to_value(db_row, idx))
                .collect();
            let row = Row::new(schema.clone(), values);
            sender
                .send(row)
                .await
                .map_err(|_| AjisaiError::Pipeline("Downstream closed".into()))?;
        }

        pool.close().await;
        Ok(())
    }
}
