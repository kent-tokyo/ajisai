use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use rhai::{Dynamic, Engine, Scope, AST};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFieldSpec {
    pub name: String,
    pub field_type: ValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStepConfig {
    /// Rhai script source. Input row fields are available as variables.
    /// Assign values to `output_fields` names to produce output.
    pub script: String,
    #[serde(default)]
    pub output_fields: Vec<OutputFieldSpec>,
}

pub struct ScriptStep {
    config: ScriptStepConfig,
    engine: Option<Engine>,
    ast: Option<AST>,
    output_schema: Option<Arc<RowSchema>>,
}

impl ScriptStep {
    pub fn new(config: ScriptStepConfig) -> Self {
        Self { config, engine: None, ast: None, output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: ScriptStepConfig = serde_json::from_value(value)
            .map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

fn value_to_dynamic(v: &Value) -> Dynamic {
    match v {
        Value::Str(s) => Dynamic::from(s.clone()),
        Value::Int(n) => Dynamic::from(*n),
        Value::Float(f) => Dynamic::from(*f),
        Value::Bool(b) => Dynamic::from(*b),
        _ => Dynamic::UNIT,
    }
}

fn dynamic_to_value(d: Dynamic, vt: &ValueType) -> Value {
    if d.is_unit() {
        return Value::Null;
    }
    match vt {
        ValueType::Integer => {
            if let Ok(n) = d.as_int() {
                Value::Int(n)
            } else if let Ok(f) = d.as_float() {
                Value::Int(f as i64)
            } else {
                Value::Null
            }
        }
        ValueType::Float => {
            if let Ok(f) = d.as_float() {
                Value::Float(f)
            } else if let Ok(n) = d.as_int() {
                Value::Float(n as f64)
            } else {
                Value::Null
            }
        }
        ValueType::Boolean => d.as_bool().ok().map(Value::Bool).unwrap_or(Value::Null),
        ValueType::String => {
            if let Ok(s) = d.into_immutable_string() {
                Value::Str(s.to_string())
            } else {
                Value::Null
            }
        }
        _ => Value::Null,
    }
}

#[async_trait]
impl Transform for ScriptStep {
    fn name(&self) -> &str {
        "ScriptStep"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        for of in &self.config.output_fields {
            fields.push(Field::new(&of.name, of.field_type.clone()));
        }
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        let engine = Engine::new();
        let ast = engine
            .compile(&self.config.script)
            .map_err(|e| AjisaiError::Config(format!("Script compile error: {}", e)))?;
        self.engine = Some(engine);
        self.ast = Some(ast);
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let engine = self.engine.as_ref().expect("open() not called");
        let ast = self.ast.as_ref().expect("open() not called");

        let schema = self
            .output_schema
            .get_or_insert_with(|| {
                let mut fields = row.schema.fields.clone();
                for of in &self.config.output_fields {
                    fields.push(Field::new(&of.name, of.field_type.clone()));
                }
                Arc::new(RowSchema::new(fields))
            })
            .clone();

        let mut scope = Scope::new();
        for (field, value) in row.schema.fields.iter().zip(row.values.iter()) {
            scope.push_dynamic(field.name.clone(), value_to_dynamic(value));
        }

        engine
            .run_ast_with_scope(&mut scope, ast)
            .map_err(|e| AjisaiError::Pipeline(format!("Script error: {}", e)))?;

        let mut values = row.values.clone();
        for of in &self.config.output_fields {
            let d = scope
                .get_value::<Dynamic>(&of.name)
                .unwrap_or(Dynamic::UNIT);
            values.push(dynamic_to_value(d, &of.field_type));
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
    use std::sync::Arc;

    fn make_row(name: &str, age: i64) -> Row {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("name", ValueType::String),
            Field::new("age", ValueType::Integer),
        ]));
        Row::new(schema, vec![Value::Str(name.into()), Value::Int(age)])
    }

    #[tokio::test]
    async fn computes_new_field() {
        let mut t = ScriptStep::new(ScriptStepConfig {
            script: "let doubled = age * 2;".into(),
            output_fields: vec![OutputFieldSpec {
                name: "doubled".into(),
                field_type: ValueType::Integer,
            }],
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let out = t.process(make_row("Alice", 21)).await.unwrap();
        assert_eq!(out[0].get("doubled"), Some(&Value::Int(42)));
    }

    #[tokio::test]
    async fn compile_error_reported() {
        let mut t = ScriptStep::new(ScriptStepConfig {
            script: "let x = ;".into(),
            output_fields: vec![],
        });
        assert!(t.open(&ExecutionContext::new()).await.is_err());
    }
}
