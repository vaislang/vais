use crate::JitError;

/// Interpreter value representation.
#[derive(Debug, Clone)]
pub enum Value {
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Unit,
    Pointer(usize),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
}

impl Value {
    /// Converts to i64, returning an error if not possible.
    pub fn as_i64(&self) -> Result<i64, JitError> {
        match self {
            Value::I64(n) => Ok(*n),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(JitError::InvalidConversion {
                from: format!("{:?}", self),
                to: "i64".to_string(),
            }),
        }
    }

    /// Converts to f64, returning an error if not possible.
    pub fn as_f64(&self) -> Result<f64, JitError> {
        match self {
            Value::F64(n) => Ok(*n),
            Value::I64(n) => Ok(*n as f64),
            _ => Err(JitError::InvalidConversion {
                from: format!("{:?}", self),
                to: "f64".to_string(),
            }),
        }
    }

    /// Converts to bool, returning an error if not possible.
    pub fn as_bool(&self) -> Result<bool, JitError> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::I64(n) => Ok(*n != 0),
            _ => Err(JitError::InvalidConversion {
                from: format!("{:?}", self),
                to: "bool".to_string(),
            }),
        }
    }
}
