use thiserror::Error;

#[derive(Debug, Error)]
pub enum AjisaiError {
    #[error("Transform error in '{transform}': {message}")]
    Transform { transform: String, message: String },

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Schema mismatch: field '{field}' not found")]
    SchemaMismatch { field: String },

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError { expected: String, actual: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, AjisaiError>;
