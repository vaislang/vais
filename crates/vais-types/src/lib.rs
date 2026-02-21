//! Vais Type System
//!
//! Static type checking with inference for AI-optimized code generation.

// Public modules
pub mod comptime;
pub mod effects;
pub mod error_report;
pub mod exhaustiveness;
pub mod object_safety;
pub mod specialization;
pub mod types;

// Private modules
mod inference;
pub mod lifetime;
pub mod ownership;
mod traits;

// Checker modules (TypeChecker impl split)
mod builtins;
mod checker_expr;
mod checker_fn;
mod checker_module;
mod free_vars;
mod lookup;
mod resolve;
mod scope;
// Re-export bidirectional type checking support
pub use inference::CheckMode;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

// Re-export core types
pub use comptime::{ComptimeEvaluator, ComptimeValue};
pub use effects::EffectInferrer;
pub use exhaustiveness::{ExhaustivenessChecker, ExhaustivenessResult};
pub use object_safety::{check_object_safety, ObjectSafetyViolation};
pub use ownership::OwnershipChecker;
use traits::TraitImpl;
pub use traits::{AssociatedTypeDef, TraitDef, TraitMethodSig};
use types::defs::VarInfo;
pub use types::{
    find_similar_name,
    // Did-you-mean support
    levenshtein_distance,
    mangle_name,
    mangle_type,
    substitute_type,
    ConstBinOp,
    ContractClause,
    // Contract support (Design by Contract)
    ContractSpec,
    // Effect system support
    Effect,
    EffectAnnotation,
    EffectSet,
    EnumDef,
    FunctionSig,
    // Monomorphization support
    GenericInstantiation,
    InstantiationKind,
    // Linear types support
    Linearity,
    // Const generics support
    ResolvedConst,
    ResolvedType,
    StructDef,
    TypeError,
    TypeResult,
    UnionDef,
    VariantFieldTypes,
};

// Type definitions have been moved to the types module

/// Static type checker with Hindley-Milner type inference.
///
/// Performs type checking, inference, and validation on the AST.
/// Supports generics, traits, and exhaustiveness checking for pattern matching.
///
/// # Examples
///
/// ```
/// use vais_types::TypeChecker;
/// use vais_parser::parse;
///
/// let source = "F id<T>(x:T)->T=x";
/// let module = parse(source).unwrap();
///
/// let mut checker = TypeChecker::new();
/// checker.check_module(&module).unwrap();
/// ```
pub struct TypeChecker {
    // Type environment
    pub(crate) structs: HashMap<String, StructDef>,
    pub(crate) enums: HashMap<String, EnumDef>,
    pub(crate) unions: HashMap<String, UnionDef>,
    pub(crate) functions: HashMap<String, FunctionSig>,
    pub(crate) type_aliases: HashMap<String, ResolvedType>,
    pub(crate) trait_aliases: HashMap<String, Vec<String>>,
    pub(crate) traits: HashMap<String, TraitDef>,
    pub(crate) trait_impls: Vec<TraitImpl>, // (type_name, trait_name) pairs
    pub(crate) constants: HashMap<String, ResolvedType>, // Constant name -> type

    // Scope stack
    pub(crate) scopes: Vec<HashMap<String, VarInfo>>,

    // Current function context
    pub(crate) current_fn_ret: Option<ResolvedType>,
    pub(crate) current_fn_name: Option<String>,

    // Current generic parameters (for type resolution)
    pub(crate) current_generics: Vec<String>,

    // Current generic bounds (maps generic param name to trait bounds)
    pub(crate) current_generic_bounds: HashMap<String, Vec<String>>,

    // Current const generic parameters (maps const param name to its type)
    pub(crate) current_const_generics: HashMap<String, ResolvedType>,

    // Current higher-kinded type parameters (maps HKT param name to arity)
    pub(crate) current_hkt_generics: HashMap<String, usize>,

    // Type variable counter for inference
    pub(crate) next_type_var: Cell<usize>,

    // Type substitutions
    pub(crate) substitutions: HashMap<usize, ResolvedType>,

    // Exhaustiveness checker for match expressions
    pub(crate) exhaustiveness_checker: ExhaustivenessChecker,

    // Warnings collected during type checking
    pub(crate) warnings: Vec<String>,

    // Generic instantiations required for monomorphization
    pub(crate) generic_instantiations: HashSet<GenericInstantiation>,

    // Memoization cache for substitute_generics
    // Key: (type hash, substitution map hash) -> Result type
    pub(crate) substitute_cache: RefCell<HashMap<(u64, u64), ResolvedType>>,

    // Lifetime inference engine
    pub(crate) lifetime_inferencer: lifetime::LifetimeInferencer,

    // Ownership checking mode: None = disabled, Some(true) = strict (errors), Some(false) = warn only
    pub(crate) ownership_check_mode: Option<bool>,

    // Number of items imported from other modules (skip ownership checking for these)
    pub(crate) imported_item_count: usize,

    // Multi-error collection mode: when true, errors are collected instead of returning immediately
    pub multi_error_mode: bool,
    pub collected_errors: Vec<TypeError>,

    // Cache of dependent type predicate expressions (predicate_string -> original AST Expr)
    // Used to evaluate refinement predicates at function call sites
    pub(crate) dependent_predicates: RefCell<HashMap<String, vais_ast::Expr>>,
}

