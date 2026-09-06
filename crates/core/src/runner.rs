use crate::model::PipelineState;
use crate::registry::TransformRegistry;
use crate::{ExecutionContext, ExecutionStats, Pipeline, PipelineEngine, Result};

pub async fn build_and_run(
    ps: &PipelineState,
    registry: &TransformRegistry,
) -> Result<ExecutionStats> {
    let mut pipeline = Pipeline::new(&ps.name);

    for node in &ps.nodes {
        let transform = registry.create(&node.type_name, node.config.clone())?;
        pipeline.add_node(&node.id, transform);
    }

    for edge in &ps.edges {
        if edge.is_error {
            pipeline.add_error_hop(&edge.from, &edge.to);
        } else {
            pipeline.add_hop(&edge.from, &edge.to);
        }
    }

    PipelineEngine::new(pipeline, ExecutionContext::new())
        .run()
        .await
}
