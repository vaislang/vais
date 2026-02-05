//! Effect System - Effect inference and checking
//!
//! This module provides effect inference for functions, tracking what
//! side effects a function may have. Effects are propagated through
//! the call graph and checked against declared effect annotations.

use crate::types::{Effect, EffectSet, FunctionSig};
use crate::TypeError;
use std::collections::{HashMap, HashSet};
use vais_ast::*;

/// Effect inference context
pub struct EffectInferrer {
    /// Function effects: function name -> inferred effects
    function_effects: HashMap<String, EffectSet>,
    /// Pending functions (for handling recursion)
    pending: HashSet<String>,
    /// Builtin function effects
    builtins: HashMap<String, EffectSet>,
}

impl Default for EffectInferrer {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectInferrer {
    /// Create a new effect inferrer
    pub fn new() -> Self {
        let mut builtins = HashMap::new();

        // IO functions
        for name in [
            "print", "println", "eprint", "eprintln", "puts", "putchar", "printf",
        ] {
            builtins.insert(name.to_string(), EffectSet::io());
        }

        // Memory allocation functions
        for name in ["malloc", "calloc", "realloc", "free", "alloc", "dealloc"] {
            builtins.insert(name.to_string(), EffectSet::alloc());
        }

        // Pure math functions
        for name in [
            "abs", "min", "max", "clamp", "sqrt", "sin", "cos", "tan", "asin", "acos", "atan",
            "atan2", "exp", "log", "log2", "log10", "pow", "floor", "ceil", "round", "fabs",
        ] {
            builtins.insert(name.to_string(), EffectSet::pure());
        }

        // Non-deterministic functions
        for name in ["random", "rand", "time", "clock", "gettimeofday"] {
            builtins.insert(name.to_string(), EffectSet::single(Effect::NonDet));
        }

        // File IO
        for name in [
            "fopen", "fclose", "fread", "fwrite", "fseek", "ftell", "open", "close", "read",
            "write", "lseek",
        ] {
            builtins.insert(name.to_string(), EffectSet::io());
        }

        // Network IO
        for name in [
            "socket", "bind", "listen", "accept", "connect", "send", "recv", "sendto", "recvfrom",
        ] {
            builtins.insert(name.to_string(), EffectSet::io());
        }

        // Unsafe operations
        for name in [
            "memcpy", "memmove", "memset", "memcmp", "strlen", "strcpy", "strcat", "strcmp",
        ] {
            builtins.insert(name.to_string(), EffectSet::single(Effect::Unsafe));
        }

        // Panic functions
        for name in ["panic", "abort", "exit", "assert", "__panic"] {
            builtins.insert(name.to_string(), EffectSet::single(Effect::Panic));
        }

        Self {
            function_effects: HashMap::new(),
            pending: HashSet::new(),
            builtins,
        }
    }

    /// Get effects for a builtin function
    pub fn get_builtin_effects(&self, name: &str) -> Option<&EffectSet> {
        self.builtins.get(name)
    }

    /// Get inferred effects for a function
    pub fn get_function_effects(&self, name: &str) -> Option<&EffectSet> {
        self.function_effects.get(name)
    }

    /// Register a function's effects
    pub fn register_function_effects(&mut self, name: String, effects: EffectSet) {
        self.function_effects.insert(name, effects);
    }

