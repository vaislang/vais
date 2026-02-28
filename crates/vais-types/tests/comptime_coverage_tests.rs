//! Comprehensive compile-time evaluator coverage tests
//!
//! Targets uncovered lines in comptime.rs (341 uncovered, 57% coverage)
//! Focus: as_string, as_array, Display impl, binary/unary ops, control flow,
//! function calls (abs, min, max, pow, len), assert, index, error paths

use vais_ast::*;
use vais_types::{ComptimeEvaluator, ComptimeValue};

fn span() -> Span {
    Span::new(0, 1)
}

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned::new(node, span())
}

fn int_expr(n: i64) -> Spanned<Expr> {
    spanned(Expr::Int(n))
}

fn float_expr(f: f64) -> Spanned<Expr> {
    spanned(Expr::Float(f))
}

fn bool_expr(b: bool) -> Spanned<Expr> {
    spanned(Expr::Bool(b))
}

fn str_expr(s: &str) -> Spanned<Expr> {
    spanned(Expr::String(s.to_string()))
}

fn ident_expr(name: &str) -> Spanned<Expr> {
    spanned(Expr::Ident(name.to_string()))
}

fn binary(op: BinOp, left: Spanned<Expr>, right: Spanned<Expr>) -> Spanned<Expr> {
    spanned(Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn unary(op: UnaryOp, expr: Spanned<Expr>) -> Spanned<Expr> {
    spanned(Expr::Unary {
        op,
        expr: Box::new(expr),
    })
}

fn call_expr(name: &str, args: Vec<Spanned<Expr>>) -> Spanned<Expr> {
    spanned(Expr::Call {
        func: Box::new(ident_expr(name)),
        args,
    })
}

fn array_expr(elements: Vec<Spanned<Expr>>) -> Spanned<Expr> {
    spanned(Expr::Array(elements))
}

// ============================================================================
// ComptimeValue accessor tests
// ============================================================================

#[test]
fn test_comptime_value_as_i64_ok() {
    let val = ComptimeValue::Int(42);
    assert_eq!(val.as_i64().unwrap(), 42);
}

#[test]
fn test_comptime_value_as_i64_err() {
    let val = ComptimeValue::Bool(true);
    assert!(val.as_i64().is_err());
}

#[test]
fn test_comptime_value_as_f64_from_float() {
    let val = ComptimeValue::Float(3.14);
    assert!((val.as_f64().unwrap() - 3.14).abs() < f64::EPSILON);
}

#[test]
fn test_comptime_value_as_f64_from_int() {
    let val = ComptimeValue::Int(5);
    assert!((val.as_f64().unwrap() - 5.0).abs() < f64::EPSILON);
}

#[test]
fn test_comptime_value_as_f64_err() {
    let val = ComptimeValue::String("hello".into());
    assert!(val.as_f64().is_err());
}

#[test]
fn test_comptime_value_as_bool_ok() {
    let val = ComptimeValue::Bool(false);
    assert!(!val.as_bool().unwrap());
}

#[test]
fn test_comptime_value_as_bool_err() {
    let val = ComptimeValue::Int(1);
    assert!(val.as_bool().is_err());
}

#[test]
fn test_comptime_value_as_string_ok() {
    let val = ComptimeValue::String("hello".into());
    assert_eq!(val.as_string().unwrap(), "hello");
}

#[test]
fn test_comptime_value_as_string_err() {
    let val = ComptimeValue::Int(42);
    assert!(val.as_string().is_err());
}

#[test]
fn test_comptime_value_as_array_ok() {
    let val = ComptimeValue::Array(vec![ComptimeValue::Int(1), ComptimeValue::Int(2)]);
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn test_comptime_value_as_array_err() {
    let val = ComptimeValue::Int(42);
    assert!(val.as_array().is_err());
}

// ============================================================================
// Display impl tests
// ============================================================================

#[test]
fn test_display_int() {
    assert_eq!(format!("{}", ComptimeValue::Int(42)), "42");
}

#[test]
fn test_display_float() {
    assert_eq!(format!("{}", ComptimeValue::Float(3.14)), "3.14");
}

#[test]
fn test_display_bool() {
    assert_eq!(format!("{}", ComptimeValue::Bool(true)), "true");
}

#[test]
fn test_display_string() {
    assert_eq!(format!("{}", ComptimeValue::String("hi".into())), "\"hi\"");
}

#[test]
fn test_display_array() {
    let val = ComptimeValue::Array(vec![
        ComptimeValue::Int(1),
        ComptimeValue::Int(2),
        ComptimeValue::Int(3),
    ]);
    assert_eq!(format!("{}", val), "[1, 2, 3]");
}

#[test]
fn test_display_empty_array() {
    let val = ComptimeValue::Array(vec![]);
    assert_eq!(format!("{}", val), "[]");
}

#[test]
fn test_display_unit() {
    assert_eq!(format!("{}", ComptimeValue::Unit), "()");
}

// ============================================================================
// Eval literal tests
// ============================================================================

#[test]
fn test_eval_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&int_expr(42)).unwrap();
    assert_eq!(result, ComptimeValue::Int(42));
}

#[test]
fn test_eval_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&float_expr(2.5)).unwrap();
    assert_eq!(result, ComptimeValue::Float(2.5));
}

