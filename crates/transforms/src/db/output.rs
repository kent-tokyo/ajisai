use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::AnyPool;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    /// INSERT INTO
    Insert,
    /// INSERT OR REPLACE / ON CONFLICT DO UPDATE (SQLite / PostgreSQL)
    Upsert,
    /// TRUNCATE then INSERT
    Overwrite,
}

impl Default for WriteMode {
    fn default() -> Self {
        WriteMode::Insert
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOutputConfig {
    pub connection_url: String,
    pub table: String,
    #[serde(default)]
    pub mode: WriteMode,
    /// Commit every N rows (0 = commit at end)
    #[serde(default)]
    pub batch_size: usize,
}

pub struct TableOutput {
    config: TableOutputConfig,
    pool: Option<AnyPool>,
    buffer: Vec<Row>,
}

/// Validate that a string is a safe SQL identifier (letters, digits, underscores only;
/// must not start with a digit). This prevents SQL injection via table/column names.
fn validate_sql_identifier(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(AjisaiError::Config("SQL identifier cannot be empty".into()));
    }
    if name
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        return Err(AjisaiError::Config(format!(
            "SQL identifier '{}' must not start with a digit",
            name
        )));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AjisaiError::Config(format!(
            "SQL identifier '{}' contains invalid characters \
             (only ASCII letters, digits, and underscores are allowed)",
            name
        )));
    }
    Ok(())
}

impl TableOutput {
    pub fn new(config: TableOutputConfig) -> Self {
        Self {
            config,
            pool: None,
            buffer: Vec::new(),
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: TableOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    async fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| AjisaiError::Pipeline("TableOutput not opened".into()))?;

        validate_sql_identifier(&self.config.table)?;

        let first = &self.buffer[0];
        let cols: Vec<&str> = first
            .schema
            .fields
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        for col in &cols {
            validate_sql_identifier(col)?;
        }
        let placeholders: String = (1..=cols.len())
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");

        let verb = match self.config.mode {
            WriteMode::Insert => "INSERT INTO",
            WriteMode::Upsert => "INSERT OR REPLACE INTO",
            WriteMode::Overwrite => "INSERT INTO",
        };

        let sql = format!(
            "{} {} ({}) VALUES ({})",
            verb,
            self.config.table,
            cols.join(", "),
            placeholders
        );

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;

        for row in self.buffer.drain(..) {
            let mut q = sqlx::query(&sql);
            for val in &row.values {
                q = match val {
                    Value::Int(n) => q.bind(*n),
                    Value::Float(f) => q.bind(*f),
                    Value::Bool(b) => q.bind(*b),
                    Value::Null => q.bind(Option::<String>::None),
                    other => q.bind(other.to_display_string()),
                };
            }
            q.execute(&mut *tx)
                .await
                .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl Transform for TableOutput {
    fn name(&self) -> &str {
        "TableOutput"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        let url = ctx.resolve(&self.config.connection_url);
        debug!("TableOutput connecting to '{}'", url);
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect(&url)
            .await
            .map_err(|e| AjisaiError::Config(format!("DB connect failed: {}", e)))?;

        if matches!(self.config.mode, WriteMode::Overwrite) {
            validate_sql_identifier(&self.config.table)?;
            sqlx::query(&format!("DELETE FROM {}", self.config.table))
                .execute(&pool)
                .await
                .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;
        }

        self.pool = Some(pool);
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        self.buffer.push(row.clone());

        let batch = self.config.batch_size;
        if batch > 0 && self.buffer.len() >= batch {
            self.flush().await?;
        }

        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        self.flush().await?;
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }
}
