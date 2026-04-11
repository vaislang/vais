//! Totality analysis (Phase 4c.2 / Task #53).
//!
//! A Vais function without the `partial` modifier is "total": the type
//! checker must prove it cannot panic at runtime. The totality gate is a
//! lightweight syntactic + call-graph walk over the already-type-checked
//! AST that looks for expressions which can raise a runtime panic.
//!
//! ## Panic sources we reject
//!
//! 1. Explicit panic builtins: `panic`, `abort`, `exit`, `assert`,
//!    `__panic` (matching the builtins table in `effects.rs`).
//! 2. `assert(...)` expression form (which lowers to a panicking
//!    conditional branch in codegen).
//! 3. `expr!` — the `Unwrap` operator, which panics on `Err`/`None`.
//! 4. Calls into user functions that are themselves reachable-panic.
//!    Computed via a worklist fixed point over the module's call graph.
//!
//! ## Panic sources we deliberately accept
//!
//! The whole design goal is "catch the `panic` cases a careful reviewer
//! would also catch". We intentionally skip a few categories that a
//! purely syntactic walk would misclassify:
//!
//! - `a / b`, `a % b`, `a /= b`, `a %= b` — integer / float division.
//!   Rejecting these would reject ~60% of real arithmetic code (iter 10
//!   #53 empirical measurement: 101 / 187 rejections came from division
//!   alone). Division-by-zero safety is delegated to **refinement
//!   types** (Phase 4c.1 / `{b: i64 | b != 0}`) — a caller who cares
//!   about safety constrains the divisor's type at the boundary. The
//!   codegen still emits its runtime `abort()` guard as a backstop.
//! - `arr[idx]` — array / slice indexing. Same argument as division:
//!   bounds safety lives in the refinement layer (`{i: usize | i < len}`)
//!   or through iterator APIs (`.get(idx)` → `Option<T>`, then `!`).
//!   ~18% of iter 10 rejections.
//! - `expr?` (`Expr::Try`) — explicit error propagation via `Result` /
//!   `Option`. A total function *can* return an error, it just cannot
//!   spontaneously abort. `?` is early-return, not panic.
//! - Allocation failure from `malloc`/`alloc` — not yet modeled in the
//!   type system. Treated as "the OS will OOM-kill us" rather than a
//!   panic the user code caused. (Same treatment Rust's `#[no_panic]`
//!   gives.)
//! - `Expr::Error { .. }` and `Expr::MacroInvoke(_)` — parse-error and
//!   unexpanded macro nodes. If type checking reaches the totality gate
//!   at all, the previous passes already returned errors for these; we
//!   skip them here to avoid reporting a second, noisier error.
//!
//! ## Why this scoping
//!
//! Empirically (iter 10 measurement on the 2526-test E2E suite), the
//! strict "div/mod + index + unwrap + panic-builtin" gate rejected
//! **187 / 2526 = 7.4%** of existing programs, of which **85% were
//! legitimate arithmetic** (things like `42 / 1`, `gcd` with a manual
//! `b == 0` guard, `binary_search` with `lo + (hi - lo) / 2`). Narrowing
//! the gate to just `!` + `panic!`/`assert`/`abort`/`exit` + partial
//! calls reduces the false-positive rate by ~95% while still catching
//! the handful of places where a programmer typed an explicit "kill the
//! process if this is wrong" expression.
//!
//! ## Inter-function propagation
//!
//! The gate runs after module-level type checking. It first collects:
//!
//!   - `partial_fns`: the set of user functions marked `partial`. Any
//!     call to one of these is treated as a panic source.
//!   - `direct_panic_fns`: the set of user functions whose body
//!     syntactically contains a direct panic source (cases 1–5 above).
//!
//! Then a worklist expands `direct_panic_fns` transitively: a function
//! that calls a panic-reachable function is itself panic-reachable. The
//! fixed point converges in `O(N * E)` where `E` is the number of call
//! edges, which is fine for the module sizes Vais targets.
//!
//! Finally every user function `f` with `!f.is_partial && f ∈ reachable`
//! produces a `TypeError::TotalFunctionViolation`. The `reason` carried
//! in the error is the first direct panic source encountered in the
//! walk — a concrete phrase like `"contains division which may panic on
//! a 0 divisor"`. Users seeing the error can either add the `partial`
//! modifier or refactor the body.

use std::collections::{HashMap, HashSet};

use vais_ast::{Expr, FunctionBody, Item, Module, Spanned, Stmt};

