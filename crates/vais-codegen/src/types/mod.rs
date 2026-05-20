//! Type definitions and type conversion utilities for Vais code generator
//!
//! # Submodules
//!
//! - `conversion`: CodeGenerator methods for type-to-LLVM conversion
//! - `tests`: Unit tests for type mapping and sizeof/alignof

pub mod coercion;
mod conversion;
pub mod sizeof;
pub mod type_gen;

#[cfg(test)]
mod tests;

use vais_ast::Type;
use vais_types::ResolvedType;

/// Format an f64 value as a valid LLVM IR floating-point constant.
/// LLVM requires format like "1.000000e+00", not Rust's "1.000000e0".
pub(crate) fn format_llvm_float(n: f64) -> String {
    // Use Rust's scientific notation then fix the exponent format
    let s = format!("{:.6e}", n);
    // Rust produces "1.000000e0", LLVM needs "1.000000e+00"
    // Find the 'e' and fix the exponent
    if let Some(e_pos) = s.rfind('e') {
        let (mantissa, exp_part) = s.split_at(e_pos);
        let exp_str = &exp_part[1..]; // skip 'e'
        if let Ok(exp_val) = exp_str.parse::<i32>() {
            if exp_val >= 0 {
                format!("{}e+{:02}", mantissa, exp_val)
            } else {
                format!("{}e-{:02}", mantissa, exp_val.unsigned_abs())
            }
        } else {
            s
        }
    } else {
        s
    }
}

/// Format an f64 value as a valid LLVM IR floating-point constant,
/// choosing between decimal and the IEEE-754 hex bit-pattern form based
/// on the target LLVM type (`float` or `double`). For `float`, LLVM
/// only accepts decimals that round-trip exactly through single
/// precision (e.g. `1.0e-8` fails); emit the double-bit hex form,
/// which LLVM interprets as "round this double to float" and is always
/// accepted.
#[allow(dead_code)]
pub(crate) fn format_llvm_float_typed(n: f64, target_llvm: &str) -> String {
    if target_llvm == "float" && n.is_finite() {
        // Quick check: does the value round-trip through f32 → f64?
        let as_f32 = n as f32;
        let round_tripped = as_f32 as f64;
        if round_tripped != n {
            let bits = n.to_bits();
            return format!("0x{:016X}", bits);
        }
    }
    format_llvm_float(n)
}

#[derive(Debug, Clone)]
pub(crate) struct LoopLabels {
    pub continue_label: String,
    pub break_label: String,
    /// `scope_str_stack.len()` at loop entry. Frames at indices ≥ this depth
    /// are loop-internal and must be freed on break/continue to prevent leaks
    /// of mid-iteration concat/push_str buffers (Phase 191 #6).
    pub scope_str_depth: usize,
    /// Phase 0 bug C1: when `B <value>` (break-with-value) is used, this
    /// holds `(slot_name, llvm_ty)` for an alloca emitted in the function's
    /// entry block. `generate_break_stmt` stores the break value into this
    /// slot before branching. After the loop end label, the loop expression
    /// loads from this slot to yield the break value. None for plain `B`
    /// loops where the loop is used in statement position.
    pub break_value_slot: Option<(String, String)>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub signature: vais_types::FunctionSig,
    pub is_extern: bool,
    pub _extern_abi: Option<String>, // ABI for extern functions (e.g., "C")
}

#[derive(Debug, Clone)]
pub(crate) struct StructInfo {
    pub _name: String,
    pub fields: Vec<(String, ResolvedType)>,
    pub _repr_c: bool, // true if #[repr(C)] attribute is present
    /// Invariant expressions for formal verification
    /// These are checked after struct construction/modification
    pub _invariants: Vec<vais_ast::Spanned<vais_ast::Expr>>,
    /// True when the struct carries heap-owned string fields and codegen must
    /// append a trailing `i64 __ownership_mask` field to the LLVM layout
    /// (Phase 191 #2b, RFC-002 §4.2 Option D). When true, `heap_fields` lists
    /// the user-visible field indices that participate in the bitmap.
    pub has_owned_mask: bool,
    /// Indices into `fields` of heap-owned candidates (Str or Vec$str). Populated
    /// when `has_owned_mask == true`. Consumed by shallow-drop helper emission (#2b-C).
    pub heap_fields: Vec<usize>,
}

