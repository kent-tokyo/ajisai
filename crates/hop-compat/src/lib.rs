pub mod model;
pub mod parser;

pub use model::{HopHop, HopPipeline, HopPipelineInfo, HopTransform, HopWorkflow};
pub use parser::{hop_pipeline_to_ajisai, hop_workflow_to_ajisai, parse_hpl, parse_hwf};
pub use parser::{parse_dtsx, parse_kjb, parse_ktr};
pub use parser::{write_hpl, write_hpl_file};

use ajisai_core::AjisaiError;

/// Load a pipeline file from disk — supports .hpl (Apache Hop), .ktr (Kettle), .dtsx (SSIS)
pub fn load_pipeline_file(path: &std::path::Path) -> Result<HopPipeline, AjisaiError> {
    let xml = std::fs::read_to_string(path).map_err(AjisaiError::Io)?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("ktr") => parse_ktr(&xml),
        Some("dtsx") => parse_dtsx(&xml),
        _ => parse_hpl(&xml),
    }
}

/// Load a workflow file from disk — supports .hwf (Apache Hop), .kjb (Kettle)
pub fn load_workflow_file(path: &std::path::Path) -> Result<HopWorkflow, AjisaiError> {
    let xml = std::fs::read_to_string(path).map_err(AjisaiError::Io)?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("kjb") => parse_kjb(&xml),
        _ => parse_hwf(&xml),
    }
}
