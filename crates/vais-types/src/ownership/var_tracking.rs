//! Variable registration and reference tracking

use crate::types::{ResolvedType, TypeError, TypeResult};
use super::{OwnershipChecker, OwnershipInfo, OwnershipState, ReferenceInfo};
use vais_ast::{Expr, Span, Spanned};

impl OwnershipChecker {
    /// Register a new variable with its ownership info
    pub fn define_var(
        &mut self,
        name: &str,
        ty: ResolvedType,
        is_mut: bool,
        defined_at: Option<Span>,
    ) {
        let is_copy = Self::is_copy_type(&ty);
        let info = OwnershipInfo {
            state: OwnershipState::Owned,
            ty,
            is_mut,
            is_copy,
            defined_in_scope: self.current_scope,
            defined_at,
        };
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), info);
        }
    }

    /// Register that a variable holds a reference to another variable
    pub fn register_reference(&mut self, ref_var: &str, source_var: &str, is_mut: bool) {
        let (source_scope, source_defined_at) = if let Some(info) = self.lookup_var(source_var) {
            (info.defined_in_scope, info.defined_at)
        } else {
            return; // Source not tracked
        };
        self.reference_sources.insert(
            ref_var.to_string(),
            ReferenceInfo {
                source_var: source_var.to_string(),
                source_scope_depth: source_scope,
                source_defined_at,
                is_mut,
            },
        );
    }

    /// Check if returning a reference expression would create a dangling pointer
    pub fn check_return_ref(
        &mut self,
        expr: &Spanned<Expr>,
        return_at: Option<Span>,
    ) -> TypeResult<()> {
        if let Expr::Ref(inner) | Expr::Deref(inner) = &expr.node {
            if let Expr::Ident(name) = &inner.node {
                return self.check_return_local_ref(name, return_at);
            }
        }
        if let Expr::Ident(name) = &expr.node {
            // Check if this ident is a reference-tracked variable pointing to a local
            if let Some(ref_info) = self.reference_sources.get(name).cloned() {
                // If the source is in the function scope (depth > 0), it's a dangling ref
                if ref_info.source_scope_depth > 0 {
                    let err = TypeError::ReturnLocalRef {
                        var_name: ref_info.source_var,
                        return_at,
                        defined_at: ref_info.source_defined_at,
                    };
                    return self.report_error(err);
                }
            }
        }
        Ok(())
    }

    /// Check if returning a reference to a local variable
    pub(super) fn check_return_local_ref(
        &mut self,
        var_name: &str,
        return_at: Option<Span>,
    ) -> TypeResult<()> {
        if let Some(info) = self.lookup_var(var_name) {
            // If the variable is defined in the function scope (not a parameter at scope 0)
            // and is not 'static, it's a dangling reference
            if info.defined_in_scope > 0 {
                let err = TypeError::ReturnLocalRef {
                    var_name: var_name.to_string(),
                    return_at,
                    defined_at: info.defined_at,
                };
                return self.report_error(err);
            }
        }
        Ok(())
    }
}