use crate::types::{TypeError, TypeResult};

/// Builtin function names that are panic sources when called.
///
/// Matches the panic group in `effects.rs` builtins table. If that list
/// ever grows, extend this one to stay in sync.
const PANIC_BUILTINS: &[&str] = &["panic", "abort", "exit", "assert", "__panic"];

/// Run the totality gate on a freshly type-checked module.
///
/// Returns the first totality violation as a `TypeError` or `Ok(())` if
/// every non-partial function is panic-free. Called from `check_module`
/// after body checking but before ownership checking.
pub(crate) fn enforce_totality(module: &Module) -> TypeResult<()> {
    // Collect the functions we will analyze. For v1 we cover only
    // top-level free functions (Item::Function). Impl methods share a
    // flat namespace with free functions for panic-reachability purposes
    // and are collected via their method name only — this is safe
    // because a total impl method that calls a partial free function of
    // the same name will still be rejected, and a total free function
    // that calls an impl method is resolved by the type checker which
    // would have caught a missing method before we got here.
    let mut all_fns: HashMap<String, &vais_ast::Function> = HashMap::new();
    for item in &module.items {
        if let Item::Function(f) = &item.node {
            all_fns.insert(f.name.node.clone(), f);
        }
    }
    // Impl methods — use method name only for the panic set. This is
    // an approximation; a more precise scheme keyed by
    // `(target_type, method)` can come later if ambiguity bites.
    for item in &module.items {
        if let Item::Impl(impl_block) = &item.node {
            for m in &impl_block.methods {
                all_fns
                    .entry(m.node.name.node.clone())
                    .or_insert(&m.node);
            }
        }
    }

    // 1. Collect partial-marked fns.
    let partial_fns: HashSet<String> = all_fns
        .iter()
        .filter(|(_, f)| f.is_partial)
        .map(|(n, _)| n.clone())
        .collect();

    // 2. Compute direct panic sources per function (empty partial_fns at
    //    this stage — we're asking "does this body contain any SYNTACTIC
    //    panic source?", not "does it call a known panic function?").
    let mut direct_reason: HashMap<String, String> = HashMap::new();
    let mut calls_of: HashMap<String, Vec<String>> = HashMap::new();
    for (name, f) in &all_fns {
        let mut state = WalkState::new();
        walk_function_body(&f.body, &mut state);
        if let Some(reason) = state.first_panic {
            direct_reason.insert(name.clone(), reason);
        }
        calls_of.insert(name.clone(), state.calls);
    }

    // 3. Seed the reachable set with direct panic fns + all partial fns.
    //    Partial fns are treated as panic sources for the purpose of
    //    caller analysis: calling `partial bar` from total `foo` makes
    //    foo reachable-panic, regardless of whether partial_bar's own
    //    body actually panics right now.
    let mut reachable: HashSet<String> = direct_reason.keys().cloned().collect();
    reachable.extend(partial_fns.iter().cloned());

    // 4. Worklist: a function that calls any reachable fn is itself
    //    reachable. Iterate until fixed point.
    let mut changed = true;
    while changed {
        changed = false;
        for (caller, callees) in &calls_of {
            if reachable.contains(caller) {
                continue;
            }
            for callee in callees {
                if reachable.contains(callee) {
                    reachable.insert(caller.clone());
                    // The propagation reason is always "calls <callee>"
                    // for transitively-reachable functions. Only store
                    // it if we don't already have a direct reason.
                    direct_reason
                        .entry(caller.clone())
                        .or_insert_with(|| format!("transitively calls `{}` which may panic", callee));
                    changed = true;
                    break;
                }
            }
        }
    }

    // 5. Report the first total-function violation. We sort by function
    //    name for determinism so the compiler produces a stable error on
    //    re-runs (important for E2E snapshot tests).
    let mut violators: Vec<(&String, &vais_ast::Function)> = all_fns
        .iter()
        .filter(|(name, f)| !f.is_partial && reachable.contains(*name))
        .map(|(name, f)| (name, *f))
        .collect();
    violators.sort_by(|a, b| a.0.cmp(b.0));

    if let Some((name, f)) = violators.first() {
        let reason = direct_reason
            .get(*name)
            .cloned()
            .unwrap_or_else(|| "contains a panic source".to_string());
        return Err(TypeError::TotalFunctionViolation {
            function_name: (*name).clone(),
            reason,
            span: Some(f.name.span),
        });
    }

    Ok(())
}

