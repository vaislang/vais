//! Comprehensive codegen coverage tests
//!
//! Targets uncovered lines in:
//! - expr_visitor.rs (483 uncovered, 31%)
//! - control_flow/pattern.rs (115 uncovered, 68%)
//! - lambda_closure.rs (164 uncovered, 45%)
//! - generics_helpers.rs (88 uncovered, 44%)
//! - contracts/decreases.rs (110 uncovered, 50%)
//! - contracts/auto_checks.rs (107 uncovered, 59%)
//! - expr_helpers_control.rs (76 uncovered, 63%)
//! - type_inference.rs (66 uncovered, 80%)
//! - stmt.rs (106 uncovered, 64%)
//! - helpers.rs (76 uncovered, 68%)
//!
//! Strategy: Generate IR from Vais source code and verify it succeeds.
//! This exercises codegen paths without requiring clang/execution.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

fn gen_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen: {}", e))
}

// ============================================================================
// Expression visitor coverage (expr_visitor.rs)
// ============================================================================

#[test]
fn test_codegen_binary_arithmetic() {
    let ir = gen_ok("F test() -> i64 = 2 + 3 * 4 - 1");
    assert!(ir.contains("add") || ir.contains("mul") || ir.contains("sub"));
}

#[test]
fn test_codegen_binary_comparison() {
    let ir = gen_ok("F test() -> bool = 1 < 2");
    assert!(ir.contains("icmp"));
}

#[test]
fn test_codegen_binary_logical() {
    let ir = gen_ok("F test() -> bool = true && false");
    assert!(ir.contains("and") || ir.contains("br"));
}

#[test]
fn test_codegen_binary_bitwise() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 255
            b := a & 15
            c := b | 48
            d := c ^ 16
            e := d << 2
            f := e >> 1
            f
        }
    "#,
    );
    assert!(
        ir.contains("and")
            || ir.contains("or")
            || ir.contains("xor")
            || ir.contains("shl")
            || ir.contains("shr")
    );
}

#[test]
fn test_codegen_unary_neg() {
    let ir = gen_ok("F test() -> i64 = -42");
    assert!(ir.contains("sub") || ir.contains("42"));
}

#[test]
fn test_codegen_unary_not() {
    let ir = gen_ok("F test() -> bool = !true");
    assert!(ir.contains("xor") || ir.contains("icmp"));
}

#[test]
fn test_codegen_ternary() {
    let ir = gen_ok("F test(x: i64) -> i64 = x > 0 ? x : 0");
    assert!(ir.contains("br") || ir.contains("phi") || ir.contains("select"));
}

#[test]
fn test_codegen_string_literal() {
    let ir = gen_ok(r#"F test() -> str = "hello""#);
    assert!(ir.contains("hello") || ir.contains("str"));
}

#[test]
fn test_codegen_array_literal() {
    let ir = gen_ok("F test() -> i64 { arr := [1, 2, 3]; R 0 }");
    assert!(ir.contains("alloca") || ir.contains("store") || ir.contains("1"));
}

#[test]
fn test_codegen_tuple_literal() {
    let ir = gen_ok("F test() -> i64 { t := (10, 20); R 0 }");
    assert!(ir.contains("10") || ir.contains("20"));
}

#[test]
fn test_codegen_struct_literal() {
    let ir = gen_ok(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            p.x
        }
    "#,
    );
    assert!(ir.contains("1") || ir.contains("2"));
}

#[test]
fn test_codegen_field_access() {
    let ir = gen_ok(
        r#"
        S Pair { first: i64, second: i64 }
        F test() -> i64 {
            p := Pair { first: 10, second: 20 }
            p.first + p.second
        }
    "#,
    );
    assert!(ir.contains("add"));
}

#[test]
fn test_codegen_index_access() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            arr := [10, 20, 30]
            arr[1]
        }
    "#,
    );
    assert!(ir.contains("getelementptr") || ir.contains("load") || ir.contains("20"));
}

#[test]
fn test_codegen_method_call() {
    let ir = gen_ok(
        r#"
        S Counter { value: i64 }
        X Counter {
            F get(self) -> i64 = self.value
        }
        F test() -> i64 {
            c := Counter { value: 42 }
            c.get()
        }
    "#,
    );
    assert!(ir.contains("Counter") || ir.contains("get") || ir.contains("42"));
}

