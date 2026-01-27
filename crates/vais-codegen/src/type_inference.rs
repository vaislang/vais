//! Type inference utilities for Vais code generator
//!
//! This module contains functions for inferring types of expressions
//! and blocks during code generation.

use crate::CodeGenerator;
use vais_ast::{Expr, Spanned, Stmt};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Infer type of a statement block (for if-else phi nodes)
    pub(crate) fn infer_block_type(&self, stmts: &[Spanned<Stmt>]) -> ResolvedType {
        // Look at the last statement to determine block type
        if let Some(last_stmt) = stmts.last() {
            match &last_stmt.node {
                Stmt::Expr(expr) => self.infer_expr_type(expr),
                Stmt::Return(Some(expr)) => self.infer_expr_type(expr),
                _ => ResolvedType::I64,
            }
        } else {
            ResolvedType::I64
        }
    }

    /// Check if block's last expression is a value (not a pointer to struct)
    /// Returns true if the value from generate_block_stmts is already a value,
    /// false if it's a pointer that needs to be loaded
    pub(crate) fn is_block_result_value(&self, stmts: &[Spanned<Stmt>]) -> bool {
        if let Some(last_stmt) = stmts.last() {
            match &last_stmt.node {
                Stmt::Expr(expr) => self.is_expr_value(expr),
                Stmt::Return(Some(expr)) => self.is_expr_value(expr),
                _ => true,
            }
        } else {
            true
        }
    }

    /// Check if an expression produces a value (not a pointer)
    /// struct literals produce pointers, if-else/match/call produce values
    /// enum variant constructors (Some(x), None) produce pointers
    pub(crate) fn is_expr_value(&self, expr: &Spanned<Expr>) -> bool {
        match &expr.node {
            Expr::StructLit { .. } => false, // struct literal is a pointer
            Expr::If { .. } => true,         // if-else produces a value (phi node)
            Expr::Match { .. } => true,      // match produces a value
            // Check if Call is an enum variant constructor
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    // Enum variant constructors (e.g., Some(x)) return pointers
                    if self.get_tuple_variant_info(name).is_some() {
                        return false;
                    }
                }
                true // regular function call produces a value
            }
            Expr::MethodCall { .. } => true,       // method call produces a value
            Expr::StaticMethodCall { .. } => true, // static method call produces a value
            // Struct-typed local variables are stored as pointers (double-pointer)
            // so generate_expr returns a pointer, not a value
            Expr::Ident(name) => {
                // Unit enum variants (e.g., None) produce pointers
                if self.is_unit_enum_variant(name) {
                    return false;
                }
                if let Some(local) = self.locals.get(name) {
                    // Parameters are passed by value (even struct types)
                    // so they produce values directly
                    if local.is_param() {
                        return true;
                    }
                    // Struct/enum local variables are stored as double-pointers (%Struct** %var)
                    // so generate_expr returns a pointer, not a value
                    !matches!(local.ty, ResolvedType::Named { .. })
                } else {
                    true
                }
            }
            _ => true,
        }
    }

    /// Infer type of expression (simple version for let statement)
    pub(crate) fn infer_expr_type(&self, expr: &Spanned<Expr>) -> ResolvedType {
        match &expr.node {
            Expr::Int(_) => ResolvedType::I64,
            Expr::Float(_) => ResolvedType::F64,
            Expr::Bool(_) => ResolvedType::Bool,
            Expr::String(_) => ResolvedType::Str,
            // @ refers to self in methods
            Expr::SelfCall => {
                if let Some(local) = self.locals.get("self") {
                    local.ty.clone()
                } else {
                    ResolvedType::I64
                }
            }
            Expr::Ident(name) => {
                // Look up local variable type
                if let Some(local) = self.locals.get(name) {
                    local.ty.clone()
                } else if self.is_unit_enum_variant(name) {
                    // Unit enum variant (e.g., None)
                    for enum_info in self.enums.values() {
                        for variant in &enum_info.variants {
                            if variant.name == *name {
                                return ResolvedType::Named {
                                    name: enum_info.name.clone(),
                                    generics: vec![],
                                };
                            }
                        }
                    }
                    ResolvedType::I64
                } else {
                    ResolvedType::I64 // Default
                }
            }
            Expr::Call { func, .. } => {
                // Get return type from function info
                if let Expr::Ident(fn_name) = &func.node {
                    // Check if this is an enum variant constructor
                    if let Some((enum_name, _)) = self.get_tuple_variant_info(fn_name) {
                        return ResolvedType::Named {
                            name: enum_name,
                            generics: vec![],
                        };
                    }
                    // Check function info
                    if let Some(fn_info) = self.functions.get(fn_name) {
                        let ret_ty = fn_info.signature.ret.clone();
                        // Convert i32 returns to i64 since codegen promotes them
                        if ret_ty == ResolvedType::I32 {
                            return ResolvedType::I64;
                        }
                        return ret_ty;
                    }
                }
                ResolvedType::I64 // Default
            }
            Expr::Array(elements) => {
                if let Some(first) = elements.first() {
                    ResolvedType::Pointer(Box::new(self.infer_expr_type(first)))
                } else {
                    ResolvedType::Pointer(Box::new(ResolvedType::I64))
                }
            }
            Expr::Tuple(elements) => {
                ResolvedType::Tuple(elements.iter().map(|e| self.infer_expr_type(e)).collect())
            }
            Expr::Ref(inner) => ResolvedType::Ref(Box::new(self.infer_expr_type(inner))),
            Expr::Deref(inner) => match self.infer_expr_type(inner) {
                ResolvedType::Pointer(inner) => *inner,
                ResolvedType::Ref(inner) => *inner,
                ResolvedType::RefMut(inner) => *inner,
                _ => ResolvedType::I64,
            },
            Expr::StructLit { name, .. } => {
                // Return Named type for struct literals
                ResolvedType::Named {
                    name: name.node.clone(),
                    generics: vec![],
                }
            }
            Expr::Index { expr: inner, index } => {
                // Check if this is a slice operation
                if matches!(index.node, Expr::Range { .. }) {
                    // Slice returns a pointer
                    let inner_ty = self.infer_expr_type(inner);
                    match inner_ty {
                        ResolvedType::Pointer(elem) => ResolvedType::Pointer(elem),
                        ResolvedType::Array(elem) => ResolvedType::Pointer(elem),
                        _ => ResolvedType::Pointer(Box::new(ResolvedType::I64)),
                    }
                } else {
                    // Regular indexing returns element type
                    let inner_ty = self.infer_expr_type(inner);
                    match inner_ty {
                        ResolvedType::Pointer(elem) => *elem,
                        ResolvedType::Array(elem) => *elem,
                        _ => ResolvedType::I64,
                    }
                }
            }
            Expr::Lambda { .. } => {
                // Lambda returns a function pointer (represented as i64)
                ResolvedType::I64
            }
            Expr::MethodCall {
                receiver, method, ..
            } => {
                // Get method return type from struct definition
                let recv_type = self.infer_expr_type(receiver);
                if let ResolvedType::Named { name, .. } = &recv_type {
                    let method_name = format!("{}_{}", name, method.node);
                    if let Some(fn_info) = self.functions.get(&method_name) {
                        return fn_info.signature.ret.clone();
                    }
                }
                ResolvedType::I64
            }
            Expr::StaticMethodCall {
                type_name, method, ..
            } => {
                // Get static method return type from function info
                let method_name = format!("{}_{}", type_name.node, method.node);
                if let Some(fn_info) = self.functions.get(&method_name) {
                    return fn_info.signature.ret.clone();
                }
                ResolvedType::I64
            }
            Expr::Comptime { body } => {
                // Infer type from the comptime expression result
                // For now, we'll evaluate it and determine the type
                self.infer_expr_type(body)
            }
            Expr::MacroInvoke(_) => {
                // Macro invocations should be expanded before type inference
                ResolvedType::Unknown
            }
            Expr::Old(inner) => {
                // old(expr) has the same type as expr
                self.infer_expr_type(inner)
            }
            Expr::Assert { .. } | Expr::Assume(_) => {
                // assert and assume return unit
                ResolvedType::Unit
            }
            Expr::Lazy(inner) => {
                // lazy expr returns Lazy<T> where T is the type of expr
                let inner_type = self.infer_expr_type(inner);
                ResolvedType::Lazy(Box::new(inner_type))
            }
            Expr::Force(inner) => {
                // force expr unwraps Lazy<T> to T, or returns T if not lazy
                let inner_type = self.infer_expr_type(inner);
                match inner_type {
                    ResolvedType::Lazy(t) => *t,
                    other => other,
                }
            }
            _ => ResolvedType::I64, // Default fallback
        }
    }
}
