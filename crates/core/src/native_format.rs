//! Versioned Ajisai-native pipeline documents.
//!
//! This module deliberately sits above the runtime `PipelineState`.  A native
//! document can be validated and explained without constructing transforms or
//! touching the filesystem/network.

use crate::model::{Edge, Node, PipelineState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

/// The first native document schema. Bump only through an explicit migration.
pub const NATIVE_FORMAT_VERSION: u32 = 1;
pub const NATIVE_FORMAT_ID: &str = "ajisai.pipeline";
pub const NATIVE_WORKFLOW_FORMAT_ID: &str = "ajisai.workflow";

pub fn migrate_pipeline_to_current(
    document: NativePipelineDocument,
) -> Result<NativePipelineDocument, String> {
    if document.format_version != NATIVE_FORMAT_VERSION {
        return Err(format!(
            "no migration registered from pipeline format version {} to {}",
            document.format_version, NATIVE_FORMAT_VERSION
        ));
    }
    Ok(document)
}

pub fn migrate_workflow_to_current(
    document: NativeWorkflowDocument,
) -> Result<NativeWorkflowDocument, String> {
    if document.format_version != NATIVE_FORMAT_VERSION {
        return Err(format!(
            "no migration registered from workflow format version {} to {}",
            document.format_version, NATIVE_FORMAT_VERSION
        ));
    }
    Ok(document)
}

/// Return deterministic JSON paths whose values differ between two native
/// documents. Object keys are compared independent of insertion order.
pub fn semantic_diff(
    left: &NativePipelineDocument,
    right: &NativePipelineDocument,
) -> serde_json::Result<Vec<String>> {
    let left = canonicalize(serde_json::to_value(left)?);
    let right = canonicalize(serde_json::to_value(right)?);
    let mut paths = Vec::new();
    diff_values(&left, &right, "$", &mut paths);
    Ok(paths)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePipelineDocument {
    /// Stable format identifier, not a product/package version.
    #[serde(default = "default_format")]
    pub format: String,
    pub format_version: u32,
    pub name: String,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// Forward-compatible fields are retained on a read/write round trip.
    #[serde(flatten)]
    pub extensions: BTreeMap<String, Value>,
}

fn default_format() -> String {
    NATIVE_FORMAT_ID.to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable machine-readable code for CLI/Studio clients.
    pub code: &'static str,
    /// JSON-like document path, for example `nodes[1].id`.
    pub path: String,
    pub message: String,
}

impl NativePipelineDocument {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            format: NATIVE_FORMAT_ID.to_owned(),
            format_version: NATIVE_FORMAT_VERSION,
            name: name.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            extensions: BTreeMap::new(),
        }
    }

    pub fn from_state(state: &PipelineState) -> Self {
        Self {
            format: NATIVE_FORMAT_ID.to_owned(),
            format_version: NATIVE_FORMAT_VERSION,
            name: state.name.clone(),
            nodes: state.nodes.clone(),
            edges: state.edges.clone(),
            extensions: BTreeMap::new(),
        }
    }

    pub fn to_state(&self) -> PipelineState {
        PipelineState {
            name: self.name.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            node_seq: self
                .nodes
                .iter()
                .filter_map(|node| node.id.strip_prefix("node_")?.parse::<u64>().ok())
                .max()
                .unwrap_or(0),
        }
    }

    /// Serialize with recursively sorted object keys for stable diffs and hashes.
    pub fn to_canonical_json(&self) -> serde_json::Result<String> {
        let value = serde_json::to_value(self)?;
        serde_json::to_string_pretty(&canonicalize(value))
    }

    /// Validate structure only. No transform construction or external I/O is
    /// performed here, so this is safe for untrusted documents.
    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if self.format != NATIVE_FORMAT_ID {
            diagnostics.push(Diagnostic {
                code: "NATIVE_FORMAT_MISMATCH",
                path: "format".into(),
                message: format!("expected '{}', got '{}'", NATIVE_FORMAT_ID, self.format),
            });
        }
        if self.format_version != NATIVE_FORMAT_VERSION {
            diagnostics.push(Diagnostic {
                code: "UNSUPPORTED_FORMAT_VERSION",
                path: "format_version".into(),
                message: format!(
                    "supported version is {}, got {}",
                    NATIVE_FORMAT_VERSION, self.format_version
                ),
            });
        }
        if self.name.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "PIPELINE_NAME_REQUIRED",
                path: "name".into(),
                message: "pipeline name must not be empty".into(),
            });
        }

        let mut node_ids = HashSet::new();
        for (index, node) in self.nodes.iter().enumerate() {
            let path = format!("nodes[{index}]");
            if node.id.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "NODE_ID_REQUIRED",
                    path: format!("{path}.id"),
                    message: "node id must not be empty".into(),
                });
            } else if !node_ids.insert(node.id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "DUPLICATE_NODE_ID",
                    path: format!("{path}.id"),
                    message: format!("node id '{}' is repeated", node.id),
                });
            }
            if !valid_stable_id(&node.id) {
                diagnostics.push(Diagnostic {
                    code: "INVALID_NODE_ID",
                    path: format!("{path}.id"),
                    message: "node id must contain only ASCII letters, digits, '.', '_' or '-'"
                        .into(),
                });
            }
            if node.type_name.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "NODE_TYPE_REQUIRED",
                    path: format!("{path}.type_name"),
                    message: "node type_name must not be empty".into(),
                });
            }
            reject_inline_secrets(&node.config, &format!("{path}.config"), &mut diagnostics);
        }

        let mut edges = HashSet::new();
        for (index, edge) in self.edges.iter().enumerate() {
            let path = format!("edges[{index}]");
            if !node_ids.contains(edge.from.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "EDGE_SOURCE_NOT_FOUND",
                    path: format!("{path}.from"),
                    message: format!("node '{}' does not exist", edge.from),
                });
            }
            if !node_ids.contains(edge.to.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "EDGE_TARGET_NOT_FOUND",
                    path: format!("{path}.to"),
                    message: format!("node '{}' does not exist", edge.to),
                });
            }
            if edge.from == edge.to {
                diagnostics.push(Diagnostic {
                    code: "SELF_EDGE_FORBIDDEN",
                    path: path.clone(),
                    message: format!("node '{}' cannot connect to itself", edge.from),
                });
            }
            if !edges.insert((&edge.from, &edge.to)) {
                diagnostics.push(Diagnostic {
                    code: "DUPLICATE_EDGE",
                    path,
                    message: format!("edge '{} -> {}' is repeated", edge.from, edge.to),
                });
            }
        }

        if diagnostics.iter().all(|diagnostic| {
            !matches!(
                diagnostic.code,
                "EDGE_SOURCE_NOT_FOUND" | "EDGE_TARGET_NOT_FOUND" | "SELF_EDGE_FORBIDDEN"
            )
        }) && has_cycle(&node_ids, &self.edges)
        {
            diagnostics.push(Diagnostic {
                code: "CYCLE_DETECTED",
                path: "edges".into(),
                message: "pipeline graph contains a cycle".into(),
            });
        }

        diagnostics
    }
}

