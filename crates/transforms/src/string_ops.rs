use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ── StringOperations ──────────────────────────────────────────────────────────

fn default_pad_char() -> String {
    " ".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum StringOp {
    Trim,
    TrimLeft,
    TrimRight,
    Uppercase,
    Lowercase,
    PadLeft {
        length: usize,
        #[serde(default = "default_pad_char")]
        pad_char: String,
    },
    PadRight {
        length: usize,
        #[serde(default = "default_pad_char")]
        pad_char: String,
    },
    Substring {
        start: usize,
        length: Option<usize>,
    },
}

impl StringOp {
    fn apply(&self, s: &str) -> String {
        match self {
            StringOp::Trim => s.trim().to_owned(),
            StringOp::TrimLeft => s.trim_start().to_owned(),
            StringOp::TrimRight => s.trim_end().to_owned(),
            StringOp::Uppercase => s.to_uppercase(),
            StringOp::Lowercase => s.to_lowercase(),
            StringOp::PadLeft { length, pad_char } => {
                let ch = pad_char.chars().next().unwrap_or(' ');
                let chars: Vec<char> = s.chars().collect();
                if chars.len() >= *length {
                    return s.to_owned();
                }
                let padding: String = std::iter::repeat(ch).take(length - chars.len()).collect();
                format!("{}{}", padding, s)
            }
            StringOp::PadRight { length, pad_char } => {
                let ch = pad_char.chars().next().unwrap_or(' ');
                let chars: Vec<char> = s.chars().collect();
                if chars.len() >= *length {
                    return s.to_owned();
                }
                let padding: String = std::iter::repeat(ch).take(length - chars.len()).collect();
                format!("{}{}", s, padding)
            }
            StringOp::Substring { start, length } => {
                let chars: Vec<char> = s.chars().collect();
                let start = (*start).min(chars.len());
                let end = length
                    .map(|l| (start + l).min(chars.len()))
                    .unwrap_or(chars.len());
                chars[start..end].iter().collect()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringOperation {
    pub field: String,
    pub op: StringOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringOperationsConfig {
    pub operations: Vec<StringOperation>,
}

pub struct StringOperations {
    config: StringOperationsConfig,
}

impl StringOperations {
    pub fn new(config: StringOperationsConfig) -> Self {
        Self { config }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: StringOperationsConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for StringOperations {
    fn name(&self) -> &str {
        "StringOperations"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let mut values = row.values.clone();
        for op in &self.config.operations {
            if let Some(idx) = row.schema.fields.iter().position(|f| f.name == op.field) {
                let s = match &values[idx] {
                    Value::Str(s) => s.clone(),
                    Value::Null => continue,
                    other => other.to_display_string(),
                };
                values[idx] = Value::Str(op.op.apply(&s));
            }
        }
        Ok(vec![Row::new(row.schema.clone(), values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── ReplaceInString ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringReplacement {
    pub field: String,
    pub search: String,
    pub replace_with: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceInStringConfig {
    pub replacements: Vec<StringReplacement>,
}

pub struct ReplaceInString {
    config: ReplaceInStringConfig,
}

impl ReplaceInString {
    pub fn new(config: ReplaceInStringConfig) -> Self {
        Self { config }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ReplaceInStringConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for ReplaceInString {
    fn name(&self) -> &str {
        "ReplaceInString"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        Ok(input.clone())
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let mut values = row.values.clone();
        for rep in &self.config.replacements {
            if let Some(idx) = row.schema.fields.iter().position(|f| f.name == rep.field) {
                if let Value::Str(s) = &values[idx] {
                    values[idx] = Value::Str(s.replace(&rep.search, &rep.replace_with));
                }
            }
        }
        Ok(vec![Row::new(row.schema.clone(), values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── ConcatFields ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcatFieldsConfig {
    pub fields: Vec<String>,
    #[serde(default)]
    pub separator: String,
    pub output_field: String,
}

pub struct ConcatFields {
    config: ConcatFieldsConfig,
    output_schema: Option<Arc<RowSchema>>,
}

impl ConcatFields {
    pub fn new(config: ConcatFieldsConfig) -> Self {
        Self {
            config,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ConcatFieldsConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for ConcatFields {
    fn name(&self) -> &str {
        "ConcatFields"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        fields.push(Field::new(self.config.output_field.clone(), ValueType::String));
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.output_schema = None;
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let schema = match &self.output_schema {
            Some(s) => s.clone(),
            None => {
                let s = Arc::new(self.output_schema(&row.schema)?);
                self.output_schema = Some(s.clone());
                s
            }
        };

        let parts: Vec<String> = self
            .config
            .fields
            .iter()
            .map(|f| {
                row.get(f)
                    .map(|v| v.to_display_string())
                    .unwrap_or_default()
            })
            .collect();

        let concatenated = parts.join(&self.config.separator);
        let mut values = row.values.clone();
        values.push(Value::Str(concatenated));
        Ok(vec![Row::new(schema, values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ajisai_core::value::{Field, RowSchema, ValueType};
    use std::sync::Arc;

    fn str_row(a: &str, b: &str) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("a", ValueType::String),
            Field::new("b", ValueType::String),
        ]));
        Row::new(
            schema,
            vec![Value::Str(a.into()), Value::Str(b.into())],
        )
    }

    #[tokio::test]
    async fn string_ops_trim_and_upper() {
        let mut t = StringOperations::new(StringOperationsConfig {
            operations: vec![
                StringOperation {
                    field: "a".into(),
                    op: StringOp::Trim,
                },
                StringOperation {
                    field: "b".into(),
                    op: StringOp::Uppercase,
                },
            ],
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(str_row("  hello  ", "world")).await.unwrap();
        assert_eq!(out[0].get("a"), Some(&Value::Str("hello".into())));
        assert_eq!(out[0].get("b"), Some(&Value::Str("WORLD".into())));
    }

    #[tokio::test]
    async fn pad_left() {
        let mut t = StringOperations::new(StringOperationsConfig {
            operations: vec![StringOperation {
                field: "a".into(),
                op: StringOp::PadLeft {
                    length: 5,
                    pad_char: "0".into(),
                },
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(str_row("42", "")).await.unwrap();
        assert_eq!(out[0].get("a"), Some(&Value::Str("00042".into())));
    }

    #[tokio::test]
    async fn replace_in_string() {
        let mut t = ReplaceInString::new(ReplaceInStringConfig {
            replacements: vec![StringReplacement {
                field: "a".into(),
                search: "hello".into(),
                replace_with: "hi".into(),
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(str_row("hello world", "")).await.unwrap();
        assert_eq!(out[0].get("a"), Some(&Value::Str("hi world".into())));
    }

    #[tokio::test]
    async fn concat_fields() {
        let mut t = ConcatFields::new(ConcatFieldsConfig {
            fields: vec!["a".into(), "b".into()],
            separator: " ".into(),
            output_field: "full".into(),
        });
        t.open(&ExecutionContext::new()).await.unwrap();
        let out = t.process(str_row("Alice", "Smith")).await.unwrap();
        assert_eq!(out[0].get("full"), Some(&Value::Str("Alice Smith".into())));
    }
}
