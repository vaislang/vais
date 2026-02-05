//! Tests for security analyzer

use super::*;
use vais_ast::*;
use vais_lexer::tokenize;
use vais_parser::Parser;

/// Helper function to parse Vais code and analyze it
fn analyze_code(source: &str) -> Vec<SecurityFinding> {
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    let mut analyzer = SecurityAnalyzer::new();
    analyzer.analyze(&module)
}

/// Helper to check if findings contain a specific category
fn has_finding(findings: &[SecurityFinding], category: FindingCategory) -> bool {
    findings.iter().any(|f| f.category == category)
}

/// Helper to count findings of a specific severity
fn count_severity(findings: &[SecurityFinding], severity: Severity) -> usize {
    findings.iter().filter(|f| f.severity == severity).count()
}

#[test]
fn test_malloc_buffer_overflow_detection() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(100)
            store_i64(ptr, 42)
            free(ptr)
            0
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::BufferOverflow));
    assert!(findings.iter().any(|f| f.description.contains("malloc")));
}

#[test]
fn test_use_after_free_detection() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(8)
            free(ptr)
            val := *ptr
            val
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::UseAfterFree));
    assert!(findings.iter().any(|f| f.description.contains("freed")));
}

#[test]
fn test_pointer_arithmetic_detection() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(100)
            offset := ptr + 10
            store_i64(offset, 42)
            free(ptr)
            0
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::UnsafePointer));
    assert!(findings
        .iter()
        .any(|f| f.description.contains("Pointer arithmetic")));
}

#[test]
fn test_array_indexing_detection() {
    let source = r#"
        F main() -> i64 {
            arr := [1, 2, 3, 4, 5]
            idx := 10
            val := arr[idx]
            val
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::BufferOverflow));
    assert!(findings.iter().any(|f| f.description.contains("indexing")));
}

#[test]
fn test_command_injection_detection() {
    let source = r#"
        F execute_cmd(user_input: String) -> i64 {
            cmd := "rm -rf " + user_input
            system(cmd)
        }
    "#;

    let findings = analyze_code(source);
    // system() call should be detected as injection risk
    assert!(has_finding(&findings, FindingCategory::Injection));
    assert!(findings.iter().any(|f| f.description.contains("system")));
}

#[test]
fn test_sql_injection_detection() {
    let source = r#"
        F get_user(username: String) -> i64 {
            db_query("SELECT * FROM users WHERE name = '" + username + "'")
        }
    "#;

    let findings = analyze_code(source);
    // Should detect SQL injection when string concatenation is used directly in query
    assert!(has_finding(&findings, FindingCategory::Injection));
    assert!(findings.iter().any(|f| f.description.contains("SQL")));
}

#[test]
fn test_hardcoded_password_detection() {
    let source = r#"
        F connect() -> i64 {
            password := "super_secret_password_123"
            login("admin", password)
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::HardcodedSecret));
    assert!(findings.iter().any(|f| f.description.contains("password")));
}

#[test]
fn test_hardcoded_api_key_detection() {
    let source = r#"
        F call_api() -> i64 {
            api_key := "sk-1234567890abcdefghijklmnopqrstuvwxyz"
            authenticate(api_key)
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::HardcodedSecret));
    assert!(count_severity(&findings, Severity::Critical) > 0);
}

#[test]
fn test_integer_overflow_on_user_input() {
    let source = r#"
        F process_input() -> i64 {
            user_val := read()
            result := user_val * 1000000
            result
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::IntegerOverflow));
}

#[test]
fn test_unsafe_c_functions_detection() {
    let source = r#"
        N "C" {
            F strcpy(dest: *i8, src: *i8) -> *i8
            F gets(buf: *i8) -> *i8
            F sprintf(buf: *i8, fmt: *i8, ...) -> i64
        }

        F main() -> i64 {
            0
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::BufferOverflow));
    // Should detect at least strcpy, gets, sprintf
    let unsafe_func_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.category == FindingCategory::BufferOverflow)
        .collect();
    assert!(unsafe_func_findings.len() >= 3);
}

#[test]
fn test_system_exec_functions_in_extern() {
    let source = r#"
        N "C" {
            F system(cmd: *i8) -> i64
            F exec(path: *i8, ...) -> i64
        }

        F main() -> i64 {
            0
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::Injection));
}

#[test]
fn test_load_store_operations() {
    let source = r#"
        F test() -> i64 {
            ptr := malloc(8)
            store_i64(ptr, 100)
            val := load_i64(ptr)
            store_byte(ptr, 42)
            b := load_byte(ptr)
            free(ptr)
            val
        }
    "#;

    let findings = analyze_code(source);
    // Should detect multiple buffer overflow risks
    let buffer_overflow_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.category == FindingCategory::BufferOverflow)
        .collect();
    assert!(buffer_overflow_findings.len() >= 3); // malloc, store_i64, load_i64, store_byte, load_byte
}

