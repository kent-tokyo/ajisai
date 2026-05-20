use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Row, RowSchema},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSpec {
    /// Value to compare against (string comparison via to_display_string())
    pub value: String,
    /// Target transform name (must match the downstream node's id)
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchCaseConfig {
    /// Field whose value is used for routing
    pub field_name: String,
    pub cases: Vec<CaseSpec>,
    /// Fallback target if no case matches; None = broadcast
    pub default_target: Option<String>,
}

pub struct SwitchCase {
    config: SwitchCaseConfig,
}

impl SwitchCase {
    pub fn new(config: SwitchCaseConfig) -> Self {
        Self { config }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: SwitchCaseConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for SwitchCase {
    fn name(&self) -> &str {
        "SwitchCase"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn route(&self, row: &Row) -> Option<String> {
        let field_val = row
            .get(&self.config.field_name)
            .map(|v| v.to_display_string())
            .unwrap_or_default();

        for case in &self.config.cases {
            if case.value == field_val {
                return Some(case.target.clone());
            }
        }
        self.config.default_target.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};
    use std::sync::Arc;

    fn make_row(status: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new(
            "status",
            ValueType::String,
        )]));
        Row::new(schema, vec![Value::Str(status.into())])
    }

    #[tokio::test]
    async fn routes_to_matching_case() {
        let t = SwitchCase::new(SwitchCaseConfig {
            field_name: "status".into(),
            cases: vec![
                CaseSpec {
                    value: "active".into(),
                    target: "ActiveOutput".into(),
                },
                CaseSpec {
                    value: "inactive".into(),
                    target: "InactiveOutput".into(),
                },
            ],
            default_target: Some("DefaultOutput".into()),
        });

        assert_eq!(t.route(&make_row("active")), Some("ActiveOutput".into()));
        assert_eq!(
            t.route(&make_row("inactive")),
            Some("InactiveOutput".into())
        );
        assert_eq!(t.route(&make_row("unknown")), Some("DefaultOutput".into()));
    }

    #[tokio::test]
    async fn broadcasts_when_no_default() {
        let t = SwitchCase::new(SwitchCaseConfig {
            field_name: "status".into(),
            cases: vec![CaseSpec {
                value: "active".into(),
                target: "ActiveOutput".into(),
            }],
            default_target: None,
        });
        assert_eq!(t.route(&make_row("other")), None);
    }

    #[tokio::test]
    async fn process_passes_rows_through() {
        let mut t = SwitchCase::new(SwitchCaseConfig {
            field_name: "status".into(),
            cases: vec![],
            default_target: None,
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let row = make_row("active");
        let out = t.process(row).await.unwrap();
        assert_eq!(out.len(), 1);
    }
}
