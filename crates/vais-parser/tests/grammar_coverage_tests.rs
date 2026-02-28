//! Grammar coverage tests for the Vais parser.
//!
//! Systematically tests every production rule in the grammar:
//! - Items (14 variants)
//! - Types (25 variants)
//! - Expressions (46 variants across 16 precedence levels + primary)
//! - Statements (8 variants)
//! - Patterns (9 variants)
//! - Generics & Where clauses
//! - Negative cases (invalid syntax must be rejected)
//! - AST variant count guards (compile-time sync checks)

use vais_ast::*;
use vais_lexer::tokenize;
use vais_parser::Parser;

// =============================================================================
// Helper functions
// =============================================================================

fn assert_parses(source: &str) {
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse_module();
    assert!(
        result.is_ok(),
        "Failed to parse: {}\nError: {:?}",
        source,
        result.err()
    );
}

fn assert_parse_fails(source: &str) {
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse_module();
    assert!(result.is_err(), "Expected parse failure for: {}", source);
}

fn assert_parses_expr(source: &str) {
    let wrapped = format!("F __test__() -> i64 {{ {} }}", source);
    assert_parses(&wrapped);
}

fn assert_expr_fails(source: &str) {
    let wrapped = format!("F __test__() -> i64 {{ {} }}", source);
    assert_parse_fails(&wrapped);
}

fn parse_ok(source: &str) -> Module {
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    parser
        .parse_module()
        .unwrap_or_else(|e| panic!("Parse failed: {}\nErr: {:?}", source, e))
}

// =============================================================================
// Section 1: Items (14 variants)
// =============================================================================
mod items {
    use super::*;

    // --- Item::Function ---