#[test]
fn test_eval_bool() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(eval.eval(&bool_expr(true)).unwrap(), ComptimeValue::Bool(true));
}

#[test]
fn test_eval_string() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&str_expr("hello")).unwrap();
    assert_eq!(result, ComptimeValue::String("hello".into()));
}

#[test]
fn test_eval_unit() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&spanned(Expr::Unit)).unwrap();
    assert_eq!(result, ComptimeValue::Unit);
}

#[test]
fn test_eval_array() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&array_expr(vec![int_expr(1), int_expr(2)])).unwrap();
    assert_eq!(
        result,
        ComptimeValue::Array(vec![ComptimeValue::Int(1), ComptimeValue::Int(2)])
    );
}

// ============================================================================
// Binary operator tests (integer)
// ============================================================================

#[test]
fn test_eval_add_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Add, int_expr(3), int_expr(4))).unwrap();
    assert_eq!(result, ComptimeValue::Int(7));
}

#[test]
fn test_eval_sub_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Sub, int_expr(10), int_expr(3))).unwrap();
    assert_eq!(result, ComptimeValue::Int(7));
}

#[test]
fn test_eval_mul_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Mul, int_expr(6), int_expr(7))).unwrap();
    assert_eq!(result, ComptimeValue::Int(42));
}

#[test]
fn test_eval_div_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Div, int_expr(15), int_expr(3))).unwrap();
    assert_eq!(result, ComptimeValue::Int(5));
}

#[test]
fn test_eval_div_by_zero() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Div, int_expr(10), int_expr(0)));
    assert!(result.is_err());
}

#[test]
fn test_eval_mod_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Mod, int_expr(10), int_expr(3))).unwrap();
    assert_eq!(result, ComptimeValue::Int(1));
}

#[test]
fn test_eval_mod_by_zero() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Mod, int_expr(10), int_expr(0)));
    assert!(result.is_err());
}

#[test]
fn test_eval_overflow_add() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Add, int_expr(i64::MAX), int_expr(1)));
    assert!(result.is_err());
}

#[test]
fn test_eval_overflow_sub() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Sub, int_expr(i64::MIN), int_expr(1)));
    assert!(result.is_err());
}

#[test]
fn test_eval_overflow_mul() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Mul, int_expr(i64::MAX), int_expr(2)));
    assert!(result.is_err());
}

// ============================================================================
// Binary operator tests (float)
// ============================================================================

#[test]
fn test_eval_add_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Add, float_expr(1.5), float_expr(2.5))).unwrap();
    assert_eq!(result, ComptimeValue::Float(4.0));
}

#[test]
fn test_eval_sub_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Sub, float_expr(5.0), float_expr(2.0))).unwrap();
    assert_eq!(result, ComptimeValue::Float(3.0));
}

#[test]
fn test_eval_mul_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Mul, float_expr(3.0), float_expr(4.0))).unwrap();
    assert_eq!(result, ComptimeValue::Float(12.0));
}

