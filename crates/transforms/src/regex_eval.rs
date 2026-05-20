use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexEvalConfig {
    /// Source field to match against
    pub field: String,
    /// Regular expression (capture groups map to output_fields positionally,
    /// or named groups are used when output_fields is empty)
    pub pattern: String,
    /// Output field names for each capture group (positional, group 1 onwards).
    /// Named groups: if empty, group names from the regex become field names.
    #[serde(default)]
    pub output_fields: Vec<String>,
    /// If true, rows that do not match are dropped; otherwise they pass through
    /// with empty strings in the output fields.
    #[serde(default)]
    pub drop_unmatched: bool,
}

pub struct RegexEval {
    config: RegexEvalConfig,
    regex: Option<Regex>,
    output_schema: Option<Arc<RowSchema>>,
}

impl RegexEval {
    pub fn new(config: RegexEvalConfig) -> Self {
        Self { config, regex: None, output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: RegexEvalConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn output_field_names(&self) -> Vec<String> {
        if !self.config.output_fields.is_empty() {
            return self.config.output_fields.clone();
        }
        // Fall back to named capture groups in pattern order
        if let Some(re) = &self.regex {
            return re
                .capture_names()
                .flatten()
                .map(|s| s.to_owned())
                .collect();
        }
        vec![]
    }
}

#[async_trait]
impl Transform for RegexEval {
    fn name(&self) -> &str {
        "RegexEval"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let re = Regex::new(&self.config.pattern)
            .map_err(|e| AjisaiError::Config(format!("Invalid regex: {}", e)))?;
        let names: Vec<String> = if !self.config.output_fields.is_empty() {
            self.config.output_fields.clone()
        } else {
            re.capture_names().flatten().map(|s| s.to_owned()).collect()
        };
        let mut fields = input.fields.clone();
        for name in &names {
            fields.push(Field::new(name.as_str(), ValueType::String));
        }
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.regex = Some(
            Regex::new(&self.config.pattern)
                .map_err(|e| AjisaiError::Config(format!("Invalid regex: {}", e)))?,
        );
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let re = self.regex.as_ref().unwrap();
        let out_names = self.output_field_names();

        let schema = self
            .output_schema
            .get_or_insert_with(|| {
                let mut fields = row.schema.fields.clone();
                for name in &out_names {
                    fields.push(Field::new(name.as_str(), ValueType::String));
                }
                Arc::new(RowSchema::new(fields))
            })
            .clone();

        let raw = row
            .get(&self.config.field)
            .map(|v| v.to_display_string())
            .unwrap_or_default();

        let captures = re.captures(&raw);
        if captures.is_none() && self.config.drop_unmatched {
            return Ok(vec![]);
        }

        let mut values = row.values.clone();
        for (i, _) in out_names.iter().enumerate() {
            let extracted = captures
                .as_ref()
                .and_then(|c| {
                    if self.config.output_fields.is_empty() {
                        // Named groups: use name
                        re.capture_names()
                            .flatten()
                            .nth(i)
                            .and_then(|name| c.name(name))
                            .map(|m| m.as_str().to_owned())
                    } else {
                        // Positional groups: group index = i + 1
                        c.get(i + 1).map(|m| m.as_str().to_owned())
                    }
                })
                .unwrap_or_default();
            values.push(Value::Str(extracted));
        }

        Ok(vec![Row::new(schema, values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, Value, ValueType};

    fn make_row(email: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![Field::new("email", ValueType::String)]));
        Row::new(schema, vec![Value::Str(email.into())])
    }

    #[tokio::test]
    async fn extracts_capture_groups() {
        let mut t = RegexEval::new(RegexEvalConfig {
            field: "email".into(),
            pattern: r"([^@]+)@(.+)".into(),
            output_fields: vec!["local".into(), "domain".into()],
            drop_unmatched: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row("alice@example.com")).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("local"), Some(&Value::Str("alice".into())));
        assert_eq!(out[0].get("domain"), Some(&Value::Str("example.com".into())));
    }

    #[tokio::test]
    async fn no_match_gives_empty_strings_by_default() {
        let mut t = RegexEval::new(RegexEvalConfig {
            field: "email".into(),
            pattern: r"([^@]+)@(.+)".into(),
            output_fields: vec!["local".into(), "domain".into()],
            drop_unmatched: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row("not-an-email")).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("local"), Some(&Value::Str("".into())));
    }

    #[tokio::test]
    async fn drop_unmatched_filters_row() {
        let mut t = RegexEval::new(RegexEvalConfig {
            field: "email".into(),
            pattern: r"([^@]+)@(.+)".into(),
            output_fields: vec!["local".into(), "domain".into()],
            drop_unmatched: true,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row("not-an-email")).await.unwrap();
        assert_eq!(out.len(), 0);
    }

    #[tokio::test]
    async fn named_groups_become_field_names() {
        let mut t = RegexEval::new(RegexEvalConfig {
            field: "email".into(),
            pattern: r"(?P<local>[^@]+)@(?P<domain>.+)".into(),
            output_fields: vec![],
            drop_unmatched: false,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row("bob@test.org")).await.unwrap();
        assert_eq!(out[0].get("local"), Some(&Value::Str("bob".into())));
        assert_eq!(out[0].get("domain"), Some(&Value::Str("test.org".into())));
    }
}
