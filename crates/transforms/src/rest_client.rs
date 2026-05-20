use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestClientConfig {
    pub url: String,
    #[serde(default = "default_get")]
    pub method: String,
    pub body_field: Option<String>,
    pub result_field: String,
    pub status_field: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

fn default_get() -> String {
    "GET".into()
}

pub struct RestClient {
    config: RestClientConfig,
    client: Option<Client>,
    output_schema: Option<Arc<RowSchema>>,
}

impl RestClient {
    pub fn new(config: RestClientConfig) -> Self {
        Self { config, client: None, output_schema: None }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: RestClientConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for RestClient {
    fn name(&self) -> &str {
        "RestClient"
    }

    fn output_schema(&self, input: &RowSchema) -> Result<RowSchema> {
        let mut fields = input.fields.clone();
        fields.push(Field::new(&self.config.result_field, ValueType::String));
        if let Some(sf) = &self.config.status_field {
            fields.push(Field::new(sf.as_str(), ValueType::Integer));
        }
        Ok(RowSchema::new(fields))
    }

    async fn open(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        self.client = Some(
            Client::builder()
                .build()
                .map_err(|e| AjisaiError::Config(format!("Failed to build HTTP client: {}", e)))?,
        );
        Ok(())
    }

    async fn process(&mut self, row: Row) -> Result<Vec<Row>> {
        let client = self.client.as_ref().expect("open() not called");
        let url = row
            .schema
            .fields
            .iter()
            .zip(&row.values)
            .fold(self.config.url.clone(), |u, (f, v)| {
                u.replace(&format!("${{{}}}", f.name), &v.to_display_string())
            });

        let method = self.config.method.to_uppercase();
        let mut req = match method.as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };

        if let Some(hdrs) = &self.config.headers {
            for (k, v) in hdrs {
                req = req.header(k.as_str(), v.as_str());
            }
        }

        if let Some(bf) = &self.config.body_field {
            if let Some(body) = row.get(bf) {
                req = req.body(body.to_display_string());
            }
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AjisaiError::Config(format!("HTTP request failed: {}", e)))?;

        let status = resp.status().as_u16() as i64;
        let body = resp
            .text()
            .await
            .map_err(|e| AjisaiError::Config(format!("Failed to read response: {}", e)))?;

        let schema = self.output_schema.get_or_insert_with(|| {
            let mut fields = row.schema.fields.clone();
            fields.push(Field::new(&self.config.result_field, ValueType::String));
            if let Some(sf) = &self.config.status_field {
                fields.push(Field::new(sf.as_str(), ValueType::Integer));
            }
            Arc::new(RowSchema::new(fields))
        });

        let mut values = row.values.clone();
        values.push(Value::Str(body));
        if self.config.status_field.is_some() {
            values.push(Value::Int(status));
        }

        Ok(vec![Row::new(schema.clone(), values)])
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
