/// Parse Microsoft SSIS .dtsx package files (best-effort).
///
/// SSIS uses a namespace-prefixed XML format (`DTS:`). This parser strips the
/// prefix and extracts Data Flow Task components and their connections into the
/// shared HopPipeline IR. Only Data Flow Tasks are converted; Control Flow
/// tasks (Execute SQL, File System, etc.) are not mapped.
///
/// Supported component class IDs:
///   Flat File Source      → CsvFileInput
///   Flat File Destination → CsvFileOutput
///   OLE DB Source         → TableInput
///   OLE DB Destination    → TableOutput
///   Conditional Split     → FilterRows
///   Derived Column        → CalculatorStep
///   Sort                  → SortRows
///   Merge Join            → MergeJoin
///   Lookup                → StreamLookup
use crate::model::hop_pipeline::{HopHop, HopPipeline, HopTransform};
use ajisai_core::AjisaiError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// Parse a SSIS .dtsx package XML into a HopPipeline IR (best-effort).
pub fn parse_dtsx(xml: &str) -> Result<HopPipeline, AjisaiError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut pipeline = HopPipeline::default();

    // DTSID → component name map (for resolving path endpoints)
    let mut id_to_name: HashMap<String, String> = HashMap::new();
    // Component name → index in pipeline.transforms (for deduplication)
    let mut name_to_idx: HashMap<String, usize> = HashMap::new();

    let mut stack: Vec<String> = Vec::new();
    let mut current_component: Option<HopTransform> = None;
    let mut current_text = String::new();
    let mut in_data_flow = false;
    let mut in_pipeline_section = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                // Strip namespace prefix — use only the local name
                let local = local_name(&e.name());
                stack.push(local.clone());
                current_text.clear();

                // Detect package name from the outermost Executable
                // Detect Data Flow Task: Executable with CreationName="Microsoft.Pipeline"
                if local == "Executable" {
                    let attrs = collect_dts_attrs(&e);
                    if attrs.get("CreationName").map(|s| s.as_str())
                        == Some("Microsoft.Pipeline")
                    {
                        in_data_flow = true;
                        if pipeline.name.is_empty() {
                            if let Some(n) = attrs.get("ObjectName") {
                                pipeline.name = n.clone();
                            }
                        }
                    }
                    // Set overall pipeline name from outermost Package
                    if attrs.get("CreationName").map(|s| s.as_str())
                        == Some("Microsoft.Package")
                    {
                        if let Some(n) = attrs.get("ObjectName") {
                            pipeline.name = n.clone();
                        }
                    }
                }

                if local == "pipeline" && in_data_flow {
                    in_pipeline_section = true;
                }

                // Parse components inside the data flow pipeline section
                if local == "component" && in_pipeline_section {
                    let attrs = collect_plain_attrs(&e);
                    let comp_name = attrs
                        .get("name")
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_owned());
                    let class_id = attrs.get("componentClassID").cloned().unwrap_or_default();
                    let dts_id = attrs.get("id").cloned().unwrap_or_default();

                    let ajisai_type = map_ssis_class_id(&class_id)
                        .or_else(|| infer_type_from_name(&comp_name))
                        .unwrap_or("Unknown")
                        .to_owned();

                    current_component = Some(HopTransform {
                        name: comp_name.clone(),
                        type_name: ajisai_type,
                        description: None,
                        xloc: None,
                        yloc: None,
                        attributes: HashMap::new(),
                    });

                    if !dts_id.is_empty() {
                        id_to_name.insert(dts_id, comp_name);
                    }
                }

                // Parse path elements to build hops
                if local == "path" && in_pipeline_section {
                    let attrs = collect_plain_attrs(&e);
                    if let (Some(start), Some(end)) =
                        (attrs.get("startId"), attrs.get("endId"))
                    {
                        // Path IDs look like "{GUID}.Outputs[name]" or "{GUID}.output[...]"
                        let from_id = extract_component_id(start);
                        let to_id = extract_component_id(end);
                        if let (Some(from_name), Some(to_name)) = (
                            id_to_name.get(from_id),
                            id_to_name.get(to_id),
                        ) {
                            pipeline.order.push(HopHop {
                                from: from_name.clone(),
                                to: to_name.clone(),
                                enabled: Some(true),
                            });
                        }
                    }
                }
            }

            Ok(Event::Text(e)) => {
                current_text = e
                    .unescape()
                    .map_err(|e| AjisaiError::Parse(e.to_string()))?
                    .into_owned();
            }

            Ok(Event::End(e)) => {
                let local = local_name(&e.name());

                if local == "component" && in_pipeline_section {
                    if let Some(t) = current_component.take() {
                        let idx = pipeline.transforms.len();
                        name_to_idx.insert(t.name.clone(), idx);
                        pipeline.transforms.push(t);
                    }
                }
                if local == "pipeline" {
                    in_pipeline_section = false;
                }
                if local == "Executable" && in_data_flow {
                    in_data_flow = false;
                }

                stack.pop();
                current_text.clear();
            }

            Ok(Event::Empty(e)) => {
                let local = local_name(&e.name());

                // Self-closing <path .../> elements
                if local == "path" && in_pipeline_section {
                    let attrs = collect_plain_attrs(&e);
                    if let (Some(start), Some(end)) =
                        (attrs.get("startId"), attrs.get("endId"))
                    {
                        let from_id = extract_component_id(start);
                        let to_id = extract_component_id(end);
                        if let (Some(from_name), Some(to_name)) = (
                            id_to_name.get(from_id),
                            id_to_name.get(to_id),
                        ) {
                            pipeline.order.push(HopHop {
                                from: from_name.clone(),
                                to: to_name.clone(),
                                enabled: Some(true),
                            });
                        }
                    }
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(AjisaiError::Parse(format!("XML parse error: {e}"))),
            _ => {}
        }
    }

    if pipeline.name.is_empty() {
        pipeline.name = "Imported SSIS Package".to_owned();
    }

    Ok(pipeline)
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Strip namespace prefix, e.g. "DTS:Executable" → "Executable"
fn local_name(qname: &quick_xml::name::QName<'_>) -> String {
    let full = String::from_utf8_lossy(qname.as_ref()).into_owned();
    full.split(':').last().unwrap_or(&full).to_owned()
}

