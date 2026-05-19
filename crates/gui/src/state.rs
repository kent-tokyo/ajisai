use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Transforms grouped by category for the palette
pub const TRANSFORM_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    (
        "I/O",
        &[
            ("CsvFileInput", "CSV Input"),
            ("CsvFileOutput", "CSV Output"),
            ("JsonFileInput", "JSON Input"),
            ("JsonFileOutput", "JSON Output"),
            ("TableInput", "Table Input"),
            ("TableOutput", "Table Output"),
        ],
    ),
    (
        "Transform",
        &[
            ("FilterRows", "Filter Rows"),
            ("SelectValues", "Select Values"),
            ("SortRows", "Sort Rows"),
            ("AddConstants", "Add Constants"),
            ("CalculatorStep", "Calculator"),
            ("Deduplicate", "Deduplicate"),
        ],
    ),
    (
        "Join / Lookup",
        &[
            ("StreamLookup", "Stream Lookup"),
            ("MergeJoin", "Merge Join"),
            ("DatabaseLookup", "DB Lookup"),
        ],
    ),
];

/// Flat list kept for compatibility with display_name()
pub const TRANSFORM_TYPES: &[(&str, &str)] = &[
    ("CsvFileInput", "CSV Input"),
    ("CsvFileOutput", "CSV Output"),
    ("JsonFileInput", "JSON Input"),
    ("JsonFileOutput", "JSON Output"),
    ("TableInput", "Table Input"),
    ("TableOutput", "Table Output"),
    ("FilterRows", "Filter Rows"),
    ("SelectValues", "Select Values"),
    ("SortRows", "Sort Rows"),
    ("AddConstants", "Add Constants"),
    ("CalculatorStep", "Calculator"),
    ("StreamLookup", "Stream Lookup"),
    ("MergeJoin", "Merge Join"),
    ("Deduplicate", "Deduplicate"),
    ("DatabaseLookup", "DB Lookup"),
];

/// Execution status of a node, shown as a colored dot overlay
#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Idle,
    Running,
    Done,
    Error,
}

/// Snapshot-based undo/redo stack (max 50 entries)
#[derive(Debug, Default)]
pub struct UndoStack {
    past: Vec<PipelineState>,
    future: Vec<PipelineState>,
}

impl UndoStack {
    const MAX: usize = 50;

    pub fn push(&mut self, state: PipelineState) {
        self.past.push(state);
        if self.past.len() > Self::MAX {
            self.past.remove(0);
        }
        self.future.clear();
    }

    pub fn undo(&mut self, current: PipelineState) -> Option<PipelineState> {
        self.past.pop().map(|prev| {
            self.future.push(current);
            prev
        })
    }

    pub fn redo(&mut self, current: PipelineState) -> Option<PipelineState> {
        self.future.pop().map(|next| {
            self.past.push(current);
            next
        })
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
}

/// A node placed on the pipeline canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub type_name: String,
    pub label: String,
    pub pos: [f32; 2],
    /// JSON config for this transform (edited via property panel)
    pub config: serde_json::Value,
}

impl Node {
    pub fn new(id: impl Into<String>, type_name: impl Into<String>, pos: [f32; 2]) -> Self {
        let type_name = type_name.into();
        let label = type_name.clone();
        Self {
            id: id.into(),
            type_name,
            label,
            pos,
            config: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn display_name(&self) -> &str {
        TRANSFORM_TYPES
            .iter()
            .find(|(t, _)| *t == self.type_name)
            .map(|(_, n)| *n)
            .unwrap_or(&self.type_name)
    }
}

/// A directed connection between two nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

/// The full pipeline editor state (serializable → save/load as JSON)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineState {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// Monotonically increasing counter — never reused even after node deletion
    #[serde(default)]
    node_seq: u64,
}

impl PipelineState {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            node_seq: 0,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        let e = Edge {
            from: from.into(),
            to: to.into(),
        };
        if !self.edges.contains(&e) {
            self.edges.push(e);
        }
    }

    pub fn remove_edge(&mut self, from: &str, to: &str) {
        self.edges.retain(|e| !(e.from == from && e.to == to));
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Generate a unique node id. Counter only ever increases,
    /// so deletion + addition never produces a duplicate.
    pub fn next_id(&mut self) -> String {
        loop {
            self.node_seq += 1;
            let candidate = format!("node_{}", self.node_seq);
            if !self.nodes.iter().any(|n| n.id == candidate) {
                return candidate;
            }
        }
    }
}

/// Runtime state (not serialized)
#[derive(Debug, Default)]
pub struct UiState {
    pub selected_node: Option<String>,
    /// Node being connected: Some(from_id) while dragging an edge
    pub connecting_from: Option<String>,
    pub log_lines: Vec<String>,
    pub pipeline_running: bool,
    pub current_file: Option<std::path::PathBuf>,
    /// Per-node execution status shown as color dots on the canvas
    pub node_status: HashMap<String, NodeStatus>,
    /// Category names that are currently collapsed in the palette
    pub collapsed_categories: HashSet<String>,
}

impl UiState {
    pub fn log(&mut self, msg: impl Into<String>) {
        self.log_lines.push(msg.into());
        if self.log_lines.len() > 500 {
            self.log_lines.drain(0..100);
        }
    }
}
