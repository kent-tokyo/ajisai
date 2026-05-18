use ajisai_hop_compat::load_pipeline_file;
use rust_i18n::t;
use std::path::PathBuf;

pub fn validate(pipeline_path: PathBuf) -> anyhow::Result<()> {
    if !pipeline_path.exists() {
        anyhow::bail!(
            "{}",
            t!(
                "error.file_not_found",
                path = pipeline_path.display().to_string().as_str()
            )
        );
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
