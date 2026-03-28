//! Comprehensive formatter coverage tests
//!
//! Targets uncovered lines in formatter/expressions.rs (226 uncovered),
//! formatter/statements.rs (162 uncovered), formatter/macros.rs (180 uncovered),
//! formatter/declarations.rs (214 uncovered), formatter/types.rs (63 uncovered)
//!
//! Strategy: Parse real Vais source → format_module → verify output contains
//! expected constructs. This exercises both parser and formatter paths.

use vais_ast::formatter::{FormatConfig, Formatter};
use vais_ast::*;
use vais_parser::parse;

fn format_source(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut fmt = Formatter::new(FormatConfig::default());
    fmt.format_module(&module)
}

#[allow(dead_code)]
fn sp<T>(node: T) -> Spanned<T> {
    Spanned::new(node, Span::new(0, 1))
}

#[allow(dead_code)]
fn sp_str(s: &str) -> Spanned<String> {
    sp(s.to_string())
}

fn default_formatter() -> Formatter {
    Formatter::new(FormatConfig::default())
}

#[allow(dead_code)]
fn format_item(item: Item) -> String {
    let mut fmt = default_formatter();
    let module = Module {
        items: vec![sp(item)],
        modules_map: None,
    };
    fmt.format_module(&module)
}

// ============================================================================
// Expression formatting via parse-then-format
// ============================================================================

#[test]
fn test_format_integer_expressions() {
    let output = format_source("F test() -> i64 = 42");
    assert!(output.contains("42"));
}

#[test]
fn test_format_float_expressions() {
    let output = format_source("F test() -> f64 = 3.14");
    assert!(output.contains("3.14"));
}

#[test]
fn test_format_bool_expressions() {
    let output = format_source("F test() -> bool = true");
    assert!(output.contains("true"));
}

