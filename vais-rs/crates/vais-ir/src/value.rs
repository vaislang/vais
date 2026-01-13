//! Runtime values for Vais IR

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique ID for async tasks
pub type TaskId = u64;

/// Unique ID for channels
pub type ChannelId = u64;

/// Future state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FutureState {
    /// Task is still running
    Pending,
    /// Task completed with a value
    Completed(Box<Value>),
    /// Task failed with an error
    Failed(String),
}

/// Channel state (non-serializable due to Mutex)
#[derive(Debug)]
pub struct ChannelState {
    pub buffer: Vec<Value>,
    pub capacity: usize,
    pub closed: bool,
}

impl ChannelState {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
            closed: false,
        }
    }
}

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

    /// Future/Promise - async computation result
    Future(TaskId),

    /// Channel for async communication
    Channel(ChannelId),
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
            Value::Future(_) => true,
            Value::Channel(_) => true,
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
            Value::Future(_) => ValueType::Future,
            Value::Channel(_) => ValueType::Channel,
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

    /// Generate a hash key for use in HashSet/HashMap
    /// More efficient than format!("{:?}", value)
    pub fn hash_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();

        // Hash the discriminant first
        std::mem::discriminant(self).hash(&mut hasher);

        match self {
            Value::Void => {}
            Value::Bool(b) => b.hash(&mut hasher),
            Value::Int(n) => n.hash(&mut hasher),
            Value::Float(f) => {
                // Use bits for consistent hashing
                f.to_bits().hash(&mut hasher);
            }
            Value::String(s) => s.hash(&mut hasher),
            Value::Bytes(b) => b.hash(&mut hasher),
            Value::Array(arr) => {
                arr.len().hash(&mut hasher);
                for item in arr {
                    item.hash_key().hash(&mut hasher);
                }
            }
            Value::Map(m) => {
                m.len().hash(&mut hasher);
                // Sort keys for consistent hashing
                let mut keys: Vec<_> = m.keys().collect();
                keys.sort();
                for k in keys {
                    k.hash(&mut hasher);
                    if let Some(v) = m.get(k) {
                        v.hash_key().hash(&mut hasher);
                    }
                }
            }
            Value::Struct(s) => {
                s.len().hash(&mut hasher);
                let mut keys: Vec<_> = s.keys().collect();
                keys.sort();
                for k in keys {
                    k.hash(&mut hasher);
                    if let Some(v) = s.get(k) {
                        v.hash_key().hash(&mut hasher);
                    }
                }
            }
            Value::Optional(opt) => {
                opt.is_some().hash(&mut hasher);
                if let Some(v) = opt {
                    v.hash_key().hash(&mut hasher);
                }
            }
            Value::Error(e) => e.hash(&mut hasher),
            Value::Closure { params, body_id, .. } => {
                params.hash(&mut hasher);
                body_id.hash(&mut hasher);
            }
            Value::Future(id) => id.hash(&mut hasher),
            Value::Channel(id) => id.hash(&mut hasher),
        }

        hasher.finish()
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
            Value::Future(id) => write!(f, "<future:{}>", id),
            Value::Channel(id) => write!(f, "<channel:{}>", id),
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
    Future,
    Channel,
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
            ValueType::Future => write!(f, "FUTURE"),
            ValueType::Channel => write!(f, "CHANNEL"),
        }
    }
}