#[test]
fn test_unchecked_unwrap() {
    let source = r#"
        F get_value() -> i64? {
            Some(42)
        }

        F main() -> i64 {
            val := unwrap(get_value())
            val
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::UncheckedError));
}

#[test]
fn test_memcpy_operations() {
    let source = r#"
        F copy_data() -> i64 {
            src := malloc(100)
            dst := malloc(100)
            memcpy(dst, src, 100)
            free(src)
            free(dst)
            0
        }
    "#;

    let findings = analyze_code(source);
    assert!(has_finding(&findings, FindingCategory::BufferOverflow));
    assert!(findings.iter().any(|f| f.description.contains("memcpy")));
}

#[test]
fn test_high_entropy_token_detection() {
    let source = r#"
        F authenticate() -> i64 {
            api_key := "xK7mP9qR2sL5nW8tV3gH4jF6bN1cM0dZ"
            send_request(api_key)
        }
    "#;

    let findings = analyze_code(source);
    // High entropy string with "api_key" in variable name should be flagged
    assert!(has_finding(&findings, FindingCategory::HardcodedSecret));
}

#[test]
fn test_clean_code_minimal_findings() {
    let source = r#"
        F add(a: i64, b: i64) -> i64 {
            a + b
        }

        F main() -> i64 {
            result := add(10, 20)
            result
        }
    "#;

    let findings = analyze_code(source);
    // Should have no critical findings
    assert_eq!(count_severity(&findings, Severity::Critical), 0);
}

#[test]
fn test_severity_levels() {
    let source = r#"
        N "C" {
            F strcpy(dst: *i8, src: *i8) -> *i8
        }

        F vulnerable() -> i64 {
            password := "hardcoded_password"
            ptr := malloc(100)
            system("rm -rf /")
            0
        }
    "#;

    let findings = analyze_code(source);

    // Should have multiple critical findings
    let critical = count_severity(&findings, Severity::Critical);
    assert!(
        critical >= 2,
        "Expected at least 2 critical findings, got {}",
        critical
    );

    // Should have at least one high severity finding
    let high = count_severity(&findings, Severity::High);
    assert!(
        high >= 1,
        "Expected at least 1 high severity finding, got {}",
        high
    );
}

#[test]
fn test_complex_pointer_operations() {
    let source = r#"
        F complex_memory() -> i64 {
            # Allocate memory
            ptr1 := malloc(64)
            ptr2 := malloc(32)

            # Pointer arithmetic
            offset := ptr1 + 8
            store_i64(offset, 100)

            # Use after free scenario
            free(ptr1)
            val := load_i64(ptr1)

            # Leak ptr2 (not freed)
            val
        }
    "#;

    let findings = analyze_code(source);

    // Should detect buffer overflow from malloc and stores
    assert!(has_finding(&findings, FindingCategory::BufferOverflow));

    // Should detect pointer arithmetic
    assert!(has_finding(&findings, FindingCategory::UnsafePointer));

    // Should detect use after free
    assert!(has_finding(&findings, FindingCategory::UseAfterFree));
}

#[test]
fn test_string_operations_safe() {
    let source = r#"
        F greet(name: String) -> String {
            greeting := "Hello, "
            message := greeting + name
            message
        }
    "#;

    let findings = analyze_code(source);

    // Normal string concatenation should not trigger injection warnings
    // (only when passed to system/exec/query functions)
    assert!(!has_finding(&findings, FindingCategory::Injection));
}

#[test]
fn test_finding_display() {
    let finding = SecurityFinding::buffer_overflow("Test buffer overflow", Span::new(0, 10));

    let display = format!("{}", finding);
    assert!(display.contains("CRITICAL"));
    assert!(display.contains("Buffer Overflow"));
    assert!(display.contains("Test buffer overflow"));
    assert!(display.contains("bounds checking"));
}

#[test]
fn test_finding_constructors() {
    let span = Span::new(5, 15);

    let f1 = SecurityFinding::buffer_overflow("test", span);
    assert_eq!(f1.severity, Severity::Critical);
    assert_eq!(f1.category, FindingCategory::BufferOverflow);

    let f2 = SecurityFinding::unsafe_pointer("test", span);
    assert_eq!(f2.severity, Severity::High);
    assert_eq!(f2.category, FindingCategory::UnsafePointer);

    let f3 = SecurityFinding::injection("test", span);
    assert_eq!(f3.severity, Severity::Critical);
    assert_eq!(f3.category, FindingCategory::Injection);

    let f4 = SecurityFinding::integer_overflow("test", span);
    assert_eq!(f4.severity, Severity::Medium);
    assert_eq!(f4.category, FindingCategory::IntegerOverflow);

    let f5 = SecurityFinding::unchecked_error("test", span);
    assert_eq!(f5.severity, Severity::Low);
    assert_eq!(f5.category, FindingCategory::UncheckedError);
}
