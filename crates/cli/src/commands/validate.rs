use ajisai_core::{NativePipelineDocument, NativeWorkflowDocument};
use ajisai_hop_compat::load_pipeline_file;
use rust_i18n::t;
use std::fs;
use std::path::PathBuf;

pub fn validate(pipeline_path: PathBuf, json: bool) -> anyhow::Result<()> {
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
    if file_name.ends_with(".ajisai.json") || file_name.ends_with(".ajp") {
        return validate_native_pipeline(&pipeline_path, json);
    }
    if file_name.ends_with(".ajisai.workflow.json") || file_name.ends_with(".ajw") {
        return validate_native_workflow(&pipeline_path, json);
    }

    let ext = pipeline_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if ext != "hpl" {
        anyhow::bail!(
            "{}",
            t!(
                "error.unsupported_format",
                path = pipeline_path.display().to_string().as_str()
            )
        );
    }

    let hop_pipeline = load_pipeline_file(&pipeline_path)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "valid": true,
                "format": "hop.hpl",
                "name": hop_pipeline.name,
                "nodes": hop_pipeline.transforms.len(),
                "edges": hop_pipeline.order.len()
            })
        );
        return Ok(());
    }

    println!(
        "{}",
        t!(
            "validate.ok",
            name = hop_pipeline.name.as_str(),
            nodes = hop_pipeline.transforms.len().to_string().as_str(),
            hops = hop_pipeline.order.len().to_string().as_str(),
        )
    );

    println!("\n{}", t!("validate.nodes_header"));
    for tr in &hop_pipeline.transforms {
        println!(
            "  [{}] {} ({})",
            tr.type_name,
            tr.name,
            tr.description.as_deref().unwrap_or(""),
        );
    }

    println!("\n{}", t!("validate.hops_header"));
    for h in &hop_pipeline.order {
        let status = if h.enabled.unwrap_or(true) { "+" } else { "-" };
        println!("  {} {} -> {}", status, h.from, h.to);
    }

    Ok(())
}

fn validate_native_pipeline(path: &PathBuf, json: bool) -> anyhow::Result<()> {
    let source = fs::read_to_string(path)?;
    let document: NativePipelineDocument = serde_json::from_str(&source)
        .map_err(|error| anyhow::anyhow!("Invalid Ajisai native document: {error}"))?;
    let diagnostics = document.validate();
    if !diagnostics.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "valid": false,
                    "format": document.format.clone(),
                    "diagnostics": diagnostics.iter().map(|d| serde_json::json!({"code": d.code, "path": d.path, "message": d.message})).collect::<Vec<_>>()
                })
            );
            anyhow::bail!("native document validation failed");
        }
        for diagnostic in &diagnostics {
            eprintln!(
                "{} [{}] {}: {}",
                diagnostic.code,
                diagnostic.path,
                diagnostic.message,
                path.display()
            );
        }
        anyhow::bail!(
            "native document validation failed ({} diagnostic(s))",
            diagnostics.len()
        );
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "valid": true,
                "format": document.format.clone(),
                "name": document.name.clone(),
                "nodes": document.nodes.len(),
                "edges": document.edges.len()
            })
        );
        return Ok(());
    }

    println!(
        "Ajisai native pipeline '{}' is valid ({} nodes, {} edges)",
        document.name,
        document.nodes.len(),
        document.edges.len()
    );
    Ok(())
}

fn validate_native_workflow(path: &PathBuf, json: bool) -> anyhow::Result<()> {
    let source = fs::read_to_string(path)?;
    let document: NativeWorkflowDocument = serde_json::from_str(&source)
        .map_err(|error| anyhow::anyhow!("Invalid Ajisai native workflow: {error}"))?;
    let diagnostics = document.validate();
    if !diagnostics.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "valid": false,
                    "format": document.format.clone(),
                    "diagnostics": diagnostics.iter().map(|d| serde_json::json!({"code": d.code, "path": d.path, "message": d.message})).collect::<Vec<_>>()
                })
            );
            anyhow::bail!("native workflow validation failed");
        }
        for diagnostic in &diagnostics {
            eprintln!(
                "{} [{}] {}: {}",
                diagnostic.code,
                diagnostic.path,
                diagnostic.message,
                path.display()
            );
        }
        anyhow::bail!(
            "native workflow validation failed ({} diagnostic(s))",
            diagnostics.len()
        );
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "valid": true,
                "format": document.format.clone(),
                "name": document.name.clone(),
                "actions": document.actions.len(),
                "hops": document.hops.len()
            })
        );
        return Ok(());
    }

    println!(
        "Ajisai native workflow '{}' is valid ({} actions, {} hops)",
        document.name,
        document.actions.len(),
        document.hops.len()
    );
    Ok(())
}
