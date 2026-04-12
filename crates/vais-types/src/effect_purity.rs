//! Effect purity analysis (Phase 4c.3 / Task #54).
//!
//! A Vais function carrying the `pure`, `io`, or `alloc` prefix keyword
//! declares a ceiling on the side effects it may perform. The type
//! checker must prove that the function's body — plus every callee
//! transitively reachable from it — stays within that ceiling.
//!
//! ## Effect ceilings
//!
//! | prefix   | allowed body effects            | forbidden effects |
//! |----------|---------------------------------|-------------------|
//! | `pure`   | nothing (plus local arithmetic) | IO, Alloc         |
//! | `io`     | IO (plus pure body)             | Alloc             |
//! | `alloc`  | Alloc (plus pure body)          | IO                |
//!
//! Subtype rule (approved by the user in iter 11):
//! `pure ⊆ io`, `pure ⊆ alloc`, `io` and `alloc` are incomparable.
//!
//! A `pure` caller may only call other `pure` callees. An `io` caller
//! may call `pure` or `io` callees. An `alloc` caller may call `pure`
//! or `alloc` callees. `io` and `alloc` cannot call each other.
//!
//! Functions *without* a prefix keyword are inferred — they do not
//! participate in the gate directly, but their effect classification
//! (from their body) is carried forward to any declared caller that
//! invokes them, so a `pure F foo()` that calls a non-declared
//! `helper()` still fails if `helper` reaches `println`.
//!
//! ## What counts as IO / Alloc
//!
//! v1 (this module) detects **direct calls to known builtin functions**
//! plus transitive closure over the module call graph. Method calls on
//! an arbitrary receiver are ignored for now — detecting that
//! `Vec::new()` is an allocation requires proper method resolution and
//! lives in a follow-up task. This is deliberately narrow: iter 10
//! #53's empirical lesson is that a syntactic walk that tries to catch
//! too much rejects ~85% of legitimate code with false positives.
//!
//! ## Builtin classification
//!
//! Kept in sync with `effects.rs`'s builtins table. If that table grows
//! a new IO or Alloc function, add it to the matching constant below.
//!
//! ## Inter-function propagation
//!
//! Like the totality gate, this pass first classifies each function's
//! *direct* effects (from its syntactic body), then runs a worklist
//! fixed point over the call graph. A caller's transitive effect is
//! the union of its own direct effects and every callee's transitive
//! effects.
//!
//! Finally, every function `f` whose transitive effect set is *not* a
//! subset of its declared ceiling raises a
//! `TypeError::PurityViolation`. The violation carries the offending
//! effect and a short phrase pointing at the first source seen.
//!
//! ## What this pass does NOT do
//!
//! - It does not touch the existing `EffectInferrer` infrastructure in
//!   `effects.rs`. That module remains used by legacy
//!   `#[pure]`/`#[effect(...)]` attributes for backwards compatibility
//!   (see `EffectInferrer::get_declared_effects`). The keyword path
//!   lives here because iter 10 (Task #53) showed that re-wiring
//!   EffectInferrer is error-prone — totality.rs was written as a
//!   dedicated walker for the same reason.
//! - It does not enforce `Effect::Async` or `Effect::Panic`. Those are
//!   handled by the `A F` modifier and the `partial F` modifier
//!   respectively; effect prefixes are orthogonal.
//! - It skips imported items. We only enforce on code authored in the
//!   current compilation unit, matching the totality gate's scope
//!   rules (see `checker_module::mod::check_module`).

use std::collections::HashMap;

use vais_ast::{EffectPrefix, Expr, Function, FunctionBody, Item, Module, Stmt};

use crate::types::{TypeError, TypeResult};

/// Builtin function names that perform IO (stdout, file, socket, ...).
///
/// Kept in sync with `effects.rs::EffectInferrer::new()` IO + file-IO +
/// network-IO entries. Adding a new IO builtin there means adding it
/// here as well.
const IO_BUILTINS: &[&str] = &[
    // stdout / stderr
    "print", "println", "eprint", "eprintln", "puts", "putchar", "printf",
    // file IO
    "fopen", "fclose", "fread", "fwrite", "fseek", "ftell", "open", "close", "read", "write",
    "lseek",
    // network IO
    "socket", "bind", "listen", "accept", "connect", "send", "recv", "sendto", "recvfrom",
];

/// Builtin function names that perform heap allocation.
///
/// Kept in sync with `effects.rs` builtins table. Note that method
/// calls like `Vec::new()` are intentionally *not* tracked here — a
/// later task can plug in method resolution when the signature DB is
/// richer.
const ALLOC_BUILTINS: &[&str] = &["malloc", "calloc", "realloc", "free", "alloc", "dealloc"];

