//! VaisX Contract Tests — JS Code Generation
//!
//! Verifies that `JsCodeGenerator::generate_module()` produces valid ESM JavaScript
//! for basic Vais constructs that will appear in VaisX `<script>` blocks.
//!
//! These tests ensure the interface contract between vaisx-compiler's
//! codegen_js.rs and the core vais-codegen-js crate remains stable.

use vais_codegen_js::{JsCodeGenerator, JsConfig};

// ============================================================================
// Helper
// ============================================================================

fn parse_and_generate(source: &str) -> String {
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut gen = JsCodeGenerator::new();
    gen.generate_module(&module).expect("JS codegen failed")
}

fn parse_and_generate_with_config(source: &str, config: JsConfig) -> String {
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut gen = JsCodeGenerator::with_config(config);
    gen.generate_module(&module).expect("JS codegen failed")
}

// ============================================================================
// 1. Function → ESM export
// ============================================================================

#[test]
fn test_public_function_generates_export() {
    let js = parse_and_generate("pub fn greet() { 42 }");
    assert!(
        js.contains("export function greet()"),
        "Public function should have 'export'"
    );
}

#[test]
fn test_private_function_no_export() {
    let js = parse_and_generate("fn helper() { 42 }");
    assert!(js.contains("function helper()"), "Should have function");
    assert!(
        !js.contains("export function helper"),
        "Private function should not export"
    );
}

#[test]
fn test_async_function() {
    let js = parse_and_generate("A fn fetchData() { 42 }");
    assert!(js.contains("async function fetchData()"), "Should be async");
}

#[test]
fn test_function_with_params() {
    let js = parse_and_generate("fn add(a: i64, b: i64) -> i64 { a + b }");
    assert!(js.contains("function add(a, b)"), "Should have params");
    assert!(js.contains("return"), "Should have return");
    assert!(js.contains("a + b"), "Should have body expression");
}

#[test]
fn test_function_with_default_params() {
    let js = parse_and_generate("fn greet(name: str = \"world\") { name }");
    assert!(
        js.contains("name = \"world\""),
        "Should have default parameter value"
    );
}

// ============================================================================
// 2. Struct → ES6 Class
// ============================================================================

#[test]
fn test_struct_generates_class() {
    let js = parse_and_generate("struct Point { x: f64, y: f64 }");
    assert!(js.contains("class Point"), "Struct should become class");
    assert!(
        js.contains("constructor(x, y)"),
        "Should have constructor with fields"
    );
    assert!(js.contains("this.x = x"), "Should assign fields");
    assert!(js.contains("this.y = y"), "Should assign fields");
}

#[test]
fn test_public_struct_exports() {
    let js = parse_and_generate("pub struct Point { x: f64, y: f64 }");
    assert!(
        js.contains("export class Point"),
        "Public struct should export"
    );
}

#[test]
fn test_struct_object_arg_support() {
    let js = parse_and_generate("struct Config { width: i64, height: i64 }");
    // Multi-field structs support object-style construction
    assert!(js.contains("typeof"), "Should support object arg pattern");
    assert!(js.contains("__obj"), "Should destructure object arg");
}

#[test]
fn test_struct_single_field() {
    let js = parse_and_generate("struct Wrapper { value: i64 }");
    assert!(
        js.contains("class Wrapper"),
        "Single field struct should work"
    );
    assert!(js.contains("constructor(value)"), "Should have constructor");
}

// ============================================================================
// 3. Enum → Tagged Union (Object.freeze)
// ============================================================================

#[test]
fn test_enum_generates_frozen_object() {
    let js = parse_and_generate("enum Color { Red, Green, Blue }");
    assert!(
        js.contains("const Color = Object.freeze"),
        "Enum should be frozen object"
    );
    assert!(js.contains("__tag: \"Red\""), "Should have Red tag");
    assert!(js.contains("__tag: \"Green\""), "Should have Green tag");
    assert!(js.contains("__tag: \"Blue\""), "Should have Blue tag");
}

#[test]
fn test_enum_tuple_variant() {
    let js = parse_and_generate("enum Shape { Circle(f64), Rect(f64, f64) }");
    assert!(
        js.contains("Circle(__0)"),
        "Tuple variant should be factory function"
    );
    assert!(
        js.contains("Rect(__0, __1)"),
        "Multi-arg tuple variant should work"
    );
    assert!(js.contains("__data: [__0]"), "Should wrap data in array");
}

#[test]
fn test_enum_public_export() {
    let js = parse_and_generate("pub enum Direction { Up, Down }");
    assert!(
        js.contains("export const Direction"),
        "Public enum should export"
    );
}

// ============================================================================
// 4. Impl → Prototype Methods
// ============================================================================

#[test]
fn test_impl_instance_method() {
    let js = parse_and_generate(
        r#"
        struct Counter { value: i64 }
        impl Counter {
            fn increment(self) { self.value + 1 }
        }
    "#,
    );
    assert!(
        js.contains("Counter.prototype.increment"),
        "Instance method on prototype"
    );
}

#[test]
fn test_impl_static_method() {
    let js = parse_and_generate(
        r#"
        struct Counter { value: i64 }
        impl Counter {
            fn new_counter() -> i64 { 0 }
        }
    "#,
    );
    assert!(js.contains("Counter.new_counter"), "Static method on class");
    assert!(
        !js.contains("Counter.prototype.new_counter"),
        "Should NOT be on prototype"
    );
}

