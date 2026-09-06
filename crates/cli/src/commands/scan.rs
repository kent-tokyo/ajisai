use ajisai_hop_compat::scan_project;
use ajisai_transforms::default_registry;
use rust_i18n::t;
use std::path::PathBuf;

/// Inventory a Hop project without executing any content.
pub fn scan(project: PathBuf, json: bool) -> anyhow::Result<()> {
    if !project.exists() {
        anyhow::bail!(
            "{}",
            t!(
                "error.file_not_found",
                path = project.display().to_string().as_str()
            )
        );
    }
    let report = scan_project(&project, &default_registry())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Project: {}", report.root);
        println!(
            "Files: {}, pipelines: {}, workflows: {}, warnings: {}",
            report.files.len(),
            report.pipelines.len(),
            report.workflows.len(),
            report.warnings.len()
        );
        for pipeline in report.pipelines {
            println!("  {}: {:?}", pipeline.pipeline, pipeline.status);
        }
        for workflow in report.workflows {
            println!(
                "  workflow {}: {} actions, {} hops",
                workflow.name,
                workflow.actions.len(),
                workflow.hops
            );
        }
        for warning in report.warnings {
            println!("  warning: {warning}");
        }
    }
    Ok(())
}
