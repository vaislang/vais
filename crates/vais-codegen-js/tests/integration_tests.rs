//! Integration tests for vais-codegen-js crate
//!
//! This test suite validates the JavaScript code generation from Vais source code.
//! Tests cover functions, expressions, control flow, structs, enums, error handling,
//! modules, tree shaking, and source maps.

use vais_codegen_js::{JsCodeGenerator, JsConfig, SourceMap};

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse Vais source and generate JavaScript
fn parse_and_generate(source: &str) -> String {
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut gen = JsCodeGenerator::new();
    gen.generate_module(&module).expect("Codegen failed")
}

/// Parse with custom config
fn parse_and_generate_with_config(source: &str, config: JsConfig) -> String {
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut gen = JsCodeGenerator::with_config(config);
    gen.generate_module(&module).expect("Codegen failed")
}

// ============================================================================
// 1. Basic Functions (4 tests)
// ============================================================================

#[test]
fn test_simple_function_definition() {
    let source = r#"
        F add(a: i64, b: i64) -> i64 { a + b }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("function add(a, b)"));
    assert!(js.contains("return"));
    assert!(js.contains("a + b"));
}

#[test]
fn test_multiple_parameter_function() {
    let source = r#"
        F calculate(x: i64, y: i64, z: i64) -> i64 {
            x * y + z
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("function calculate(x, y, z)"));
    assert!(js.contains("x * y"));
    assert!(js.contains("+ z"));
}

#[test]
fn test_void_return_function() {
    let source = r#"
        F print_hello() {
            42
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("function print_hello()"));
    assert!(js.contains("return"));
}

#[test]
fn test_recursive_function_with_selfcall() {
    let source = r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 {
                R 1
            }
            n * @(n - 1)
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("function factorial(n)"));
    assert!(js.contains("factorial("));
}

// ============================================================================
// 2. Expressions (6 tests)
// ============================================================================

#[test]
fn test_arithmetic_and_comparison_operators() {
    let source = r#"
        F test() -> bool {
            (5 + 3) * 2 == 16
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("5 + 3"));
    assert!(js.contains("* 2"));
    assert!(js.contains("==="));
}

#[test]
fn test_ternary_operator() {
    let source = r#"
        F max(a: i64, b: i64) -> i64 {
            a > b ? a : b
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("function max(a, b)"));
    assert!(js.contains("a > b"));
    assert!(js.contains("?"));
    assert!(js.contains(":"));
}

