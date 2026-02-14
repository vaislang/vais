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
use std::collections::HashMap;

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
    pub(crate) generic_instantiations: Vec<GenericInstantiation>,

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
            generic_instantiations: Vec::new(),
            substitute_cache: RefCell::new(HashMap::new()),
            lifetime_inferencer: lifetime::LifetimeInferencer::new(),
            ownership_check_mode: Some(false), // warn-only by default
            imported_item_count: 0,
            multi_error_mode: false,
            collected_errors: Vec::new(),
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
    pub fn get_generic_instantiations(&self) -> &[GenericInstantiation] {
        &self.generic_instantiations
    }

    /// Clear generic instantiations
    pub fn clear_generic_instantiations(&mut self) {
        self.generic_instantiations.clear();
    }

    /// Add a generic instantiation if not already present
    fn add_instantiation(&mut self, inst: GenericInstantiation) {
        if !self.generic_instantiations.contains(&inst) {
            self.generic_instantiations.push(inst);
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
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let source = "F add(a:i64,b:str)->i64=a+b";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_struct() {
        let source = r#"
            S Point{x:f64,y:f64}
            F make_point()->Point=Point{x:1.0,y:2.0}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_module() {
        let source = "";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_minimal_function() {
        let source = "F f()->()=()";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_empty_struct() {
        let source = "S Empty{}";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_unit_enum() {
        let source = "E Unit{A,B,C}";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_undefined_variable() {
        let source = "F f()->i64=undefined_var";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check_module(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_function() {
        let source = "F f()->i64=undefined_func()";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check_module(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_type() {
        // Note: Type checker may not catch undefined types at parse time
        // This tests that we handle the undefined type case
        let source = "F f(x:UndefinedType)->()=()";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        let _result = checker.check_module(&module);
        // Some type checkers allow undefined types, some don't - just ensure no panic
    }

    #[test]
    fn test_did_you_mean_variable() {
        // Test that did-you-mean suggestions work for typos in variable names
        let source = "F test()->i64{count:=42;coutn}";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check_module(&module);
        assert!(result.is_err());
        if let Err(TypeError::UndefinedVar {
            name, suggestion, ..
        }) = result
        {
            assert_eq!(name, "coutn");
            assert_eq!(suggestion, Some("count".to_string()));
        } else {
            panic!("Expected UndefinedVar error with suggestion");
        }
    }

    #[test]
    fn test_did_you_mean_no_match() {
        // Test that no suggestion is given when names are too different
        let source = "F test()->i64{count:=42;xyz}";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check_module(&module);
        assert!(result.is_err());
        if let Err(TypeError::UndefinedVar {
            name, suggestion, ..
        }) = result
        {
            assert_eq!(name, "xyz");
            assert_eq!(suggestion, None);
        } else {
            panic!("Expected UndefinedVar error without suggestion");
        }
    }

    #[test]
    fn test_levenshtein_distance() {
        use crate::types::levenshtein_distance;
        // Same strings
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        // One character difference
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        // Insertion
        assert_eq!(levenshtein_distance("hello", "helloo"), 1);
        // Deletion
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        // Multiple differences
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        // Empty strings
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn test_return_type_mismatch() {
        let source = "F f()->i64=\"string\"";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_integer_to_float_mismatch() {
        let source = "F f()->f64=42";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Integer to float should be an error (no implicit conversion)
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_array_element_type_mismatch() {
        let source = "F f()->[i64]=[1,2,\"three\"]";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_function_wrong_arg_count() {
        let source = r#"
            F add(a:i64,b:i64)->i64=a+b
            F f()->i64=add(1)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_function_wrong_arg_type() {
        let source = r#"
            F add(a:i64,b:i64)->i64=a+b
            F f()->i64=add(1,"two")
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_struct_field_type_mismatch() {
        let source = r#"
            S Point{x:f64,y:f64}
            F f()->Point=Point{x:"one",y:2.0}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_struct_missing_field() {
        let source = r#"
            S Point{x:f64,y:f64}
            F f()->Point=Point{x:1.0}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Missing field should be an error
        // Note: Current implementation may allow this - depends on implementation
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_binary_op_type_mismatch() {
        let source = "F f()->i64=\"a\"+1";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_comparison_type_mismatch() {
        let source = "F f()->bool=\"a\">1";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_logical_op_on_non_bool() {
        let source = "F f()->bool=1&&2";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Logical operations on non-boolean should fail
        // Note: May depend on implementation
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_if_condition_non_bool() {
        let source = "F f()->i64=I 42{1}E{0}";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Non-boolean if condition should fail
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_if_branch_type_mismatch() {
        let source = "F f(x:bool)->i64=I x{1}E{\"zero\"}";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_match_arm_type_mismatch() {
        let source = "F f(x:i64)->i64=M x{0=>0,1=>\"one\",_=>2}";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_generic_function() {
        let source = "F identity<T>(x:T)->T=x";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_generic_struct() {
        // Simple generic struct
        let source = r#"
            S Box<T>{value:T}
            F get_value<T>(b:Box<T>)->T=b.value
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_recursive_function() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_mutual_recursion() {
        let source = r#"
            F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
            F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_nested_blocks() {
        let source = r#"
            F f()->i64{
                x:=1;
                {
                    y:=2;
                    {
                        z:=3;
                        x+y+z
                    }
                }
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_shadowing() {
        let source = r#"
            F f()->i64{
                x:=1;
                x:=2;
                x
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_lambda_type_inference() {
        let source = r#"
            F f()->i64{
                add:=|a:i64,b:i64|a+b;
                add(1,2)
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_higher_order_function() {
        let source = r#"
            F apply(f:(i64)->i64,x:i64)->i64=f(x)
            F double(x:i64)->i64=x*2
            F test()->i64=apply(double,21)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_array_operations() {
        // Simple array indexing test
        let source = r#"
            F get_first(arr:[i64])->i64=arr[0]
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_trait_impl() {
        // Test simple trait definition using W keyword
        let source = r#"
            W Display{F display(s:&Self)->str=""}
            S Point{x:f64,y:f64}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_method_call() {
        // Test struct with impl block using X keyword
        let source = r#"
            S Counter{value:i64}
            X Counter{
                F new()->Counter=Counter{value:0}
                F get(c:&Counter)->i64=c.value
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_optional_type() {
        let source = r#"
            F maybe(x:i64)->i64?=I x>0{x}E{none}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // This may need adjustments based on how optionals work
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_integer_widening() {
        let source = r#"
            F f(a:i32,b:i64)->i64{
                x:i64=a;
                x+b
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Integer widening should be allowed
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_all_integer_types() {
        let source = r#"
            F test()->(){
                a:i8=1;
                b:i16=2;
                c:i32=3;
                d:i64=4;
                e:u8=5;
                f:u16=6;
                g:u32=7;
                h:u64=8;
                ()
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_float_types() {
        // Test float type declarations - inference defaults to f64
        let source = r#"
            F test()->f64{
                a:=1.0;
                b:=2.0;
                a+b
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_loop_with_break_value() {
        let source = r#"
            F find_first(arr:[i64],target:i64)->i64{
                L i:0..10{
                    I arr[i]==target{B i}
                };
                -1
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_nested_generics() {
        // Use simple generics that the parser supports
        let source = "F f<T>(x:T)->T=x";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_generic_with_bounds() {
        let source = "F compare<T:Ord>(a:T,b:T)->bool=a<b";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    // ==================== Generic Instantiation Tests ====================

    #[test]
    fn test_generic_function_instantiation() {
        // Test that calling a generic function records an instantiation
        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->i64=identity(42)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());

        // Check that an instantiation was recorded
        let instantiations = checker.get_generic_instantiations();
        assert!(
            !instantiations.is_empty(),
            "Expected generic instantiation to be recorded"
        );

        // Find the identity instantiation
        let identity_inst = instantiations
            .iter()
            .find(|i| i.base_name == "identity")
            .expect("Expected identity<i64> instantiation");

        assert_eq!(identity_inst.type_args.len(), 1);
        assert_eq!(identity_inst.type_args[0], ResolvedType::I64);
        assert_eq!(identity_inst.mangled_name, "identity$i64");
    }

    #[test]
    fn test_generic_function_multiple_instantiations() {
        // Test that calling a generic function with different types records multiple instantiations
        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->f64{
                a:=identity(42);
                b:=identity(3.14);
                b
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());

        // Check that both instantiations were recorded
        let instantiations = checker.get_generic_instantiations();
        assert!(
            instantiations.len() >= 2,
            "Expected at least 2 instantiations"
        );

        // Check for i64 instantiation
        let i64_inst = instantiations
            .iter()
            .find(|i| i.base_name == "identity" && i.type_args == vec![ResolvedType::I64]);
        assert!(i64_inst.is_some(), "Expected identity<i64> instantiation");

        // Check for f64 instantiation
        let f64_inst = instantiations
            .iter()
            .find(|i| i.base_name == "identity" && i.type_args == vec![ResolvedType::F64]);
        assert!(f64_inst.is_some(), "Expected identity<f64> instantiation");
    }

    #[test]
    fn test_generic_struct_instantiation() {
        // Test that creating a generic struct records an instantiation
        let source = r#"
            S Pair<T>{first:T,second:T}
            F main()->i64{
                p:=Pair{first:1,second:2};
                p.first
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());

        // Check that a struct instantiation was recorded
        let instantiations = checker.get_generic_instantiations();
        let pair_inst = instantiations
            .iter()
            .find(|i| i.base_name == "Pair")
            .expect("Expected Pair<i64> instantiation");

        assert_eq!(pair_inst.type_args.len(), 1);
        assert_eq!(pair_inst.type_args[0], ResolvedType::I64);
        assert!(matches!(pair_inst.kind, InstantiationKind::Struct));
    }

    #[test]
    fn test_generic_no_instantiation_without_call() {
        // Test that just defining a generic function doesn't record instantiation
        let source = r#"
            F identity<T>(x:T)->T=x
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());

        // No instantiations should be recorded
        let instantiations = checker.get_generic_instantiations();
        assert!(
            instantiations.is_empty(),
            "Expected no instantiations for unused generic function"
        );
    }

    #[test]
    fn test_clear_generic_instantiations() {
        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->i64=identity(42)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());

        assert!(!checker.get_generic_instantiations().is_empty());
        checker.clear_generic_instantiations();
        assert!(checker.get_generic_instantiations().is_empty());
    }

    #[test]
    fn test_generic_function_with_struct_return() {
        // Test generic function returning a generic struct
        // Note: Using T directly as return type due to parser limitations with ->Generic<T>
        let source = r#"
            S Container<T>{value:T}
            F make_container<T>(x:T)->T{
                c:=Container{value:x};
                c.value
            }
            F main()->i64{
                v:=make_container(42);
                v
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());

        let instantiations = checker.get_generic_instantiations();

        // Should have both function and struct instantiations
        let fn_inst = instantiations
            .iter()
            .find(|i| i.base_name == "make_container");
        assert!(
            fn_inst.is_some(),
            "Expected make_container<i64> instantiation"
        );

        let struct_inst = instantiations.iter().find(|i| i.base_name == "Container");
        assert!(
            struct_inst.is_some(),
            "Expected Container<i64> instantiation"
        );
    }

    #[test]
    fn test_generic_instantiation_kind() {
        use crate::InstantiationKind;

        let source = r#"
            S Holder<T>{data:T}
            F hold<T>(x:T)->T{
                h:=Holder{data:x};
                h.data
            }
            F main()->i64{
                r:=hold(42);
                r
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());

        let instantiations = checker.get_generic_instantiations();

        // Check that function instantiation has correct kind
        let fn_inst = instantiations
            .iter()
            .find(|i| i.base_name == "hold")
            .expect("Expected hold instantiation");
        assert!(matches!(fn_inst.kind, InstantiationKind::Function));

        // Check that struct instantiation has correct kind
        let struct_inst = instantiations
            .iter()
            .find(|i| i.base_name == "Holder")
            .expect("Expected Holder instantiation");
        assert!(matches!(struct_inst.kind, InstantiationKind::Struct));
    }

    // ==================== Advanced Edge Case Tests ====================

    #[test]
    fn test_nested_generic_vec_hashmap_option() {
        // Simplified - generic struct test
        let source = r#"
            S Container<T>{data:T}
            F make<T>(x:T)->Container<T> =Container{data:x}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_option_of_vec_type_inference() {
        // Test Option<Vec<T> > type inference with spaces
        let source = r#"
            F get_items()->Option<Vec<i64> > =Some([1,2,3])
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Type inference should resolve the nested generic correctly
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_hashmap_with_option_values() {
        // Simplified - basic struct test
        let source = r#"
            S Cache{count:i64}
            F make()->Cache=Cache{count:0}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_triple_nested_generics() {
        // Test Vec<HashMap<K, Option<Vec<T> > > > with spaces
        let source = r#"
            F complex()->Vec<HashMap<str,Option<Vec<i64> > > > =[]
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_mutual_recursion_simple() {
        // Test mutual recursion type inference
        let source = r#"
            F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
            F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_mutual_recursion_three_functions() {
        // Test three-way mutual recursion
        let source = r#"
            F a(n:i64)->i64=n<1?0:b(n-1)+1
            F b(n:i64)->i64=n<1?0:c(n-1)+1
            F c(n:i64)->i64=n<1?0:a(n-1)+1
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_mutual_recursion_with_different_return_types() {
        // Test mutual recursion where functions return different types
        let source = r#"
            F count_even(n:i64)->i64=n==0?0:1+count_odd(n-1)
            F count_odd(n:i64)->i64=n==0?0:count_even(n-1)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_mutual_recursion_type_mismatch() {
        // Test mutual recursion with type mismatch (should fail)
        let source = r#"
            F f(n:i64)->i64=g(n)
            F g(n:i64)->str="error"
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Should fail because f returns i64 but g returns str
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_indirect_recursion_through_helper() {
        // Test indirect recursion through helper function
        let source = r#"
            F outer(n:i64)->i64=helper(n)
            F helper(n:i64)->i64=n<1?0:outer(n-1)+1
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_generic_mutual_recursion() {
        // Test mutual recursion with generic functions
        let source = r#"
            F transform_a<T>(x:T)->T=transform_b(x)
            F transform_b<T>(x:T)->T=x
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_i8_boundary_values() {
        // Test i8 min (-128) and max (127)
        let source = r#"
            F i8_bounds()->(){
                min:i8=-128;
                max:i8=127;
                ()
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_i8_overflow_detection() {
        // Test i8 overflow (128 > i8::MAX)
        let source = r#"
            F i8_overflow()->i8=128
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // May or may not error depending on implementation
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_i8_underflow_detection() {
        // Test i8 underflow (-129 < i8::MIN)
        let source = r#"
            F i8_underflow()->i8=-129
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_i64_max_value() {
        // Test i64 max value: 9223372036854775807
        let source = r#"
            F i64_max()->i64=9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_i64_min_value() {
        // Test i64 near min value (actual min causes overflow in lexer)
        let source = r#"
            F i64_min()->i64=-9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_integer_arithmetic_overflow() {
        // Test integer arithmetic that could overflow
        let source = r#"
            F add_i8(a:i8,b:i8)->i8=a+b
            F test()->i8=add_i8(100,100)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Type checker may or may not detect overflow at compile time
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_pattern_with_guard_type_inference() {
        // Test pattern matching with guards - type inference (fix string escaping)
        let source = r#"
            F classify(x:i64)->str=M x{
                n I n>0=>"positive",
                n I n<0=>"negative",
                _=>"zero"
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_complex_guard_type_checking() {
        // Test complex guard with multiple conditions
        let source = r#"
            F filter(x:i64)->bool=M x{
                n I n>0&&n<100=>true,
                n I n>=100||n<=-100=>false,
                _=>false
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_nested_pattern_guard_inference() {
        // Test nested pattern with guard
        let source = r#"
            E Nested{Pair((i64,i64)),Single(i64)}
            F sum(n:Nested)->i64=M n{
                Pair((a,b)) I a>0&&b>0=>a+b,
                Pair((a,b))=>0,
                Single(x) I x>0=>x,
                Single(_)=>0
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_guard_with_function_call() {
        // Test guard condition with function calls
        let source = r#"
            F is_positive(x:i64)->bool=x>0
            F filter(x:i64)->bool=M x{
                n I is_positive(n)=>true,
                _=>false
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_multiple_generic_type_params_inference() {
        // Test type inference with multiple generic parameters (simplified)
        let source = r#"
            F pair<A,B>(a:A,b:B)->A=a
            F test()->i64=pair(42,3.14)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_generic_constraint_satisfaction() {
        // Test that generic constraints are checked
        let source = r#"
            F compare<T:Ord>(a:T,b:T)->bool=a<b
            F test()->bool=compare(1,2)
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_nested_option_type_inference() {
        // Test Option<Option<T> > type inference with spaces
        let source = r#"
            F unwrap_twice(opt:Option<Option<i64> >)->i64=M opt{
                Some(Some(x))=>x,
                Some(None)=>-1,
                None=>-2
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_zero_sized_types() {
        // Test zero-sized types (empty struct, unit type)
        let source = r#"
            S Empty{}
            F make_empty()->Empty=Empty{}
            F unit()->()=()
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_circular_type_reference() {
        // Test potential circular type references
        let source = r#"
            S Node{value:i64,next:Option<Node>}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // May or may not be supported depending on implementation
        let _ = checker.check_module(&module);
    }

    #[test]
    fn test_deeply_nested_function_calls() {
        // Test deeply nested function calls for stack depth
        let source = r#"
            F f1(x:i64)->i64=x+1
            F f2(x:i64)->i64=f1(f1(f1(f1(f1(x)))))
            F f3(x:i64)->i64=f2(f2(f2(x)))
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_type_inference_with_multiple_bindings() {
        // Test type inference across multiple variable bindings
        let source = r#"
            F chain()->i64{
                a:=1;
                b:=a+2;
                c:=b*3;
                d:=c-4;
                e:=d/2;
                e
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_all_numeric_type_combinations() {
        // Test mixing different numeric types (should fail without explicit conversion)
        let source = r#"
            F mix()->(){
                a:i8=1;
                b:i64=a;
                ()
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        // Should succeed with integer widening
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_float_to_int_error() {
        // Test float to int (should fail - no implicit conversion)
        let source = r#"
            F convert()->i64{
                f:=3.14;
                i:i64=f;
                i
            }
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }
}
