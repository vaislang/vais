//! Property-based fuzz tests for the Vais lexer.
//! These tests run as part of `cargo test` and exercise the lexer
//! with randomly generated inputs to ensure it never panics.

use proptest::prelude::*;
use vais_lexer::tokenize;

/// The lexer must never panic on arbitrary UTF-8 strings.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn lexer_never_panics_on_arbitrary_input(input in "\\PC{0,500}") {
        let _ = tokenize(&input);
    }

    #[test]
    fn lexer_never_panics_on_ascii_input(input in "[[:ascii:]]{0,1000}") {
        let _ = tokenize(&input);
    }

    #[test]
    fn lexer_never_panics_on_keyword_soup(
        input in prop::collection::vec(
            prop_oneof![
                Just("F"), Just("S"), Just("E"), Just("I"), Just("L"),
                Just("M"), Just("R"), Just("B"), Just("C"), Just("T"),
                Just("U"), Just("P"), Just("W"), Just("X"), Just("D"),
                Just("A"), Just("Y"), Just("N"), Just("G"), Just("O"),
                Just("mut"), Just("self"), Just("true"), Just("false"),
                Just("spawn"), Just("yield"), Just("where"), Just("dyn"),
                Just("macro"), Just("as"), Just("const"), Just("move"),
                Just(" "), Just("\n"), Just("\t"), Just("("), Just(")"),
                Just("{"), Just("}"), Just("["), Just("]"), Just(":="),
                Just("->"), Just("=>"), Just("|>"), Just(".."), Just("@"),
                Just("#"), Just("//"), Just("/*"), Just("*/"),
                Just("+"), Just("-"), Just("*"), Just("/"), Just("%"),
                Just("=="), Just("!="), Just("<"), Just(">"), Just("<="),
                Just(">="), Just("&&"), Just("||"), Just("!"), Just("?"),
                Just("0"), Just("42"), Just("3.14"), Just("\"hello\""),
                Just("identifier"), Just("_x"), Just("camelCase"),
            ],
            0..50
        )
    ) {
        let source: String = input.join("");
        let _ = tokenize(&source);
    }

    #[test]
    fn lexer_never_panics_on_nested_comments(
        depth in 0usize..20,
        inner in "[a-z ]{0,20}"
    ) {
        let mut source = String::new();
        for _ in 0..depth {
            source.push_str("/* ");
        }
        source.push_str(&inner);
        for _ in 0..depth {
            source.push_str(" */");
        }
        let _ = tokenize(&source);
    }

    #[test]
    fn lexer_never_panics_on_string_literals(
        content in "[^\"\\\\]{0,100}",
        terminated in proptest::bool::ANY
    ) {
        let source = if terminated {
            format!("\"{}\"", content)
        } else {
            format!("\"{}",  content)
        };
        let _ = tokenize(&source);
    }

    #[test]
    fn lexer_never_panics_on_numeric_literals(
        digits in "[0-9]{1,20}",
        suffix in prop_oneof![Just(""), Just(".0"), Just("e10"), Just(".5e-3"), Just("i64"), Just("u8")]
    ) {
        let source = format!("{}{}", digits, suffix);
        let _ = tokenize(&source);
    }

    #[test]
    fn lexer_handles_mixed_operators_and_identifiers(input in "[a-zA-Z0-9_ +\\-*/(){}\\[\\]:=;,.<>!?&|^~@#$\n\t]{1,200}") {
        // The lexer should handle any mix of operators, identifiers, and whitespace.
        // Comments (#) may consume remaining input, producing zero tokens.
        let _ = tokenize(&input);
    }
}
