#![cfg_attr(not(test), warn(clippy::unwrap_used))]
// Warn on `.unwrap()` in non-test code so any new Category D site
// (user-input reachable panic) is rejected at review time. See
// `docs/UNWRAP_CLASSIFICATION.md`. Test code is exempt.

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
mod effect_purity;
mod free_vars;
mod lookup;
mod resolve;
mod scope;
mod totality;
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
    GenericCallee,
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
    // Phase 17.H1: file identity used to namespace expr_types keys across
    // modules. Set via `set_current_file_id` before check_module; used to
    // stamp every `expr_types.insert` with the active module's id even
    // when the parser-produced span still has file_id=0.
    pub(crate) current_file_id: u32,

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
    pub(crate) globals: HashMap<String, ResolvedType>, // Global variable name -> type

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

    // 2-level memoization cache for substitute_generics
    //
    // L1: Small fixed-size cache (16 entries) for fast path lookups.
    //     Uses full (u64, u64) hash keys with direct-mapped indexing.
    //     Avoids HashMap overhead for the most frequently accessed entries.
    //
    // L2: Bounded HashMap (max 256 entries) for overflow.
    //     When capacity is exceeded, the oldest half is evicted.
    pub(crate) substitute_cache_l1: RefCell<Vec<((u64, u64), ResolvedType)>>,
    pub(crate) substitute_cache_l2: RefCell<HashMap<(u64, u64), ResolvedType>>,

    // Lifetime inference engine
    pub(crate) lifetime_inferencer: lifetime::LifetimeInferencer,

    // Ownership checking mode: None = disabled, Some(true) = strict (errors), Some(false) = warn only
    pub(crate) ownership_check_mode: Option<bool>,

    // Number of items imported from other modules (skip ownership checking for these)
    pub(crate) imported_item_count: usize,

    // Track user-defined function names (for duplicate detection)
    // Builtins are registered before check_module and are NOT in this set.
    pub(crate) user_defined_functions: HashSet<String>,

    // Multi-error collection mode: when true, errors are collected instead of returning immediately
    pub multi_error_mode: bool,
    pub collected_errors: Vec<TypeError>,

    // Cache of dependent type predicate expressions (predicate_string -> original AST Expr)
    // Used to evaluate refinement predicates at function call sites
    pub(crate) dependent_predicates: RefCell<HashMap<String, vais_ast::Expr>>,

    // Implicit error propagation mode (--implicit-try opt-in, Phase 4b.1 / #7).
    //
    // When enabled, a call-site argument whose type is `Result<T, E>` or `Option<T>`
    // passed to a parameter of type `T` is automatically treated as if `?` had been
    // written on the argument, provided the enclosing function returns a compatible
    // `Result<_, E>` or `Option<_>`. The set of argument spans that were implicitly
    // unwrapped is recorded in `implicit_try_sites` so that codegen can emit the
    // same IR path as an explicit `Expr::Try`.
    pub implicit_try_mode: bool,
    pub(crate) implicit_try_sites: HashSet<(usize, usize)>,

    // Loop nesting depth — used to validate break/continue are inside a loop
    pub(crate) loop_depth: usize,

    // Variables that have been moved (passed by value to a function as a struct type)
    pub(crate) moved_vars: HashSet<String>,

    /// Expression types from type checking, keyed by expression span.
    /// Used by codegen to get accurate types instead of re-inferring.
    ///
    /// Phase 17.H1: key is `(file_id, start, end)` — including `file_id`
    /// prevents span collisions between expressions in different source
    /// files. Prior to this, two expressions sharing the same byte range
    /// in different modules could poison each other's stored types
    /// (observed as "body_size (I64) silently becomes Vec<u8>" in
    /// vaisdb's cross-module builds).
    pub(crate) expr_types: HashMap<(u32, usize, usize), ResolvedType>,

    /// Pending method instantiations accumulated while checking a function body
    /// when type args still contain fresh type variables. Drained at end of
    /// `check_function` after body/return unification so vars resolve to
    /// concrete types (e.g., `Vec.with_capacity(n)` inside `fn -> Vec<i64>`).
    /// Entries: (struct_name, method_name, type_args-possibly-with-vars).
    pub(crate) pending_method_instantiations: Vec<(String, String, Vec<ResolvedType>)>,

    /// Phase 6.27c.3: enum name hint stack for bare-variant disambiguation.
    /// When an Ident matches a variant in multiple enums (e.g. `Not` lives in
    /// both `TokenKind` and `UnaryOp`), the topmost hint whose enum contains
    /// the variant wins over alphabetical sort. Pushed by callers that know
    /// the expected type (e.g. struct-lit field checks, fn arg checks) and
    /// popped before the next unrelated expression is checked.
    pub(crate) enum_hint_stack: Vec<String>,

    /// Phase 17.H4.15: expected-type hint stack. When a caller knows the
    /// full expected type of a child expression (e.g. struct-literal field
    /// `applied_versions: Vec.new()` where the field type is
    /// `Vec<MigrationRecord>`), it pushes that type before recursing into
    /// `check_expr`. Zero-arg generic static methods like `Vec.new()` then
    /// unify their fresh generic type vars with the hint, so the call's
    /// stamped `expr_types` entry carries a fully-resolved generic — codegen
    /// can then route to the specialized `Vec_new$MigrationRecord`.
    pub(crate) expected_type_stack: Vec<ResolvedType>,

    /// Phase Ω P1.7 (iter 134): lambda-param hint stack. Pushed by callers
    /// that know the expected closure-param type (e.g. `Vec<T>.sort_by(|a, b|
    /// ...)` pushes `&T` so `a`/`b`'s `Type::Infer` resolves to a typed Var
    /// instead of an opaque fresh one). The Lambda check pops one entry per
    /// param resolved via Type::Infer.
    pub(crate) lambda_param_hint_stack: Vec<ResolvedType>,
}

