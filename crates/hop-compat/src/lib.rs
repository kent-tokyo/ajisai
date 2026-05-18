pub mod model;
pub mod parser;

pub use model::{HopHop, HopPipeline, HopPipelineInfo, HopTransform, HopWorkflow};
pub use parser::{hop_pipeline_to_ajisai, hop_workflow_to_ajisai, parse_hpl, parse_hwf};
pub use parser::{write_hpl, write_hpl_file};

use ajisai_core::AjisaiError;

/// Load a .hpl or .hwf file from disk and return parsed IR
pub fn load_pipeline_file(path: &std::path::Path) -> Result<HopPipeline, AjisaiError> {
    let xml = std::fs::read_to_string(path).map_err(AjisaiError::Io)?;
    parse_hpl(&xml)
}

pub fn load_workflow_file(path: &std::path::Path) -> Result<HopWorkflow, AjisaiError> {
    let xml = std::fs::read_to_string(path).map_err(AjisaiError::Io)?;
    parse_hwf(&xml)
}
