use crate::error::{AjisaiError, Result};
use serde::{Deserialize, Serialize};

/// A single action in a workflow
pub struct WorkflowAction {
    pub name:      String,
    pub action:    Box<dyn Action>,
}

/// Workflow is a DAG of actions with success/failure/unconditional hops
pub struct Workflow {
    pub name:    String,
    pub actions: Vec<WorkflowAction>,
    pub hops:    Vec<WorkflowHop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHop {
    pub from:          String,
    pub to:            String,
    pub evaluation:    HopEvaluation,
    pub unconditional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HopEvaluation {
    /// Follow when the source action succeeded
    Success,
    /// Follow when the source action failed
    Failure,
    /// Always follow regardless of outcome
    Unconditional,
}

/// Result of running a single action
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
}

impl ActionResult {
    pub fn ok() -> Self { Self { success: true, message: None } }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, message: Some(msg.into()) }
    }
}

#[async_trait::async_trait]
pub trait Action: Send {
    fn name(&self) -> &str;
    async fn execute<'a>(&'a mut self, ctx: &'a crate::context::ExecutionContext) -> ActionResult;
}

impl Workflow {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), actions: Vec::new(), hops: Vec::new() }
    }

    pub fn add_action(&mut self, name: impl Into<String>, action: Box<dyn Action>) {
        self.actions.push(WorkflowAction { name: name.into(), action });
    }

    pub fn add_hop(&mut self, from: impl Into<String>, to: impl Into<String>, evaluation: HopEvaluation, unconditional: bool) {
        self.hops.push(WorkflowHop {
            from: from.into(),
            to: to.into(),
            evaluation,
            unconditional,
        });
    }

    fn start_actions(&self) -> Vec<&str> {
        let has_incoming: std::collections::HashSet<&str> =
            self.hops.iter().map(|h| h.to.as_str()).collect();
        self.actions.iter()
            .map(|a| a.name.as_str())
            .filter(|n| !has_incoming.contains(n))
            .collect()
    }

    fn next_actions(&self, from: &str, success: bool) -> Vec<&str> {
        self.hops.iter()
            .filter(|h| h.from == from)
            .filter(|h| {
                h.unconditional
                    || h.evaluation == HopEvaluation::Unconditional
                    || (success && h.evaluation == HopEvaluation::Success)
                    || (!success && h.evaluation == HopEvaluation::Failure)
            })
            .map(|h| h.to.as_str())
            .collect()
    }
}

pub struct WorkflowEngine {
    pub workflow: Workflow,
    pub context:  crate::context::ExecutionContext,
}

impl WorkflowEngine {
    pub fn new(workflow: Workflow, context: crate::context::ExecutionContext) -> Self {
        Self { workflow, context }
    }

    pub async fn run(mut self) -> Result<WorkflowStats> {
        let start = std::time::Instant::now();
        tracing::info!("Running workflow '{}'", self.workflow.name);

        let mut pending: Vec<String> = self.workflow.start_actions()
            .into_iter().map(String::from).collect();
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut total_actions = 0usize;
        let mut failed_actions = 0usize;

        while !pending.is_empty() {
            let current_name = pending.remove(0);
            if !visited.insert(current_name.clone()) {
                continue; // already executed
            }

            let action = self.workflow.actions.iter_mut()
                .find(|a| a.name == current_name)
                .ok_or_else(|| AjisaiError::Pipeline(
                    format!("Workflow action '{}' not found", current_name)
                ))?;

            tracing::info!("Executing action '{}'", current_name);
            let result = action.action.execute(&self.context).await;
            total_actions += 1;

            if !result.success {
                failed_actions += 1;
                if let Some(msg) = &result.message {
                    tracing::error!("Action '{}' failed: {}", current_name, msg);
                }
            }

            let nexts = self.workflow.next_actions(&current_name, result.success);
            for next in nexts {
                if !visited.contains(next) {
                    pending.push(next.to_owned());
                }
            }
        }

        Ok(WorkflowStats {
            total_actions,
            failed_actions,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
}

#[derive(Debug, Default)]
pub struct WorkflowStats {
    pub total_actions:  usize,
    pub failed_actions: usize,
    pub elapsed_ms:     u64,
}
