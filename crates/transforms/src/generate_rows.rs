use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateFieldSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: ValueType,
    /// Fixed value string; use "${ROW_NR}" to embed the 1-based row number.
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRowsConfig {
    pub fields: Vec<GenerateFieldSpec>,
    pub limit: u64,
}

pub struct GenerateRows {
    config: GenerateRowsConfig,
    schema: Option<Arc<RowSchema>>,
}

impl GenerateRows {
    pub fn new(config: GenerateRowsConfig) -> Self {
        Self {
            config,
            schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: GenerateRowsConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    fn build_schema(&self) -> Arc<RowSchema> {
        let fields = self
            .config
            .fields
            .iter()
            .map(|f| Field::new(f.name.as_str(), f.value_type.clone()))
            .collect();
        Arc::new(RowSchema::new(fields))
    }

    fn make_value(spec: &GenerateFieldSpec, row_nr: u64) -> Value {
        let raw = spec.value.replace("${ROW_NR}", &row_nr.to_string());
        match spec.value_type {
            ValueType::Integer => raw.parse::<i64>().map(Value::Int).unwrap_or(Value::Null),
            ValueType::Float => raw.parse::<f64>().map(Value::Float).unwrap_or(Value::Null),
            ValueType::Boolean => {
                Value::Bool(matches!(raw.to_lowercase().as_str(), "true" | "1" | "yes"))
            }
            _ => Value::Str(raw),
        }
    }
}

#[async_trait]
impl Transform for GenerateRows {
    fn name(&self) -> &str {
        "GenerateRows"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        Ok((*self.build_schema()).clone())
    }

    fn is_source(&self) -> bool {
        true
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.schema = Some(self.build_schema());
        Ok(())
    }

    async fn produce(&mut self, sender: Sender<Row>) -> Result<()> {
        let schema = self.schema.clone().expect("open() not called");
        for i in 1..=self.config.limit {
            let values = self
                .config
                .fields
                .iter()
                .map(|f| Self::make_value(f, i))
                .collect();
            let row = Row::new(schema.clone(), values);
            if sender.send(row).await.is_err() {
                break;
            }
        }
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        Ok(vec![row])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn generates_fixed_rows() {
        let mut t = GenerateRows::new(GenerateRowsConfig {
            fields: vec![
                GenerateFieldSpec {
                    name: "id".into(),
                    value_type: ValueType::Integer,
                    value: "${ROW_NR}".into(),
                },
                GenerateFieldSpec {
                    name: "label".into(),
                    value_type: ValueType::String,
                    value: "hello".into(),
                },
            ],
            limit: 3,
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::channel(16);
        t.produce(tx).await.unwrap();

        let mut rows = vec![];
        while let Some(r) = rx.recv().await {
            rows.push(r);
        }
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].get("id"), Some(&Value::Int(1)));
        assert_eq!(rows[2].get("id"), Some(&Value::Int(3)));
        assert_eq!(rows[0].get("label"), Some(&Value::Str("hello".into())));
    }
}