#[test]
fn test_codegen_self_recursion() {
    let ir = gen_ok(
        r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 { R 1 }
            R n * @(n - 1)
        }
    "#,
    );
    assert!(ir.contains("factorial") || ir.contains("call"));
}

#[test]
fn test_codegen_cast() {
    let ir = gen_ok("F test() -> f64 { x := 42; x as f64 }");
    assert!(ir.contains("sitofp") || ir.contains("f64"));
}

#[test]
fn test_codegen_ref_deref() {
    let ir = gen_ok("F test(x: i64) -> i64 { y := &x; *y }");
    assert!(ir.contains("alloca") || ir.contains("load") || ir.contains("store"));
}

// ============================================================================
// Control flow coverage (control_flow/)
// ============================================================================

#[test]
fn test_codegen_if_else() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 {
                R x
            } E {
                R 0
            }
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("icmp"));
}

#[test]
fn test_codegen_if_elseif_else() {
    let ir = gen_ok(
        r#"
        F classify(x: i64) -> i64 {
            I x > 0 {
                R 1
            } E I x < 0 {
                R -1
            } E {
                R 0
            }
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_codegen_for_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i:0..10 {
                sum = sum + i
            }
            sum
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("phi"));
}

#[test]
fn test_codegen_while_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            L x < 100 {
                x = x + 1
            }
            x
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_codegen_infinite_loop_break() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            L {
                x = x + 1
                I x > 10 { B }
            }
            x
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_codegen_match_int() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 100,
                1 => 200,
                2 => 300,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("switch") || ir.contains("icmp"));
}

#[test]
fn test_codegen_match_with_guard() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                n I n > 0 => n * 2,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("br"));
}

#[test]
fn test_codegen_match_tuple() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            t := (1, 2)
            M t {
                (a, b) => a + b,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("add"));
}

#[test]
fn test_codegen_nested_if() {
    let ir = gen_ok(
        r#"
        F test(x: i64, y: i64) -> i64 {
            I x > 0 {
                I y > 0 {
                    R x + y
                } E {
                    R x
                }
            } E {
                R 0
            }
        }
    "#,
    );
    assert!(ir.contains("br"));
}

// ============================================================================
// Statement codegen (stmt.rs)
// ============================================================================

#[test]
fn test_codegen_let_binding() {
    let ir = gen_ok("F test() -> i64 { x := 42; x }");
    assert!(ir.contains("42"));
}

#[test]
fn test_codegen_mut_binding() {
    let ir = gen_ok("F test() -> i64 { x := mut 0; x = 42; x }");
    assert!(ir.contains("store") || ir.contains("42"));
}

#[test]
fn test_codegen_multiple_bindings() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 1
            b := 2
            c := 3
            a + b + c
        }
    "#,
    );
    assert!(ir.contains("add"));
}

#[test]
fn test_codegen_assign_ops() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 10
            x += 5
            x -= 1
            x *= 2
            x
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("sub") || ir.contains("mul"));
}

#[test]
fn test_codegen_return_void() {
    let ir = gen_ok("F test() { R }");
    assert!(ir.contains("ret"));
}

#[test]
fn test_codegen_early_return() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x < 0 { R 0 }
            R x
        }
    "#,
    );
    assert!(ir.contains("ret"));
}

// ============================================================================
// Lambda/closure codegen (lambda_closure.rs)
// ============================================================================

#[test]
fn test_codegen_lambda_simple() {
    let ir = gen_ok("F test() -> i64 { f := |x: i64| x * 2; f(21) }");
    assert!(ir.contains("mul") || ir.contains("21"));
}

#[test]
fn test_codegen_lambda_multi_param() {
    let ir = gen_ok("F test() -> i64 { f := |x: i64, y: i64| x + y; f(1, 2) }");
    assert!(ir.contains("add"));
}

#[test]
fn test_codegen_lambda_as_param() {
    let ir = gen_ok(
        r#"
        F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)
        F test() -> i64 = apply(|x: i64| x * 2, 21)
    "#,
    );
    assert!(ir.contains("apply") || ir.contains("mul"));
}

// ============================================================================
// Generics codegen (generics_helpers.rs)
// ============================================================================