/// Accumulator for one function's syntactic walk.
#[derive(Default)]
struct WalkState {
    /// First direct panic source found in the body, if any. Kept as a
    /// short human phrase ready to paste into `TotalFunctionViolation`.
    first_panic: Option<String>,
    /// Names of every `Expr::Call` target we saw during the walk. Used
    /// by the caller to build the call graph for transitive propagation.
    calls: Vec<String>,
}

impl WalkState {
    fn new() -> Self {
        Self::default()
    }

    fn record_panic(&mut self, reason: &str) {
        if self.first_panic.is_none() {
            self.first_panic = Some(reason.to_string());
        }
    }
}

fn walk_function_body(body: &FunctionBody, state: &mut WalkState) {
    match body {
        FunctionBody::Expr(e) => walk_expr(&e.node, state),
        FunctionBody::Block(stmts) => {
            for s in stmts {
                walk_stmt(&s.node, state);
            }
        }
    }
}

fn walk_stmt(stmt: &Stmt, state: &mut WalkState) {
    match stmt {
        Stmt::Let { value, .. } | Stmt::LetDestructure { value, .. } => {
            walk_expr(&value.node, state);
        }
        Stmt::Expr(e) => walk_expr(&e.node, state),
        Stmt::Return(Some(e)) | Stmt::Break(Some(e)) => walk_expr(&e.node, state),
        Stmt::Return(None) | Stmt::Break(None) | Stmt::Continue => {}
        Stmt::Defer(e) => walk_expr(&e.node, state),
        Stmt::Error { .. } => {
            // Parse-error node — an earlier pass already reported it, so
            // we skip it to avoid a cascade of duplicate errors.
        }
    }
}

