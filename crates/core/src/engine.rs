use crate::{
    context::ExecutionContext,
    error::{AjisaiError, Result},
    pipeline::Pipeline,
    value::Row,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

const CHANNEL_BUFFER: usize = 1024;

#[derive(Debug, Default)]
pub struct ExecutionStats {
    pub rows_read: u64,
    pub rows_written: u64,
    pub elapsed_ms: u64,
}

pub struct PipelineEngine {
    pub pipeline: Pipeline,
    pub context: ExecutionContext,
}

impl PipelineEngine {
    pub fn new(pipeline: Pipeline, context: ExecutionContext) -> Self {
        Self { pipeline, context }
    }

    /// Execute the pipeline end-to-end.
    ///
    /// Topology: each node runs as a tokio task.
    /// Nodes communicate via bounded mpsc channels (backpressure built-in).
    ///
    /// For join/lookup transforms that declare `side_input_count() > 0`:
    ///   - The last N incoming hops are treated as side inputs.
    ///   - All rows from those channels are collected in memory first,
    ///     then passed to `load_side_input()` before main-stream processing begins.
    ///
    /// After all main-stream input is drained, `flush()` is called to emit
    /// any buffered rows (e.g., SortRows emits sorted output here).
    pub async fn run(self) -> Result<ExecutionStats> {
        let start = std::time::Instant::now();
        let order = self.pipeline.topological_order();

        if order.is_empty() {
            return Err(AjisaiError::Pipeline("Pipeline has no nodes".into()));
        }

        if order.len() < self.pipeline.nodes.len() {
            return Err(AjisaiError::Pipeline(
                "Cycle detected in pipeline graph".into(),
            ));
        }

        info!(
            "Running pipeline '{}' ({} nodes)",
            self.pipeline.name,
            order.len()
        );

        // Build channels: one sender/receiver pair per directed hop edge.
        let mut senders: HashMap<(String, String), mpsc::Sender<Row>> = HashMap::new();
        let mut receivers: HashMap<(String, String), mpsc::Receiver<Row>> = HashMap::new();

        for hop in &self.pipeline.hops {
            let (tx, rx) = mpsc::channel::<Row>(CHANNEL_BUFFER);
            senders.insert((hop.from.clone(), hop.to.clone()), tx);
            receivers.insert((hop.from.clone(), hop.to.clone()), rx);
        }

        let mut handles = Vec::new();
        let context = self.context.clone();

        for mut node in self.pipeline.nodes {
            let node_id = node.id.clone();
            let ctx = context.clone();

            // Outgoing senders for this node
            let out_senders: Vec<mpsc::Sender<Row>> = self
                .pipeline
                .hops
                .iter()
                .filter(|h| h.from == node_id)
                .filter_map(|h| senders.remove(&(h.from.clone(), h.to.clone())))
                .collect();

            // Incoming receivers in hop-declaration order (side inputs are the last N)
            let mut in_receivers: Vec<mpsc::Receiver<Row>> = self
                .pipeline
                .hops
                .iter()
                .filter(|h| h.to == node_id)
                .filter_map(|h| receivers.remove(&(h.from.clone(), h.to.clone())))
                .collect();

            let is_source = node.transform.is_source();
            let side_count = node.transform.side_input_count();

            let handle = tokio::spawn(async move {
                node.transform.open(&ctx).await?;

                if is_source {
                    // Source: produce rows → fan-out to all successors
                    let (produce_tx, mut produce_rx) = mpsc::channel::<Row>(CHANNEL_BUFFER);
                    let produce_fut = node.transform.produce(produce_tx);

                    let fan_out = async move {
                        while let Some(row) = produce_rx.recv().await {
                            for tx in &out_senders {
                                if tx.send(row.clone()).await.is_err() {
                                    break;
                                }
                            }
                        }
                    };

                    tokio::try_join!(produce_fut, async {
                        fan_out.await;
                        Ok(())
                    })?;
                } else {
                    // Split side inputs from main inputs.
                    // Side inputs are the *last* `side_count` receivers.
                    let split_at = in_receivers.len().saturating_sub(side_count);
                    let side_receivers: Vec<_> = in_receivers.drain(split_at..).collect();
                    let main_receivers = in_receivers;

                    // Pre-collect side inputs and load them into the transform.
                    // Running concurrently with the upstream source tasks, so no deadlock.
                    for (idx, mut rx) in side_receivers.into_iter().enumerate() {
                        let mut side_rows: Vec<Row> = Vec::new();
                        while let Some(row) = rx.recv().await {
                            side_rows.push(row);
                        }
                        node.transform.load_side_input(idx, side_rows).await?;
                    }

                    // Process main input stream
                    for mut rx in main_receivers {
                        while let Some(row) = rx.recv().await {
                            let out_rows = node.transform.process(row).await?;
                            for out_row in out_rows {
                                for tx in &out_senders {
                                    if tx.send(out_row.clone()).await.is_err() {
                                        return Err(AjisaiError::Pipeline(
                                            "Downstream channel closed".into(),
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    // Flush buffered output (e.g., SortRows emits here)
                    let flush_rows = node.transform.flush().await?;
                    for out_row in flush_rows {
                        for tx in &out_senders {
                            if tx.send(out_row.clone()).await.is_err() {
                                return Err(AjisaiError::Pipeline(
                                    "Downstream channel closed".into(),
                                ));
                            }
                        }
                    }
                }

                node.transform.close().await?;
                debug!("Node '{}' finished", node_id);
                Ok::<(), AjisaiError>(())
            });

            handles.push(handle);
        }

        let mut rows_processed = 0u64;
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => rows_processed += 1,
                Ok(Err(e)) => {
                    error!("Pipeline node failed: {}", e);
                    return Err(e);
                }
                Err(join_err) => {
                    return Err(AjisaiError::Pipeline(format!(
                        "Task panicked: {}",
                        join_err
                    )));
                }
            }
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        info!(
            "Pipeline '{}' completed in {}ms ({} nodes)",
            self.pipeline.name, elapsed_ms, rows_processed
        );

        Ok(ExecutionStats {
            rows_read: 0,
            rows_written: 0,
            elapsed_ms,
        })
    }
}