#[test]
fn test_codegen_generic_function() {
    let ir = gen_ok(
        r#"
        F id<T>(x: T) -> T = x
        F test() -> i64 = id(42)
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_codegen_generic_struct() {
    let ir = gen_ok(
        r#"
        S Box<T> { value: T }
        F test() -> i64 {
            b := Box { value: 42 }
            b.value
        }
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_codegen_generic_multiple_instantiation() {
    let ir = gen_ok(
        r#"
        F first<T>(x: T, y: T) -> T = x
        F test() -> i64 {
            a := first(1, 2)
            a
        }
    "#,
    );
    assert!(ir.contains("1"));
}

// ============================================================================
// Type inference codegen (type_inference.rs)
// ============================================================================

#[test]
fn test_codegen_inferred_types() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 42
            y := x + 1
            z := y * 2
            z
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("mul"));
}

#[test]
fn test_codegen_inferred_bool() {
    let ir = gen_ok("F test() -> bool { x := 1 < 2; x }");
    assert!(ir.contains("icmp"));
}

// ============================================================================
// Helpers (helpers.rs)
// ============================================================================

#[test]
fn test_codegen_print_call() {
    let ir = gen_ok("F test() -> i64 { print(42); R 0 }");
    assert!(ir.contains("print") || ir.contains("42"));
}

#[test]
fn test_codegen_multiple_functions() {
    let ir = gen_ok(
        r#"
        F add(x: i64, y: i64) -> i64 = x + y
        F sub(x: i64, y: i64) -> i64 = x - y
        F test() -> i64 = add(10, sub(5, 3))
    "#,
    );
    assert!(ir.contains("add") || ir.contains("sub"));
}

// ============================================================================
// Enum codegen
// ============================================================================

#[test]
fn test_codegen_enum_variant() {
    let ir = gen_ok(
        r#"
        E Color { Red, Green, Blue }
        F test() -> i64 {
            c := Red
            M c {
                Red => 1,
                Green => 2,
                Blue => 3,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("1") || ir.contains("switch") || ir.contains("icmp"));
}

#[test]
fn test_codegen_enum_with_data() {
    let ir = gen_ok(
        r#"
        E Shape {
            Circle(i64),
            Rect(i64, i64)
        }
        F area(s: Shape) -> i64 {
            M s {
                Circle(r) => r * r,
                Rect(w, h) => w * h,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("mul"));
}

// ============================================================================
// Trait codegen
// ============================================================================

#[test]
fn test_codegen_trait_impl() {
    let ir = gen_ok(
        r#"
        W Describable {
            F name(self) -> str
        }
        S Dog { breed: str }
        X Dog: Describable {
            F name(self) -> str = "dog"
        }
        F test() -> str {
            d := Dog { breed: "lab" }
            d.name()
        }
    "#,
    );
    assert!(ir.contains("dog") || ir.contains("name"));
}

#[test]
fn test_codegen_impl_methods() {
    let ir = gen_ok(
        r#"
        S Stack { top: i64 }
        X Stack {
            F new() -> Stack = Stack { top: 0 }
            F peek(self) -> i64 = self.top
        }
        F test() -> i64 {
            s := Stack::new()
            s.peek()
        }
    "#,
    );
    assert!(ir.contains("Stack") || ir.contains("new") || ir.contains("peek"));
}

// ============================================================================
// Error paths
// ============================================================================

#[test]
fn test_codegen_error_undefined_var() {
    let result = gen_result("F test() -> i64 { R undefined_xyz }");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("undefined_xyz"));
}

#[test]
fn test_codegen_error_undefined_function() {
    let result = gen_result("F test() -> i64 = nonexistent_func(42)");
    assert!(result.is_err());
}

// ============================================================================
// Complex programs
// ============================================================================

#[test]
fn test_codegen_fibonacci() {
    let ir = gen_ok(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { R n }
            R @(n - 1) + @(n - 2)
        }
        F main() -> i64 = fib(10)
    "#,
    );
    assert!(ir.contains("fib") || ir.contains("call"));
}

#[test]
fn test_codegen_gcd() {
    let ir = gen_ok(
        r#"
        F gcd(a: i64, b: i64) -> i64 {
            I b == 0 { R a }
            R @(b, a % b)
        }
    "#,
    );
    assert!(ir.contains("gcd") || ir.contains("srem"));
}

#[test]
fn test_codegen_nested_structs() {
    let ir = gen_ok(
        r#"
        S Inner { x: i64 }
        S Outer { inner: Inner, extra: i64 }
        F test() -> i64 {
            o := Outer { inner: Inner { x: 42 }, extra: 10 }
            o.inner.x + o.extra
        }
    "#,
    );
    assert!(ir.contains("42") || ir.contains("10"));
}

#[test]
fn test_codegen_complex_control_flow() {
    let ir = gen_ok(
        r#"
        F classify(x: i64) -> i64 {
            I x > 100 {
                I x > 1000 { R 3 }
                R 2
            } E I x > 0 {
                R 1
            } E I x == 0 {
                R 0
            } E {
                R -1
            }
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("ret"));
}

#[test]
fn test_codegen_accumulator_loop() {
    let ir = gen_ok(
        r#"
        F sum_to(n: i64) -> i64 {
            total := mut 0
            L i:0..n {
                total = total + i
            }
            total
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("phi"));
}

// ============================================================================
// Type alias codegen
// ============================================================================

#[test]
fn test_codegen_type_alias() {
    let ir = gen_ok(
        r#"
        T Num = i64
        F double(x: Num) -> Num = x * 2
        F test() -> i64 = double(21)
    "#,
    );
    assert!(ir.contains("mul") || ir.contains("21"));
}

// ============================================================================
// Const declaration codegen
// ============================================================================

#[test]
fn test_codegen_const() {
    let ir = gen_ok(
        r#"
        C MAX: i64 = 100
        F test() -> i64 = MAX
    "#,
    );
    assert!(ir.contains("100"));
}

// ============================================================================
// Block expression codegen
// ============================================================================

#[test]
fn test_codegen_block_expression() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := {
                a := 10
                b := 20
                a + b
            }
            x
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("10"));
}

// ============================================================================
// Break with value, continue in nested loops
// ============================================================================

#[test]
fn test_codegen_break_continue() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i:0..20 {
                I i % 2 == 0 { C }
                I i > 15 { B }
                sum = sum + i
            }
            sum
        }
    "#,
    );
    assert!(ir.contains("br"));
}

// ============================================================================
// Lambda closures (lambda_closure.rs)
// ============================================================================

#[test]
fn test_codegen_lambda_closure_capture() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 10
            f := |y: i64| x + y
            f(32)
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("10") || ir.contains("32"));
}

