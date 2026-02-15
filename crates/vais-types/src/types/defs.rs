//! Type definitions for functions, structs, enums, and unions

use std::collections::HashMap;
use vais_ast::Span;

use super::effects::{EffectAnnotation, EffectSet};
use super::resolved::ResolvedType;

/// Contract clause for formal verification (requires/ensures)
#[derive(Debug, Clone)]
pub struct ContractClause {
    /// Original expression string for error messages
    pub expr_str: String,
    /// Source span for error reporting
    pub span: Span,
}

/// Contract specification for Design by Contract
#[derive(Debug, Clone, Default)]
pub struct ContractSpec {
    /// Preconditions (requires clauses)
    pub requires: Vec<ContractClause>,
    /// Postconditions (ensures clauses)
    pub ensures: Vec<ContractClause>,
}

impl ContractSpec {
    /// Check if the contract specification has any clauses
    pub fn is_empty(&self) -> bool {
        self.requires.is_empty() && self.ensures.is_empty()
    }
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub generics: Vec<String>,
    pub generic_bounds: HashMap<String, Vec<String>>, // generic name -> trait bounds
    pub params: Vec<(String, ResolvedType, bool)>,    // (name, type, is_mut)
    pub ret: ResolvedType,
    pub is_async: bool,
    pub is_vararg: bool, // true for variadic C functions (printf, etc.)
    /// Number of required parameters (those without default values)
    /// If None, all parameters are required (backward compatible)
    pub required_params: Option<usize>,
    /// Contract specification for formal verification (requires/ensures)
    pub contracts: Option<ContractSpec>,
    /// Effect annotation - declared or inferred effects
    pub effect_annotation: EffectAnnotation,
    /// Inferred effects (populated during type checking)
    pub inferred_effects: Option<EffectSet>,
    /// Higher-kinded type parameters: maps HKT param name to expected arity
    pub hkt_params: HashMap<String, usize>,
}

impl Default for FunctionSig {
    fn default() -> Self {
        Self {
            name: String::new(),
            generics: vec![],
            generic_bounds: HashMap::new(),
            params: vec![],
            ret: ResolvedType::Unit,
            is_async: false,
            is_vararg: false,
            required_params: None,
            contracts: None,
            effect_annotation: EffectAnnotation::Infer,
            inferred_effects: None,
            hkt_params: HashMap::new(),
        }
    }
}

impl FunctionSig {
    /// Return the minimum number of required arguments
    pub fn min_args(&self) -> usize {
        self.required_params.unwrap_or(self.params.len())
    }
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
    pub field_order: Vec<String>, // Preserves declaration order for tuple literal syntax
    pub methods: HashMap<String, FunctionSig>,
    pub repr_c: bool, // true if #[repr(C)] attribute is present
}

/// Enum variant field types
#[derive(Debug, Clone)]
pub enum VariantFieldTypes {
    /// Unit variant (no fields)
    Unit,
    /// Tuple variant with positional fields
    Tuple(Vec<ResolvedType>),
    /// Struct variant with named fields
    Struct(HashMap<String, ResolvedType>),
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: HashMap<String, VariantFieldTypes>,
    pub methods: HashMap<String, FunctionSig>,
}

/// Union definition (untagged, C-style)
/// All fields share the same memory location (offset 0).
/// No runtime tag - caller is responsible for knowing which field is active.
#[derive(Debug, Clone)]
pub struct UnionDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
}

/// Linearity mode for linear type system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Linearity {
    /// Unrestricted (can be used any number of times)
    #[default]
    Unrestricted,
    /// Linear (must be used exactly once)
    Linear,
    /// Affine (can be used at most once - dropped without use is OK)
    Affine,
}

impl Linearity {
    /// Check if this linearity requires tracking
    pub fn requires_tracking(&self) -> bool {
        matches!(self, Linearity::Linear | Linearity::Affine)
    }

    /// Check if this linearity allows dropping without use
    pub fn allows_drop_without_use(&self) -> bool {
        matches!(self, Linearity::Unrestricted | Linearity::Affine)
    }

    /// Check if a use count is valid for this linearity
    pub fn is_valid_use_count(&self, count: usize) -> bool {
        match self {
            Linearity::Unrestricted => true,
            Linearity::Linear => count == 1,
            Linearity::Affine => count <= 1,
        }
    }
}

/// Variable info (internal to type checker).
/// Reserved for linear/affine type tracking.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct VarInfo {
    pub(crate) ty: ResolvedType,
    pub(crate) is_mut: bool,
    pub(crate) linearity: Linearity,
    pub(crate) use_count: usize,
    /// Span where the variable was defined (for error messages)
    pub(crate) defined_at: Option<Span>,
}

/// Generic instantiation tracking for monomorphization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericInstantiation {
    /// Base name of the generic item (function or struct)
    pub base_name: String,
    /// Concrete type arguments
    pub type_args: Vec<ResolvedType>,
    /// Concrete const arguments (name -> value)
    pub const_args: Vec<(String, i64)>,
    /// Mangled name for code generation
    pub mangled_name: String,
    /// Kind of instantiation
    pub kind: InstantiationKind,
}

/// Kind of generic instantiation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstantiationKind {
    Function,
    Struct,
    Method { struct_name: String },
}

impl GenericInstantiation {
    /// Create a new function instantiation
    pub fn function(base_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let mangled = super::mangle::mangle_name(base_name, &type_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args: Vec::new(),
            mangled_name: mangled,
            kind: InstantiationKind::Function,
        }
    }

    /// Create a new function instantiation with const generic arguments
    pub fn function_with_consts(
        base_name: &str,
        type_args: Vec<ResolvedType>,
        const_args: Vec<(String, i64)>,
    ) -> Self {
        let mangled = super::mangle::mangle_name_with_consts(base_name, &type_args, &const_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args,
            mangled_name: mangled,
            kind: InstantiationKind::Function,
        }
    }

    /// Create a new struct instantiation
    pub fn struct_type(base_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let mangled = super::mangle::mangle_name(base_name, &type_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args: Vec::new(),
            mangled_name: mangled,
            kind: InstantiationKind::Struct,
        }
    }

    /// Create a new struct instantiation with const generic arguments
    pub fn struct_type_with_consts(
        base_name: &str,
        type_args: Vec<ResolvedType>,
        const_args: Vec<(String, i64)>,
    ) -> Self {
        let mangled = super::mangle::mangle_name_with_consts(base_name, &type_args, &const_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args,
            mangled_name: mangled,
            kind: InstantiationKind::Struct,
        }
    }

    /// Create a new method instantiation
    pub fn method(struct_name: &str, method_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let base = format!("{}_{}", struct_name, method_name);
        let mangled = super::mangle::mangle_name(&base, &type_args);
        Self {
            base_name: method_name.to_string(),
            type_args,
            const_args: Vec::new(),
            mangled_name: mangled,
            kind: InstantiationKind::Method {
                struct_name: struct_name.to_string(),
            },
        }
    }
}
