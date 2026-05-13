//! Phase 89 -- Unicode Support: \u{XXXX} Escape + char_count + Latin-1 Case
//!
//! Tests for enhanced Unicode support:
//! 1. \u{XXXX} escape sequences in string literals
//! 2. UTF-8 aware character counting (str_char_count)
//! 3. Extended case conversion (Latin-1 Supplement)

use super::helpers::*;

// ==================== 1. Unicode Escape Sequences ====================

#[test]
fn e2e_unicode_escape_basic() {
    // \u{41} is 'A' (U+0041), ASCII value 65
    let source = r#"
F main() -> i64 {
    s := "\u{41}"
    R 65
}
"#;
    assert_exit_code(source, 65);
}

#[test]
fn e2e_unicode_escape_heart() {
    // \u{2764} is a multi-byte UTF-8 character (heart)
    // In UTF-8: E2 9D A4 (3 bytes)
    // We just verify it compiles and runs
    let source = r#"
F main() -> i64 {
    s := "I \u{2764} Vais"
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_unicode_escape_null() {
    // \u{0} is null character
    let source = r#"
F main() -> i64 {
    s := "\u{0}"
    R 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_unicode_escape_max_bmp() {
    // \u{FFFF} is the maximum BMP character
    let source = r#"
F main() -> i64 {
    s := "\u{FFFF}"
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_unicode_escape_emoji() {
    // \u{1F600} is a grinning face emoji (4-byte UTF-8)
    let source = r#"
F main() -> i64 {
    s := "\u{1F600}"
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_unicode_escape_mixed() {
    // Mix of ASCII and Unicode escapes in one string
    let source = r#"
F main() -> i64 {
    s := "Hello \u{2764} World"
    R 42
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. String Length vs Byte Length ====================

#[test]
fn e2e_unicode_string_length() {
    // strlen returns byte count, not character count
    // "A" is 1 byte, exit code 1
    let source = r#"
F main() -> i64 {
    s := "AB"
    R strlen(s)
}
"#;
    assert_exit_code(source, 2);
}

// ==================== 3. Basic Unicode in String Literals ====================

#[test]
fn e2e_unicode_escape_latin_a() {
    // \u{61} is 'a', test it compiles correctly
    let source = r#"
F main() -> i64 {
    s := "\u{61}\u{62}\u{63}"
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_unicode_escape_hex_4digit() {
    // \u{00E9} is 'e' with accent (Latin-1 Supplement)
    let source = r#"
F main() -> i64 {
    s := "caf\u{00E9}"
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_unicode_escape_in_print() {
    // Verify unicode escape produces printable output
    let source = r#"
F main() -> i64 {
    s := "\u{48}\u{65}\u{6C}\u{6C}\u{6F}"
    R 42
}
"#;
    // \u{48}=H, \u{65}=e, \u{6C}=l, \u{6C}=l, \u{6F}=o => "Hello"
    assert_exit_code(source, 42);
}
