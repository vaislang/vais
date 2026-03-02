//! Additional JS codegen coverage tests (part 2)
//!
//! Targets: items.rs (class/enum generation), modules.rs (ESM import/export),
//! types.rs (type mapping), stmt.rs (statement generation), sourcemap.rs,
//! and JsConfig options.

use vais_codegen_js::{JsCodeGenerator, JsConfig, JsType, SourceMap};
use vais_parser::parse;

fn gen_js(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = JsCodeGenerator::new();
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("JS codegen failed: {}", e))
}

fn gen_js_with_config(source: &str, config: JsConfig) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = JsCodeGenerator::with_config(config);
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("JS codegen failed: {}", e))
}

fn gen_js_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("{:?}", e))?;
    let mut gen = JsCodeGenerator::new();
    gen.generate_module(&module)
        .map_err(|e| format!("{}", e))
}

// ============================================================================
// JsConfig options
// ============================================================================

#[test]
fn test_js_config_default() {
    let config = JsConfig::default();
    assert!(config.use_const_let);
    let js = gen_js_with_config("F f() -> i64 = 42", config);
    assert!(!js.is_empty());
}

#[test]
fn test_js_config_var_mode() {
    let mut config = JsConfig::default();
    config.use_const_let = false;
    let js = gen_js_with_config("F f() -> i64 = 42", config);
    assert!(!js.is_empty());
}

#[test]
fn test_js_config_jsdoc() {
    let mut config = JsConfig::default();
    config.emit_jsdoc = true;
    let js = gen_js_with_config("F add(x: i64, y: i64) -> i64 = x + y", config);
    assert!(!js.is_empty());
}

