//! Type conversion helpers

use super::OwnershipChecker;
use crate::types::ResolvedType;
use vais_ast::*;

impl OwnershipChecker {
    /// Convert AST type to a simplified ResolvedType for ownership tracking
    /// Infer a basic type from an expression for ownership tracking purposes.
    /// This is a lightweight inference - the real type checker has already validated types.
    pub(super) fn infer_type_from_expr(&self, expr: &Spanned<Expr>) -> ResolvedType {
        match &expr.node {
            Expr::Int(_) => ResolvedType::I64,
            Expr::Float(_) => ResolvedType::F64,
            Expr::Bool(_) => ResolvedType::Bool,
            Expr::String(_) => ResolvedType::Str,
            Expr::Ident(name) => {
                // Look up the variable's registered type
                self.lookup_var(name)
                    .map(|info| info.ty.clone())
                    .unwrap_or(ResolvedType::Unknown)
            }
            Expr::Binary { left, .. } => self.infer_type_from_expr(left),
            Expr::Unary { expr: inner, .. } => self.infer_type_from_expr(inner),
            Expr::Ref(inner) => ResolvedType::Ref(Box::new(self.infer_type_from_expr(inner))),
            Expr::Tuple(elems) => {
                ResolvedType::Tuple(elems.iter().map(|e| self.infer_type_from_expr(e)).collect())
            }
            Expr::Array(elems) => {
                let elem_ty = elems
                    .first()
                    .map(|e| self.infer_type_from_expr(e))
                    .unwrap_or(ResolvedType::Unknown);
                ResolvedType::Array(Box::new(elem_ty))
            }
            Expr::Call { .. } | Expr::MethodCall { .. } => {
                // Can't easily determine return type without full type info
                // Conservatively treat as Copy (since the type checker already validated)
                ResolvedType::I64
            }
            _ => ResolvedType::Unknown,
        }
    }

    pub(super) fn ast_type_to_resolved(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, .. } => match name.as_str() {
                "i8" => ResolvedType::I8,
                "i16" => ResolvedType::I16,
                "i32" => ResolvedType::I32,
                "i64" | "int" => ResolvedType::I64,
                "i128" => ResolvedType::I128,
                "u8" => ResolvedType::U8,
                "u16" => ResolvedType::U16,
                "u32" => ResolvedType::U32,
                "u64" => ResolvedType::U64,
                "u128" => ResolvedType::U128,
                "f32" => ResolvedType::F32,
                "f64" | "float" => ResolvedType::F64,
                "bool" => ResolvedType::Bool,
                "str" | "String" => ResolvedType::Str,
                _ => ResolvedType::Named {
                    name: name.clone(),
                    generics: vec![],
                },
            },
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Slice(inner) => {
                ResolvedType::Slice(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::SliceMut(inner) => {
                ResolvedType::SliceMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Array(inner) => {
                ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Tuple(elems) => ResolvedType::Tuple(
                elems
                    .iter()
                    .map(|e| self.ast_type_to_resolved(&e.node))
                    .collect(),
            ),
            Type::Unit => ResolvedType::Unit,
            Type::Infer => ResolvedType::Unknown,
            _ => ResolvedType::Unknown,
        }
    }
}
