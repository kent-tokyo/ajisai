use crate::{
    context::{ExecutionContext, RunEvent},
    error::{AjisaiError, Result},
    pipeline::Pipeline,
    value::{Field, Row, RowSchema, Value, ValueType},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Build an error row from the original row plus two error metadata fields.
fn make_error_row(original: Row, err: &AjisaiError) -> Row {
    let mut fields = original.schema.fields.clone();
    fields.push(Field::new("__error_desc", ValueType::String));
    fields.push(Field::new("__error_code", ValueType::String));
    let error_schema = Arc::new(RowSchema::new(fields));

    let mut values = original.values.clone();
    values.push(Value::Str(err.to_string()));
    values.push(Value::Str(error_code(err).into()));
    Row::new(error_schema, values)
}

fn error_code(err: &AjisaiError) -> &'static str {
    match err {
        AjisaiError::Parse(_) => "PARSE_ERROR",
        AjisaiError::Io(_) => "IO_ERROR",
        AjisaiError::Config(_) => "CONFIG_ERROR",
        AjisaiError::Pipeline(_) => "PIPELINE_ERROR",
        _ => "ERROR",
    }
}

/// Send a row to outgoing channels.
/// - `target = None`  → broadcast to all senders
/// - `target = Some(id)` → send only to the matching sender; error if not found
async fn send_row(
    row: &Row,
    target: Option<&str>,
    senders: &[(String, mpsc::Sender<Row>)],
) -> Result<()> {
    match target {
        None => {
            for (_, tx) in senders {
                if tx.send(row.clone()).await.is_err() {
                    return Err(AjisaiError::Pipeline("Downstream channel closed".into()));
                }
            }
        }
        Some(t) => {
            let tx = senders
                .iter()
                .find(|(id, _)| id == t)
                .map(|(_, tx)| tx)
                .ok_or_else(|| {
                    AjisaiError::Pipeline(format!("No route found for target '{}'", t))
                })?;
            if tx.send(row.clone()).await.is_err() {
                return Err(AjisaiError::Pipeline("Downstream channel closed".into()));
            }
        }
    }
    Ok(())
}

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
        let pipeline_name = self.pipeline.name.clone();
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
            pipeline_name,
            order.len()
        );
        self.context.record_event(RunEvent {
            schema_version: 1,
            kind: "run_started".into(),
            pipeline: self.pipeline.name.clone(),
            node_id: None,
            status: "running".into(),
            rows: None,
            elapsed_ms: Some(0),
        });

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
        let rows_read = Arc::new(AtomicU64::new(0));
        let rows_written = Arc::new(AtomicU64::new(0));

        for mut node in self.pipeline.nodes {
            let node_id = node.id.clone();
            let ctx = context.clone();
            let read_counter = rows_read.clone();
            let write_counter = rows_written.clone();
            let pipeline_name = self.pipeline.name.clone();

            // Outgoing senders split into normal and error hops
            let mut out_senders: Vec<(String, mpsc::Sender<Row>)> = Vec::new();
            let mut error_senders: Vec<(String, mpsc::Sender<Row>)> = Vec::new();
            for h in self.pipeline.hops.iter().filter(|h| h.from == node_id) {
                if let Some(tx) = senders.remove(&(h.from.clone(), h.to.clone())) {
                    if h.is_error {
                        error_senders.push((h.to.clone(), tx));
                    } else {
                        out_senders.push((h.to.clone(), tx));
                    }
                }
            }

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
                ctx.record_event(RunEvent {
                    schema_version: 1,
                    kind: "node_started".into(),
                    pipeline: pipeline_name.clone(),
                    node_id: Some(node_id.clone()),
                    status: "running".into(),
                    rows: None,
                    elapsed_ms: None,
                });
                if ctx.is_cancelled() {
                    return Err(AjisaiError::Pipeline("Pipeline cancelled".into()));
                }
                if ctx.is_timed_out() {
                    return Err(AjisaiError::Pipeline(
                        "Execution time limit exceeded".into(),
                    ));
                }
                node.transform.open(&ctx).await?;

                let event_ctx = ctx.clone();
                let execution_result: Result<()> = async {
                    if is_source {
                        // Source: produce rows → fan-out to all successors
                        let (produce_tx, mut produce_rx) = mpsc::channel::<Row>(CHANNEL_BUFFER);
                        let produce_fut = node.transform.produce(produce_tx);

                        let fan_out = async move {
                            while let Some(row) = produce_rx.recv().await {
                                if ctx.is_cancelled() {
                                    return Err(AjisaiError::Pipeline("Pipeline cancelled".into()));
                                }
                                if ctx.is_timed_out() {
                                    return Err(AjisaiError::Pipeline(
                                        "Execution time limit exceeded".into(),
                                    ));
                                }
                                if !ctx.try_accept_row() {
                                    return Err(AjisaiError::Pipeline("Row limit exceeded".into()));
                                }
                                read_counter.fetch_add(1, Ordering::Relaxed);
                                send_row(&row, None, &out_senders).await?;
                            }
                            Ok::<(), AjisaiError>(())
                        };

                        tokio::try_join!(produce_fut, fan_out)?;
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
                                if !ctx.try_buffer_row() {
                                    return Err(AjisaiError::Pipeline(
                                        "Buffered row limit exceeded".into(),
                                    ));
                                }
                                side_rows.push(row);
                            }
                            node.transform.load_side_input(idx, side_rows).await?;
                        }

                        // Process main input stream
                        for mut rx in main_receivers {
                            while let Some(row) = rx.recv().await {
                                if ctx.is_cancelled() {
                                    return Err(AjisaiError::Pipeline("Pipeline cancelled".into()));
                                }
                                match node.transform.process(row.clone()).await {
                                    Ok(out_rows) => {
                                        for out_row in out_rows {
                                            if out_senders.is_empty() {
                                                write_counter.fetch_add(1, Ordering::Relaxed);
                                            }
                                            send_row(
                                                &out_row,
                                                node.transform.route(&out_row).as_deref(),
                                                &out_senders,
                                            )
                                            .await?;
                                        }
                                    }
                                    Err(e) if !error_senders.is_empty() => {
                                        let error_row = make_error_row(row, &e);
                                        send_row(&error_row, None, &error_senders).await?;
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                        }

                        // Flush buffered output (e.g., SortRows emits here)
                        let flush_rows = node.transform.flush().await?;
                        for out_row in flush_rows {
                            if out_senders.is_empty() {
                                write_counter.fetch_add(1, Ordering::Relaxed);
                            }
                            send_row(
                                &out_row,
                                node.transform.route(&out_row).as_deref(),
                                &out_senders,
                            )
                            .await?;
                        }
                    }
                    Ok(())
                }
                .await;

                let close_result = node.transform.close().await;
                if let Err(error) = execution_result {
                    event_ctx.record_event(RunEvent {
                        schema_version: 1,
                        kind: "node_finished".into(),
                        pipeline: pipeline_name.clone(),
                        node_id: Some(node_id.clone()),
                        status: "failed".into(),
                        rows: None,
                        elapsed_ms: None,
                    });
                    return Err(error);
                }
                if let Err(error) = close_result {
                    event_ctx.record_event(RunEvent {
                        schema_version: 1,
                        kind: "node_finished".into(),
                        pipeline: pipeline_name.clone(),
                        node_id: Some(node_id.clone()),
                        status: "failed".into(),
                        rows: None,
                        elapsed_ms: None,
                    });
                    return Err(error);
                }
                debug!("Node '{}' finished", node_id);
                event_ctx.record_event(RunEvent {
                    schema_version: 1,
                    kind: "node_finished".into(),
                    pipeline: pipeline_name,
                    node_id: Some(node_id),
                    status: "succeeded".into(),
                    rows: None,
                    elapsed_ms: None,
                });
                Ok::<(), AjisaiError>(())
            });

            handles.push(handle);
        }

        let mut rows_processed = 0u64;
        let mut handles = handles;
        while let Some(handle) = handles.pop() {
            match handle.await {
                Ok(Ok(())) => rows_processed += 1,
                Ok(Err(e)) => {
                    for pending in handles {
                        pending.abort();
                    }
                    error!("Pipeline node failed: {}", e);
                    context.record_event(RunEvent {
                        schema_version: 1,
                        kind: "run_finished".into(),
                        pipeline: pipeline_name.clone(),
                        node_id: None,
                        status: "failed".into(),
                        rows: None,
                        elapsed_ms: Some(start.elapsed().as_millis() as u64),
                    });
                    return Err(e);
                }
                Err(join_err) => {
                    for pending in handles {
                        pending.abort();
                    }
                    context.record_event(RunEvent {
                        schema_version: 1,
                        kind: "run_finished".into(),
                        pipeline: pipeline_name.clone(),
                        node_id: None,
                        status: "failed".into(),
                        rows: None,
                        elapsed_ms: Some(start.elapsed().as_millis() as u64),
                    });
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
            pipeline_name, elapsed_ms, rows_processed
        );

        let stats = ExecutionStats {
            rows_read: rows_read.load(Ordering::Relaxed),
            rows_written: rows_written.load(Ordering::Relaxed),
            elapsed_ms,
        };
        context.record_event(RunEvent {
            schema_version: 1,
            kind: "run_finished".into(),
            pipeline: pipeline_name,
            node_id: None,
            status: "succeeded".into(),
            rows: Some(stats.rows_written),
            elapsed_ms: Some(stats.elapsed_ms),
        });
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct PanicSource;

    #[async_trait]
    impl crate::Transform for PanicSource {
        fn name(&self) -> &str {
            "PanicSource"
        }
        fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
            Ok(input.clone())
        }
        async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
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
        async fn produce(&mut self, _sender: mpsc::Sender<Row>) -> Result<()> {
            panic!("intentional source panic")
        }
    }

    #[tokio::test]
    async fn source_panic_is_reported_as_pipeline_error() {
        let mut pipeline = Pipeline::new("panic-test");
        pipeline.add_node("panic", Box::new(PanicSource));
        let error = PipelineEngine::new(pipeline, ExecutionContext::new())
            .run()
            .await
            .expect_err("panic source must fail the run");
        assert!(error.to_string().contains("Task panicked"));
    }

    struct OneRowSource;

    #[async_trait]
    impl crate::Transform for OneRowSource {
        fn name(&self) -> &str {
            "OneRowSource"
        }
        fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
            Ok(input.clone())
        }
        async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
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
            let schema = Arc::new(RowSchema::new(vec![Field::new(
                "value",
                ValueType::Integer,
            )]));
            sender
                .send(Row::new(schema, vec![Value::Int(1)]))
                .await
                .map_err(|_| AjisaiError::Pipeline("receiver closed".into()))
        }
    }

    #[tokio::test]
    async fn emits_versioned_run_and_node_events() {
        let mut pipeline = Pipeline::new("events-test");
        pipeline.add_node("source", Box::new(OneRowSource));
        let context = ExecutionContext::new();
        PipelineEngine::new(pipeline, context.clone())
            .run()
            .await
            .unwrap();
        let events = context.events();
        assert_eq!(
            events.first().map(|event| event.kind.as_str()),
            Some("run_started")
        );
        assert!(events.iter().any(
            |event| event.kind == "node_started" && event.node_id.as_deref() == Some("source")
        ));
        assert_eq!(
            events.last().map(|event| event.kind.as_str()),
            Some("run_finished")
        );
        assert!(events.iter().all(|event| event.schema_version == 1));
    }

    struct ManyRowSource {
        count: u64,
    }

    #[async_trait]
    impl crate::Transform for ManyRowSource {
        fn name(&self) -> &str {
            "ManyRowSource"
        }
        fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
            Ok(input.clone())
        }
        async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
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
            let schema = Arc::new(RowSchema::new(vec![Field::new(
                "value",
                ValueType::Integer,
            )]));
            for value in 0..self.count {
                sender
                    .send(Row::new(schema.clone(), vec![Value::Int(value as i64)]))
                    .await
                    .map_err(|_| AjisaiError::Pipeline("receiver closed".into()))?;
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn large_source_stops_at_row_budget_without_unbounded_buffering() {
        let mut pipeline = Pipeline::new("large-budget-test");
        pipeline.add_node("source", Box::new(ManyRowSource { count: 100_000 }));
        let mut context = ExecutionContext::new();
        context.set_row_limit(Some(32));

        let error = PipelineEngine::new(pipeline, context)
            .run()
            .await
            .expect_err("large source must stop at the configured row budget");
        assert!(error.to_string().contains("Row limit exceeded"));
    }

    struct FailAfter {
        seen: u64,
        fail_at: u64,
    }

    #[async_trait]
    impl crate::Transform for FailAfter {
        fn name(&self) -> &str {
            "FailAfter"
        }
        fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
            Ok(input.clone())
        }
        async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
            Ok(())
        }
        async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
            self.seen += 1;
            if self.seen >= self.fail_at {
                return Err(AjisaiError::Pipeline("injected transform failure".into()));
            }
            Ok(vec![row])
        }
        async fn close(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn injected_failure_stops_large_source_and_records_terminal_failure() {
        let mut pipeline = Pipeline::new("injected-failure-test");
        pipeline.add_node("source", Box::new(ManyRowSource { count: 100_000 }));
        pipeline.add_node(
            "fail",
            Box::new(FailAfter {
                seen: 0,
                fail_at: 8,
            }),
        );
        pipeline.add_hop("source", "fail");
        let context = ExecutionContext::new();

        let error = PipelineEngine::new(pipeline, context.clone())
            .run()
            .await
            .expect_err("injected transform failure must fail the run");
        assert!(error.to_string().contains("injected transform failure"));
        assert!(context.events().iter().any(|event| {
            event.kind == "node_finished"
                && event.node_id.as_deref() == Some("fail")
                && event.status == "failed"
        }));
        assert_eq!(
            context.events().last().map(|event| event.status.as_str()),
            Some("failed")
        );
    }

    struct TwoRowSource;

    #[async_trait]
    impl crate::Transform for TwoRowSource {
        fn name(&self) -> &str {
            "TwoRowSource"
        }
        fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
            Ok(input.clone())
        }
        async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
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
            let schema = Arc::new(RowSchema::new(vec![Field::new(
                "value",
                ValueType::Integer,
            )]));
            for value in [1_i64, 2] {
                sender
                    .send(Row::new(schema.clone(), vec![Value::Int(value)]))
                    .await
                    .map_err(|_| AjisaiError::Pipeline("receiver closed".into()))?;
            }
            Ok(())
        }
    }

    struct SideCollector;

    #[async_trait]
    impl crate::Transform for SideCollector {
        fn name(&self) -> &str {
            "SideCollector"
        }
        fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
            Ok(input.clone())
        }
        async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
            Ok(())
        }
        async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
            Ok(vec![row])
        }
        async fn close(&mut self) -> Result<()> {
            Ok(())
        }
        fn side_input_count(&self) -> usize {
            1
        }
        async fn load_side_input(&mut self, _idx: usize, _rows: Vec<Row>) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn rejects_side_input_when_buffer_policy_is_exceeded() {
        let mut pipeline = Pipeline::new("buffer-limit-test");
        pipeline.add_node("main", Box::new(OneRowSource));
        pipeline.add_node("side", Box::new(TwoRowSource));
        pipeline.add_node("join", Box::new(SideCollector));
        pipeline.add_hop("main", "join");
        pipeline.add_hop("side", "join");
        let mut context = ExecutionContext::new();
        context.set_buffered_row_limit(Some(1));
        let error = PipelineEngine::new(pipeline, context)
            .run()
            .await
            .expect_err("side input must respect the buffer policy");
        assert!(error.to_string().contains("Buffered row limit exceeded"));
    }
}
