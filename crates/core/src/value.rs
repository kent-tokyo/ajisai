use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    Timestamp,
    Bytes,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::String    => write!(f, "String"),
            ValueType::Integer   => write!(f, "Integer"),
            ValueType::Float     => write!(f, "Float"),
            ValueType::Boolean   => write!(f, "Boolean"),
            ValueType::Date      => write!(f, "Date"),
            ValueType::Timestamp => write!(f, "Timestamp"),
            ValueType::Bytes     => write!(f, "Bytes"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    /// Days since Unix epoch
    Date(i32),
    /// Microseconds since Unix epoch
    Timestamp(i64),
    Bytes(Vec<u8>),
    Null,
}

impl Value {
    pub fn value_type(&self) -> Option<ValueType> {
        match self {
            Value::Str(_)       => Some(ValueType::String),
            Value::Int(_)       => Some(ValueType::Integer),
            Value::Float(_)     => Some(ValueType::Float),
            Value::Bool(_)      => Some(ValueType::Boolean),
            Value::Date(_)      => Some(ValueType::Date),
            Value::Timestamp(_) => Some(ValueType::Timestamp),
            Value::Bytes(_)     => Some(ValueType::Bytes),
            Value::Null         => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Value::Str(s) = self { Some(s) } else { None }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Value::Int(n) = self { Some(*n) } else { None }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(n)   => Some(*n as f64),
            _               => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self { Some(*b) } else { None }
    }

    /// Coerce value to string representation
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Str(s)       => s.clone(),
            Value::Int(n)       => n.to_string(),
            Value::Float(f)     => f.to_string(),
            Value::Bool(b)      => b.to_string(),
            Value::Date(d)      => d.to_string(),
            Value::Timestamp(t) => t.to_string(),
            Value::Bytes(b)     => format!("<{} bytes>", b.len()),
            Value::Null         => String::from(""),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self { Value::Str(s) }
}
impl From<&str> for Value {
    fn from(s: &str) -> Self { Value::Str(s.to_owned()) }
}
impl From<i64> for Value {
    fn from(n: i64) -> Self { Value::Int(n) }
}
impl From<f64> for Value {
    fn from(f: f64) -> Self { Value::Float(f) }
}
impl From<bool> for Value {
    fn from(b: bool) -> Self { Value::Bool(b) }
}

/// Definition of a single column
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name:       String,
    pub value_type: ValueType,
    pub nullable:   bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length:     Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision:  Option<usize>,
}

impl Field {
    pub fn new(name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
            nullable: true,
            length: None,
            precision: None,
        }
    }
}

/// Ordered set of field definitions shared across rows
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RowSchema {
    pub fields: Vec<Field>,
}

impl RowSchema {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// A single row of data with a shared schema reference
#[derive(Debug, Clone)]
pub struct Row {
    pub schema: Arc<RowSchema>,
    pub values: Vec<Value>,
}

impl Row {
    pub fn new(schema: Arc<RowSchema>, values: Vec<Value>) -> Self {
        Self { schema, values }
    }

    pub fn empty(schema: Arc<RowSchema>) -> Self {
        let len = schema.fields.len();
        Self {
            schema,
            values: vec![Value::Null; len],
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        let idx = self.schema.field_index(name)?;
        self.values.get(idx)
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&Value> {
        self.values.get(idx)
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if let Some(idx) = self.schema.field_index(name) {
            self.values[idx] = value;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_get_set() {
        let schema = Arc::new(RowSchema::new(vec![
            Field::new("name", ValueType::String),
            Field::new("age",  ValueType::Integer),
        ]));
        let mut row = Row::empty(schema);
        row.set("name", Value::Str("Alice".into()));
        row.set("age",  Value::Int(30));

        assert_eq!(row.get("name"), Some(&Value::Str("Alice".into())));
        assert_eq!(row.get("age"),  Some(&Value::Int(30)));
        assert_eq!(row.get("missing"), None);
    }
}
