//! Core type definitions for the Vais type system
//!
//! This module contains the fundamental type definitions used throughout
//! the type checker, including resolved types, type errors, and type signatures.
//!
//! # Module Structure
//! - `defs`: Function, struct, enum, union definitions and type signatures
//! - `resolved`: ResolvedType enum and const expression types
//! - `error`: Type errors (32 variants) and error types
//! - `effects`: Effect system types (IO, Async, Unsafe, etc.)
//! - `mangle`: Name mangling for generic monomorphization
//! - `substitute`: Generic type substitution
//! - `utils`: Helper functions (levenshtein distance, name suggestions)

// Submodules
pub mod defs;
pub mod effects;
pub mod error;
pub mod mangle;
pub mod resolved;
pub mod substitute;
pub mod utils;

// Re-exports for public API
pub use defs::{
    ContractClause, ContractSpec, EnumDef, FunctionSig, GenericCallee, GenericInstantiation,
    InstantiationKind, Linearity, StructDef, UnionDef, VariantFieldTypes,
};
pub use effects::{Effect, EffectAnnotation, EffectSet};
pub use error::{TypeError, TypeResult};
pub use mangle::{mangle_name, mangle_name_with_consts, mangle_type};
pub use resolved::{ConstBinOp, ResolvedConst, ResolvedType};
pub use substitute::{substitute_const, substitute_const_values, substitute_type};
pub use utils::{find_similar_name, levenshtein_distance};
