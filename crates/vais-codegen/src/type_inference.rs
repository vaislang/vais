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

/// Phase 16 A2: Reject TC → local type upgrades whose node shape can't possibly
/// produce `ty`. Span is (start, end) byte-offset-only with no file_id, so in
/// cross-module builds different source files can collide on the same span key
/// and TC's stored type for one file bleeds into another. A small fast check
/// on the AST node itself is enough to catch the bleed — an Int literal can
/// never carry a Vec<f32> type, and a Float literal can never carry a Named
/// struct type — without having to thread file identity through every span key.
/// Unknown / composite shapes fall through to `true` so this guard never
/// rejects a legitimate upgrade.
fn expr_shape_matches_type(node: &Expr, ty: &ResolvedType) -> bool {
    use ResolvedType as R;
    match node {
        Expr::Int(_) => matches!(
            ty,
            R::I8
                | R::I16
                | R::I32
                | R::I64
                | R::I128
                | R::U8
                | R::U16
                | R::U32
                | R::U64
                | R::U128
                | R::F32
                | R::F64
                | R::Bool
        ),
        Expr::Float(_) => matches!(ty, R::F32 | R::F64),
        Expr::Bool(_) => matches!(ty, R::Bool | R::I64),
        Expr::String(_) | Expr::StringInterp(_) => {
            matches!(ty, R::Str) || matches!(ty, R::Ref(inner) if matches!(inner.as_ref(), R::Str))
        }
        _ => true,
    }
}

/// Phase 6.30.2: recursive check for unresolved inference variables inside a
/// ResolvedType. TC sometimes stores expr_types containing Var(n) when the
/// type is unified later without rewriting the stored entry. Treat these as
/// less-informative than the codegen-local inference and skip the upgrade.
fn contains_unresolved_var(ty: &ResolvedType) -> bool {
    match ty {
        ResolvedType::Var(_) => true,
        ResolvedType::Array(inner)
        | ResolvedType::Optional(inner)
        | ResolvedType::Pointer(inner)
        | ResolvedType::Ref(inner)
        | ResolvedType::RefMut(inner)
        | ResolvedType::Slice(inner)
        | ResolvedType::SliceMut(inner)
        | ResolvedType::Range(inner)
        | ResolvedType::Future(inner) => contains_unresolved_var(inner),
        ResolvedType::ConstArray { element, .. } => contains_unresolved_var(element),
        ResolvedType::Map(k, v) => contains_unresolved_var(k) || contains_unresolved_var(v),
        ResolvedType::Result(ok, err) => {
            contains_unresolved_var(ok) || contains_unresolved_var(err)
        }
        ResolvedType::Tuple(items) => items.iter().any(contains_unresolved_var),
        ResolvedType::Named { generics, .. } => generics.iter().any(contains_unresolved_var),
        ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
            params.iter().any(contains_unresolved_var) || contains_unresolved_var(ret)
        }
        _ => false,
    }
}

fn contains_unresolved_generic_or_var(ty: &ResolvedType) -> bool {
    match ty {
        ResolvedType::Var(_) | ResolvedType::Generic(_) => true,
        ResolvedType::Array(inner)
        | ResolvedType::Optional(inner)
        | ResolvedType::Pointer(inner)
        | ResolvedType::Ref(inner)
        | ResolvedType::RefMut(inner)
        | ResolvedType::Slice(inner)
        | ResolvedType::SliceMut(inner)
        | ResolvedType::Range(inner)
        | ResolvedType::Future(inner) => contains_unresolved_generic_or_var(inner),
        ResolvedType::ConstArray { element, .. } => contains_unresolved_generic_or_var(element),
        ResolvedType::Map(k, v) => {
            contains_unresolved_generic_or_var(k) || contains_unresolved_generic_or_var(v)
        }
        ResolvedType::Result(ok, err) => {
            contains_unresolved_generic_or_var(ok) || contains_unresolved_generic_or_var(err)
        }
        ResolvedType::Tuple(items) => items.iter().any(contains_unresolved_generic_or_var),
        ResolvedType::Named { generics, .. } => {
            generics.iter().any(contains_unresolved_generic_or_var)
        }
        ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
            params.iter().any(contains_unresolved_generic_or_var)
                || contains_unresolved_generic_or_var(ret)
        }
        _ => false,
    }
}