    #[test]
    fn grammar_item_function_expr_body() {
        let m = parse_ok("F add(a: i64, b: i64) -> i64 = a + b");
        assert_eq!(m.items.len(), 1);
        match &m.items[0].node {
            Item::Function(f) => {
                assert_eq!(f.name.node, "add");
                assert_eq!(f.params.len(), 2);
                assert!(matches!(f.body, FunctionBody::Expr(_)));
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_function_block_body() {
        let m = parse_ok("F foo(x: i64) -> i64 { R x + 1 }");
        match &m.items[0].node {
            Item::Function(f) => {
                assert!(matches!(f.body, FunctionBody::Block(_)));
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_function_no_return_type() {
        assert_parses("F greet() { R 0 }");
    }

    #[test]
    fn grammar_item_function_no_params() {
        assert_parses("F zero() -> i64 = 0");
    }

    #[test]
    fn grammar_item_async_function() {
        let m = parse_ok("A F fetch() -> i64 = 0");
        match &m.items[0].node {
            Item::Function(f) => assert!(f.is_async),
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_public_function() {
        let m = parse_ok("P F hello() -> i64 = 42");
        match &m.items[0].node {
            Item::Function(f) => assert!(f.is_pub),
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_generic_function() {
        let m = parse_ok("F id<T>(x: T) -> T = x");
        match &m.items[0].node {
            Item::Function(f) => {
                assert_eq!(f.generics.len(), 1);
                assert_eq!(f.generics[0].name.node, "T");
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_function_with_where() {
        assert_parses("F print_val<T>(x: T) -> i64 where T: Display = 0");
    }

    #[test]
    fn grammar_item_function_with_attributes() {
        assert_parses("#[inline]\nF fast() -> i64 = 0");
    }

    // --- Item::Struct ---

    #[test]
    fn grammar_item_struct_simple() {
        let m = parse_ok("S Point { x: f64, y: f64 }");
        match &m.items[0].node {
            Item::Struct(s) => {
                assert_eq!(s.name.node, "Point");
                assert_eq!(s.fields.len(), 2);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_struct_generic() {
        let m = parse_ok("S Pair<A, B> { first: A, second: B }");
        match &m.items[0].node {
            Item::Struct(s) => assert_eq!(s.generics.len(), 2),
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_struct_public() {
        let m = parse_ok("P S Color { r: i64, g: i64, b: i64 }");
        match &m.items[0].node {
            Item::Struct(s) => assert!(s.is_pub),
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_struct_with_where() {
        assert_parses("S Container<T> where T: Clone { value: T }");
    }

    // --- Item::Enum ---

    #[test]
    fn grammar_item_enum_simple() {
        let m = parse_ok("E Color { Red, Green, Blue }");
        match &m.items[0].node {
            Item::Enum(e) => {
                assert_eq!(e.name.node, "Color");
                assert_eq!(e.variants.len(), 3);
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_enum_generic() {
        let m = parse_ok("E Option<T> { Some(T), None }");
        match &m.items[0].node {
            Item::Enum(e) => {
                assert_eq!(e.generics.len(), 1);
                assert!(matches!(
                    e.variants[0].fields,
                    VariantFields::Tuple(_)
                ));
                assert!(matches!(e.variants[1].fields, VariantFields::Unit));
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_enum_struct_variant() {
        assert_parses("E Shape { Circle { radius: f64 }, Rect { w: f64, h: f64 } }");
    }

    #[test]
    fn grammar_item_enum_public() {
        let m = parse_ok("P E Dir { Up, Down }");
        match &m.items[0].node {
            Item::Enum(e) => assert!(e.is_pub),
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    // --- Item::Union ---

    #[test]
    fn grammar_item_union() {
        let m = parse_ok("O Value { i: i64, f: f64 }");
        match &m.items[0].node {
            Item::Union(u) => {
                assert_eq!(u.name.node, "Value");
                assert_eq!(u.fields.len(), 2);
            }
            other => panic!("Expected Union, got {:?}", other),
        }
    }

    // --- Item::TypeAlias ---

    #[test]
    fn grammar_item_type_alias() {
        let m = parse_ok("T Num = i64");
        match &m.items[0].node {
            Item::TypeAlias(ta) => {
                assert_eq!(ta.name.node, "Num");
            }
            other => panic!("Expected TypeAlias, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_type_alias_generic() {
        assert_parses("T Pair<A, B> = (A, B)");
    }

    // --- Item::TraitAlias ---

    #[test]
    fn grammar_item_trait_alias() {
        let m = parse_ok("T Printable = Display + Debug");
        match &m.items[0].node {
            Item::TraitAlias(ta) => {
                assert_eq!(ta.name.node, "Printable");
                assert!(ta.bounds.len() >= 2);
            }
            other => panic!("Expected TraitAlias, got {:?}", other),
        }
    }

    // --- Item::Use ---

    #[test]
    fn grammar_item_use_simple() {
        let m = parse_ok("U std");
        match &m.items[0].node {
            Item::Use(u) => {
                assert!(!u.path.is_empty());
            }
            other => panic!("Expected Use, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_use_dotted_path() {
        assert_parses("U std.io");
    }

    #[test]
    fn grammar_item_use_selective() {
        assert_parses("U std/collections.{HashMap, HashSet}");
    }

    // --- Item::Trait ---

    #[test]
    fn grammar_item_trait_empty() {
        let m = parse_ok("W Marker { }");
        match &m.items[0].node {
            Item::Trait(t) => assert_eq!(t.name.node, "Marker"),
            other => panic!("Expected Trait, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_trait_with_methods() {
        assert_parses(
            "W Greet {
                F greet(self) -> i64
            }",
        );
    }

    #[test]
    fn grammar_item_trait_generic() {
        assert_parses("W Container<T> { F get(self) -> T }");
    }

    #[test]
    fn grammar_item_trait_with_super() {
        assert_parses("W Iterator: Iterable { F next(self) -> i64 }");
    }

    #[test]
    fn grammar_item_trait_with_default_impl() {
        assert_parses(
            "W Greeter {
                F greet(self) -> i64 = 42
            }",
        );
    }

    // --- Item::Impl ---

    #[test]
    fn grammar_item_impl_inherent() {
        assert_parses(
            "S Foo { x: i64 }
             X Foo {
                 F get_x(self) -> i64 = self.x
             }",
        );
    }

    #[test]
    fn grammar_item_impl_for_trait() {
        assert_parses(
            "W Greet { F greet(self) -> i64 }
             S Bar { v: i64 }
             X Bar: Greet {
                 F greet(self) -> i64 = self.v
             }",
        );
    }

    #[test]
    fn grammar_item_impl_generic() {
        assert_parses(
            "S Wrapper<T> { val: T }
             X Wrapper<T> {
                 F unwrap(self) -> T = self.val
             }",
        );
    }

    // --- Item::Macro ---

    #[test]
    fn grammar_item_macro_definition() {
        let m = parse_ok(
            "macro my_add! {
                ($a:expr, $b:expr) => { $a + $b }
            }",
        );
        match &m.items[0].node {
            Item::Macro(md) => assert_eq!(md.name.node, "my_add"),
            other => panic!("Expected Macro, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_macro_empty_pattern() {
        assert_parses(
            "macro unit! {
                () => { 0 }
            }",
        );
    }

    // --- Item::ExternBlock ---

    #[test]
    fn grammar_item_extern_block() {
        let m = parse_ok(
            r#"N "C" {
                F puts(s: *i8) -> i32
            }"#,
        );
        match &m.items[0].node {
            Item::ExternBlock(eb) => {
                assert_eq!(eb.abi, "C");
                assert_eq!(eb.functions.len(), 1);
            }
            other => panic!("Expected ExternBlock, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_extern_varargs() {
        assert_parses(
            r#"N "C" {
                F printf(fmt: *i8, ...) -> i32
            }"#,
        );
    }

    // --- Item::Const ---

    #[test]
    fn grammar_item_const() {
        let m = parse_ok("C MAX: i64 = 100");
        match &m.items[0].node {
            Item::Const(c) => assert_eq!(c.name.node, "MAX"),
            other => panic!("Expected Const, got {:?}", other),
        }
    }

    #[test]
    fn grammar_item_const_public() {
        let m = parse_ok("P C PI: f64 = 3.14");
        match &m.items[0].node {
            Item::Const(c) => assert!(c.is_pub),
            other => panic!("Expected Const, got {:?}", other),
        }
    }

    // --- Item::Global ---

    #[test]
    fn grammar_item_global() {
        let m = parse_ok("G counter: i64 = 0");
        match &m.items[0].node {
            Item::Global(g) => assert_eq!(g.name.node, "counter"),
            other => panic!("Expected Global, got {:?}", other),
        }
    }

    // --- Multiple items ---

    #[test]
    fn grammar_item_multiple_items() {
        let m = parse_ok(
            "S Point { x: i64, y: i64 }
             F origin() -> Point = Point { x: 0, y: 0 }
             C ZERO: i64 = 0",
        );
        assert_eq!(m.items.len(), 3);
    }

    // --- Comments ---

    #[test]
    fn grammar_comment_line() {
        assert_parses(
            "# This is a comment
             F foo() -> i64 = 42",
        );
    }
}

// =============================================================================
// Section 2: Types (25 variants)
// =============================================================================
mod types {
    use super::*;

    #[test]
    fn grammar_type_named_simple() {
        assert_parses("F f(x: i64) -> i64 = x");
        assert_parses("F f(x: f64) -> f64 = x");
        assert_parses("F f(x: bool) -> bool = x");
        assert_parses("F f(x: str) -> str = x");
        assert_parses("F f(x: i8) -> i8 = x");
        assert_parses("F f(x: u32) -> u32 = x");
    }

    #[test]
    fn grammar_type_named_generic() {
        assert_parses("F f(x: Vec<i64>) -> Vec<i64> = x");
        assert_parses("F f(x: HashMap<str, i64>) -> HashMap<str, i64> = x");
    }

    #[test]
    fn grammar_type_named_nested_generic() {
        assert_parses("F f(x: Vec<Vec<i64>>) -> Vec<Vec<i64>> = x");
    }

    #[test]
    fn grammar_type_fn_ptr() {
        assert_parses("F f(cb: fn(i64) -> i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_fn_ptr_vararg() {
        assert_parses("F f(cb: fn(i64, ...) -> i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_array() {
        assert_parses("F f(arr: [i64]) -> i64 = 0");
    }

    #[test]
    fn grammar_type_const_array() {
        assert_parses("F f(arr: [i64; 10]) -> i64 = 0");
    }

    #[test]
    fn grammar_type_map() {
        assert_parses("F f(m: [str:i64]) -> i64 = 0");
    }

    #[test]
    fn grammar_type_tuple() {
        assert_parses("F f(t: (i64, f64)) -> i64 = 0");
        assert_parses("F f(t: (i64, f64, bool)) -> i64 = 0");
    }

    #[test]
    fn grammar_type_optional() {
        assert_parses("F f(x: i64?) -> i64 = 0");
    }

    #[test]
    fn grammar_type_result() {
        assert_parses("F f(x: i64!) -> i64 = 0");
    }

    #[test]
    fn grammar_type_pointer() {
        assert_parses("F f(p: *i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_ref() {
        assert_parses("F f(r: &i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_ref_mut() {
        assert_parses("F f(r: &mut i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_slice() {
        assert_parses("F f(s: &[i64]) -> i64 = 0");
    }

    #[test]
    fn grammar_type_slice_mut() {
        assert_parses("F f(s: &mut [i64]) -> i64 = 0");
    }

    #[test]
    fn grammar_type_ref_lifetime() {
        assert_parses("F f<'a>(r: &'a i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_ref_mut_lifetime() {
        assert_parses("F f<'a>(r: &'a mut i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_lazy() {
        assert_parses("F f(x: Lazy<i64>) -> i64 = 0");
    }

    #[test]
    fn grammar_type_fn_type() {
        assert_parses("F f(cb: (i64, i64) -> i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_unit() {
        assert_parses("F f() -> () = ()");
    }

    #[test]
    fn grammar_type_dyn_trait() {
        assert_parses("F f(x: dyn Display) -> i64 = 0");
    }

    #[test]
    fn grammar_type_dyn_trait_generic() {
        assert_parses("F f(x: dyn Iterator<i64>) -> i64 = 0");
    }

    #[test]
    fn grammar_type_linear() {
        assert_parses("F f(x: linear i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_affine() {
        assert_parses("F f(x: affine i64) -> i64 = 0");
    }

    #[test]
    fn grammar_type_impl_trait() {
        // Vais uses `X Trait` for existential/impl trait (not `impl`)
        assert_parses("F f(x: i64) -> X Display = 0");
    }

    #[test]
    fn grammar_type_impl_trait_multi_bound() {
        assert_parses("F f(x: i64) -> X Display + Debug = 0");
    }
}

// =============================================================================
// Section 3: Expressions (46 variants, by precedence + primary)
// =============================================================================
mod expressions {
    use super::*;

    // --- Literals & Primary ---

    #[test]
    fn grammar_expr_int() {
        assert_parses_expr("42");
        assert_parses_expr("0");
        assert_parses_expr("1000000");
    }

    #[test]
    fn grammar_expr_float() {
        assert_parses_expr("3.14");
        assert_parses_expr("0.0");
    }

    #[test]
    fn grammar_expr_bool() {
        assert_parses_expr("true");
        assert_parses_expr("false");
    }

    #[test]
    fn grammar_expr_string() {
        assert_parses_expr(r#""hello world""#);
        assert_parses_expr(r#""""#);
    }

    #[test]
    fn grammar_expr_string_interp() {
        assert_parses("F f(name: str) -> str = ~\"hello {name}\"");
    }

    #[test]
    fn grammar_expr_unit() {
        assert_parses_expr("()");
    }

    #[test]
    fn grammar_expr_ident() {
        assert_parses_expr("x");
    }

    #[test]
    fn grammar_expr_self_call() {
        assert_parses("F fact(n: i64) -> i64 = I n <= 1 { R 1 } E { R n * @(n - 1) }");
    }

    // --- Binary operators by precedence ---

    #[test]
    fn grammar_expr_binary_arithmetic() {
        assert_parses_expr("1 + 2");
        assert_parses_expr("3 - 1");
        assert_parses_expr("2 * 3");
        assert_parses_expr("10 / 2");
        assert_parses_expr("7 % 3");
    }

    #[test]
    fn grammar_expr_binary_comparison() {
        assert_parses_expr("1 < 2");
        assert_parses_expr("1 <= 2");
        assert_parses_expr("2 > 1");
        assert_parses_expr("2 >= 1");
        assert_parses_expr("1 == 1");
        assert_parses_expr("1 != 2");
    }

    #[test]
    fn grammar_expr_binary_logical() {
        assert_parses_expr("true && false");
        assert_parses_expr("true || false");
    }

    #[test]
    fn grammar_expr_binary_bitwise() {
        assert_parses_expr("1 & 2");
        assert_parses_expr("1 | 2");
        assert_parses_expr("1 ^ 2");
        assert_parses_expr("1 << 2");
        assert_parses_expr("1 >> 2");
    }

    #[test]
    fn grammar_expr_binary_precedence() {
        // Multiplication binds tighter than addition
        assert_parses_expr("1 + 2 * 3");
        // Comparison vs arithmetic
        assert_parses_expr("1 + 2 < 3 + 4");
        // Logical vs comparison
        assert_parses_expr("1 < 2 && 3 > 4");
    }

    // --- Unary operators ---

    #[test]
    fn grammar_expr_unary_neg() {
        assert_parses_expr("-1");
        assert_parses_expr("-x");
    }

    #[test]
    fn grammar_expr_unary_not() {
        assert_parses_expr("!true");
        assert_parses_expr("!x");
    }

    #[test]
    fn grammar_expr_unary_bitnot() {
        assert_parses_expr("~x");
    }

    // --- Ternary ---

    #[test]
    fn grammar_expr_ternary() {
        assert_parses_expr("x > 0 ? 1 : 0");
    }

    // --- If expression ---

    #[test]
    fn grammar_expr_if_simple() {
        assert_parses_expr("I x > 0 { 1 } E { 0 }");
    }

    #[test]
    fn grammar_expr_if_no_else() {
        assert_parses_expr("I x > 0 { 1 }");
    }

    #[test]
    fn grammar_expr_if_else_if() {
        assert_parses_expr("I x > 0 { 1 } E I x == 0 { 0 } E { -1 }");
    }

    // --- Loop ---

    #[test]
    fn grammar_expr_loop_for_range() {
        assert_parses_expr("L i:0..10 { i }");
    }

    #[test]
    fn grammar_expr_loop_infinite() {
        assert_parses_expr("L { B }");
    }

    // --- While ---

    #[test]
    fn grammar_expr_while() {
        assert_parses_expr("L x > 0 { x = x - 1 }");
    }

    // --- Match ---

    #[test]
    fn grammar_expr_match_simple() {
        assert_parses_expr("M x { 0 => 1, 1 => 2, _ => 0 }");
    }

    #[test]
    fn grammar_expr_match_with_guard() {
        // Guard uses `I` (If keyword), not `if`
        assert_parses_expr("M x { n I n > 0 => 1, _ => 0 }");
    }

    // --- Call ---

    #[test]
    fn grammar_expr_call() {
        assert_parses_expr("foo(1, 2)");
        assert_parses_expr("bar()");
    }

    // --- Method call ---

    #[test]
    fn grammar_expr_method_call() {
        assert_parses_expr("obj.method(1)");
        assert_parses_expr("x.to_string()");
    }

    // --- Static method call ---

    #[test]
    fn grammar_expr_static_method_call() {
        assert_parses("F f() -> i64 = Vec.new()");
    }

    // --- Field access ---

    #[test]
    fn grammar_expr_field() {
        assert_parses_expr("p.x");
        assert_parses_expr("p.x.y");
    }

    // --- Index ---

    #[test]
    fn grammar_expr_index() {
        assert_parses_expr("arr[0]");
        assert_parses_expr("arr[i + 1]");
    }

    // --- Array literal ---

    #[test]
    fn grammar_expr_array() {
        assert_parses_expr("[1, 2, 3]");
        assert_parses_expr("[]");
    }

    // --- Tuple literal ---

    #[test]
    fn grammar_expr_tuple() {
        assert_parses_expr("(1, 2)");
        assert_parses_expr("(1, 2, 3)");
    }

    // --- Struct literal ---

    #[test]
    fn grammar_expr_struct_lit() {
        assert_parses(
            "S Point { x: i64, y: i64 }
             F f() -> Point = Point { x: 1, y: 2 }",
        );
    }

    // --- Range ---

    #[test]
    fn grammar_expr_range() {
        assert_parses_expr("0..10");
    }

    #[test]
    fn grammar_expr_range_inclusive() {
        assert_parses_expr("0..=10");
    }

    // --- Block ---

    #[test]
    fn grammar_expr_block() {
        assert_parses_expr("{ x := 1\n x + 1 }");
    }

    // --- Await (postfix: expr.Y) ---

    #[test]
    fn grammar_expr_await() {
        // Await is postfix: `expr.Y` (not prefix)
        assert_parses("A F f() -> i64 { x := fetch().Y\n R x }");
    }

    // --- Try ---

    #[test]
    fn grammar_expr_try() {
        assert_parses_expr("foo()?");
    }

    // --- Unwrap ---

    #[test]
    fn grammar_expr_unwrap() {
        assert_parses_expr("foo()!");
    }

    // --- MapLit ---

    #[test]
    fn grammar_expr_map_lit() {
        assert_parses("F f() -> i64 { m := {\"a\": 1, \"b\": 2}\n R 0 }");
    }

    // --- Spread ---

    #[test]
    fn grammar_expr_spread() {
        assert_parses_expr("[1, ..rest]");
    }

    // --- Ref ---

    #[test]
    fn grammar_expr_ref() {
        assert_parses_expr("&x");
    }

    // --- Deref ---

    #[test]
    fn grammar_expr_deref() {
        assert_parses_expr("*p");
    }

    // --- Cast ---

    #[test]
    fn grammar_expr_cast() {
        assert_parses_expr("x as f64");
        assert_parses_expr("42 as i8");
    }

    // --- Assign ---

    #[test]
    fn grammar_expr_assign() {
        assert_parses_expr("x = 5");
    }

    // --- AssignOp (only +=, -=, *=, /= are lexer tokens) ---

    #[test]
    fn grammar_expr_assign_op() {
        assert_parses_expr("x += 1");
        assert_parses_expr("x -= 1");
        assert_parses_expr("x *= 2");
        assert_parses_expr("x /= 2");
    }

    // --- Lambda ---

    #[test]
    fn grammar_expr_lambda_expr() {
        assert_parses_expr("|x| x + 1");
    }

    #[test]
    fn grammar_expr_lambda_block() {
        assert_parses_expr("|x, y| { x + y }");
    }

    #[test]
    fn grammar_expr_lambda_typed() {
        assert_parses_expr("|x: i64| x * 2");
    }

    #[test]
    fn grammar_expr_lambda_move() {
        assert_parses_expr("move |x| x + 1");
    }

    // --- Spawn ---

    #[test]
    fn grammar_expr_spawn() {
        assert_parses_expr("spawn foo()");
    }

    // --- Yield ---

    #[test]
    fn grammar_expr_yield() {
        // `yield` is the keyword (not `Y` which is await)
        assert_parses("F gen() -> i64 { yield 42 }");
    }

    // --- Comptime ---

    #[test]
    fn grammar_expr_comptime() {
        assert_parses_expr("comptime { 2 + 2 }");
    }

    // --- MacroInvoke ---

    #[test]
    fn grammar_expr_macro_invoke() {
        assert_parses_expr("vec!(1, 2, 3)");
    }

    // --- Old ---

    #[test]
    fn grammar_expr_old() {
        assert_parses_expr("old(x)");
    }

    // --- Assert ---

    #[test]
    fn grammar_expr_assert() {
        assert_parses_expr("assert(x > 0)");
    }

    #[test]
    fn grammar_expr_assert_with_message() {
        assert_parses_expr("assert(x > 0, \"x must be positive\")");
    }

    // --- Assume ---

    #[test]
    fn grammar_expr_assume() {
        assert_parses_expr("assume(x > 0)");
    }

    // --- Lazy ---

    #[test]
    fn grammar_expr_lazy() {
        assert_parses_expr("lazy 42");
    }

    // --- Force ---

    #[test]
    fn grammar_expr_force() {
        assert_parses_expr("force x");
    }

    // --- Pipe operator ---

    #[test]
    fn grammar_expr_pipe() {
        assert_parses_expr("x |> foo");
    }

    #[test]
    fn grammar_expr_pipe_chain() {
        assert_parses_expr("x |> foo |> bar");
    }
}

// =============================================================================
// Section 4: Statements (8 variants)
// =============================================================================
mod statements {
    use super::*;

    #[test]
    fn grammar_stmt_let_simple() {
        assert_parses("F f() -> i64 { x := 5\n R x }");
    }

    #[test]
    fn grammar_stmt_let_typed() {
        assert_parses("F f() -> i64 { x: i64 = 5\n R x }");
    }

    #[test]
    fn grammar_stmt_let_mutable() {
        assert_parses("F f() -> i64 { x := mut 0\n x = 1\n R x }");
    }

    #[test]
    fn grammar_stmt_let_destructure() {
        assert_parses("F f() -> i64 { (a, b) := (1, 2)\n R a + b }");
    }

    #[test]
    fn grammar_stmt_expr() {
        assert_parses("F f() -> i64 { foo()\n R 0 }");
    }

    #[test]
    fn grammar_stmt_return_explicit() {
        assert_parses("F f() -> i64 { R 42 }");
    }

    #[test]
    fn grammar_stmt_return_void() {
        assert_parses("F f() { R }");
    }

    #[test]
    fn grammar_stmt_break() {
        assert_parses("F f() -> i64 { L { B } R 0 }");
    }

    #[test]
    fn grammar_stmt_break_with_value() {
        assert_parses("F f() -> i64 { L { B 42 } }");
    }

    #[test]
    fn grammar_stmt_continue() {
        assert_parses("F f() -> i64 { L i:0..10 { C } R 0 }");
    }

    #[test]
    fn grammar_stmt_defer() {
        assert_parses("F f() -> i64 { D cleanup()\n R 0 }");
    }
}

// =============================================================================
// Section 5: Patterns (9 variants)
// =============================================================================
mod patterns {
    use super::*;

    #[test]
    fn grammar_pattern_wildcard() {
        assert_parses("F f(x: i64) -> i64 = M x { _ => 0 }");
    }

    #[test]
    fn grammar_pattern_ident() {
        assert_parses("F f(x: i64) -> i64 = M x { n => n }");
    }

    #[test]
    fn grammar_pattern_literal_int() {
        assert_parses("F f(x: i64) -> i64 = M x { 0 => 1, _ => 0 }");
    }

    #[test]
    fn grammar_pattern_literal_bool() {
        assert_parses("F f(x: bool) -> i64 = M x { true => 1, false => 0 }");
    }

    #[test]
    fn grammar_pattern_literal_string() {
        assert_parses("F f(x: str) -> i64 = M x { \"hi\" => 1, _ => 0 }");
    }

    #[test]
    fn grammar_pattern_tuple() {
        assert_parses("F f(x: (i64, i64)) -> i64 = M x { (a, b) => a + b }");
    }

    #[test]
    fn grammar_pattern_struct() {
        // Struct patterns are not currently supported in the parser.
        // Variant patterns with parens are used instead: `Point(x, y)`
        assert_parses(
            "E Shape { Circle(f64), Rect(f64, f64) }
             F f(s: Shape) -> f64 = M s { Circle(r) => r, Rect(w, h) => w + h }",
        );
    }

    #[test]
    fn grammar_pattern_variant() {
        assert_parses(
            "E Opt { Some(i64), None }
             F f(o: Opt) -> i64 = M o { Some(v) => v, None => 0 }",
        );
    }

    #[test]
    fn grammar_pattern_range() {
        assert_parses("F f(x: i64) -> i64 = M x { 0..10 => 1, _ => 0 }");
    }

    #[test]
    fn grammar_pattern_or() {
        assert_parses("F f(x: i64) -> i64 = M x { 0 | 1 => 1, _ => 0 }");
    }

    #[test]
    fn grammar_pattern_alias() {
        assert_parses("F f(x: i64) -> i64 = M x { n @ 0 => n, _ => 0 }");
    }

    #[test]
    fn grammar_pattern_nested() {
        assert_parses(
            "E Tree { Leaf(i64), Node(Tree, Tree) }
             F f(t: Tree) -> i64 = M t { Leaf(v) => v, Node(_, _) => 0 }",
        );
    }

    #[test]
    fn grammar_pattern_with_guard() {
        // Guard uses `I` (If keyword), not `if`
        assert_parses("F f(x: i64) -> i64 = M x { n I n > 0 => n, _ => 0 }");
    }

    #[test]
    fn grammar_pattern_negative_int() {
        assert_parses("F f(x: i64) -> i64 = M x { -1 => 1, _ => 0 }");
    }
}

// =============================================================================
// Section 6: Generics & Where clauses
// =============================================================================
mod generics {
    use super::*;

    #[test]
    fn grammar_generic_single_type_param() {
        assert_parses("F id<T>(x: T) -> T = x");
    }

    #[test]
    fn grammar_generic_multiple_type_params() {
        assert_parses("F pair<A, B>(a: A, b: B) -> (A, B) = (a, b)");
    }

    #[test]
    fn grammar_generic_with_bounds() {
        assert_parses("F show<T: Display>(x: T) -> i64 = 0");
    }

    #[test]
    fn grammar_generic_multi_bounds() {
        assert_parses("F show<T: Display + Clone>(x: T) -> i64 = 0");
    }

    #[test]
    fn grammar_generic_lifetime() {
        assert_parses("F borrow<'a>(x: &'a i64) -> &'a i64 = x");
    }

    #[test]
    fn grammar_generic_struct() {
        assert_parses("S Box<T> { value: T }");
    }

    #[test]
    fn grammar_generic_enum() {
        assert_parses("E Result<T, E> { Ok(T), Err(E) }");
    }

    #[test]
    fn grammar_where_clause_single() {
        assert_parses("F f<T>(x: T) -> i64 where T: Clone = 0");
    }

    #[test]
    fn grammar_where_clause_multiple() {
        assert_parses("F f<T, U>(x: T, y: U) -> i64 where T: Clone, U: Display = 0");
    }

    #[test]
    fn grammar_where_clause_multi_bound() {
        assert_parses("F f<T>(x: T) -> i64 where T: Clone + Display = 0");
    }

    #[test]
    fn grammar_generic_nested_closing() {
        // >> must be split into two > for nested generics
        assert_parses("F f(x: Vec<Vec<i64>>) -> i64 = 0");
    }

    #[test]
    fn grammar_generic_trait() {
        assert_parses("W Functor<F> { F map(self, f: fn(i64) -> i64) -> i64 }");
    }
}

// =============================================================================
// Section 7: Negative cases (invalid syntax must be rejected)
// =============================================================================
mod negative {
    use super::*;

    #[test]
    fn grammar_neg_function_missing_name() {
        assert_parse_fails("F (x: i64) -> i64 = x");
    }

    #[test]
    fn grammar_neg_function_missing_body() {
        assert_parse_fails("F test(x: i64) -> i64");
    }

    #[test]
    fn grammar_neg_function_missing_closing_paren() {
        assert_parse_fails("F broken(x: i64 -> i64 = x");
    }

    #[test]
    fn grammar_neg_struct_unclosed() {
        assert_parse_fails("S Point { x: f64");
    }

    #[test]
    fn grammar_neg_struct_missing_field_type() {
        assert_parse_fails("S Point { x:, y: f64 }");
    }

    #[test]
    fn grammar_neg_enum_unclosed() {
        assert_parse_fails("E Color { Red, Green");
    }

    #[test]
    fn grammar_neg_match_missing_arrow() {
        assert_parse_fails("F f(x: i64) -> i64 = M x { 0 1, _ => 0 }");
    }

    #[test]
    fn grammar_neg_let_missing_value() {
        assert_parse_fails("F f() -> i64 { x := \n R 0 }");
    }

    #[test]
    fn grammar_neg_unclosed_paren_expr() {
        assert_parse_fails("F f() -> i64 = (1 + 2");
    }

    #[test]
    fn grammar_neg_unclosed_bracket() {
        assert_parse_fails("F f() -> i64 = [1, 2");
    }

    #[test]
    fn grammar_neg_double_arrow_in_lambda() {
        assert_parse_fails("F f() -> i64 = |x| =>");
    }

    #[test]
    fn grammar_neg_extern_without_brace() {
        // N without braces should fail
        assert_parse_fails("N F foo() -> i64");
    }

    #[test]
    fn grammar_neg_const_missing_type() {
        assert_parse_fails("C MAX = 100");
    }

    #[test]
    fn grammar_neg_const_missing_value() {
        assert_parse_fails("C MAX: i64");
    }

    #[test]
    fn grammar_neg_global_missing_type() {
        assert_parse_fails("G counter = 0");
    }

    #[test]
    fn grammar_neg_trait_missing_brace() {
        assert_parse_fails("W Foo F bar(self) -> i64 }");
    }

    #[test]
    fn grammar_neg_impl_missing_brace() {
        assert_parse_fails("X Foo F bar(self) -> i64 = 0 }");
    }

    #[test]
    fn grammar_neg_use_empty() {
        assert_parse_fails("U");
    }
}

// =============================================================================
// Section 8: AST Variant Count Guard (grammar_sync_check)
// =============================================================================

/// Compile-time guard: if Expr variants change, this match will fail to compile.
/// Update this test and add coverage tests for any new variants.
#[test]
fn grammar_sync_expr_variants() {
    fn _check(e: &Expr) {
        match e {
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::StringInterp(_)
            | Expr::Unit
            | Expr::Ident(_)
            | Expr::SelfCall
            | Expr::Binary { .. }
            | Expr::Unary { .. }
            | Expr::Ternary { .. }
            | Expr::If { .. }
            | Expr::Loop { .. }
            | Expr::While { .. }
            | Expr::Match { .. }
            | Expr::Call { .. }
            | Expr::MethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::Field { .. }
            | Expr::Index { .. }
            | Expr::Array(_)
            | Expr::Tuple(_)
            | Expr::StructLit { .. }
            | Expr::Range { .. }
            | Expr::Block(_)
            | Expr::Await(_)
            | Expr::Try(_)
            | Expr::Unwrap(_)
            | Expr::MapLit(_)
            | Expr::Spread(_)
            | Expr::Ref(_)
            | Expr::Deref(_)
            | Expr::Cast { .. }
            | Expr::Assign { .. }
            | Expr::AssignOp { .. }
            | Expr::Lambda { .. }
            | Expr::Spawn(_)
            | Expr::Yield(_)
            | Expr::Comptime { .. }
            | Expr::MacroInvoke(_)
            | Expr::Old(_)
            | Expr::Assert { .. }
            | Expr::Assume(_)
            | Expr::Error { .. }
            | Expr::Lazy(_)
            | Expr::Force(_) => {}
        }
    }
    // Count: 46 variants as of Phase 64
}

/// Compile-time guard: if Item variants change, this match will fail to compile.
#[test]
fn grammar_sync_item_variants() {
    fn _check(i: &Item) {
        match i {
            Item::Function(_)
            | Item::Struct(_)
            | Item::Enum(_)
            | Item::Union(_)
            | Item::TypeAlias(_)
            | Item::TraitAlias(_)
            | Item::Use(_)
            | Item::Trait(_)
            | Item::Impl(_)
            | Item::Macro(_)
            | Item::ExternBlock(_)
            | Item::Const(_)
            | Item::Global(_)
            | Item::Error { .. } => {}
        }
    }
    // Count: 14 variants as of Phase 64
}

/// Compile-time guard: if Stmt variants change, this match will fail to compile.
#[test]
fn grammar_sync_stmt_variants() {
    fn _check(s: &Stmt) {
        match s {
            Stmt::Let { .. }
            | Stmt::LetDestructure { .. }
            | Stmt::Expr(_)
            | Stmt::Return(_)
            | Stmt::Break(_)
            | Stmt::Continue
            | Stmt::Defer(_)
            | Stmt::Error { .. } => {}
        }
    }
    // Count: 8 variants as of Phase 64
}

/// Compile-time guard: if Type variants change, this match will fail to compile.
#[test]
fn grammar_sync_type_variants() {
    fn _check(t: &Type) {
        match t {
            Type::Named { .. }
            | Type::FnPtr { .. }
            | Type::Array(_)
            | Type::ConstArray { .. }
            | Type::Map(_, _)
            | Type::Tuple(_)
            | Type::Optional(_)
            | Type::Result(_)
            | Type::Pointer(_)
            | Type::Ref(_)
            | Type::RefMut(_)
            | Type::Slice(_)
            | Type::SliceMut(_)
            | Type::RefLifetime { .. }
            | Type::RefMutLifetime { .. }
            | Type::Lazy(_)
            | Type::Fn { .. }
            | Type::Unit
            | Type::Infer
            | Type::DynTrait { .. }
            | Type::Associated { .. }
            | Type::Linear(_)
            | Type::Affine(_)
            | Type::ImplTrait { .. }
            | Type::Dependent { .. } => {}
        }
    }
    // Count: 25 variants as of Phase 64
}

/// Compile-time guard: if Pattern variants change, this match will fail to compile.
#[test]
fn grammar_sync_pattern_variants() {
    fn _check(p: &Pattern) {
        match p {
            Pattern::Wildcard
            | Pattern::Ident(_)
            | Pattern::Literal(_)
            | Pattern::Tuple(_)
            | Pattern::Struct { .. }
            | Pattern::Variant { .. }
            | Pattern::Range { .. }
            | Pattern::Or(_)
            | Pattern::Alias { .. } => {}
        }
    }
    // Count: 9 variants as of Phase 64
}

// =============================================================================
// Section 9: Additional coverage for edge cases
// =============================================================================
mod edge_cases {
    use super::*;

    #[test]
    fn grammar_edge_empty_module() {
        let m = parse_ok("");
        assert_eq!(m.items.len(), 0);
    }

    #[test]
    fn grammar_edge_only_comments() {
        let m = parse_ok("# just a comment\n# another comment");
        assert_eq!(m.items.len(), 0);
    }

    #[test]
    fn grammar_edge_chained_method_calls() {
        assert_parses_expr("x.foo().bar().baz()");
    }

    #[test]
    fn grammar_edge_chained_field_access() {
        assert_parses_expr("a.b.c.d");
    }

    #[test]
    fn grammar_edge_nested_if() {
        assert_parses_expr("I a > 0 { I b > 0 { 1 } E { 2 } } E { 3 }");
    }

    #[test]
    fn grammar_edge_nested_match() {
        assert_parses_expr("M x { 0 => M y { 0 => 1, _ => 2 }, _ => 3 }");
    }

    #[test]
    fn grammar_edge_deeply_nested_parens() {
        assert_parses_expr("((((1 + 2))))");
    }

    #[test]
    fn grammar_edge_complex_type_annotation() {
        assert_parses("F f(cb: fn(Vec<i64>, &[i64]) -> (i64, bool)) -> i64 = 0");
    }

    #[test]
    fn grammar_edge_multiple_attributes() {
        assert_parses("#[inline]\n#[cfg(test)]\nF f() -> i64 = 0");
    }

    #[test]
    fn grammar_edge_function_with_default_param() {
        // Default parameter values
        assert_parses("F f(x: i64 = 10) -> i64 = x");
    }

    #[test]
    fn grammar_edge_struct_with_methods() {
        assert_parses(
            "S Counter { count: i64 }
             X Counter {
                 F new() -> Counter = Counter { count: 0 }
                 F inc(self) -> i64 = self.count + 1
             }",
        );
    }

    #[test]
    fn grammar_edge_enum_with_multiple_variant_types() {
        assert_parses(
            "E Value {
                 Int(i64),
                 Float(f64),
                 Pair(i64, f64),
                 Named { x: i64, y: i64 },
                 Unit
             }",
        );
    }

    #[test]
    fn grammar_edge_trait_with_associated_type() {
        assert_parses(
            "W Iterator {
                 T Item
                 F next(self) -> i64
             }",
        );
    }

    #[test]
    fn grammar_edge_impl_with_associated_type() {
        assert_parses(
            "S Nums { data: i64 }
             W Iter { T Item\n F next(self) -> i64 }
             X Nums: Iter {
                 T Item = i64
                 F next(self) -> i64 = 0
             }",
        );
    }

    #[test]
    fn grammar_edge_complex_expression() {
        // Mix of operators, calls, field access
        assert_parses_expr("(a + b) * c.field - foo(1, 2)");
    }

    #[test]
    fn grammar_edge_lambda_as_argument() {
        assert_parses_expr("map(|x| x + 1)");
    }

    #[test]
    fn grammar_edge_index_after_call() {
        assert_parses_expr("get_arr()[0]");
    }

    #[test]
    fn grammar_edge_try_after_call() {
        assert_parses_expr("parse()?");
    }

    #[test]
    fn grammar_edge_cast_chain() {
        assert_parses_expr("x as i64 as f64");
    }

    #[test]
    fn grammar_edge_range_in_loop() {
        assert_parses("F f() -> i64 { L i:0..=100 { i }\n R 0 }");
    }

    #[test]
    fn grammar_edge_multiple_stmts() {
        assert_parses(
            "F f() -> i64 {
                 x := 1
                 y := 2
                 z := x + y
                 R z
             }",
        );
    }

    #[test]
    fn grammar_edge_pub_struct() {
        let m = parse_ok("P S Pub { x: i64 }");
        match &m.items[0].node {
            Item::Struct(s) => assert!(s.is_pub),
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_edge_pub_enum() {
        let m = parse_ok("P E Dir { Up, Down }");
        match &m.items[0].node {
            Item::Enum(e) => assert!(e.is_pub),
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn grammar_edge_pub_trait() {
        let m = parse_ok("P W Show { F show(self) -> i64 }");
        match &m.items[0].node {
            Item::Trait(t) => assert!(t.is_pub),
            other => panic!("Expected Trait, got {:?}", other),
        }
    }

    #[test]
    fn grammar_edge_macro_invocation_in_item() {
        assert_parses("F f() -> i64 = println!(\"hello\")");
    }

    #[test]
    fn grammar_edge_hex_literal() {
        assert_parses_expr("0xFF");
    }

    #[test]
    fn grammar_edge_binary_literal() {
        assert_parses_expr("0b1010");
    }

    #[test]
    fn grammar_edge_octal_literal() {
        assert_parses_expr("0o77");
    }

    #[test]
    fn grammar_edge_underscore_in_number() {
        assert_parses_expr("1_000_000");
    }

    #[test]
    fn grammar_edge_all_assign_ops() {
        // Only +=, -=, *=, /= are supported compound assignments
        assert_parses_expr("x += 1");
        assert_parses_expr("x -= 1");
        assert_parses_expr("x *= 2");
        assert_parses_expr("x /= 2");
    }

    #[test]
    fn grammar_edge_multiline_struct() {
        assert_parses(
            "S Config {
                 width: i64,
                 height: i64,
                 depth: i64,
                 name: str
             }",
        );
    }

    #[test]
    fn grammar_edge_generic_impl_for_trait() {
        assert_parses(
            "W Add { F add(self, other: i64) -> i64 }
             S Num { v: i64 }
             X Num: Add {
                 F add(self, other: i64) -> i64 = self.v + other
             }",
        );
    }

    #[test]
    fn grammar_edge_union_public() {
        let m = parse_ok("P O Val { i: i64, f: f64 }");
        match &m.items[0].node {
            Item::Union(u) => assert!(u.is_pub),
            other => panic!("Expected Union, got {:?}", other),
        }
    }
}

// =============================================================================
// Section 10: Dependent Types (Refinement types)
// =============================================================================
mod dependent_types {
    use super::*;

    #[test]
    fn grammar_dependent_type_basic() {
        // {n: i64 | n > 0} — positive integer refinement
        assert_parses("F f(x: {n: i64 | n > 0}) -> i64 = x");
    }

    #[test]
    fn grammar_dependent_type_ast_structure() {
        let m = parse_ok("F f(x: {n: i64 | n > 0}) -> i64 = x");
        match &m.items[0].node {
            Item::Function(f) => {
                let param_ty = &f.params[0].ty.node;
                match param_ty {
                    Type::Dependent {
                        var_name,
                        base,
                        predicate,
                    } => {
                        assert_eq!(var_name, "n");
                        assert!(matches!(base.node, Type::Named { .. }));
                        // predicate is n > 0, a binary expression
                        assert!(matches!(predicate.node, Expr::Binary { .. }));
                    }
                    other => panic!("Expected Dependent type, got {:?}", other),
                }
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_dependent_type_equality_predicate() {
        // {x: i64 | x == 42}
        assert_parses("F f(x: {x: i64 | x == 42}) -> i64 = x");
    }

    #[test]
    fn grammar_dependent_type_complex_predicate() {
        // {n: i64 | n >= 0 && n < 100}
        assert_parses("F f(x: {n: i64 | n >= 0 && n < 100}) -> i64 = x");
    }

    #[test]
    fn grammar_dependent_type_in_return_position() {
        assert_parses("F abs(x: i64) -> {r: i64 | r >= 0} = I x < 0 { 0 - x } E { x }");
    }

    #[test]
    fn grammar_dependent_type_with_function_call_predicate() {
        // {s: str | len(s) > 0}
        assert_parses("F f(x: {s: str | len(s) > 0}) -> i64 = 0");
    }

    #[test]
    fn grammar_dependent_type_bool_base() {
        // {b: bool | b == true}
        assert_parses("F f(x: {b: bool | b == true}) -> i64 = 0");
    }

    #[test]
    fn grammar_dependent_type_nested_in_generic() {
        // Vec<{n: i64 | n > 0}>
        assert_parses("F f(x: Vec<{n: i64 | n > 0}>) -> i64 = 0");
    }
}

// =============================================================================
// Section 11: Contract Attributes (requires/ensures/invariant/decreases)
// =============================================================================
mod contract_attributes {
    use super::*;

    #[test]
    fn grammar_contract_requires_basic() {
        let m = parse_ok("#[requires(x > 0)]\nF f(x: i64) -> i64 = x");
        match &m.items[0].node {
            Item::Function(f) => {
                assert_eq!(f.attributes.len(), 1);
                assert_eq!(f.attributes[0].name, "requires");
                assert!(f.attributes[0].expr.is_some());
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_contract_ensures_basic() {
        let m = parse_ok("#[ensures(result >= 0)]\nF abs(x: i64) -> i64 = I x < 0 { 0 - x } E { x }");
        match &m.items[0].node {
            Item::Function(f) => {
                assert_eq!(f.attributes[0].name, "ensures");
                assert!(f.attributes[0].expr.is_some());
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_contract_invariant() {
        assert_parses("#[invariant(self.count >= 0)]\nF inc(self) -> i64 = self.count + 1");
    }

    #[test]
    fn grammar_contract_decreases() {
        assert_parses("#[decreases(n)]\nF fib(n: i64) -> i64 = I n <= 1 { n } E { @(n - 1) + @(n - 2) }");
    }

    #[test]
    fn grammar_contract_multiple_contracts() {
        assert_parses(
            "#[requires(x > 0)]\n#[requires(y > 0)]\n#[ensures(result > 0)]\nF mul(x: i64, y: i64) -> i64 = x * y",
        );
    }

    #[test]
    fn grammar_contract_requires_complex_expr() {
        // Contract with logical AND in predicate
        assert_parses("#[requires(x >= 0 && x < 100)]\nF f(x: i64) -> i64 = x");
    }

    #[test]
    fn grammar_contract_ensures_with_old() {
        // old(expr) in ensures — references pre-state value
        assert_parses("F inc(x: i64) -> i64 { old(x)\n R x + 1 }");
    }

    #[test]
    fn grammar_contract_assert_builtin() {
        assert_parses_expr("assert(x > 0)");
    }

    #[test]
    fn grammar_contract_assert_with_message() {
        assert_parses_expr("assert(x > 0, \"x must be positive\")");
    }

    #[test]
    fn grammar_contract_assume_builtin() {
        assert_parses_expr("assume(x > 0)");
    }

    #[test]
    fn grammar_contract_old_in_expr() {
        assert_parses_expr("old(x) + 1");
    }
}

// =============================================================================
// Section 12: Const Parameters & Variance Annotations
// =============================================================================
mod const_params_and_variance {
    use super::*;

    // --- Const generic parameters ---

    #[test]
    fn grammar_const_param_basic() {
        assert_parses("F f<const N: u64>() -> i64 = 0");
    }

    #[test]
    fn grammar_const_param_ast_structure() {
        let m = parse_ok("F f<const N: u64>() -> i64 = 0");
        match &m.items[0].node {
            Item::Function(f) => {
                assert_eq!(f.generics.len(), 1);
                let param = &f.generics[0];
                assert_eq!(param.name.node, "N");
                assert!(param.is_const());
                match &param.kind {
                    GenericParamKind::Const { ty } => {
                        assert!(matches!(ty.node, Type::Named { .. }));
                    }
                    other => panic!("Expected Const param kind, got {:?}", other),
                }
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_const_param_with_type_param() {
        // Mixed type + const params
        assert_parses("F f<T, const N: u64>(arr: T) -> i64 = 0");
    }

    #[test]
    fn grammar_const_param_multiple() {
        assert_parses("F f<const M: i64, const N: i64>() -> i64 = 0");
    }

    #[test]
    fn grammar_const_param_struct() {
        assert_parses("S Array<T, const N: u64> { data: T }");
    }

    #[test]
    fn grammar_const_param_i64_type() {
        assert_parses("F f<const SIZE: i64>() -> i64 = 0");
    }

    // --- Variance annotations ---

    #[test]
    fn grammar_variance_covariant() {
        let m = parse_ok("S Producer<+T> { value: T }");
        match &m.items[0].node {
            Item::Struct(s) => {
                assert_eq!(s.generics.len(), 1);
                assert!(s.generics[0].is_covariant());
                assert_eq!(s.generics[0].variance, Variance::Covariant);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_variance_contravariant() {
        let m = parse_ok("S Consumer<-T> { handler: fn(T) -> i64 }");
        match &m.items[0].node {
            Item::Struct(s) => {
                assert_eq!(s.generics.len(), 1);
                assert!(s.generics[0].is_contravariant());
                assert_eq!(s.generics[0].variance, Variance::Contravariant);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_variance_invariant_default() {
        let m = parse_ok("S Container<T> { value: T }");
        match &m.items[0].node {
            Item::Struct(s) => {
                assert_eq!(s.generics[0].variance, Variance::Invariant);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_variance_mixed() {
        // +T covariant, -U contravariant, V invariant (default)
        let m = parse_ok("S Mixed<+T, -U, V> { a: T, b: V }");
        match &m.items[0].node {
            Item::Struct(s) => {
                assert_eq!(s.generics.len(), 3);
                assert_eq!(s.generics[0].variance, Variance::Covariant);
                assert_eq!(s.generics[1].variance, Variance::Contravariant);
                assert_eq!(s.generics[2].variance, Variance::Invariant);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn grammar_variance_in_function() {
        assert_parses("F f<+T>(x: T) -> i64 = 0");
    }

    #[test]
    fn grammar_variance_covariant_with_bound() {
        assert_parses("F f<+T: Display>(x: T) -> i64 = 0");
    }

    #[test]
    fn grammar_variance_contravariant_with_bound() {
        assert_parses("F f<-T: Clone>(x: T) -> i64 = 0");
    }

    // --- Higher-kinded type parameters (HKT) ---

    #[test]
    fn grammar_hkt_basic() {
        // Note: use 'Ctr' not 'F' because F is a keyword in Vais
        // Use space between > > to avoid >> being tokenized as Shr
        let m = parse_ok("F f<Ctr<_> >(x: i64) -> i64 = 0");
        match &m.items[0].node {
            Item::Function(f) => {
                assert!(f.generics[0].is_higher_kinded());
                match &f.generics[0].kind {
                    GenericParamKind::HigherKinded { arity, .. } => {
                        assert_eq!(arity, &1);
                    }
                    other => panic!("Expected HigherKinded, got {:?}", other),
                }
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_hkt_multi_arity() {
        // Use space between > > to avoid >> tokenized as Shr
        let m = parse_ok("F f<Ctr<_, _> >(x: i64) -> i64 = 0");
        match &m.items[0].node {
            Item::Function(f) => match &f.generics[0].kind {
                GenericParamKind::HigherKinded { arity, .. } => {
                    assert_eq!(arity, &2);
                }
                other => panic!("Expected HigherKinded, got {:?}", other),
            },
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_hkt_with_bound() {
        // Use space between > > to avoid >> tokenized as Shr
        assert_parses("F f<Ctr<_>: Functor>(x: i64) -> i64 = 0");
    }
}

// =============================================================================
// Section 13: Map/Block Ambiguity (backtracking path tests)
// =============================================================================
mod map_block_ambiguity {
    use super::*;

    #[test]
    fn grammar_map_literal_string_keys() {
        // Clear map literal: {"key": value, ...}
        assert_parses("F f() -> i64 { m := {\"a\": 1, \"b\": 2}\n R 0 }");
    }

    #[test]
    fn grammar_map_literal_single_entry() {
        assert_parses("F f() -> i64 { m := {\"key\": 42}\n R 0 }");
    }

    #[test]
    fn grammar_map_literal_trailing_comma() {
        assert_parses("F f() -> i64 { m := {\"a\": 1, \"b\": 2,}\n R 0 }");
    }

    #[test]
    fn grammar_map_literal_numeric_keys() {
        assert_parses("F f() -> i64 { m := {1: \"one\", 2: \"two\"}\n R 0 }");
    }

    #[test]
    fn grammar_block_simple() {
        // Clear block: {stmts}
        assert_parses_expr("{ x := 1\n x + 1 }");
    }

    #[test]
    fn grammar_block_single_expr() {
        // {expr} — must parse as block, not map
        assert_parses_expr("{ 42 }");
    }

    #[test]
    fn grammar_block_with_let() {
        // {let ...} — clearly a block (let statement)
        assert_parses_expr("{ x := 10\n R x }");
    }

    #[test]
    fn grammar_empty_braces() {
        // {} — empty block
        assert_parses_expr("{}");
    }

    #[test]
    fn grammar_map_literal_is_map_ast() {
        // Verify {k: v} is parsed as MapLit, not Block
        let m = parse_ok("F f() -> i64 { m := {\"x\": 1}\n R 0 }");
        match &m.items[0].node {
            Item::Function(f) => match &f.body {
                FunctionBody::Block(stmts) => {
                    // First stmt is Let binding to map literal
                    match &stmts[0].node {
                        Stmt::Let { value, .. } => match &value.node {
                            Expr::MapLit(pairs) => {
                                assert_eq!(pairs.len(), 1);
                            }
                            other => panic!("Expected MapLit, got {:?}", other),
                        },
                        other => panic!("Expected Let, got {:?}", other),
                    }
                }
                other => panic!("Expected Block body, got {:?}", other),
            },
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn grammar_block_with_ident_not_map() {
        // {x + 1} — ident followed by operator, not colon → block, not map
        assert_parses_expr("{ x + 1 }");
    }

    #[test]
    fn grammar_block_with_return() {
        // {R 0} — return statement → clearly block
        assert_parses("F f() -> i64 { R 0 }");
    }

    #[test]
    fn grammar_map_vs_block_ident_colon() {
        // Ambiguous: {x: 1} could be map or struct-like
        // The parser tries map first (key: value pattern)
        assert_parses("F f() -> i64 { m := {x: 1}\n R 0 }");
    }
}

// =============================================================================
// Section 14: Negative cases for new features
// =============================================================================
mod negative_new_features {
    use super::*;

    #[test]
    fn grammar_neg_dependent_type_missing_pipe() {
        // {n: i64 n > 0} — missing |
        assert_parse_fails("F f(x: {n: i64 n > 0}) -> i64 = x");
    }

    #[test]
    fn grammar_neg_dependent_type_missing_closing_brace() {
        // {n: i64 | n > 0 — missing }
        assert_parse_fails("F f(x: {n: i64 | n > 0) -> i64 = x");
    }

    #[test]
    fn grammar_neg_dependent_type_no_var_name() {
        // {: i64 | true} — missing variable name
        assert_parse_fails("F f(x: {: i64 | true}) -> i64 = x");
    }

    #[test]
    fn grammar_neg_contract_requires_no_parens() {
        // #[requires x > 0] — missing parentheses
        assert_parse_fails("#[requires x > 0]\nF f(x: i64) -> i64 = x");
    }

    #[test]
    fn grammar_neg_const_param_no_type() {
        // const N — missing type annotation
        assert_parse_fails("F f<const N>() -> i64 = 0");
    }
}