#[test]
fn test_impl_trait() {
    let js = parse_and_generate(
        r#"
        trait Display {
            fn display(self) -> str
        }
        struct Point { x: f64, y: f64 }
        impl Point: Display {
            fn display(self) -> str { "point" }
        }
    "#,
    );
    assert!(
        js.contains("__implements"),
        "Trait impl should track __implements"
    );
    assert!(js.contains("\"Display\""), "Should register trait name");
}

// ============================================================================
// 5. Trait → Base Class
// ============================================================================

#[test]
fn test_trait_generates_class() {
    let js = parse_and_generate(
        r#"
        trait Drawable {
            fn draw(self) -> str
        }
    "#,
    );
    assert!(js.contains("class Drawable"), "Trait should become class");
    assert!(
        js.contains("throw new Error"),
        "Abstract method should throw"
    );
}

// ============================================================================
// 6. Const / Global
// ============================================================================

#[test]
fn test_const_declaration() {
    let js = parse_and_generate("C MAX_SIZE: i64 = 100");
    assert!(
        js.contains("const MAX_SIZE = 100"),
        "Const should generate const"
    );
}

#[test]
fn test_public_const_exports() {
    let js = parse_and_generate("pub const pi: f64 = 3.14");
    assert!(js.contains("export const pi"), "Public const should export");
}

// ============================================================================
// 7. Use → ESM Import
// ============================================================================

#[test]
fn test_use_generates_import() {
    let js = parse_and_generate("use math");
    assert!(
        js.contains("import * as math from './math.js'"),
        "Use should become import"
    );
}

// ============================================================================
// 8. Attributes on Functions (#[server], #[wasm])
// ============================================================================

#[test]
fn test_function_with_server_attribute() {
    let js = parse_and_generate(
        r#"
        #[server]
        A fn loadItems() -> i64 {
            42
        }
    "#,
    );
    assert!(
        js.contains("async function loadItems()"),
        "Server function should generate"
    );
}

#[test]
fn test_function_with_wasm_attribute() {
    let js = parse_and_generate(
        r#"
        #[wasm]
        fn processData(raw: i64) -> i64 {
            raw
        }
    "#,
    );
    assert!(
        js.contains("function processData(raw)"),
        "Wasm function should generate"
    );
}

// ============================================================================
// 9. JsConfig options
// ============================================================================

#[test]
fn test_custom_indent() {
    let config = JsConfig {
        indent: "    ".to_string(),
        ..JsConfig::default()
    };
    let js = parse_and_generate_with_config("fn test() { 1 }", config);
    assert!(js.contains("    "), "Should use 4-space indent");
}

// ============================================================================
// 10. Combined patterns — simulating VaisX <script> block output
// ============================================================================

#[test]
fn test_combined_vaisx_script_output() {
    // Simulates what vais-codegen-js would produce for a VaisX component's
    // regular Vais code (after desugar, the non-reactive parts)
    let js = parse_and_generate(
        r#"
        struct __VxProps__ {
            initial: i64
        }

        fn increment(count: i64) -> i64 {
            count + 1
        }

        fn reset() -> i64 {
            0
        }

        pub fn formatCount(n: i64) -> str {
            "count"
        }
    "#,
    );

    assert!(
        js.contains("class __VxProps__"),
        "Props struct should become class"
    );
    assert!(
        js.contains("function increment(count)"),
        "increment should generate"
    );
    assert!(js.contains("function reset()"), "reset should generate");
    assert!(
        js.contains("export function formatCount(n)"),
        "Public fn should export"
    );
}

#[test]
fn test_async_server_function_output() {
    let js = parse_and_generate(
        r#"
        #[server]
        A fn loadItems() -> i64 { 42 }

        #[server]
        A fn saveItem(item: i64) -> i64 { item }
    "#,
    );

    assert!(
        js.contains("async function loadItems()"),
        "Server load should be async"
    );
    assert!(
        js.contains("async function saveItem(item)"),
        "Server save should be async"
    );
}

// ============================================================================
// 11. Source Map availability
// ============================================================================

#[test]
fn test_source_map_creation() {
    use vais_codegen_js::SourceMap;

    let mut map = SourceMap::new("component.vaisx", "component.js");
    map.add_mapping(0, 0, 0, 0);
    map.add_mapping(1, 2, 3, 4);

    let json = map.to_json();
    assert!(json.contains("\"version\":3"), "Should be v3 source map");
    assert!(
        json.contains("component.vaisx"),
        "Should reference source file"
    );
    assert!(
        json.contains("component.js"),
        "Should reference generated file"
    );

    let inline = map.to_inline_comment();
    assert!(
        inline.starts_with("//# sourceMappingURL=data:application/json;charset=utf-8;base64,"),
        "Inline comment should have data URI"
    );
}

// ============================================================================
// 12. Tree shaking availability
// ============================================================================

#[test]
fn test_tree_shaking_removes_unused() {
    use vais_codegen_js::tree_shaking::TreeShaker;

    let module = vais_parser::parse(
        r#"
        pub fn used() -> i64 { 1 }
        fn unused() -> i64 { 2 }
    "#,
    )
    .expect("Parse failed");

    let shaken = TreeShaker::shake(&module);

    // Public function should be kept
    let has_used = shaken
        .items
        .iter()
        .any(|item| matches!(&item.node, vais_ast::Item::Function(f) if f.name.node == "used"));
    assert!(has_used, "Public function should be kept");

    // Private unreferenced function should be removed
    let has_unused = shaken
        .items
        .iter()
        .any(|item| matches!(&item.node, vais_ast::Item::Function(f) if f.name.node == "unused"));
    assert!(!has_unused, "Unused private function should be removed");
}