fn unresolved_generic_can_upgrade_to(local: &ResolvedType, tc_ty: &ResolvedType) -> bool {
    if contains_unresolved_generic_or_var(tc_ty) {
        return false;
    }

    match (local, tc_ty) {
        (ResolvedType::Var(_) | ResolvedType::Generic(_), _) => true,
        (
            ResolvedType::Named {
                name: l_name,
                generics: l_gen,
            },
            ResolvedType::Named {
                name: r_name,
                generics: r_gen,
            },
        ) if l_name == r_name && l_gen.len() == r_gen.len() => {
            l_gen.iter().any(contains_unresolved_generic_or_var)
        }
        (ResolvedType::Ref(l), ResolvedType::Ref(r))
        | (ResolvedType::RefMut(l), ResolvedType::RefMut(r))
        | (ResolvedType::Pointer(l), ResolvedType::Pointer(r))
        | (ResolvedType::Optional(l), ResolvedType::Optional(r))
        | (ResolvedType::Slice(l), ResolvedType::Slice(r))
        | (ResolvedType::SliceMut(l), ResolvedType::SliceMut(r))
        | (ResolvedType::Array(l), ResolvedType::Array(r))
        | (ResolvedType::Future(l), ResolvedType::Future(r))
        | (ResolvedType::Range(l), ResolvedType::Range(r)) => {
            unresolved_generic_can_upgrade_to(l, r)
        }
        (ResolvedType::Tuple(l_items), ResolvedType::Tuple(r_items))
            if l_items.len() == r_items.len() =>
        {
            l_items
                .iter()
                .zip(r_items)
                .any(|(l, r)| unresolved_generic_can_upgrade_to(l, r))
        }
        (ResolvedType::Result(l_ok, l_err), ResolvedType::Result(r_ok, r_err)) => {
            unresolved_generic_can_upgrade_to(l_ok, r_ok)
                || unresolved_generic_can_upgrade_to(l_err, r_err)
        }
        (ResolvedType::Map(l_k, l_v), ResolvedType::Map(r_k, r_v)) => {
            unresolved_generic_can_upgrade_to(l_k, r_k)
                || unresolved_generic_can_upgrade_to(l_v, r_v)
        }
        _ => false,
    }
}

fn is_span_bleed_prone_container(ty: &ResolvedType) -> bool {
    match ty {
        ResolvedType::Named { name, .. } => {
            matches!(
                name.as_str(),
                "Vec" | "HashMap" | "Option" | "Result" | "Box"
            )
        }
        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) | ResolvedType::Pointer(inner) => {
            is_span_bleed_prone_container(inner)
        }
        _ => false,
    }
}

