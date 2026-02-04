//! Phase 34 Stage 1: Stability Tests
//!
//! Tests for strict borrow checker default, ICE handling, and crash recovery.

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Compile Vais source to LLVM IR string with strict ownership
fn compile_strict(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker.set_strict_ownership(true);
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("strict_test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen
        .generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

/// Compile Vais source to LLVM IR with warn-only ownership
fn compile_warn_only(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    // warn-only: ownership issues are warnings, not errors
    checker.set_strict_ownership(false);
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("warn_test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen
        .generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

// ============================================
// Strict mode: valid programs should compile
// ============================================

#[test]
fn strict_basic_arithmetic() {
    let source = r#"
F main() -> i64 {
    x := 10
    y := 20
    x + y
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("define i64 @main"));
}

#[test]
fn strict_function_calls() {
    let source = r#"
F add(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    add(3, 4)
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("define i64 @add"));
}

#[test]
fn strict_struct_creation() {
    let source = r#"
S Point {
    x: i64,
    y: i64
}

F main() -> i64 {
    p := Point { x: 10, y: 20 }
    p.x + p.y
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("Point"));
}

#[test]
fn strict_conditionals_and_loops() {
    let source = r#"
F main() -> i64 {
    sum := 0
    i := 0
    L i < 10 {
        sum = sum + i
        i = i + 1
    }
    sum
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("define i64 @main"));
}

#[test]
fn strict_match_expression() {
    let source = r#"
F classify(x: i64) -> i64 {
    M x {
        0 => 0,
        1 => 1,
        _ => 2
    }
}

F main() -> i64 {
    classify(5)
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("define i64 @classify"));
}

#[test]
fn strict_string_constants() {
    let source = r#"
F main() -> i64 {
    msg := "hello"
    0
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("hello"));
}

#[test]
fn strict_nested_structs() {
    let source = r#"
S Inner {
    value: i64
}

S Outer {
    a: Inner,
    b: i64
}

F main() -> i64 {
    inner := Inner { value: 42 }
    outer := Outer { a: inner, b: 10 }
    outer.a.value + outer.b
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("Inner"));
    assert!(ir.contains("Outer"));
}

#[test]
fn strict_impl_methods() {
    let source = r#"
S Counter {
    count: i64
}

X Counter {
    F new() -> Counter {
        Counter { count: 0 }
    }

    F get(&self) -> i64 {
        self.count
    }
}

F main() -> i64 {
    c := Counter::new()
    c.get()
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("Counter"));
}

#[test]
fn strict_recursive_function() {
    let source = r#"
F factorial(n: i64) -> i64 {
    I n <= 1 {
        1
    } E {
        n * @(n - 1)
    }
}

F main() -> i64 {
    factorial(5)
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("factorial"));
}

#[test]
fn strict_constants() {
    let source = r#"
C MAX_SIZE: i64 = 100
C PI_APPROX: f64 = 3.14

F main() -> i64 {
    MAX_SIZE
}
"#;
    let ir = compile_strict(source).expect("Should compile in strict mode");
    assert!(ir.contains("100"));
}

// ============================================
// Crash recovery: compiler should not panic on bad input
// ============================================

#[test]
fn no_crash_on_empty_source() {
    // Empty source should produce a reasonable error or empty module
    let result = compile_strict("");
    // Should not panic - either Ok or Err is fine
    let _ = result;
}

#[test]
fn no_crash_on_deeply_nested_expressions() {
    // Build a moderately nested expression: ((((1 + 1) + 1) + 1) ...)
    // Keep depth low to avoid stack overflow (parser is recursive descent)
    let mut source = String::from("F main() -> i64 {\n    ");
    let depth = 10;
    for _ in 0..depth {
        source.push('(');
    }
    source.push('1');
    for _ in 0..depth {
        source.push_str(" + 1)");
    }
    source.push_str("\n}\n");

    let result = compile_strict(&source);
    assert!(result.is_ok(), "Moderate nesting should compile: {:?}", result.err());
}

#[test]
fn no_crash_on_invalid_syntax() {
    let source = "F main( { }}}}";
    let result = compile_strict(source);
    // Should return Err, not panic
    assert!(result.is_err());
}

#[test]
fn no_crash_on_type_mismatch() {
    let source = r#"
F main() -> i64 {
    x := "hello"
    x + 5
}
"#;
    let result = compile_strict(source);
    // Should return Err for type mismatch, not panic
    assert!(result.is_err());
}
