use ajisai_hop_compat::{assess_pipeline, load_pipeline_file};
use ajisai_transforms::default_registry;
use rust_i18n::t;
use std::path::PathBuf;

/// Assess Hop compatibility without converting or executing the pipeline.
pub fn assess(pipeline: PathBuf, json: bool) -> anyhow::Result<()> {
    if !pipeline.exists() {
        anyhow::bail!(
            "{}",
            t!(
                "error.file_not_found",
                path = pipeline.display().to_string().as_str()
            )
        );
    }
    if pipeline.extension().and_then(|value| value.to_str()) != Some("hpl") {
        anyhow::bail!(
            "{}",
            t!(
                "error.unsupported_format",
                path = pipeline.display().to_string().as_str()
            )
        );
    }
    let parsed = load_pipeline_file(&pipeline)?;
    let report = assess_pipeline(&parsed, &default_registry());
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}: {:?}", report.pipeline, report.status);
        for transform in report.transforms {
            println!(
                "  {} [{} -> {}] {:?} ({})",
                transform.name,
                transform.source_type,
                transform.ajisai_type,
                transform.status,
                transform.reason_code
            );
        }
    }
    Ok(())
}