/// Native workflow document. Workflow and pipeline remain separate concepts:
/// a workflow controls actions and branching, while a pipeline transforms a
/// stream of rows. A `pipeline` action references a pipeline document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeWorkflowDocument {
    #[serde(default = "default_workflow_format")]
    pub format: String,
    pub format_version: u32,
    pub name: String,
    #[serde(default)]
    pub actions: Vec<NativeWorkflowAction>,
    #[serde(default)]
    pub hops: Vec<NativeWorkflowHop>,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, Value>,
}

fn default_workflow_format() -> String {
    NATIVE_WORKFLOW_FORMAT_ID.to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeWorkflowAction {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeWorkflowHop {
    pub from: String,
    pub to: String,
    #[serde(default = "default_evaluation")]
    pub evaluation: String,
    #[serde(default)]
    pub unconditional: bool,
}

fn default_evaluation() -> String {
    "unconditional".into()
}

impl NativeWorkflowDocument {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            format: NATIVE_WORKFLOW_FORMAT_ID.to_owned(),
            format_version: NATIVE_FORMAT_VERSION,
            name: name.into(),
            actions: Vec::new(),
            hops: Vec::new(),
            extensions: BTreeMap::new(),
        }
    }

    pub fn to_canonical_json(&self) -> serde_json::Result<String> {
        let value = serde_json::to_value(self)?;
        serde_json::to_string_pretty(&canonicalize(value))
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if self.format != NATIVE_WORKFLOW_FORMAT_ID {
            diagnostics.push(Diagnostic {
                code: "NATIVE_WORKFLOW_FORMAT_MISMATCH",
                path: "format".into(),
                message: format!(
                    "expected '{}', got '{}'",
                    NATIVE_WORKFLOW_FORMAT_ID, self.format
                ),
            });
        }
        if self.format_version != NATIVE_FORMAT_VERSION {
            diagnostics.push(Diagnostic {
                code: "UNSUPPORTED_FORMAT_VERSION",
                path: "format_version".into(),
                message: format!(
                    "supported version is {}, got {}",
                    NATIVE_FORMAT_VERSION, self.format_version
                ),
            });
        }
        if self.name.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "WORKFLOW_NAME_REQUIRED",
                path: "name".into(),
                message: "workflow name must not be empty".into(),
            });
        }

        let mut ids = HashSet::new();
        for (index, action) in self.actions.iter().enumerate() {
            let path = format!("actions[{index}]");
            if action.id.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "ACTION_ID_REQUIRED",
                    path: format!("{path}.id"),
                    message: "action id must not be empty".into(),
                });
            } else if !ids.insert(action.id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "DUPLICATE_ACTION_ID",
                    path: format!("{path}.id"),
                    message: format!("action id '{}' is repeated", action.id),
                });
            }
            if !valid_stable_id(&action.id) {
                diagnostics.push(Diagnostic {
                    code: "INVALID_ACTION_ID",
                    path: format!("{path}.id"),
                    message: "action id must contain only ASCII letters, digits, '.', '_' or '-'"
                        .into(),
                });
            }
            if action.type_name.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "ACTION_TYPE_REQUIRED",
                    path: format!("{path}.type"),
                    message: "action type must not be empty".into(),
                });
            }
            reject_inline_secrets(&action.config, &format!("{path}.config"), &mut diagnostics);
        }

        for (index, hop) in self.hops.iter().enumerate() {
            let path = format!("hops[{index}]");
            if !ids.contains(hop.from.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "WORKFLOW_SOURCE_NOT_FOUND",
                    path: format!("{path}.from"),
                    message: format!("action '{}' does not exist", hop.from),
                });
            }
            if !ids.contains(hop.to.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "WORKFLOW_TARGET_NOT_FOUND",
                    path: format!("{path}.to"),
                    message: format!("action '{}' does not exist", hop.to),
                });
            }
            if !matches!(
                hop.evaluation.as_str(),
                "success" | "failure" | "unconditional"
            ) {
                diagnostics.push(Diagnostic {
                    code: "INVALID_WORKFLOW_EVALUATION",
                    path: format!("{path}.evaluation"),
                    message: "evaluation must be success, failure, or unconditional".into(),
                });
            }
        }
        diagnostics
    }
}

