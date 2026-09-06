use ajisai_core::{
    NativePipelineDocument, NativeWorkflowDocument, migrate_pipeline_to_current,
    migrate_workflow_to_current,
};
use std::fs;
use std::path::PathBuf;

pub fn migrate(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    let source = fs::read_to_string(&input)?;
    let file_name = input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let canonical = if file_name.ends_with(".ajw") || file_name.ends_with(".ajisai.workflow.json") {
        let document: NativeWorkflowDocument = serde_json::from_str(&source)
            .map_err(|error| anyhow::anyhow!("Invalid native workflow: {error}"))?;
        let diagnostics = document.validate();
        if !diagnostics.is_empty() {
            anyhow::bail!(
                "native workflow validation failed ({} diagnostic(s))",
                diagnostics.len()
            );
        }
        migrate_workflow_to_current(document)
            .map_err(|error| anyhow::anyhow!(error))?
            .to_canonical_json()?
    } else {
        let document: NativePipelineDocument = serde_json::from_str(&source)
            .map_err(|error| anyhow::anyhow!("Invalid native pipeline: {error}"))?;
        let diagnostics = document.validate();
        if !diagnostics.is_empty() {
            anyhow::bail!(
                "native pipeline validation failed ({} diagnostic(s))",
                diagnostics.len()
            );
        }
        migrate_pipeline_to_current(document)
            .map_err(|error| anyhow::anyhow!(error))?
            .to_canonical_json()?
    };
    fs::write(&output, format!("{canonical}\n"))?;
    println!(
        "migrate.success: {} -> {}",
        input.display(),
        output.display()
    );
    Ok(())
}
