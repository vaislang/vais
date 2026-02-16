//! Literal and identifier expression checking

use vais_ast::*;
use crate::TypeChecker;
use crate::types::{ResolvedType, TypeResult};

impl TypeChecker {
    /// Check literal expressions and identifiers
    pub(crate) fn check_literal_or_ident(&mut self, expr: &Spanned<Expr>) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Int(_) => Some(Ok(ResolvedType::I64)),
            Expr::Float(_) => Some(Ok(ResolvedType::F64)),
            Expr::Bool(_) => Some(Ok(ResolvedType::Bool)),
            Expr::String(_) => Some(Ok(ResolvedType::Str)),
            Expr::StringInterp(parts) => {
                // Type-check each interpolated expression
                for part in parts {
                    if let StringInterpPart::Expr(expr) = part {
                        if let Err(e) = self.check_expr(expr) {
                            return Some(Err(e));
                        }
                    }
                }
                Some(Ok(ResolvedType::Str))
            }
            Expr::Unit => Some(Ok(ResolvedType::Unit)),
            Expr::Ident(name) => {
                // Mark variable as used for linear type tracking
                self.mark_var_used(name);
                Some(self.lookup_var_or_err(name))
            }
            _ => None,
        }
    }
}