fn valid_stable_id(id: &str) -> bool {
    !id.is_empty()
        && id.trim() == id
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn reject_inline_secrets(value: &Value, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = format!("{path}.{}", key);
                let sensitive = matches!(
                    key.to_ascii_lowercase().as_str(),
                    "password"
                        | "passwd"
                        | "secret"
                        | "token"
                        | "api_key"
                        | "apikey"
                        | "authorization"
                );
                if sensitive && child.as_str().is_some_and(|value| !value.is_empty()) {
                    diagnostics.push(Diagnostic {
                        code: "INLINE_SECRET_FORBIDDEN",
                        path: child_path.clone(),
                        message: "use a secret reference instead of an inline secret value".into(),
                    });
                }
                reject_inline_secrets(child, &child_path, diagnostics);
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                reject_inline_secrets(child, &format!("{path}[{index}]"), diagnostics);
            }
        }
        _ => {}
    }
}

fn canonicalize(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut sorted = BTreeMap::new();
            for (key, child) in object {
                sorted.insert(key, canonicalize(child));
            }
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        other => other,
    }
}

fn diff_values(left: &Value, right: &Value, path: &str, paths: &mut Vec<String>) {
    match (left, right) {
        (Value::Object(left), Value::Object(right)) => {
            let keys = left
                .keys()
                .chain(right.keys())
                .collect::<std::collections::BTreeSet<_>>();
            for key in keys {
                let child = format!("{path}.{}", key);
                match (left.get(key), right.get(key)) {
                    (Some(a), Some(b)) => diff_values(a, b, &child, paths),
                    _ => paths.push(child),
                }
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                paths.push(path.to_owned());
            }
            for (index, (a, b)) in left.iter().zip(right.iter()).enumerate() {
                diff_values(a, b, &format!("{path}[{index}]"), paths);
            }
        }
        _ if left != right => paths.push(path.to_owned()),
        _ => {}
    }
}

