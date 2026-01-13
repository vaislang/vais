//! Runtime values for AOEL IR

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Void/null value
    Void,

    /// Boolean value
    Bool(bool),

    /// 64-bit signed integer
    Int(i64),

    /// 64-bit floating point
    Float(f64),

    /// UTF-8 string
    String(String),

    /// Byte array
    Bytes(Vec<u8>),

    /// Array of values
    Array(Vec<Value>),

    /// Map/dictionary
    Map(HashMap<String, Value>),

    /// Struct with named fields
    Struct(HashMap<String, Value>),

    /// Optional value (Some or None)
    Optional(Option<Box<Value>>),

    /// Error value (for error propagation)
    Error(String),

    /// Closure (params, captured environment, body instruction index)
    /// The instructions are stored separately; this just tracks the closure metadata
    Closure {
        params: Vec<String>,
        captured: HashMap<String, Value>,
        body_id: usize,
    },
}

impl Value {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Void => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Bytes(b) => !b.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Struct(s) => !s.is_empty(),
            Value::Optional(o) => o.is_some(),
            Value::Error(_) => false,
            Value::Closure { .. } => true,
        }
    }

    /// Get the type of this value
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Void => ValueType::Void,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::Bytes(_) => ValueType::Bytes,
            Value::Array(_) => ValueType::Array,
            Value::Map(_) => ValueType::Map,
            Value::Struct(_) => ValueType::Struct,
            Value::Optional(_) => ValueType::Optional,
            Value::Error(_) => ValueType::Error,
            Value::Closure { .. } => ValueType::Closure,
        }
    }

    /// Try to get as integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Int(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as array
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get field from struct
    pub fn get_field(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Struct(s) => s.get(name),
            Value::Map(m) => m.get(name),
            _ => None,
        }
    }

    /// Get array length or string length (Unicode char count for strings)
    pub fn len(&self) -> Option<usize> {
        match self {
            Value::String(s) => Some(s.chars().count()), // Unicode char count
            Value::Bytes(b) => Some(b.len()),
            Value::Array(a) => Some(a.len()),
            Value::Map(m) => Some(m.len()),
            _ => None,
        }
    }

    /// Check if the value is empty (for collection types)
    pub fn is_empty(&self) -> Option<bool> {
        self.len().map(|l| l == 0)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "VOID"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            Value::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Struct(s) => {
                write!(f, "STRUCT {{")?;
                for (i, (k, v)) in s.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Optional(Some(v)) => write!(f, "Some({})", v),
            Value::Optional(None) => write!(f, "None"),
            Value::Error(e) => write!(f, "Error({})", e),
            Value::Closure { params, .. } => write!(f, "<closure({})>", params.join(", ")),
        }
    }
}

/// Value type enumeration (for runtime type checking)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    Void,
    Bool,
    Int,
    Float,
    String,
    Bytes,
    Array,
    Map,
    Struct,
    Optional,
    Error,
    Closure,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Void => write!(f, "VOID"),
            ValueType::Bool => write!(f, "BOOL"),
            ValueType::Int => write!(f, "INT"),
            ValueType::Float => write!(f, "FLOAT"),
            ValueType::String => write!(f, "STRING"),
            ValueType::Bytes => write!(f, "BYTES"),
            ValueType::Array => write!(f, "ARRAY"),
            ValueType::Map => write!(f, "MAP"),
            ValueType::Struct => write!(f, "STRUCT"),
            ValueType::Optional => write!(f, "OPTIONAL"),
            ValueType::Error => write!(f, "ERROR"),
            ValueType::Closure => write!(f, "CLOSURE"),
        }
    }
}
