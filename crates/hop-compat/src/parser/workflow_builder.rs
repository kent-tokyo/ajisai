use crate::model::hop_workflow::HopWorkflow;
use ajisai_core::{
    workflow::{Action, ActionResult, HopEvaluation, Workflow},
    AjisaiError, ExecutionContext,
};
use std::path::{Path, PathBuf};

/// Action that runs a pipeline (.hpl file)
pub struct PipelineAction {
    pub name: String,
    pub pipeline_path: PathBuf,
}

#[async_trait::async_trait]
impl Action for PipelineAction {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext) -> ActionResult {
        let path_str = ctx.resolve(self.pipeline_path.to_str().unwrap_or(""));
        let path = Path::new(&path_str);

        let hop_pipeline = match ajisai_hop_compat_inner::load_pipeline_file(path) {
            Ok(p) => p,
            Err(e) => return ActionResult::err(e.to_string()),
        };

        let registry = ajisai_transforms::default_registry();
        let pipeline =
            match crate::parser::pipeline::hop_pipeline_to_ajisai(hop_pipeline, &registry) {
                Ok(p) => p,
                Err(e) => return ActionResult::err(e.to_string()),
            };

        let engine = ajisai_core::PipelineEngine::new(pipeline, ctx.clone());
        match engine.run().await {
            Ok(_) => ActionResult::ok(),
            Err(e) => ActionResult::err(e.to_string()),
        }
    }
}

// Internal helper to avoid circular import — uses lib.rs directly
mod ajisai_hop_compat_inner {
    use crate::model::hop_pipeline::HopPipeline;
    use ajisai_core::AjisaiError;

    pub fn load_pipeline_file(path: &std::path::Path) -> Result<HopPipeline, AjisaiError> {
        let xml = std::fs::read_to_string(path).map_err(AjisaiError::Io)?;
        crate::parser::pipeline::parse_hpl(&xml)
    }
}

/// Convert a HopWorkflow IR into an ajisai-core Workflow.
/// `base_dir` is used to resolve relative pipeline file paths.
pub fn hop_workflow_to_ajisai(hw: HopWorkflow, base_dir: &Path) -> Result<Workflow, AjisaiError> {
    let mut workflow = Workflow::new(hw.name);

    for action in hw.actions {
        let pipeline_file = action
            .attributes
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let path = if pipeline_file.is_empty() {
            // Fallback: look for <action_name>.hpl next to the .hwf file
            base_dir.join(format!("{}.hpl", action.name))
        } else {
            let p = Path::new(pipeline_file);
            if p.is_absolute() {
                p.to_owned()
            } else {
                base_dir.join(p)
            }
        };

        let action_name = action.name.clone();
        workflow.add_action(
            action_name,
            Box::new(PipelineAction {
                name: action.name,
                pipeline_path: path,
            }),
        );
    }

    for hop in hw.hops {
        let evaluation = if hop.unconditional.unwrap_or(false) {
            HopEvaluation::Unconditional
        } else {
            match hop.evaluation.as_deref() {
                Some("true") => HopEvaluation::Success,
                Some("false") => HopEvaluation::Failure,
                _ => HopEvaluation::Unconditional,
            }
        };
        workflow.add_hop(
            hop.from,
            hop.to,
            evaluation,
            hop.unconditional.unwrap_or(false),
        );
    }

    Ok(workflow)
}