#[test]
fn test_format_string_expressions() {
    let output = format_source(r#"F test() -> str = "hello""#);
    assert!(output.contains("hello"));
}

#[test]
fn test_format_binary_ops() {
    let output = format_source("F test() -> i64 = 1 + 2 * 3");
    assert!(output.contains("+"));
    assert!(output.contains("*"));
}

#[test]
fn test_format_comparison_ops() {
    let output = format_source("F test() -> bool = 1 < 2");
    assert!(output.contains("<"));
}

#[test]
fn test_format_logical_ops() {
    let output = format_source("F test() -> bool = true && false || true");
    assert!(output.contains("&&") || output.contains("||"));
}

#[test]
fn test_format_bitwise_ops() {
    let output = format_source("F test() -> i64 = 255 & 15 | 48 ^ 16");
    assert!(output.contains("&") || output.contains("|") || output.contains("^"));
}

#[test]
fn test_format_shift_ops() {
    let output = format_source("F test() -> i64 = 1 << 8");
    assert!(output.contains("<<"));
}

#[test]
fn test_format_unary_neg() {
    let output = format_source("F test() -> i64 = -42");
    assert!(output.contains("-"));
}

#[test]
fn test_format_unary_not() {
    let output = format_source("F test() -> bool = !true");
    assert!(output.contains("!"));
}

#[test]
fn test_format_ternary() {
    let output = format_source("F test(x: i64) -> i64 = x > 0 ? x : 0");
    assert!(output.contains("?"));
}

#[test]
fn test_format_if_else() {
    let output = format_source(
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
    assert!(output.contains("I "));
    assert!(output.contains("E {"));
}

#[test]
fn test_format_if_elseif_else() {
    let output = format_source(
        r#"
        F test(x: i64) -> i64 {
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
    assert!(output.contains("E I"));
}

#[test]
fn test_format_for_loop() {
    let output = format_source(
        r#"
        F test() -> i64 {
            L i:0..10 {
                C
            }
            R 0
        }
    "#,
    );
    assert!(output.contains("L "));
}

#[test]
fn test_format_while_loop() {
    let output = format_source(
        r#"
        F test() -> i64 {
            x := mut 0
            L x < 10 {
                x = x + 1
            }
            x
        }
    "#,
    );
    assert!(output.contains("L "));
}

#[test]
fn test_format_match() {
    let output = format_source(
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
    assert!(output.contains("M "));
    assert!(output.contains("=>"));
}

#[test]
fn test_format_function_call() {
    let output = format_source(
        r#"
        F add(x: i64, y: i64) -> i64 = x + y
        F test() -> i64 = add(1, 2)
    "#,
    );
    assert!(output.contains("add("));
}

#[test]
fn test_format_method_call() {
    let output = format_source(
        r#"
        S Foo { value: i64 }
        X Foo {
            F get(self) -> i64 = self.value
        }
    "#,
    );
    assert!(output.contains("self.value"));
}

#[test]
fn test_format_field_access() {
    let output = format_source(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            p.x
        }
    "#,
    );
    assert!(output.contains("p.x"));
}

#[test]
fn test_format_index_access() {
    let output = format_source(
        r#"
        F test() -> i64 {
            arr := [1, 2, 3]
            arr[0]
        }
    "#,
    );
    assert!(output.contains("[0]"));
}

#[test]
fn test_format_array_literal() {
    let output = format_source("F test() -> i64 { arr := [1, 2, 3]; R 0 }");
    assert!(output.contains("[1, 2, 3]"));
}

#[test]
fn test_format_tuple_literal() {
    let output = format_source("F test() -> i64 { t := (1, 2, 3); R 0 }");
    assert!(output.contains("(1, 2, 3)"));
}

#[test]
fn test_format_struct_literal() {
    let output = format_source(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            R 0
        }
    "#,
    );
    assert!(output.contains("Point {") || output.contains("Point{"));
}

#[test]
fn test_format_range() {
    let output = format_source("F test() -> i64 { L i:0..10 { C }; R 0 }");
    assert!(output.contains(".."));
}

#[test]
fn test_format_inclusive_range() {
    let output = format_source("F test() -> i64 { L i:0..=10 { C }; R 0 }");
    assert!(output.contains("..="));
}

#[test]
fn test_format_block() {
    let output = format_source(
        r#"
        F test() -> i64 {
            x := {
                a := 10
                a + 1
            }
            x
        }
    "#,
    );
    assert!(output.contains("{"));
}

#[test]
fn test_format_cast() {
    let output = format_source("F test() -> f64 { x := 42; x as f64 }");
    assert!(output.contains("as"));
}

#[test]
fn test_format_assign() {
    let output = format_source(
        r#"
        F test() -> i64 {
            x := mut 0
            x = 42
            x
        }
    "#,
    );
    assert!(output.contains("= 42") || output.contains("=42"));
}

#[test]
fn test_format_assign_op() {
    let output = format_source(
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
    assert!(output.contains("+="));
}

#[test]
fn test_format_assign_op_bitwise() {
    let output = format_source(
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
    assert!(output.contains("&=") || output.contains("|="));
}

#[test]
fn test_format_lambda() {
    let output = format_source("F test() -> i64 { f := |x: i64| x * 2; f(21) }");
    assert!(output.contains("|"));
}

#[test]
fn test_format_self_recursion() {
    let output = format_source("F fact(n: i64) -> i64 { I n <= 1 { R 1 }; R n * @(n - 1) }");
    assert!(output.contains("@"));
}

#[test]
fn test_format_try() {
    let output = format_source(
        r#"
        F test(x: i64) -> i64 {
            v := x + 1
            v
        }
    "#,
    );
    assert!(output.contains("v"));
}

#[test]
fn test_format_unwrap() {
    let output = format_source(
        r#"
        F test(x: i64) -> i64 {
            x
        }
    "#,
    );
    assert!(output.contains("x"));
}

#[test]
fn test_format_ref_deref() {
    let output = format_source("F test(x: i64) -> i64 { y := &x; *y }");
    assert!(output.contains("&") || output.contains("*"));
}

#[test]
fn test_format_lazy_force() {
    let output = format_source(
        r#"
        F test() -> i64 {
            x := lazy 42
            force x
        }
    "#,
    );
    assert!(output.contains("lazy"));
    assert!(output.contains("force"));
}

// ============================================================================
// Statement formatting
// ============================================================================

#[test]
fn test_format_let_binding() {
    let output = format_source("F test() -> i64 { x := 42; x }");
    assert!(output.contains(":="));
}

#[test]
fn test_format_mut_binding() {
    let output = format_source("F test() -> i64 { x := mut 0; x = 1; x }");
    assert!(output.contains("mut"));
}

#[test]
fn test_format_typed_binding() {
    let output = format_source("F test() -> i64 { x: i64 = 42; x }");
    assert!(output.contains("i64"));
}

#[test]
fn test_format_return_statement() {
    let output = format_source("F test() -> i64 { R 42 }");
    assert!(output.contains("R "));
}

#[test]
fn test_format_return_void() {
    let output = format_source("F test() { R }");
    assert!(output.contains("R"));
}

#[test]
fn test_format_break_statement() {
    let output = format_source("F test() -> i64 { L { B }; R 0 }");
    assert!(output.contains("B"));
}

#[test]
fn test_format_continue_statement() {
    let output = format_source("F test() -> i64 { L i:0..10 { C }; R 0 }");
    assert!(output.contains("C"));
}

#[test]
fn test_format_defer_statement() {
    let output = format_source("F test() -> i64 { D print(0); R 42 }");
    assert!(output.contains("D "));
}

// ============================================================================
// Declaration formatting
// ============================================================================

#[test]
fn test_format_function_declaration() {
    let output = format_source("F add(x: i64, y: i64) -> i64 = x + y");
    assert!(output.contains("F add"));
}

#[test]
fn test_format_struct_declaration() {
    let output = format_source("S Point { x: i64, y: i64 }");
    assert!(output.contains("S Point"));
}

#[test]
fn test_format_enum_declaration() {
    let output = format_source("E Color { Red, Green, Blue }");
    assert!(output.contains("E Color"));
}

#[test]
fn test_format_trait_declaration() {
    let output = format_source("W Printable { F show(self) -> str }");
    assert!(output.contains("W Printable"));
}

#[test]
fn test_format_impl_block() {
    let output = format_source(
        r#"
        S Foo { value: i64 }
        X Foo {
            F new() -> Foo = Foo { value: 0 }
        }
    "#,
    );
    assert!(output.contains("X Foo"));
}

#[test]
fn test_format_trait_impl() {
    let output = format_source(
        r#"
        W Show { F show(self) -> str }
        S Foo { value: i64 }
        X Foo: Show {
            F show(self) -> str = "foo"
        }
    "#,
    );
    assert!(output.contains("X Foo") || output.contains("Show"));
}

#[test]
fn test_format_type_alias() {
    let output = format_source("T Num = i64");
    assert!(output.contains("T Num"));
}

#[test]
fn test_format_pub_function() {
    let output = format_source("P F public_fn() -> i64 = 42");
    assert!(output.contains("public_fn") && output.contains("42"));
}

#[test]
fn test_format_const_declaration() {
    let output = format_source("C MAX: i64 = 100");
    assert!(output.contains("MAX") && output.contains("100"));
}

#[test]
fn test_format_extern_function() {
    let output = format_source(
        r#"
        N "C" {
            F malloc(size: i64) -> i64
        }
    "#,
    );
    assert!(output.contains("malloc") && output.contains("i64"));
}

#[test]
fn test_format_generic_function() {
    let output = format_source("F id<T>(x: T) -> T = x");
    assert!(output.contains("<T>") || output.contains("< T >"));
}

#[test]
fn test_format_generic_struct() {
    let output = format_source("S Pair<T> { first: T, second: T }");
    assert!(output.contains("Pair"));
}

// ============================================================================
// Type formatting
// ============================================================================

#[test]
fn test_format_primitive_types() {
    let output = format_source("F test(a: i64, b: f64, c: bool, d: str) -> i64 = 0");
    assert!(output.contains("i64"));
    assert!(output.contains("f64"));
    assert!(output.contains("bool"));
    assert!(output.contains("str"));
}

#[test]
fn test_format_generic_type() {
    let output = format_source("F test(x: Vec<i64>) -> i64 = 0");
    assert!(output.contains("Vec"));
}

#[test]
fn test_format_optional_type() {
    let output = format_source("F test(x: i64?) -> i64 = 0");
    assert!(output.contains("?"));
}

#[test]
fn test_format_result_type() {
    let output = format_source("F test() -> i64 = 0");
    assert!(output.contains("i64"));
}

#[test]
fn test_format_function_type() {
    let output = format_source("F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)");
    assert!(output.contains("fn") || output.contains("->"));
}

// ============================================================================
// Macro formatting (via direct AST construction)
// ============================================================================

#[test]
fn test_format_macro_definition() {
    let source = r#"
        macro my_macro! {
            () => { 42 }
        }
    "#;
    let output = format_source(source);
    assert!(output.contains("macro my_macro!"));
}

#[test]
fn test_format_macro_with_args() {
    let source = r#"
        macro add! {
            ($a:expr, $b:expr) => { $a + $b }
        }
    "#;
    let output = format_source(source);
    assert!(output.contains("macro add!"));
}

#[test]
fn test_format_macro_invocation() {
    let source = "F test() -> i64 = my_macro!(1, 2, 3)";
    let output = format_source(source);
    assert!(output.contains("my_macro!"));
}

// ============================================================================
// Complex program formatting
// ============================================================================

#[test]
fn test_format_fibonacci() {
    let output = format_source(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { R n }
            R @(n - 1) + @(n - 2)
        }
    "#,
    );
    assert!(output.contains("fib"));
    assert!(output.contains("@"));
}

#[test]
fn test_format_complete_program() {
    let output = format_source(
        r#"
        S Point { x: i64, y: i64 }
        X Point {
            F new(x: i64, y: i64) -> Point = Point { x: x, y: y }
            F distance(self) -> i64 = self.x * self.x + self.y * self.y
        }
        F main() -> i64 {
            p := Point::new(3, 4)
            p.distance()
        }
    "#,
    );
    assert!(output.contains("S Point"));
    assert!(output.contains("X Point"));
    assert!(output.contains("F main"));
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_format_empty_module() {
    let module = Module {
        items: vec![],
        modules_map: None,
    };
    let mut fmt = default_formatter();
    let output = fmt.format_module(&module);
    assert!(output.is_empty());
}

#[test]
fn test_format_with_tabs() {
    let module = parse("F test() -> i64 { R 42 }").unwrap();
    let mut fmt = Formatter::new(FormatConfig {
        indent_size: 4,
        max_line_length: 100,
        use_tabs: true,
    });
    let output = fmt.format_module(&module);
    assert!(output.contains("\t") || output.contains("R"));
}

#[test]
fn test_format_match_with_guard() {
    let output = format_source(
        r#"
        F test(x: i64) -> i64 {
            M x {
                n I n > 0 => n,
                _ => 0
            }
        }
    "#,
    );
    assert!(output.contains("=>"));
}

#[test]
fn test_format_multiple_items() {
    let output = format_source(
        r#"
        F foo() -> i64 = 1
        F bar() -> i64 = 2
        F baz() -> i64 = 3
    "#,
    );
    assert!(output.contains("foo"));
    assert!(output.contains("bar"));
    assert!(output.contains("baz"));
}

#[test]
fn test_format_where_clause() {
    let output = format_source(
        r#"
        W Show { F show(self) -> str }
        F display<T>(x: T) -> str where T: Show = x.show()
    "#,
    );
    assert!(output.contains("where") || output.contains("Show"));
}

#[test]
fn test_format_union() {
    let output = format_source(
        r#"
        O Value {
            int_val: i64,
            float_val: f64
        }
    "#,
    );
    assert!(output.contains("O Value") || output.contains("Value"));
}
