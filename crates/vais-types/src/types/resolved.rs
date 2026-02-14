//! Resolved types and const expressions

use super::effects::EffectSet;

/// Resolved const value for const generics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedConst {
    /// Concrete integer value
    Value(i64),
    /// Unresolved const parameter
    Param(String),
    /// Binary operation (for type display/error messages)
    BinOp {
        op: ConstBinOp,
        left: Box<ResolvedConst>,
        right: Box<ResolvedConst>,
    },
    /// Unary negation
    Negate(Box<ResolvedConst>),
}

impl ResolvedConst {
    /// Try to evaluate to a concrete value
    pub fn try_evaluate(&self) -> Option<i64> {
        match self {
            ResolvedConst::Value(n) => Some(*n),
            ResolvedConst::Param(_) => None,
            ResolvedConst::BinOp { op, left, right } => {
                let l = left.try_evaluate()?;
                let r = right.try_evaluate()?;
                Some(match op {
                    ConstBinOp::Add => l.checked_add(r)?,
                    ConstBinOp::Sub => l.checked_sub(r)?,
                    ConstBinOp::Mul => l.checked_mul(r)?,
                    ConstBinOp::Div => {
                        if r == 0 {
                            return None;
                        }
                        l.checked_div(r)?
                    }
                    ConstBinOp::Mod => {
                        if r == 0 {
                            return None;
                        }
                        l.checked_rem(r)?
                    }
                    ConstBinOp::BitAnd => l & r,
                    ConstBinOp::BitOr => l | r,
                    ConstBinOp::BitXor => l ^ r,
                    ConstBinOp::Shl => {
                        if r < 0 || r >= 64 {
                            return None;
                        }
                        l.wrapping_shl(r as u32)
                    }
                    ConstBinOp::Shr => {
                        if r < 0 || r >= 64 {
                            return None;
                        }
                        l.wrapping_shr(r as u32)
                    }
                })
            }
            ResolvedConst::Negate(inner) => inner.try_evaluate().and_then(|v| v.checked_neg()),
        }
    }
}

impl std::fmt::Display for ResolvedConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedConst::Value(n) => write!(f, "{}", n),
            ResolvedConst::Param(name) => write!(f, "{}", name),
            ResolvedConst::BinOp { op, left, right } => write!(f, "({} {} {})", left, op, right),
            ResolvedConst::Negate(inner) => write!(f, "(-{})", inner),
        }
    }
}

/// Const binary operation for const expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

impl std::fmt::Display for ConstBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstBinOp::Add => write!(f, "+"),
            ConstBinOp::Sub => write!(f, "-"),
            ConstBinOp::Mul => write!(f, "*"),
            ConstBinOp::Div => write!(f, "/"),
            ConstBinOp::Mod => write!(f, "%"),
            ConstBinOp::BitAnd => write!(f, "&"),
            ConstBinOp::BitOr => write!(f, "|"),
            ConstBinOp::BitXor => write!(f, "^"),
            ConstBinOp::Shl => write!(f, "<<"),
            ConstBinOp::Shr => write!(f, ">>"),
        }
    }
}

