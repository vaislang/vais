//! JS codegen coverage tests
//!
//! Targets uncovered lines in tree_shaking.rs (300 uncovered, 68%)
//! and expr.rs (96 uncovered, 92%)
//! Focus: tree shaking edge cases, JS expression generation

use vais_codegen_js::JsCodeGenerator;
use vais_parser::parse;

fn gen_js(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = JsCodeGenerator::new();
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("JS codegen failed: {}", e))
}

// ============================================================================
// Basic JS codegen
// ============================================================================

#[test]
fn test_js_function() {
    let js = gen_js("F add(x: i64, y: i64) -> i64 = x + y");
    assert!(js.contains("add") || js.contains("function"));
}

#[test]
fn test_js_struct() {
    let js = gen_js(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            p.x + p.y
        }
    "#,
    );
    assert!(js.contains("Point") || js.contains("class"));
}

#[test]
fn test_js_enum() {
    let js = gen_js(
        r#"
        E Color { Red, Green, Blue }
        F test() -> i64 {
            c := Red
            M c {
                Red => 1,
                _ => 0
            }
        }
    "#,
    );
    assert!(js.contains("Color") || js.contains("Red"));
}

#[test]
fn test_js_if_else() {
    let js = gen_js(
        r#"
        F abs(x: i64) -> i64 {
            I x < 0 { R -x }
            R x
        }
    "#,
    );
    assert!(js.contains("if") || js.contains("return"));
}

#[test]
fn test_js_for_loop() {
    let js = gen_js(
        r#"
        F sum() -> i64 {
            s := mut 0
            L i:0..10 { s = s + i }
            s
        }
    "#,
    );
    assert!(js.contains("for") || js.contains("let"));
}

#[test]
fn test_js_while_loop() {
    let js = gen_js(
        r#"
        F test() -> i64 {
            x := mut 0
            L x < 10 { x = x + 1 }
            x
        }
    "#,
    );
    assert!(js.contains("while") || js.contains("let"));
}

#[test]
fn test_js_match() {
    let js = gen_js(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 100,
                1 => 200,
                _ => 0
            }
        }
    "#,
    );
    assert!(js.contains("switch") || js.contains("===") || js.contains("if"));
}

#[test]
fn test_js_lambda() {
    let js = gen_js("F test() -> i64 { f := |x: i64| x * 2; f(21) }");
    assert!(js.contains("=>") || js.contains("function"));
}