#[test]
fn test_eval_div_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Div, float_expr(10.0), float_expr(4.0))).unwrap();
    assert_eq!(result, ComptimeValue::Float(2.5));
}

// ============================================================================
// Comparison operators
// ============================================================================

#[test]
fn test_eval_lt() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Lt, int_expr(1), int_expr(2))).unwrap(),
        ComptimeValue::Bool(true)
    );
    assert_eq!(
        eval.eval(&binary(BinOp::Lt, int_expr(2), int_expr(1))).unwrap(),
        ComptimeValue::Bool(false)
    );
}

#[test]
fn test_eval_lte() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Lte, int_expr(2), int_expr(2))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

#[test]
fn test_eval_gt() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Gt, int_expr(3), int_expr(2))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

#[test]
fn test_eval_gte() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Gte, int_expr(2), int_expr(2))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

#[test]
fn test_eval_eq_int() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Eq, int_expr(5), int_expr(5))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

#[test]
fn test_eval_neq_int() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Neq, int_expr(5), int_expr(3))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

// ============================================================================
// String operators
// ============================================================================

#[test]
fn test_eval_string_concat() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Add, str_expr("hello"), str_expr(" world"))).unwrap();
    assert_eq!(result, ComptimeValue::String("hello world".into()));
}

#[test]
fn test_eval_string_eq() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Eq, str_expr("abc"), str_expr("abc"))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

#[test]
fn test_eval_string_neq() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Neq, str_expr("abc"), str_expr("def"))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

// ============================================================================
// Logical operators
// ============================================================================

#[test]
fn test_eval_and() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::And, bool_expr(true), bool_expr(false))).unwrap(),
        ComptimeValue::Bool(false)
    );
}

#[test]
fn test_eval_or() {
    let mut eval = ComptimeEvaluator::new();
    assert_eq!(
        eval.eval(&binary(BinOp::Or, bool_expr(false), bool_expr(true))).unwrap(),
        ComptimeValue::Bool(true)
    );
}

// ============================================================================
// Bitwise operators
// ============================================================================

#[test]
fn test_eval_bit_and() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::BitAnd, int_expr(0xFF), int_expr(0x0F))).unwrap();
    assert_eq!(result, ComptimeValue::Int(0x0F));
}

#[test]
fn test_eval_bit_or() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::BitOr, int_expr(0xF0), int_expr(0x0F))).unwrap();
    assert_eq!(result, ComptimeValue::Int(0xFF));
}

#[test]
fn test_eval_bit_xor() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::BitXor, int_expr(0xFF), int_expr(0x0F))).unwrap();
    assert_eq!(result, ComptimeValue::Int(0xF0));
}

#[test]
fn test_eval_shl() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Shl, int_expr(1), int_expr(4))).unwrap();
    assert_eq!(result, ComptimeValue::Int(16));
}

#[test]
fn test_eval_shr() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&binary(BinOp::Shr, int_expr(16), int_expr(4))).unwrap();
    assert_eq!(result, ComptimeValue::Int(1));
}

#[test]
fn test_eval_incompatible_binary_op() {
    let mut eval = ComptimeEvaluator::new();
    // bool + int should fail
    let result = eval.eval(&binary(BinOp::Add, bool_expr(true), int_expr(1)));
    assert!(result.is_err());
}

// ============================================================================
// Unary operator tests
// ============================================================================

#[test]
fn test_eval_neg_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&unary(UnaryOp::Neg, int_expr(42))).unwrap();
    assert_eq!(result, ComptimeValue::Int(-42));
}

#[test]
fn test_eval_neg_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&unary(UnaryOp::Neg, float_expr(3.14))).unwrap();
    assert_eq!(result, ComptimeValue::Float(-3.14));
}

#[test]
fn test_eval_not_bool() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&unary(UnaryOp::Not, bool_expr(true))).unwrap();
    assert_eq!(result, ComptimeValue::Bool(false));
}

