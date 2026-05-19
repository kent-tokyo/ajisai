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
