//! Compile-time expression evaluator
//!
//! This module provides an interpreter that evaluates expressions at compile time.
//! It supports arithmetic operations, conditionals, loops, and pure function calls.

mod builtins;
mod evaluator;
mod operators;

#[cfg(test)]
mod tests;

use crate::types::{TypeError, TypeResult};
use std::collections::HashMap;
use vais_ast::*;

/// Compile-time value
#[derive(Debug, Clone, PartialEq)]
pub enum ComptimeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<ComptimeValue>),
    Unit,
}

impl ComptimeValue {
    pub fn as_i64(&self) -> TypeResult<i64> {
        match self {
            ComptimeValue::Int(n) => Ok(*n),
            _ => Err(TypeError::Mismatch {
                expected: "i64".to_string(),
                found: format!("{:?}", self),
                span: None, // Comptime values have no source location
            }),
        }
    }

    pub fn as_f64(&self) -> TypeResult<f64> {
        match self {
            ComptimeValue::Float(f) => Ok(*f),
            ComptimeValue::Int(n) => Ok(*n as f64),
            _ => Err(TypeError::Mismatch {
                expected: "f64".to_string(),
                found: format!("{:?}", self),
                span: None, // Comptime values have no source location
            }),
        }
    }

    pub fn as_bool(&self) -> TypeResult<bool> {
        match self {
            ComptimeValue::Bool(b) => Ok(*b),
            _ => Err(TypeError::Mismatch {
                expected: "bool".to_string(),
                found: format!("{:?}", self),
                span: None, // Comptime values have no source location
            }),
        }
    }

    pub fn as_string(&self) -> TypeResult<String> {
        match self {
            ComptimeValue::String(s) => Ok(s.clone()),
            _ => Err(TypeError::Mismatch {
                expected: "string".to_string(),
                found: format!("{:?}", self),
                span: None, // Comptime values have no source location
            }),
        }
    }

    pub fn as_array(&self) -> TypeResult<Vec<ComptimeValue>> {
        match self {
            ComptimeValue::Array(arr) => Ok(arr.clone()),
            _ => Err(TypeError::Mismatch {
                expected: "array".to_string(),
                found: format!("{:?}", self),
                span: None, // Comptime values have no source location
            }),
        }
    }
}

impl std::fmt::Display for ComptimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComptimeValue::Int(n) => write!(f, "{}", n),
            ComptimeValue::Float(fl) => write!(f, "{}", fl),
            ComptimeValue::Bool(b) => write!(f, "{}", b),
            ComptimeValue::String(s) => write!(f, "\"{}\"", s),
            ComptimeValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            ComptimeValue::Unit => write!(f, "()"),
        }
    }
}

/// Control flow signal for break/continue
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ControlFlow {
    None,
    Break(Option<ComptimeValue>),
    Continue,
}

/// Compile-time expression evaluator
pub struct ComptimeEvaluator {
    /// Variable bindings in current scope
    pub(crate) vars: HashMap<String, ComptimeValue>,
    /// Stack of variable scopes
    pub(crate) scope_stack: Vec<HashMap<String, ComptimeValue>>,
    /// Control flow state for break/continue
    pub(crate) control_flow: ControlFlow,
}

impl Default for ComptimeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ComptimeEvaluator {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            scope_stack: Vec::new(),
            control_flow: ControlFlow::None,
        }
    }
}