/// Resolved type in the type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedType {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Str,
    Unit,

    // Compound types
    Array(Box<ResolvedType>),
    /// Const-sized array: `[T; N]` where N is a const expression
    ConstArray {
        element: Box<ResolvedType>,
        size: ResolvedConst,
    },
    Map(Box<ResolvedType>, Box<ResolvedType>),
    Tuple(Vec<ResolvedType>),
    Optional(Box<ResolvedType>),
    Result(Box<ResolvedType>, Box<ResolvedType>),
    Pointer(Box<ResolvedType>),
    Ref(Box<ResolvedType>),
    RefMut(Box<ResolvedType>),
    /// Immutable slice: `&[T]` — fat pointer (ptr, len)
    Slice(Box<ResolvedType>),
    /// Mutable slice: `&mut [T]` — fat pointer (ptr, len)
    SliceMut(Box<ResolvedType>),
    Range(Box<ResolvedType>),
    Future(Box<ResolvedType>),

    // Function type (with optional effect annotation)
    Fn {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
        /// Effect set for this function type (None = infer)
        effects: Option<Box<EffectSet>>,
    },

    // Function pointer type (for C FFI callbacks)
    FnPtr {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
        is_vararg: bool,
        /// Effect set for this function pointer (None = total effects)
        effects: Option<Box<EffectSet>>,
    },

    // Named type (struct/enum)
    Named {
        name: String,
        generics: Vec<ResolvedType>,
    },

    // Type variable for inference
    Var(usize),

    // Generic type parameter (e.g., T in F foo<T>)
    Generic(String),

    // Const generic parameter (e.g., N in F foo<const N: u64>)
    ConstGeneric(String),

    // Unknown/Error type
    Unknown,

    // Never type - represents a type that never returns (e.g., return, break, continue)
    // This type unifies with any other type
    Never,

    // SIMD vector type: <lanes x element_type>
    // e.g., Vector { element: F32, lanes: 4 } -> <4 x float>
    Vector {
        element: Box<ResolvedType>,
        lanes: u32,
    },

    /// Dynamic trait object: `dyn Trait` or `dyn Trait<T>`
    /// Stored as a fat pointer: (vtable*, data*)
    /// Used for runtime polymorphism via vtable-based dispatch.
    DynTrait {
        trait_name: String,
        generics: Vec<ResolvedType>,
    },

    /// Associated type: `<T as Trait>::Item` or unresolved `Self::Item`
    /// GAT support: `<T as Trait>::Item<'a, i64>` with generic arguments
    /// After resolution, this becomes the concrete type
    Associated {
        /// Base type (T in <T as Trait>::Item)
        base: Box<ResolvedType>,
        /// Trait name (None if using Self::Item syntax)
        trait_name: Option<String>,
        /// Associated type name (Item)
        assoc_name: String,
        /// GAT generic arguments (e.g., ['a, i64] in Self::Item<'a, i64>)
        generics: Vec<ResolvedType>,
    },

    /// Linear type wrapper - must be used exactly once
    Linear(Box<ResolvedType>),

    /// Affine type wrapper - can be used at most once
    Affine(Box<ResolvedType>),

    /// Dependent type (Refinement type): a type refined by a predicate
    /// Example: `{n: i64 | n > 0}` represents positive integers
    /// The predicate is stored as a string representation for display/comparison
    Dependent {
        /// The bound variable name
        var_name: String,
        /// The base type being refined
        base: Box<ResolvedType>,
        /// The predicate expression (stored as string for comparison)
        predicate: String,
    },

    /// Reference with explicit lifetime: `&'a T`
    RefLifetime {
        lifetime: String,
        inner: Box<ResolvedType>,
    },

    /// Mutable reference with explicit lifetime: `&'a mut T`
    RefMutLifetime {
        lifetime: String,
        inner: Box<ResolvedType>,
    },

    /// Lifetime parameter (e.g., 'a, 'static)
    Lifetime(String),

    /// Lazy type: `Lazy<T>` - Deferred evaluation thunk
    /// Contains a closure that computes T when forced, and caches the result.
    Lazy(Box<ResolvedType>),

    /// Existential type: `impl Trait` (Vais: `X Trait`)
    /// Represents an opaque return type implementing the given trait bounds.
    /// During monomorphization, resolved to the concrete return type.
    ImplTrait { bounds: Vec<String> },
}

impl ResolvedType {
    pub fn is_numeric(&self) -> bool {
        match self {
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128
            | ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128
            | ResolvedType::F32
            | ResolvedType::F64
            | ResolvedType::Generic(_) // Generics are assumed to support numeric ops
            | ResolvedType::Var(_) // Type variables might resolve to numeric
            | ResolvedType::Unknown => true, // Unknown might be numeric
            // Wrapper types delegate to inner type
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.is_numeric(),
            ResolvedType::Dependent { base, .. } => base.is_numeric(),
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128
            | ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128 => true,
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.is_integer(),
            ResolvedType::Dependent { base, .. } => base.is_integer(),
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            ResolvedType::F32 | ResolvedType::F64 => true,
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.is_float(),
            ResolvedType::Dependent { base, .. } => base.is_float(),
            _ => false,
        }
    }