impl TypeChecker {
    /// Creates a new type checker with built-in types and functions registered.
    pub fn new() -> Self {
        let mut checker = Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            unions: HashMap::new(),
            functions: HashMap::new(),
            type_aliases: HashMap::new(),
            trait_aliases: HashMap::new(),
            traits: HashMap::new(),
            trait_impls: Vec::new(),
            constants: HashMap::new(),
            scopes: vec![HashMap::new()],
            current_fn_ret: None,
            current_fn_name: None,
            current_generics: Vec::new(),
            current_generic_bounds: HashMap::new(),
            current_const_generics: HashMap::new(),
            current_hkt_generics: HashMap::new(),
            next_type_var: Cell::new(0),
            substitutions: HashMap::new(),
            exhaustiveness_checker: ExhaustivenessChecker::new(),
            warnings: Vec::new(),
            generic_instantiations: HashSet::new(),
            substitute_cache: RefCell::new(HashMap::new()),
            lifetime_inferencer: lifetime::LifetimeInferencer::new(),
            ownership_check_mode: Some(false), // warn-only by default
            imported_item_count: 0,
            multi_error_mode: false,
            collected_errors: Vec::new(),
            dependent_predicates: RefCell::new(HashMap::new()),
        };
        checker.register_builtins();
        checker
    }

    /// Enable strict ownership checking (errors instead of warnings)
    pub fn set_strict_ownership(&mut self, strict: bool) {
        self.ownership_check_mode = Some(strict);
    }

    /// Disable ownership checking entirely
    pub fn disable_ownership_check(&mut self) {
        self.ownership_check_mode = None;
    }

    /// Set the number of imported items to skip during ownership checking
    pub fn set_imported_item_count(&mut self, count: usize) {
        self.imported_item_count = count;
    }

    /// Collect an error instead of returning it immediately (when multi_error_mode is on)
    pub fn try_or_collect(&mut self, result: Result<(), TypeError>) -> Result<(), TypeError> {
        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                if self.multi_error_mode && self.collected_errors.len() < 20 {
                    self.collected_errors.push(e);
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Get collected errors from multi-error mode
    pub fn get_collected_errors(&self) -> &[TypeError] {
        &self.collected_errors
    }

    /// Get warnings collected during type checking
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Clear warnings
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();
    }

    /// Get generic instantiations required for monomorphization
    pub fn get_generic_instantiations(&self) -> Vec<GenericInstantiation> {
        self.generic_instantiations.iter().cloned().collect()
    }

    /// Clear generic instantiations
    pub fn clear_generic_instantiations(&mut self) {
        self.generic_instantiations.clear();
    }

    /// Add a generic instantiation if not already present
    fn add_instantiation(&mut self, inst: GenericInstantiation) {
        self.generic_instantiations.insert(inst);
    }

    /// Check if a function has generic parameters
    pub fn is_generic_function(&self, name: &str) -> bool {
        self.functions
            .get(name)
            .map(|f| !f.generics.is_empty())
            .unwrap_or(false)
    }

    /// Check if a struct has generic parameters
    pub fn is_generic_struct(&self, name: &str) -> bool {
        self.structs
            .get(name)
            .map(|s| !s.generics.is_empty())
            .unwrap_or(false)
    }

    /// Get the function signature (for codegen)
    pub fn get_function(&self, name: &str) -> Option<&FunctionSig> {
        self.functions.get(name)
    }

    /// Get all function signatures (for passing resolved types to codegen)
    pub fn get_all_functions(&self) -> &HashMap<String, FunctionSig> {
        &self.functions
    }

    /// Get all type aliases (for passing to codegen)
    pub fn get_type_aliases(&self) -> &HashMap<String, ResolvedType> {
        &self.type_aliases
    }

    /// Get the struct definition (for codegen)
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    /// Get the enum definition (for codegen)
    pub fn get_enum(&self, name: &str) -> Option<&EnumDef> {
        self.enums.get(name)
    }

    /// Get the union definition (for codegen)
    pub fn get_union(&self, name: &str) -> Option<&UnionDef> {
        self.unions.get(name)
    }

    /// Get the lifetime inferencer (for external analysis)
    pub fn get_lifetime_inferencer(&self) -> &lifetime::LifetimeInferencer {
        &self.lifetime_inferencer
    }

    /// Clone type definitions from another checker (for parallel type-checking).
    /// Copies all type information but not runtime state like scopes or current function context.
    pub fn clone_type_defs_from(&mut self, other: &TypeChecker) {
        self.structs = other.structs.clone();
        self.enums = other.enums.clone();
        self.unions = other.unions.clone();
        self.functions = other.functions.clone();
        self.type_aliases = other.type_aliases.clone();
        self.traits = other.traits.clone();
        self.trait_impls = other.trait_impls.clone();
        self.constants = other.constants.clone();
    }

    /// Merge type definitions from another checker (for parallel type-checking).
    /// Extends this checker with types from another, allowing duplicates to be overwritten.
    pub fn merge_type_defs_from(&mut self, other: TypeChecker) {
        self.structs.extend(other.structs);
        self.enums.extend(other.enums);
        self.unions.extend(other.unions);
        self.functions.extend(other.functions);
        self.type_aliases.extend(other.type_aliases);
        self.traits.extend(other.traits);
        self.trait_impls.extend(other.trait_impls);
        self.constants.extend(other.constants);
        self.warnings.extend(other.warnings);
        self.collected_errors.extend(other.collected_errors);

        for inst in other.generic_instantiations {
            self.add_instantiation(inst);
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
