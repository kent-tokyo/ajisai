use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMeta {
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

/// Runtime context passed to every Transform during execution
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub variables: HashMap<String, String>,
    pub connections: HashMap<String, ConnectionMeta>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    pub fn get_var(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(String::as_str)
    }

    /// Resolve ${VAR} references in a string
    pub fn resolve(&self, s: &str) -> String {
        let mut result = s.to_owned();
        for (k, v) in &self.variables {
            result = result.replace(&format!("${{{}}}", k), v);
        }
        result
    }
}
