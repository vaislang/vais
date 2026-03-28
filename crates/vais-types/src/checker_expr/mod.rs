//! Expression and statement type checking.

mod async_effects;
mod calls;
mod collections;
mod control_flow;
mod literals;
mod references;
mod special;
mod stmts;

use crate::types::{ResolvedType, TypeResult};
use crate::TypeChecker;
use vais_ast::*;

impl TypeChecker {
    /// Check an expression - main dispatcher
    pub(crate) fn check_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<ResolvedType> {
        let result = self.check_expr_inner(expr);
        // Store the resolved type in expr_types for codegen to reference later
        if let Ok(ref ty) = result {
            let span_key = (expr.span.start, expr.span.end);
            self.expr_types.insert(span_key, ty.clone());
        }
        result
    }

    /// Inner dispatcher for check_expr (without span recording)
    fn check_expr_inner(&mut self, expr: &Spanned<Expr>) -> TypeResult<ResolvedType> {
        // Try each category of expression checkers in order

        // 1. Literals and identifiers
        if let Some(result) = self.check_literal_or_ident(expr) {
            return result;
        }

        // 2. Self call
        if matches!(&expr.node, Expr::SelfCall) {
            return self.check_self_call(expr.span);
        }

        // 3. Calls (Call, MethodCall, StaticMethodCall)
        if let Expr::Call { func, args } = &expr.node {
            return self.check_call_expr(func, args, expr.span);
        }
        if let Expr::MethodCall {
            receiver,
            method,
            args,
        } = &expr.node
        {
            return self.check_method_call(receiver, method, args, expr.span);
        }
        if let Expr::StaticMethodCall {
            type_name,
            method,
            args,
        } = &expr.node
        {
            return self.check_static_method_call(type_name, method, args, expr.span);
        }

        // 4. Control flow
        if let Some(result) = self.check_control_flow(expr) {
            return result;
        }

        // 5. Collections (includes binary/unary ops, field, index, array, tuple, map, struct, range)
        if let Some(result) = self.check_collection_expr(expr) {
            return result;
        }

        // 6. References
        if let Some(result) = self.check_reference_expr(expr) {
            return result;
        }

        // 7. Async effects
        if let Some(result) = self.check_async_expr(expr) {
            return result;
        }

        // 8. Special constructs
        if let Some(result) = self.check_special_expr(expr) {
            return result;
        }

        // 9. Enum namespace access: EnumName::VariantName
        if let Expr::EnumAccess {
            enum_name,
            variant,
            data,
        } = &expr.node
        {
            return self.check_enum_access(enum_name, variant, data.as_deref(), expr.span);
        }

        // If we reach here, we've missed an expression type
        // This shouldn't happen - it's a compiler bug
        Err(crate::types::TypeError::InternalError {
            message: format!("unhandled expression type in check_expr: {:?}", expr.node),
            span: Some(expr.span),
        })
    }

    /// Check `EnumName::VariantName` or `EnumName::VariantName(data)` expressions.
    pub(crate) fn check_enum_access(
        &mut self,
        enum_name: &str,
        variant: &str,
        data: Option<&Spanned<Expr>>,
        span: Span,
    ) -> TypeResult<ResolvedType> {
        use crate::types::{TypeError, VariantFieldTypes};

        let enum_def =
            self.enums
                .get(enum_name)
                .cloned()
                .ok_or_else(|| TypeError::UndefinedType {
                    name: format!(
                        "enum '{}' not found (in '{}::{}')",
                        enum_name, enum_name, variant
                    ),
                    span: Some(span),
                    suggestion: None,
                })?;

        if !enum_def.variants.contains_key(variant) {
            return Err(TypeError::UndefinedVar {
                name: format!("{}::{}", enum_name, variant),
                span: Some(span),
                suggestion: None,
            });
        }

        // If data is provided, type-check it (for tuple variants)
        if let Some(data_expr) = data {
            let _data_ty = self.check_expr(data_expr)?;
        } else {
            // Verify it's a unit variant (no required data)
            if let Some(VariantFieldTypes::Tuple(fields)) = enum_def.variants.get(variant) {
                if !fields.is_empty() {
                    return Err(TypeError::ArgCount {
                        expected: fields.len(),
                        got: 0,
                        span: Some(span),
                    });
                }
            }
        }

        let generics: Vec<ResolvedType> = enum_def
            .generics
            .iter()
            .map(|_| self.fresh_type_var())
            .collect();

        Ok(ResolvedType::Named {
            name: enum_name.to_string(),
            generics,
        })
    }
}
