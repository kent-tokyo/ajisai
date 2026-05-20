use egui::{ComboBox, Ui};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Show a form-based config editor for the given transform type.
/// Returns true if the config was modified.
pub fn show_config_form(
    ui: &mut Ui,
    type_name: &str,
    config: &mut Value,
    node_id: &str,
    json_buf: &mut HashMap<String, String>,
) -> bool {
    if !config.is_object() {
        *config = Value::Object(Map::new());
    }

    let mut changed = false;

    match type_name {
        "CsvFileInput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= str_row(ui, "Delimiter", config, "delimiter");
            changed |= bool_row(ui, "Has header", config, "has_header");
            changed |= bool_row(ui, "Trim whitespace", config, "trim");
        }
        "CsvFileOutput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= str_row(ui, "Delimiter", config, "delimiter");
            changed |= bool_row(ui, "Write header", config, "header");
        }
        "JsonFileInput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= combo_row(ui, "Format", config, "format", &["array", "jsonl"]);
        }
        "JsonFileOutput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= combo_row(ui, "Format", config, "format", &["array", "jsonl"]);
            changed |= bool_row(ui, "Pretty print", config, "pretty");
        }
        "TableInput" => {
            changed |= str_row(ui, "Connection URL", config, "connection_url");
            changed |= text_area_row(ui, "SQL", config, "sql", 4);
        }
        "TableOutput" => {
            changed |= str_row(ui, "Connection URL", config, "connection_url");
            changed |= str_row(ui, "Table", config, "table");
            changed |= combo_row(
                ui,
                "Mode",
                config,
                "mode",
                &["insert", "upsert", "overwrite"],
            );
            changed |= u64_row(ui, "Batch size (0=end)", config, "batch_size");
        }
        "FilterRows" => {
            changed |= str_row(ui, "Condition", config, "condition");
        }
        "SelectValues" => {
            changed |= str_list_row(ui, "Fields (one per line)", config, "fields");
        }
        "SortRows" => {
            changed |= sort_keys_editor(ui, config);
        }
        "AddConstants" => {
            changed |= const_fields_editor(ui, config);
        }
        "CalculatorStep" => {
            ui.label(
                egui::RichText::new("Edit calculations as JSON:")
                    .weak()
                    .size(11.0),
            );
            changed |= json_fallback(ui, config, 8, node_id, json_buf);
        }
        "StreamLookup" => {
            changed |= str_row(ui, "Lookup transform", config, "lookup_transform");
            changed |= str_row(ui, "Key field", config, "key_field");
            changed |= str_row(ui, "Lookup key field", config, "lookup_key_field");
            changed |= str_list_row(ui, "Return fields (one/line)", config, "return_fields");
            changed |= str_row(ui, "No-match value", config, "no_match_value");
        }
        "MergeJoin" => {
            changed |= str_row(ui, "Left key", config, "left_key");
            changed |= str_row(ui, "Right key", config, "right_key");
            changed |= combo_row(
                ui,
                "Join type",
                config,
                "join_type",
                &["inner", "left_outer", "right_outer", "full"],
            );
            changed |= str_row(ui, "Right prefix", config, "right_prefix");
        }
        "Deduplicate" => {
            changed |= str_list_row(
                ui,
                "Key fields (one/line,\nempty = all fields)",
                config,
                "key_fields",
            );
        }
        "DatabaseLookup" => {
            changed |= str_row(ui, "Connection URL", config, "connection_url");
            changed |= text_area_row(ui, "SQL", config, "sql", 4);
            changed |= str_row(ui, "Key field", config, "key_field");
            changed |= str_list_row(ui, "Return fields (one/line)", config, "return_fields");
        }
        "IfNull" => {
            changed |= if_null_editor(ui, config);
        }
        "StringOperations" => {
            ui.label(
                egui::RichText::new(
                    "Edit operations as JSON:\n[{\"field\":\"f\",\"op\":\"trim\"}]",
                )
                .weak()
                .size(11.0),
            );
            changed |= json_fallback(ui, config, 8, node_id, json_buf);
        }
        "ReplaceInString" => {
            changed |= replace_in_string_editor(ui, config);
        }
        "ConcatFields" => {
            changed |= str_list_row(ui, "Fields (one per line)", config, "fields");
            changed |= str_row(ui, "Separator", config, "separator");
            changed |= str_row(ui, "Output field", config, "output_field");
        }
        "SplitFieldToRows" => {
            changed |= str_row(ui, "Source field", config, "field");
            changed |= str_row(ui, "Delimiter", config, "delimiter");
            changed |= str_row(ui, "Output field", config, "output_field");
            changed |= bool_row(ui, "Trim tokens", config, "trim");
        }
        "ExcelFileInput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= str_row(ui, "Sheet name (blank = first)", config, "sheet_name");
            changed |= bool_row(ui, "Header row present", config, "header_present");
        }
        "ExcelFileOutput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= str_row(ui, "Sheet name", config, "sheet_name");
            changed |= bool_row(ui, "Write header", config, "header");
        }
        "ParquetFileInput" => {
            changed |= str_row(ui, "File path", config, "filename");
        }
        "ParquetFileOutput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= combo_row(ui, "Compression", config, "compression", &["snappy", "gzip", "none"]);
        }
        "XmlFileInput" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= str_row(ui, "Record element", config, "record_element");
            ui.label(egui::RichText::new("Fields (JSON array):").weak().size(11.0));
            changed |= json_fallback(ui, config, 6, node_id, json_buf);
        }
        "GenerateRows" => {
            changed |= u64_row(ui, "Number of rows", config, "limit");
            ui.label(egui::RichText::new("Fields (JSON array):").weak().size(11.0));
            changed |= json_fallback(ui, config, 6, node_id, json_buf);
        }
        "RestClient" => {
            changed |= str_row(ui, "URL (${field} supported)", config, "url");
            changed |= combo_row(ui, "Method", config, "method", &["GET", "POST", "PUT", "DELETE"]);
            changed |= str_row(ui, "Result field", config, "result_field");
            changed |= str_row(ui, "Status field (optional)", config, "status_field");
            changed |= str_row(ui, "Body field (optional)", config, "body_field");
        }
        "WriteToLog" => {
            changed |= combo_row(ui, "Log level", config, "level", &["info", "debug", "warn", "error"]);
            changed |= str_list_row(ui, "Fields to log (blank = all)", config, "fields");
        }
        "MemoryGroupBy" => {
            changed |= str_list_row(ui, "Group by fields (one/line)", config, "group_fields");
            ui.label(egui::RichText::new("Aggregates (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"field":"f","aggregate":"sum","output":"total"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "AddSequence" => {
            changed |= str_row(ui, "Output field name", config, "field_name");
            changed |= i64_row(ui, "Start value", config, "start");
            changed |= i64_row(ui, "Increment", config, "increment");
        }
        "AppendStreams" => {
            ui.label(egui::RichText::new("No configuration needed.\nConnect multiple inputs.").weak().size(11.0));
        }
        "RowNormaliser" => {
            changed |= str_row(ui, "Type field (label output)", config, "type_field");
            changed |= str_row(ui, "Value field (value output)", config, "value_field");
            changed |= str_list_row(ui, "Non-pivot fields (one/line)", config, "non_pivot_fields");
            ui.label(egui::RichText::new("Normalize specs (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"type_value":"Q1","fields":["q1_sales"]}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "RowDenormaliser" => {
            changed |= str_list_row(ui, "Group fields (one/line)", config, "group_fields");
            changed |= str_row(ui, "Key field (pivot key)", config, "key_field");
            changed |= str_row(ui, "Value field (data value)", config, "value_field");
            ui.label(egui::RichText::new("Target fields (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"key_value":"Q1","result_field":"q1","aggregate":"first"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "SetVariable" => {
            ui.label(egui::RichText::new("Variables (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"variable_name":"MY_VAR","field_name":"src_field"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "GetVariable" => {
            ui.label(egui::RichText::new("Variables (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"field_name":"env","variable":"${MY_VAR}","field_type":"string"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "SwitchCase" => {
            changed |= str_row(ui, "Switch field", config, "field_name");
            ui.label(egui::RichText::new("Cases (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"value":"active","target":"node_id"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "GetFileNames" => {
            changed |= str_row(ui, "Directory", config, "directory");
            changed |= str_row(ui, "Pattern (regex, optional)", config, "pattern");
            changed |= bool_row(ui, "Include subdirectories", config, "include_subdirs");
        }
        "LoadFileContent" => {
            changed |= str_row(ui, "Path field", config, "path_field");
            changed |= str_row(ui, "Content field", config, "content_field");
            changed |= combo_row(ui, "Encoding", config, "encoding", &["utf8", "base64"]);
        }
        "WriteToFile" => {
            changed |= str_row(ui, "File path", config, "filename");
            changed |= str_row(ui, "Source field", config, "field");
            changed |= bool_row(ui, "Append mode", config, "append");
            changed |= bool_row(ui, "Add newline", config, "newline");
        }
        "Dummy" => {
            ui.label(egui::RichText::new("No configuration — rows pass through unchanged.").weak().size(11.0));
        }
        "Abort" => {
            changed |= str_row(ui, "Abort message", config, "message");
            ui.label(egui::RichText::new("Condition (JSON, optional):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"{"op":"eq","field":"status","value":"ERROR"}"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 4, node_id, json_buf);
        }
        "RegexEval" => {
            changed |= str_row(ui, "Source field", config, "field");
            changed |= str_row(ui, "Pattern (regex)", config, "pattern");
            changed |= str_list_row(ui, "Output fields (one per line; blank = use named groups)", config, "output_fields");
            changed |= bool_row(ui, "Drop unmatched rows", config, "drop_unmatched");
        }
        "CloneRow" => {
            changed |= u64_row(ui, "Clone count", config, "clone_count");
        }
        "FieldSplitter" => {
            changed |= str_row(ui, "Source field", config, "field");
            changed |= str_row(ui, "Delimiter", config, "delimiter");
            changed |= str_list_row(ui, "Output fields (one per line)", config, "output_fields");
            changed |= bool_row(ui, "Trim whitespace", config, "trim");
        }
        "UniqueRows" => {
            changed |= str_list_row(ui, "Key fields (empty = all)", config, "key_fields");
        }
        "NumberRange" => {
            changed |= str_row(ui, "Source field", config, "field");
            changed |= str_row(ui, "Output field", config, "output_field");
            changed |= str_row(ui, "Default value", config, "default_value");
            ui.label(egui::RichText::new("Ranges (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"lower":0,"upper":59.9,"result":"F"},{"lower":60,"result":"Pass"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "ValueMapper" => {
            changed |= str_row(ui, "Source field", config, "field");
            changed |= str_row(ui, "Output field (blank = in-place)", config, "output_field");
            changed |= str_row(ui, "Default value (blank = passthrough)", config, "default_value");
            ui.label(egui::RichText::new("Mappings (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"source_value":"Y","target_value":"Yes"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "ExecuteSQL" => {
            changed |= str_row(ui, "Connection URL", config, "connection_url");
            changed |= text_area_row(ui, "SQL", config, "sql", 4);
            changed |= bool_row(ui, "Execute per row", config, "execute_per_row");
        }
        "ScriptStep" => {
            changed |= text_area_row(ui, "Rhai script", config, "script", 8);
            ui.label(egui::RichText::new("Output fields (JSON array):").weak().size(11.0));
            ui.label(
                egui::RichText::new(r#"[{"name":"result","field_type":"integer"}]"#)
                    .weak()
                    .size(10.0),
            );
            changed |= json_fallback(ui, config, 5, node_id, json_buf);
        }
        "PipelineExecutor" => {
            changed |= str_row(ui, "Sub-pipeline path (.hpl)", config, "sub_pipeline_path");
            changed |= bool_row(ui, "Inherit variables", config, "inherit_variables");
        }
        _ => {
            changed |= json_fallback(ui, config, 8, node_id, json_buf);
        }
    }

    changed
}

// ── helpers ────────────────────────────────────────────────────────────────

fn str_row(ui: &mut Ui, label: &str, config: &mut Value, key: &str) -> bool {
    let obj = config.as_object_mut().unwrap();
    let mut val = obj
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    ui.label(egui::RichText::new(label).size(11.0));
    let resp = ui.add(
        egui::TextEdit::singleline(&mut val)
            .font(egui::FontId::monospace(11.0))
            .desired_width(f32::INFINITY),
    );
    if resp.changed() {
        obj.insert(key.to_owned(), Value::String(val));
        true
    } else {
        false
    }
}

fn bool_row(ui: &mut Ui, label: &str, config: &mut Value, key: &str) -> bool {
    let obj = config.as_object_mut().unwrap();
    let mut val = obj.get(key).and_then(|v| v.as_bool()).unwrap_or(false);
    if ui
        .checkbox(&mut val, egui::RichText::new(label).size(11.0))
        .changed()
    {
        obj.insert(key.to_owned(), Value::Bool(val));
        true
    } else {
        false
    }
}

fn combo_row(ui: &mut Ui, label: &str, config: &mut Value, key: &str, options: &[&str]) -> bool {
    let obj = config.as_object_mut().unwrap();
    let mut current = obj
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(options.first().copied().unwrap_or(""))
        .to_owned();
    ui.label(egui::RichText::new(label).size(11.0));
    let mut changed = false;
    ComboBox::from_id_salt(format!("combo_{}_{}", key, label))
        .selected_text(&current)
        .width(f32::INFINITY)
        .show_ui(ui, |ui| {
            for &opt in options {
                if ui
                    .selectable_value(&mut current, opt.to_owned(), opt)
                    .changed()
                {
                    changed = true;
                }
            }
        });
    if changed {
        obj.insert(key.to_owned(), Value::String(current));
    }
    changed
}

fn u64_row(ui: &mut Ui, label: &str, config: &mut Value, key: &str) -> bool {
    let obj = config.as_object_mut().unwrap();
    let mut val: u64 = obj.get(key).and_then(|v| v.as_u64()).unwrap_or(0);
    ui.label(egui::RichText::new(label).size(11.0));
    let resp = ui.add(egui::DragValue::new(&mut val));
    if resp.changed() {
        obj.insert(key.to_owned(), Value::Number(val.into()));
        true
    } else {
        false
    }
}

fn i64_row(ui: &mut Ui, label: &str, config: &mut Value, key: &str) -> bool {
    let obj = config.as_object_mut().unwrap();
    let mut val: i64 = obj.get(key).and_then(|v| v.as_i64()).unwrap_or(1);
    ui.label(egui::RichText::new(label).size(11.0));
    let resp = ui.add(egui::DragValue::new(&mut val));
    if resp.changed() {
        obj.insert(key.to_owned(), Value::Number(val.into()));
        true
    } else {
        false
    }
}

fn text_area_row(ui: &mut Ui, label: &str, config: &mut Value, key: &str, rows: usize) -> bool {
    let obj = config.as_object_mut().unwrap();
    let mut val = obj
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    ui.label(egui::RichText::new(label).size(11.0));
    let resp = ui.add(
        egui::TextEdit::multiline(&mut val)
            .font(egui::FontId::monospace(11.0))
            .desired_rows(rows)
            .desired_width(f32::INFINITY),
    );
    if resp.changed() {
        obj.insert(key.to_owned(), Value::String(val));
        true
    } else {
        false
    }
}

/// String array rendered as one item per line in a textarea
fn str_list_row(ui: &mut Ui, label: &str, config: &mut Value, key: &str) -> bool {
    let obj = config.as_object_mut().unwrap();
    let current_text = obj
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    ui.label(egui::RichText::new(label).size(11.0));
    let mut buf = current_text;
    let resp = ui.add(
        egui::TextEdit::multiline(&mut buf)
            .font(egui::FontId::monospace(11.0))
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    if resp.changed() {
        let arr: Vec<Value> = buf
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| Value::String(l.trim().to_owned()))
            .collect();
        obj.insert(key.to_owned(), Value::Array(arr));
        true
    } else {
        false
    }
}

/// Dynamic list editor for SortRows sort keys
fn sort_keys_editor(ui: &mut Ui, config: &mut Value) -> bool {
    let obj = config.as_object_mut().unwrap();

    let mut keys: Vec<(String, bool)> = obj
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|k| {
                    let field = k
                        .get("field")
                        .and_then(|f| f.as_str())
                        .unwrap_or("")
                        .to_owned();
                    let asc = k.get("ascending").and_then(|a| a.as_bool()).unwrap_or(true);
                    (field, asc)
                })
                .collect()
        })
        .unwrap_or_default();

    ui.label(egui::RichText::new("Sort keys").size(11.0));

    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (i, (field, asc)) in keys.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::TextEdit::singleline(field)
                        .font(egui::FontId::monospace(11.0))
                        .desired_width(110.0),
                )
                .changed()
            {
                changed = true;
            }
            let old = *asc;
            ui.selectable_value(asc, true, "↑");
            ui.selectable_value(asc, false, "↓");
            if *asc != old {
                changed = true;
            }
            if ui.small_button("−").clicked() {
                to_remove = Some(i);
                changed = true;
            }
        });
    }

    if let Some(i) = to_remove {
        keys.remove(i);
    }

    if ui.small_button("＋ Add key").clicked() {
        keys.push(("field".to_owned(), true));
        changed = true;
    }

    if changed {
        let arr: Vec<Value> = keys
            .iter()
            .map(|(f, a)| serde_json::json!({ "field": f, "ascending": a }))
            .collect();
        obj.insert("keys".to_owned(), Value::Array(arr));
    }

    changed
}

/// Dynamic list editor for AddConstants constant fields
fn const_fields_editor(ui: &mut Ui, config: &mut Value) -> bool {
    let obj = config.as_object_mut().unwrap();

    let mut items: Vec<(String, String, String)> = obj
        .get("fields")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|f| {
                    let name = f
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    let value = f
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    let typ = f
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("String")
                        .to_owned();
                    (name, value, typ)
                })
                .collect()
        })
        .unwrap_or_default();

    ui.label(egui::RichText::new("Constant fields").size(11.0));

    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (i, (name, value, typ)) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::TextEdit::singleline(name)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("name")
                        .desired_width(70.0),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    egui::TextEdit::singleline(value)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("value")
                        .desired_width(70.0),
                )
                .changed()
            {
                changed = true;
            }
        });
        ui.horizontal(|ui| {
            let old_typ = typ.clone();
            ComboBox::from_id_salt(format!("const_type_{}", i))
                .selected_text(typ.as_str())
                .width(90.0)
                .show_ui(ui, |ui| {
                    for t in &["String", "Integer", "Float", "Boolean"] {
                        ui.selectable_value(typ, t.to_string(), *t);
                    }
                });
            if *typ != old_typ {
                changed = true;
            }
            if ui.small_button("−").clicked() {
                to_remove = Some(i);
                changed = true;
            }
        });
        ui.add_space(2.0);
    }

    if let Some(i) = to_remove {
        items.remove(i);
    }

    if ui.small_button("＋ Add field").clicked() {
        items.push(("name".to_owned(), "value".to_owned(), "String".to_owned()));
        changed = true;
    }

    if changed {
        let arr: Vec<Value> = items
            .iter()
            .map(|(n, v, t)| serde_json::json!({ "name": n, "value": v, "type": t }))
            .collect();
        obj.insert("fields".to_owned(), Value::Array(arr));
    }

    changed
}

/// Dynamic list editor for IfNull replacements
fn if_null_editor(ui: &mut Ui, config: &mut Value) -> bool {
    let obj = config.as_object_mut().unwrap();

    let mut items: Vec<(String, String)> = obj
        .get("replacements")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|r| {
                    let field = r
                        .get("field")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    let default_value = r
                        .get("default_value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    (field, default_value)
                })
                .collect()
        })
        .unwrap_or_default();

    ui.label(egui::RichText::new("Null replacements").size(11.0));

    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (i, (field, default)) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::TextEdit::singleline(field)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("field")
                        .desired_width(100.0),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    egui::TextEdit::singleline(default)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("default")
                        .desired_width(100.0),
                )
                .changed()
            {
                changed = true;
            }
            if ui.small_button("−").clicked() {
                to_remove = Some(i);
                changed = true;
            }
        });
    }

    if let Some(i) = to_remove {
        items.remove(i);
    }

    if ui.small_button("＋ Add").clicked() {
        items.push(("field".to_owned(), "".to_owned()));
        changed = true;
    }

    if changed {
        let arr: Vec<Value> = items
            .iter()
            .map(|(f, d)| serde_json::json!({ "field": f, "default_value": d }))
            .collect();
        obj.insert("replacements".to_owned(), Value::Array(arr));
    }

    changed
}

