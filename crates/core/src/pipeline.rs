use crate::transform::Transform;
use serde::{Deserialize, Serialize};

/// A directed edge between two transform nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hop {
    pub from: String,
    pub to: String,
    /// When true, this hop carries error rows instead of normal output rows.
    /// Error rows are emitted when the source transform's process() returns Err.
    #[serde(default)]
    pub is_error: bool,
}

/// A single node in the pipeline DAG
pub struct TransformNode {
    pub id: String,
    pub transform: Box<dyn Transform>,
}

/// The full pipeline definition
pub struct Pipeline {
    pub name: String,
    pub nodes: Vec<TransformNode>,
    pub hops: Vec<Hop>,
}

impl Pipeline {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            hops: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: impl Into<String>, transform: Box<dyn Transform>) {
        self.nodes.push(TransformNode {
            id: id.into(),
            transform,
        });
    }

    pub fn add_hop(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.hops.push(Hop {
            from: from.into(),
            to: to.into(),
            is_error: false,
        });
    }

    /// Add a hop that carries error rows — rows emitted when the source transform fails.
    pub fn add_error_hop(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.hops.push(Hop {
            from: from.into(),
            to: to.into(),
            is_error: true,
        });
    }

    /// Returns node IDs in topological order (sources first)
    pub fn topological_order(&self) -> Vec<String> {
        let all_ids: Vec<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        let has_incoming: std::collections::HashSet<String> =
            self.hops.iter().map(|h| h.to.clone()).collect();

        // Simple topo sort: sources first, then the rest in declaration order
        let mut order: Vec<String> = all_ids
            .iter()
            .filter(|id| !has_incoming.contains(*id))
            .cloned()
            .collect();

        let mut visited: std::collections::HashSet<String> = order.iter().cloned().collect();
        let mut queue = std::collections::VecDeque::from(order.clone());

        while let Some(current) = queue.pop_front() {
            for hop in &self.hops {
                if hop.from == current && !visited.contains(&hop.to) {
                    visited.insert(hop.to.clone());
                    order.push(hop.to.clone());
                    queue.push_back(hop.to.clone());
                }
            }
        }

        order
    }

    pub fn successors(&self, id: &str) -> Vec<&str> {
        self.hops
            .iter()
            .filter(|h| h.from == id)
            .map(|h| h.to.as_str())
            .collect()
    }

    pub fn predecessors(&self, id: &str) -> Vec<&str> {
        self.hops
            .iter()
            .filter(|h| h.to == id)
            .map(|h| h.from.as_str())
            .collect()
    }
}