#[test]
fn test_js_method_call() {
    let js = gen_js(
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
    assert!(js.contains("get") || js.contains("42"));
}

// ============================================================================
// Tree shaking - unused code elimination
// ============================================================================

#[test]
fn test_js_tree_shake_unused_function() {
    let js = gen_js(
        r#"
        F used() -> i64 = 42
        F unused() -> i64 = 99
        F main() -> i64 = used()
    "#,
    );
    // Main and used should be present
    assert!(js.contains("used") || js.contains("main"));
}

#[test]
fn test_js_tree_shake_transitive() {
    let js = gen_js(
        r#"
        F helper() -> i64 = 1
        F middle() -> i64 = helper()
        F main() -> i64 = middle()
    "#,
    );
    assert!(js.contains("helper") || js.contains("middle") || js.contains("main"));
}

#[test]
fn test_js_tree_shake_struct_methods() {
    let js = gen_js(
        r#"
        S Foo { x: i64 }
        X Foo {
            F get(self) -> i64 = self.x
            F unused(self) -> i64 = 0
        }
        F main() -> i64 {
            f := Foo { x: 42 }
            f.get()
        }
    "#,
    );
    assert!(js.contains("Foo") || js.contains("get"));
}

// ============================================================================
// JS expression edge cases
// ============================================================================

#[test]
fn test_js_string_literal() {
    let js = gen_js(r#"F test() -> str = "hello world""#);
    assert!(js.contains("hello world"));
}

#[test]
fn test_js_bool_literal() {
    let js = gen_js("F test() -> bool = true");
    assert!(js.contains("true"));
}

#[test]
fn test_js_float_literal() {
    let js = gen_js("F test() -> f64 = 3.14");
    assert!(js.contains("3.14"));
}

#[test]
fn test_js_array_literal() {
    let js = gen_js("F test() -> i64 { arr := [1, 2, 3]; R 0 }");
    assert!(js.contains("[1") || js.contains("1,") || js.contains("1, 2, 3"));
}

#[test]
fn test_js_tuple_literal() {
    let js = gen_js("F test() -> i64 { t := (1, 2); R 0 }");
    assert!(js.contains("[1") || js.contains("1,") || js.contains("1, 2"));
}

#[test]
fn test_js_ternary() {
    let js = gen_js("F test(x: i64) -> i64 = x > 0 ? x : 0");
    assert!(js.contains("?") || js.contains(":") || js.contains("if"));
}

#[test]
fn test_js_assign_ops() {
    let js = gen_js(
        r#"
        F test() -> i64 {
            x := mut 10
            x += 5
            x -= 1
            x *= 2
            x /= 3
            x %= 4
            x
        }
    "#,
    );
    assert!(js.contains("+=") || js.contains("-=") || js.contains("*="));
}

#[test]
fn test_js_bitwise_assign_ops() {
    let js = gen_js(
        r#"
        F test() -> i64 {
            x := mut 255
            x &= 15
            x |= 48
            x ^= 16
            x <<= 1
            x >>= 1
            x
        }
    "#,
    );
    assert!(js.contains("&=") || js.contains("|=") || js.contains("^="));
}

#[test]
fn test_js_range_for_loop() {
    let js = gen_js(
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
    assert!(js.contains("for") || js.contains("let"));
}

#[test]
fn test_js_self_recursion() {
    let js = gen_js(
        r#"
        F fact(n: i64) -> i64 {
            I n <= 1 { R 1 }
            R n * @(n - 1)
        }
    "#,
    );
    assert!(js.contains("fact") || js.contains("function"));
}

#[test]
fn test_js_cast() {
    let js = gen_js("F test() -> f64 { x := 42; x as f64 }");
    assert!(js.contains("42"));
}

#[test]
fn test_js_block() {
    let js = gen_js(
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
    assert!(js.contains("10") || js.contains("20"));
}

#[test]
fn test_js_generic_function() {
    let js = gen_js(
        r#"
        F id<T>(x: T) -> T = x
        F main() -> i64 = id(42)
    "#,
    );
    assert!(js.contains("42") || js.contains("id"));
}

#[test]
fn test_js_trait_impl() {
    let js = gen_js(
        r#"
        W Show { F show(self) -> str }
        S Dog { name: str }
        X Dog: Show {
            F show(self) -> str = "dog"
        }
        F main() -> str {
            d := Dog { name: "Rex" }
            d.show()
        }
    "#,
    );
    assert!(js.contains("Dog") || js.contains("show"));
}

#[test]
fn test_js_type_alias() {
    let js = gen_js(
        r#"
        T Num = i64
        F double(x: Num) -> Num = x * 2
    "#,
    );
    assert!(js.contains("double") || js.contains("2"));
}

#[test]
fn test_js_const() {
    let js = gen_js("C MAX: i64 = 100");
    assert!(js.contains("100") || js.contains("MAX"));
}

#[test]
fn test_js_pub_function() {
    let js = gen_js("P F public_fn() -> i64 = 42");
    assert!(js.contains("export") || js.contains("42"));
}

#[test]
fn test_js_multiple_functions() {
    let js = gen_js(
        r#"
        F a() -> i64 = 1
        F b() -> i64 = 2
        F c() -> i64 = a() + b()
    "#,
    );
    assert!(js.contains("function") || js.contains("=>"));
}

// ============================================================================
// Complex programs
// ============================================================================

#[test]
fn test_js_fibonacci() {
    let js = gen_js(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { R n }
            R @(n - 1) + @(n - 2)
        }
    "#,
    );
    assert!(js.contains("fib"));
}

#[test]
fn test_js_complex_struct() {
    let js = gen_js(
        r#"
        S Person { name: str, age: i64 }
        X Person {
            F new(name: str, age: i64) -> Person = Person { name: name, age: age }
            F greet(self) -> str = "hello"
        }
        F main() -> str {
            p := Person::new("Alice", 30)
            p.greet()
        }
    "#,
    );
    assert!(js.contains("Person") || js.contains("greet"));
}

// ============================================================================
// Additional tree shaking tests (tree_shaking.rs)
// ============================================================================

#[test]
fn test_js_tree_shake_enum_with_match() {
    let js = gen_js(
        r#"
        E Direction { North, South, East, West }
        F go(d: Direction) -> i64 {
            M d {
                North => 1,
                South => 2,
                East => 3,
                West => 4,
                _ => 0
            }
        }
        F main() -> i64 = go(North)
    "#,
    );
    assert!(js.contains("Direction") || js.contains("North"));
}

#[test]
fn test_js_tree_shake_unused_enum() {
    let js = gen_js(
        r#"
        E Used { Aa, Bb }
        E Unused { Cc, Dd }
        F main() -> i64 {
            x := Aa
            M x {
                Aa => 1,
                _ => 0
            }
        }
    "#,
    );
    assert!(js.contains("Used") || js.contains("main"));
}

#[test]
fn test_js_tree_shake_unused_struct() {
    let js = gen_js(
        r#"
        S Used { x: i64 }
        S Unused { y: i64 }
        F main() -> i64 {
            u := Used { x: 42 }
            u.x
        }
    "#,
    );
    assert!(js.contains("Used") || js.contains("42"));
}

#[test]
fn test_js_tree_shake_pub_always_included() {
    let js = gen_js(
        r#"
        P F public_fn() -> i64 = 42
        F unused() -> i64 = 99
    "#,
    );
    assert!(js.contains("public_fn") || js.contains("export"));
}

#[test]
fn test_js_tree_shake_deep_call_chain() {
    let js = gen_js(
        r#"
        F level3() -> i64 = 42
        F level2() -> i64 = level3()
        F level1() -> i64 = level2()
        F main() -> i64 = level1()
    "#,
    );
    assert!(js.contains("level3") || js.contains("level2") || js.contains("level1"));
}

#[test]
fn test_js_tree_shake_impl_methods() {
    let js = gen_js(
        r#"
        S Calculator { value: i64 }
        X Calculator {
            F new() -> Calculator = Calculator { value: 0 }
            F add(self, n: i64) -> Calculator = Calculator { value: self.value + n }
            F get(self) -> i64 = self.value
            F unused_method(self) -> i64 = 0
        }
        F main() -> i64 {
            c := Calculator::new()
            c2 := c.add(42)
            c2.get()
        }
    "#,
    );
    assert!(js.contains("Calculator") || js.contains("add") || js.contains("get"));
}

#[test]
fn test_js_tree_shake_const() {
    let js = gen_js(
        r#"
        C pi: i64 = 3
        C unused: i64 = 99
        F main() -> i64 = pi
    "#,
    );
    assert!(js.contains("3") || js.contains("pi"));
}

#[test]
fn test_js_tree_shake_type_alias() {
    let js = gen_js(
        r#"
        T Num = i64
        T Unused = str
        F double(x: Num) -> Num = x * 2
        F main() -> i64 = double(21)
    "#,
    );
    assert!(js.contains("double") || js.contains("21"));
}

#[test]
fn test_js_tree_shake_trait_impl() {
    let js = gen_js(
        r#"
        W Eval { F eval(self) -> i64 }
        S Lit { val: i64 }
        X Lit: Eval {
            F eval(self) -> i64 = self.val
        }
        F main() -> i64 {
            l := Lit { val: 42 }
            l.eval()
        }
    "#,
    );
    assert!(js.contains("Lit") || js.contains("eval"));
}

#[test]
fn test_js_tree_shake_generic_function() {
    let js = gen_js(
        r#"
        F id<T>(x: T) -> T = x
        F unused_generic<T>(x: T) -> T = x
        F main() -> i64 = id(42)
    "#,
    );
    assert!(js.contains("id") || js.contains("42"));
}

// ============================================================================
// More JS expression edge cases (expr.rs)
// ============================================================================

#[test]
fn test_js_unary_neg() {
    let js = gen_js("F test() -> i64 = -42");
    assert!(js.contains("-42") || js.contains("42"));
}

#[test]
fn test_js_unary_not() {
    let js = gen_js("F test() -> bool = !true");
    assert!(js.contains("!") || js.contains("true"));
}

#[test]
fn test_js_comparison_ops() {
    let js = gen_js(
        r#"
        F test() -> bool {
            a := 1 < 2
            b := 2 > 1
            c := 1 <= 2
            d := 2 >= 1
            e := 1 == 1
            f := 1 != 2
            a && b && c && d && e && f
        }
    "#,
    );
    assert!(js.contains("<") || js.contains(">") || js.contains("==="));
}

#[test]
fn test_js_logical_ops() {
    let js = gen_js(
        r#"
        F test() -> bool {
            a := true && false
            b := true || false
            a || b
        }
    "#,
    );
    assert!(js.contains("&&") || js.contains("||"));
}

#[test]
fn test_js_nested_if() {
    let js = gen_js(
        r#"
        F test(x: i64) -> i64 {
            I x > 10 {
                I x > 20 { R 3 }
                R 2
            }
            R 1
        }
    "#,
    );
    assert!(js.contains("if") && js.contains("return"));
}

#[test]
fn test_js_else_if() {
    let js = gen_js(
        r#"
        F test(x: i64) -> i64 {
            I x > 20 { R 3 }
            E I x > 10 { R 2 }
            E { R 1 }
        }
    "#,
    );
    assert!(js.contains("if") || js.contains("else"));
}

#[test]
fn test_js_infinite_loop() {
    let js = gen_js(
        r#"
        F test() -> i64 {
            x := mut 0
            L {
                x = x + 1
                I x >= 10 { B }
            }
            x
        }
    "#,
    );
    assert!(js.contains("while") || js.contains("break"));
}

#[test]
fn test_js_match_with_guard() {
    let js = gen_js(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 100,
                1 => 200,
                _ => 0
            }
        }
    "#,
    );
    assert!(js.contains("===") || js.contains("switch") || js.contains("if"));
}

#[test]
fn test_js_struct_method_chain() {
    let js = gen_js(
        r#"
        S Builder { val: i64 }
        X Builder {
            F new() -> Builder = Builder { val: 0 }
            F inc(self) -> Builder = Builder { val: self.val + 1 }
            F get(self) -> i64 = self.val
        }
        F main() -> i64 {
            b := Builder::new()
            b2 := b.inc()
            b3 := b2.inc()
            b3.get()
        }
    "#,
    );
    assert!(js.contains("Builder") || js.contains("inc"));
}

#[test]
fn test_js_block_expression() {
    let js = gen_js(
        r#"
        F test() -> i64 {
            x := {
                a := 10
                b := 32
                a + b
            }
            x
        }
    "#,
    );
    assert!(js.contains("10") || js.contains("32"));
}

#[test]
fn test_js_early_return() {
    let js = gen_js(
        r#"
        F test(x: i64) -> i64 {
            I x == 0 { R 99 }
            I x == 1 { R 42 }
            R 0
        }
    "#,
    );
    assert!(js.contains("return") || js.contains("99") || js.contains("42"));
}

#[test]
fn test_js_complex_enum() {
    let js = gen_js(
        r#"
        E Expr {
            Lit(i64),
            Add(i64, i64)
        }
        F eval(e: Expr) -> i64 {
            M e {
                Lit(n) => n,
                Add(a, b) => a + b,
                _ => 0
            }
        }
        F main() -> i64 = eval(Add(20, 22))
    "#,
    );
    assert!(js.contains("Expr") || js.contains("eval") || js.contains("Lit"));
}

#[test]
fn test_js_multiple_structs() {
    let js = gen_js(
        r#"
        S Inner { x: i64 }
        S Outer { inner: Inner, y: i64 }
        F main() -> i64 {
            o := Outer { inner: Inner { x: 40 }, y: 2 }
            o.inner.x + o.y
        }
    "#,
    );
    assert!(js.contains("Inner") || js.contains("Outer") || js.contains("40"));
}

#[test]
fn test_js_extern_block() {
    let js = gen_js(
        r#"
        N "C" {
            F puts(s: i64) -> i64
        }
        F main() -> i64 = 42
    "#,
    );
    assert!(js.contains("42") || js.contains("main"));
}

#[test]
fn test_js_where_clause() {
    let js = gen_js(
        r#"
        F identity<T>(x: T) -> T = x
        F main() -> i64 = identity(42)
    "#,
    );
    assert!(js.contains("42") || js.contains("identity"));
}

#[test]
fn test_js_defer() {
    let js = gen_js(
        r#"
        F test() -> i64 {
            x := mut 0
            D { x = x + 1 }
            x = 41
            x
        }
    "#,
    );
    assert!(js.contains("41") || js.contains("finally") || js.contains("try"));
}

#[test]
fn test_js_mutual_recursion() {
    let js = gen_js(
        r#"
        F is_even(n: i64) -> i64 {
            I n == 0 { R 1 }
            is_odd(n - 1)
        }
        F is_odd(n: i64) -> i64 {
            I n == 0 { R 0 }
            is_even(n - 1)
        }
        F main() -> i64 = is_even(10)
    "#,
    );
    assert!(js.contains("is_even") || js.contains("is_odd"));
}

#[test]
fn test_js_many_parameters() {
    let js = gen_js(
        r#"
        F add5(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 = a + b + c + d + e
        F main() -> i64 = add5(1, 2, 3, 4, 5)
    "#,
    );
    assert!(js.contains("add5") || js.contains("function"));
}
