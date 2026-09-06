use ajisai_core::{NativePipelineDocument, NativeWorkflowDocument};
use std::fs;
use std::path::PathBuf;

/// Print a side-effect-free explanation of a native Pipeline or Workflow.
pub fn explain(path: PathBuf) -> anyhow::Result<()> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let source = fs::read_to_string(&path)?;

    if file_name.ends_with(".ajw") || file_name.ends_with(".ajisai.workflow.json") {
        let document: NativeWorkflowDocument = serde_json::from_str(&source)?;
        let diagnostics = document.validate();
        if !diagnostics.is_empty() {
            anyhow::bail!(
                "workflow has {} validation diagnostic(s)",
                diagnostics.len()
            );
        }
        println!("Workflow: {}", document.name);
        println!("Actions:");
        for action in &document.actions {
            println!("  - {} ({})", action.id, action.type_name);
        }
        println!("Hops:");
        for hop in &document.hops {
            println!(
                "  - {} -> {} [{}{}]",
                hop.from,
                hop.to,
                hop.evaluation,
                if hop.unconditional {
                    ", unconditional"
                } else {
                    ""
                }
            );
        }
        return Ok(());
    }

    if file_name.ends_with(".ajp") || file_name.ends_with(".ajisai.json") {
        let document: NativePipelineDocument = serde_json::from_str(&source)?;
        let diagnostics = document.validate();
        if !diagnostics.is_empty() {
            anyhow::bail!(
                "pipeline has {} validation diagnostic(s)",
                diagnostics.len()
            );
        }
        println!("Pipeline: {}", document.name);
        println!("Nodes:");
        for node in &document.nodes {
            println!("  - {} ({})", node.id, node.type_name);
        }
        println!("Edges:");
        for edge in &document.edges {
            println!("  - {} -> {}", edge.from, edge.to);
        }
        return Ok(());
    }

    anyhow::bail!("explain currently supports .ajp, .ajisai.json, and .ajw")
}
