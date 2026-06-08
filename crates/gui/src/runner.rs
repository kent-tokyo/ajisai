use ajisai_core::{build_and_run as core_build_and_run, ExecutionStats, PipelineState, Result};
use ajisai_transforms::default_registry;

pub async fn build_and_run(ps: &PipelineState) -> Result<ExecutionStats> {
    let registry = default_registry();
    core_build_and_run(ps, &registry).await
}
