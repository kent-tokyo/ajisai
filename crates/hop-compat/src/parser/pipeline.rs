use crate::model::hop_pipeline::{HopHop, HopPipeline, HopTransform};
use ajisai_core::AjisaiError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// Parse a .hpl file from a string into a HopPipeline intermediate representation
pub fn parse_hpl(xml: &str) -> Result<HopPipeline, AjisaiError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut pipeline = HopPipeline::default();
    let mut stack: Vec<String> = Vec::new();
    let mut current_transform: Option<HopTransform> = None;
    let mut current_hop: Option<HopHop> = None;
    let mut current_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                stack.push(tag.clone());
                current_text.clear();

                match tag.as_str() {
                    "transform" => {
                        current_transform = Some(HopTransform {
                            name: String::new(),
                            type_name: String::new(),
                            description: None,
                            xloc: None,
                            yloc: None,
                            attributes: HashMap::new(),
                        });
                    }
                    "hop" if in_context(&stack, "order") => {
                        current_hop = Some(HopHop {
                            from: String::new(),
                            to: String::new(),
                            enabled: Some(true),
                        });
                    }
                    _ => {}
                }
            }

            Ok(Event::Text(e)) => {
                current_text = e
                    .unescape()
                    .map_err(|e| AjisaiError::Parse(e.to_string()))?
                    .into_owned();
            }

            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let text = current_text.trim().to_owned();
                let depth = stack.len();

                // Pipeline-level fields
                if depth == 2 && stack.first().map(|s| s == "pipeline").unwrap_or(false) {
                    match tag.as_str() {
                        "name" => pipeline.name = text.clone(),
                        _ => {}
                    }
                }

                // Transform fields
                if let Some(ref mut t) = current_transform {
                    if in_context(&stack, "transform") {
                        match tag.as_str() {
                            "name" => t.name = text.clone(),
                            "type" => t.type_name = text.clone(),
                            "description" => t.description = Some(text.clone()),
                            "xloc" => t.xloc = text.parse().ok(),
                            "yloc" => t.yloc = text.parse().ok(),
                            // Everything else becomes an attribute
                            other if depth > 2 && !text.is_empty() => {
                                t.attributes.insert(
                                    other.to_owned(),
                                    serde_json::Value::String(text.clone()),
                                );
                            }
                            _ => {}
                        }
                    }
                }

                // Hop fields
                if let Some(ref mut h) = current_hop {
                    if in_context(&stack, "order") {
                        match tag.as_str() {
                            "from" => h.from = text.clone(),
                            "to" => h.to = text.clone(),
                            "enabled" => h.enabled = Some(text == "Y" || text == "true"),
                            _ => {}
                        }
                    }
                }

                // Finalize elements
                match tag.as_str() {
                    "transform" => {
                        if let Some(t) = current_transform.take() {
                            pipeline.transforms.push(t);
                        }
                    }
                    "hop" if in_context(&stack, "order") => {
                        if let Some(h) = current_hop.take() {
                            pipeline.order.push(h);
                        }
                    }
                    _ => {}
                }

                stack.pop();
                current_text.clear();
            }

            Ok(Event::Empty(e)) => {
                // Self-closing tags (e.g. <hop from="A" to="B" enabled="Y"/>)
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                if tag == "hop" {
                    let mut h = HopHop {
                        from: String::new(),
                        to: String::new(),
                        enabled: Some(true),
                    };
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                        let val = String::from_utf8_lossy(&attr.value).into_owned();
                        match key.as_str() {
                            "from" => h.from = val,
                            "to" => h.to = val,
                            "enabled" => h.enabled = Some(val == "Y" || val == "true"),
                            _ => {}
                        }
                    }
                    pipeline.order.push(h);
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(AjisaiError::Parse(format!("XML parse error: {}", e))),
            _ => {}
        }
    }

    Ok(pipeline)
}

fn in_context(stack: &[String], tag: &str) -> bool {
    stack.iter().any(|s| s == tag)
}

/// Convert a HopPipeline IR into an ajisai-core Pipeline.
/// The transform registry is used to instantiate each transform.
pub fn hop_pipeline_to_ajisai(
    hop: HopPipeline,
    registry: &ajisai_transforms::TransformRegistry,
) -> Result<ajisai_core::Pipeline, AjisaiError> {
    let mut pipeline = ajisai_core::Pipeline::new(hop.name);

    for ht in hop.transforms {
        // Map Hop transform type names to ajisai type names
        let ajisai_type = map_transform_type(&ht.type_name);
        let config = build_transform_config(&ht);

        let transform = registry.create(ajisai_type, config)?;
        pipeline.add_node(ht.name, transform);
    }

    for hop in hop.order {
        if hop.enabled.unwrap_or(true) {
            pipeline.add_hop(hop.from, hop.to);
        }
    }

    Ok(pipeline)
}

/// Map Apache Hop transform type names to ajisai type names
fn map_transform_type(hop_type: &str) -> &str {
    match hop_type {
        "CSVFileInput" | "CsvInput" => "CsvFileInput",
        "CSVFileOutput" | "CsvOutput" => "CsvFileOutput",
        "FilterRows" => "FilterRows",
        "SelectValues" => "SelectValues",
        "SortRows" => "SortRows",
        "AddConstants" | "Constant" => "AddConstants",
        other => other,
    }
}

/// Build a serde_json::Value config from a HopTransform's attributes
fn build_transform_config(ht: &HopTransform) -> serde_json::Value {
    let mut map = serde_json::Map::new();

    // Common fields
    if let Some(filename) = ht.attributes.get("filename") {
        map.insert("filename".into(), filename.clone());
    }
    if let Some(sep) = ht.attributes.get("separator") {
        // Hop uses "separator" for delimiter
        if let Some(s) = sep.as_str() {
            map.insert("delimiter".into(), serde_json::Value::String(s.to_owned()));
        }
    }
    if let Some(header) = ht.attributes.get("header") {
        let has_header = header
            .as_str()
            .map(|s| s == "Y" || s == "true")
            .unwrap_or(true);
        map.insert("header_present".into(), serde_json::Value::Bool(has_header));
    }

    // Merge remaining attributes
    for (k, v) in &ht.attributes {
        map.entry(k.clone()).or_insert_with(|| v.clone());
    }

    serde_json::Value::Object(map)
}
