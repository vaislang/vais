//! Type inference utilities for Vais code generator.
//!
//! ## Why codegen re-infers types
//!
//! The type checker (vais-types) resolves all types at the semantic level,
//! but codegen needs additional type information that the TC doesn't provide:
//!
//! - **Block type** (`infer_block_type`): determines LLVM phi node types for
//!   if-else/match expressions. The TC resolves the overall expression type,
//!   but codegen needs to know the type of each branch independently.
//!
//! - **Value vs pointer** (`is_expr_value`): struct literals produce alloca
//!   pointers, while function calls produce SSA values. This distinction is
//!   purely an IR concern — the TC doesn't track it.
//!
//! - **Expression type** (`infer_expr_type`): used for integer width coercion,
//!   struct-to-i64 erasure, and cross-module type tag resolution. The TC result
//!   is available via `tc_expr_type()` (span-keyed lookup) but not yet integrated
//!   into all codegen paths.
//!
//! ## Future optimization
//!
//! When `expr_types` (span→ResolvedType map from TC) is fully integrated,
//! many `infer_expr_type` calls can be replaced with `tc_expr_type()` lookups,
//! reducing redundant AST traversal in codegen.

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
                Stmt::Expr(expr) => {
                    let ty = self.infer_expr_type(expr);
                    // If the last expression is an Ident that returns I64 (fallback),
                    // check if there's a Let in this block that defines it —
                    // this handles vec![] macro blocks where the var isn't in locals yet
                    if let Expr::Ident(var_name) = &expr.node {
                        let in_locals = self.fn_ctx.locals.contains_key(var_name.as_str());
                        if !in_locals || matches!(ty, ResolvedType::I64) {
                            // Search block for a Let defining this variable
                            for stmt in stmts.iter().rev().skip(1) {
                                if let Stmt::Let { name, value, .. } = &stmt.node {
                                    if name.node == *var_name {
                                        let resolved = self.infer_expr_type(value);
                                        if !matches!(resolved, ResolvedType::I64) {
                                            return resolved;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    ty
                }
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
                Stmt::Expr(expr) => {
                    // For Ident expressions referencing block-local Let bindings,
                    // the variable will be stored as struct (Named type) → not a value
                    if let Expr::Ident(var_name) = &expr.node {
                        if !self.fn_ctx.locals.contains_key(var_name.as_str()) {
                            // Search block for a Let defining this variable
                            for stmt in stmts.iter().rev().skip(1) {
                                if let Stmt::Let { name, value, .. } = &stmt.node {
                                    if name.node == *var_name {
                                        let ty = self.infer_expr_type(value);
                                        if matches!(ty, ResolvedType::Named { .. }) {
                                            return false; // struct-typed local → pointer
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    self.is_expr_value(expr)
                }
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
                    // load_typed() for Named types returns an alloca pointer, not a value.
                    // The codegen for Named/Str in load_typed always uses the
                    // alloca+memcpy path regardless of size.
                    if name == "load_typed" {
                        if let Some(concrete) = self.get_generic_substitution("T") {
                            if matches!(concrete, ResolvedType::Named { .. }) {
                                return false;
                            }
                        }
                    }
                }
                true // regular function call produces a value
            }
            Expr::MethodCall { .. } => true, // method call produces a value
            Expr::StaticMethodCall { .. } => true, // static method call produces a value
            // Struct-typed local variables are stored as pointers (single alloca %T*)
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
                    // Struct/enum local variables are stored as single-pointer alloca (%Struct* %var)
                    // so generate_expr returns a pointer, not a value
                    !matches!(local.ty, ResolvedType::Named { .. })
                } else {
                    true
                }
            }
            Expr::Block(stmts) => {
                // Block value-ness is determined by its last expression
                self.is_block_result_value(stmts)
            }
            Expr::Field { expr: obj, field } => {
                // Field access on a struct returns a GEP pointer for Named fields.
                // Check if the field type is Named (struct/enum) — if so, the GEP
                // returns a pointer, not a value.
                let obj_type = self.infer_expr_type(obj);
                let resolved = match &obj_type {
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
                    other => other,
                };
                if let ResolvedType::Named { name, .. } = resolved {
                    let struct_name = self.resolve_struct_name(name);
                    if let Some(struct_info) = self.types.structs.get(&struct_name) {
                        if let Some((_fname, fty)) =
                            struct_info.fields.iter().find(|(n, _)| n == &field.node)
                        {
                            return !matches!(fty, ResolvedType::Named { .. });
                        }
                    }
                }
                true
            }
            _ => true,
        }
    }

    /// Check if param_ty is a Slice type but inferred_ty is a Vec (or Ref(Vec)).
    /// In cross-module codegen, the parameter LLVM type ({ ptr, i64 }) differs from
    /// the Vec struct type (%Vec), causing a type mismatch. Using the inferred type
    /// for the call argument avoids this — both are layout-compatible.
    pub(crate) fn is_vec_to_slice_coercion(
        param_ty: &ResolvedType,
        inferred_ty: &ResolvedType,
    ) -> bool {
        let param_is_slice = matches!(param_ty, ResolvedType::Slice(_) | ResolvedType::SliceMut(_))
            || matches!(param_ty, ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
            if matches!(inner.as_ref(), ResolvedType::Slice(_) | ResolvedType::SliceMut(_)));

        if !param_is_slice {
            return false;
        }

        // Check if inferred type is Vec or Ref(Vec)
        let is_vec = |ty: &ResolvedType| -> bool {
            matches!(ty, ResolvedType::Named { name, .. } if name == "Vec")
        };
        match inferred_ty {
            t if is_vec(t) => true,
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => is_vec(inner),
            _ => false,
        }
    }

    /// Infer type of expression (simple version for let statement)
    pub(crate) fn infer_expr_type(&self, expr: &Spanned<Expr>) -> ResolvedType {
        stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
            self.infer_expr_type_inner(expr)
        })
    }

    /// Get the TC-resolved type for an expression, if available.
    /// This is more accurate than infer_expr_type but may not be available for all expressions.
    /// Use for targeted lookups where TC accuracy is critical (e.g., generic type resolution).
    #[allow(dead_code)]
    pub(crate) fn tc_expr_type(&self, expr: &Spanned<Expr>) -> Option<&ResolvedType> {
        let span_key = (expr.span.start, expr.span.end);
        self.expr_types.get(&span_key)
    }

    #[inline(never)]
    fn infer_expr_type_inner(&self, expr: &Spanned<Expr>) -> ResolvedType {
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
                } else if let Some(global_info) = self.types.globals.get(name) {
                    // Global variable: use its declared type
                    global_info._ty.clone()
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
                } else if let Some(const_info) = self.types.constants.get(name) {
                    // Cross-module constant: infer type from constant's declared type
                    let ty = const_info._ty.clone();
                    if matches!(ty, ResolvedType::Unknown) {
                        // Fallback: infer from constant value expression
                        self.infer_expr_type(&const_info.value.clone())
                    } else {
                        ty
                    }
                } else if self.types.structs.contains_key(name) {
                    // Bare struct name used as a type reference (e.g., DistanceMetric in DistanceMetric.Cosine)
                    ResolvedType::Named {
                        name: name.clone(),
                        generics: vec![],
                    }
                } else if self.types.enums.contains_key(name) {
                    // Bare enum name used as a type reference (e.g., DistanceMetric enum)
                    ResolvedType::Named {
                        name: name.clone(),
                        generics: vec![],
                    }
                } else if self.generics.struct_aliases.contains_key(name) {
                    // Generic struct alias (e.g., "Vec" -> "Vec$i64")
                    ResolvedType::Named {
                        name: name.clone(),
                        generics: vec![],
                    }
                } else if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    // PascalCase identifier not found in locals — likely a cross-module
                    // struct/enum type name. Return Named type so downstream field access
                    // and method call resolution can work.
                    //
                    // When the struct/enum is actually registered, use the resolved name
                    // (which may be a mangled specialization). This ensures that cross-module
                    // field access generates correct IR when the struct is found in the registry.
                    let resolved_name = if self.types.structs.contains_key(name.as_str())
                        || self.types.enums.contains_key(name.as_str())
                    {
                        // Struct/enum is directly registered — use its canonical name.
                        name.clone()
                    } else if self.generics.generated_structs.contains_key(name.as_str()) {
                        // Already a specialized/mangled struct name (e.g., "Point$i64").
                        name.clone()
                    } else if let Some(mangled) = self.generics.struct_aliases.get(name.as_str()) {
                        // Base generic name has a registered specialization alias.
                        mangled.clone()
                    } else {
                        // Not found anywhere — return as-is; may be resolved later.
                        name.clone()
                    };
                    ResolvedType::Named {
                        name: resolved_name,
                        generics: vec![],
                    }
                } else {
                    ResolvedType::I64 // Default for lowercase identifiers
                }
            }
            Expr::Call { func, args } => {
                // Handle @(args) — self-recursive call returns the current function's type
                if let Expr::SelfCall = &func.node {
                    if let Some(fn_name) = &self.fn_ctx.current_function {
                        // In async poll functions, current_function is "name__poll"
                        // but the function info is stored under the original name.
                        let lookup_name = fn_name
                            .strip_suffix("__poll")
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| fn_name.clone());
                        if let Some(fn_info) = self.types.functions.get(&lookup_name) {
                            let ret_ty = fn_info.signature.ret.clone();
                            if fn_info.signature.is_async {
                                return ResolvedType::Future(Box::new(ret_ty));
                            }
                            if ret_ty == ResolvedType::I32 {
                                return ResolvedType::I64;
                            }
                            return ret_ty;
                        }
                    }
                    return self
                        .fn_ctx
                        .current_return_type
                        .clone()
                        .unwrap_or(ResolvedType::I64);
                }
                // Get return type from function info
                if let Expr::Ident(fn_name) = &func.node {
                    // Check if this is an enum variant constructor.
                    // For well-known generic enums (Option, Result), propagate the
                    // argument type as a generic parameter so downstream pattern matching
                    // can resolve variant field types correctly (fixes struct erasure).
                    if let Some((enum_name, _)) = self.get_tuple_variant_info(fn_name) {
                        let generics = match fn_name.as_str() {
                            "Some" => {
                                // Option<T> — T is the argument type
                                if let Some(arg) = args.first() {
                                    vec![self.infer_expr_type(arg)]
                                } else {
                                    vec![]
                                }
                            }
                            "Ok" => {
                                // Result<T, E> — T is the argument type, E is unknown
                                if let Some(arg) = args.first() {
                                    vec![self.infer_expr_type(arg), ResolvedType::I64]
                                } else {
                                    vec![]
                                }
                            }
                            "Err" => {
                                // Result<T, E> — T is unknown, E is the argument type
                                if let Some(arg) = args.first() {
                                    vec![ResolvedType::I64, self.infer_expr_type(arg)]
                                } else {
                                    vec![]
                                }
                            }
                            _ => vec![],
                        };
                        return ResolvedType::Named {
                            name: enum_name,
                            generics,
                        };
                    }
                    // Builtins: load_typed returns T, type_size returns i64, sizeof returns i64
                    if fn_name == "load_typed" {
                        // load_typed(ptr) -> T where T is the current generic substitution.
                        // Only return the concrete type when the current function is
                        // actually specialized (mangled with '$'). In non-specialized
                        // (generic) functions, the IR value is i64 (zext'd).
                        let fn_is_specialized = self
                            .fn_ctx
                            .current_function
                            .as_ref()
                            .map(|n| n.contains('$'))
                            .unwrap_or(false);
                        if fn_is_specialized {
                            if let Some(concrete) = self.get_generic_substitution("T") {
                                return concrete;
                            }
                        }
                        return ResolvedType::I64;
                    }
                    // Check function info
                    if let Some(fn_info) = self.types.functions.get(fn_name) {
                        let ret_ty = fn_info.signature.ret.clone();
                        // Async functions return Future<T> from the caller's perspective.
                        // The create function returns an i64 state pointer, but for type
                        // inference, we need to express that `check()` yields Future<bool>.
                        if fn_info.signature.is_async {
                            return ResolvedType::Future(Box::new(ret_ty));
                        }
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
                    // Fallback: check resolved_function_sigs from type checker
                    if let Some(sig) = self.types.resolved_function_sigs.get(fn_name) {
                        return sig.ret.clone();
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
            Expr::StructLit { name, fields, .. } => {
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
                        ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => *elem,
                        // Vec<T>[idx] → T
                        ResolvedType::Named {
                            ref name,
                            ref generics,
                        } if name == "Vec" && !generics.is_empty() => generics[0].clone(),
                        ResolvedType::Ref(ref inner) | ResolvedType::RefMut(ref inner) => {
                            match inner.as_ref() {
                                ResolvedType::Named {
                                    ref name,
                                    ref generics,
                                } if name == "Vec" && !generics.is_empty() => generics[0].clone(),
                                ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                                    *elem.clone()
                                }
                                ResolvedType::Array(elem) => *elem.clone(),
                                _ => ResolvedType::I64,
                            }
                        }
                        _ => ResolvedType::I64,
                    }
                }
            }
            Expr::Lambda { .. } => {
                // Lambda returns a function pointer (represented as i64)
                ResolvedType::I64
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
                ..
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

                // Vec<T> method return types
                let vec_elem = match &recv_type {
                    ResolvedType::Named { name, generics }
                        if name == "Vec" && !generics.is_empty() =>
                    {
                        Some(generics[0].clone())
                    }
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::Named { name, generics } = inner.as_ref() {
                            if name == "Vec" && !generics.is_empty() {
                                Some(generics[0].clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(elem_ty) = vec_elem {
                    return match method.node.as_str() {
                        "len" | "capacity" => ResolvedType::U64,
                        "push" | "insert" | "remove" | "clear" | "truncate" | "resize" | "swap"
                        | "sort" | "reverse" => ResolvedType::I64,
                        "pop" | "get" | "last" | "first" => elem_ty,
                        "clone" => recv_type,
                        "data" => ResolvedType::I64,
                        _ => ResolvedType::I64,
                    };
                }

                // Look up registered function signature first — this is the ground truth
                // from the type checker and always takes priority over hardcoded heuristics.
                // Unwrap Ref/RefMut to get the inner Named type
                let inner_recv = match &recv_type {
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
                    other => other,
                };
                if let ResolvedType::Named { name, generics } = inner_recv {
                    let method_name = format!("{}_{}", name, method.node);
                    // Check specialized versions first via fn_instantiations.
                    // Strategy 1: use receiver's generic args to construct mangled name
                    // (e.g., Cell<bool>.get() -> Cell_get$bool)
                    if !generics.is_empty() {
                        let mangled = vais_types::mangle_name(&method_name, generics);
                        if let Some(fn_info) = self.types.functions.get(&mangled) {
                            return fn_info.signature.ret.clone();
                        }
                    }
                    // Strategy 2: use fn_instantiations + resolve from arg types
                    // (e.g., Box_map -> Box_map$i64 when called with i64 args)
                    if let Some(inst_list) =
                        self.generics.fn_instantiations.get(&method_name).cloned()
                    {
                        let arg_types: Vec<ResolvedType> =
                            args.iter().map(|a| self.infer_expr_type(a)).collect();
                        let mangled =
                            self.resolve_generic_call(&method_name, &arg_types, &inst_list);
                        if let Some(fn_info) = self.types.functions.get(&mangled) {
                            return fn_info.signature.ret.clone();
                        }
                    }
                    if let Some(fn_info) = self.types.functions.get(&method_name) {
                        return fn_info.signature.ret.clone();
                    }
                    // Fallback: check resolved_function_sigs from type checker
                    if let Some(sig) = self.types.resolved_function_sigs.get(&method_name) {
                        return sig.ret.clone();
                    }
                }

                // Hardcoded heuristics for types without registered signatures
                // (e.g., std library types used without explicit impl blocks in scope)
                if let ResolvedType::Named { name, .. } = inner_recv {
                    if name == "ByteBuffer" {
                        return match method.node.as_str() {
                            "read_u8" | "read_i8" | "read_u16" | "read_i16" | "read_u32"
                            | "read_i32" | "read_u64" | "read_i64" => ResolvedType::I64,
                            "read_f32" => ResolvedType::F32,
                            "read_f64" => ResolvedType::F64,
                            "read_str" | "read_string" => ResolvedType::Str,
                            "read_bool" => ResolvedType::Bool,
                            "len" | "position" | "remaining" | "capacity" => ResolvedType::I64,
                            "to_vec" | "as_bytes" => ResolvedType::Named {
                                name: "Vec".to_string(),
                                generics: vec![ResolvedType::U8],
                            },
                            "clone" => recv_type,
                            _ => ResolvedType::I64,
                        };
                    }

                    // Mutex.lock() returns MutexGuard
                    if name == "Mutex" && method.node == "lock" {
                        return ResolvedType::Named {
                            name: "MutexGuard".to_string(),
                            generics: vec![],
                        };
                    }
                }

                // clone on any type returns the same type
                if method.node == "clone" {
                    return recv_type;
                }

                // Fallback: when receiver type is not Named (I64/Unknown fallback),
                // search self.types.functions for any `StructName_method` that matches.
                // This handles cross-module method calls where the receiver type
                // couldn't be resolved from locals alone.
                if !matches!(inner_recv, ResolvedType::Named { .. }) {
                    let method_suffix = format!("_{}", method.node);
                    let mut candidates = Vec::new();
                    for (fn_name, fn_info) in &self.types.functions {
                        if fn_name.ends_with(&method_suffix) {
                            candidates.push((fn_name.clone(), fn_info.signature.ret.clone()));
                        }
                    }
                    // Also check resolved_function_sigs
                    for (fn_name, sig) in &self.types.resolved_function_sigs {
                        if fn_name.ends_with(&method_suffix)
                            && !candidates.iter().any(|(n, _)| n == fn_name)
                        {
                            candidates.push((fn_name.clone(), sig.ret.clone()));
                        }
                    }
                    // If exactly one match, use it unambiguously
                    if candidates.len() == 1 {
                        return candidates[0].1.clone();
                    }
                }

                ResolvedType::I64
            }
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                // Get static method return type from function info.
                // Check specialized versions first via fn_instantiations so
                // generic return types (e.g., Box<T>) are resolved to concrete
                // types (e.g., Box<i64>) when a specialization exists.
                let method_name = format!("{}_{}", type_name.node, method.node);
                if let Some(inst_list) = self.generics.fn_instantiations.get(&method_name).cloned()
                {
                    let arg_types: Vec<ResolvedType> =
                        args.iter().map(|a| self.infer_expr_type(a)).collect();
                    let mangled = self.resolve_generic_call(&method_name, &arg_types, &inst_list);
                    if let Some(fn_info) = self.types.functions.get(&mangled) {
                        return fn_info.signature.ret.clone();
                    }
                }
                if let Some(fn_info) = self.types.functions.get(&method_name) {
                    return fn_info.signature.ret.clone();
                }
                // Fallback: check resolved_function_sigs from type checker
                if let Some(sig) = self.types.resolved_function_sigs.get(&method_name) {
                    return sig.ret.clone();
                }
                // Fallback: keep I64 as default for codegen (must match actual IR)
                // The codegen's type inference must match the actual IR function signatures.
                // Static methods on unknown types return i64 at IR level.
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

                // Unwrap Ref/RefMut/Pointer to get the inner type
                let inner_type = match &obj_type {
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
                    ResolvedType::Pointer(inner) => inner.as_ref(),
                    other => other,
                };

                // If it's a named type (struct), look up the field type
                if let ResolvedType::Named {
                    name: struct_name, ..
                } = inner_type
                {
                    // Try direct name, then resolved alias name
                    let resolved = self.resolve_struct_name(struct_name);
                    let names_to_try = [struct_name.as_str(), resolved.as_str()];
                    for candidate in &names_to_try {
                        if let Some(struct_info) = self.types.structs.get(*candidate) {
                            for (field_name, field_type) in &struct_info.fields {
                                if field_name == &field.node {
                                    return field_type.clone();
                                }
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
                    // For integer arithmetic, the codegen promotes to the wider type.
                    // If either operand is i64/u64, the result is i64 in the IR.
                    let left_bits = self.get_integer_bits(&left_ty);
                    let right_bits = self.get_integer_bits(&right_ty);
                    if (left_bits > 0 && right_bits > 0 && left_bits != right_bits)
                        || left_bits == 64
                        || right_bits == 64
                    {
                        ResolvedType::I64
                    } else {
                        left_ty
                    }
                }
            }
            Expr::Unary { op, expr: inner } => {
                match op {
                    vais_ast::UnaryOp::Not => ResolvedType::Bool,
                    _ => self.infer_expr_type(inner),
                }
            }
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
                        debug_assert!(false, "ICE: await on non-Future type `{other}` in codegen");
                        other
                    }
                }
            }
            Expr::Yield(inner) => self.infer_expr_type(inner),
            Expr::Unwrap(inner) => {
                // Unwrap: Result<T, E> → T, Optional<T> → T
                let inner_ty = self.infer_expr_type(inner);
                match inner_ty {
                    ResolvedType::Result(ok, _) => *ok,
                    ResolvedType::Optional(inner) => *inner,
                    ResolvedType::Named {
                        ref name,
                        ref generics,
                    } if name == "Result" => {
                        // Result<T, E> — first generic is the Ok type
                        if !generics.is_empty() {
                            generics[0].clone()
                        } else {
                            ResolvedType::I64
                        }
                    }
                    ResolvedType::Named {
                        ref name,
                        ref generics,
                    } if name == "Option" => {
                        // Option<T> — first generic is the inner type
                        if !generics.is_empty() {
                            generics[0].clone()
                        } else {
                            ResolvedType::I64
                        }
                    }
                    other => other, // If not Result/Option, pass through
                }
            }
            Expr::Try(inner) => {
                // Try (?) operator: same unwrapping as Unwrap for type purposes
                let inner_ty = self.infer_expr_type(inner);
                match inner_ty {
                    ResolvedType::Result(ok, _) => *ok,
                    ResolvedType::Optional(inner) => *inner,
                    ResolvedType::Named {
                        ref name,
                        ref generics,
                    } if name == "Result" => {
                        if !generics.is_empty() {
                            generics[0].clone()
                        } else {
                            ResolvedType::I64
                        }
                    }
                    ResolvedType::Named {
                        ref name,
                        ref generics,
                    } if name == "Option" => {
                        if !generics.is_empty() {
                            generics[0].clone()
                        } else {
                            ResolvedType::I64
                        }
                    }
                    other => other,
                }
            }
            Expr::Block(stmts) => {
                // Block returns the type of its last expression
                self.infer_block_type(stmts)
            }
            Expr::Assign { value, .. } | Expr::AssignOp { value, .. } => {
                // Assignment returns the assigned value's type
                self.infer_expr_type(value)
            }
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
