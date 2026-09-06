use ajisai_core::{
    AjisaiError, Transform,
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
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
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
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
        Self {
            config,
            client: None,
            output_schema: None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: RestClientConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }

    async fn validate_target(&self, url: &str) -> Result<()> {
        let parsed = reqwest::Url::parse(url)
            .map_err(|e| AjisaiError::Config(format!("Invalid HTTP URL: {}", e)))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(AjisaiError::Config("Only HTTP(S) URLs are allowed".into()));
        }
        if parsed.username() != "" || parsed.password().is_some() {
            return Err(AjisaiError::Config(
                "URL credentials are not allowed".into(),
            ));
        }
        let host = parsed
            .host_str()
            .ok_or_else(|| AjisaiError::Config("HTTP URL host is required".into()))?;
        let normalized_host = host.trim_matches(&['[', ']'][..]);
        let lower = normalized_host.to_ascii_lowercase();
        if lower == "localhost" || lower == "metadata.google.internal" {
            return Err(AjisaiError::Config(
                "HTTP target host is blocked by SSRF policy".into(),
            ));
        }
        if let Ok(ip) = normalized_host.parse::<IpAddr>()
            && is_private_or_local_ip(ip)
        {
            return Err(AjisaiError::Config(
                "HTTP target IP is blocked by SSRF policy".into(),
            ));
        }
        if normalized_host.parse::<IpAddr>().is_err() {
            let port = parsed
                .port_or_known_default()
                .ok_or_else(|| AjisaiError::Config("HTTP URL port is required".into()))?;
            let addresses = tokio::net::lookup_host((normalized_host, port))
                .await
                .map_err(|_| AjisaiError::Config("HTTP host DNS resolution failed".into()))?;
            if addresses
                .map(|address| address.ip())
                .any(is_private_or_local_ip)
            {
                return Err(AjisaiError::Config(
                    "HTTP DNS target resolves to a blocked IP range".into(),
                ));
            }
        }
        if !self.config.allowed_hosts.is_empty()
            && !self
                .config
                .allowed_hosts
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(normalized_host))
        {
            return Err(AjisaiError::Config(format!(
                "HTTP target host '{}' is not in the allowlist",
                normalized_host
            )));
        }
        Ok(())
    }
}

fn is_private_or_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.octets()[0] == 0
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
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

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        if !ctx.network_allowed() {
            return Err(AjisaiError::Config(
                "Network access disabled by execution policy".into(),
            ));
        }
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
        self.validate_target(&url).await?;

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

        if let Some(bf) = &self.config.body_field
            && let Some(body) = row.get(bf)
        {
            req = req.body(body.to_display_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn network_policy_blocks_client_initialization() {
        let mut client = RestClient::new(RestClientConfig {
            url: "https://example.invalid".into(),
            method: "GET".into(),
            body_field: None,
            result_field: "body".into(),
            status_field: None,
            headers: None,
            allowed_hosts: vec![],
        });
        let mut context = ExecutionContext::new();
        context.set_network_allowed(false);
        let error = client.open(&context).await.unwrap_err();
        assert!(error.to_string().contains("Network access disabled"));
    }

    fn client_with_hosts(hosts: Vec<String>) -> RestClient {
        RestClient::new(RestClientConfig {
            url: "https://example.invalid".into(),
            method: "GET".into(),
            body_field: None,
            result_field: "body".into(),
            status_field: None,
            headers: None,
            allowed_hosts: hosts,
        })
    }

    #[tokio::test]
    async fn ssrf_policy_rejects_local_and_credentialed_urls() {
        let client = client_with_hosts(vec![]);
        assert!(
            client
                .validate_target("http://127.0.0.1/admin")
                .await
                .is_err()
        );
        assert!(
            client
                .validate_target("http://10.0.0.8/admin")
                .await
                .is_err()
        );
        assert!(is_private_or_local_ip("fd00::1".parse().unwrap()));
        assert!(
            client
                .validate_target("http://[fd00::1]/admin")
                .await
                .is_err()
        );
        assert!(
            client
                .validate_target("http://user:pass@192.0.2.1/")
                .await
                .is_err()
        );
        assert!(client.validate_target("file:///etc/passwd").await.is_err());
    }

    #[tokio::test]
    async fn host_allowlist_requires_explicit_match() {
        let client = client_with_hosts(vec!["192.0.2.10".into()]);
        assert!(
            client
                .validate_target("https://192.0.2.10/v1")
                .await
                .is_ok()
        );
        assert!(
            client
                .validate_target("https://192.0.2.11/v1")
                .await
                .is_err()
        );
    }
}