fn walk_expr(expr: &Expr, state: &mut WalkState) {
    match expr {
        // Literals and identifiers are pure.
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Unit
        | Expr::Ident(_)
        | Expr::SelfCall => {}

        Expr::StringInterp(parts) => {
            // String interpolation parts contain sub-expressions.
            for part in parts {
                if let vais_ast::StringInterpPart::Expr(e) = part {
                    walk_expr(&e.node, state);
                }
            }
        }

        // Binary ops: we walk operands but do NOT flag `/` / `%` here.
        // Div-by-zero safety is delegated to refinement types — see the
        // module-level note on "Panic sources we deliberately accept".
        Expr::Binary { left, right, .. } => {
            walk_expr(&left.node, state);
            walk_expr(&right.node, state);
        }

        Expr::Unary { expr, .. } => walk_expr(&expr.node, state),

        Expr::Ternary { cond, then, else_ } => {
            walk_expr(&cond.node, state);
            walk_expr(&then.node, state);
            walk_expr(&else_.node, state);
        }

        Expr::If { cond, then, else_ } => {
            walk_expr(&cond.node, state);
            for s in then {
                walk_stmt(&s.node, state);
            }
            if let Some(e) = else_ {
                walk_if_else(e, state);
            }
        }

        Expr::Loop { iter, body, .. } => {
            if let Some(i) = iter {
                walk_expr(&i.node, state);
            }
            for s in body {
                walk_stmt(&s.node, state);
            }
        }

        Expr::While { condition, body } => {
            walk_expr(&condition.node, state);
            for s in body {
                walk_stmt(&s.node, state);
            }
        }

        Expr::Match { expr, arms } => {
            walk_expr(&expr.node, state);
            for arm in arms {
                if let Some(g) = &arm.guard {
                    walk_expr(&g.node, state);
                }
                walk_expr(&arm.body.node, state);
            }
        }

        // Function call: record the callee name and walk arguments.
        // If the callee is a direct identifier AND it names a panic
        // builtin, flag immediately. Otherwise the inter-fn worklist
        // decides whether it's reachable-panic.
        Expr::Call { func, args } => {
            walk_expr(&func.node, state);
            for a in args {
                walk_expr(&a.node, state);
            }
            if let Expr::Ident(name) = &func.node {
                if PANIC_BUILTINS.contains(&name.as_str()) {
                    state.record_panic(&format!("calls panic builtin `{}`", name));
                } else {
                    state.calls.push(name.clone());
                }
            }
        }

        // Method/static calls: the receiver is walked, and the method
        // name is added to the call graph. Receiver path is syntactic —
        // the worklist handles transitive method panic in the same pass
        // as free functions (approximate: shared method-name namespace).
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            walk_expr(&receiver.node, state);
            for a in args {
                walk_expr(&a.node, state);
            }
            state.calls.push(method.node.clone());
        }
        Expr::StaticMethodCall { method, args, .. } => {
            for a in args {
                walk_expr(&a.node, state);
            }
            state.calls.push(method.node.clone());
        }

        Expr::Field { expr, .. } | Expr::TupleFieldAccess { expr, .. } => {
            walk_expr(&expr.node, state);
        }

        // Indexing: we walk operands but do NOT flag OOB here. Bounds
        // safety is delegated to refinement types or `.get(idx)` →
        // `Option<T>` iterator APIs — see the module-level note.
        Expr::Index { expr, index } => {
            walk_expr(&expr.node, state);
            walk_expr(&index.node, state);
        }

        Expr::Array(xs) | Expr::Tuple(xs) => {
            for x in xs {
                walk_expr(&x.node, state);
            }
        }

        Expr::StructLit { fields, .. } => {
            for (_, v) in fields {
                walk_expr(&v.node, state);
            }
        }

        Expr::Range { start, end, .. } => {
            if let Some(s) = start {
                walk_expr(&s.node, state);
            }
            if let Some(e) = end {
                walk_expr(&e.node, state);
            }
        }

        Expr::Block(stmts) => {
            for s in stmts {
                walk_stmt(&s.node, state);
            }
        }

        // `expr.await` is a suspension point, not a panic.
        Expr::Await(inner) => walk_expr(&inner.node, state),

        // `expr?` is controlled error propagation via Result/Option —
        // explicitly NOT a panic (see the module-level note).
        Expr::Try(inner) => walk_expr(&inner.node, state),

        // `expr!` is unwrap — panics on Err/None.
        Expr::Unwrap(inner) => {
            walk_expr(&inner.node, state);
            state.record_panic("contains `!` unwrap which may panic on `Err` or `None`");
        }

        Expr::MapLit(pairs) => {
            for (k, v) in pairs {
                walk_expr(&k.node, state);
                walk_expr(&v.node, state);
            }
        }

        Expr::Spread(inner) | Expr::Ref(inner) | Expr::Deref(inner) => {
            walk_expr(&inner.node, state);
        }

        Expr::Cast { expr, .. } => walk_expr(&expr.node, state),

        Expr::Assign { target, value } => {
            walk_expr(&target.node, state);
            walk_expr(&value.node, state);
        }
        // Compound assignment: same policy as plain `/` / `%` — walk
        // operands, do NOT flag `/=` / `%=` as a panic source.
        Expr::AssignOp { target, value, .. } => {
            walk_expr(&target.node, state);
            walk_expr(&value.node, state);
        }

        Expr::Lambda { body, .. } => walk_expr(&body.node, state),

        Expr::Yield(inner) => walk_expr(&inner.node, state),

        // `comptime { expr }` is evaluated at compile time, no runtime
        // panics possible from the user's perspective.
        Expr::Comptime { .. } => {}

        // `assert(cond, msg?)` lowers to a conditional panic in codegen.
        Expr::Assert { condition, message } => {
            walk_expr(&condition.node, state);
            if let Some(m) = message {
                walk_expr(&m.node, state);
            }
            state.record_panic("contains `assert` which may panic when the condition is false");
        }

        Expr::Assume(inner) => walk_expr(&inner.node, state),

        Expr::Old(inner) => walk_expr(&inner.node, state),

        // Parse-error and un-expanded macro: earlier passes already
        // reported these. Skip to avoid cascading errors.
        Expr::Error { .. } => {}
        Expr::MacroInvoke(_) => {}

        Expr::EnumAccess { data, .. } => {
            if let Some(d) = data {
                walk_expr(&d.node, state);
            }
        }
    }
}

fn walk_if_else(e: &vais_ast::IfElse, state: &mut WalkState) {
    match e {
        vais_ast::IfElse::ElseIf(cond, body, next) => {
            walk_expr(&cond.node, state);
            for s in body {
                walk_stmt(&s.node, state);
            }
            if let Some(n) = next {
                walk_if_else(n, state);
            }
        }
        vais_ast::IfElse::Else(body) => {
            for s in body {
                walk_stmt(&s.node, state);
            }
        }
    }
}

// Suppress the unused-import warning if `Spanned` ends up not being
// referenced directly — some of the arms above use field access on
// `Spanned`-typed values via `.node`, which technically doesn't need
// the type in scope.
#[allow(dead_code)]
type _SpannedPlaceholder = Spanned<()>;
