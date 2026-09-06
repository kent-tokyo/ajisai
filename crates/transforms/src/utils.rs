use ajisai_core::{
    AjisaiError,
    error::Result,
    value::{Value, ValueType},
};

/// Coerce a string value to the target ValueType.
/// Returns `Value::Null` when parsing fails for numeric types.
pub fn coerce(s: &str, vt: &ValueType) -> Value {
    match vt {
        ValueType::Integer => s.parse::<i64>().map(Value::Int).unwrap_or(Value::Null),
        ValueType::Float => s.parse::<f64>().map(Value::Float).unwrap_or(Value::Null),
        ValueType::Boolean => {
            Value::Bool(matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
        }
        _ => Value::Str(s.to_owned()),
    }
}

/// Validate that a file path does not contain `..` components.
/// Returns the canonicalized `PathBuf` on success.
pub fn resolve_safe_path(path: &str) -> Result<std::path::PathBuf> {
    let p = std::path::Path::new(path);
    let mut current = std::path::PathBuf::new();
    for component in p.components() {
        if component == std::path::Component::ParentDir {
            return Err(AjisaiError::Config(format!(
                "Path traversal not allowed: '{}'",
                path
            )));
        }
        current.push(component.as_os_str());
        if let Ok(metadata) = std::fs::symlink_metadata(&current) {
            // macOS exposes /var and /tmp as fixed system links into /private.
            // Permit only these known OS aliases; reject all project/user links.
            let known_macos_alias = cfg!(target_os = "macos")
                && (current == std::path::Path::new("/var")
                    || current == std::path::Path::new("/tmp"));
            if metadata.file_type().is_symlink() && !known_macos_alias {
                return Err(AjisaiError::Config(format!(
                    "Symlink path component not allowed: '{}'",
                    current.display()
                )));
            }
        }
    }
    Ok(p.to_path_buf())
}

/// Resolve a project-relative path while enforcing an explicit project root.
/// The root must already exist; output files may be new descendants.
pub fn resolve_path_in_root(
    root: &std::path::Path,
    relative_path: &str,
) -> Result<std::path::PathBuf> {
    let canonical_root = root.canonicalize().map_err(AjisaiError::Io)?;
    let candidate = std::path::Path::new(relative_path);
    if candidate.is_absolute() {
        return Err(AjisaiError::Config(format!(
            "Absolute paths are not allowed in a project root: '{}'",
            relative_path
        )));
    }
    let joined = canonical_root.join(candidate);
    let safe = resolve_safe_path(
        joined
            .to_str()
            .ok_or_else(|| AjisaiError::Config("Project path is not valid UTF-8".into()))?,
    )?;
    if !safe.starts_with(&canonical_root) {
        return Err(AjisaiError::Config(format!(
            "Path escapes project root: '{}'",
            relative_path
        )));
    }
    Ok(safe)
}

/// Resolve a path using the optional execution project root.
pub fn resolve_context_path(
    context: &ajisai_core::ExecutionContext,
    path: &str,
) -> Result<std::path::PathBuf> {
    if let Some(root) = context.project_root() {
        resolve_path_in_root(root, path)
    } else {
        resolve_safe_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_components() {
        assert!(resolve_safe_path("output/../secret").is_err());
    }

    #[test]
    fn enforces_explicit_project_root() {
        let root = tempfile::tempdir().unwrap();
        let inside = resolve_path_in_root(root.path(), "data/output.csv").unwrap();
        assert!(inside.starts_with(root.path().canonicalize().unwrap()));
        assert!(resolve_path_in_root(root.path(), "../outside.csv").is_err());
        assert!(resolve_path_in_root(root.path(), root.path().to_str().unwrap()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_components() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        let link = root.path().join("link");
        std::fs::create_dir(&target).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(resolve_safe_path(link.join("out.csv").to_str().unwrap()).is_err());
    }
}
