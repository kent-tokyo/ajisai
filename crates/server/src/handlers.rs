use ajisai_core::{PipelineState, TRANSFORM_CATEGORIES, build_and_run, default_config};
use ajisai_hop_compat::load_pipeline_file;
use ajisai_transforms::default_registry;
use serde_json::{Value, json};
use std::path::PathBuf;

pub async fn ping() -> Value {
    json!({ "pong": true })
}

pub fn get_transforms() -> Value {
    let mut categories = Vec::new();

    for (cat_name, cat_transforms) in TRANSFORM_CATEGORIES {
        let transforms = cat_transforms
            .iter()
            .map(|(type_name, display_name)| {
                json!({
                    "type_name": type_name,
                    "display_name": display_name,
                    "default_config": default_config(type_name)
                })
            })
            .collect::<Vec<_>>();

        categories.push(json!({
            "name": cat_name,
            "transforms": transforms
        }));
    }

    json!({ "categories": categories })
}

pub async fn run_pipeline(pipeline_json: Value) -> Result<Value, String> {
    let pipeline: PipelineState = serde_json::from_value(pipeline_json)
        .map_err(|e| format!("Failed to deserialize pipeline: {}", e))?;

    let registry = default_registry();
    let stats = build_and_run(&pipeline, &registry)
        .await
        .map_err(|e| format!("Pipeline execution failed: {}", e))?;

    Ok(json!({
        "elapsed_ms": stats.elapsed_ms,
        "rows_read": stats.rows_read,
        "rows_written": stats.rows_written,
    }))
}

pub fn validate_pipeline(pipeline_json: Value) -> Result<Value, String> {
    let pipeline: PipelineState = serde_json::from_value(pipeline_json)
        .map_err(|e| format!("Failed to deserialize pipeline: {}", e))?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if pipeline.nodes.is_empty() {
        warnings.push("Pipeline has no transforms".to_string());
    }

    let registry = default_registry();
    for node in &pipeline.nodes {
        match registry.create(&node.type_name, node.config.clone()) {
            Ok(_) => {}
            Err(e) => {
                errors.push(format!("Node {}: {}", node.id, e));
            }
        }
    }

    Ok(json!({
        "ok": errors.is_empty(),
        "warnings": warnings,
        "errors": errors,
    }))
}

pub fn load_pipeline(path_str: String) -> Result<Value, String> {
    let path = PathBuf::from(&path_str);

    match load_pipeline_file(&path) {
        Ok(hop) => {
            let mut ps = PipelineState::new(&hop.name);
            for tr in hop.transforms {
                let mapped_type = ajisai_hop_compat::map_transform_type(&tr.type_name);
                let mut node = ajisai_core::Node::new(&tr.name, mapped_type, [0.0, 0.0]);
                node.label = tr.name.clone();
                node.config = serde_json::to_value(&tr.attributes).unwrap_or_default();
                ps.add_node(node);
            }
            for h in &hop.order {
                if h.enabled.unwrap_or(true) {
                    ps.add_edge(&h.from, &h.to);
                }
            }
            serde_json::to_value(&ps).map_err(|e| format!("Failed to serialize pipeline: {}", e))
        }
        Err(e) => Err(format!("Failed to load pipeline: {}", e)),
    }
}

pub fn save_pipeline(
    pipeline_json: Value,
    path_str: String,
    format: String,
) -> Result<Value, String> {
    let _pipeline: PipelineState = serde_json::from_value(pipeline_json)
        .map_err(|e| format!("Failed to deserialize pipeline: {}", e))?;

    match format.as_str() {
        "json" => {
            let path = PathBuf::from(&path_str);
            std::fs::write(&path, serde_json::to_string_pretty(&_pipeline).unwrap())
                .map_err(|e| format!("Failed to write JSON file: {}", e))?;
            Ok(json!({ "path": path_str }))
        }
        "hpl" => Err("HPL format export not yet implemented".to_string()),
        _ => Err(format!("Unknown format: {}", format)),
    }
}
