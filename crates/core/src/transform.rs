use crate::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
};
use async_trait::async_trait;
use std::sync::Arc;

/// Runtime interface every transform must implement
#[async_trait]
pub trait Transform: Send {
    fn name(&self) -> &str;

    /// Derive output schema from input schema (called before execution for validation)
    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema>;

    /// Initialize resources (open files, DB connections, etc.)
    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()>;

    /// Process one input row, returning 0..N output rows.
    /// - Filter:   0 or 1
    /// - Normal:   1
    /// - Explode:  N
    async fn process(&mut self, row: Row) -> Result<Vec<Row>>;

    /// Called by the engine after all process() calls finish.
    /// Buffering transforms (e.g., SortRows) emit their accumulated output here.
    async fn flush(&mut self) -> Result<Vec<Row>> {
        Ok(vec![])
    }

    /// Flush buffered data and release resources
    async fn close(&mut self) -> Result<()>;

    /// Source transforms (CSV Input, DB Input) generate rows rather than processing them
    fn is_source(&self) -> bool {
        false
    }

    /// Called only when `is_source()` returns true.
    /// Sends all produced rows to the channel, then drops the sender to signal EOF.
    async fn produce(&mut self, sender: tokio::sync::mpsc::Sender<Row>) -> Result<()> {
        let _ = sender;
        Ok(())
    }

    /// For join/lookup transforms: how many of this node's input channels are "side inputs"
    /// that must be fully pre-loaded before main-stream processing begins.
    /// Side inputs are always the *last* N incoming hops in the pipeline.
    fn side_input_count(&self) -> usize {
        0
    }

    /// Called by the engine with all rows from side-input channel `idx`,
    /// before any main-stream process() calls begin.
    async fn load_side_input(&mut self, _idx: usize, _rows: Vec<Row>) -> Result<()> {
        Ok(())
    }

    /// Routing transforms (e.g. SwitchCase) override this to direct each output row
    /// to a specific downstream node by name.
    /// Return `None` to broadcast the row to all connected downstream nodes (default).
    /// Return `Some(target_node_id)` to send only to that node.
    fn route(&self, _row: &Row) -> Option<String> {
        None
    }
}

/// Prototype factory: given a serialized config, construct a boxed Transform
pub type TransformFactory =
    Arc<dyn Fn(serde_json::Value) -> Result<Box<dyn Transform>> + Send + Sync>;
