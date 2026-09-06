use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Versioned, machine-readable execution event. Human-readable logs are a
/// rendering of these records; consumers should key on `kind` and IDs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunEvent {
    pub schema_version: u16,
    pub kind: String,
    pub pipeline: String,
    pub node_id: Option<String>,
    pub status: String,
    pub rows: Option<u64>,
    pub elapsed_ms: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionMeta {
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
}

impl fmt::Debug for ConnectionMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionMeta")
            .field("name", &self.name)
            .field("db_type", &self.db_type)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}

/// Runtime context passed to every Transform during execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub variables: HashMap<String, String>,
    shared_variables: Arc<std::sync::Mutex<HashMap<String, String>>>,
    pub connections: HashMap<String, ConnectionMeta>,
    cancelled: Arc<AtomicBool>,
    row_limit: Option<u64>,
    rows_seen: Arc<AtomicU64>,
    buffered_row_limit: Option<u64>,
    buffered_rows: Arc<AtomicU64>,
    deadline: Option<Arc<Instant>>,
    network_allowed: bool,
    project_root: Option<std::path::PathBuf>,
    events: Arc<std::sync::Mutex<Vec<RunEvent>>>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            shared_variables: Arc::new(std::sync::Mutex::new(HashMap::new())),
            connections: HashMap::new(),
            cancelled: Arc::new(AtomicBool::new(false)),
            row_limit: None,
            rows_seen: Arc::new(AtomicU64::new(0)),
            buffered_row_limit: None,
            buffered_rows: Arc::new(AtomicU64::new(0)),
            deadline: None,
            network_allowed: true,
            project_root: None,
            events: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        self.variables.insert(key.clone(), value.clone());
        if let Ok(mut shared) = self.shared_variables.lock() {
            shared.insert(key, value);
        }
    }

    /// Set a variable from a running transform or workflow action.
    ///
    /// This updates the clone-shared store without mutating process-global
    /// environment variables, keeping execution deterministic and safe.
    pub fn set_var_shared(&self, key: impl Into<String>, value: impl Into<String>) {
        if let Ok(mut shared) = self.shared_variables.lock() {
            shared.insert(key.into(), value.into());
        }
    }

    pub fn get_var(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(String::as_str)
    }

    /// Request cooperative cancellation for all transforms sharing this context.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Return whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn set_row_limit(&mut self, limit: Option<u64>) {
        self.row_limit = limit;
    }

    pub fn try_accept_row(&self) -> bool {
        let Some(limit) = self.row_limit else {
            return true;
        };
        self.rows_seen
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |seen| {
                (seen < limit).then_some(seen + 1)
            })
            .is_ok()
    }

    /// Set the maximum number of rows a blocking transform may retain.
    pub fn set_buffered_row_limit(&mut self, limit: Option<u64>) {
        self.buffered_row_limit = limit;
    }

    /// Reserve one row in a transform-owned buffer under the execution policy.
    pub fn try_buffer_row(&self) -> bool {
        let Some(limit) = self.buffered_row_limit else {
            return true;
        };
        self.buffered_rows
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |seen| {
                (seen < limit).then_some(seen + 1)
            })
            .is_ok()
    }

    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.deadline = timeout.map(|duration| Arc::new(Instant::now() + duration));
    }

    pub fn is_timed_out(&self) -> bool {
        self.deadline
            .as_ref()
            .is_some_and(|deadline| Instant::now() >= **deadline)
    }

    pub fn set_network_allowed(&mut self, allowed: bool) {
        self.network_allowed = allowed;
    }

    pub fn network_allowed(&self) -> bool {
        self.network_allowed
    }

    /// Restrict filesystem-capable transforms to this canonical project root.
    pub fn set_project_root(&mut self, root: impl Into<std::path::PathBuf>) {
        self.project_root = Some(root.into());
    }

    /// Return the configured filesystem root, if the run has one.
    pub fn project_root(&self) -> Option<&std::path::Path> {
        self.project_root.as_deref()
    }

    /// Append a structured event for observability clients.
    pub fn record_event(&self, event: RunEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    /// Return a point-in-time copy of events emitted by this run.
    pub fn events(&self) -> Vec<RunEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    /// Resolve ${VAR} references in a string
    pub fn resolve(&self, s: &str) -> String {
        let mut result = s.to_owned();
        if let Ok(shared) = self.shared_variables.lock() {
            for (k, v) in shared.iter() {
                result = result.replace(&format!("${{{}}}", k), v);
            }
        }
        for (k, v) in &self.variables {
            result = result.replace(&format!("${{{}}}", k), v);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{ConnectionMeta, ExecutionContext};
    use std::time::Duration;

    #[test]
    fn cancellation_is_shared_across_clones() {
        let context = ExecutionContext::new();
        let clone = context.clone();
        assert!(!clone.is_cancelled());
        context.cancel();
        assert!(clone.is_cancelled());
    }

    #[test]
    fn row_limit_is_shared_across_clones() {
        let mut context = ExecutionContext::new();
        context.set_row_limit(Some(2));
        let clone = context.clone();
        assert!(context.try_accept_row());
        assert!(clone.try_accept_row());
        assert!(!context.try_accept_row());
    }

    #[test]
    fn buffered_row_limit_is_shared_across_clones() {
        let mut context = ExecutionContext::new();
        context.set_buffered_row_limit(Some(2));
        let clone = context.clone();
        assert!(context.try_buffer_row());
        assert!(clone.try_buffer_row());
        assert!(!context.try_buffer_row());
    }

    #[test]
    fn workflow_variables_are_shared_without_process_environment_mutation() {
        let context = ExecutionContext::new();
        let clone = context.clone();
        context.set_var_shared("AJISAI_WORKFLOW_TOKEN", "redacted");
        assert_eq!(clone.resolve("${AJISAI_WORKFLOW_TOKEN}"), "redacted");
    }

    #[test]
    fn timeout_is_shared_across_clones() {
        let mut context = ExecutionContext::new();
        context.set_timeout(Some(Duration::ZERO));
        assert!(context.clone().is_timed_out());
    }

    #[test]
    fn connection_password_is_not_serialized() {
        let meta = ConnectionMeta {
            name: "db".into(),
            db_type: "sqlite".into(),
            host: "localhost".into(),
            port: 0,
            database: "data".into(),
            username: "user".into(),
            password: "super-secret".into(),
        };
        let encoded = serde_json::to_string(&meta).unwrap();
        assert!(!encoded.contains("super-secret"));
        assert!(!encoded.contains("password"));
    }
}
