use ajisai_core::{ExecutionContext, WorkflowEngine};
use ajisai_hop_compat::{hop_workflow_to_ajisai, load_workflow_file};
use rust_i18n::t;
use std::path::PathBuf;
use tracing::error;

pub async fn run_workflow(workflow_path: PathBuf, env_vars: Vec<String>) -> anyhow::Result<()> {
    let mut ctx = ExecutionContext::new();
    for kv in &env_vars {
        if let Some((k, v)) = kv.split_once('=') {
            ctx.set_var(k.trim(), v.trim());
        } else {
            eprintln!("{}", t!("error.invalid_env", value = kv.as_str()));
        }
    }

    if !workflow_path.exists() {
        anyhow::bail!(
            "{}",
            t!(
                "error.file_not_found",
                path = workflow_path.display().to_string().as_str()
            )
        );
    }
    let wf_ext = workflow_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(wf_ext, "hwf" | "kjb") {
        anyhow::bail!(
            "{}",
            t!(
                "error.unsupported_format",
                path = workflow_path.display().to_string().as_str()
            )
        );
    }

    let hop_workflow = load_workflow_file(&workflow_path)?;
    let name = hop_workflow.name.clone();

    println!("{}", t!("workflow.start", name = name.as_str()));

    let base_dir = workflow_path.parent().unwrap_or(std::path::Path::new("."));
    let workflow = hop_workflow_to_ajisai(hop_workflow, base_dir)?;

    let engine = WorkflowEngine::new(workflow, ctx);
    match engine.run().await {
        Ok(stats) => {
            println!(
                "{}",
                t!(
                    "workflow.success",
                    name = name.as_str(),
                    ms = stats.elapsed_ms.to_string().as_str()
                )
            );
            if stats.failed_actions > 0 {
                anyhow::bail!("{} action(s) failed", stats.failed_actions);
            }
        }
        Err(e) => {
            error!("Workflow failed: {}", e);
            anyhow::bail!("{}", t!("workflow.error", error = e.to_string().as_str()));
        }
    }

    Ok(())
}