impl CodeGenerator {
    /// Refine a weak codegen-local inference result using the type registered
    /// by the expression emitter for the actual SSA value.
    ///
    /// This keeps statement lowering from allocating/storing as the legacy i64
    /// fallback when the generated value is known to be a struct, tuple, float,
    /// bool, or narrow integer. It intentionally only upgrades weak inference
    /// results so explicit annotations and precise local inference keep winning.
    pub(crate) fn refine_weak_inferred_type_from_value(
        &self,
        inferred_ty: ResolvedType,
        generated_value: &str,
    ) -> ResolvedType {
        if !matches!(
            inferred_ty,
            ResolvedType::I64 | ResolvedType::Unknown | ResolvedType::Never
        ) {
            return inferred_ty;
        }

        let Some(generated_ty) = self.fn_ctx.get_temp_type(generated_value).cloned() else {
            return inferred_ty;
        };

        if matches!(
            generated_ty,
            ResolvedType::I64 | ResolvedType::Unknown | ResolvedType::Never
        ) {
            inferred_ty
        } else {
            generated_ty
        }
    }

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
            Expr::Call { func, args } => {
                if let Expr::Ident(name) = &func.node {
                    // Enum variant constructors (e.g., Some(x)) return pointers
                    if self.get_tuple_variant_info(name).is_some() {
                        return false;
                    }
                    // Phase D: stdlib enum variants (Ok/Err/Some/None) may
                    // not be registered in self.types.enums in every
                    // module subset (e.g., a non-main module doesn't
                    // always import Result's enum body). They still lower
                    // to `alloca %BaseEnum` constructors which return a
                    // pointer — mark them as non-value so the caller
                    // loads the aggregate.
                    if matches!(name.as_str(), "Ok" | "Err" | "Some" | "None") {
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
                        let explicit_t = args.get(1).and_then(|arg| {
                            if let Expr::Ident(ident) = &arg.node {
                                self.get_generic_substitution(ident.as_str())
                            } else {
                                None
                            }
                        });
                        let concrete_t = explicit_t.or_else(|| self.get_generic_substitution("T"));
                        if let Some(concrete) = concrete_t {
                            if matches!(concrete, ResolvedType::Named { .. } | ResolvedType::Str) {
                                return false;
                            }
                        }
                    }
                }
                true // regular function call produces a value
            }
            Expr::MethodCall { .. } => true, // method call produces a value
            Expr::StaticMethodCall {
                type_name, method, ..
            } => {
                // Qualified enum constructors such as `Option.Some(x)` and
                // `Result.Err(e)` lower through generate_enum_variant_constructor,
                // which returns an alloca pointer to the enum aggregate.
                if self
                    .types
                    .enums
                    .get(type_name.node.as_str())
                    .map(|enum_info| enum_info.variants.iter().any(|v| v.name == method.node))
                    .unwrap_or(false)
                {
                    return false;
                }
                // Stdlib enum constructors may be available in cross-module
                // codegen even when the enum registry entry is absent.
                if matches!(
                    (type_name.node.as_str(), method.node.as_str()),
                    ("Option", "Some") | ("Option", "None") | ("Result", "Ok") | ("Result", "Err")
                ) {
                    return false;
                }
                true
            }
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
                    // Enum namespace access (`EnumName.Variant`) is lowered to an
                    // alloca of the enum type, so generate_expr returns a pointer.
                    if let Some(enum_info) = self
                        .types
                        .enums
                        .get(struct_name.as_str())
                        .or_else(|| self.types.enums.get(name.as_str()))
                    {
                        if enum_info.variants.iter().any(|v| v.name == field.node) {
                            return false;
                        }
                    }
                }
                true
            }
            // Vec/Array/Slice indexing whose element type is Named (struct/enum)
            // returns a pointer from an alloca+memcpy (see expr_helpers_data.rs).
            // Non-Named elements return an SSA value.
            Expr::Index { expr: inner, index } => {
                if matches!(index.node, Expr::Range { .. }) {
                    return true;
                }
                let inner_ty = self.infer_expr_type(inner);
                let outer = match &inner_ty {
                    ResolvedType::Ref(i) | ResolvedType::RefMut(i) => i.as_ref(),
                    other => other,
                };
                let elem_ty = match outer {
                    ResolvedType::Pointer(e) | ResolvedType::Array(e) => Some(e.as_ref()),
                    ResolvedType::Slice(e) | ResolvedType::SliceMut(e) => Some(e.as_ref()),
                    ResolvedType::Named { name, generics }
                        if name == "Vec" && !generics.is_empty() =>
                    {
                        Some(&generics[0])
                    }
                    _ => None,
                };
                if let Some(ty) = elem_ty {
                    return !matches!(ty, ResolvedType::Named { .. });
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

    /// Infer type of expression (simple version for let statement).
    /// Phase 3.15/6.27b: run codegen-local inference first, then upgrade to TC's
    /// type only when codegen returned I64 but TC has richer info.
    /// This preserves the behavior of existing passing files (codegen's own
    /// inference for Ref/RefMut/struct paths) while recovering element
    /// types that would otherwise erase through Vec<T> indexing.
    pub(crate) fn infer_expr_type(&self, expr: &Spanned<Expr>) -> ResolvedType {
        let local = stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
            self.infer_expr_type_inner(expr)
        });
        // Phase 6.27b: upgrade from TC expr_types even when local gave Named{Vec,[I64]}
        // (erased from Vec<Tuple> etc). TC's expr_types stores the actual post-unification
        // type at the expression's span, which is more accurate than codegen-local
        // inference through locals whose ty was frozen at declaration.
        // Phase 17.H1: look up with span's file_id, falling back to
        // codegen's current_file_id when the span is unstamped (0). TC
        // stamped expr_types keys with either the span's own file_id or
        // its own current_file_id under the identical rule, so the two
        // sides stay consistent.
        let file_id = if expr.span.file_id != 0 {
            expr.span.file_id
        } else {
            self.current_file_id
        };
        let span_key = (file_id, expr.span.start, expr.span.end);
        // Phase 17.H1 fallback: when serial TC ran, all expr_types were
        // stamped with the driver's single current_file_id while codegen
        // walks modules with distinct per-module file_ids. In that case
        // the exact (file_id, start, end) key won't match, but a unique
        // (*, start, end) entry often exists. Only promote when exactly
        // one such entry matches, to avoid cross-module span-bleed.
        let exact_tc_ty = self.expr_types.get(&span_key).cloned();
        let tc_lookup_is_exact = exact_tc_ty.is_some();
        let fallback_tc_ty = if exact_tc_ty.is_none() {
            let mut iter = self
                .expr_types
                .iter()
                .filter(|((_, s, e), _)| *s == expr.span.start && *e == expr.span.end);
            let first = iter.next();
            let second = iter.next();
            match (first, second) {
                (Some((_, ty)), None) => Some(ty.clone()),
                _ => None,
            }
        } else {
            None
        };
        if let Some(tc_ty_owned) = exact_tc_ty.or(fallback_tc_ty) {
            let tc_ty = &tc_ty_owned;
            // Phase 6.30.2: skip upgrade when tc_ty carries unresolved inference
            // vars (TC can leave Var(n) in expr_types when later unification is
            // not propagated back). The local codegen inference — which walks
            // through concrete function return types — is strictly more useful
            // than a Var. Otherwise alloca types freeze to the base name and
            // produce IR type mismatches vs the mangled call-site result.
            if contains_unresolved_var(tc_ty) {
                return local;
            }
            // Phase 16 A2: Span is (start, end) byte-offset-only with no file_id,
            // so in cross-module builds different files can share the same span
            // key and TC's stored type for one file bleeds into another. Refuse
            // to upgrade when the node shape of `expr` cannot possibly match
            // `tc_ty`. Without this guard, an Int literal whose span collides
            // with a Vec<f32> literal elsewhere gets promoted to Vec<f32>, which
            // then poisons the enclosing `let` alloca.
            if !expr_shape_matches_type(&expr.node, tc_ty) {
                return local;
            }
            // Phase E: for an Ident, trust the codegen-local binding when it
            // carries a concrete *narrow* primitive type (not plain I64,
            // which is the default erased type that TC often refines).
            // This catches the cross-module span bleed case where
            // body_size (locally U64) was being promoted to Vec<u8>
            // because a span-colliding expression elsewhere had that type.
            if let Expr::Ident(name) = &expr.node {
                if let Some(local_var) = self.fn_ctx.locals.get(name) {
                    let is_narrow_primitive = matches!(
                        local_var.ty,
                        ResolvedType::I8
                            | ResolvedType::I16
                            | ResolvedType::I32
                            | ResolvedType::I128
                            | ResolvedType::U8
                            | ResolvedType::U16
                            | ResolvedType::U32
                            | ResolvedType::U64
                            | ResolvedType::U128
                            | ResolvedType::F32
                            | ResolvedType::F64
                            | ResolvedType::Bool
                            | ResolvedType::Str
                    );
                    if is_narrow_primitive {
                        return local_var.ty.clone();
                    }
                    // Phase E: local I64 + TC Named/Vec (generic container)
                    // is almost always span-bleed from another module. Trust
                    // local unless TC refines to a tuple/named type that
                    // matches the local's semantic (rare — TC usually agrees
                    // on tuple shape when it's legitimate).
                    //
                    // Guard: block upgrade from I64 → Named{Vec,..} when the
                    // local's declared type is I64 — a legitimate Vec<T>
                    // binding would have been stored as Named{Vec,..} from
                    // the start, not I64.
                    if matches!(local_var.ty, ResolvedType::I64)
                        && !tc_lookup_is_exact
                        && is_span_bleed_prone_container(tc_ty)
                    {
                        return local_var.ty.clone();
                    }
                }
            }
            // Only upgrade when TC has strictly more info than local.
            let should_upgrade = match (&local, tc_ty) {
                (ResolvedType::I64, ResolvedType::Tuple(_)) => true,
                (ResolvedType::I64, ResolvedType::Named { generics, .. })
                    if !generics.is_empty() =>
                {
                    true
                }
                (
                    ResolvedType::I64,
                    ResolvedType::Ref(inner)
                    | ResolvedType::RefMut(inner)
                    | ResolvedType::Pointer(inner),
                ) if matches!(
                    inner.as_ref(),
                    ResolvedType::Named { generics, .. } if !generics.is_empty()
                ) =>
                {
                    true
                }
                // Phase 6.27b+: local Unit but TC has concrete type.
                // Happens when codegen-local infer returns Unit for method
                // calls it can't resolve (cross-module, generic receivers).
                (ResolvedType::Unit, tc) if !matches!(tc, ResolvedType::Unit) => true,
                // Package/import builds can leave codegen-local inference at
                // `Vec<T>` or bare `T` while TC expr_types has already
                // resolved the slot from later push/get usage. Use the TC
                // concrete type when it is the same outer shape with only
                // generic/var holes filled in.
                (local_ty, tc) if unresolved_generic_can_upgrade_to(local_ty, tc) => true,
                (
                    ResolvedType::Named {
                        name: l_name,
                        generics: l_gen,
                    },
                    ResolvedType::Named {
                        name: r_name,
                        generics: r_gen,
                    },
                ) if tc_lookup_is_exact
                    && l_name == r_name
                    && l_gen.is_empty()
                    && !r_gen.is_empty()
                    && !contains_unresolved_generic_or_var(tc_ty) =>
                {
                    true
                }
                // Exact TC info disambiguates bare unit enum variants such as
                // `None` when multiple enums expose the same variant name. Local
                // codegen inference can only pick one registry entry; TC has the
                // surrounding assignment/return constraints.
                (ResolvedType::Named { .. }, ResolvedType::Named { .. })
                    if tc_lookup_is_exact
                        && matches!(&expr.node, Expr::Ident(name) if self.is_unit_enum_variant(name))
                        && !contains_unresolved_generic_or_var(tc_ty) =>
                {
                    true
                }
                // local says Vec<I64> (generic erased) but TC says Vec<Tuple<..>>
                (
                    ResolvedType::Named {
                        name: l_name,
                        generics: l_gen,
                    },
                    ResolvedType::Named {
                        name: r_name,
                        generics: r_gen,
                    },
                ) if l_name == r_name
                    && l_gen.len() == r_gen.len()
                    && l_gen.iter().all(|g| matches!(g, ResolvedType::I64))
                    && r_gen.iter().any(|g| !matches!(g, ResolvedType::I64)) =>
                {
                    true
                }
                // local says Ref(Vec<I64>) but TC says Ref(Vec<Tuple>)
                (ResolvedType::Ref(l_inner), ResolvedType::Ref(r_inner))
                | (ResolvedType::RefMut(l_inner), ResolvedType::RefMut(r_inner)) => {
                    match (l_inner.as_ref(), r_inner.as_ref()) {
                        (
                            ResolvedType::Named {
                                name: l_n,
                                generics: l_g,
                            },
                            ResolvedType::Named {
                                name: r_n,
                                generics: r_g,
                            },
                        ) if l_n == r_n
                            && l_g.len() == r_g.len()
                            && l_g.iter().all(|g| matches!(g, ResolvedType::I64))
                            && r_g.iter().any(|g| !matches!(g, ResolvedType::I64)) =>
                        {
                            true
                        }
                        _ => false,
                    }
                }
                _ => false,
            };
            if should_upgrade {
                return tc_ty.clone();
            }
        }
        local
    }

    /// Get the TC-resolved type for an expression, if available.
    /// This is more accurate than infer_expr_type but may not be available for all expressions.
    /// Use for targeted lookups where TC accuracy is critical (e.g., generic type resolution).
    #[allow(dead_code)]
    pub(crate) fn tc_expr_type(&self, expr: &Spanned<Expr>) -> Option<&ResolvedType> {
        let file_id = if expr.span.file_id != 0 {
            expr.span.file_id
        } else {
            self.current_file_id
        };
        let span_key = (file_id, expr.span.start, expr.span.end);
        if let Some(ty) = self.expr_types.get(&span_key) {
            return Some(ty);
        }
        // Phase 17.H1 fallback: serial TC path stores under a single file_id.
        // Match (start, end) only if unique, otherwise bail out.
        let mut iter = self
            .expr_types
            .iter()
            .filter(|((_, s, e), _)| *s == expr.span.start && *e == expr.span.end);
        let first = iter.next();
        let second = iter.next();
        match (first, second) {
            (Some((_, ty)), None) => Some(ty),
            _ => None,
        }
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
                    if let Some(expected_ty) = self.expected_enum_type_for_variant(
                        name,
                        self.fn_ctx.expected_expr_types.last(),
                    ) {
                        return expected_ty;
                    }
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
                    if let Some(expected_ty) = self.expected_enum_type_for_variant(
                        fn_name,
                        self.fn_ctx.expected_expr_types.last(),
                    ) {
                        return expected_ty;
                    }
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
                            if let Some(Spanned {
                                node: Expr::Ident(ident),
                                ..
                            }) = args.get(1)
                            {
                                if let Some(concrete) = self.get_generic_substitution(ident) {
                                    return concrete;
                                }
                            }
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
                            if fn_info.signature.is_async {
                                return ResolvedType::Future(Box::new(ret_ty));
                            }
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
                        let ret_ty = sig.ret.clone();
                        if sig.is_async {
                            return ResolvedType::Future(Box::new(ret_ty));
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
                ResolvedType::Named { name, generics } if name == "Box" && generics.len() == 1 => {
                    generics[0].clone()
                }
                // Mirror the type checker and deref codegen: when a previous
                // deref already materialized a value, an extra `*` is a no-op.
                other => other,
            },
            Expr::StructLit {
                name,
                fields,
                enum_name,
            } => {
                // Enum struct variant literal: `EnumName.Variant { ... }`.
                // The whole expression produces a value of the enum type, not of
                // a standalone struct named after the variant.
                if let Some(en) = enum_name {
                    if self.types.enums.contains_key(en) {
                        return ResolvedType::Named {
                            name: en.clone(),
                            generics: vec![],
                        };
                    }
                }
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
                    // Slice expression `arr[a..b]` produces a slice fat
                    // pointer `{ i8*, i64 }` in text-IR codegen. Returning
                    // `Slice(elem)` keeps the type tag in sync with the
                    // emitted value, so downstream `s[i]` indexing takes
                    // the `is_fat_ptr=true` branch (extractvalue + GEP).
                    // Previously this returned `Pointer(elem)`, causing the
                    // index path to GEP straight into the struct value and
                    // produce a "{ ptr, i64 } expected ptr" verifier error.
                    let inner_ty = self.infer_expr_type(inner);
                    match inner_ty {
                        ResolvedType::Pointer(elem) => ResolvedType::Slice(elem),
                        ResolvedType::Array(elem) => ResolvedType::Slice(elem),
                        ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                            ResolvedType::Slice(elem)
                        }
                        _ => ResolvedType::Slice(Box::new(ResolvedType::I64)),
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
                        "len" | "charAt" | "char_at" | "byte_at" | "indexOf" | "contains"
                        | "startsWith" | "endsWith" | "isEmpty" | "is_empty" | "starts_with"
                        | "ends_with" => ResolvedType::I64,
                        "substring" | "push_str" | "to_uppercase" | "to_lowercase" | "trim"
                        | "to_string" | "clone" => ResolvedType::Str,
                        "as_bytes" | "into_bytes" => ResolvedType::Named {
                            name: "Vec".to_string(),
                            generics: vec![ResolvedType::U8],
                        },
                        "split" | "lines" | "split_whitespace" => ResolvedType::Named {
                            name: "Vec".to_string(),
                            generics: vec![ResolvedType::Str],
                        },
                        "parse_i64" | "parse_u64" | "parse_i32" | "parse_u32" => {
                            ResolvedType::Result(
                                Box::new(ResolvedType::I64),
                                Box::new(ResolvedType::Named {
                                    name: "VaisError".to_string(),
                                    generics: vec![],
                                }),
                            )
                        }
                        "parse_f64" | "parse_f32" => ResolvedType::Result(
                            Box::new(ResolvedType::F64),
                            Box::new(ResolvedType::Named {
                                name: "VaisError".to_string(),
                                generics: vec![],
                            }),
                        ),
                        _ => ResolvedType::I64,
                    };
                }

                if matches!(
                    recv_type,
                    ResolvedType::I8
                        | ResolvedType::U8
                        | ResolvedType::I16
                        | ResolvedType::U16
                        | ResolvedType::I32
                        | ResolvedType::U32
                        | ResolvedType::I64
                        | ResolvedType::U64
                        | ResolvedType::F32
                        | ResolvedType::F64
                ) && matches!(
                    method.node.as_str(),
                    "to_le_bytes" | "to_be_bytes" | "to_ne_bytes"
                ) {
                    return ResolvedType::Named {
                        name: "Vec".to_string(),
                        generics: vec![ResolvedType::U8],
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
                        // Option-returning accessors: without this, `let opt := v.get_opt(i)`
                        // falls back to I64, takes the SSA branch in stmt.rs, and downstream
                        // `M opt { Some(..) => .. }` GEPs the %Option value as if it were a ptr.
                        "get_opt" | "pop_opt" | "first_opt" | "last_opt" => ResolvedType::Named {
                            name: "Option".to_string(),
                            generics: vec![elem_ty],
                        },
                        "clone" => recv_type,
                        "data" => ResolvedType::I64,
                        _ => ResolvedType::I64,
                    };
                }

                // Look up registered function signature first — this is the ground truth
                // from the type checker and always takes priority over hardcoded heuristics.
                // Unwrap Ref/RefMut to get the inner Named type
                let inner_recv_base = match &recv_type {
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
                    other => other,
                };
                let guard_forward_inner = match inner_recv_base {
                    ResolvedType::Named { name, generics }
                        if matches!(
                            name.as_str(),
                            "MutexGuard" | "RwLockReadGuard" | "RwLockWriteGuard"
                        ) && !generics.is_empty() =>
                    {
                        let guard_method_arity = match method.node.as_str() {
                            "new" => Some(1),
                            "get" | "unlock" => Some(0),
                            "set" => Some(1),
                            _ => None,
                        };
                        if guard_method_arity == Some(args.len()) {
                            None
                        } else {
                            Some(generics[0].clone())
                        }
                    }
                    _ => None,
                };
                let effective_inner_recv_owned;
                let inner_recv = if let Some(inner) = &guard_forward_inner {
                    effective_inner_recv_owned = inner.clone();
                    &effective_inner_recv_owned
                } else {
                    inner_recv_base
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
                if let ResolvedType::Named { name, generics } = inner_recv {
                    if name == "Option" {
                        let generics = if let ResolvedType::Named { generics, .. } = inner_recv {
                            generics.clone()
                        } else {
                            vec![]
                        };
                        let ok_ty = generics.first().cloned().unwrap_or(ResolvedType::I64);
                        let err_ty = args.first().map(|arg| self.infer_expr_type(arg)).unwrap_or(
                            ResolvedType::Named {
                                name: "VaisError".to_string(),
                                generics: vec![],
                            },
                        );
                        return match method.node.as_str() {
                            "is_some" | "is_none" => ResolvedType::Bool,
                            "unwrap" => ok_ty,
                            "unwrap_or" | "unwrap_or_default" | "unwrap_or_else" => ok_ty,
                            "ok_or" | "ok_or_else" => ResolvedType::Named {
                                name: "Result".to_string(),
                                generics: vec![ok_ty, err_ty],
                            },
                            "as_ref" => ResolvedType::Named {
                                name: "Option".to_string(),
                                generics: vec![ResolvedType::Ref(Box::new(ok_ty))],
                            },
                            "clone" => recv_type,
                            _ => ResolvedType::I64,
                        };
                    }

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

                    if name == "Mutex" {
                        let guard_ty = ResolvedType::Named {
                            name: "MutexGuard".to_string(),
                            generics: generics.clone(),
                        };
                        return match method.node.as_str() {
                            "lock" => guard_ty,
                            "try_lock" => ResolvedType::Optional(Box::new(guard_ty)),
                            _ => ResolvedType::I64,
                        };
                    }

                    if name == "RwLock" {
                        let read_guard_ty = ResolvedType::Named {
                            name: "RwLockReadGuard".to_string(),
                            generics: generics.clone(),
                        };
                        let write_guard_ty = ResolvedType::Named {
                            name: "RwLockWriteGuard".to_string(),
                            generics: generics.clone(),
                        };
                        return match method.node.as_str() {
                            "read" | "read_lock" => read_guard_ty,
                            "write" | "write_lock" => write_guard_ty,
                            "try_read" => ResolvedType::Optional(Box::new(read_guard_ty)),
                            "try_write" => ResolvedType::Optional(Box::new(write_guard_ty)),
                            _ => ResolvedType::I64,
                        };
                    }

                    // HashMap<K, V> / StringMap<V> method return types.
                    // Keeping these in sync with the stdlib signatures avoids the
                    // I64 fallback that made `map.keys()[i]` trigger C003 in codegen.
                    if name == "HashMap" || name == "StringMap" {
                        let generics = if let ResolvedType::Named { generics, .. } = inner_recv {
                            generics.clone()
                        } else {
                            vec![]
                        };
                        let key_ty = generics.get(0).cloned().unwrap_or(ResolvedType::Str);
                        let val_ty = generics
                            .get(1)
                            .cloned()
                            .or_else(|| generics.get(0).cloned())
                            .unwrap_or(ResolvedType::I64);
                        return match method.node.as_str() {
                            "keys" => ResolvedType::Named {
                                name: "Vec".to_string(),
                                generics: vec![key_ty],
                            },
                            "values" => ResolvedType::Named {
                                name: "Vec".to_string(),
                                generics: vec![val_ty],
                            },
                            "get" => ResolvedType::Optional(Box::new(ResolvedType::Ref(Box::new(
                                val_ty,
                            )))),
                            "get_mut" => ResolvedType::Optional(Box::new(ResolvedType::RefMut(
                                Box::new(val_ty),
                            ))),
                            "insert" | "remove" => ResolvedType::Optional(Box::new(val_ty)),
                            "contains_key" | "is_empty" => ResolvedType::Bool,
                            "len" | "capacity" => ResolvedType::I64,
                            "clear" => ResolvedType::Unit,
                            _ => ResolvedType::I64,
                        };
                    }
                }

                // Optional<T> method inference — must come before the cross-module
                // function search, otherwise `.ok_or()` would fall through to the
                // I64 fallback and codegen-level ? / field-access would misbehave.
                if let ResolvedType::Optional(ref inner) = recv_type {
                    let err_ty = args.first().map(|arg| self.infer_expr_type(arg)).unwrap_or(
                        ResolvedType::Named {
                            name: "VaisError".to_string(),
                            generics: vec![],
                        },
                    );
                    return match method.node.as_str() {
                        "is_some" | "is_none" => ResolvedType::Bool,
                        "unwrap" => (**inner).clone(),
                        "unwrap_or" | "unwrap_or_default" => (**inner).clone(),
                        "unwrap_or_else" => (**inner).clone(),
                        "ok_or" | "ok_or_else" => ResolvedType::Named {
                            name: "Result".to_string(),
                            generics: vec![(**inner).clone(), err_ty],
                        },
                        "as_ref" => ResolvedType::Optional(Box::new(ResolvedType::Ref(Box::new(
                            (**inner).clone(),
                        )))),
                        "clone" => recv_type.clone(),
                        _ => ResolvedType::I64,
                    };
                }
                // Result<T,E> method inference
                if let ResolvedType::Result(ref ok_ty, ref err_ty) = recv_type {
                    return match method.node.as_str() {
                        "is_ok" | "is_err" => ResolvedType::Bool,
                        "unwrap" => (**ok_ty).clone(),
                        "unwrap_err" => (**err_ty).clone(),
                        "ok" => ResolvedType::Optional(Box::new((**ok_ty).clone())),
                        "err" => ResolvedType::Optional(Box::new((**err_ty).clone())),
                        "clone" => recv_type.clone(),
                        _ => ResolvedType::I64,
                    };
                }

                // clone on any type returns the same type
                if method.node == "clone" {
                    if guard_forward_inner.is_some() {
                        return inner_recv.clone();
                    }
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
                if matches!(
                    (type_name.node.as_str(), method.node.as_str()),
                    ("Option", "Some") | ("Option", "None") | ("Result", "Ok") | ("Result", "Err")
                ) {
                    if let Some(expected_ty) = self.expected_enum_type_for_variant(
                        &method.node,
                        self.fn_ctx.expected_expr_types.last(),
                    ) {
                        return expected_ty;
                    }
                    let generics = match (type_name.node.as_str(), method.node.as_str()) {
                        ("Option", "Some") => args
                            .first()
                            .map(|arg| vec![self.infer_expr_type(arg)])
                            .unwrap_or_default(),
                        ("Result", "Ok") => args
                            .first()
                            .map(|arg| vec![self.infer_expr_type(arg), ResolvedType::I64])
                            .unwrap_or_default(),
                        ("Result", "Err") => args
                            .first()
                            .map(|arg| vec![ResolvedType::I64, self.infer_expr_type(arg)])
                            .unwrap_or_default(),
                        _ => vec![],
                    };
                    return ResolvedType::Named {
                        name: type_name.node.clone(),
                        generics,
                    };
                }
                if self
                    .types
                    .enums
                    .get(type_name.node.as_str())
                    .map(|enum_info| enum_info.variants.iter().any(|v| v.name == method.node))
                    .unwrap_or(false)
                {
                    if let Some(expected_ty) = self.expected_enum_type_for_variant(
                        &method.node,
                        self.fn_ctx.expected_expr_types.last(),
                    ) {
                        return expected_ty;
                    }
                    return ResolvedType::Named {
                        name: type_name.node.clone(),
                        generics: vec![],
                    };
                }
                if matches!(
                    (type_name.node.as_str(), method.node.as_str()),
                    ("Vec", "new")
                        | ("Vec", "with_capacity")
                        | ("HashMap", "new")
                        | ("HashMap", "with_capacity")
                        | ("HashSet", "new")
                        | ("HashSet", "with_capacity")
                ) {
                    if let Some(expected) = self.fn_ctx.expected_expr_types.last() {
                        if let ResolvedType::Named { name, .. } = expected {
                            if name == &type_name.node {
                                return expected.clone();
                            }
                        }
                    }
                    return ResolvedType::Named {
                        name: type_name.node.clone(),
                        generics: vec![],
                    };
                }
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

                // Tuple field access: `.0`, `.1`, ... return the element type so
                // a tuple-of-structs payload doesn't collapse to i64 downstream
                // (Result/enum tuple binding → `pair.1` storage).
                if let ResolvedType::Tuple(elem_types) = inner_type {
                    if let Ok(idx) = field.node.parse::<usize>() {
                        if let Some(elem_ty) = elem_types.get(idx) {
                            return elem_ty.clone();
                        }
                    }
                }

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
                    // Enum namespace access: `EnumName.Variant` — if `struct_name`
                    // is an enum and `field.node` is one of its variants, the whole
                    // expression produces a value of the enum type.
                    for candidate in &names_to_try {
                        if let Some(enum_info) = self.types.enums.get(*candidate) {
                            if enum_info.variants.iter().any(|v| v.name == field.node) {
                                return ResolvedType::Named {
                                    name: enum_info.name.clone(),
                                    generics: vec![],
                                };
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
            Expr::Unary { op, expr: inner } => match op {
                vais_ast::UnaryOp::Not => ResolvedType::Bool,
                _ => self.infer_expr_type(inner),
            },
            Expr::Ternary { then, .. } => {
                // Ternary returns the type of its then branch
                self.infer_expr_type(then)
            }
            Expr::If { then, .. } => {
                // If-else returns the type of its then block
                self.infer_block_type(then)
            }
            Expr::Match { expr, arms } => {
                // Match returns the first informative arm type. Pattern-bound
                // identifiers such as `Ok(v) => v` need the matched enum's
                // concrete payload type, not the fallback Ident/i64 type.
                let match_type = self.infer_expr_type(expr);
                let mut fallback = ResolvedType::I64;
                for arm in arms {
                    let ty = self.infer_match_arm_result_type(arm, &match_type);
                    match ty {
                        ResolvedType::Unknown | ResolvedType::Never => {}
                        ResolvedType::I64 => {
                            fallback = ResolvedType::I64;
                        }
                        other => return other,
                    }
                }
                fallback
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
            Expr::Await(inner) => {
                let inner_ty = self.infer_expr_type(inner);
                // Await unwraps Future<T> to T
                match inner_ty {
                    ResolvedType::Future(t) => *t,
                    ResolvedType::Unit => {
                        // Await on void async function — the call returned Unit because
                        // the async function wasn't found in types.functions. Treat as Unit.
                        ResolvedType::Unit
                    }
                    ResolvedType::I64 => {
                        // Await on async function whose type fell through to I64 fallback.
                        // The actual async create function returns i64 (state pointer).
                        // The await codegen handles this correctly — just return I64.
                        ResolvedType::I64
                    }
                    other => {
                        // Non-fatal: await on non-Future type, possibly cross-module async.
                        // Return the type as-is rather than panicking.
                        eprintln!(
                            "[WARN] await on non-Future type `{other}` — treating as passthrough"
                        );
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
            Expr::Assign { target, value, .. } | Expr::AssignOp { target, value, .. } => {
                // Compound assignments like `x += 1` produce a value whose
                // type matches the *target* (the stored-into variable), not
                // the literal right-hand side. Fall back to the RHS type
                // only if the target is unknown.
                let target_ty = self.infer_expr_type(target);
                if !matches!(target_ty, ResolvedType::I64) {
                    return target_ty;
                }
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
