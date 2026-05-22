/// Intermediate representation of a Hop pipeline XML file (.hpl)
/// Mirrors the Hop XML structure without importing ajisai-core types.
use serde::{Deserialize, Serialize};

/// Root element of a .hpl file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HopPipeline {
    pub name: String,
    pub transforms: Vec<HopTransform>,
    pub order: Vec<HopHop>,
    pub info: HopPipelineInfo,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HopPipelineInfo {
    pub description: Option<String>,
}

/// A single transform node in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopTransform {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub description: Option<String>,
    pub xloc: Option<i32>,
    pub yloc: Option<i32>,
    /// Raw attributes and child elements as key-value pairs
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
}

/// A directed connection between two transform nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopHop {
    pub from: String,
    pub to: String,
    pub enabled: Option<bool>,
    pub error_hop: Option<bool>,
}
