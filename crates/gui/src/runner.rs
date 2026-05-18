use ajisai_core::{ExecutionContext, ExecutionStats, Pipeline, PipelineEngine, Result};
use ajisai_transforms::default_registry;

use crate::state::PipelineState;

pub async fn build_and_run(ps: &PipelineState) -> Result<ExecutionStats> {
    let registry = default_registry();
    let mut pipeline = Pipeline::new(&ps.name);

    for node in &ps.nodes {
        let transform = registry.create(&node.type_name, node.config.clone())?;
        pipeline.add_node(&node.id, transform);
    }

    for edge in &ps.edges {
        pipeline.add_hop(&edge.from, &edge.to);
    }

    PipelineEngine::new(pipeline, ExecutionContext::new()).run().await
}
