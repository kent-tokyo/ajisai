use ajisai_core::{ExecutionContext, PipelineEngine};
use ajisai_hop_compat::{hop_pipeline_to_ajisai, load_pipeline_file};
use ajisai_transforms::default_registry;
use indicatif::{ProgressBar, ProgressStyle};
use rust_i18n::t;
use std::path::PathBuf;
use std::time::Duration;
use tracing::error;

pub async fn run(pipeline_path: PathBuf, env_vars: Vec<String>) -> anyhow::Result<()> {
    let mut ctx = ExecutionContext::new();

    for kv in &env_vars {
        if let Some((k, v)) = kv.split_once('=') {
            ctx.set_var(k.trim(), v.trim());
        } else {
            eprintln!("{}", t!("error.invalid_env", value = kv.as_str()));
        }
    }

    if !pipeline_path.exists() {
        anyhow::bail!("{}", t!("error.file_not_found", path = pipeline_path.display().to_string().as_str()));
    }

    let ext = pipeline_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "hpl" {
        anyhow::bail!("{}", t!("error.unsupported_format", path = pipeline_path.display().to_string().as_str()));
    }

    let hop_pipeline = load_pipeline_file(&pipeline_path)?;
    let name = hop_pipeline.name.clone();
    let node_count = hop_pipeline.transforms.len();

    println!("{}", t!("run.start", name = name.as_str()));

    let registry = default_registry();
    let pipeline = hop_pipeline_to_ajisai(hop_pipeline, &registry)?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(t!("run.nodes", count = node_count.to_string().as_str()).to_string());
    pb.enable_steady_tick(Duration::from_millis(100));

    let engine = PipelineEngine::new(pipeline, ctx);
    match engine.run().await {
        Ok(stats) => {
            pb.finish_and_clear();
            println!("{}", t!("run.success", name = name.as_str(), ms = stats.elapsed_ms.to_string().as_str()));
        }
        Err(e) => {
            pb.finish_and_clear();
            error!("Pipeline failed: {}", e);
            anyhow::bail!("{}", t!("run.error", error = e.to_string().as_str()));
        }
    }

    Ok(())
}
