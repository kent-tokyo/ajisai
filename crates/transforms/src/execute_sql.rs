use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::AnyPool;
use tracing::info;

/// ExecuteSQL runs one or more DDL/DML statements against a database.
/// It is a pass-through transform: all input rows are forwarded unchanged.
/// The SQL is executed once during close() after all rows have been processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteSQLConfig {
    /// JDBC-style connection URL
    pub connection_url: String,
    /// SQL statement to execute (DDL or DML — no result rows expected)
    pub sql: String,
    /// Execute once per row instead of once at close
    #[serde(default)]
    pub execute_per_row: bool,
}

pub struct ExecuteSQL {
    config: ExecuteSQLConfig,
    resolved_url: Option<String>,
    pool: Option<AnyPool>,
}

impl ExecuteSQL {
    pub fn new(config: ExecuteSQLConfig) -> Self {
        Self {
            config,
            resolved_url: None,
            pool: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ExecuteSQLConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    async fn get_pool(&mut self) -> Result<&AnyPool> {
        if self.pool.is_none() {
            let url = self
                .resolved_url
                .as_deref()
                .unwrap_or(&self.config.connection_url);
            sqlx::any::install_default_drivers();
            let pool = AnyPool::connect(url)
                .await
                .map_err(|e| AjisaiError::Config(format!("DB connect failed: {}", e)))?;
            self.pool = Some(pool);
        }
        Ok(self.pool.as_ref().unwrap())
    }

    async fn run_sql(&mut self) -> Result<()> {
        let sql = self.config.sql.clone();
        let pool = self.get_pool().await?;
        // ExecuteSQL is intentionally an explicit user-authored statement;
        // SQLx's audit wrapper keeps that boundary visible in code review.
        let result = sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(pool)
            .await
            .map_err(|e| AjisaiError::Pipeline(format!("ExecuteSQL failed: {}", e)))?;
        info!("ExecuteSQL: {} rows affected", result.rows_affected());
        Ok(())
    }
}

#[async_trait]
impl Transform for ExecuteSQL {
    fn name(&self) -> &str {
        "ExecuteSQL"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_url = Some(ctx.resolve(&self.config.connection_url));
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        if self.config.execute_per_row {
            self.run_sql().await?;
        }
        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        if !self.config.execute_per_row {
            self.run_sql().await?;
        }
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }
}
