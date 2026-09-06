use ajisai_core::{ExecutionContext, NativePipelineDocument};
use ajisai_transforms::default_registry;
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Preview rows from a Native CSV source without running downstream sinks.
pub async fn preview(path: PathBuf, limit: usize) -> anyhow::Result<()> {
    if limit == 0 {
        anyhow::bail!("preview row limit must be greater than zero");
    }
    let source = fs::read_to_string(&path)?;
    let document: NativePipelineDocument = serde_json::from_str(&source)?;
    let diagnostics = document.validate();
    if !diagnostics.is_empty() {
        anyhow::bail!(
            "native pipeline validation failed ({} diagnostic(s))",
            diagnostics.len()
        );
    }
    let state = document.to_state();
    let source_ids: std::collections::HashSet<_> =
        state.edges.iter().map(|edge| edge.to.as_str()).collect();
    let node = state
        .nodes
        .iter()
        .find(|node| !source_ids.contains(node.id.as_str()))
        .ok_or_else(|| anyhow::anyhow!("pipeline has no source node"))?;
    if node.type_name != "CsvFileInput" {
        anyhow::bail!(
            "preview currently supports CsvFileInput sources only (found {})",
            node.type_name
        );
    }
    let registry = default_registry();
    let mut transform = registry.create(&node.type_name, node.config.clone())?;
    let context = ExecutionContext::new();
    transform.open(&context).await?;
    let (sender, mut receiver) = mpsc::channel(limit.min(1024));
    let producer = tokio::spawn(async move { transform.produce(sender).await });
    let mut rows = Vec::new();
    let mut schema = Vec::new();
    while rows.len() < limit {
        match receiver.recv().await {
            Some(row) => {
                if schema.is_empty() {
                    schema = row
                        .schema
                        .fields
                        .iter()
                        .map(|field| serde_json::json!({"name": field.name, "type": format!("{:?}", field.value_type)}))
                        .collect();
                }
                rows.push(serde_json::json!({"values": row.values.iter().map(|value| value.to_display_string()).collect::<Vec<_>>()}));
            }
            None => break,
        }
    }
    producer.abort();
    println!(
        "{}",
        serde_json::json!({"pipeline": document.name, "source": node.id, "schema": schema, "rows": rows, "limit": limit})
    );
    Ok(())
}
