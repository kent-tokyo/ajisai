use ajisai_core::NativePipelineDocument;
use std::fs;
use std::path::PathBuf;

/// Create a minimal, valid native pipeline document without executing it.
pub fn new_pipeline(output: PathBuf, name: Option<String>) -> anyhow::Result<()> {
    let pipeline_name = name.unwrap_or_else(|| {
        output
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.trim().is_empty())
            .unwrap_or("new-pipeline")
            .to_owned()
    });
    let document = NativePipelineDocument::new(pipeline_name);
    let diagnostics = document.validate();
    if !diagnostics.is_empty() {
        anyhow::bail!("generated pipeline failed validation");
    }
    let parent = output.parent().unwrap_or_else(|| std::path::Path::new("."));
    fs::create_dir_all(parent)?;
    fs::write(&output, format!("{}\n", document.to_canonical_json()?))?;
    println!("new.success: {}", output.display());
    Ok(())
}