/// Effect classification tracked by this pass.
///
/// A function's transitive effect is the union of these flags. The
/// declared ceiling accepts a function iff the function's flags stay
/// within it (`pure` accepts only `{}`; `io` accepts `io`; `alloc`
/// accepts `alloc`; `io` and `alloc` are disjoint).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct EffectFlags {
    io: bool,
    alloc: bool,
}

impl EffectFlags {
    fn is_pure(&self) -> bool {
        !self.io && !self.alloc
    }

    fn union(&self, other: &Self) -> Self {
        Self {
            io: self.io || other.io,
            alloc: self.alloc || other.alloc,
        }
    }

    /// Is `self` within the ceiling permitted by `prefix`?
    fn satisfies(&self, prefix: EffectPrefix) -> bool {
        match prefix {
            EffectPrefix::Pure => self.is_pure(),
            // An `io`-declared function may do IO but must not allocate.
            EffectPrefix::Io => !self.alloc,
            // An `alloc`-declared function may allocate but must not do IO.
            EffectPrefix::Alloc => !self.io,
        }
    }
}

/// Bundle of state captured while walking one function body.
#[derive(Default)]
struct WalkState {
    flags: EffectFlags,
    /// First IO source seen (for error messages). A short human phrase.
    first_io_source: Option<String>,
    /// First Alloc source seen (for error messages).
    first_alloc_source: Option<String>,
    /// Names of every direct call target we encountered. Used to build
    /// the call graph for transitive propagation.
    calls: Vec<String>,
}

impl WalkState {
    fn record_io(&mut self, source: &str) {
        self.flags.io = true;
        if self.first_io_source.is_none() {
            self.first_io_source = Some(source.to_string());
        }
    }

    fn record_alloc(&mut self, source: &str) {
        self.flags.alloc = true;
        if self.first_alloc_source.is_none() {
            self.first_alloc_source = Some(source.to_string());
        }
    }
}

/// Run the effect-purity gate on a freshly type-checked module.
///
/// Returns the first purity violation as a `TypeError`, or `Ok(())` if
/// every declared-effect function stays within its ceiling. Called
/// from `check_module` right after the totality gate (Task #53) and
/// before ownership checking.
pub(crate) fn enforce_effect_purity(module: &Module) -> TypeResult<()> {
    // Collect every function we will analyze. We use the same flat
    // name-keyed map as totality.rs — impl methods share the free-
    // function namespace for propagation purposes. This is an
    // approximation; a precise call graph keyed by `(target, method)`
    // is possible once we see a concrete ambiguity.
    let mut all_fns: HashMap<String, &Function> = HashMap::new();
    for item in &module.items {
        if let Item::Function(f) = &item.node {
            all_fns.insert(f.name.node.clone(), f);
        }
    }
    for item in &module.items {
        if let Item::Impl(impl_block) = &item.node {
            for m in &impl_block.methods {
                all_fns.entry(m.node.name.node.clone()).or_insert(&m.node);
            }
        }
    }

    // Short-circuit: if no function in the module declares an effect
    // prefix, the gate has nothing to enforce. This is the common case
    // today (Task #54 has just landed — real user code has not yet
    // adopted the prefixes) and keeps the pass O(0) for the entire
    // 2539-test E2E baseline.
    let has_any_declared = all_fns
        .values()
        .any(|f| f.declared_effect.is_some());
    if !has_any_declared {
        return Ok(());
    }

    // 1. Walk each body syntactically, collecting direct effects and
    //    the call graph.
    let mut direct: HashMap<String, WalkState> = HashMap::new();
    for (name, f) in &all_fns {
        let mut state = WalkState::default();
        walk_function_body(&f.body, &mut state);
        direct.insert(name.clone(), state);
    }

    // 2. Worklist fixed point: a function's transitive effect is the
    //    union of its direct effect and every reachable callee's
    //    transitive effect. Iterate until nothing changes.
    let mut transitive: HashMap<String, EffectFlags> = direct
        .iter()
        .map(|(n, s)| (n.clone(), s.flags))
        .collect();
    let mut changed = true;
    while changed {
        changed = false;
        for (caller, state) in &direct {
            let mut flags = transitive[caller];
            for callee in &state.calls {
                if let Some(cf) = transitive.get(callee) {
                    let unioned = flags.union(cf);
                    if unioned != flags {
                        flags = unioned;
                    }
                }
            }
            if flags != transitive[caller] {
                transitive.insert(caller.clone(), flags);
                changed = true;
            }
        }
    }

    // 3. Report the first violation. Sort by function name for
    //    determinism so the compiler produces a stable error on
    //    re-runs (important for snapshot tests).
    let mut violators: Vec<(&String, &Function, EffectFlags, EffectPrefix)> = all_fns
        .iter()
        .filter_map(|(name, f)| {
            let prefix = f.declared_effect?;
            let flags = transitive.get(name).copied().unwrap_or_default();
            if flags.satisfies(prefix) {
                None
            } else {
                Some((name, *f, flags, prefix))
            }
        })
        .collect();
    violators.sort_by(|a, b| a.0.cmp(b.0));

    if let Some((name, f, flags, prefix)) = violators.first() {
        let ceiling = match prefix {
            EffectPrefix::Pure => "pure",
            EffectPrefix::Io => "io",
            EffectPrefix::Alloc => "alloc",
        };
        // Which effect broke the ceiling? Pure rejects both; io
        // rejects alloc; alloc rejects io.
        let offending = match prefix {
            EffectPrefix::Pure => {
                if flags.io {
                    "io"
                } else {
                    "alloc"
                }
            }
            EffectPrefix::Io => "alloc",
            EffectPrefix::Alloc => "io",
        };
        // Reach into the direct walk state for the first concrete
        // source — it gives users a specific phrase rather than a
        // bare "this function has io effect".
        let reason = match offending {
            "io" => direct
                .get(*name)
                .and_then(|s| s.first_io_source.clone())
                .unwrap_or_else(|| {
                    "transitively calls a function that performs io".to_string()
                }),
            "alloc" => direct
                .get(*name)
                .and_then(|s| s.first_alloc_source.clone())
                .unwrap_or_else(|| {
                    "transitively calls a function that allocates".to_string()
                }),
            _ => "performs a forbidden effect".to_string(),
        };
        return Err(TypeError::PurityViolation {
            callee: (*name).clone(),
            effects: format!(
                "declared `{ceiling} F {name}` but body has `{offending}` effect: {reason}",
                ceiling = ceiling,
                name = name,
                offending = offending,
                reason = reason,
            ),
            span: Some(f.name.span),
        });
    }

    Ok(())
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
            walk_expr(&value.node, state)
        }
        Stmt::Expr(e) => walk_expr(&e.node, state),
        Stmt::Return(Some(e)) | Stmt::Break(Some(e)) => walk_expr(&e.node, state),
        Stmt::Return(None) | Stmt::Break(None) | Stmt::Continue => {}
        Stmt::Defer(e) => walk_expr(&e.node, state),
        Stmt::Error { .. } => {
            // Parse error. Earlier pass already reported it.
        }
    }
}