#[test]
fn test_eval_bitnot_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&unary(UnaryOp::BitNot, int_expr(0))).unwrap();
    assert_eq!(result, ComptimeValue::Int(!0i64));
}

#[test]
fn test_eval_incompatible_unary() {
    let mut eval = ComptimeEvaluator::new();
    // Neg on bool should fail
    let result = eval.eval(&unary(UnaryOp::Neg, bool_expr(true)));
    assert!(result.is_err());
}

// ============================================================================
// Variable / ident tests
// ============================================================================

#[test]
fn test_eval_undefined_var() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&ident_expr("undefined_var"));
    assert!(result.is_err());
}

// ============================================================================
// Function call tests (abs, min, max, pow, len)
// ============================================================================

#[test]
fn test_eval_abs_int_positive() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("abs", vec![int_expr(5)])).unwrap();
    assert_eq!(result, ComptimeValue::Int(5));
}

#[test]
fn test_eval_abs_int_negative() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("abs", vec![int_expr(-5)])).unwrap();
    assert_eq!(result, ComptimeValue::Int(5));
}

#[test]
fn test_eval_abs_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("abs", vec![float_expr(-3.14)])).unwrap();
    assert_eq!(result, ComptimeValue::Float(3.14));
}

#[test]
fn test_eval_abs_wrong_arg_count() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("abs", vec![int_expr(1), int_expr(2)]));
    assert!(result.is_err());
}

#[test]
fn test_eval_abs_wrong_type() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("abs", vec![bool_expr(true)]));
    assert!(result.is_err());
}

#[test]
fn test_eval_min_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("min", vec![int_expr(3), int_expr(7)])).unwrap();
    assert_eq!(result, ComptimeValue::Int(3));
}

#[test]
fn test_eval_min_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("min", vec![float_expr(1.5), float_expr(2.5)])).unwrap();
    assert_eq!(result, ComptimeValue::Float(1.5));
}

#[test]
fn test_eval_min_wrong_args() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("min", vec![int_expr(1)])).is_err());
}

#[test]
fn test_eval_min_wrong_type() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("min", vec![bool_expr(true), int_expr(1)])).is_err());
}

#[test]
fn test_eval_max_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("max", vec![int_expr(3), int_expr(7)])).unwrap();
    assert_eq!(result, ComptimeValue::Int(7));
}

#[test]
fn test_eval_max_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("max", vec![float_expr(1.5), float_expr(2.5)])).unwrap();
    assert_eq!(result, ComptimeValue::Float(2.5));
}

#[test]
fn test_eval_max_wrong_args() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("max", vec![int_expr(1)])).is_err());
}

#[test]
fn test_eval_max_wrong_type() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("max", vec![str_expr("a"), str_expr("b")])).is_err());
}

#[test]
fn test_eval_pow_int() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("pow", vec![int_expr(2), int_expr(10)])).unwrap();
    assert_eq!(result, ComptimeValue::Int(1024));
}

#[test]
fn test_eval_pow_float() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("pow", vec![float_expr(2.0), float_expr(3.0)])).unwrap();
    assert_eq!(result, ComptimeValue::Float(8.0));
}

#[test]
fn test_eval_pow_negative_exponent() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("pow", vec![int_expr(2), int_expr(-1)]));
    assert!(result.is_err());
}

#[test]
fn test_eval_pow_wrong_args() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("pow", vec![int_expr(1)])).is_err());
}

#[test]
fn test_eval_pow_wrong_type() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("pow", vec![str_expr("a"), str_expr("b")])).is_err());
}

#[test]
fn test_eval_len_string() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("len", vec![str_expr("hello")])).unwrap();
    assert_eq!(result, ComptimeValue::Int(5));
}

#[test]
fn test_eval_len_array() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval
        .eval(&call_expr("len", vec![array_expr(vec![int_expr(1), int_expr(2), int_expr(3)])]))
        .unwrap();
    assert_eq!(result, ComptimeValue::Int(3));
}

#[test]
fn test_eval_len_wrong_args() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("len", vec![])).is_err());
}

#[test]
fn test_eval_len_wrong_type() {
    let mut eval = ComptimeEvaluator::new();
    assert!(eval.eval(&call_expr("len", vec![int_expr(42)])).is_err());
}

