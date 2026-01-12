//! AOEL Type Checker
//!
//! Performs semantic analysis on parsed AOEL AST.
//!
//! # Example
//!
//! ```ignore
//! use aoel_parser::parse;
//! use aoel_typeck::check;
//!
//! let source = r#"
//! UNIT FUNCTION test V1.0.0
//! ...
//! END
//! "#;
//!
//! let unit = parse(source)?;
//! check(&unit)?;  // Returns Ok(()) or Err(Vec<TypeCheckError>)
//! ```

mod error;
mod symbol;
mod types;
mod infer;
mod checker;

pub use error::{TypeCheckError, TypeCheckResult};
pub use symbol::{SymbolTable, Symbol, SymbolKind, ScopeLevel};
pub use checker::TypeChecker;

/// Type check an AOEL unit
///
/// Returns `Ok(())` if the unit is valid, or `Err(Vec<TypeCheckError>)`
/// containing all type errors found.
pub fn check(unit: &aoel_ast::Unit) -> TypeCheckResult<()> {
    TypeChecker::new(unit).check()
}

#[cfg(test)]
mod tests;
