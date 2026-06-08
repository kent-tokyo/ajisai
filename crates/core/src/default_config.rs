/// Return default configuration JSON for a given transform type.
pub fn default_config(type_name: &str) -> serde_json::Value {
    match type_name {
        "CsvFileInput" => {
            serde_json::json!({ "filename": "input.csv",  "delimiter": ",", "has_header": true })
        }
        "CsvFileOutput" => {
            serde_json::json!({ "filename": "output.csv", "delimiter": ",", "header": true })
        }
        "JsonFileInput" => serde_json::json!({ "filename": "input.json",  "format": "array" }),
        "JsonFileOutput" => {
            serde_json::json!({ "filename": "output.json", "format": "array", "pretty": true })
        }
        "TableInput" => {
            serde_json::json!({ "connection_url": "sqlite://data.db", "sql": "SELECT * FROM table_name" })
        }
        "TableOutput" => {
            serde_json::json!({ "connection_url": "sqlite://data.db", "table": "table_name", "mode": "insert", "batch_size": 0 })
        }
        "FilterRows" => serde_json::json!({ "condition": null }),
        "SelectValues" => serde_json::json!({ "fields": [] }),
        "SortRows" => serde_json::json!({ "keys": [{ "field": "id", "ascending": true }] }),
        "AddConstants" => serde_json::json!({ "fields": [] }),
        "CalculatorStep" => serde_json::json!({ "calculations": [] }),
        "StreamLookup" => {
            serde_json::json!({ "lookup_transform": "", "key_field": "id", "lookup_key_field": "id", "return_fields": [] })
        }
        "MergeJoin" => {
            serde_json::json!({ "left_key": "id", "right_key": "id", "join_type": "inner", "right_prefix": "r_" })
        }
        "Deduplicate" => serde_json::json!({ "key_fields": [] }),
        "DatabaseLookup" => {
            serde_json::json!({ "connection_url": "sqlite://data.db", "sql": "SELECT * FROM t WHERE id = ?", "key_field": "id", "return_fields": [] })
        }
        _ => serde_json::Value::Object(serde_json::Map::new()),
    }
}
