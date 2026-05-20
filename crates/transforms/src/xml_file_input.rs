use crate::utils::{coerce, resolve_safe_path};
use ajisai_core::{
    context::ExecutionContext,
    error::Result,
    value::{Field, Row, RowSchema, Value, ValueType},
    AjisaiError, Transform,
};
use async_trait::async_trait;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlFieldSpec {
    pub name: String,
    /// Path relative to the record element, e.g. "city" or "address/city"
    pub xpath: String,
    pub field_type: ValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlFileInputConfig {
    pub filename: String,
    /// The element name whose occurrences define individual rows (e.g. "record")
    pub record_element: String,
    pub fields: Vec<XmlFieldSpec>,
}

pub struct XmlFileInput {
    config: XmlFileInputConfig,
    resolved_path: String,
}

impl XmlFileInput {
    pub fn new(config: XmlFileInputConfig) -> Self {
        let resolved_path = config.filename.clone();
        Self {
            config,
            resolved_path,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Result<Box<dyn Transform>> {
        let config: XmlFileInputConfig =
            serde_json::from_value(value).map_err(|e| AjisaiError::Config(e.to_string()))?;
        Ok(Box::new(Self::new(config)))
    }
}

#[async_trait]
impl Transform for XmlFileInput {
    fn name(&self) -> &str {
        "XmlFileInput"
    }

    fn output_schema(&self, _input: &RowSchema) -> Result<RowSchema> {
        let fields = self
            .config
            .fields
            .iter()
            .map(|f| Field::new(f.name.as_str(), f.field_type.clone()))
            .collect();
        Ok(RowSchema::new(fields))
    }

    fn is_source(&self) -> bool {
        true
    }

    async fn open(&mut self, ctx: &ExecutionContext) -> Result<()> {
        self.resolved_path = ctx.resolve(&self.config.filename);
        Ok(())
    }

    async fn produce(&mut self, sender: Sender<Row>) -> Result<()> {
        let safe_path = resolve_safe_path(&self.resolved_path)?;
        let xml = std::fs::read_to_string(&safe_path).map_err(|e| {
            AjisaiError::Config(format!(
                "Cannot read XML file '{}': {}",
                self.resolved_path, e
            ))
        })?;

        let schema = Arc::new(RowSchema::new(
            self.config
                .fields
                .iter()
                .map(|f| Field::new(f.name.as_str(), f.field_type.clone()))
                .collect::<Vec<_>>(),
        ));

        let record_elem = self.config.record_element.as_bytes().to_vec();
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);

        let mut in_record = false;
        // current path segments inside a record
        let mut path_stack: Vec<String> = Vec::new();
        // current text buffer
        let mut text_buf = String::new();
        // accumulated field values for the current record
        let mut current: HashMap<String, String> = HashMap::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                    if !in_record && local.as_bytes() == record_elem.as_slice() {
                        in_record = true;
                        path_stack.clear();
                        current.clear();
                    } else if in_record {
                        path_stack.push(local);
                        text_buf.clear();
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_record {
                        text_buf = e.unescape().unwrap_or_default().into_owned();
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                    if in_record {
                        if local.as_bytes() == record_elem.as_slice() {
                            // Emit row
                            let values: Vec<Value> = self
                                .config
                                .fields
                                .iter()
                                .map(|f| {
                                    current
                                        .get(&f.xpath)
                                        .map(|s| coerce(s, &f.field_type))
                                        .unwrap_or(Value::Null)
                                })
                                .collect();
                            let row = Row::new(schema.clone(), values);
                            if sender.send(row).await.is_err() {
                                return Ok(());
                            }
                            in_record = false;
                        } else {
                            // Store text for the current path
                            let path = path_stack.join("/");
                            if !text_buf.is_empty() {
                                current.insert(path.clone(), text_buf.clone());
                                // Also store leaf name alone for simple xpaths
                                if let Some(leaf) = path_stack.last() {
                                    current
                                        .entry(leaf.clone())
                                        .or_insert_with(|| text_buf.clone());
                                }
                            }
                            path_stack.pop();
                            text_buf.clear();
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(AjisaiError::Parse(format!("XML parse error: {}", e)));
                }
                _ => {}
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
    async fn parses_xml_records() {
        let xml = r#"
<records>
  <record><name>Alice</name><age>30</age></record>
  <record><name>Bob</name><age>25</age></record>
</records>"#;

        let tmp = std::env::temp_dir().join("test_xml_input.xml");
        std::fs::write(&tmp, xml).unwrap();

        let mut t = XmlFileInput::new(XmlFileInputConfig {
            filename: tmp.to_str().unwrap().into(),
            record_element: "record".into(),
            fields: vec![
                XmlFieldSpec {
                    name: "name".into(),
                    xpath: "name".into(),
                    field_type: ValueType::String,
                },
                XmlFieldSpec {
                    name: "age".into(),
                    xpath: "age".into(),
                    field_type: ValueType::Integer,
                },
            ],
        });
        t.open(&ExecutionContext::new()).await.unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::channel(16);
        t.produce(tx).await.unwrap();

        let mut rows = vec![];
        while let Some(r) = rx.recv().await {
            rows.push(r);
        }

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("name"), Some(&Value::Str("Alice".into())));
        assert_eq!(rows[0].get("age"), Some(&Value::Int(30)));
        assert_eq!(rows[1].get("name"), Some(&Value::Str("Bob".into())));
    }
}
