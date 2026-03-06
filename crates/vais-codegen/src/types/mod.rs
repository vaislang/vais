//! Type definitions and type conversion utilities for Vais code generator
//!
//! # Submodules
//!
//! - `conversion`: CodeGenerator methods for type-to-LLVM conversion
//! - `tests`: Unit tests for type mapping and sizeof/alignof

mod conversion;

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

#[derive(Debug, Clone)]
pub(crate) struct LoopLabels {
    pub continue_label: String,
    pub break_label: String,
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
}

#[derive(Debug, Clone)]
pub(crate) struct EnumInfo {
    pub name: String,
    pub variants: Vec<EnumVariantInfo>,
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
}

impl LocalVar {
    /// Create a new parameter variable (SSA value, no alloca)
    pub fn param(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Param,
            llvm_name: llvm_name.into(),
        }
    }

    /// Create a new SSA variable (immutable simple binding, no alloca)
    pub fn ssa(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Ssa,
            llvm_name: llvm_name.into(),
        }
    }

    /// Create a new alloca variable (stack-allocated)
    pub fn alloca(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Alloca,
            llvm_name: llvm_name.into(),
        }
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

/// Information about a lazy thunk (deferred evaluation)
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for lazy evaluation codegen
pub(crate) struct LazyThunkInfo {
    /// The generated LLVM thunk function name
    pub thunk_name: String,
    /// Captured variable names, LLVM types, and loaded values (var_name, llvm_type, llvm_value)
    pub captures: Vec<(String, String, String)>,
    /// LLVM type of the inner (computed) value
    pub inner_llvm_ty: String,
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
