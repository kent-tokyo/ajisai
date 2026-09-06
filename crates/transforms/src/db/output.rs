use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema, Value},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::AnyPool;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum WriteMode {
    /// INSERT INTO
    #[default]
    Insert,
    /// INSERT OR REPLACE / ON CONFLICT DO UPDATE (SQLite / PostgreSQL)
    Upsert,
    /// TRUNCATE then INSERT
    Overwrite,
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
    needs_delete: bool,
}

/// Mask the password in a connection URL for safe logging.
fn mask_url_password(url: &str) -> String {
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
            needs_delete: false,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: TableOutputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    async fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() && !self.needs_delete {
            return Ok(());
        }
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| AjisaiError::Pipeline("TableOutput not opened".into()))?;

        validate_sql_identifier(&self.config.table)?;

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;

        // Execute deferred DELETE inside the transaction on the first flush
        if self.needs_delete {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "DELETE FROM {}",
                self.config.table
            )))
            .execute(&mut *tx)
            .await
            .map_err(|e| AjisaiError::Pipeline(e.to_string()))?;
            self.needs_delete = false;
        }

        if !self.buffer.is_empty() {
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

            for row in self.buffer.drain(..) {
                // Table and column identifiers are validated above; values use
                // bind parameters, so this dynamic statement is SQL-safe.
                let mut q = sqlx::query(sqlx::AssertSqlSafe(sql.clone()));
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

        if !ctx.network_allowed()
            && !url.to_ascii_lowercase().starts_with("sqlite:")
            && !url.to_ascii_lowercase().starts_with("sqlite::memory:")
        {
            return Err(AjisaiError::Config(
                "Network access disabled by execution policy".into(),
            ));
        }

        // Upsert mode is only supported for SQLite
        if matches!(self.config.mode, WriteMode::Upsert)
            && !url.to_lowercase().starts_with("sqlite")
        {
            return Err(AjisaiError::Config(
                "Upsert is only supported for SQLite databases".into(),
            ));
        }

        debug!("TableOutput connecting to '{}'", mask_url_password(&url));
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect(&url)
            .await
            .map_err(|e| AjisaiError::Config(format!("DB connect failed: {}", e)))?;

        if matches!(self.config.mode, WriteMode::Overwrite) {
            validate_sql_identifier(&self.config.table)?;
            self.needs_delete = true;
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

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, ValueType};
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn row(id: i64) -> Row {
        Row::new(
            Arc::new(RowSchema::new(vec![Field::new("id", ValueType::Integer)])),
            vec![Value::Int(id)],
        )
    }

    #[test]
    fn sql_identifier_policy_rejects_injection_tokens() {
        assert!(validate_sql_identifier("events").is_ok());
        assert!(validate_sql_identifier("events; DROP TABLE users").is_err());
        assert!(validate_sql_identifier("1events").is_err());
    }

    #[tokio::test]
    async fn network_policy_blocks_remote_database_before_connect() {
        let mut output = TableOutput::new(TableOutputConfig {
            connection_url: "postgres://user:pass@example.invalid/db".into(),
            table: "events".into(),
            mode: WriteMode::Insert,
            batch_size: 0,
        });
        let mut context = ExecutionContext::new();
        context.set_network_allowed(false);
        let error = output.open(&context).await.unwrap_err();
        assert!(error.to_string().contains("Network access disabled"));
    }

    #[tokio::test]
    async fn sqlite_commit_at_close_persists_all_rows() {
        sqlx::any::install_default_drivers();
        let db = NamedTempFile::new().unwrap();
        let url = format!("sqlite://{}", db.path().display());
        let pool = AnyPool::connect(&url).await.unwrap();
        sqlx::query("CREATE TABLE events (id INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        let mut output = TableOutput::new(TableOutputConfig {
            connection_url: url.clone(),
            table: "events".into(),
            mode: WriteMode::Insert,
            batch_size: 0,
        });
        output.open(&ExecutionContext::new()).await.unwrap();
        output.process(row(1)).await.unwrap();
        output.process(row(2)).await.unwrap();
        output.close().await.unwrap();

        let verify = AnyPool::connect(&url).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&verify)
            .await
            .unwrap();
        assert_eq!(count, 2);
        verify.close().await;
    }
}
