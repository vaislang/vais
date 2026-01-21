//! Comprehensive tests for the Vais code formatter
//!
//! Tests cover:
//! - Function formatting (simple functions, multiple parameters, generics)
//! - Struct and enum formatting
//! - Control flow formatting (if-else, loops, match)
//! - Expression formatting (nested expressions, operators)
//! - Indentation configuration

use vais_ast::*;
use vais_codegen::formatter::{FormatConfig, Formatter};

// ============================================================================
// Helper Functions for Creating AST Nodes
// ============================================================================

fn span() -> Span {
    Span::new(0, 0)
}

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned::new(node, span())
}

fn ident(name: &str) -> Spanned<String> {
    spanned(name.to_string())
}

fn int_expr(n: i64) -> Expr {
    Expr::Int(n)
}

fn bool_expr(b: bool) -> Expr {
    Expr::Bool(b)
}

fn string_expr(s: &str) -> Expr {
    Expr::String(s.to_string())
}

fn ident_expr(name: &str) -> Expr {
    Expr::Ident(name.to_string())
}

fn named_type(name: &str) -> Type {
    Type::Named {
        name: name.to_string(),
        generics: vec![],
    }
}

fn generic_type(name: &str, generics: Vec<Type>) -> Type {
    Type::Named {
        name: name.to_string(),
        generics: generics.into_iter().map(|t| spanned(t)).collect(),
    }
}

// ============================================================================
// Function Formatting Tests
// ============================================================================