impl TypeChecker {
    /// Creates a new type checker with built-in types and functions registered.
    pub fn new() -> Self {
        let mut checker = Self {
            current_file_id: 0,
            structs: HashMap::with_capacity(32),
            enums: HashMap::with_capacity(16),
            unions: HashMap::new(),
            functions: HashMap::with_capacity(128), // ~109 builtins + user functions
            type_aliases: HashMap::with_capacity(16),
            trait_aliases: HashMap::new(),
            traits: HashMap::with_capacity(16),
            trait_impls: Vec::with_capacity(16),
            constants: HashMap::with_capacity(16),
            globals: HashMap::with_capacity(16),
            scopes: vec![HashMap::with_capacity(32)],
            current_fn_ret: None,
            current_fn_name: None,
            current_generics: Vec::new(),
            current_generic_bounds: HashMap::new(),
            current_const_generics: HashMap::new(),
            next_type_var: Cell::new(0),
            substitutions: HashMap::with_capacity(32),
            exhaustiveness_checker: ExhaustivenessChecker::new(),
            warnings: Vec::new(),
            generic_instantiations: HashSet::new(),
            substitute_cache_l1: RefCell::new(Vec::with_capacity(16)),
            substitute_cache_l2: RefCell::new(HashMap::with_capacity(64)),
            lifetime_inferencer: lifetime::LifetimeInferencer::new(),
            ownership_check_mode: Some(false), // warn-only by default
            imported_item_count: 0,
            user_defined_functions: HashSet::with_capacity(64),
            multi_error_mode: false,
            collected_errors: Vec::new(),
            dependent_predicates: RefCell::new(HashMap::new()),
            implicit_try_mode: false,
            implicit_try_sites: HashSet::new(),
            loop_depth: 0,
            moved_vars: HashSet::new(),
            expr_types: HashMap::new(),
            pending_method_instantiations: Vec::new(),
            enum_hint_stack: Vec::new(),
            expected_type_stack: Vec::new(),
            lambda_param_hint_stack: Vec::new(),
        };
        checker.register_builtins();
        checker
    }

    /// Enable strict ownership checking (errors instead of warnings)
    pub fn set_strict_ownership(&mut self, strict: bool) {
        self.ownership_check_mode = Some(strict);
    }

