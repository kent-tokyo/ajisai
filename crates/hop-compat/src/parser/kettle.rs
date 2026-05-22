/// Parse Pentaho/Kettle .ktr transformation and .kjb job files.
///
/// Kettle is the ancestor of Apache Hop, so the XML structure is very similar:
///   .ktr  — <transformation> root, <step> elements, same <order><hop> structure
///   .kjb  — <job> root, <entries><entry> elements, <hops><hop> structure
use crate::model::hop_pipeline::{HopHop, HopPipeline, HopTransform};
use crate::model::hop_workflow::{HopAction, HopWorkflow, HopWorkflowHop};
use ajisai_core::AjisaiError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

// ── .ktr parser ──────────────────────────────────────────────────────────────

/// Parse a Kettle/Pentaho .ktr transformation XML into a HopPipeline IR.
///
/// Differences from .hpl:
/// - Root element is `<transformation>` instead of `<pipeline>`
/// - Steps use `<step>` instead of `<transform>`
/// - Canvas positions are nested under `<step><GUI><xloc>`
pub fn parse_ktr(xml: &str) -> Result<HopPipeline, AjisaiError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut pipeline = HopPipeline::default();
    let mut stack: Vec<String> = Vec::new();
    let mut current_step: Option<HopTransform> = None;
    let mut current_hop: Option<HopHop> = None;
    let mut current_text = String::new();
    let mut in_gui = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                stack.push(tag.clone());
                current_text.clear();

                match tag.as_str() {
                    "step"
                        if in_context(&stack, "transformation") && !in_context(&stack, "GUI") =>
                    {
                        current_step = Some(HopTransform {
                            name: String::new(),
                            type_name: String::new(),
                            description: None,
                            xloc: None,
                            yloc: None,
                            attributes: HashMap::new(),
                        });
                    }
                    "GUI" if in_context(&stack, "step") => {
                        in_gui = true;
                    }
                    "hop" if in_context(&stack, "order") => {
                        current_hop = Some(HopHop {
                            from: String::new(),
                            to: String::new(),
                            enabled: Some(true),
                            error_hop: None,
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

                // Pipeline-level name
                if depth == 2 && tag == "name" {
                    if stack
                        .first()
                        .map(|s| s == "transformation")
                        .unwrap_or(false)
                    {
                        pipeline.name = text.clone();
                    }
                }

                // Step fields
                if let Some(ref mut t) = current_step {
                    if in_context(&stack, "step") {
                        match tag.as_str() {
                            "name" if !in_gui => t.name = text.clone(),
                            "type" => {
                                t.type_name = map_kettle_step_type(&text).to_owned();
                            }
                            "description" => t.description = Some(text.clone()),
                            "xloc" if in_gui => t.xloc = text.parse().ok(),
                            "yloc" if in_gui => t.yloc = text.parse().ok(),
                            other if !in_gui && depth > 2 && !text.is_empty() => {
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
                    match tag.as_str() {
                        "from" => h.from = text.clone(),
                        "to" => h.to = text.clone(),
                        "enabled" => h.enabled = Some(text == "Y" || text == "true"),
                        _ => {}
                    }
                }

                match tag.as_str() {
                    "GUI" => in_gui = false,
                    "step" => {
                        if let Some(t) = current_step.take() {
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

            Ok(Event::Eof) => break,
            Err(e) => return Err(AjisaiError::Parse(format!("XML parse error: {e}"))),
            _ => {}
        }
    }

    Ok(pipeline)
}

/// Map Kettle step type names to ajisai type names.
/// Kettle and Hop share most type names; only diverged types need mapping.
fn map_kettle_step_type(kettle_type: &str) -> &str {
    match kettle_type {
        // I/O
        "CSVFileInput" | "CsvInput" => "CsvFileInput",
        "CSVFileOutput" | "CsvOutput" => "CsvFileOutput",
        "TextFileInput" => "CsvFileInput",
        "TextFileOutput" => "CsvFileOutput",
        "JsonInput" => "JsonFileInput",
        "JsonOutput" => "JsonFileOutput",
        "TableInput" => "TableInput",
        "TableOutput" => "TableOutput",
        // Transform
        "FilterRows" => "FilterRows",
        "SelectValues" => "SelectValues",
        "SortRows" => "SortRows",
        "Constant" | "AddConstants" => "AddConstants",
        "Deduplicate" | "Unique" => "Deduplicate",
        "Calculator" => "CalculatorStep",
        "IfNull" => "IfNull",
        "StringOperations" => "StringOperations",
        "ReplaceInString" => "ReplaceInString",
        "ConcatFields" => "ConcatFields",
        "SplitFieldToRows" => "SplitFieldToRows",
        // Join / Lookup
        "MergeJoin" => "MergeJoin",
        "StreamLookup" => "StreamLookup",
        "DatabaseLookup" => "DatabaseLookup",
        // Pass through unknown names unchanged
        other => other,
    }
}

// ── .kjb parser ──────────────────────────────────────────────────────────────

/// Parse a Kettle/Pentaho .kjb job XML into a HopWorkflow IR.
///
/// Differences from .hwf:
/// - Root element is `<job>` instead of `<workflow>`
/// - Actions use `<entries><entry>` instead of `<actions><action>`
/// - Hops use `<hops><hop>` instead of `<order><hop>`
/// - Canvas positions nested under `<entry><GUI>`
pub fn parse_kjb(xml: &str) -> Result<HopWorkflow, AjisaiError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut workflow = HopWorkflow::default();
    let mut stack: Vec<String> = Vec::new();
    let mut current_action: Option<HopAction> = None;
    let mut current_hop: Option<HopWorkflowHop> = None;
    let mut current_text = String::new();
    let mut in_gui = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                stack.push(tag.clone());
                current_text.clear();

                match tag.as_str() {
                    "entry" => {
                        current_action = Some(HopAction {
                            name: String::new(),
                            type_name: String::new(),
                            xloc: None,
                            yloc: None,
                            attributes: Default::default(),
                        });
                    }
                    "GUI" if in_context(&stack, "entry") => {
                        in_gui = true;
                    }
                    "hop" if in_context(&stack, "hops") => {
                        current_hop = Some(HopWorkflowHop {
                            from: String::new(),
                            to: String::new(),
                            enabled: Some(true),
                            evaluation: None,
                            unconditional: None,
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

                // Workflow-level name
                if depth == 2 && tag == "name" {
                    if stack.first().map(|s| s == "job").unwrap_or(false) {
                        workflow.name = text.clone();
                    }
                }

                // Entry fields
                if let Some(ref mut a) = current_action {
                    if in_context(&stack, "entry") {
                        match tag.as_str() {
                            "name" if !in_gui => a.name = text.clone(),
                            "type" => a.type_name = map_kjb_entry_type(&text).to_owned(),
                            "xloc" if in_gui => a.xloc = text.parse().ok(),
                            "yloc" if in_gui => a.yloc = text.parse().ok(),
                            other if !in_gui && depth > 2 && !text.is_empty() => {
                                a.attributes.insert(
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
                    match tag.as_str() {
                        "from" => h.from = text.clone(),
                        "to" => h.to = text.clone(),
                        "enabled" => h.enabled = Some(text == "Y" || text == "true"),
                        "evaluation" => h.evaluation = Some(text.clone()),
                        "unconditional" => h.unconditional = Some(text == "Y" || text == "true"),
                        _ => {}
                    }
                }

                match tag.as_str() {
                    "GUI" => in_gui = false,
                    "entry" => {
                        if let Some(a) = current_action.take() {
                            workflow.actions.push(a);
                        }
                    }
                    "hop" if in_context(&stack, "hops") => {
                        if let Some(h) = current_hop.take() {
                            workflow.hops.push(h);
                        }
                    }
                    _ => {}
                }

                stack.pop();
                current_text.clear();
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(AjisaiError::Parse(format!("XML parse error: {e}"))),
            _ => {}
        }
    }

    Ok(workflow)
}

fn map_kjb_entry_type(kettle_type: &str) -> &str {
    match kettle_type {
        "SPECIAL" => "Start",
        "TRANS" => "Pipeline",
        "JOB" => "Workflow",
        "MAIL" => "Mail",
        "SHELL" => "Shell",
        "SQL" => "Sql",
        "HTTP" => "Http",
        "FTP" => "Ftp",
        "SFTP" => "Sftp",
        "COPY_FILES" => "CopyFiles",
        "DELETE_FILES" => "DeleteFiles",
        "MOVE_FILES" => "MoveFiles",
        "FOLDER_IS_EMPTY" => "FolderIsEmpty",
        "FILE_EXISTS" => "FileExists",
        "TABLE_EXISTS" => "TableExists",
        "SET_VARIABLES" => "SetVariables",
        other => other,
    }
}

fn in_context(stack: &[String], tag: &str) -> bool {
    stack.iter().any(|s| s == tag)
}
