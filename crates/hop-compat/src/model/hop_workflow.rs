/// Intermediate representation of a Hop workflow XML file (.hwf)

use serde::{Deserialize, Serialize};

/// Root element of a .hwf file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HopWorkflow {
    pub name:    String,
    pub actions: Vec<HopAction>,
    pub hops:    Vec<HopWorkflowHop>,
}

/// An action (step) in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopAction {
    pub name:      String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub xloc:      Option<i32>,
    pub yloc:      Option<i32>,
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
}

/// A directed hop in a workflow (with success/failure/unconditional semantics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopWorkflowHop {
    pub from:          String,
    pub to:            String,
    pub enabled:       Option<bool>,
    /// "true", "false", or None for unconditional
    pub evaluation:    Option<String>,
    pub unconditional: Option<bool>,
}