    /// Enable implicit error propagation (Phase 4b.1 / #7).
    ///
    /// When on, a call-site argument of type `Result<T, E>` passed to a `T`
    /// parameter is auto-unwrapped via the same semantics as `?`, provided
    /// the enclosing function returns a matching `Result<_, E>` (ditto for
    /// `Option<T>`). The transformation is recorded per argument span so that
    /// codegen can emit the same IR path as an explicit `Expr::Try`.
    pub fn set_implicit_try_mode(&mut self, enable: bool) {
        self.implicit_try_mode = enable;
    }

    /// Query whether a given argument span was implicitly unwrapped by the
    /// implicit error propagation pass. Codegen uses this to wrap the
    /// argument in Try semantics on the fly.
    pub fn is_implicit_try_site(&self, span: (usize, usize)) -> bool {
        self.implicit_try_sites.contains(&span)
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
                if self.multi_error_mode && self.collected_errors.len() < 200 {
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

    /// Record concrete generic struct instantiations that appear inside a type tree.
    ///
    /// Method-call based inference already records many instantiations, but HNSW-shaped
    /// data such as `Vec<Vec<Neighbor>>` can enter through struct fields or explicit
    /// local annotations before any method call needs the outer container. Codegen must
    /// still emit both `%Vec$Neighbor` and `%Vec$Vec_Neighbor` before those types are
    /// allocated or embedded in another struct.
    pub(crate) fn record_type_instantiations(&mut self, ty: &ResolvedType) {
        match ty {
            ResolvedType::Named { name, generics } => {
                for generic in generics {
                    self.record_type_instantiations(generic);
                }

                if generics.is_empty()
                    || generics
                        .iter()
                        .any(Self::contains_unresolved_instantiation_type)
                {
                    return;
                }

                self.add_instantiation(GenericInstantiation::struct_type(name, generics.clone()));
            }
            ResolvedType::Pointer(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Linear(inner)
            | ResolvedType::Affine(inner)
            | ResolvedType::Range(inner)
            | ResolvedType::Array(inner)
            | ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. }
            | ResolvedType::Dependent { base: inner, .. }
            | ResolvedType::Vector { element: inner, .. } => {
                self.record_type_instantiations(inner);
            }
            ResolvedType::ConstArray { element, .. } => {
                self.record_type_instantiations(element);
            }
            ResolvedType::Result(ok, err) | ResolvedType::Map(ok, err) => {
                self.record_type_instantiations(ok);
                self.record_type_instantiations(err);
            }
            ResolvedType::Tuple(items) => {
                for item in items {
                    self.record_type_instantiations(item);
                }
            }
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                for param in params {
                    self.record_type_instantiations(param);
                }
                self.record_type_instantiations(ret);
            }
            ResolvedType::DynTrait { generics, .. } => {
                for generic in generics {
                    self.record_type_instantiations(generic);
                }
            }
            ResolvedType::Associated { base, generics, .. } => {
                self.record_type_instantiations(base);
                for generic in generics {
                    self.record_type_instantiations(generic);
                }
            }
            _ => {}
        }
    }

    fn contains_unresolved_instantiation_type(ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Var(_)
            | ResolvedType::Unknown
            | ResolvedType::Generic(_)
            | ResolvedType::ConstGeneric(_)
            | ResolvedType::Lifetime(_) => true,
            ResolvedType::Pointer(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Linear(inner)
            | ResolvedType::Affine(inner)
            | ResolvedType::Range(inner)
            | ResolvedType::Array(inner)
            | ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. }
            | ResolvedType::Dependent { base: inner, .. }
            | ResolvedType::Vector { element: inner, .. } => {
                Self::contains_unresolved_instantiation_type(inner)
            }
            ResolvedType::ConstArray { element, .. } => {
                Self::contains_unresolved_instantiation_type(element)
            }
            ResolvedType::Result(ok, err) | ResolvedType::Map(ok, err) => {
                Self::contains_unresolved_instantiation_type(ok)
                    || Self::contains_unresolved_instantiation_type(err)
            }
            ResolvedType::Tuple(items) => items
                .iter()
                .any(Self::contains_unresolved_instantiation_type),
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                params
                    .iter()
                    .any(Self::contains_unresolved_instantiation_type)
                    || Self::contains_unresolved_instantiation_type(ret)
            }
            ResolvedType::Named { generics, .. } | ResolvedType::DynTrait { generics, .. } => {
                generics
                    .iter()
                    .any(Self::contains_unresolved_instantiation_type)
            }
            ResolvedType::Associated { base, generics, .. } => {
                Self::contains_unresolved_instantiation_type(base)
                    || generics
                        .iter()
                        .any(Self::contains_unresolved_instantiation_type)
            }
            _ => false,
        }
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

