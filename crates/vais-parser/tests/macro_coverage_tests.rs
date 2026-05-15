//! Parser macro surface tests.
//!
//! Macro invocation syntax is still accepted in expression position. Surface
//! `macro name! { ... }` declarations are intentionally hard-blocked until the
//! language semantics are certified.

use vais_parser::parse;

fn parse_ok(source: &str) {
    parse(source).unwrap_or_else(|e| panic!("Parse failed for: {}\nErr: {:?}", source, e));
}

fn parse_err(source: &str) {
    assert!(
        parse(source).is_err(),
        "Expected parse error for: {}",
        source
    );
}

#[test]
fn macro_declarations_are_hard_blocked() {
    let declarations = [
        r#"macro empty! { () => {} }"#,
        r#"macro one! { () => { 1 } }"#,
        r#"macro add! { ($a:expr, $b:expr) => { $a + $b } }"#,
        r#"macro list! { ($($x:expr),*) => { 0 } }"#,
        r#"macro repeat_plus! { ($($x:expr),+) => { $($x)+ } }"#,
        r#"macro typed! { ($t:ty) => { 0 } }"#,
        r#"macro token_tree! { ($t:tt) => { $t } }"#,
        r#"pub macro exported! { () => { 42 } }"#,
    ];

    for source in declarations {
        parse_err(source);
    }
}

#[test]
fn macro_invocation_paren() {
    parse_ok("fn test() -> i64 = my_macro!(1, 2, 3)");
}

#[test]
fn macro_invocation_bracket() {
    parse_ok("fn test() -> i64 = my_macro![1, 2, 3]");
}

#[test]
fn macro_invocation_no_args() {
    parse_ok("fn test() -> i64 = my_macro!()");
}

#[test]
fn macro_invocation_brace() {
    parse_ok("fn test() -> i64 = my_macro!{1, 2, 3}");
}