#[test]
fn test_eval_unknown_function() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&call_expr("unknown_func", vec![int_expr(1)]));
    assert!(result.is_err());
}

#[test]
fn test_eval_indirect_call_error() {
    let mut eval = ComptimeEvaluator::new();
    // Call with a non-ident as function - e.g. (1)(2)
    let result = eval.eval(&spanned(Expr::Call {
        func: Box::new(int_expr(42)),
        args: vec![int_expr(1)],
    }));
    assert!(result.is_err());
}

// ============================================================================
// Index tests
// ============================================================================

#[test]
fn test_eval_index_array() {
    let mut eval = ComptimeEvaluator::new();
    let arr = array_expr(vec![int_expr(10), int_expr(20), int_expr(30)]);
    let result = eval.eval(&spanned(Expr::Index {
        expr: Box::new(arr),
        index: Box::new(int_expr(1)),
    })).unwrap();
    assert_eq!(result, ComptimeValue::Int(20));
}

#[test]
fn test_eval_index_array_out_of_bounds() {
    let mut eval = ComptimeEvaluator::new();
    let arr = array_expr(vec![int_expr(10)]);
    let result = eval.eval(&spanned(Expr::Index {
        expr: Box::new(arr),
        index: Box::new(int_expr(5)),
    }));
    assert!(result.is_err());
}

#[test]
fn test_eval_index_array_negative() {
    let mut eval = ComptimeEvaluator::new();
    let arr = array_expr(vec![int_expr(10)]);
    let result = eval.eval(&spanned(Expr::Index {
        expr: Box::new(arr),
        index: Box::new(int_expr(-1)),
    }));
    assert!(result.is_err());
}

#[test]
fn test_eval_index_string() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&spanned(Expr::Index {
        expr: Box::new(str_expr("abc")),
        index: Box::new(int_expr(1)),
    })).unwrap();
    assert_eq!(result, ComptimeValue::String("b".into()));
}

#[test]
fn test_eval_index_string_out_of_bounds() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&spanned(Expr::Index {
        expr: Box::new(str_expr("ab")),
        index: Box::new(int_expr(5)),
    }));
    assert!(result.is_err());
}

#[test]
fn test_eval_index_non_indexable() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&spanned(Expr::Index {
        expr: Box::new(int_expr(42)),
        index: Box::new(int_expr(0)),
    }));
    assert!(result.is_err());
}

// ============================================================================
// Ternary test
// ============================================================================

#[test]
fn test_eval_ternary_true() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&spanned(Expr::Ternary {
        cond: Box::new(bool_expr(true)),
        then: Box::new(int_expr(1)),
        else_: Box::new(int_expr(2)),
    })).unwrap();
    assert_eq!(result, ComptimeValue::Int(1));
}

#[test]
fn test_eval_ternary_false() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&spanned(Expr::Ternary {
        cond: Box::new(bool_expr(false)),
        then: Box::new(int_expr(1)),
        else_: Box::new(int_expr(2)),
    })).unwrap();
    assert_eq!(result, ComptimeValue::Int(2));
}

// ============================================================================
// Comptime block (nested)
// ============================================================================

#[test]
fn test_eval_comptime_nested() {
    let mut eval = ComptimeEvaluator::new();
    let result = eval.eval(&spanned(Expr::Comptime {
        body: Box::new(int_expr(99)),
    })).unwrap();
    assert_eq!(result, ComptimeValue::Int(99));
}

// ============================================================================
// Unsupported expression
// ============================================================================

#[test]
fn test_eval_unsupported_expr() {
    let mut eval = ComptimeEvaluator::new();
    // Spawn is not comptime-evaluable
    let result = eval.eval(&spanned(Expr::Spawn(Box::new(int_expr(1)))));
    assert!(result.is_err());
}

// ============================================================================
// Default impl
// ============================================================================

#[test]
fn test_evaluator_default() {
    let eval = ComptimeEvaluator::default();
    // Should be equivalent to new()
    drop(eval);
}