#[test]
fn test_string_literal() {
    let source = r#"
        F greet() -> str {
            "Hello, World!"
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains(r#""Hello, World!""#));
}

#[test]
fn test_array_literal() {
    let source = r#"
        F get_numbers() {
            [1, 2, 3, 4, 5]
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("[1, 2, 3, 4, 5]"));
}

#[test]
fn test_pipe_operator() {
    let source = r#"
        F double(x: i64) -> i64 { x * 2 }
        F test() -> i64 {
            5 |> double
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("double(5)"));
}

#[test]
fn test_range_operator() {
    let source = r#"
        F test() {
            1..10
        }
    "#;
    let js = parse_and_generate(source);
    // Range should generate some helper or inline range creation
    assert!(js.contains("1"));
    assert!(js.contains("10"));
}

// ============================================================================
// 3. Control Flow (4 tests)
// ============================================================================

#[test]
fn test_if_else_to_js_iife() {
    let source = r#"
        F test(x: i64) -> i64 {
            I x > 0 {
                R 1
            } E {
                R -1
            }
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("if"));
    assert!(js.contains("x > 0"));
    assert!(js.contains("return 1"));
    assert!(js.contains("return -1") || js.contains("return (-1)"));
}

#[test]
fn test_match_to_if_else_chain() {
    let source = r#"
        F test(x: i64) -> str {
            M x {
                1 => "one",
                2 => "two",
                _ => "other"
            }
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("if"));
    assert!(js.contains(r#""one""#));
    assert!(js.contains(r#""two""#));
    assert!(js.contains(r#""other""#));
}

#[test]
fn test_loop_while_to_js() {
    let source = r#"
        F test() {
            x := 0
            L x < 10 {
                x = x + 1
            }
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("let x = 0") || js.contains("const x = 0"));
    assert!(js.contains("while"));
    assert!(js.contains("x < 10"));
}

#[test]
fn test_break_continue() {
    let source = r#"
        F test() {
            x := 0
            L x < 100 {
                I x == 50 {
                    B
                }
                x = x + 1
            }
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("break"));
}

// ============================================================================
// 4. Structs & Enums (4 tests)
// ============================================================================

#[test]
fn test_struct_to_js_class() {
    let source = r#"
        S Point {
            x: f64,
            y: f64
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("class Point"));
    assert!(js.contains("constructor"));
    assert!(js.contains("this.x"));
    assert!(js.contains("this.y"));
}

#[test]
fn test_enum_to_frozen_object() {
    let source = r#"
        E Color {
            Red,
            Green,
            Blue
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("Color"));
    assert!(js.contains("Red"));
    assert!(js.contains("Green"));
    assert!(js.contains("Blue"));
}

#[test]
fn test_impl_block_to_prototype_methods() {
    let source = r#"
        S Counter {
            value: i64
        }
        X Counter {
            F increment(&self) -> i64 {
                self.value + 1
            }
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("class Counter"));
    assert!(js.contains("increment"));
    assert!(js.contains("this.value"));
}

#[test]
fn test_trait_to_base_class() {
    let source = r#"
        W Drawable {
            F draw()
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("Drawable"));
    assert!(js.contains("draw"));
}

// ============================================================================
// 5. Error Handling (4 tests)
// ============================================================================

#[test]
fn test_result_enum_helpers() {
    let source = r#"
        E Result<T, E> {
            Ok(T),
            Err(E)
        }
        F test() -> Result<i64, str> {
            Result::Ok(42)
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("Result"));
    assert!(js.contains("Ok"));
    assert!(js.contains("Err"));
}

#[test]
fn test_option_enum_helpers() {
    let source = r#"
        E Option<T> {
            Some(T),
            None
        }
        F test() -> Option<i64> {
            Option::Some(42)
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("Option"));
    assert!(js.contains("Some"));
    assert!(js.contains("None"));
}

#[test]
fn test_try_operator_codegen() {
    // Test that Try expression in AST generates JavaScript
    // Note: Parser may not support `?` syntax in all contexts yet
    use vais_ast::*;

    let module = Module {
        items: vec![Spanned::new(
            Item::Function(Function {
                name: Spanned::new("get_value".to_string(), Span::new(0, 9)),
                generics: vec![],
                params: vec![],
                ret_type: Some(Spanned::new(
                    Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    Span::new(0, 3),
                )),
                body: FunctionBody::Expr(Box::new(Spanned::new(
                    Expr::Try(Box::new(Spanned::new(
                        Expr::Call {
                            func: Box::new(Spanned::new(
                                Expr::Ident("some_result".to_string()),
                                Span::new(0, 11),
                            )),
                            args: vec![],
                        },
                        Span::new(0, 11),
                    ))),
                    Span::new(0, 11),
                ))),
                is_pub: false,
                is_async: false,
                attributes: vec![],
            }),
            Span::new(0, 20),
        )],
        modules_map: None,
    };

    let mut gen = JsCodeGenerator::new();
    let js = gen.generate_module(&module).expect("Codegen failed");
    assert!(js.contains("some_result"));
    assert!(js.contains("function get_value()"));
}

#[test]
fn test_unwrap_operator() {
    let source = r#"
        F get_value() -> i64 {
            x := some_option!
            x
        }
    "#;
    let js = parse_and_generate(source);
    // Should generate unwrap helper
    assert!(js.contains("some_option"));
}

// ============================================================================
// 6. Module System (3 tests)
// ============================================================================

#[test]
fn test_use_to_esm_import() {
    let source = r#"
        U std::collections::Vec
        F test() {
            42
        }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("import"));
    assert!(js.contains("Vec"));
    assert!(js.contains(".js"));
}

#[test]
fn test_barrel_export_generation() {
    let gen = JsCodeGenerator::new();
    let modules = vec![
        "module1".to_string(),
        "module2".to_string(),
        "utils".to_string(),
    ];
    let result = gen.generate_barrel_export(&modules);
    assert!(result.contains("export * from './module1.js';"));
    assert!(result.contains("export * from './module2.js';"));
    assert!(result.contains("export * from './utils.js';"));
    assert!(result.contains("// Auto-generated barrel export"));
}

#[test]
fn test_multifile_mode() {
    let source = r#"
        P F func1() -> i64 { 1 }
        F func2() -> i64 { 2 }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut gen = JsCodeGenerator::new();
    let files = gen.generate_module_to_files(&module).expect("Codegen failed");

    // Single file mode should produce index.js
    assert!(files.contains_key("index.js"));
    let content = files.get("index.js").unwrap();
    assert!(content.contains("export function func1()"));
    assert!(content.contains("function func2()"));
}

// ============================================================================
// 7. Tree Shaking (3 tests)
// ============================================================================

#[test]
fn test_unreferenced_private_function_removed() {
    let source = r#"
        F main() -> i64 { 42 }
        F unused() -> i64 { 0 }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let shaken = vais_codegen_js::tree_shaking::TreeShaker::shake(&module);

    assert_eq!(shaken.items.len(), 1);
    match &shaken.items[0].node {
        vais_ast::Item::Function(f) => assert_eq!(f.name.node, "main"),
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_transitive_dependencies_kept() {
    let source = r#"
        F main() -> i64 { helper1() }
        F helper1() -> i64 { helper2() }
        F helper2() -> i64 { 42 }
        F unused() -> i64 { 0 }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let shaken = vais_codegen_js::tree_shaking::TreeShaker::shake(&module);

    // Should keep main, helper1, helper2 but not unused
    assert_eq!(shaken.items.len(), 3);

    let names: Vec<String> = shaken
        .items
        .iter()
        .filter_map(|item| match &item.node {
            vais_ast::Item::Function(f) => Some(f.name.node.clone()),
            _ => None,
        })
        .collect();

    assert!(names.contains(&"main".to_string()));
    assert!(names.contains(&"helper1".to_string()));
    assert!(names.contains(&"helper2".to_string()));
    assert!(!names.contains(&"unused".to_string()));
}

#[test]
fn test_public_functions_always_kept() {
    let source = r#"
        F main() -> i64 { 42 }
        P F public_api() -> i64 { 100 }
        F unused_private() -> i64 { 0 }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let shaken = vais_codegen_js::tree_shaking::TreeShaker::shake(&module);

    // Should keep main and public_api
    assert_eq!(shaken.items.len(), 2);

    let names: Vec<String> = shaken
        .items
        .iter()
        .filter_map(|item| match &item.node {
            vais_ast::Item::Function(f) => Some(f.name.node.clone()),
            _ => None,
        })
        .collect();

    assert!(names.contains(&"main".to_string()));
    assert!(names.contains(&"public_api".to_string()));
    assert!(!names.contains(&"unused_private".to_string()));
}

// ============================================================================
// 8. SourceMap (3 tests)
// ============================================================================

#[test]
fn test_sourcemap_mapping_and_json() {
    let mut map = SourceMap::new("test.vais", "test.js");
    map.add_mapping(0, 0, 0, 0);
    map.add_mapping(0, 10, 0, 5);
    map.add_mapping(1, 0, 1, 0);

    let json = map.to_json();
    assert!(json.contains(r#""version":3"#));
    assert!(json.contains(r#""file":"test.js""#));
    assert!(json.contains(r#""sources":["test.vais"]"#));
    assert!(json.contains(r#""mappings":"#));
}

#[test]
fn test_sourcemap_inline_comment() {
    let mut map = SourceMap::new("example.vais", "example.js");
    map.add_mapping(0, 0, 0, 0);

    let comment = map.to_inline_comment();
    assert!(comment.starts_with("//# sourceMappingURL=data:application/json;charset=utf-8;base64,"));

    // Verify base64 content exists
    let base64_part = comment
        .strip_prefix("//# sourceMappingURL=data:application/json;charset=utf-8;base64,")
        .unwrap();
    assert!(!base64_part.is_empty());
}

#[test]
fn test_sourcemap_file_comment() {
    let comment = SourceMap::to_file_comment("output.js.map");
    assert_eq!(comment, "//# sourceMappingURL=output.js.map");
}

// ============================================================================
// 9. Additional Edge Cases (2 tests)
// ============================================================================

#[test]
fn test_config_custom_indent() {
    let source = r#"
        S Point {
            x: i64,
            y: i64
        }
    "#;

    // Test with default 2-space indent
    let config1 = JsConfig {
        indent: "  ".to_string(),
        ..Default::default()
    };
    let js1 = parse_and_generate_with_config(source, config1);
    assert!(js1.contains("class Point"));

    // Test with 4-space indent
    let config2 = JsConfig {
        indent: "    ".to_string(),
        ..Default::default()
    };
    let js2 = parse_and_generate_with_config(source, config2);
    assert!(js2.contains("class Point"));
    // Both should generate valid JavaScript
    assert!(js2.contains("constructor"));
}

#[test]
fn test_public_export_functions() {
    let source = r#"
        P F public_func() -> i64 { 42 }
        F private_func() -> i64 { 0 }
    "#;
    let js = parse_and_generate(source);
    assert!(js.contains("export function public_func()"));
    assert!(js.contains("function private_func()"));
    assert!(!js.contains("export function private_func()"));
}
