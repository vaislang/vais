//! Derive macro framework
//!
//! Provides automatic code generation for common traits via #[derive(...)] attributes.
//! Note: Currently, the Vais AST does not support attributes on struct/enum definitions.
//! This module provides the infrastructure for when that support is added.
//!
//! Supported derive macros (planned):
//! - Debug: Generate debug formatting
//! - Clone: Generate clone implementation
//! - PartialEq: Generate equality comparison
//! - Default: Generate default value constructor
//! - Hash: Generate hash implementation
//! - Error: Generate Error trait implementation (code + message methods)

use std::collections::HashMap;
use vais_ast::Module;

/// Result type for derive macro operations
pub type DeriveResult<T> = Result<T, DeriveError>;

/// Error type for derive macro failures
#[derive(Debug, Clone)]
pub enum DeriveError {
    /// Unsupported derive macro
    UnsupportedDerive(String),
    /// Cannot derive for this type
    CannotDerive { derive: String, reason: String },
    /// Missing required trait bound
    MissingBound {
        derive: String,
        field: String,
        bound: String,
    },
}

impl std::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeriveError::UnsupportedDerive(name) => {
                write!(f, "Unsupported derive macro: '{}'. Supported: Debug, Clone, PartialEq, Default, Hash, Error", name)
            }
            DeriveError::CannotDerive { derive, reason } => {
                write!(f, "Cannot derive '{}': {}", derive, reason)
            }
            DeriveError::MissingBound {
                derive,
                field,
                bound,
            } => {
                write!(
                    f,
                    "Cannot derive '{}': field '{}' requires '{}' bound",
                    derive, field, bound
                )
            }
        }
    }
}

impl std::error::Error for DeriveError {}

/// Derive macro registry
///
/// Maps derive macro names to their generators.
#[derive(Default)]
pub struct DeriveRegistry {
    supported: HashMap<String, ()>,
}

impl DeriveRegistry {
    /// Create a new registry with built-in derive macros
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.supported.insert("Debug".to_string(), ());
        registry.supported.insert("Clone".to_string(), ());
        registry.supported.insert("PartialEq".to_string(), ());
        registry.supported.insert("Default".to_string(), ());
        registry.supported.insert("Hash".to_string(), ());
        registry.supported.insert("Error".to_string(), ());
        registry
    }

    /// Check if a derive macro is supported
    pub fn is_supported(&self, name: &str) -> bool {
        self.supported.contains_key(name)
    }
}

/// Trait for derive macro generators
pub trait DeriveGenerator: Send + Sync {
    /// Get the trait name this derive generates
    fn trait_name(&self) -> &str;

    /// Get required bounds for fields
    fn required_bounds(&self) -> Vec<String> {
        vec![self.trait_name().to_string()]
    }
}

/// Process derive attributes on a module and generate impl blocks
///
/// Note: Currently a no-op since the Vais AST does not support attributes
/// on struct/enum definitions. When that support is added, this function
/// will scan for #[derive(...)] attributes and generate corresponding
/// impl blocks.
pub fn process_derives(_module: &mut Module) -> DeriveResult<()> {
    // Currently a no-op - will be implemented when struct/enum attributes
    // are added to the AST
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_registry() {
        let registry = DeriveRegistry::new();
        assert!(registry.is_supported("Debug"));
        assert!(registry.is_supported("Clone"));
        assert!(registry.is_supported("PartialEq"));
        assert!(registry.is_supported("Default"));
        assert!(registry.is_supported("Hash"));
        assert!(registry.is_supported("Error"));
        assert!(!registry.is_supported("Custom"));
    }
}
