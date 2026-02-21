//! Type inference utilities for Vais code generator
//!
//! This module contains functions for inferring types of expressions
//! and blocks during code generation.

use crate::CodeGenerator;
use vais_ast::{BinOp, Expr, Spanned, Stmt};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Infer type of a statement block (for if-else phi nodes)
    #[inline]
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
    #[inline]
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
            // Check if Call is an enum variant constructor or struct tuple literal
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    // Enum variant constructors (e.g., Some(x)) return pointers
                    if self.get_tuple_variant_info(name).is_some() {
                        return false;
                    }
                    // Struct tuple literals (e.g., Point(40, 2)) return pointers
                    let resolved = self.resolve_struct_name(name);
                    if self.types.structs.contains_key(&resolved)
                        && !self.types.functions.contains_key(name)
                    {
                        return false;
                    }
                }
                true // regular function call produces a value
            }
            Expr::MethodCall { .. } => true, // method call produces a value
            Expr::StaticMethodCall { .. } => true, // static method call produces a value
            // Struct-typed local variables are stored as pointers (double-pointer)
            // so generate_expr returns a pointer, not a value
            Expr::Ident(name) => {
                // Unit enum variants (e.g., None) produce pointers
                if self.is_unit_enum_variant(name) {
                    return false;
                }
                if let Some(local) = self.fn_ctx.locals.get(name) {
                    // The `self` parameter in methods is passed as a pointer (%Struct* %self),
                    // not by value, so it should not be treated as a value expression.
                    if name == "self"
                        && local.is_param()
                        && matches!(local.ty, ResolvedType::Named { .. })
                    {
                        return false;
                    }
                    // Other parameters are passed by value (even struct types)
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
            Expr::String(_) | Expr::StringInterp(_) => ResolvedType::Str,
            // @ refers to self in methods
            Expr::SelfCall => {
                if let Some(local) = self.fn_ctx.locals.get("self") {
                    local.ty.clone()
                } else {
                    ResolvedType::I64
                }
            }
            Expr::Ident(name) => {
                // Look up local variable type
                if let Some(local) = self.fn_ctx.locals.get(name) {
                    local.ty.clone()
                } else if self.is_unit_enum_variant(name) {
                    // Unit enum variant (e.g., None)
                    for enum_info in self.types.enums.values() {
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
            Expr::Call { func, args } => {
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
                    if let Some(fn_info) = self.types.functions.get(fn_name) {
                        let ret_ty = fn_info.signature.ret.clone();
                        // Convert i32 returns to i64 since codegen promotes them
                        if ret_ty == ResolvedType::I32 {
                            return ResolvedType::I64;
                        }
                        return ret_ty;
                    }
                    // Check generic instantiation: if fn_name is a generic function,
                    // resolve the specialization based on argument types and return its ret type.
                    if let Some(inst_list) = self.generics.fn_instantiations.get(fn_name).cloned() {
                        let arg_types: Vec<ResolvedType> =
                            args.iter().map(|a| self.infer_expr_type(a)).collect();
                        let mangled = self.resolve_generic_call(fn_name, &arg_types, &inst_list);
                        if let Some(fn_info) = self.types.functions.get(&mangled) {
                            let ret_ty = fn_info.signature.ret.clone();
                            if ret_ty == ResolvedType::I32 {
                                return ResolvedType::I64;
                            }
                            return ret_ty;
                        }
                    }
                    // Struct tuple literal: Point(40, 2) — not a function, but a struct
                    let resolved = self.resolve_struct_name(fn_name);
                    if self.types.structs.contains_key(&resolved) {
                        return ResolvedType::Named {
                            name: resolved,
                            generics: vec![],
                        };
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
            Expr::MapLit(pairs) => {
                if let Some((k, v)) = pairs.first() {
                    ResolvedType::Map(
                        Box::new(self.infer_expr_type(k)),
                        Box::new(self.infer_expr_type(v)),
                    )
                } else {
                    ResolvedType::Map(Box::new(ResolvedType::I64), Box::new(ResolvedType::I64))
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
            Expr::StructLit { name, fields } => {
                // First try to get the struct from non-generic structs
                if let Some(struct_info) = self.types.structs.get(&name.node) {
                    // Collect generic parameters from struct fields
                    let mut generic_params = Vec::new();
                    for (_, field_ty) in &struct_info.fields {
                        if let ResolvedType::Generic(param) = field_ty {
                            if !generic_params.contains(param) {
                                generic_params.push(param.clone());
                            }
                        }
                    }

                    // If the struct is generic, infer concrete types from the field values
                    if !generic_params.is_empty() {
                        let mut inferred_types = Vec::new();

                        // For each generic parameter, find the first field that uses it and infer from the value
                        for param in &generic_params {
                            let mut inferred = None;
                            for (field_name, field_expr) in fields {
                                // Find the field info
                                if let Some((_, ResolvedType::Generic(p))) = struct_info
                                    .fields
                                    .iter()
                                    .find(|(name, _)| name == &field_name.node)
                                {
                                    if p == param {
                                        inferred = Some(self.infer_expr_type(field_expr));
                                        break;
                                    }
                                }
                            }
                            inferred_types.push(inferred.unwrap_or(ResolvedType::I64));
                        }

                        return ResolvedType::Named {
                            name: name.node.clone(),
                            generics: inferred_types,
                        };
                    }
                }

                // Try to get from generic struct definitions (AST)
                if let Some(generic_struct) = self.generics.struct_defs.get(&name.node) {
                    // Infer generic type arguments from field values
                    if !generic_struct.generics.is_empty() {
                        let mut inferred_types = Vec::new();

                        // For each generic parameter, find a field that uses it and infer from the value
                        for generic_param in &generic_struct.generics {
                            // Skip lifetime parameters
                            if matches!(
                                generic_param.kind,
                                vais_ast::GenericParamKind::Lifetime { .. }
                            ) {
                                continue;
                            }

                            let param_name = &generic_param.name.node;
                            let mut inferred = None;

                            // Look through the struct fields to find one that uses this generic parameter
                            for struct_field in &generic_struct.fields {
                                if let vais_ast::Type::Named {
                                    name: field_type_name,
                                    ..
                                } = &struct_field.ty.node
                                {
                                    if field_type_name == param_name {
                                        // This field uses the generic parameter - find the corresponding field value
                                        if let Some((_, field_expr)) =
                                            fields.iter().find(|(field_name, _)| {
                                                field_name.node == struct_field.name.node
                                            })
                                        {
                                            inferred = Some(self.infer_expr_type(field_expr));
                                            break;
                                        }
                                    }
                                }
                            }

                            inferred_types.push(inferred.unwrap_or(ResolvedType::I64));
                        }

                        return ResolvedType::Named {
                            name: name.node.clone(),
                            generics: inferred_types,
                        };
                    }
                }

                // Return Named type for struct literals (non-generic case)
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

                // String method return types
                // Note: bool methods return i64 (0/1) at runtime, matching comparison ops
                if matches!(recv_type, ResolvedType::Str) {
                    return match method.node.as_str() {
                        "len" | "charAt" | "indexOf" | "contains" | "startsWith" | "endsWith"
                        | "isEmpty" => ResolvedType::I64,
                        "substring" => ResolvedType::Str,
                        _ => ResolvedType::I64,
                    };
                }

                if let ResolvedType::Named { name, .. } = &recv_type {
                    let method_name = format!("{}_{}", name, method.node);
                    if let Some(fn_info) = self.types.functions.get(&method_name) {
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
                if let Some(fn_info) = self.types.functions.get(&method_name) {
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
            Expr::Field {
                expr: obj_expr,
                field,
            } => {
                // Get the type of the object being accessed
                let obj_type = self.infer_expr_type(obj_expr);
                // If it's a named type (struct), look up the field type
                if let ResolvedType::Named {
                    name: struct_name, ..
                } = &obj_type
                {
                    if let Some(struct_info) = self.types.structs.get(struct_name) {
                        for (field_name, field_type) in &struct_info.fields {
                            if field_name == &field.node {
                                return field_type.clone();
                            }
                        }
                    }
                }
                // Default fallback if field not found
                ResolvedType::I64
            }
            Expr::Binary { left, right, op } => {
                // For comparison/logical ops, result is always integer (i64)
                if matches!(
                    op,
                    BinOp::Lt
                        | BinOp::Lte
                        | BinOp::Gt
                        | BinOp::Gte
                        | BinOp::Eq
                        | BinOp::Neq
                        | BinOp::And
                        | BinOp::Or
                ) {
                    return ResolvedType::I64;
                }
                // For arithmetic, propagate float type from either operand
                let left_ty = self.infer_expr_type(left);
                let right_ty = self.infer_expr_type(right);
                if matches!(left_ty, ResolvedType::F64) || matches!(right_ty, ResolvedType::F64) {
                    ResolvedType::F64
                } else if matches!(left_ty, ResolvedType::F32)
                    || matches!(right_ty, ResolvedType::F32)
                {
                    ResolvedType::F32
                } else {
                    left_ty
                }
            }
            Expr::Unary { expr: inner, .. } => self.infer_expr_type(inner),
            Expr::Ternary { then, .. } => {
                // Ternary returns the type of its then branch
                self.infer_expr_type(then)
            }
            Expr::If { then, .. } => {
                // If-else returns the type of its then block
                self.infer_block_type(then)
            }
            Expr::Match { arms, .. } => {
                // Match returns the type of its first arm body
                if let Some(arm) = arms.first() {
                    self.infer_expr_type(&arm.body)
                } else {
                    ResolvedType::I64
                }
            }
            Expr::Cast { ty, .. } => {
                // Cast returns the target type
                self.ast_type_to_resolved(&ty.node)
            }
            Expr::Range { start, .. } => {
                // Range returns Range<T> where T is the type of start (or i64 default)
                let elem_type = if let Some(s) = start {
                    self.infer_expr_type(s)
                } else {
                    ResolvedType::I64
                };
                ResolvedType::Range(Box::new(elem_type))
            }
            Expr::Unit => ResolvedType::Unit,
            Expr::Spawn(inner) => {
                let inner_ty = self.infer_expr_type(inner);
                // Spawn wraps non-Future values in Future<T>
                match inner_ty {
                    ResolvedType::Future(_) => inner_ty,
                    other => ResolvedType::Future(Box::new(other)),
                }
            }
            Expr::Await(inner) => {
                let inner_ty = self.infer_expr_type(inner);
                // Await unwraps Future<T> to T
                match inner_ty {
                    ResolvedType::Future(t) => *t,
                    other => {
                        // ICE: await on non-Future type is likely a type checker bug
                        eprintln!(
                            "ICE: await on non-Future type `{}` in codegen, treating as passthrough",
                            other
                        );
                        other
                    }
                }
            }
            Expr::Yield(inner) => self.infer_expr_type(inner),
            _ => ResolvedType::I64, // Default fallback for remaining expressions
        }
    }

    /// Generate a condition conversion to i1 for branch instructions.
    /// If the expression is already i1 (bool), use it directly.
    /// Otherwise, convert via `icmp ne <type> %val, 0`.
    pub(crate) fn generate_cond_to_i1(
        &mut self,
        cond_expr: &Spanned<Expr>,
        cond_val: &str,
        counter: &mut usize,
    ) -> (String, String) {
        let cond_ty = self.infer_expr_type(cond_expr);
        if cond_ty == ResolvedType::Bool {
            // Already i1, use directly
            (cond_val.to_string(), String::new())
        } else {
            let cond_bool = self.next_temp(counter);
            let llvm_ty = self.type_to_llvm(&cond_ty);
            let ir = format!("  {} = icmp ne {} {}, 0\n", cond_bool, llvm_ty, cond_val);
            (cond_bool, ir)
        }
    }
}
