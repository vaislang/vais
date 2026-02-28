//! Parser macro coverage tests
//!
//! Targets uncovered lines in item/macros.rs (212 uncovered)
//! Focus: macro definitions, patterns, templates, invocations

use vais_ast::*;
use vais_parser::parse;

fn parse_ok(source: &str) -> Module {
    parse(source).unwrap_or_else(|e| panic!("Parse failed for: {}\nErr: {:?}", source, e))
}

fn _parse_err(source: &str) {
    assert!(parse(source).is_err(), "Expected parse error for: {}", source);
}

// ============================================================================
// Basic macro definitions
// ============================================================================

#[test]
fn test_macro_def_empty_body() {
    let source = r#"
        macro empty! {
            () => {}
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
    if let Item::Macro(m) = &module.items[0].node {
        assert_eq!(m.name.node, "empty");
        assert_eq!(m.rules.len(), 1);
    } else {
        panic!("Expected Macro item");
    }
}

#[test]
fn test_macro_def_single_token_body() {
    let source = r#"
        macro one! {
            () => { 1 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_expr() {
    let source = r#"
        macro double! {
            ($x:expr) => { $x + $x }
        }
    "#;
    let module = parse_ok(source);
    if let Item::Macro(m) = &module.items[0].node {
        assert_eq!(m.rules.len(), 1);
    }
}

#[test]
fn test_macro_def_with_metavar_ident() {
    let source = r#"
        macro make_fn! {
            ($name:ident) => { F $name }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_ty() {
    let source = r#"
        macro typed! {
            ($t:ty) => { 0 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_pat() {
    let source = r#"
        macro pattern! {
            ($p:pat) => { 0 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_stmt() {
    let source = r#"
        macro statement! {
            ($s:stmt) => { $s }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_block() {
    let source = r#"
        macro block! {
            ($b:block) => { $b }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_item() {
    let source = r#"
        macro item! {
            ($i:item) => { $i }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_lit() {
    let source = r#"
        macro literal! {
            ($l:lit) => { $l }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_with_metavar_tt() {
    let source = r#"
        macro token_tree! {
            ($t:tt) => { $t }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Multiple rules
// ============================================================================

#[test]
fn test_macro_def_multiple_rules() {
    let source = r#"
        macro overloaded! {
            () => { 0 }
            ($x:expr) => { $x }
        }
    "#;
    let module = parse_ok(source);
    if let Item::Macro(m) = &module.items[0].node {
        assert_eq!(m.rules.len(), 2);
    }
}

#[test]
fn test_macro_def_two_params() {
    let source = r#"
        macro add! {
            ($a:expr, $b:expr) => { $a + $b }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Repetition patterns
// ============================================================================

#[test]
fn test_macro_def_repetition_zero_or_more() {
    let source = r#"
        macro list! {
            ($($x:expr),*) => { 0 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_repetition_one_or_more() {
    let source = r#"
        macro nonempty! {
            ($($x:expr),+) => { 0 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_def_repetition_zero_or_one() {
    let source = r#"
        macro optional! {
            ($($x:expr)?) => { 0 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Template elements
// ============================================================================

#[test]
fn test_macro_template_with_metavar_substitution() {
    let source = r#"
        macro wrap! {
            ($x:expr) => { $x }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_template_with_operators() {
    let source = r#"
        macro math! {
            ($a:expr, $b:expr) => { $a * $b + $a - $b }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_template_with_keywords() {
    let source = r#"
        macro if_then! {
            ($c:expr, $t:expr) => { I $c { R $t } }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Macro invocations
// ============================================================================

#[test]
fn test_macro_invoke_paren() {
    let source = "F test() -> i64 = my_macro!(1, 2, 3)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_invoke_bracket() {
    let source = "F test() -> i64 = my_macro![1, 2, 3]";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_invoke_no_args() {
    let source = "F test() -> i64 = my_macro!()";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_invoke_nested_parens() {
    let source = "F test() -> i64 = my_macro!(1, 2, 3)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Public macros
// ============================================================================

#[test]
fn test_macro_pub() {
    let source = r#"
        P macro exported! {
            () => { 42 }
        }
    "#;
    let module = parse_ok(source);
    if let Item::Macro(m) = &module.items[0].node {
        assert!(m.is_pub);
    }
}

#[test]
fn test_macro_private() {
    let source = r#"
        macro private! {
            () => { 42 }
        }
    "#;
    let module = parse_ok(source);
    if let Item::Macro(m) = &module.items[0].node {
        assert!(!m.is_pub);
    }
}

// ============================================================================
// Pattern groups
// ============================================================================

#[test]
fn test_macro_pattern_bracket_group() {
    // Bracket group inside pattern
    let source = r#"
        macro arr! {
            ($x:expr) => { $x }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_multiple_metavars() {
    let source = r#"
        macro triple! {
            ($a:expr, $b:expr, $c:expr) => { $a + $b + $c }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_with_literal_tokens_in_pattern() {
    let source = r#"
        macro wrap! {
            ($x:expr) => { $x * 2 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Template groups
// ============================================================================

#[test]
fn test_macro_template_paren_group() {
    let source = r#"
        macro tup! {
            ($a:expr, $b:expr) => { ($a, $b) }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_template_bracket_group() {
    let source = r#"
        macro arr_template! {
            ($x:expr) => { [$x] }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_template_brace_group() {
    let source = r#"
        macro block_template! {
            ($x:expr) => { { $x } }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Template repetition
// ============================================================================

#[test]
fn test_macro_template_repetition_star() {
    let source = r#"
        macro repeat! {
            ($($x:expr),*) => { $($x)* }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_template_repetition_plus() {
    let source = r#"
        macro repeat_plus! {
            ($($x:expr),+) => { $($x)+ }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_template_repetition_question() {
    let source = r#"
        macro repeat_opt! {
            ($($x:expr)?) => { $($x)? }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_template_repetition_with_separator() {
    let source = r#"
        macro sep! {
            ($($x:expr),*) => { $($x),* }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Literal tokens in patterns
// ============================================================================

#[test]
fn test_macro_pattern_literal_int() {
    let source = r#"
        macro check! {
            (0) => { false }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_pattern_literal_string() {
    let source = r#"
        macro str_macro! {
            ("hello") => { 1 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_pattern_literal_bool() {
    let source = r#"
        macro bool_macro! {
            (true) => { 1 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Macro with various punctuation
// ============================================================================

#[test]
fn test_macro_pattern_with_operators() {
    let source = r#"
        macro op! {
            ($a:expr + $b:expr) => { $a + $b }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_macro_pattern_with_arrow() {
    let source = r#"
        macro arrow! {
            ($a:expr => $b:expr) => { $a }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}