    /// Infer effects for an expression
    pub fn infer_expr_effects(
        &mut self,
        expr: &Expr,
        functions: &HashMap<String, &FunctionSig>,
    ) -> EffectSet {
        match expr {
            // Literals are pure
            Expr::Int(_) | Expr::Float(_) | Expr::String(_) | Expr::Bool(_) | Expr::Unit => {
                EffectSet::pure()
            }

            // Variables/Identifiers are pure (reading from local scope)
            Expr::Ident(_) | Expr::SelfCall => EffectSet::pure(),

            // Binary operations are pure (unless operands have effects)
            Expr::Binary { left, right, .. } => {
                let left_effects = self.infer_expr_effects(&left.node, functions);
                let right_effects = self.infer_expr_effects(&right.node, functions);
                left_effects.union(&right_effects)
            }

            // Unary operations are pure
            Expr::Unary { expr, .. } => self.infer_expr_effects(&expr.node, functions),

            // Ternary expression
            Expr::Ternary {
                cond, then, else_, ..
            } => {
                let cond_effects = self.infer_expr_effects(&cond.node, functions);
                let then_effects = self.infer_expr_effects(&then.node, functions);
                let else_effects = self.infer_expr_effects(&else_.node, functions);
                cond_effects.union(&then_effects).union(&else_effects)
            }

            // Function calls inherit callee's effects
            Expr::Call { func, args, .. } => {
                let mut effects = EffectSet::pure();

                // Infer effects from arguments
                for arg in args {
                    let arg_effects = self.infer_expr_effects(&arg.node, functions);
                    effects = effects.union(&arg_effects);
                }

                // Get function name and add its effects
                if let Expr::Ident(name) = &func.node {
                    if let Some(builtin_effects) = self.get_builtin_effects(name) {
                        effects = effects.union(builtin_effects);
                    } else if let Some(fn_effects) = self.get_function_effects(name) {
                        effects = effects.union(fn_effects);
                    } else if let Some(sig) = functions.get(name) {
                        if let Some(inferred) = &sig.inferred_effects {
                            effects = effects.union(inferred);
                        } else {
                            // Unknown function - assume total effects
                            effects = effects.union(&EffectSet::total());
                        }
                    }
                } else {
                    // Indirect call - assume total effects
                    effects = effects.union(&EffectSet::total());
                }

                effects
            }

            // Method calls
            Expr::MethodCall { receiver, args, .. } => {
                let mut effects = self.infer_expr_effects(&receiver.node, functions);
                for arg in args {
                    let arg_effects = self.infer_expr_effects(&arg.node, functions);
                    effects = effects.union(&arg_effects);
                }
                // Method calls may have effects - conservatively assume read/write
                effects.add(Effect::Read);
                effects.add(Effect::Write);
                effects
            }

            // Static method calls
            Expr::StaticMethodCall { args, .. } => {
                let mut effects = EffectSet::pure();
                for arg in args {
                    let arg_effects = self.infer_expr_effects(&arg.node, functions);
                    effects = effects.union(&arg_effects);
                }
                // Conservatively assume read/write
                effects.add(Effect::Read);
                effects.add(Effect::Write);
                effects
            }

            // Field access is pure (reading)
            Expr::Field { expr, .. } => self.infer_expr_effects(&expr.node, functions),

            // Index access is pure (reading)
            Expr::Index { expr, index, .. } => {
                let expr_effects = self.infer_expr_effects(&expr.node, functions);
                let index_effects = self.infer_expr_effects(&index.node, functions);
                expr_effects.union(&index_effects)
            }

            // Assignment has write effect
            Expr::Assign { target, value, .. } => {
                let target_effects = self.infer_expr_effects(&target.node, functions);
                let value_effects = self.infer_expr_effects(&value.node, functions);
                let mut effects = target_effects.union(&value_effects);
                effects.add(Effect::Write);
                effects
            }

            // Compound assignment has write effect
            Expr::AssignOp { target, value, .. } => {
                let target_effects = self.infer_expr_effects(&target.node, functions);
                let value_effects = self.infer_expr_effects(&value.node, functions);
                let mut effects = target_effects.union(&value_effects);
                effects.add(Effect::Write);
                effects
            }

            // If expression
            Expr::If {
                cond, then, else_, ..
            } => {
                let mut effects = self.infer_expr_effects(&cond.node, functions);
                for stmt in then {
                    let stmt_effects = self.infer_stmt_effects(&stmt.node, functions);
                    effects = effects.union(&stmt_effects);
                }
                if let Some(else_branch) = else_ {
                    effects = effects.union(&self.infer_if_else_effects(else_branch, functions));
                }
                effects
            }

            // Loop expression - may diverge
            Expr::Loop { iter, body, .. } => {
                let mut effects = EffectSet::single(Effect::Diverge);
                if let Some(iter_expr) = iter {
                    let iter_effects = self.infer_expr_effects(&iter_expr.node, functions);
                    effects = effects.union(&iter_effects);
                }
                for stmt in body {
                    let stmt_effects = self.infer_stmt_effects(&stmt.node, functions);
                    effects = effects.union(&stmt_effects);
                }
                effects
            }

            // While loop - may diverge
            Expr::While {
                condition, body, ..
            } => {
                let mut effects = self.infer_expr_effects(&condition.node, functions);
                effects.add(Effect::Diverge);
                for stmt in body {
                    let stmt_effects = self.infer_stmt_effects(&stmt.node, functions);
                    effects = effects.union(&stmt_effects);
                }
                effects
            }

            // Match expression
            Expr::Match { expr, arms, .. } => {
                let mut effects = self.infer_expr_effects(&expr.node, functions);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        let guard_effects = self.infer_expr_effects(&guard.node, functions);
                        effects = effects.union(&guard_effects);
                    }
                    let body_effects = self.infer_expr_effects(&arm.body.node, functions);
                    effects = effects.union(&body_effects);
                }
                effects
            }

