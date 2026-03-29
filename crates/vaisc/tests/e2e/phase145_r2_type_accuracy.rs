//! Phase 145: R2 IR Type Accuracy — float coercion + lazy/force type registration
//!
//! Tests for:
//! 1. float parameter correctness in functions and methods
//! 2. f32 <-> f64 coercion in arithmetic and assignments
//! 3. str parameter passed as fat pointer in impl methods
//! 4. lazy/force expression type tracking (register_temp_type)

use super::helpers::*;

// ==================== 1. Float parameter accuracy ====================

#[test]
fn e2e_p145_f64_param_basic() {
    // f64 parameter passed and returned correctly
    let source = r#"
F identity_f64(x: f64) -> f64 {
    x
}

F main() -> i64 {
    v := identity_f64(3.0)
    I v > 2.0 { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p145_f32_param_basic() {
    // f32 parameter passed and returned correctly
    // Phase 158: explicit f32/f64 cast required
    let source = r#"
F identity_f32(x: f32) -> f32 {
    x
}

F main() -> i64 {
    v := identity_f32(5.0 as f32)
    I v > (4.0 as f32) { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p145_f64_method_param() {
    // f64 parameter in an impl method is correctly handled
    let source = r#"
S Circle {
    radius: f64
}

X Circle {
    F area(self) -> f64 {
        self.radius * self.radius
    }
}

F main() -> i64 {
    c := Circle { radius: 3.0 }
    a := c.area()
    I a > 8.0 { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p145_f64_method_arg() {
    // f64 argument passed to method is correctly typed
    let source = r#"
S Scale {
    factor: f64
}

X Scale {
    F apply(self, x: f64) -> f64 {
        self.factor * x
    }
}

F main() -> i64 {
    s := Scale { factor: 2.0 }
    result := s.apply(5.0)
    I result > 9.0 { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

// ==================== 2. f32 <-> f64 coercion ====================

#[test]
fn e2e_p145_f32_arithmetic() {
    // f32 arithmetic should compile and produce correct results
    // Phase 158: explicit f32/f64 cast required
    let source = r#"
F add_f32(a: f32, b: f32) -> f32 {
    a + b
}

F main() -> i64 {
    result := add_f32(3.0 as f32, 4.0 as f32)
    I result > (6.0 as f32) { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p145_f64_arithmetic() {
    // f64 arithmetic should compile and produce correct results
    let source = r#"
F mul_f64(a: f64, b: f64) -> f64 {
    a * b
}

F main() -> i64 {
    result := mul_f64(3.0, 4.0)
    I result > 11.0 { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p145_float_coerce_ir_fpext() {
    // IR for f32->f64 coercion must contain fpext instruction
    // Phase 158: explicit f32/f64 cast required
    let source = r#"
F widen(x: f32) -> f64 {
    x as f64
}

F main() -> i64 {
    R 0
}
"#;
    // This verifies the IR compiles without error (fpext is emitted by cast codegen)
    assert_compiles(source);
}

#[test]
fn e2e_p145_float_coerce_ir_fptrunc() {
    // IR for f64->f32 coercion must contain fptrunc instruction
    // Phase 158: explicit f32/f64 cast required
    let source = r#"
F narrow(x: f64) -> f32 {
    x as f32
}

F main() -> i64 {
    R 0
}
"#;
    // This verifies the IR compiles without error (fptrunc is emitted by cast codegen)
    assert_compiles(source);
}

// ==================== 3. str parameter fat pointer in impl methods ====================

#[test]
fn e2e_p145_str_param_in_method() {
    // str parameter in impl method should work as fat pointer
    let source = r#"
S Greeter {
    val: i64
}

X Greeter {
    F greet(self, _name: str) -> i64 {
        self.val
    }
}

F main() -> i64 {
    g := Greeter { val: 42 }
    result := g.greet("world")
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145_str_param_function() {
    // str parameter passed to regular function
    let source = r#"
F get_len(_s: str) -> i64 {
    # str fat pointer: { i8*, i64 } — return fixed value for test
    7
}

F main() -> i64 {
    l := get_len("hello")
    I l > 0 { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

// ==================== 4. lazy/force type tracking ====================

#[test]
fn e2e_p145_lazy_force_i64() {
    // lazy/force with i64 — result should be trackable
    let source = r#"
F main() -> i64 {
    x := lazy { 42 }
    result := force x
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145_lazy_force_bool() {
    // lazy/force with bool — type should be registered correctly
    let source = r#"
F main() -> i64 {
    flag := lazy { true }
    v := force flag
    I v { R 1 }
    R 0
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p145_lazy_force_expression() {
    // lazy captures outer variable, force evaluates it
    let source = r#"
F main() -> i64 {
    base := 10
    doubled := lazy { base * 2 }
    result := force doubled
    result
}
"#;
    assert_exit_code(source, 20);
}

#[test]
fn e2e_p145_lazy_type_registration_ir() {
    // Verify that lazy/force compiles without IR type errors
    let source = r#"
F compute(x: i64) -> i64 {
    x + 1
}

F main() -> i64 {
    n := 41
    lazy_val := lazy { compute(n) }
    result := force lazy_val
    result
}
"#;
    assert_exit_code(source, 42);
}