/// Collect attributes that use the DTS: prefix (e.g. DTS:ObjectName)
fn collect_dts_attrs(e: &quick_xml::events::BytesStart<'_>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        let local_key = key.split(':').last().unwrap_or(&key).to_owned();
        let val = String::from_utf8_lossy(&attr.value).into_owned();
        map.insert(local_key, val);
    }
    map
}

/// Collect plain (non-namespaced) attributes
fn collect_plain_attrs(e: &quick_xml::events::BytesStart<'_>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        let val = String::from_utf8_lossy(&attr.value).into_owned();
        map.insert(key, val);
    }
    map
}

/// Extract the component GUID from a path endpoint ID.
/// Format: "{GUID}.Outputs[name]" or "{GUID}.output[name]"
fn extract_component_id(endpoint: &str) -> &str {
    endpoint.split('.').next().unwrap_or(endpoint)
}

/// Map SSIS component class IDs (GUIDs) to ajisai transform type names.
fn map_ssis_class_id(class_id: &str) -> Option<&'static str> {
    // Normalize: strip braces, uppercase
    let normalized = class_id
        .trim_matches('{')
        .trim_matches('}')
        .to_uppercase();
    match normalized.as_str() {
        // I/O
        "5ACD952A-F16A-11D2-9A7A-00C04F72DB40" => Some("CsvFileInput"),
        "5ACD9531-F16A-11D2-9A7A-00C04F72DB40" => Some("CsvFileOutput"),
        "BCEFE59B-6819-47F7-A125-63753B33ABB7" => Some("TableInput"),
        "5A0B62D8-AA10-4CB7-B9B8-88E2BCED854B" => Some("TableOutput"),
        // Transform
        "494F9CDB-2007-4475-98E5-B68AF94AE037" => Some("FilterRows"),
        "2932025B-AB99-11D2-9A7A-00C04F72DB40" => Some("CalculatorStep"),
        "FC3B3B3A-A23D-11D2-9A7A-00C04F72DB40" => Some("SortRows"),
        // Join / Lookup
        "7ACC6C33-5B73-4A77-AA56-D8DDD1A3EB4D" => Some("MergeJoin"),
        "98A4BEFE-AE52-422F-82DC-86C72F563B2E" => Some("StreamLookup"),
        _ => None,
    }
}

/// Infer ajisai transform type from the component display name as a fallback.
fn infer_type_from_name(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    if lower.contains("flat file source") || lower.contains("csv source") {
        Some("CsvFileInput")
    } else if lower.contains("flat file destination") || lower.contains("csv destination") {
        Some("CsvFileOutput")
    } else if lower.contains("ole db source") || lower.contains("table source") {
        Some("TableInput")
    } else if lower.contains("ole db destination") || lower.contains("table destination") {
        Some("TableOutput")
    } else if lower.contains("conditional split") || lower.contains("filter") {
        Some("FilterRows")
    } else if lower.contains("derived column") || lower.contains("calculator") {
        Some("CalculatorStep")
    } else if lower.contains("sort") {
        Some("SortRows")
    } else if lower.contains("merge join") {
        Some("MergeJoin")
    } else if lower.contains("lookup") {
        Some("StreamLookup")
    } else {
        None
    }
}