            // Lambda - capture the body's effects
            Expr::Lambda { body, .. } => self.infer_expr_effects(&body.node, functions),

            // Struct literal is pure
            Expr::StructLit { fields, .. } => {
                let mut effects = EffectSet::pure();
                for (_, value) in fields {
                    let field_effects = self.infer_expr_effects(&value.node, functions);
                    effects = effects.union(&field_effects);
                }
                effects
            }

            // Array literal is pure
            Expr::Array(elements) => {
                let mut effects = EffectSet::pure();
                for elem in elements {
                    let elem_effects = self.infer_expr_effects(&elem.node, functions);
                    effects = effects.union(&elem_effects);
                }
                effects
            }

            // Tuple literal is pure
            Expr::Tuple(elements) => {
                let mut effects = EffectSet::pure();
                for elem in elements {
                    let elem_effects = self.infer_expr_effects(&elem.node, functions);
                    effects = effects.union(&elem_effects);
                }
                effects
            }

            // Range is pure
            Expr::Range { start, end, .. } => {
                let mut effects = EffectSet::pure();
                if let Some(start) = start {
                    let start_effects = self.infer_expr_effects(&start.node, functions);
                    effects = effects.union(&start_effects);
                }
                if let Some(end) = end {
                    let end_effects = self.infer_expr_effects(&end.node, functions);
                    effects = effects.union(&end_effects);
                }
                effects
            }

            // Try (?) operator - may panic
            Expr::Try(inner) => {
                let mut effects = self.infer_expr_effects(&inner.node, functions);
                effects.add(Effect::Panic);
                effects
            }

            // Unwrap (!) operator - may panic
            Expr::Unwrap(inner) => {
                let mut effects = self.infer_expr_effects(&inner.node, functions);
                effects.add(Effect::Panic);
                effects
            }

            // Await has async effect
            Expr::Await(inner) => {
                let mut effects = self.infer_expr_effects(&inner.node, functions);
                effects.add(Effect::Async);
                effects
            }

            // Spawn has async effect
            Expr::Spawn(inner) => {
                let mut effects = self.infer_expr_effects(&inner.node, functions);
                effects.add(Effect::Async);
                effects
            }

            // Cast is pure
            Expr::Cast { expr, .. } => self.infer_expr_effects(&expr.node, functions),

            // Reference is pure
            Expr::Ref(inner) | Expr::Spread(inner) => {
                self.infer_expr_effects(&inner.node, functions)
            }

            // Dereference may have read effect
            Expr::Deref(inner) => {
                let mut effects = self.infer_expr_effects(&inner.node, functions);
                effects.add(Effect::Read);
                effects.add(Effect::Unsafe);
                effects
            }

            // Block expression
            Expr::Block(stmts) => {
                let mut effects = EffectSet::pure();
                for stmt in stmts {
                    let stmt_effects = self.infer_stmt_effects(&stmt.node, functions);
                    effects = effects.union(&stmt_effects);
                }
                effects
            }

            // Comptime is pure (evaluated at compile time)
            Expr::Comptime { .. } => EffectSet::pure(),

            // Old expression (for contracts) is pure
            Expr::Old(_) => EffectSet::pure(),

