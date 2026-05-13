//! Type inference logic for the Vais type system
//!
//! This module contains the type inference algorithms including unification,
//! substitution, fresh type variable generation, and bidirectional type checking.
//!
//! ## Bidirectional Type Checking
//!
//! The type checker supports two modes:
//! - `Infer`: Bottom-up inference where the type is computed from the expression
//! - `Check`: Top-down checking where the expression is verified against an expected type
//!
//! This allows for better type inference in cases like:
//! - Lambda parameters: `|x| x + 1` can infer `x: i64` from context
//! - Generic instantiation: Type arguments can be inferred from expected return type
//! - Better error messages with more precise location information

mod inference_modes;
mod substitution;
mod unification;

use crate::types::ResolvedType;

/// Mode for bidirectional type checking
#[derive(Debug, Clone)]
pub enum CheckMode {
    /// Infer the type of the expression (bottom-up)
    Infer,
    /// Check the expression against an expected type (top-down)
    Check(ResolvedType),
}

impl CheckMode {
    /// Create a Check mode with the given expected type
    pub fn check(expected: ResolvedType) -> Self {
        CheckMode::Check(expected)
    }

    /// Check if this is Infer mode
    pub fn is_infer(&self) -> bool {
        matches!(self, CheckMode::Infer)
    }

    /// Get the expected type if in Check mode
    pub fn expected(&self) -> Option<&ResolvedType> {
        match self {
            CheckMode::Infer => None,
            CheckMode::Check(ty) => Some(ty),
        }
    }
}
