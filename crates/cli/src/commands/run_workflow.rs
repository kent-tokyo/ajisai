use ajisai_core::{ExecutionContext, WorkflowEngine};
use ajisai_hop_compat::{hop_workflow_to_ajisai, load_workflow_file};
use ajisai_transforms::default_registry;
use rust_i18n::t;
use std::path::PathBuf;
use tracing::error;

pub async fn run_workflow(
    workflow_path: PathBuf,
    env_vars: Vec<String>,
    project_root: Option<PathBuf>,
    json_output: bool,
    dry_run: bool,
    record_path: Option<PathBuf>,
    run_id: String,
) -> anyhow::Result<()> {
    let mut ctx = ExecutionContext::new();
    if let Some(root) = project_root {
        ctx.set_project_root(root);
    }
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
    let wf_ext = workflow_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
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
    let action_count = hop_workflow.actions.len();
    let hop_count = hop_workflow.hops.len();
    let base_dir = workflow_path.parent().unwrap_or(std::path::Path::new("."));
    let workflow = hop_workflow_to_ajisai(hop_workflow, base_dir, default_registry)?;

    if dry_run {
        let result = serde_json::json!({
            "schema_version": 1,
            "status": "dry_run",
            "run_id": run_id,
            "workflow": name,
            "actions": action_count,
            "hops": hop_count,
            "side_effects": false,
        });
        if json_output {
            println!("{}", result);
        } else {
            println!(
                "Dry run: {} (actions={}, hops={}, side_effects=false)",
                result["workflow"], result["actions"], result["hops"]
            );
        }
        crate::commands::run::persist_record(record_path.as_deref(), &result)?;
        return Ok(());
    }

    if !json_output {
        println!("{}", t!("workflow.start", name = name.as_str()));
    }

    let engine = WorkflowEngine::new(workflow, ctx);
    match engine.run().await {
        Ok(stats) => {
            let result = serde_json::json!({
                "schema_version": 1,
                "status": if stats.failed_actions > 0 { "failed" } else { "succeeded" },
                "run_id": run_id,
                "workflow": name,
                "total_actions": stats.total_actions,
                "failed_actions": stats.failed_actions,
                "elapsed_ms": stats.elapsed_ms,
            });
            if json_output {
                println!("{}", result);
            } else {
                println!(
                    "{}",
                    t!(
                        "workflow.success",
                        name = name.as_str(),
                        ms = stats.elapsed_ms.to_string().as_str()
                    )
                );
            }
            crate::commands::run::persist_record(record_path.as_deref(), &result)?;
            if stats.failed_actions > 0 {
                anyhow::bail!("{} action(s) failed", stats.failed_actions);
            }
        }
        Err(e) => {
            error!("Workflow failed: {}", e);
            let result = serde_json::json!({
                "schema_version": 1,
                "status": "failed",
                "run_id": run_id,
                "workflow": name,
                "error": e.to_string(),
            });
            if json_output {
                println!("{}", result);
            }
            crate::commands::run::persist_record(record_path.as_deref(), &result)?;
            anyhow::bail!("{}", t!("workflow.error", error = e.to_string().as_str()));
        }
    }

    Ok(())
}