            // Assert has panic effect
            Expr::Assert { .. } => EffectSet::single(Effect::Panic),

            // Assume is pure (compiler hint)
            Expr::Assume(_) => EffectSet::pure(),

            // Error expressions - assume total effects
            Expr::Error { .. } => EffectSet::total(),

            // Macro invocation - unknown effects
            Expr::MacroInvoke(_) => EffectSet::total(),

            // Lazy expression - defers effects until forced
            Expr::Lazy(inner) => self.infer_expr_effects(&inner.node, functions),

            // Force expression - evaluates lazy value
            Expr::Force(inner) => self.infer_expr_effects(&inner.node, functions),

            // Map literal is pure
            Expr::MapLit(pairs) => {
                let mut effects = EffectSet::pure();
                for (k, v) in pairs {
                    effects = effects.union(&self.infer_expr_effects(&k.node, functions));
                    effects = effects.union(&self.infer_expr_effects(&v.node, functions));
                }
                effects
            }
            Expr::StringInterp(parts) => {
                let mut effects = EffectSet::pure();
                for part in parts {
                    if let vais_ast::StringInterpPart::Expr(e) = part {
                        effects = effects.union(&self.infer_expr_effects(&e.node, functions));
                    }
                }
                effects
            }
        }
    }

    /// Infer effects for if-else branches
    fn infer_if_else_effects(
        &mut self,
        if_else: &IfElse,
        functions: &HashMap<String, &FunctionSig>,
    ) -> EffectSet {
        match if_else {
            IfElse::ElseIf(cond, stmts, next) => {
                let mut effects = self.infer_expr_effects(&cond.node, functions);
                for stmt in stmts {
                    let stmt_effects = self.infer_stmt_effects(&stmt.node, functions);
                    effects = effects.union(&stmt_effects);
                }
                if let Some(next_branch) = next {
                    effects = effects.union(&self.infer_if_else_effects(next_branch, functions));
                }
                effects
            }
            IfElse::Else(stmts) => {
                let mut effects = EffectSet::pure();
                for stmt in stmts {
                    let stmt_effects = self.infer_stmt_effects(&stmt.node, functions);
                    effects = effects.union(&stmt_effects);
                }
                effects
            }
        }
    }

    /// Infer effects for a statement
    pub fn infer_stmt_effects(
        &mut self,
        stmt: &Stmt,
        functions: &HashMap<String, &FunctionSig>,
    ) -> EffectSet {
        match stmt {
            Stmt::Let { value, .. } | Stmt::LetDestructure { value, .. } => {
                self.infer_expr_effects(&value.node, functions)
            }

            Stmt::Expr(expr) => self.infer_expr_effects(&expr.node, functions),

            Stmt::Return(Some(expr)) => self.infer_expr_effects(&expr.node, functions),
            Stmt::Return(None) => EffectSet::pure(),

            Stmt::Break(Some(expr)) => self.infer_expr_effects(&expr.node, functions),
            Stmt::Break(None) => EffectSet::pure(),

            Stmt::Continue => EffectSet::pure(),

            // Defer wraps an expression
            Stmt::Defer(inner) => self.infer_expr_effects(&inner.node, functions),

            // Error statements - assume total effects
            Stmt::Error { .. } => EffectSet::total(),
        }
    }

    /// Infer effects for a function
    pub fn infer_function_effects(
        &mut self,
        func: &Function,
        functions: &HashMap<String, &FunctionSig>,
    ) -> EffectSet {
        let name = &func.name.node;

        // Check for recursion
        if self.pending.contains(name) {
            // Recursive call - return what we have so far (bottom-up)
            return self
                .function_effects
                .get(name)
                .cloned()
                .unwrap_or_else(EffectSet::pure);
        }

        // Mark as pending
        self.pending.insert(name.clone());

        // Check for explicit annotation
        let mut effects = match self.get_declared_effects(func) {
            Some(declared) => declared,
            None => {
                // Infer from body
                match &func.body {
                    FunctionBody::Expr(expr) => self.infer_expr_effects(&expr.node, functions),
                    FunctionBody::Block(stmts) => {
                        let mut effects = EffectSet::pure();
                        for stmt in stmts {
                            let stmt_effects = self.infer_stmt_effects(&stmt.node, functions);
                            effects = effects.union(&stmt_effects);
                        }
                        effects
                    }
                }
            }
        };

        // Async functions have async effect
        if func.is_async {
            effects.add(Effect::Async);
        }

        // Remove from pending
        self.pending.remove(name);

        // Store the result
        self.function_effects.insert(name.clone(), effects.clone());

        effects
    }

    /// Get declared effects from function attributes
    fn get_declared_effects(&self, func: &Function) -> Option<EffectSet> {
        for attr in &func.attributes {
            match attr.name.as_str() {
                "pure" => return Some(EffectSet::pure()),
                "effect" => {
                    let mut effects = EffectSet::pure();
                    for arg in &attr.args {
                        match arg.as_str() {
                            "read" => effects.add(Effect::Read),
                            "write" => effects.add(Effect::Write),
                            "alloc" => effects.add(Effect::Alloc),
                            "io" => effects.add(Effect::IO),
                            "async" => effects.add(Effect::Async),
                            "panic" => effects.add(Effect::Panic),
                            "nondet" => effects.add(Effect::NonDet),
                            "unsafe" => effects.add(Effect::Unsafe),
                            "diverge" => effects.add(Effect::Diverge),
                            _ => {}
                        }
                    }
                    return Some(effects);
                }
                _ => {}
            }
        }
        None
    }

    /// Check that a function's effects are consistent with its declaration
    pub fn check_effects(&self, func: &Function, inferred: &EffectSet) -> Result<(), TypeError> {
        if let Some(declared) = self.get_declared_effects(func) {
            if !inferred.is_subset_of(&declared) {
                return Err(TypeError::EffectMismatch {
                    declared: declared.to_string(),
                    actual: inferred.to_string(),
                    span: Some(func.name.span),
                });
            }
        }
        Ok(())
    }

    /// Check purity violation: pure function calling impure function
    pub fn check_purity_violation(
        &self,
        caller: &Function,
        callee_name: &str,
        callee_effects: &EffectSet,
        span: Span,
    ) -> Result<(), TypeError> {
        if let Some(declared) = self.get_declared_effects(caller) {
            if declared.is_pure() && !callee_effects.is_pure() {
                return Err(TypeError::PurityViolation {
                    callee: callee_name.to_string(),
                    effects: callee_effects.to_string(),
                    span: Some(span),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_set_pure() {
        let effects = EffectSet::pure();
        assert!(effects.is_pure());
        assert!(effects.is_readonly());
    }

    #[test]
    fn test_effect_set_single() {
        let effects = EffectSet::single(Effect::IO);
        assert!(!effects.is_pure());
        assert!(!effects.is_readonly());
        assert!(effects.contains(Effect::IO));
    }

    #[test]
    fn test_effect_set_union() {
        let a = EffectSet::single(Effect::Read);
        let b = EffectSet::single(Effect::Write);
        let c = a.union(&b);
        assert!(c.contains(Effect::Read));
        assert!(c.contains(Effect::Write));
        assert!(!c.is_pure());
    }

    #[test]
    fn test_effect_set_subset() {
        let read = EffectSet::single(Effect::Read);
        let read_write = EffectSet::read_write();
        assert!(read.is_subset_of(&read_write));
        assert!(!read_write.is_subset_of(&read));
    }

    #[test]
    fn test_effect_display() {
        let pure = EffectSet::pure();
        assert_eq!(pure.to_string(), "pure");

        let io = EffectSet::single(Effect::IO);
        assert!(io.to_string().contains("io"));
    }

    #[test]
    fn test_builtin_effects() {
        let inferrer = EffectInferrer::new();

        // IO functions
        assert!(!inferrer.get_builtin_effects("println").unwrap().is_pure());

        // Math functions
        assert!(inferrer.get_builtin_effects("sqrt").unwrap().is_pure());

        // Allocation
        assert!(inferrer
            .get_builtin_effects("malloc")
            .unwrap()
            .contains(Effect::Alloc));
    }
}