#[test]
fn test_codegen_lambda_no_params() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            f := || 42
            f()
        }
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_codegen_lambda_multi_capture() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 10
            b := 20
            f := |c: i64| a + b + c
            f(12)
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("12"));
}

// ============================================================================
// Generics (generics_helpers.rs)
// ============================================================================

#[test]
fn test_codegen_generic_wrapper() {
    let ir = gen_ok(
        r#"
        S Wrapper<T> { value: T }
        F test() -> i64 {
            w := Wrapper { value: 42 }
            w.value
        }
    "#,
    );
    assert!(ir.contains("42") || ir.contains("Wrapper"));
}

#[test]
fn test_codegen_generic_pair() {
    let ir = gen_ok(
        r#"
        F first<T>(a: T, b: T) -> T = a
        F test() -> i64 = first(42, 0)
    "#,
    );
    assert!(ir.contains("42") || ir.contains("first"));
}

// ============================================================================
// Additional control flow
// ============================================================================

#[test]
fn test_codegen_match_many_cases() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 100,
                1 => 200,
                2 => 300,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("switch") || ir.contains("br"));
}

#[test]
fn test_codegen_deeply_nested_if() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 {
                I x > 10 {
                    I x > 20 {
                        R x
                    }
                    R 20
                }
                R 10
            }
            R 0
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("br"));
}

#[test]
fn test_codegen_loop_accumulator() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            result := mut 1
            L i:1..11 {
                result = result * i
            }
            result
        }
    "#,
    );
    assert!(ir.contains("mul") || ir.contains("phi") || ir.contains("br"));
}

// ============================================================================
// String operations
// ============================================================================