    /// Get the base type, unwrapping any refinement wrappers (Linear, Affine, Dependent)
    pub fn base_type(&self) -> &ResolvedType {
        match self {
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.base_type(),
            ResolvedType::Dependent { base, .. } => base.base_type(),
            _ => self,
        }
    }
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::I8 => write!(f, "i8"),
            ResolvedType::I16 => write!(f, "i16"),
            ResolvedType::I32 => write!(f, "i32"),
            ResolvedType::I64 => write!(f, "i64"),
            ResolvedType::I128 => write!(f, "i128"),
            ResolvedType::U8 => write!(f, "u8"),
            ResolvedType::U16 => write!(f, "u16"),
            ResolvedType::U32 => write!(f, "u32"),
            ResolvedType::U64 => write!(f, "u64"),
            ResolvedType::U128 => write!(f, "u128"),
            ResolvedType::F32 => write!(f, "f32"),
            ResolvedType::F64 => write!(f, "f64"),
            ResolvedType::Bool => write!(f, "bool"),
            ResolvedType::Str => write!(f, "str"),
            ResolvedType::Unit => write!(f, "()"),
            ResolvedType::Array(t) => write!(f, "[{}]", t),
            ResolvedType::ConstArray { element, size } => write!(f, "[{}; {}]", element, size),
            ResolvedType::Map(k, v) => write!(f, "[{}:{}]", k, v),
            ResolvedType::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            ResolvedType::Optional(t) => write!(f, "{}?", t),
            ResolvedType::Result(t, e) => write!(f, "Result<{}, {}>", t, e),
            ResolvedType::Pointer(t) => write!(f, "*{}", t),
            ResolvedType::Ref(t) => write!(f, "&{}", t),
            ResolvedType::RefMut(t) => write!(f, "&mut {}", t),
            ResolvedType::Slice(t) => write!(f, "&[{}]", t),
            ResolvedType::SliceMut(t) => write!(f, "&mut [{}]", t),
            ResolvedType::Range(t) => write!(f, "Range<{}>", t),
            ResolvedType::Future(t) => write!(f, "Future<{}>", t),
            ResolvedType::Fn {
                params,
                ret,
                effects,
            } => {
                if let Some(effects) = effects {
                    if !effects.is_pure() {
                        write!(f, "{} ", effects)?;
                    }
                }
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::FnPtr {
                params,
                ret,
                is_vararg,
                effects,
            } => {
                if let Some(effects) = effects {
                    if !effects.is_pure() {
                        write!(f, "{} ", effects)?;
                    }
                }
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                if *is_vararg {
                    if !params.is_empty() {
                        write!(f, ",")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::Named { name, generics } => {
                write!(f, "{}", name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Var(id) => write!(f, "?{}", id),
            ResolvedType::Generic(name) => write!(f, "{}", name),
            ResolvedType::ConstGeneric(name) => write!(f, "const {}", name),
            ResolvedType::Unknown => write!(f, "?"),
            ResolvedType::Never => write!(f, "!"),
            ResolvedType::Vector { element, lanes } => write!(f, "Vec{}x{}", lanes, element),
            ResolvedType::DynTrait {
                trait_name,
                generics,
            } => {
                write!(f, "dyn {}", trait_name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => {
                if let Some(trait_name) = trait_name {
                    write!(f, "<{} as {}>::{}", base, trait_name, assoc_name)?;
                } else {
                    write!(f, "{}::{}", base, assoc_name)?;
                }
                // Display GAT parameters if present
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Linear(inner) => write!(f, "linear {}", inner),
            ResolvedType::Affine(inner) => write!(f, "affine {}", inner),
            ResolvedType::Dependent {
                var_name,
                base,
                predicate,
            } => {
                write!(f, "{{{}: {} | {}}}", var_name, base, predicate)
            }
            ResolvedType::RefLifetime { lifetime, inner } => {
                write!(f, "&'{} {}", lifetime, inner)
            }
            ResolvedType::RefMutLifetime { lifetime, inner } => {
                write!(f, "&'{} mut {}", lifetime, inner)
            }
            ResolvedType::Lifetime(name) => write!(f, "'{}", name),
            ResolvedType::Lazy(inner) => write!(f, "Lazy<{}>", inner),
            ResolvedType::ImplTrait { bounds } => {
                write!(f, "impl {}", bounds.join(" + "))
            }
        }
    }
}
