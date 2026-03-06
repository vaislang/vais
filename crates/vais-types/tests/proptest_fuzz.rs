//! Property-based fuzz tests for the Vais type checker.
//! Exercises the type checker with randomly generated valid-ish programs
//! to ensure it never panics.

use proptest::prelude::*;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Helper: parse and type-check a source string, returning whether TC succeeded.
fn parse_and_check(source: &str) -> bool {
    if let Ok(module) = parse(source) {
        let mut checker = TypeChecker::new();
        checker.check_module(&module).is_ok()
    } else {
        // Parse failed, TC not exercised but that's OK
        true
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn type_checker_never_panics_on_arbitrary_input(input in "\\PC{0,200}") {
        if let Ok(module) = parse(&input) {
            let mut checker = TypeChecker::new();
            let _ = checker.check_module(&module);
        }
    }

    #[test]
    fn type_checker_never_panics_on_valid_functions(
        name in "[a-z_][a-z0-9_]{0,8}",
        ret_type in prop_oneof![Just("i64"), Just("bool"), Just("f64"), Just("str")],
        body in prop_oneof![
            Just("0"),
            Just("42"),
            Just("true"),
            Just("false"),
            Just("3.14"),
            Just("\"hello\""),
        ],
    ) {
        let source = format!("F {}() -> {} {}", name, ret_type, body);
        let _ = parse_and_check(&source);
    }

    #[test]
    fn type_checker_never_panics_on_binary_ops(
        lhs in prop_oneof![Just("1"), Just("2"), Just("42"), Just("100")],
        op in prop_oneof![Just("+"), Just("-"), Just("*"), Just("/"), Just("%")],
        rhs in prop_oneof![Just("1"), Just("2"), Just("42"), Just("100")],
    ) {
        let source = format!("F f() -> i64 {} {} {}", lhs, op, rhs);
        let _ = parse_and_check(&source);
    }

    #[test]
    fn type_checker_never_panics_on_variable_bindings(
        count in 1usize..10,
    ) {
        let mut lines = Vec::new();
        lines.push("F f() -> i64 {".to_string());
        for i in 0..count {
            lines.push(format!("    x{} := {}", i, i * 7));
        }
        // Return sum of first and last
        if count > 0 {
            lines.push(format!("    x0 + x{}", count - 1));
        } else {
            lines.push("    0".to_string());
        }
        lines.push("}".to_string());
        let source = lines.join("\n");
        let _ = parse_and_check(&source);
    }

    #[test]
    fn type_checker_never_panics_on_struct_programs(
        name in "[A-Z][a-zA-Z]{0,8}",
        field_count in 1usize..5,
    ) {
        let mut fields = Vec::new();
        for i in 0..field_count {
            fields.push(format!("    f{}: i64", i));
        }
        let field_init: Vec<String> = (0..field_count)
            .map(|i| format!("f{}: {}", i, i * 10))
            .collect();
        let source = format!(
            "S {} {{\n{}\n}}\n\nF main() -> i64 {{\n    s := {} {{ {} }}\n    s.f0\n}}",
            name,
            fields.join(",\n"),
            name,
            field_init.join(", ")
        );
        let _ = parse_and_check(&source);
    }

    #[test]
    fn type_checker_never_panics_on_generic_functions(
        type_params in prop::collection::vec("[A-Z]", 1..4),
    ) {
        let params: String = type_params
            .iter()
            .enumerate()
            .map(|(i, t)| format!("x{}: {}", i, t))
            .collect::<Vec<_>>()
            .join(", ");
        let generics = type_params.join(", ");
        let source = format!(
            "F identity<{generics}>({params}) -> {} {{\n    x0\n}}",
            type_params[0]
        );
        let _ = parse_and_check(&source);
    }

    #[test]
    fn type_checker_never_panics_on_if_expressions(
        depth in 1usize..8,
    ) {
        let mut source = String::from("F f(x: i64) -> i64 ");
        for i in 0..depth {
            source.push_str(&format!("I x == {} {{ {} }} E ", i, i * 10));
        }
        source.push_str("{ 0 }");
        let _ = parse_and_check(&source);
    }

    #[test]
    fn type_checker_never_panics_on_recursive_functions(
        base_case in 0i64..10,
    ) {
        let source = format!(
            "F fib(n: i64) -> i64 I n <= {} {{ n }} E {{ @(n - 1) + @(n - 2) }}",
            base_case
        );
        let _ = parse_and_check(&source);
    }

    #[test]
    fn type_checker_never_panics_on_trait_programs(
        method_count in 1usize..4,
    ) {
        let mut methods = Vec::new();
        let mut impls = Vec::new();
        for i in 0..method_count {
            methods.push(format!("    F m{}(self) -> i64", i));
            impls.push(format!("    F m{}(self) -> i64 {}", i, i * 100));
        }
        let source = format!(
            "W MyTrait {{\n{}\n}}\n\nS MyStruct {{}}\n\nX MyStruct: MyTrait {{\n{}\n}}",
            methods.join("\n"),
            impls.join("\n")
        );
        let _ = parse_and_check(&source);
    }
}
