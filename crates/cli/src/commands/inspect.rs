use std::fs;
use std::path::PathBuf;

/// Inspect a pipeline/workflow document without executing it.
pub fn inspect(path: PathBuf) -> anyhow::Result<()> {
    let metadata = fs::metadata(&path)
        .map_err(|error| anyhow::anyhow!("cannot inspect '{}': {error}", path.display()))?;
    let source = fs::read_to_string(&path)?;
    let value: serde_json::Value = serde_json::from_str(&source)
        .map_err(|error| anyhow::anyhow!("invalid JSON document: {error}"))?;
    let format = value
        .get("format")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let kind = if format == "ajisai.workflow" {
        "workflow"
    } else if format == "ajisai.pipeline" {
        "pipeline"
    } else {
        "unknown"
    };

    println!("Path: {}", path.display());
    println!("Format: {format}");
    println!("Kind: {kind}");
    println!("Bytes: {}", metadata.len());
    if let Some(name) = value.get("name").and_then(serde_json::Value::as_str) {
        println!("Name: {name}");
    }
    if kind == "pipeline" {
        println!(
            "Nodes: {}",
            value
                .get("nodes")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len)
        );
        println!(
            "Edges: {}",
            value
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len)
        );
        if let Some(nodes) = value.get("nodes").and_then(serde_json::Value::as_array) {
            println!("Node schema:");
            for node in nodes {
                let id = node
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("?");
                let node_type = node
                    .get("type_name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("?");
                let keys = node
                    .get("config")
                    .and_then(serde_json::Value::as_object)
                    .map(|config| config.keys().cloned().collect::<Vec<_>>().join(","))
                    .unwrap_or_default();
                println!("  - {id}: {node_type} (config: {keys})");
            }
        }
    } else if kind == "workflow" {
        println!(
            "Actions: {}",
            value
                .get("actions")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len)
        );
        println!(
            "Hops: {}",
            value
                .get("hops")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len)
        );
        if let Some(actions) = value.get("actions").and_then(serde_json::Value::as_array) {
            println!("Action schema:");
            for action in actions {
                let id = action
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("?");
                let action_type = action
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("?");
                println!("  - {id}: {action_type}");
            }
        }
    }
    Ok(())
}
