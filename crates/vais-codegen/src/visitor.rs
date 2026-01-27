//! Visitor pattern traits for AST code generation
//!
//! This module defines the visitor traits that decompose code generation into
//! separate concerns for expressions, statements, and items.

use vais_ast::{Spanned, Expr, Stmt, BinOp, UnaryOp, MatchArm, Param, Type};
use crate::CodegenResult;

/// Result type for code generation: (value, ir_code)
pub type GenResult = CodegenResult<(String, String)>;

/// Result type for block generation: (value, ir_code, is_terminated)
pub type BlockResult = CodegenResult<(String, String, bool)>;

/// Visitor trait for expression code generation
///
/// This trait defines methods for visiting each type of expression in the AST.
/// Implementations generate LLVM IR for each expression type.
pub trait ExprVisitor {
    /// Visit any expression (dispatcher method)
    fn visit_expr(&mut self, expr: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit an integer literal
    fn visit_int(&mut self, n: i64) -> GenResult;

    /// Visit a float literal
    fn visit_float(&mut self, n: f64) -> GenResult;

    /// Visit a boolean literal
    fn visit_bool(&mut self, b: bool) -> GenResult;

    /// Visit a string literal
    fn visit_string(&mut self, s: &str, counter: &mut usize) -> GenResult;

    /// Visit a unit value
    fn visit_unit(&mut self) -> GenResult;

    /// Visit an identifier reference
    fn visit_ident(&mut self, name: &str, counter: &mut usize) -> GenResult;

    /// Visit a self-call expression (@)
    fn visit_self_call(&mut self) -> GenResult;

    /// Visit a binary operation
    fn visit_binary(
        &mut self,
        op: &BinOp,
        left: &Spanned<Expr>,
        right: &Spanned<Expr>,
        counter: &mut usize,
        span: vais_ast::Span,
    ) -> GenResult;

    /// Visit a unary operation
    fn visit_unary(
        &mut self,
        op: &UnaryOp,
        expr: &Spanned<Expr>,
        counter: &mut usize,
        span: vais_ast::Span,
    ) -> GenResult;

    /// Visit a ternary expression (cond ? then : else)
    fn visit_ternary(
        &mut self,
        cond: &Spanned<Expr>,
        then: &Spanned<Expr>,
        else_: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a function call
    fn visit_call(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: vais_ast::Span,
    ) -> GenResult;

    /// Visit an if expression
    fn visit_if(
        &mut self,
        cond: &Spanned<Expr>,
        then: &[Spanned<Stmt>],
        else_: Option<&vais_ast::IfElse>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a loop expression
    fn visit_loop(
        &mut self,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a while loop expression
    fn visit_while(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a block expression
    fn visit_block(&mut self, stmts: &[Spanned<Stmt>], counter: &mut usize) -> GenResult;

    /// Visit an assignment expression
    fn visit_assign(
        &mut self,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a compound assignment expression (+=, -=, etc.)
    fn visit_assign_op(
        &mut self,
        op: &BinOp,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit an array literal
    fn visit_array(&mut self, elements: &[Spanned<Expr>], counter: &mut usize) -> GenResult;

    /// Visit a tuple literal
    fn visit_tuple(&mut self, elements: &[Spanned<Expr>], counter: &mut usize) -> GenResult;

    /// Visit a struct literal
    fn visit_struct_lit(
        &mut self,
        name: &Spanned<String>,
        fields: &[(Spanned<String>, Spanned<Expr>)],
        counter: &mut usize,
    ) -> GenResult;

    /// Visit an index expression
    fn visit_index(
        &mut self,
        array: &Spanned<Expr>,
        index: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a field access expression
    fn visit_field(
        &mut self,
        obj: &Spanned<Expr>,
        field: &Spanned<String>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a method call expression
    fn visit_method_call(
        &mut self,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a static method call expression
    fn visit_static_method_call(
        &mut self,
        type_name: &Spanned<String>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a reference expression (&expr)
    fn visit_ref(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a dereference expression (*expr)
    fn visit_deref(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a type cast expression (expr as Type)
    fn visit_cast(
        &mut self,
        expr: &Spanned<Expr>,
        ty: &Spanned<Type>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a match expression
    fn visit_match(
        &mut self,
        expr: &Spanned<Expr>,
        arms: &[MatchArm],
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a range expression
    fn visit_range(
        &mut self,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit an await expression
    fn visit_await(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a spawn expression
    fn visit_spawn(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a lambda expression
    fn visit_lambda(
        &mut self,
        params: &[Param],
        body: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit a try expression (expr?)
    fn visit_try(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit an unwrap expression (expr!)
    fn visit_unwrap(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a comptime expression (comptime { ... })
    fn visit_comptime(&mut self, body: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a macro invocation (name!(...))
    /// Note: Macro invocations should be expanded before codegen, so this should
    /// never be called in practice. If it is, it indicates a compiler bug.
    fn visit_macro_invoke(&mut self, invoke: &vais_ast::MacroInvoke) -> GenResult;

    /// Visit an old expression (old(expr))
    /// Used in ensures clauses to reference pre-state values.
    fn visit_old(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit an assert expression (assert(condition) or assert(condition, message))
    /// Generates runtime check that panics if condition is false.
    fn visit_assert(
        &mut self,
        condition: &Spanned<Expr>,
        message: Option<&Spanned<Expr>>,
        counter: &mut usize,
    ) -> GenResult;

    /// Visit an assume expression (assume(condition))
    /// Tells the verifier/optimizer to assume condition is true.
    fn visit_assume(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a lazy expression (lazy expr)
    /// Creates a thunk that defers evaluation until forced.
    fn visit_lazy(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;

    /// Visit a force expression (force expr)
    /// Forces evaluation of a lazy expression, returning the cached value.
    fn visit_force(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult;
}

/// Visitor trait for statement code generation
pub trait StmtVisitor {
    /// Visit any statement (dispatcher method)
    fn visit_stmt(&mut self, stmt: &Spanned<Stmt>, counter: &mut usize) -> GenResult;

    /// Visit a block of statements
    fn visit_block_stmts(&mut self, stmts: &[Spanned<Stmt>], counter: &mut usize) -> BlockResult;
}

/// Visitor trait for top-level items (functions, structs, enums, etc.)
///
/// Note: This trait is designed for future expansion.
/// Currently, item generation is handled directly in lib.rs.
pub trait ItemVisitor {
    /// Generate code for a function
    fn visit_function(&mut self, func: &vais_ast::Function, span: vais_ast::Span) -> CodegenResult<String>;

    /// Generate code for a method
    fn visit_method(
        &mut self,
        struct_name: &str,
        func: &vais_ast::Function,
        span: vais_ast::Span,
    ) -> CodegenResult<String>;
}