#[test]
fn test_js_config_custom_indent() {
    let mut config = JsConfig::default();
    config.indent = "    ".to_string(); // 4 spaces
    let js = gen_js_with_config(
        r#"
        F f() -> i64 {
            x := 42
            x
        }
    "#,
        config,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Items: Struct/Class generation (items.rs)
// ============================================================================

#[test]
fn test_js_struct_simple() {
    let js = gen_js("S Point { x: i64, y: i64 }");
    assert!(js.contains("Point") || js.contains("class"));
}

#[test]
fn test_js_struct_single_field() {
    let js = gen_js("S Wrapper { value: i64 }");
    assert!(js.contains("Wrapper") || js.contains("value"));
}

#[test]
fn test_js_struct_with_methods() {
    let js = gen_js(
        r#"
        S Counter { n: i64 }
        X Counter {
            F get(self) -> i64 = self.n
            F inc(self) -> i64 = self.n + 1
        }
    "#,
    );
    assert!(js.contains("Counter"));
    assert!(js.contains("get") || js.contains("inc"));
}

#[test]
fn test_js_struct_many_fields() {
    let js = gen_js("S Big { a: i64, b: f64, c: bool, d: str }");
    assert!(js.contains("Big"));
}

// ============================================================================
// Items: Enum generation
// ============================================================================

#[test]
fn test_js_enum_unit_variants() {
    let js = gen_js("E Color { Red, Green, Blue }");
    assert!(js.contains("Color") || js.contains("Red"));
}

#[test]
fn test_js_enum_with_data() {
    let js = gen_js("E Shape { Circle(i64), Square(i64, i64) }");
    assert!(js.contains("Shape") || js.contains("Circle"));
}

#[test]
fn test_js_enum_variant_access() {
    let js = gen_js(
        r#"
        E Dir { North, South, East, West }
        F test() -> i64 {
            d := North
            0
        }
    "#,
    );
    assert!(js.contains("Dir") || js.contains("North"))
}

// ============================================================================
// Statements (stmt.rs)
// ============================================================================

#[test]
fn test_js_let_binding() {
    let js = gen_js("F f() -> i64 { x := 42; x }");
    assert!(js.contains("42"));
}

#[test]
fn test_js_mutable_let() {
    let js = gen_js("F f() -> i64 { x := mut 0; x = 42; x }");
    assert!(js.contains("let") || js.contains("var") || js.contains("42"));
}

#[test]
fn test_js_return_statement() {
    let js = gen_js("F f() -> i64 { R 42 }");
    assert!(js.contains("return") || js.contains("42"));
}

#[test]
fn test_js_early_return() {
    let js = gen_js(
        r#"
        F f(x: i64) -> i64 {
            I x < 0 { R -1 }
            x
        }
    "#,
    );
    assert!(js.contains("return") || js.contains("if"));
}

// ============================================================================
// Expressions (expr.rs)
// ============================================================================

#[test]
fn test_js_binary_ops() {
    let js = gen_js("F f(a: i64, b: i64) -> i64 = a + b - a * b");
    assert!(js.contains("+") || js.contains("-") || js.contains("*"));
}

#[test]
fn test_js_comparison() {
    let js = gen_js("F f(a: i64, b: i64) -> bool = a < b");
    assert!(js.contains("<"));
}

#[test]
fn test_js_logical_ops() {
    let js = gen_js("F f(a: bool, b: bool) -> bool = a && b || !a");
    assert!(js.contains("&&") || js.contains("||") || js.contains("!"));
}

#[test]
fn test_js_ternary() {
    let js = gen_js("F f(x: i64) -> i64 = x > 0 ? x : 0");
    assert!(js.contains("?") || js.contains(":") || js.contains("if"));
}

#[test]
fn test_js_string_literal() {
    let js = gen_js(r#"F f() -> str = "hello""#);
    assert!(js.contains("hello"));
}

#[test]
fn test_js_array_literal() {
    let js = gen_js("F f() -> i64 { arr := [1, 2, 3]; 0 }");
    assert!(js.contains("[") || js.contains("1"));
}

#[test]
fn test_js_if_else() {
    let js = gen_js("F f(x: i64) -> i64 = I x > 0 { 1 } E { 0 }");
    assert!(js.contains("if") || js.contains("else") || js.contains("?"));
}

#[test]
fn test_js_match() {
    let js = gen_js(
        r#"
        F f(x: i64) -> i64 = M x {
            0 => 10,
            1 => 20,
            _ => 30
        }
    "#,
    );
    assert!(js.contains("switch") || js.contains("if") || js.contains("==="));
}

#[test]
fn test_js_lambda() {
    let js = gen_js(
        r#"
        F f() -> i64 {
            g := |x: i64| x * 2
            g(21)
        }
    "#,
    );
    assert!(js.contains("=>") || js.contains("function") || js.contains("21"));
}

#[test]
fn test_js_block_expression() {
    let js = gen_js(
        r#"
        F f() -> i64 {
            result := {
                a := 10
                b := 20
                a + b
            }
            result
        }
    "#,
    );
    assert!(js.contains("10") || js.contains("20"));
}

// ============================================================================
// Type mapping (types.rs)
// ============================================================================

#[test]
fn test_js_type_number() {
    let js = gen_js("F f(x: i64) -> i64 = x");
    // i64 maps to number in JS
    assert!(!js.is_empty());
}

#[test]
fn test_js_type_boolean() {
    let js = gen_js("F f(x: bool) -> bool = x");
    assert!(!js.is_empty());
}

#[test]
fn test_js_type_string() {
    let js = gen_js("F f(x: str) -> str = x");
    assert!(!js.is_empty());
}

// ============================================================================
// JsType enum
// ============================================================================

#[test]
fn test_js_type_display() {
    assert_eq!(format!("{}", JsType::Number), "number");
    assert_eq!(format!("{}", JsType::Boolean), "boolean");
    assert_eq!(format!("{}", JsType::String), "string");
    assert_eq!(format!("{}", JsType::Void), "void");
    assert_eq!(format!("{}", JsType::Any), "any");
}

// ============================================================================
// SourceMap
// ============================================================================

#[test]
fn test_sourcemap_new() {
    let sm = SourceMap::new("test.vais", "test.js");
    assert!(!sm.to_json().is_empty());
}

#[test]
fn test_sourcemap_add_mapping() {
    let mut sm = SourceMap::new("test.vais", "test.js");
    sm.add_mapping(0, 0, 0, 0);
    sm.add_mapping(1, 0, 1, 0);
    let json = sm.to_json();
    assert!(json.contains("test.vais"));
}

#[test]
fn test_sourcemap_with_name() {
    let mut sm = SourceMap::new("main.vais", "main.js");
    sm.add_mapping(0, 0, 0, 0);
    sm.add_mapping(1, 4, 1, 2);
    let json = sm.to_json();
    assert!(json.contains("main.vais"));
    assert!(json.contains("main.js"));
}

// ============================================================================
// Loop codegen
// ============================================================================

#[test]
fn test_js_for_loop() {
    let js = gen_js(
        r#"
        F f() -> i64 {
            sum := mut 0
            L i:0..10 { sum = sum + i }
            sum
        }
    "#,
    );
    assert!(js.contains("for") || js.contains("while") || js.contains("sum"));
}

// ============================================================================
// Trait impl in JS
// ============================================================================

#[test]
fn test_js_trait_impl() {
    let js = gen_js(
        r#"
        W Show { F show(self) -> str }
        S Foo { x: i64 }
        X Foo: Show {
            F show(self) -> str = "foo"
        }
    "#,
    );
    assert!(js.contains("Foo") || js.contains("show"));
}

// ============================================================================
// Multiple functions
// ============================================================================

#[test]
fn test_js_multiple_functions() {
    let js = gen_js(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F sub(a: i64, b: i64) -> i64 = a - b
        F mul(a: i64, b: i64) -> i64 = a * b
    "#,
    );
    assert!(js.contains("add") || js.contains("sub") || js.contains("mul"));
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_js_empty_function() {
    let js = gen_js("F noop() -> i64 = 0");
    assert!(!js.is_empty());
}

#[test]
fn test_js_nested_if() {
    let js = gen_js(
        r#"
        F f(x: i64) -> i64 = I x > 0 { I x > 10 { 2 } E { 1 } } E { 0 }
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_recursive_function() {
    let js = gen_js("F fib(n: i64) -> i64 = I n < 2 { n } E { @(n-1) + @(n-2) }");
    assert!(js.contains("fib"));
}

#[test]
fn test_js_type_alias() {
    let js = gen_js(
        r#"
        T Num = i64
        F f(x: Num) -> Num = x + 1
    "#,
    );
    assert!(!js.is_empty());
}