fn walk_expr(expr: &Expr, state: &mut WalkState) {
    match expr {
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Unit
        | Expr::Ident(_)
        | Expr::SelfCall => {}

        Expr::StringInterp(parts) => {
            for part in parts {
                if let vais_ast::StringInterpPart::Expr(e) = part {
                    walk_expr(&e.node, state);
                }
            }
        }

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

        // Direct function call. If the callee is a recognised IO or
        // Alloc builtin, record the effect immediately. Otherwise just
        // add it to the call graph for the worklist.
        Expr::Call { func, args } => {
            walk_expr(&func.node, state);
            for a in args {
                walk_expr(&a.node, state);
            }
            if let Expr::Ident(name) = &func.node {
                if IO_BUILTINS.contains(&name.as_str()) {
                    state.record_io(&format!("calls `{}` which performs io", name));
                } else if ALLOC_BUILTINS.contains(&name.as_str()) {
                    state.record_alloc(&format!("calls `{}` which allocates", name));
                } else {
                    state.calls.push(name.clone());
                }
            }
        }

        // Method and static calls: we walk the receiver/arguments but
        // do NOT flag them as effects. v1 scope is conservative;
        // method-resolution-based effect tracking lives in a follow-up.
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
            walk_expr(&expr.node, state)
        }
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

        Expr::Await(inner) => walk_expr(&inner.node, state),
        Expr::Try(inner) => walk_expr(&inner.node, state),
        Expr::Unwrap(inner) => walk_expr(&inner.node, state),

        Expr::MapLit(pairs) => {
            for (k, v) in pairs {
                walk_expr(&k.node, state);
                walk_expr(&v.node, state);
            }
        }

        Expr::Spread(inner) | Expr::Ref(inner) | Expr::Deref(inner) => {
            walk_expr(&inner.node, state)
        }
        Expr::Cast { expr, .. } => walk_expr(&expr.node, state),

        Expr::Assign { target, value } => {
            walk_expr(&target.node, state);
            walk_expr(&value.node, state);
        }
        Expr::AssignOp { target, value, .. } => {
            walk_expr(&target.node, state);
            walk_expr(&value.node, state);
        }

        Expr::Lambda { body, .. } => walk_expr(&body.node, state),
        Expr::Yield(inner) => walk_expr(&inner.node, state),

        // `comptime { expr }` runs at compile time: no runtime effects.
        Expr::Comptime { .. } => {}

        // `assert(...)` panics but that's the totality gate's concern.
        Expr::Assert { condition, message } => {
            walk_expr(&condition.node, state);
            if let Some(m) = message {
                walk_expr(&m.node, state);
            }
        }
        Expr::Assume(inner) => walk_expr(&inner.node, state),
        Expr::Old(inner) => walk_expr(&inner.node, state),

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