fn has_cycle(node_ids: &HashSet<&str>, edges: &[Edge]) -> bool {
    let mut incoming = node_ids
        .iter()
        .map(|id| (*id, 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<&str, Vec<&str>>::new();
    for edge in edges {
        if incoming.contains_key(edge.from.as_str()) && incoming.contains_key(edge.to.as_str()) {
            outgoing
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
            if let Some(target) = incoming.get_mut(edge.to.as_str()) {
                *target += 1;
            }
        }
    }

    let mut queue = incoming
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect::<std::collections::VecDeque<_>>();
    let mut visited = 0usize;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        if let Some(targets) = outgoing.get(id) {
            for target in targets {
                if let Some(count) = incoming.get_mut(target) {
                    *count -= 1;
                    if *count == 0 {
                        queue.push_back(target);
                    }
                }
            }
        }
    }
    visited != node_ids.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str) -> Node {
        Node::new(id, "Dummy", [0.0, 0.0])
    }

    #[test]
    fn valid_document_round_trips_extensions() {
        let mut document = NativePipelineDocument::new("demo");
        document.nodes.push(node("a"));
        document.extensions.insert(
            "future_feature".into(),
            serde_json::json!({"enabled": true}),
        );

        let json = serde_json::to_string(&document).unwrap();
        let decoded: NativePipelineDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_value(&decoded).unwrap(),
            serde_json::to_value(&document).unwrap()
        );
        assert!(decoded.validate().is_empty());
    }

    #[test]
    fn canonical_json_is_stable_for_nested_object_keys() {
        let mut document = NativePipelineDocument::new("canonical");
        document.extensions.insert(
            "zeta".into(),
            serde_json::json!({"b": 2, "a": {"d": 4, "c": 3}}),
        );
        let first = document.to_canonical_json().unwrap();
        let decoded: NativePipelineDocument = serde_json::from_str(&first).unwrap();
        assert_eq!(first, decoded.to_canonical_json().unwrap());
        assert!(first.find("\"a\"").unwrap() < first.find("\"b\"").unwrap());
    }

    #[test]
    fn semantic_diff_reports_structural_paths() {
        let mut left = NativePipelineDocument::new("same");
        left.extensions
            .insert("meta".into(), serde_json::json!({"a": 1}));
        let mut right = left.clone();
        right.name = "changed".into();
        right
            .extensions
            .insert("meta".into(), serde_json::json!({"a": 2}));
        let paths = semantic_diff(&left, &right).unwrap();
        assert_eq!(paths, vec!["$.meta.a", "$.name"]);
    }

    #[test]
    fn migration_api_is_explicit_about_supported_versions() {
        let document = NativePipelineDocument::new("v1");
        assert!(migrate_pipeline_to_current(document).is_ok());
        let mut future = NativeWorkflowDocument::new("future");
        future.format_version = 2;
        assert!(migrate_workflow_to_current(future).is_err());
    }

    #[test]
    fn error_edges_round_trip_without_loss() {
        let mut document = NativePipelineDocument::new("error-route");
        document.nodes.extend([node("source"), node("errors")]);
        document.edges.push(Edge {
            from: "source".into(),
            to: "errors".into(),
            is_error: true,
        });
        let encoded = serde_json::to_string(&document).unwrap();
        let decoded: NativePipelineDocument = serde_json::from_str(&encoded).unwrap();
        assert!(decoded.validate().is_empty());
        assert!(decoded.edges[0].is_error);
    }

    #[test]
    fn diagnostics_are_stable_and_located() {
        let mut document = NativePipelineDocument::new("");
        document.nodes.push(node("a"));
        document.nodes.push(node("a"));
        document.edges.push(Edge {
            from: "a".into(),
            to: "missing".into(),
            is_error: false,
        });

        let diagnostics = document.validate();
        assert_eq!(diagnostics[0].code, "PIPELINE_NAME_REQUIRED");
        assert_eq!(diagnostics[0].path, "name");
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "DUPLICATE_NODE_ID" && d.path == "nodes[1].id")
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "EDGE_TARGET_NOT_FOUND" && d.path == "edges[0].to")
        );
    }

    #[test]
    fn stable_ids_reject_whitespace_and_unsafe_characters() {
        let mut document = NativePipelineDocument::new("ids");
        document.nodes.push(node("bad id"));
        assert!(
            document
                .validate()
                .iter()
                .any(|d| d.code == "INVALID_NODE_ID")
        );

        let mut workflow = NativeWorkflowDocument::new("ids");
        workflow.actions.push(NativeWorkflowAction {
            id: "bad/id".into(),
            type_name: "log".into(),
            config: Value::Null,
        });
        assert!(
            workflow
                .validate()
                .iter()
                .any(|d| d.code == "INVALID_ACTION_ID")
        );
    }

    #[test]
    fn inline_secret_values_are_rejected_in_configs() {
        let mut document = NativePipelineDocument::new("secrets");
        let mut source = node("source");
        source.config = serde_json::json!({
            "url": "https://example.invalid",
            "headers": {"Authorization": "Bearer secret-value"}
        });
        document.nodes.push(source);
        let diagnostics = document.validate();
        assert!(diagnostics.iter().any(|d| {
            d.code == "INLINE_SECRET_FORBIDDEN" && d.path == "nodes[0].config.headers.Authorization"
        }));
    }

    #[test]
    fn cycle_is_rejected() {
        let mut document = NativePipelineDocument::new("cycle");
        document.nodes.extend([node("a"), node("b")]);
        document.edges.extend([
            Edge {
                from: "a".into(),
                to: "b".into(),
                is_error: false,
            },
            Edge {
                from: "b".into(),
                to: "a".into(),
                is_error: false,
            },
        ]);
        assert!(
            document
                .validate()
                .iter()
                .any(|d| d.code == "CYCLE_DETECTED")
        );
    }

    #[test]
    fn workflow_preserves_hop_concepts_and_rejects_unknown_action() {
        let mut document = NativeWorkflowDocument::new("daily");
        document.actions.push(NativeWorkflowAction {
            id: "run_pipeline".into(),
            type_name: "pipeline".into(),
            config: serde_json::json!({"path": "pipelines/daily.ajp"}),
        });
        document.hops.push(NativeWorkflowHop {
            from: "run_pipeline".into(),
            to: "notify".into(),
            evaluation: "failure".into(),
            unconditional: false,
        });

        let diagnostics = document.validate();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "WORKFLOW_TARGET_NOT_FOUND")
        );
        assert_eq!(document.hops[0].evaluation, "failure");
    }
}
