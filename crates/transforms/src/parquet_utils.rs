use ajisai_core::value::{Value, ValueType};
use arrow_array::Array;
use arrow_schema::DataType;

pub fn arrow_to_value_type(dt: &DataType) -> ValueType {
    match dt {
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64 => ValueType::Integer,
        DataType::Float16 | DataType::Float32 | DataType::Float64 => ValueType::Float,
        DataType::Boolean => ValueType::Boolean,
        DataType::Date32 | DataType::Date64 => ValueType::Date,
        DataType::Timestamp(_, _) => ValueType::Timestamp,
        DataType::Binary | DataType::LargeBinary => ValueType::Bytes,
        _ => ValueType::String,
    }
}

pub fn value_type_to_arrow(vt: &ValueType) -> DataType {
    match vt {
        ValueType::Integer => DataType::Int64,
        ValueType::Float => DataType::Float64,
        ValueType::Boolean => DataType::Boolean,
        _ => DataType::Utf8,
    }
}

pub fn array_value_at(col: &dyn Array, row: usize) -> Value {
    use arrow_array::*;

    if col.is_null(row) {
        return Value::Null;
    }

    if let Some(a) = col.as_any().downcast_ref::<Int64Array>() {
        return Value::Int(a.value(row));
    }
    if let Some(a) = col.as_any().downcast_ref::<Int32Array>() {
        return Value::Int(a.value(row) as i64);
    }
    if let Some(a) = col.as_any().downcast_ref::<Int16Array>() {
        return Value::Int(a.value(row) as i64);
    }
    if let Some(a) = col.as_any().downcast_ref::<Int8Array>() {
        return Value::Int(a.value(row) as i64);
    }
    if let Some(a) = col.as_any().downcast_ref::<UInt64Array>() {
        return Value::Int(a.value(row) as i64);
    }
    if let Some(a) = col.as_any().downcast_ref::<UInt32Array>() {
        return Value::Int(a.value(row) as i64);
    }
    if let Some(a) = col.as_any().downcast_ref::<Float64Array>() {
        return Value::Float(a.value(row));
    }
    if let Some(a) = col.as_any().downcast_ref::<Float32Array>() {
        return Value::Float(a.value(row) as f64);
    }
    if let Some(a) = col.as_any().downcast_ref::<BooleanArray>() {
        return Value::Bool(a.value(row));
    }
    if let Some(a) = col.as_any().downcast_ref::<StringArray>() {
        return Value::Str(a.value(row).to_owned());
    }
    if let Some(a) = col.as_any().downcast_ref::<LargeStringArray>() {
        return Value::Str(a.value(row).to_owned());
    }
    if let Some(a) = col.as_any().downcast_ref::<BinaryArray>() {
        return Value::Bytes(a.value(row).to_vec());
    }

    Value::Str(format!("{:?}", col.data_type()))
}
