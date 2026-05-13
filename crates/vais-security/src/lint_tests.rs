use super::*;

#[test]
fn test_is_snake_case() {
    assert!(is_snake_case("hello_world"));
    assert!(is_snake_case("foo"));
    assert!(is_snake_case("_private"));
    assert!(is_snake_case("my_func_2"));
    assert!(!is_snake_case("HelloWorld"));
    assert!(!is_snake_case("camelCase"));
    assert!(!is_snake_case("CONSTANT"));
}

#[test]
fn test_is_pascal_case() {
    assert!(is_pascal_case("HelloWorld"));
    assert!(is_pascal_case("Vec"));
    assert!(is_pascal_case("MyStruct"));
    assert!(!is_pascal_case("hello_world"));
    assert!(!is_pascal_case("my_struct"));
}

#[test]
fn test_to_snake_case() {
    assert_eq!(to_snake_case("HelloWorld"), "hello_world");
    assert_eq!(to_snake_case("myFunc"), "my_func");
    assert_eq!(to_snake_case("already_snake"), "already_snake");
}

#[test]
fn test_lint_level_display() {
    assert_eq!(LintLevel::Warning.to_string(), "warning");
    assert_eq!(LintLevel::Error.to_string(), "error");
    assert_eq!(LintLevel::Hint.to_string(), "hint");
}

#[test]
fn test_lint_category_display() {
    assert_eq!(LintCategory::DeadCode.to_string(), "dead-code");
    assert_eq!(LintCategory::UnusedImport.to_string(), "unused-import");
    assert_eq!(
        LintCategory::NamingConvention.to_string(),
        "naming-convention"
    );
}

#[test]
fn test_dead_code_detection() {
    let source = r#"
F helper() -> i64 {
42
}

F main() -> i64 {
0
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut analyzer = LintAnalyzer::new();
    let diagnostics = analyzer.analyze(&module);

    let dead_code: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category == LintCategory::DeadCode)
        .collect();
    assert_eq!(dead_code.len(), 1);
    assert!(dead_code[0].message.contains("helper"));
}

#[test]
fn test_no_dead_code_when_called() {
    let source = r#"
F helper() -> i64 {
42
}

F main() -> i64 {
helper()
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut analyzer = LintAnalyzer::new();
    let diagnostics = analyzer.analyze(&module);

    let dead_code: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category == LintCategory::DeadCode)
        .collect();
    assert_eq!(dead_code.len(), 0);
}

#[test]
fn test_unused_import_detection() {
    let source = r#"
U std::io
U std::math

F main() -> i64 {
0
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut analyzer = LintAnalyzer::new();
    let diagnostics = analyzer.analyze(&module);

    let unused: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category == LintCategory::UnusedImport)
        .collect();
    // Both io and math are unused
    assert!(
        unused.len() >= 2,
        "Expected >= 2 unused imports, got {}",
        unused.len()
    );
}

#[test]
fn test_naming_convention() {
    let source = r#"
F BadName() -> i64 {
42
}

F main() -> i64 {
0
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut analyzer = LintAnalyzer::new();
    let diagnostics = analyzer.analyze(&module);

    let naming: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category == LintCategory::NamingConvention)
        .collect();
    assert!(!naming.is_empty(), "Expected naming convention warnings");
    assert!(naming[0].message.contains("BadName"));
}

#[test]
fn test_underscore_prefix_suppresses_dead_code() {
    let source = r#"
F _internal() -> i64 {
42
}

F main() -> i64 {
0
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut analyzer = LintAnalyzer::new();
    let diagnostics = analyzer.analyze(&module);

    let dead_code: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category == LintCategory::DeadCode)
        .collect();
    assert_eq!(
        dead_code.len(),
        0,
        "Underscore prefix should suppress dead code"
    );
}

#[test]
fn test_unreachable_code() {
    let source = r#"
F main() -> i64 {
R 0
x := 42
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut analyzer = LintAnalyzer::new();
    let diagnostics = analyzer.analyze(&module);

    let unreachable: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category == LintCategory::UnreachableCode)
        .collect();
    assert!(!unreachable.is_empty(), "Expected unreachable code warning");
}

#[test]
fn test_unsafe_audit_deref() {
    let source = r#"
F main() -> i64 {
ptr := malloc(100)
*ptr
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut analyzer = LintAnalyzer::new();
    let diagnostics = analyzer.analyze(&module);

    let unsafe_audit: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category == LintCategory::UnsafeAudit)
        .collect();
    assert!(
        !unsafe_audit.is_empty(),
        "Expected unsafe audit warnings for malloc/deref"
    );
}

#[test]
fn test_lint_diagnostic_display() {
    let diag = LintDiagnostic {
        code: "L100".to_string(),
        level: LintLevel::Warning,
        category: LintCategory::DeadCode,
        message: "function 'foo' is never called".to_string(),
        span: Span { start: 0, end: 10 },
        suggestion: Some("Remove or prefix with '_'".to_string()),
    };
    let display = format!("{}", diag);
    assert!(display.contains("L100"));
    assert!(display.contains("warning"));
    assert!(display.contains("foo"));
}