/// Dynamic list editor for ReplaceInString replacements
fn replace_in_string_editor(ui: &mut Ui, config: &mut Value) -> bool {
    let obj = config.as_object_mut().unwrap();

    let mut items: Vec<(String, String, String)> = obj
        .get("replacements")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|r| {
                    let field = r
                        .get("field")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    let search = r
                        .get("search")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    let replace = r
                        .get("replace_with")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    (field, search, replace)
                })
                .collect()
        })
        .unwrap_or_default();

    ui.label(egui::RichText::new("Replacements").size(11.0));

    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (i, (field, search, replace)) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::TextEdit::singleline(field)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("field")
                        .desired_width(70.0),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    egui::TextEdit::singleline(search)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("search")
                        .desired_width(70.0),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    egui::TextEdit::singleline(replace)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("replace")
                        .desired_width(70.0),
                )
                .changed()
            {
                changed = true;
            }
            if ui.small_button("−").clicked() {
                to_remove = Some(i);
                changed = true;
            }
        });
    }

    if let Some(i) = to_remove {
        items.remove(i);
    }

    if ui.small_button("＋ Add").clicked() {
        items.push(("field".to_owned(), "".to_owned(), "".to_owned()));
        changed = true;
    }

    if changed {
        let arr: Vec<Value> = items
            .iter()
            .map(|(f, s, r)| {
                serde_json::json!({ "field": f, "search": s, "replace_with": r })
            })
            .collect();
        obj.insert("replacements".to_owned(), Value::Array(arr));
    }

    changed
}

/// Raw JSON fallback for complex / unknown configs
fn json_fallback(
    ui: &mut Ui,
    config: &mut Value,
    rows: usize,
    node_id: &str,
    json_buf: &mut HashMap<String, String>,
) -> bool {
    let current_json = serde_json::to_string_pretty(config).unwrap_or_default();
    let buf = json_buf
        .entry(node_id.to_string())
        .or_insert_with(|| current_json);
    let resp = ui.add(
        egui::TextEdit::multiline(buf)
            .font(egui::FontId::monospace(11.0))
            .desired_rows(rows)
            .desired_width(f32::INFINITY),
    );
    if resp.changed() {
        if let Ok(v) = serde_json::from_str::<Value>(buf) {
            *config = v;
            return true;
        }
    }
    false
}
