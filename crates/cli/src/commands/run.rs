use crate::spinner::Spinner;
use ajisai_core::{
    ExecutionContext, ExecutionStats, NativePipelineDocument, Pipeline, PipelineEngine,
};
use ajisai_hop_compat::{hop_pipeline_to_ajisai, load_pipeline_file};
use ajisai_transforms::default_registry;
use rust_i18n::t;
use std::fs;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tracing::error;

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub max_rows: Option<u64>,
    pub max_buffered_rows: Option<u64>,
    pub timeout_secs: Option<u64>,
    pub no_network: bool,
    pub json_output: bool,
    pub dry_run: bool,
    pub record_path: Option<PathBuf>,
    pub run_id: String,
    pub project_root: Option<PathBuf>,
}

pub async fn run(
    pipeline_path: PathBuf,
    env_vars: Vec<String>,
    options: RunOptions,
) -> anyhow::Result<()> {
    let mut ctx = ExecutionContext::new();
    ctx.set_row_limit(options.max_rows);
    ctx.set_buffered_row_limit(options.max_buffered_rows);
    ctx.set_timeout(options.timeout_secs.map(Duration::from_secs));
    ctx.set_network_allowed(!options.no_network);
    if let Some(root) = &options.project_root {
        ctx.set_project_root(root.clone());
    }

    for kv in &env_vars {
        if let Some((k, v)) = kv.split_once('=') {
            ctx.set_var(k.trim(), v.trim());
        } else {
            eprintln!("{}", t!("error.invalid_env", value = kv.as_str()));
        }
    }

    if !pipeline_path.exists() {
        anyhow::bail!(
            "{}",
            t!(
                "error.file_not_found",
                path = pipeline_path.display().to_string().as_str()
            )
        );
    }

    let file_name = pipeline_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if file_name.ends_with(".ajp") || file_name.ends_with(".ajisai.json") {
        return run_native(
            &pipeline_path,
            ctx,
            options.json_output,
            options.dry_run,
            options.record_path.as_deref(),
            &options.run_id,
        )
        .await;
    }

    let ext = pipeline_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !matches!(ext, "hpl" | "ktr" | "dtsx") {
        anyhow::bail!(
            "{}",
            t!(
                "error.unsupported_format",
                path = pipeline_path.display().to_string().as_str()
            )
        );
    }

    let hop_pipeline = load_pipeline_file(&pipeline_path)?;
    let name = hop_pipeline.name.clone();
    let node_count = hop_pipeline.transforms.len();

    if !options.json_output {
        println!("{}", t!("run.start", name = name.as_str()));
    }

    let registry = default_registry();
    let pipeline = hop_pipeline_to_ajisai(hop_pipeline, &registry)?;

    if options.dry_run {
        print_dry_run(
            &name,
            node_count,
            pipeline.hops.len(),
            options.json_output,
            options.record_path.as_deref(),
            &options.run_id,
        )?;
        return Ok(());
    }

    let mut spinner = if options.json_output {
        None
    } else {
        Some(Spinner::new(
            t!("run.nodes", count = node_count.to_string().as_str()).to_string(),
        ))
    };

    let engine = PipelineEngine::new(pipeline, ctx.clone());
    match run_with_ctrl_c(engine, ctx.clone()).await {
        Ok(stats) => {
            if let Some(spinner) = spinner.as_mut() {
                spinner.finish_and_clear();
            }
            if options.json_output {
                print_json_result(
                    &name,
                    &stats,
                    &ctx,
                    options.record_path.as_deref(),
                    &options.run_id,
                )?;
            } else {
                println!(
                    "{} (rows_read={}, rows_written={})",
                    t!(
                        "run.success",
                        name = name.as_str(),
                        ms = stats.elapsed_ms.to_string().as_str()
                    ),
                    stats.rows_read,
                    stats.rows_written
                );
            }
        }
        Err(e) => {
            if let Some(spinner) = spinner.as_mut() {
                spinner.finish_and_clear();
            }
            error!("Pipeline failed: {}", e);
            if e.to_string().to_ascii_lowercase().contains("cancel") {
                anyhow::bail!("Pipeline cancelled by user");
            }
            anyhow::bail!("{}", t!("run.error", error = e.to_string().as_str()));
        }
    }

    Ok(())
}

async fn run_native(
    pipeline_path: &PathBuf,
    ctx: ExecutionContext,
    json_output: bool,
    dry_run: bool,
    record_path: Option<&std::path::Path>,
    run_id: &str,
) -> anyhow::Result<()> {
    let source = fs::read_to_string(pipeline_path)?;
    let document: NativePipelineDocument = serde_json::from_str(&source)
        .map_err(|error| anyhow::anyhow!("Invalid Ajisai native document: {error}"))?;
    let diagnostics = document.validate();
    if !diagnostics.is_empty() {
        for diagnostic in &diagnostics {
            eprintln!(
                "{} [{}] {}",
                diagnostic.code, diagnostic.path, diagnostic.message
            );
        }
        anyhow::bail!(
            "native pipeline validation failed ({} diagnostic(s))",
            diagnostics.len()
        );
    }

    if !json_output {
        println!("{}", t!("run.start", name = document.name.as_str()));
    }
    let registry = default_registry();
    let state = document.to_state();
    let mut pipeline = Pipeline::new(&state.name);
    for node in &state.nodes {
        pipeline.add_node(
            &node.id,
            registry.create(&node.type_name, node.config.clone())?,
        );
    }
    for edge in &state.edges {
        if edge.is_error {
            pipeline.add_error_hop(&edge.from, &edge.to);
        } else {
            pipeline.add_hop(&edge.from, &edge.to);
        }
    }

    if dry_run {
        print_dry_run(
            &document.name,
            pipeline.nodes.len(),
            pipeline.hops.len(),
            json_output,
            record_path,
            run_id,
        )?;
        return Ok(());
    }

    match run_with_ctrl_c(PipelineEngine::new(pipeline, ctx.clone()), ctx.clone()).await {
        Ok(stats) => {
            if json_output {
                print_json_result(&document.name, &stats, &ctx, record_path, run_id)?;
            } else {
                println!(
                    "{} (rows_read={}, rows_written={})",
                    t!(
                        "run.success",
                        name = document.name.as_str(),
                        ms = stats.elapsed_ms.to_string().as_str()
                    ),
                    stats.rows_read,
                    stats.rows_written
                );
            }
        }
        Err(error) => {
            error!("Native pipeline failed: {}", error);
            if error.to_string().to_ascii_lowercase().contains("cancel") {
                anyhow::bail!("Pipeline cancelled by user");
            }
            anyhow::bail!("{}", t!("run.error", error = error.to_string().as_str()));
        }
    }
    Ok(())
}

