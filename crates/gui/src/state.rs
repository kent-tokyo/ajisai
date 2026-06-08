pub use ajisai_core::model::{Node, PipelineState, TRANSFORM_CATEGORIES};

use std::collections::{HashMap, HashSet, VecDeque};

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
    past: VecDeque<PipelineState>,
    future: Vec<PipelineState>,
}

impl UndoStack {
    const MAX: usize = 50;

    pub fn push(&mut self, state: PipelineState) {
        self.past.push_back(state);
        if self.past.len() > Self::MAX {
            self.past.pop_front();
        }
        self.future.clear();
    }

    pub fn undo(&mut self, current: PipelineState) -> Option<PipelineState> {
        self.past.pop_back().map(|prev| {
            self.future.push(current);
            prev
        })
    }

    pub fn redo(&mut self, current: PipelineState) -> Option<PipelineState> {
        self.future.pop().map(|next| {
            self.past.push_back(current);
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
    /// Persistent raw-JSON edit buffer for the json_fallback editor, keyed by node id
    pub json_edit_buf: HashMap<String, String>,
}

impl UiState {
    pub fn log(&mut self, msg: impl Into<String>) {
        self.log_lines.push(msg.into());
        if self.log_lines.len() > 500 {
            self.log_lines.drain(0..100);
        }
    }
}