#[test]
fn test_codegen_string_return() {
    let ir = gen_ok(
        r#"
        F test() -> str = "hello world"
    "#,
    );
    assert!(ir.contains("hello world") || ir.contains("global") || ir.contains("constant"));
}

// ============================================================================
// Multiple return points
// ============================================================================

#[test]
fn test_codegen_multiple_returns() {
    let ir = gen_ok(
        r#"
        F classify(n: i64) -> i64 {
            I n < 0 { R -1 }
            I n == 0 { R 0 }
            I n < 10 { R 1 }
            I n < 100 { R 2 }
            R 3
        }
    "#,
    );
    assert!(ir.contains("ret"));
}

// ============================================================================
// Comparison operators
// ============================================================================

#[test]
fn test_codegen_all_comparisons() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 1 < 2
            b := 2 > 1
            c := 1 <= 2
            d := 2 >= 1
            e := 1 == 1
            f := 1 != 2
            I a && b && c && d && e && f { R 1 }
            R 0
        }
    "#,
    );
    assert!(ir.contains("icmp slt") || ir.contains("icmp sgt") || ir.contains("icmp eq"));
}

// ============================================================================
// Nested struct construction
// ============================================================================

#[test]
fn test_codegen_nested_struct_construction() {
    let ir = gen_ok(
        r#"
        S Inner { x: i64 }
        S Outer { inner: Inner, y: i64 }
        F test() -> i64 {
            o := Outer { inner: Inner { x: 40 }, y: 2 }
            o.inner.x + o.y
        }
    "#,
    );
    assert!(
        ir.contains("insertvalue")
            || ir.contains("extractvalue")
            || ir.contains("Inner")
            || ir.contains("Outer")
    );
}

// ============================================================================
// Complex call chains
// ============================================================================

#[test]
fn test_codegen_call_chain() {
    let ir = gen_ok(
        r#"
        F inc(x: i64) -> i64 = x + 1
        F dbl(x: i64) -> i64 = x * 2
        F test() -> i64 = inc(dbl(inc(dbl(5))))
    "#,
    );
    assert!(ir.contains("call") || ir.contains("inc") || ir.contains("dbl"));
}

// ============================================================================
// Complex enum patterns
// ============================================================================

#[test]
fn test_codegen_enum_wildcard_match() {
    let ir = gen_ok(
        r#"
        E Animal { Cat, Dog, Fish, Bird }
        F score(a: Animal) -> i64 {
            M a {
                Cat => 4,
                Dog => 3,
                _ => 1
            }
        }
        F test() -> i64 = score(Fish)
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("br"));
}

// ============================================================================
// Expression-body with complex expressions
// ============================================================================

#[test]
fn test_codegen_expression_body_complex() {
    let ir = gen_ok(
        r#"
        F test(a: i64, b: i64) -> i64 = (a + b) * (a - b)
    "#,
    );
    assert!(ir.contains("add") && ir.contains("sub") && ir.contains("mul"));
}

#[test]
fn test_codegen_expression_body_ternary() {
    let ir = gen_ok(
        r#"
        F max(a: i64, b: i64) -> i64 = a > b ? a : b
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("select") || ir.contains("br"));
}

// ============================================================================
// Mutual recursion
// ============================================================================

#[test]
fn test_codegen_mutual_recursion() {
    let ir = gen_ok(
        r#"
        F is_even(n: i64) -> i64 {
            I n == 0 { R 1 }
            is_odd(n - 1)
        }
        F is_odd(n: i64) -> i64 {
            I n == 0 { R 0 }
            is_even(n - 1)
        }
    "#,
    );
    assert!(ir.contains("is_even") && ir.contains("is_odd"));
}

// ============================================================================
// Defer codegen
// ============================================================================

#[test]
fn test_codegen_defer() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            D { x = x + 1 }
            x = 41
            x
        }
    "#,
    );
    assert!(ir.contains("store") || ir.contains("41"));
}

// ============================================================================
// Extern function declaration
// ============================================================================

#[test]
fn test_codegen_extern_declaration() {
    let ir = gen_ok(
        r#"
        N "C" {
            F puts(s: i64) -> i64
        }
        F test() -> i64 = 42
    "#,
    );
    assert!(ir.contains("declare") || ir.contains("puts") || ir.contains("42"));
}
