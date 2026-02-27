//! Vais Abstract Syntax Tree
//!
//! AI-optimized AST with minimal node types for efficient parsing and code generation.

pub mod aggregate_types;
pub mod aliases;
pub mod ast_types;
pub mod constants;
pub mod expressions;
pub mod extern_block;
pub mod formatter;
pub mod function;
pub mod generics;
pub mod infrastructure;
pub mod items;
pub mod macros;
pub mod operators;
pub mod patterns;
pub mod statements;
pub mod traits;

// Re-export infrastructure types
pub use infrastructure::{Attribute, Span, Spanned};

// Re-export operator types
pub use operators::{BinOp, UnaryOp};

// Re-export constant types
pub use constants::{ConstBinOp, ConstDef, ConstExpr, GlobalDef};

// Re-export type expression
pub use ast_types::Type;

// Re-export pattern types
pub use patterns::{CaptureMode, Literal, Pattern};

// Re-export macro types
pub use macros::{
    Delimiter, MacroDef, MacroInvoke, MacroLiteral, MacroPattern, MacroPatternElement, MacroRule,
    MacroTemplate, MacroTemplateElement, MacroToken, MetaVarKind, RepetitionKind,
};

// Re-export expression types
pub use expressions::{Expr, IfElse, MatchArm, StringInterpPart};

// Re-export statement types
pub use statements::Stmt;

// Re-export function types
pub use function::{CallArgs, Function, FunctionBody, NamedArg, Ownership, Param};

// Re-export generic types
pub use generics::{GenericParam, GenericParamKind, Variance, WherePredicate};

// Re-export aggregate types
pub use aggregate_types::{Enum, Field, Struct, Union, Variant, VariantFields};

// Re-export alias types
pub use aliases::{AssociatedType, TraitAlias, TypeAlias};

// Re-export trait types
pub use traits::{AssociatedTypeImpl, Impl, Trait, TraitMethod};

// Re-export item types
pub use items::{Item, Module, Use};

// Re-export extern block types
pub use extern_block::{ExternBlock, ExternFunction};

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named { name, generics } => {
                write!(f, "{}", name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", g.node)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::FnPtr {
                params,
                ret,
                is_vararg,
            } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.node)?;
                }
                if *is_vararg {
                    if !params.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ") -> {}", ret.node)
            }
            Type::Array(inner) => write!(f, "[{}]", inner.node),
            Type::ConstArray { element, size } => write!(f, "[{}; {}]", element.node, size),
            Type::Map(key, val) => write!(f, "[{}:{}]", key.node, val.node),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t.node)?;
                }
                write!(f, ")")
            }
            Type::Optional(inner) => write!(f, "{}?", inner.node),
            Type::Result(inner) => write!(f, "{}!", inner.node),
            Type::Pointer(inner) => write!(f, "*{}", inner.node),
            Type::Ref(inner) => write!(f, "&{}", inner.node),
            Type::RefMut(inner) => write!(f, "&mut {}", inner.node),
            Type::Slice(inner) => write!(f, "&[{}]", inner.node),
            Type::SliceMut(inner) => write!(f, "&mut [{}]", inner.node),
            Type::Fn { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.node)?;
                }
                write!(f, ") -> {}", ret.node)
            }
            Type::Unit => write!(f, "()"),
            Type::Infer => write!(f, "_"),
            Type::DynTrait {
                trait_name,
                generics,
            } => {
                write!(f, "dyn {}", trait_name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", g.node)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => {
                if let Some(trait_name) = trait_name {
                    write!(f, "<{} as {}>::{}", base.node, trait_name, assoc_name)?;
                } else {
                    write!(f, "{}::{}", base.node, assoc_name)?;
                }
                // Display GAT parameters if present
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", g.node)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Linear(inner) => write!(f, "linear {}", inner.node),
            Type::Affine(inner) => write!(f, "affine {}", inner.node),
            Type::Dependent {
                var_name,
                base,
                predicate,
            } => {
                write!(f, "{{{}: {} | {:?}}}", var_name, base.node, predicate.node)
            }
            Type::RefLifetime { lifetime, inner } => write!(f, "&'{} {}", lifetime, inner.node),
            Type::RefMutLifetime { lifetime, inner } => {
                write!(f, "&'{} mut {}", lifetime, inner.node)
            }
            Type::Lazy(inner) => write!(f, "Lazy<{}>", inner.node),
            Type::ImplTrait { bounds } => {
                let names: Vec<&str> = bounds.iter().map(|b| b.node.as_str()).collect();
                write!(f, "impl {}", names.join(" + "))
            }
        }
    }
}
