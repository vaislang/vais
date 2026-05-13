//! Property-based fuzz tests for the Vais parser.
//! Exercises the parser with randomly generated inputs to ensure
//! it never panics and always returns Ok or Err gracefully.

use proptest::prelude::*;
use vais_parser::parse;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn parser_never_panics_on_arbitrary_input(input in "\\PC{0,300}") {
        let _ = parse(&input);
    }

    #[test]
    fn parser_never_panics_on_ascii_input(input in "[[:ascii:]]{0,500}") {
        let _ = parse(&input);
    }

    #[test]
    fn parser_never_panics_on_function_fragments(
        name in "[a-z_][a-z0-9_]{0,10}",
        params in prop::collection::vec(
            ("[a-z_][a-z0-9_]{0,5}", prop_oneof![Just("i64"), Just("str"), Just("bool"), Just("f64")]),
            0..5
        ),
        body_expr in prop_oneof![
            Just("0"),
            Just("42"),
            Just("true"),
            Just("x + y"),
            Just("x * 2 + 1"),
            Just("\"hello\""),
        ],
        has_return_type in proptest::bool::ANY,
        has_body_braces in proptest::bool::ANY,
    ) {
        let param_str: String = params
            .iter()
            .map(|(n, t)| format!("{}: {}", n, t))
            .collect::<Vec<_>>()
            .join(", ");
        let ret = if has_return_type { " -> i64" } else { "" };
        let body = if has_body_braces {
            format!("{{ {} }}", body_expr)
        } else {
            body_expr.to_string()
        };
        let source = format!("F {}({}){} {}", name, param_str, ret, body);
        let _ = parse(&source);
    }

    #[test]
    fn parser_never_panics_on_struct_fragments(
        name in "[A-Z][a-zA-Z0-9_]{0,10}",
        fields in prop::collection::vec(
            ("[a-z_][a-z0-9_]{0,5}", prop_oneof![Just("i64"), Just("str"), Just("bool"), Just("f64"), Just("Vec<i64>")]),
            0..8
        ),
    ) {
        let field_str: String = fields
            .iter()
            .map(|(n, t)| format!("    {}: {}", n, t))
            .collect::<Vec<_>>()
            .join(",\n");
        let source = format!("S {} {{\n{}\n}}", name, field_str);
        let _ = parse(&source);
    }

    #[test]
    fn parser_never_panics_on_enum_fragments(
        name in "[A-Z][a-zA-Z0-9_]{0,10}",
        variants in prop::collection::vec(
            "[A-Z][a-zA-Z0-9_]{0,8}",
            1..6
        ),
    ) {
        let variant_str = variants.join(",\n    ");
        let source = format!("E {} {{\n    {}\n}}", name, variant_str);
        let _ = parse(&source);
    }

    #[test]
    fn parser_never_panics_on_nested_expressions(
        depth in 1usize..30,
        op in prop_oneof![Just("+"), Just("-"), Just("*"), Just("/"), Just("%")],
    ) {
        let mut source = String::from("F f() -> i64 ");
        for _ in 0..depth {
            source.push('(');
        }
        source.push('1');
        for i in 0..depth {
            source.push_str(&format!(" {} {}", op, i + 2));
            source.push(')');
        }
        let _ = parse(&source);
    }

    #[test]
    fn parser_never_panics_on_match_expressions(
        arms in prop::collection::vec(
            (prop_oneof![Just("0"), Just("1"), Just("42"), Just("_"), Just("x")],
             prop_oneof![Just("0"), Just("1"), Just("true"), Just("\"ok\"")]),
            1..10
        ),
    ) {
        let arm_str: String = arms
            .iter()
            .map(|(pat, body)| format!("    {} => {}", pat, body))
            .collect::<Vec<_>>()
            .join(",\n");
        let source = format!("F f(x: i64) -> i64 M x {{\n{}\n}}", arm_str);
        let _ = parse(&source);
    }

    #[test]
    fn parser_never_panics_on_if_chains(
        depth in 1usize..15,
    ) {
        let mut source = String::from("F f(x: i64) -> i64 ");
        for i in 0..depth {
            source.push_str(&format!("I x == {} {{ {} }} E ", i, i * 10));
        }
        source.push_str("{ 0 }");
        let _ = parse(&source);
    }
}
