use crate::{transform::TransformFactory, AjisaiError, Result};
use std::collections::HashMap;

/// Maps transform type names to their factory functions.
/// Lives in ajisai-core so both ajisai-transforms and ajisai-hop-compat can reference it
/// without creating a circular dependency.
pub struct TransformRegistry {
    factories: HashMap<String, TransformFactory>,
}

impl TransformRegistry {
    pub fn new() -> Self {
        Self { factories: HashMap::new() }
    }

    pub fn register(&mut self, type_name: impl Into<String>, factory: TransformFactory) {
        self.factories.insert(type_name.into(), factory);
    }

    pub fn create(
        &self,
        type_name: &str,
        config: serde_json::Value,
    ) -> Result<Box<dyn crate::Transform>> {
        let factory = self.factories.get(type_name).ok_or_else(|| {
            AjisaiError::Config(format!(
                "Unknown transform type: '{}'. Available: {}",
                type_name,
                self.list().join(", ")
            ))
        })?;
        factory(config)
    }

    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.factories.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}

impl Default for TransformRegistry {
    fn default() -> Self {
        Self::new()
    }
}