    /// Get all function signatures INCLUDING struct methods with mangled names.
    /// E.g., TestSuite's `new` method becomes key `TestSuite_new`.
    /// This is needed for codegen return type resolution in monolithic builds
    /// where methods from imported modules may not be in the flat `functions` map.
    pub fn get_all_functions_with_methods(&self) -> HashMap<String, FunctionSig> {
        let mut result = self.functions.clone();
        for (struct_name, struct_def) in &self.structs {
            for (method_name, method_sig) in &struct_def.methods {
                let mangled = format!("{}_{}", struct_name, method_name);
                result.entry(mangled).or_insert_with(|| method_sig.clone());
            }
        }
        result
    }

    /// Get all type aliases (for passing to codegen)
    pub fn get_type_aliases(&self) -> &HashMap<String, ResolvedType> {
        &self.type_aliases
    }

    /// Get expression types recorded during type checking (for passing to codegen).
    /// Keyed by `(file_id, span.start, span.end)` triples (Phase 17.H1).
    pub fn get_expr_types(&self) -> &HashMap<(u32, usize, usize), ResolvedType> {
        &self.expr_types
    }

    /// Return the first type fragment that is not allowed to reach codegen.
    ///
    /// Generic and const-generic parameters are intentionally not rejected
    /// here: generic templates may still carry them until monomorphization.
    /// `ResolvedType::Var` and `ResolvedType::Unknown` are different — they
    /// mean type checking did not finish a concrete obligation, so codegen
    /// would have to guess.
    pub fn codegen_unresolved_type(ty: &ResolvedType) -> Option<String> {
        match ty {
            ResolvedType::Var(id) => Some(format!("type variable #{}", id)),
            ResolvedType::Unknown => Some("unknown type".to_string()),
            ResolvedType::Pointer(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Linear(inner)
            | ResolvedType::Affine(inner)
            | ResolvedType::Range(inner)
            | ResolvedType::Array(inner) => Self::codegen_unresolved_type(inner),
            ResolvedType::ConstArray { element, .. } => Self::codegen_unresolved_type(element),
            ResolvedType::Result(ok, err) | ResolvedType::Map(ok, err) => {
                Self::codegen_unresolved_type(ok).or_else(|| Self::codegen_unresolved_type(err))
            }
            ResolvedType::Tuple(items) => items.iter().find_map(Self::codegen_unresolved_type),
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                params
                    .iter()
                    .find_map(Self::codegen_unresolved_type)
                    .or_else(|| Self::codegen_unresolved_type(ret))
            }
            ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. } => Self::codegen_unresolved_type(inner),
            ResolvedType::Dependent { base, .. } => Self::codegen_unresolved_type(base),
            ResolvedType::Vector { element, .. } => Self::codegen_unresolved_type(element),
            ResolvedType::Named { generics, .. } | ResolvedType::DynTrait { generics, .. } => {
                generics.iter().find_map(Self::codegen_unresolved_type)
            }
            ResolvedType::Associated { base, generics, .. } => Self::codegen_unresolved_type(base)
                .or_else(|| generics.iter().find_map(Self::codegen_unresolved_type)),
            _ => None,
        }
    }

    /// Validate the post-typecheck expression type map that codegen consumes.
    ///
    /// This is the Core certification boundary: a successful type check must
    /// not leave `Unknown` or inference variables for codegen to reinterpret.
    pub fn assert_fully_resolved_for_codegen(&self) -> TypeResult<()> {
        self.assert_fully_resolved_for_codegen_where(|_| true)
    }

    /// Validate only expression types stamped for one source file.
    ///
    /// This is useful for Core certification while imported generic stdlib
    /// templates can still contain inference variables that are resolved later
    /// by monomorphization.
    pub fn assert_fully_resolved_for_codegen_file(&self, file_id: u32) -> TypeResult<()> {
        self.assert_fully_resolved_for_codegen_where(|key| key.0 == file_id)
    }

    /// Validate expression types for the original source range only.
    ///
    /// Some build paths combine imported modules into one AST while stamping
    /// expression types with the main file id. Their spans sit outside the
    /// original source length, so this keeps Core certification focused on the
    /// user input file until import/template certification is split out.
    pub fn assert_fully_resolved_for_codegen_source(
        &self,
        file_id: u32,
        source_len: usize,
    ) -> TypeResult<()> {
        self.assert_fully_resolved_for_codegen_where(|key| {
            key.0 == file_id && key.1 <= source_len && key.2 <= source_len
        })
    }

    fn assert_fully_resolved_for_codegen_where<F>(&self, include: F) -> TypeResult<()>
    where
        F: Fn(&(u32, usize, usize)) -> bool,
    {
        for (key, ty) in self.get_resolved_expr_types() {
            if !include(&key) {
                continue;
            }
            if let Some(unresolved) = Self::codegen_unresolved_type(&ty) {
                return Err(TypeError::InferFailed {
                    kind: "expression type".to_string(),
                    name: format!("span({},{},{})", key.0, key.1, key.2),
                    context: format!("codegen input contains {}", unresolved),
                    span: None,
                    suggestion: Some(
                        "Resolve this type during type checking before codegen".to_string(),
                    ),
                });
            }
        }
        Ok(())
    }

    /// Phase 17.H1: set the file identifier used when `check_expr` stamps
    /// its span key into `expr_types`. The driver should call this before
    /// `check_module` for each module being typechecked, passing a distinct
    /// id per source file (typically derived from the canonical source
    /// path). A stable mapping across runs is not required; only
    /// uniqueness within a single check session is.
    pub fn set_current_file_id(&mut self, file_id: u32) {
        self.current_file_id = file_id;
    }

    /// Phase 6.27b: return expression types with all type variables resolved
    /// via the TC's final substitution map. This surfaces information that
    /// was captured during check_expr but wasn't yet fully unified at that
    /// point (e.g., `v := Vec.with_capacity(0)` gets Vec<Var(N)>, which
    /// only becomes Vec<Tuple<..>> after a later `v.push(tuple)` unifies N).
    pub fn get_resolved_expr_types(&self) -> HashMap<(u32, usize, usize), ResolvedType> {
        self.expr_types
            .iter()
            .map(|(k, v)| (*k, self.apply_substitutions(v)))
            .collect()
    }

    /// Get the set of argument spans that were auto-unwrapped by the
    /// implicit error propagation pass (Phase 4b.1 / #7).
    ///
    /// Codegen consumes this via `set_implicit_try_sites` to wrap the
    /// corresponding call-site arguments in `Expr::Try` semantics.
    pub fn get_implicit_try_sites(&self) -> &HashSet<(usize, usize)> {
        &self.implicit_try_sites
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
        self.trait_aliases = other.trait_aliases.clone();
        self.traits = other.traits.clone();
        self.trait_impls = other.trait_impls.clone();
        self.constants = other.constants.clone();
        self.globals = other.globals.clone();
    }

    /// Merge type definitions from another checker (for parallel type-checking).
    /// Extends this checker with types from another, allowing duplicates to be overwritten.
    pub fn merge_type_defs_from(&mut self, other: TypeChecker) {
        self.structs.extend(other.structs);
        self.enums.extend(other.enums);
        self.unions.extend(other.unions);
        self.functions.extend(other.functions);
        self.type_aliases.extend(other.type_aliases);
        self.trait_aliases.extend(other.trait_aliases);
        self.traits.extend(other.traits);
        self.trait_impls.extend(other.trait_impls);
        self.constants.extend(other.constants);
        self.globals.extend(other.globals);
        self.warnings.extend(other.warnings);
        self.collected_errors.extend(other.collected_errors);

        // Phase 17.H1: merge per-module expr_types so codegen can look up
        // TC-resolved types via `(file_id, start, end)` keys. Each source
        // module stamped its entries with a distinct `file_id`, so keys
        // cannot collide between modules.
        self.expr_types.extend(other.expr_types);
        self.implicit_try_sites.extend(other.implicit_try_sites);

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
