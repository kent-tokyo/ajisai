use ajisai_core::{error::Result, value::{Value, ValueType}, AjisaiError};

/// Coerce a string value to the target ValueType.
/// Returns `Value::Null` when parsing fails for numeric types.
pub fn coerce(s: &str, vt: &ValueType) -> Value {
    match vt {
        ValueType::Integer => s.parse::<i64>().map(Value::Int).unwrap_or(Value::Null),
        ValueType::Float => s.parse::<f64>().map(Value::Float).unwrap_or(Value::Null),
        ValueType::Boolean => Value::Bool(matches!(s.to_lowercase().as_str(), "true" | "1" | "yes")),
        _ => Value::Str(s.to_owned()),
    }
}

/// Validate that a file path does not contain `..` components.
/// Returns the canonicalized `PathBuf` on success.
pub fn resolve_safe_path(path: &str) -> Result<std::path::PathBuf> {
    let p = std::path::Path::new(path);
    for component in p.components() {
        if component == std::path::Component::ParentDir {
            return Err(AjisaiError::Config(format!(
                "Path traversal not allowed: '{}'",
                path
            )));
        }
    }
    Ok(p.to_path_buf())
}