impl StructInfo {
    /// Classify a field as heap-owned for ownership-mask purposes.
    ///
    /// Returns true for direct `Str` and for the monomorphized `Vec$str` Named
    /// container. Nested user structs with their own mask are handled by #2c.
    pub fn field_is_heap_owned(ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Str => true,
            ResolvedType::Named { name, generics, .. } => {
                if name == "Vec$str" {
                    return true;
                }
                // Vec<str> before name-mangling: Named("Vec", [Str]).
                if name == "Vec" && generics.len() == 1 {
                    return matches!(generics[0], ResolvedType::Str);
                }
                false
            }
            _ => false,
        }
    }

    /// Compute heap_fields + has_owned_mask from a field list. Keeps the two
    /// registration sites (register_struct + generate_specialized_struct_type)
    /// in sync.
    pub fn derive_ownership_mask(fields: &[(String, ResolvedType)]) -> (bool, Vec<usize>) {
        let heap_fields: Vec<usize> = fields
            .iter()
            .enumerate()
            .filter_map(|(i, (_, ty))| Self::field_is_heap_owned(ty).then_some(i))
            .collect();
        (!heap_fields.is_empty(), heap_fields)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EnumInfo {
    pub name: String,
    pub variants: Vec<EnumVariantInfo>,
    /// True when at least one variant has a direct `str` payload field (in a
    /// tuple or struct variant). When true, codegen appends a trailing
    /// `i64 __ownership_mask` field (field index 2) to the LLVM layout so a
    /// later DB-free step can distinguish heap-owned string payloads from
    /// literal/borrowed ones (Phase 191, RFC-002 section 4.2 Option D for enums).
    /// Non-`str` enums keep their original `{ i32, { i64*N } }` layout.
    pub has_owned_mask: bool,
}

impl EnumInfo {
    /// True if any variant carries at least one direct `str` payload field.
    /// Only direct `str` qualifies; nested containers and generic
    /// `Option<str>`/`Result<str, E>` are out of scope for this slice.
    pub fn derive_ownership_mask(variants: &[EnumVariantInfo]) -> bool {
        variants.iter().any(|v| match &v.fields {
            EnumVariantFields::Unit => false,
            EnumVariantFields::Tuple(types) => types.iter().any(|t| matches!(t, ResolvedType::Str)),
            EnumVariantFields::Struct(fields) => {
                fields.iter().any(|(_, t)| matches!(t, ResolvedType::Str))
            }
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct UnionInfo {
    pub _name: String,
    pub fields: Vec<(String, ResolvedType)>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumVariantInfo {
    pub name: String,
    pub _tag: u32,
    pub fields: EnumVariantFields,
}

#[derive(Debug, Clone)]
pub(crate) enum EnumVariantFields {
    Unit,
    Tuple(Vec<ResolvedType>),
    Struct(Vec<(String, ResolvedType)>),
}

/// Constant definition info
#[derive(Debug, Clone)]
pub(crate) struct ConstInfo {
    pub _name: String,
    pub _ty: ResolvedType,
    pub value: vais_ast::Spanned<vais_ast::Expr>,
}

/// Global variable definition info
#[derive(Debug, Clone)]
pub(crate) struct GlobalInfo {
    pub _name: String,
    pub _ty: ResolvedType,
    pub _value: vais_ast::Spanned<vais_ast::Expr>,
    pub _is_mutable: bool,
}

/// Represents the storage kind of a local variable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalVarKind {
    /// Function parameter (SSA value, no alloca)
    Param,
    /// Immutable simple binding (SSA style, no alloca, direct register)
    Ssa,
    /// Stack-allocated variable (uses alloca)
    Alloca,
}

#[derive(Debug, Clone)]
pub(crate) struct LocalVar {
    pub ty: ResolvedType,
    /// The storage kind of this variable
    pub kind: LocalVarKind,
    /// The actual LLVM IR name for this variable (may differ from source name in loops)
    pub llvm_name: String,
    /// True when the alloca stores a pointer-to-struct (`%Type**`), requiring a
    /// load before passing to drop/shallow-free. False for direct struct allocas
    /// (`%Type*`) where the alloca IS the struct pointer.
    pub is_double_ptr: bool,
    /// For array-literal locals (`x := [a, b, c]`), the compile-time known
    /// element count. Used by `&x` coercion to build a `{ i8*, i64 }` slice
    /// fat pointer when the target parameter expects `&[T]`.
    /// `None` for non-array locals (default).
    pub array_length: Option<u64>,
}

impl LocalVar {
    /// Create a new parameter variable (SSA value, no alloca)
    pub fn param(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Param,
            llvm_name: llvm_name.into(),
            is_double_ptr: false,
            array_length: None,
        }
    }

    /// Create a new SSA variable (immutable simple binding, no alloca)
    pub fn ssa(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Ssa,
            llvm_name: llvm_name.into(),
            is_double_ptr: false,
            array_length: None,
        }
    }

    /// Create a new alloca variable (stack-allocated, single pointer `%Type*`)
    pub fn alloca(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Alloca,
            llvm_name: llvm_name.into(),
            is_double_ptr: false,
            array_length: None,
        }
    }

    /// Create a new alloca variable that stores a pointer to struct (`%Type**`)
    pub fn alloca_double_ptr(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Alloca,
            llvm_name: llvm_name.into(),
            is_double_ptr: true,
            array_length: None,
        }
    }

    /// Tag a LocalVar with a compile-time known array length (e.g., from an
    /// `[a, b, c]` literal init) so `&x` can build a slice fat pointer.
    pub fn with_array_length(mut self, length: u64) -> Self {
        self.array_length = Some(length);
        self
    }

    /// Returns true if this is a function parameter
    #[inline]
    pub fn is_param(&self) -> bool {
        matches!(self.kind, LocalVarKind::Param)
    }

    /// Returns true if this variable uses SSA style (no alloca)
    #[inline]
    pub fn is_ssa(&self) -> bool {
        matches!(self.kind, LocalVarKind::Ssa)
    }

    /// Returns true if this variable uses alloca
    #[inline]
    pub fn is_alloca(&self) -> bool {
        matches!(self.kind, LocalVarKind::Alloca)
    }
}

/// Information about a closure (lambda with captures)
#[derive(Debug, Clone)]
pub(crate) struct ClosureInfo {
    /// The generated LLVM function name for this lambda
    pub func_name: String,
    /// Captured variable names and their loaded values (var_name, llvm_value)
    pub captures: Vec<(String, String)>,
    /// Whether captures are passed by reference (pointer) vs by value
    pub is_ref_capture: bool,
}

/// Information about an await point in an async function.
/// Used to track suspension points in the async state machine.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for async state machine codegen
pub(crate) struct AsyncAwaitPoint {
    /// State index after this await
    pub state_index: usize,
    /// Variable to store the awaited result
    pub result_var: String,
    /// LLVM type of the result
    pub result_type: String,
}

/// Information about the current async function being compiled.
/// Tracks the state machine structure for async function code generation.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for async function codegen
pub(crate) struct AsyncFunctionInfo {
    /// Original function name
    pub name: String,
    /// State struct name for this async function
    pub state_struct: String,
    /// Captured variables that need to be stored in state
    pub captured_vars: Vec<(String, ResolvedType)>,
    /// Return type of the future
    pub ret_type: ResolvedType,
}
