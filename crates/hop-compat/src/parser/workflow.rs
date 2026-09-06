use crate::model::hop_workflow::{HopAction, HopWorkflow, HopWorkflowHop};
use ajisai_core::AjisaiError;
use quick_xml::Reader;
use quick_xml::events::Event;

/// Parse a .hwf file from a string
pub fn parse_hwf(xml: &str) -> Result<HopWorkflow, AjisaiError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut workflow = HopWorkflow::default();
    let mut stack: Vec<String> = Vec::new();
    let mut current_action: Option<HopAction> = None;
    let mut current_hop: Option<HopWorkflowHop> = None;
    let mut current_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                stack.push(tag.clone());
                current_text.clear();

                match tag.as_str() {
                    "action" => {
                        current_action = Some(HopAction {
                            name: String::new(),
                            type_name: String::new(),
                            xloc: None,
                            yloc: None,
                            attributes: Default::default(),
                        });
                    }
                    "hop" => {
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
                    .decode()
                    .map_err(|e| AjisaiError::Parse(e.to_string()))?
                    .into_owned();
            }

            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let text = current_text.trim().to_owned();
                let depth = stack.len();

                if depth == 2 && tag == "name" {
                    workflow.name = text.clone();
                }

                if let Some(ref mut a) = current_action {
                    match tag.as_str() {
                        "name" if depth > 2 => a.name = text.clone(),
                        "type" => a.type_name = text.clone(),
                        "xloc" => a.xloc = text.parse().ok(),
                        "yloc" => a.yloc = text.parse().ok(),
                        other if depth > 2 && !text.is_empty() => {
                            a.attributes
                                .insert(other.to_owned(), serde_json::Value::String(text.clone()));
                        }
                        _ => {}
                    }
                }

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
                    "action" => {
                        if let Some(a) = current_action.take() {
                            workflow.actions.push(a);
                        }
                    }
                    "hop" => {
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
            Err(e) => return Err(AjisaiError::Parse(format!("XML parse error: {}", e))),
            _ => {}
        }
    }

    Ok(workflow)
}