#[test]
fn test_format_simple_function() {
    let func = Function {
        name: ident("add"),
        params: vec![
            Param {
                name: ident("a"),
                ty: spanned(named_type("i64")),
                is_mut: false,
            },
            Param {
                name: ident("b"),
                ty: spanned(named_type("i64")),
                is_mut: false,
            },
        ],
        ret_type: Some(spanned(named_type("i64"))),
        body: FunctionBody::Expr(Box::new(spanned(Expr::Binary {
            op: BinOp::Add,
            left: Box::new(spanned(ident_expr("a"))),
            right: Box::new(spanned(ident_expr("b"))),
        }))),
        is_pub: true,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub F add(a: i64, b: i64) -> i64 = a + b"));
}

#[test]
fn test_format_function_with_no_params() {
    let func = Function {
        name: ident("get_answer"),
        params: vec![],
        ret_type: Some(spanned(named_type("i64"))),
        body: FunctionBody::Expr(Box::new(spanned(int_expr(42)))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("F get_answer() -> i64 = 42"));
}

#[test]
fn test_format_function_with_generics() {
    let func = Function {
        name: ident("identity"),
        generics: vec![GenericParam::new_type(ident("T"), vec![])],
        params: vec![Param {
            name: ident("x"),
            ty: spanned(named_type("T")),
            is_mut: false,
        }],
        ret_type: Some(spanned(named_type("T"))),
        body: FunctionBody::Expr(Box::new(spanned(ident_expr("x")))),
        is_pub: true,
        is_async: false,
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub F identity<T>(x: T) -> T = x"));
}

#[test]
fn test_format_function_with_bounded_generics() {
    let func = Function {
        name: ident("display"),
        generics: vec![GenericParam::new_type(ident("T"), vec![ident("Display"), ident("Clone")])],
        params: vec![Param {
            name: ident("value"),
            ty: spanned(named_type("T")),
            is_mut: false,
        }],
        ret_type: Some(spanned(Type::Unit)),
        body: FunctionBody::Expr(Box::new(spanned(Expr::Unit))),
        is_pub: false,
        is_async: false,
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("F display<T: Display + Clone>(value: T) -> () = ()"));
}

#[test]
fn test_format_async_function() {
    let func = Function {
        name: ident("fetch_data"),
        params: vec![],
        ret_type: Some(spanned(named_type("String"))),
        body: FunctionBody::Expr(Box::new(spanned(string_expr("data")))),
        is_pub: true,
        is_async: true,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub async F fetch_data() -> String"));
}

#[test]
fn test_format_function_with_block_body() {
    let func = Function {
        name: ident("compute"),
        params: vec![Param {
            name: ident("x"),
            ty: spanned(named_type("i64")),
            is_mut: false,
        }],
        ret_type: Some(spanned(named_type("i64"))),
        body: FunctionBody::Block(vec![
            spanned(Stmt::Let {
                name: ident("y"),
                ty: None,
                value: Box::new(spanned(int_expr(10))),
                is_mut: false,
            }),
            spanned(Stmt::Return(Some(Box::new(spanned(Expr::Binary {
                op: BinOp::Add,
                left: Box::new(spanned(ident_expr("x"))),
                right: Box::new(spanned(ident_expr("y"))),
            }))))),
        ]),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("F compute(x: i64) -> i64 {"));
    assert!(output.contains("    y := 10"));
    assert!(output.contains("    R x + y"));
    assert!(output.contains("}"));
}

#[test]
fn test_format_function_with_attributes() {
    let func = Function {
        name: ident("test_func"),
        params: vec![],
        ret_type: None,
        body: FunctionBody::Expr(Box::new(spanned(Expr::Unit))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![
            Attribute {
                name: "inline".to_string(),
                args: vec![],
            },
            Attribute {
                name: "cfg".to_string(),
                args: vec!["test".to_string()],
            },
        ],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("#[inline]"));
    assert!(output.contains("#[cfg(test)]"));
}

#[test]
fn test_format_function_with_mut_params() {
    let func = Function {
        name: ident("increment"),
        params: vec![Param {
            name: ident("x"),
            ty: spanned(named_type("i64")),
            is_mut: true,
        }],
        ret_type: Some(spanned(Type::Unit)),
        body: FunctionBody::Expr(Box::new(spanned(Expr::Unit))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("F increment(mut x: i64)"));
}

// ============================================================================
// Struct Formatting Tests
// ============================================================================

#[test]
fn test_format_simple_struct() {
    let s = Struct {
        name: ident("Point"),
        fields: vec![
            Field {
                name: ident("x"),
                ty: spanned(named_type("i64")),
                is_pub: true,
            },
            Field {
                name: ident("y"),
                ty: spanned(named_type("i64")),
                is_pub: true,
            },
        ],
        methods: vec![],
        is_pub: true,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Struct(s))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub S Point {"));
    assert!(output.contains("    pub x: i64,"));
    assert!(output.contains("    pub y: i64,"));
    assert!(output.contains("}"));
}

#[test]
fn test_format_generic_struct() {
    let s = Struct {
        name: ident("Container"),
        generics: vec![GenericParam::new_type(ident("T"), vec![])],
        fields: vec![Field {
            name: ident("value"),
            ty: spanned(named_type("T")),
            is_pub: true,
        }],
        methods: vec![],
        is_pub: true,
    };

    let module = Module {
        items: vec![spanned(Item::Struct(s))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub S Container<T> {"));
    assert!(output.contains("    pub value: T,"));
}

#[test]
fn test_format_struct_with_methods() {
    let method = Function {
        name: ident("new"),
        params: vec![Param {
            name: ident("x"),
            ty: spanned(named_type("i64")),
            is_mut: false,
        }],
        ret_type: Some(spanned(named_type("Point"))),
        body: FunctionBody::Expr(Box::new(spanned(Expr::StructLit {
            name: ident("Point"),
            fields: vec![
                (ident("x"), spanned(ident_expr("x"))),
                (ident("y"), spanned(int_expr(0))),
            ],
        }))),
        is_pub: true,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let s = Struct {
        name: ident("Point"),
        fields: vec![
            Field {
                name: ident("x"),
                ty: spanned(named_type("i64")),
                is_pub: false,
            },
            Field {
                name: ident("y"),
                ty: spanned(named_type("i64")),
                is_pub: false,
            },
        ],
        methods: vec![spanned(method)],
        is_pub: true,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Struct(s))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub S Point {"));
    assert!(output.contains("    x: i64,"));
    assert!(output.contains("    pub F new(x: i64) -> Point"));
}

// ============================================================================
// Enum Formatting Tests
// ============================================================================

#[test]
fn test_format_simple_enum() {
    let e = Enum {
        name: ident("Color"),
        variants: vec![
            Variant {
                name: ident("Red"),
                fields: VariantFields::Unit,
            },
            Variant {
                name: ident("Green"),
                fields: VariantFields::Unit,
            },
            Variant {
                name: ident("Blue"),
                fields: VariantFields::Unit,
            },
        ],
        is_pub: true,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Enum(e))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub E Color {"));
    assert!(output.contains("    Red,"));
    assert!(output.contains("    Green,"));
    assert!(output.contains("    Blue,"));
}

#[test]
fn test_format_enum_with_tuple_variants() {
    let e = Enum {
        name: ident("Option"),
        generics: vec![GenericParam::new_type(ident("T"), vec![])],
        variants: vec![
            Variant {
                name: ident("Some"),
                fields: VariantFields::Tuple(vec![spanned(named_type("T"))]),
            },
            Variant {
                name: ident("None"),
                fields: VariantFields::Unit,
            },
        ],
        is_pub: true,
    };

    let module = Module {
        items: vec![spanned(Item::Enum(e))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub E Option<T> {"));
    assert!(output.contains("    Some(T),"));
    assert!(output.contains("    None,"));
}

#[test]
fn test_format_enum_with_struct_variants() {
    let e = Enum {
        name: ident("Message"),
        variants: vec![
            Variant {
                name: ident("Quit"),
                fields: VariantFields::Unit,
            },
            Variant {
                name: ident("Move"),
                fields: VariantFields::Struct(vec![
                    Field {
                        name: ident("x"),
                        ty: spanned(named_type("i64")),
                        is_pub: false,
                    },
                    Field {
                        name: ident("y"),
                        ty: spanned(named_type("i64")),
                        is_pub: false,
                    },
                ]),
            },
        ],
        is_pub: true,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Enum(e))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub E Message {"));
    assert!(output.contains("    Quit,"));
    assert!(output.contains("    Move {"));
    assert!(output.contains("        x: i64,"));
    assert!(output.contains("        y: i64,"));
}

// ============================================================================
// Control Flow Formatting Tests
// ============================================================================

#[test]
fn test_format_if_else() {
    let if_expr = Expr::If {
        cond: Box::new(spanned(ident_expr("x"))),
        then: vec![spanned(Stmt::Return(Some(Box::new(spanned(int_expr(1))))))],
        else_: Some(IfElse::Else(vec![spanned(Stmt::Return(
            Some(Box::new(spanned(int_expr(0)))),
        ))])),
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(named_type("i64"))),
        body: FunctionBody::Block(vec![spanned(Stmt::Expr(Box::new(spanned(if_expr))))]),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("I x {"));
    assert!(output.contains("} E {"));
}

#[test]
fn test_format_else_if_chain() {
    let if_expr = Expr::If {
        cond: Box::new(spanned(Expr::Binary {
            op: BinOp::Lt,
            left: Box::new(spanned(ident_expr("x"))),
            right: Box::new(spanned(int_expr(0))),
        })),
        then: vec![spanned(Stmt::Return(Some(Box::new(spanned(int_expr(-1))))))],
        else_: Some(IfElse::ElseIf(
            Box::new(spanned(Expr::Binary {
                op: BinOp::Gt,
                left: Box::new(spanned(ident_expr("x"))),
                right: Box::new(spanned(int_expr(0))),
            })),
            vec![spanned(Stmt::Return(Some(Box::new(spanned(int_expr(1))))))],
            Some(Box::new(IfElse::Else(vec![spanned(Stmt::Return(
                Some(Box::new(spanned(int_expr(0)))),
            ))]))),
        )),
    };

    let func = Function {
        name: ident("sign"),
        params: vec![],
        ret_type: Some(spanned(named_type("i64"))),
        body: FunctionBody::Block(vec![spanned(Stmt::Expr(Box::new(spanned(if_expr))))]),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("I x < 0 {"));
    assert!(output.contains("} E I x > 0 {"));
    assert!(output.contains("} E {"));
}

#[test]
fn test_format_loop() {
    let loop_expr = Expr::Loop {
        pattern: Some(spanned(Pattern::Ident("i".to_string()))),
        iter: Some(Box::new(spanned(Expr::Range {
            start: Some(Box::new(spanned(int_expr(0)))),
            end: Some(Box::new(spanned(int_expr(10)))),
            inclusive: false,
        }))),
        body: vec![spanned(Stmt::Expr(Box::new(spanned(Expr::Call {
            func: Box::new(spanned(ident_expr("println"))),
            args: vec![spanned(ident_expr("i"))],
        }))))],
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: None,
        body: FunctionBody::Block(vec![spanned(Stmt::Expr(Box::new(spanned(loop_expr))))]),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("L i:0..10 {"));
}

#[test]
fn test_format_match_expression() {
    let match_expr = Expr::Match {
        expr: Box::new(spanned(ident_expr("value"))),
        arms: vec![
            MatchArm {
                pattern: spanned(Pattern::Literal(Literal::Int(0))),
                guard: None,
                body: Box::new(spanned(string_expr("zero"))),
            },
            MatchArm {
                pattern: spanned(Pattern::Literal(Literal::Int(1))),
                guard: None,
                body: Box::new(spanned(string_expr("one"))),
            },
            MatchArm {
                pattern: spanned(Pattern::Wildcard),
                guard: None,
                body: Box::new(spanned(string_expr("other"))),
            },
        ],
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(named_type("String"))),
        body: FunctionBody::Block(vec![spanned(Stmt::Return(Some(Box::new(spanned(match_expr)))))]),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("M value {"));
    assert!(output.contains("0 => \"zero\","));
    assert!(output.contains("1 => \"one\","));
    assert!(output.contains("_ => \"other\","));
}

#[test]
fn test_format_match_with_guard() {
    let match_expr = Expr::Match {
        expr: Box::new(spanned(ident_expr("x"))),
        arms: vec![MatchArm {
            pattern: spanned(Pattern::Ident("n".to_string())),
            guard: Some(Box::new(spanned(Expr::Binary {
                op: BinOp::Gt,
                left: Box::new(spanned(ident_expr("n"))),
                right: Box::new(spanned(int_expr(10))),
            }))),
            body: Box::new(spanned(string_expr("large"))),
        }],
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(named_type("String"))),
        body: FunctionBody::Expr(Box::new(spanned(match_expr))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("n I n > 10 => \"large\""));
}

// ============================================================================
// Expression Formatting Tests
// ============================================================================

#[test]
fn test_format_binary_expressions() {
    let expr = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(spanned(Expr::Binary {
            op: BinOp::Mul,
            left: Box::new(spanned(int_expr(2))),
            right: Box::new(spanned(int_expr(3))),
        })),
        right: Box::new(spanned(int_expr(4))),
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(named_type("i64"))),
        body: FunctionBody::Expr(Box::new(spanned(expr))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("2 * 3 + 4"));
}

#[test]
fn test_format_unary_expressions() {
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(spanned(Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(spanned(ident_expr("x"))),
        })),
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(named_type("bool"))),
        body: FunctionBody::Expr(Box::new(spanned(expr))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("-!x"));
}

#[test]
fn test_format_method_call() {
    let expr = Expr::MethodCall {
        receiver: Box::new(spanned(ident_expr("vec"))),
        method: ident("push"),
        args: vec![spanned(int_expr(42))],
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: None,
        body: FunctionBody::Expr(Box::new(spanned(expr))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("vec.push(42)"));
}

#[test]
fn test_format_array_literal() {
    let expr = Expr::Array(vec![
        spanned(int_expr(1)),
        spanned(int_expr(2)),
        spanned(int_expr(3)),
    ]);

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(Type::Array(Box::new(spanned(named_type("i64")))))),
        body: FunctionBody::Expr(Box::new(spanned(expr))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("[1, 2, 3]"));
}

#[test]
fn test_format_tuple() {
    let expr = Expr::Tuple(vec![
        spanned(int_expr(1)),
        spanned(string_expr("hello")),
        spanned(bool_expr(true)),
    ]);

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(Type::Tuple(vec![
            spanned(named_type("i64")),
            spanned(named_type("String")),
            spanned(named_type("bool")),
        ]))),
        body: FunctionBody::Expr(Box::new(spanned(expr))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("(1, \"hello\", true)"));
}

#[test]
fn test_format_struct_literal() {
    let expr = Expr::StructLit {
        name: ident("Point"),
        fields: vec![
            (ident("x"), spanned(int_expr(10))),
            (ident("y"), spanned(int_expr(20))),
        ],
    };

    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: Some(spanned(named_type("Point"))),
        body: FunctionBody::Expr(Box::new(spanned(expr))),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("Point { x: 10, y: 20 }"));
}

// ============================================================================
// Type Formatting Tests
// ============================================================================

#[test]
fn test_format_types() {
    let type_alias = TypeAlias {
        name: ident("IntArray"),
        ty: spanned(Type::Array(Box::new(spanned(named_type("i64"))))),
        is_pub: true,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::TypeAlias(type_alias))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("pub T IntArray = [i64]"));
}

#[test]
fn test_format_generic_types() {
    let type_alias = TypeAlias {
        name: ident("VecOfVec"),
        ty: spanned(generic_type(
            "Vec",
            vec![generic_type("Vec", vec![named_type("i64")])],
        )),
        is_pub: false,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::TypeAlias(type_alias))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("T VecOfVec = Vec<Vec<i64>>"));
}

#[test]
fn test_format_optional_type() {
    let type_alias = TypeAlias {
        name: ident("MaybeInt"),
        ty: spanned(Type::Optional(Box::new(spanned(named_type("i64"))))),
        is_pub: false,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::TypeAlias(type_alias))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("T MaybeInt = i64?"));
}

#[test]
fn test_format_result_type() {
    let type_alias = TypeAlias {
        name: ident("ResultInt"),
        ty: spanned(Type::Result(Box::new(spanned(named_type("i64"))))),
        is_pub: false,
        generics: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::TypeAlias(type_alias))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("T ResultInt = i64!"));
}

// ============================================================================
// Indentation Configuration Tests
// ============================================================================

#[test]
fn test_custom_indent_size() {
    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: None,
        body: FunctionBody::Block(vec![spanned(Stmt::Let {
            name: ident("x"),
            ty: None,
            value: Box::new(spanned(int_expr(42))),
            is_mut: false,
        })]),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let config = FormatConfig {
        indent_size: 2,
        max_line_length: 100,
        use_tabs: false,
    };

    let mut formatter = Formatter::new(config);
    let output = formatter.format_module(&module);

    // Should use 2-space indentation
    assert!(output.contains("  x := 42"));
}

#[test]
fn test_use_tabs_indentation() {
    let func = Function {
        name: ident("test"),
        params: vec![],
        ret_type: None,
        body: FunctionBody::Block(vec![spanned(Stmt::Let {
            name: ident("x"),
            ty: None,
            value: Box::new(spanned(int_expr(42))),
            is_mut: false,
        })]),
        is_pub: false,
        is_async: false,
        generics: vec![],
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func))],
    };

    let config = FormatConfig {
        indent_size: 4,
        max_line_length: 100,
        use_tabs: true,
    };

    let mut formatter = Formatter::new(config);
    let output = formatter.format_module(&module);

    // Should use tab indentation
    assert!(output.contains("\tx := 42"));
}

// ============================================================================
// Multiple Items Test
// ============================================================================

#[test]
fn test_format_multiple_items() {
    let module = Module {
        items: vec![
            spanned(Item::TypeAlias(TypeAlias {
                name: ident("Int"),
                ty: spanned(named_type("i64")),
                is_pub: true,
                generics: vec![],
            })),
            spanned(Item::Function(Function {
                name: ident("add"),
                params: vec![
                    Param {
                        name: ident("a"),
                        ty: spanned(named_type("Int")),
                        is_mut: false,
                    },
                    Param {
                        name: ident("b"),
                        ty: spanned(named_type("Int")),
                        is_mut: false,
                    },
                ],
                ret_type: Some(spanned(named_type("Int"))),
                body: FunctionBody::Expr(Box::new(spanned(Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(spanned(ident_expr("a"))),
                    right: Box::new(spanned(ident_expr("b"))),
                }))),
                is_pub: true,
                is_async: false,
                generics: vec![],
                attributes: vec![],
            })),
        ],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    // Should have blank line between items
    assert!(output.contains("pub T Int = i64\n\npub F add"));
}

// ============================================================================
// Use Statement Tests
// ============================================================================

#[test]
fn test_format_use_statement() {
    let use_stmt = Use {
        path: vec![ident("std"), ident("io"), ident("println")],
        alias: None,
    };

    let module = Module {
        items: vec![spanned(Item::Use(use_stmt))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("U std::io::println"));
}

#[test]
fn test_format_use_with_alias() {
    let use_stmt = Use {
        path: vec![ident("std"), ident("collections"), ident("HashMap")],
        alias: Some(ident("Map")),
    };

    let module = Module {
        items: vec![spanned(Item::Use(use_stmt))],
    };

    let mut formatter = Formatter::new(FormatConfig::default());
    let output = formatter.format_module(&module);

    assert!(output.contains("U std::collections::HashMap as Map"));
}