fn print_json_result(
    name: &str,
    stats: &ExecutionStats,
    context: &ExecutionContext,
    record_path: Option<&std::path::Path>,
    run_id: &str,
) -> anyhow::Result<()> {
    let result = serde_json::json!({
        "schema_version": 1,
        "status": "succeeded",
        "run_id": run_id,
        "pipeline": name,
        "rows_read": stats.rows_read,
        "rows_written": stats.rows_written,
        "elapsed_ms": stats.elapsed_ms,
        "events": context.events(),
    });
    println!("{}", result);
    persist_record(record_path, &result)
}

fn print_dry_run(
    name: &str,
    nodes: usize,
    hops: usize,
    json_output: bool,
    record_path: Option<&std::path::Path>,
    run_id: &str,
) -> anyhow::Result<()> {
    let result = serde_json::json!({
        "schema_version": 1,
        "status": "dry_run",
        "run_id": run_id,
        "pipeline": name,
        "nodes": nodes,
        "hops": hops,
        "side_effects": false,
    });
    if json_output {
        println!("{}", result);
    } else {
        println!("Dry run: {name} (nodes={nodes}, hops={hops}, side_effects=false)");
    }
    persist_record(record_path, &result)
}

pub fn persist_record(
    path: Option<&std::path::Path>,
    value: &serde_json::Value,
) -> anyhow::Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    fs::create_dir_all(parent)?;
    // Keep concurrent CLI invocations from sharing the same staging file.
    // The final rename remains atomic on the target filesystem.
    let temp = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temp, serde_json::to_vec_pretty(value)?)?;
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }
    Ok(())
}

async fn run_with_ctrl_c(
    engine: PipelineEngine,
    context: ExecutionContext,
) -> anyhow::Result<ExecutionStats> {
    run_with_signal(engine, context.clone(), tokio::signal::ctrl_c()).await
}

async fn run_with_signal<F>(
    engine: PipelineEngine,
    context: ExecutionContext,
    signal: F,
) -> anyhow::Result<ExecutionStats>
where
    F: Future<Output = io::Result<()>>,
{
    tokio::select! {
        result = engine.run() => result.map_err(|error| anyhow::anyhow!(error.to_string())),
        signal = signal => {
            signal.map_err(|error| anyhow::anyhow!(error.to_string()))?;
            context.cancel();
            Err(anyhow::anyhow!("Pipeline cancelled by user"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::{AjisaiError, Row, RowSchema, Transform};
    use async_trait::async_trait;
    use tokio::sync::mpsc::Sender;

    struct PendingSource;

    #[async_trait]
    impl Transform for PendingSource {
        fn name(&self) -> &str {
            "PendingSource"
        }
        fn output_schema(&self, input: &RowSchema) -> ajisai_core::Result<RowSchema> {
            Ok(input.clone())
        }
        async fn open(&mut self, _ctx: &ExecutionContext) -> ajisai_core::Result<()> {
            Ok(())
        }
        fn is_source(&self) -> bool {
            true
        }
        async fn produce(&mut self, _sender: Sender<Row>) -> ajisai_core::Result<()> {
            std::future::pending::<()>().await;
            Ok(())
        }
        async fn process(&mut self, _row: Row) -> ajisai_core::Result<Vec<Row>> {
            Err(AjisaiError::Pipeline("not used".into()))
        }
        async fn close(&mut self) -> ajisai_core::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn cancellation_signal_cancels_shared_context() {
        let mut pipeline = ajisai_core::Pipeline::new("cancel-test");
        pipeline.add_node("source", Box::new(PendingSource));
        let context = ExecutionContext::new();
        let signal = async { Ok::<(), io::Error>(()) };

        let result = run_with_signal(
            PipelineEngine::new(pipeline, context.clone()),
            context.clone(),
            signal,
        )
        .await;

        assert_eq!(
            result.unwrap_err().to_string(),
            "Pipeline cancelled by user"
        );
        assert!(context.is_cancelled());
    }

    #[test]
    fn persist_record_uses_atomic_final_path_without_leaving_staging_file() {
        let path = std::env::temp_dir().join(format!(
            "ajisai-record-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let value = serde_json::json!({"status": "succeeded"});
        persist_record(Some(&path), &value).unwrap();
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("succeeded")
        );
        let staging = path.with_extension(format!("tmp.{}", std::process::id()));
        assert!(!staging.exists());
        std::fs::remove_file(path).unwrap();
    }
}
