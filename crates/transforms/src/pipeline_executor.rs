use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, PipelineEngine, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecutorConfig {
    /// Path to the .hpl sub-pipeline file to execute.
    pub sub_pipeline_path: String,
    /// When true, the parent pipeline's variables are passed into the sub-pipeline context.
    #[serde(default)]
    pub inherit_variables: bool,
}

/// Runs a sub-pipeline (.hpl) once after all rows have passed through this node.
/// Rows are passed through unchanged; the sub-pipeline runs independently in `close()`.
/// The sub-pipeline must have its own source nodes.
pub struct PipelineExecutor {
    config: PipelineExecutorConfig,
    ctx: Option<ExecutionContext>,
    buffered_rows: Vec<Row>,
    output_schema: Option<Arc<RowSchema>>,
}

impl PipelineExecutor {
    pub fn new(config: PipelineExecutorConfig) -> Self {
        Self { config, ctx: None, buffered_rows: Vec::new(), output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: PipelineExecutorConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for PipelineExecutor {
    fn name(&self) -> &str {
        "PipelineExecutor"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.ctx = Some(ctx.clone());
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        self.output_schema.get_or_insert_with(|| Arc::clone(&row.schema));
        self.buffered_rows.push(row);
        Ok(vec![])
    }

    async fn flush(&mut self) -> Result<Vec<Row>> {
        Ok(std::mem::take(&mut self.buffered_rows))
    }

    async fn close(&mut self) -> Result<()> {
        let ctx = self.ctx.clone().unwrap_or_default();
        let resolved_path = ctx.resolve(&self.config.sub_pipeline_path);

        let path = crate::utils::resolve_safe_path(&resolved_path)?;
        let xml = std::fs::read_to_string(&path).map_err(AjisaiError::Io)?;

        let hop_pipeline = ajisai_hop_compat::parse_hpl(&xml)?;

        let registry = crate::default_registry();
        let pipeline = ajisai_hop_compat::hop_pipeline_to_ajisai(hop_pipeline, &registry)?;

        let sub_ctx = if self.config.inherit_variables {
            ctx
        } else {
            ExecutionContext::new()
        };

        PipelineEngine::new(pipeline, sub_ctx)
            .run()
            .await
            .map(|_| ())
            .map_err(|e| AjisaiError::Pipeline(format!("Sub-pipeline failed: {}", e)))
    }
}
