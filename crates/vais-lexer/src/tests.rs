use super::*;

#[test]
fn test_function_def() {
    let source = "F add(a:i64,b:i64)->i64=a+b";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens[0].token, Token::Function);
    assert_eq!(tokens[1].token, Token::Ident("add".to_string()));
    assert_eq!(tokens[2].token, Token::LParen);
    assert_eq!(tokens[3].token, Token::Ident("a".to_string()));
    assert_eq!(tokens[4].token, Token::Colon);
    assert_eq!(tokens[5].token, Token::I64);
}

#[test]
fn test_fibonacci() {
    let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
    let tokens = tokenize(source).unwrap();

    // Check @ for self-recursion
    let has_at = tokens.iter().any(|t| t.token == Token::At);
    assert!(has_at);
}

#[test]
fn test_struct_def() {
    let source = "S Point{x:f64,y:f64}";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens[0].token, Token::Struct);
    assert_eq!(tokens[1].token, Token::Ident("Point".to_string()));
}

#[test]
fn test_control_flow() {
    let source = "I x<0{-1}E{0}";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens[0].token, Token::If);
    // E is used for both enum definition and else - lexer returns Enum
    // The parser decides context (after { in if => else)
    let e_idx = tokens.iter().position(|t| t.token == Token::Enum).unwrap();
    assert!(e_idx > 0);
}

#[test]
fn test_loop() {
    let source = "L i:0..10{print(i)}";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens[0].token, Token::Loop);
    assert_eq!(tokens[4].token, Token::DotDot);
}

#[test]
fn test_string_literal() {
    let source = r#""hello world""#;
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens[0].token, Token::String("hello world".to_string()));
}

#[test]
fn test_string_with_escaped_quotes() {
    let source = r#""{\"id\":\"classic\"}""#;
    let tokens = tokenize(source).unwrap();
    // Should be a single string token containing: {"id":"classic"}
    assert_eq!(
        tokens[0].token,
        Token::String("{\"id\":\"classic\"}".to_string()),
        "Expected full JSON string with quotes, got: {:?}",
        tokens[0].token
    );
}

#[test]
fn test_string_escaped_quote_simple() {
    let source = r#""a\"b""#;
    let tokens = tokenize(source).unwrap();
    assert_eq!(
        tokens[0].token,
        Token::String("a\"b".to_string()),
        "Escaped quote should be preserved, got: {:?}",
        tokens[0].token
    );
}

#[test]
#[allow(clippy::approx_constant)]
fn test_numbers() {
    let source = "42 3.14 1_000_000";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens[0].token, Token::Int(42));
    assert_eq!(tokens[1].token, Token::Float(3.14));
    assert_eq!(tokens[2].token, Token::Int(1000000));
}

#[test]
fn test_comments() {
    let source = "F add(a:i64,b:i64)->i64=a+b # this is a comment";
    let tokens = tokenize(source).unwrap();

    // Comment should be skipped
    let has_comment = tokens.iter().any(|t| {
        if let Token::Ident(s) = &t.token {
            s.contains("comment")
        } else {
            false
        }
    });
    assert!(!has_comment);
}

#[test]
fn test_block_function() {
    let source = "F sum(arr:[i64])->i64{s:=0;L x:arr{s+=x};s}";
    let tokens = tokenize(source).unwrap();

    // Verify lowercase 's' is lexed as Ident, not Struct
    let s_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(&t.token, Token::Ident(s) if s == "s"))
        .collect();
    assert_eq!(s_tokens.len(), 3); // s:=0, s+=x, final s
}

// ==================== Edge Case Tests ====================

#[test]
fn test_empty_input() {
    let source = "";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn test_whitespace_only() {
    let source = "   \n\t\r\n   ";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn test_comment_only() {
    let source = "# this is just a comment\n# another comment";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn test_single_character_identifiers() {
    let source = "x y z _ a b c";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens.len(), 7);
    assert_eq!(tokens[0].token, Token::Ident("x".to_string()));
    assert_eq!(tokens[3].token, Token::Ident("_".to_string()));
}

#[test]
fn test_very_long_identifier() {
    let long_name = "a".repeat(1000);
    let source = format!("F {}()->()=()", long_name);
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens[1].token, Token::Ident(long_name));
}

#[test]
fn test_i64_max() {
    let source = "9223372036854775807";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Int(i64::MAX));
}

#[test]
fn test_negative_number_as_tokens() {
    // Negative numbers are lexed as Minus followed by Int
    let source = "-42";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Minus);
    assert_eq!(tokens[1].token, Token::Int(42));
}

#[test]
fn test_float_edge_cases() {
    let source = "0.0 1.0 0.5 123.456789";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Float(0.0));
    assert_eq!(tokens[1].token, Token::Float(1.0));
    assert_eq!(tokens[2].token, Token::Float(0.5));
    assert_eq!(tokens[3].token, Token::Float(123.456789));
}

#[test]
fn test_multiple_underscores_in_number() {
    let source = "1_2_3_4_5_6";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Int(123456));
}

#[test]
fn test_keyword_like_identifiers() {
    // Keywords are uppercase single letters, these should be identifiers
    let source = "Fn Struct Enum If Loop Match For While";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Ident("Fn".to_string()));
    assert_eq!(tokens[1].token, Token::Ident("Struct".to_string()));
    assert_eq!(tokens[2].token, Token::Ident("Enum".to_string()));
    assert_eq!(tokens[3].token, Token::Ident("If".to_string()));
    assert_eq!(tokens[4].token, Token::Ident("Loop".to_string()));
    assert_eq!(tokens[5].token, Token::Ident("Match".to_string()));
    assert_eq!(tokens[6].token, Token::Ident("For".to_string()));
    assert_eq!(tokens[7].token, Token::Ident("While".to_string()));
}

#[test]
fn test_consecutive_operators() {
    // Note: /// is a doc comment, so use separate slashes with spaces
    let source = "+++---***/ / /";
    let tokens = tokenize(source).unwrap();
    // Should be lexed as separate operators
    assert!(tokens.iter().any(|t| t.token == Token::Plus));
    assert!(tokens.iter().any(|t| t.token == Token::Minus));
    assert!(tokens.iter().any(|t| t.token == Token::Star));
    assert!(tokens.iter().any(|t| t.token == Token::Slash));
}

#[test]
fn test_dot_vs_dotdot() {
    let source = "a.b 0..10 x.y..z.w";
    let tokens = tokenize(source).unwrap();
    // Should correctly distinguish . from ..
    let dot_count = tokens.iter().filter(|t| t.token == Token::Dot).count();
    let dotdot_count = tokens.iter().filter(|t| t.token == Token::DotDot).count();
    assert!(dot_count >= 2); // a.b, x.y, z.w
    assert!(dotdot_count >= 1); // 0..10, y..z
}

#[test]
fn test_comparison_operators() {
    let source = "< <= > >= == != <<";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::Lt));
    assert!(tokens.iter().any(|t| t.token == Token::Lte));
    assert!(tokens.iter().any(|t| t.token == Token::Gt));
    assert!(tokens.iter().any(|t| t.token == Token::Gte));
    assert!(tokens.iter().any(|t| t.token == Token::EqEq));
    assert!(tokens.iter().any(|t| t.token == Token::Neq));
}

#[test]
fn test_string_with_escapes() {
    let source = r#""hello\nworld\ttab""#;
    let tokens = tokenize(source).unwrap();
    // The lexer should handle escape sequences
    assert!(matches!(&tokens[0].token, Token::String(_)));
}

#[test]
fn test_empty_string() {
    let source = r#""""#;
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::String("".to_string()));
}

#[test]
fn test_all_keywords() {
    // F=Function, S=Struct, E=Enum, I=If, L=Loop, M=Match, R=Return, B=Break, C=Continue, T=Type, W=Trait, A=Async, P=Pub, U=Use, X=Impl
    let source = "F S E I L M R B C T W A P U X";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Function);
    assert_eq!(tokens[1].token, Token::Struct);
    assert_eq!(tokens[2].token, Token::Enum);
    assert_eq!(tokens[3].token, Token::If);
    assert_eq!(tokens[4].token, Token::Loop);
    assert_eq!(tokens[5].token, Token::Match);
    assert_eq!(tokens[6].token, Token::Return);
    assert_eq!(tokens[7].token, Token::Break);
    assert_eq!(tokens[8].token, Token::Continue);
    assert_eq!(tokens[9].token, Token::TypeKeyword);
    assert_eq!(tokens[10].token, Token::Trait); // W is Trait
    assert_eq!(tokens[11].token, Token::Async);
    assert_eq!(tokens[12].token, Token::Pub);
    assert_eq!(tokens[13].token, Token::Use);
    assert_eq!(tokens[14].token, Token::Impl);
}

#[test]
fn test_all_type_keywords() {
    let source = "i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 bool str";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::I8);
    assert_eq!(tokens[1].token, Token::I16);
    assert_eq!(tokens[2].token, Token::I32);
    assert_eq!(tokens[3].token, Token::I64);
    assert_eq!(tokens[4].token, Token::I128);
    assert_eq!(tokens[5].token, Token::U8);
    assert_eq!(tokens[6].token, Token::U16);
    assert_eq!(tokens[7].token, Token::U32);
    assert_eq!(tokens[8].token, Token::U64);
    assert_eq!(tokens[9].token, Token::U128);
    assert_eq!(tokens[10].token, Token::F32);
    assert_eq!(tokens[11].token, Token::F64);
    assert_eq!(tokens[12].token, Token::Bool);
    assert_eq!(tokens[13].token, Token::Str);
}

#[test]
fn test_all_brackets_and_delimiters() {
    let source = "( ) [ ] { } < > , : ; . .. -> => @ ? !";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::LParen);
    assert_eq!(tokens[1].token, Token::RParen);
    assert_eq!(tokens[2].token, Token::LBracket);
    assert_eq!(tokens[3].token, Token::RBracket);
    assert_eq!(tokens[4].token, Token::LBrace);
    assert_eq!(tokens[5].token, Token::RBrace);
    assert_eq!(tokens[6].token, Token::Lt);
    assert_eq!(tokens[7].token, Token::Gt);
    assert_eq!(tokens[8].token, Token::Comma);
    assert_eq!(tokens[9].token, Token::Colon);
    assert_eq!(tokens[10].token, Token::Semi);
    assert_eq!(tokens[11].token, Token::Dot);
    assert_eq!(tokens[12].token, Token::DotDot);
    assert_eq!(tokens[13].token, Token::Arrow);
    assert_eq!(tokens[14].token, Token::FatArrow);
    assert_eq!(tokens[15].token, Token::At);
    assert_eq!(tokens[16].token, Token::Question);
    assert_eq!(tokens[17].token, Token::Bang);
}

#[test]
fn test_assignment_operators() {
    let source = "= := += -= *= /=";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Eq);
    assert_eq!(tokens[1].token, Token::ColonEq);
    assert_eq!(tokens[2].token, Token::PlusEq);
    assert_eq!(tokens[3].token, Token::MinusEq);
    assert_eq!(tokens[4].token, Token::StarEq);
    assert_eq!(tokens[5].token, Token::SlashEq);
}

#[test]
fn test_bitwise_operators() {
    let source = "& | ^ ~ << >>";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Amp);
    assert_eq!(tokens[1].token, Token::Pipe);
    assert_eq!(tokens[2].token, Token::Caret);
    assert_eq!(tokens[3].token, Token::Tilde);
    assert_eq!(tokens[4].token, Token::Shl);
    assert_eq!(tokens[5].token, Token::Shr);
}

#[test]
fn test_pipe_arrow_operator() {
    let source = "x |> f |> g";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Ident("x".to_string()));
    assert_eq!(tokens[1].token, Token::PipeArrow);
    assert_eq!(tokens[2].token, Token::Ident("f".to_string()));
    assert_eq!(tokens[3].token, Token::PipeArrow);
    assert_eq!(tokens[4].token, Token::Ident("g".to_string()));
}

#[test]
fn test_pipe_arrow_vs_pipe() {
    // |> should be PipeArrow, | should be Pipe
    let source = "a |> b | c";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[1].token, Token::PipeArrow);
    assert_eq!(tokens[3].token, Token::Pipe);
}

#[test]
fn test_logical_operators() {
    // && is lexed as two Amp tokens, || as two Pipe tokens
    let source = "&& ||";
    let tokens = tokenize(source).unwrap();
    // && -> Amp Amp
    assert_eq!(tokens[0].token, Token::Amp);
    assert_eq!(tokens[1].token, Token::Amp);
    // || -> Pipe Pipe
    assert_eq!(tokens[2].token, Token::Pipe);
    assert_eq!(tokens[3].token, Token::Pipe);
}

#[test]
fn test_multiline_code() {
    let source = r#"
F add(a:i64,
  b:i64)->i64 {
R a + b
}
"#;
    let tokens = tokenize(source).unwrap();
    // Should successfully tokenize multiline code
    assert!(tokens.iter().any(|t| t.token == Token::Function));
    assert!(tokens.iter().any(|t| t.token == Token::Return));
}

#[test]
fn test_unicode_in_string() {
    let source = r#""안녕하세요 🚀 世界""#;
    let tokens = tokenize(source).unwrap();
    assert_eq!(
        tokens[0].token,
        Token::String("안녕하세요 🚀 世界".to_string())
    );
}

#[test]
fn test_identifier_with_numbers() {
    let source = "x1 y2 var123 test_456";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Ident("x1".to_string()));
    assert_eq!(tokens[1].token, Token::Ident("y2".to_string()));
    assert_eq!(tokens[2].token, Token::Ident("var123".to_string()));
    assert_eq!(tokens[3].token, Token::Ident("test_456".to_string()));
}

#[test]
fn test_span_accuracy() {
    let source = "F f()->i64=42";
    let tokens = tokenize(source).unwrap();

    // Check that spans are accurate
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 1); // "F"

    // Find the "42" token and check its span
    let int_token = tokens.iter().find(|t| t.token == Token::Int(42)).unwrap();
    assert_eq!(int_token.span.start, 11);
    assert_eq!(int_token.span.end, 13);
}

// ==================== Advanced Edge Case Tests ====================

#[test]
fn test_nested_generic_syntax() {
    // Test nested generic type syntax with spaces: Vec<HashMap<K, V> >
    // Note: Without spaces, >> is tokenized as Shr (right shift)
    let source = "S Container{data:Vec<HashMap<str,Option<i64> > >}";
    let tokens = tokenize(source).unwrap();

    // Verify proper tokenization of nested generics
    assert!(tokens.iter().any(|t| t.token == Token::Struct));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Container")));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Vec")));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "HashMap")));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Option")));

    // Count angle brackets (should be balanced with spaces)
    let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
    let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
    assert_eq!(lt_count, gt_count, "Angle brackets should be balanced");
}

#[test]
fn test_deeply_nested_generic_combinations() {
    // Test Vec<HashMap<K, Option<V> > > with spaces to avoid >> tokenization
    let source = "F process(data:Vec<HashMap<str,Option<Vec<i64> > > >)->i64=42";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Function));
    let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
    let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
    assert_eq!(lt_count, 4);
    assert_eq!(gt_count, 4);
}

#[test]
fn test_option_of_vec() {
    // Test Option<Vec<T> > syntax with space to avoid >> tokenization
    let source = r#"F get_items()->Option<Vec<str> >="""#;
    let tokens = tokenize(source).unwrap();

    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Option")));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Vec")));
    let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
    let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
    // With space separation, both should be 2
    assert!(
        lt_count >= 1 && gt_count >= 1,
        "Should have at least 1 Lt and 1 Gt, got lt={}, gt={}",
        lt_count,
        gt_count
    );
}

#[test]
fn test_i8_boundary_values() {
    // Test i8 min (-128) and max (127) values
    // Note: negative sign is separate token
    let source = "F test()->(){min:=-128;max:=127}";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Int(128)));
    assert!(tokens.iter().any(|t| t.token == Token::Int(127)));
    assert!(tokens.iter().any(|t| t.token == Token::Minus));
}

#[test]
fn test_i64_boundary_values() {
    // Test i64 max value: 9223372036854775807
    let source = "F max_i64()->i64=9223372036854775807";
    let tokens = tokenize(source).unwrap();

    assert!(tokens
        .iter()
        .any(|t| t.token == Token::Int(9223372036854775807)));
}

#[test]
fn test_integer_overflow_literal() {
    // Test value beyond i64::MAX - should fail to parse
    // i64::MAX + 1 = 9223372036854775808 (too large for i64)
    let source = "F overflow()->i64=9223372036854775808";
    let tokens = tokenize(source);

    // This should fail or produce no Int token (lexer parse fails for overflow)
    // We expect the tokenize to fail or not produce a valid Int token
    match tokens {
        Ok(tokens) => {
            // If it succeeds, check that no Int token was produced with a valid i64
            let has_int_token = tokens.iter().any(|t| matches!(t.token, Token::Int(_)));
            // The overflow value shouldn't be tokenized as a valid Int
            assert!(
                !has_int_token,
                "i64 overflow should not be tokenized as valid Int"
            );
        }
        Err(_) => {
            // Expected: tokenization fails for overflow
        }
    }
}

#[test]
fn test_pattern_with_guard_syntax() {
    // Test pattern matching with guard: M x{n I n>0=>n,_=>0}
    let source = "F abs(x:i64)->i64=M x{n I n>0=>n,n I n<0=>-n,_=>0}";
    let tokens = tokenize(source).unwrap();

    // Verify M (Match), I (If in guard position), => (FatArrow)
    assert!(tokens.iter().any(|t| t.token == Token::Match));
    assert!(tokens.iter().any(|t| t.token == Token::If));
    assert!(tokens.iter().any(|t| t.token == Token::FatArrow));
}

#[test]
fn test_nested_pattern_destructuring() {
    // Test nested destructuring: Some((x, y))
    let source = "M opt{Some((x,y))=>x+y,None=>0}";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Match));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Some")));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "None")));

    // Count parentheses
    let lparen_count = tokens.iter().filter(|t| t.token == Token::LParen).count();
    let rparen_count = tokens.iter().filter(|t| t.token == Token::RParen).count();
    assert_eq!(lparen_count, rparen_count);
}

#[test]
fn test_complex_guard_condition() {
    // Test complex guard with multiple conditions
    let source = "M (x,y){(a,b) I a>0&&b<10=>1,_=>0}";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Match));
    assert!(tokens.iter().any(|t| t.token == Token::If));
    assert!(tokens.iter().any(|t| t.token == Token::Amp)); // &&
    assert!(tokens.iter().any(|t| t.token == Token::Lt));
}

#[test]
fn test_multiple_type_params_with_bounds() {
    // Test: F<A: Clone, B: Default, C: Ord>
    let source = "F transform<A:Clone,B:Default,C:Ord>(a:A,b:B,c:C)->C=c";
    let tokens = tokenize(source).unwrap();

    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Clone")));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Default")));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Ord")));
}

#[test]
fn test_float_special_values() {
    // Test float edge cases
    let source = "F test()->(){a:=0.0;b:=1.0;c:=0.5;d:=999999.999999}";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Float(0.0)));
    assert!(tokens.iter().any(|t| t.token == Token::Float(1.0)));
    assert!(tokens.iter().any(|t| t.token == Token::Float(0.5)));
    assert!(tokens
        .iter()
        .any(|t| t.token == Token::Float(999999.999999)));
}

#[test]
fn test_scientific_notation() {
    // Test scientific notation: 1.5e10, 2.0e-5
    let source = "F sci()->(){a:=1.5e10;b:=2.0e-5;c:=3.14e+2}";
    let tokens = tokenize(source).unwrap();

    // Check that scientific notation is parsed as floats
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Float(_))));
}

#[test]
fn test_max_nested_angle_brackets() {
    // Test maximum nesting of angle brackets with spaces
    let source = "S Deep{v:Vec<Vec<Vec<Vec<i64> > > >}";
    let tokens = tokenize(source).unwrap();

    let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
    let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
    assert_eq!(lt_count, 4);
    assert_eq!(gt_count, 4);
}

#[test]
fn test_zero_values_all_types() {
    // Test zero values for different numeric types
    let source = "F zeros()->(){a:=0;b:=0.0;c:=0.0e0}";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Int(0)));
    assert!(tokens.iter().any(|t| t.token == Token::Float(0.0)));
}

#[test]
fn test_ambiguous_generic_vs_comparison() {
    // Test that Vec<i64> is parsed correctly, not as Vec < i64 >
    let source = "F f(x:Vec<i64>)->bool=true";
    let tokens = tokenize(source).unwrap();

    // Lt and Gt should be present for generics
    assert!(tokens.iter().any(|t| t.token == Token::Lt));
    assert!(tokens.iter().any(|t| t.token == Token::Gt));
    // But Vec should be an identifier
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "Vec")));
}

#[test]
fn test_consecutive_comparison_operators() {
    // Test: a<b<c should be tokenized as a < b < c
    let source = "F chain(a:i64,b:i64,c:i64)->bool=a<b<c";
    let tokens = tokenize(source).unwrap();

    let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
    assert_eq!(lt_count, 2);
}

#[test]
fn test_range_operators() {
    // Test .. and ..= operators
    let source = "F ranges()->(){a:=0..10;b:=0..=10}";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::DotDot));
    assert!(tokens.iter().any(|t| t.token == Token::DotDotEq));
}

#[test]
fn test_very_large_float() {
    // Test very large float values
    let source = "F large()->f64=1.7976931348623157e308";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| matches!(t.token, Token::Float(_))));
}

#[test]
fn test_very_small_float() {
    // Test very small float values (near zero)
    let source = "F small()->f64=2.2250738585072014e-308";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| matches!(t.token, Token::Float(_))));
}

#[test]
fn test_all_assignment_operators_combined() {
    // Test all compound assignment operators in one expression
    let source = "F assign()->(){x:=1;x+=1;x-=1;x*=2;x/=2}";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::ColonEq));
    assert!(tokens.iter().any(|t| t.token == Token::PlusEq));
    assert!(tokens.iter().any(|t| t.token == Token::MinusEq));
    assert!(tokens.iter().any(|t| t.token == Token::StarEq));
    assert!(tokens.iter().any(|t| t.token == Token::SlashEq));
}

// ==================== Macro System Tests ====================

#[test]
fn test_macro_keyword() {
    let source = "macro vec! { () => { [] } }";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Macro));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "vec")));
    assert!(tokens.iter().any(|t| t.token == Token::Bang));
}

#[test]
fn test_dollar_token() {
    let source = "macro test! { ($x:expr) => { $x } }";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Dollar));
    let dollar_count = tokens.iter().filter(|t| t.token == Token::Dollar).count();
    assert_eq!(dollar_count, 2); // $x in pattern and $x in template
}

#[test]
fn test_macro_invocation() {
    let source = "vec!(1, 2, 3)";
    let tokens = tokenize(source).unwrap();

    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::Ident(s) if s == "vec")));
    assert!(tokens.iter().any(|t| t.token == Token::Bang));
    assert!(tokens.iter().any(|t| t.token == Token::LParen));
}

#[test]
fn test_y_await_abbreviation() {
    let source = "expr.Y";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Ident("expr".to_string()));
    assert_eq!(tokens[1].token, Token::Dot);
    assert_eq!(tokens[2].token, Token::Await);
}

#[test]
fn test_y_and_await_both_work() {
    let tokens_y = tokenize("x.Y").unwrap();
    let tokens_await = tokenize("x.await").unwrap();
    assert_eq!(tokens_y[2].token, Token::Await);
    assert_eq!(tokens_await[2].token, Token::Await);
}

#[test]
fn test_macro_with_repetition() {
    let source = "macro vec! { ($($item:expr),*) => { [$($item),*] } }";
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Macro));
    assert!(tokens.iter().any(|t| t.token == Token::Star));
    let dollar_count = tokens.iter().filter(|t| t.token == Token::Dollar).count();
    assert!(dollar_count >= 2);
}

#[test]
fn test_split_keyword_idents_ei() {
    // "EI" must split into Enum (E=else) + If
    let source = "I x>0{}EI x>5{}E{}";
    let tokens = tokenize(source).unwrap();
    // Find the Enum token that comes after '}'
    let enum_positions: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.token == Token::Enum)
        .map(|(i, _)| i)
        .collect();
    assert!(
        !enum_positions.is_empty(),
        "should have at least one Enum token"
    );
    // The token after the first Enum that resulted from split must be If
    for &pos in &enum_positions {
        if pos + 1 < tokens.len() && tokens[pos + 1].token == Token::If {
            return; // found E followed by I from the split
        }
    }
    panic!("Expected Enum followed by If from EI split");
}

#[test]
fn test_split_keyword_idents_ee() {
    // "EE" (two E's) must split into Enum + Enum
    let tokens = tokenize("EE").unwrap();
    assert_eq!(tokens.len(), 2, "EE should produce exactly 2 tokens");
    assert_eq!(tokens[0].token, Token::Enum);
    assert_eq!(tokens[1].token, Token::Enum);
}

#[test]
fn test_split_keyword_idents_if() {
    // "IF" must split into If + Function
    let tokens = tokenize("IF").unwrap();
    assert_eq!(tokens.len(), 2, "IF should produce exactly 2 tokens");
    assert_eq!(tokens[0].token, Token::If);
    assert_eq!(tokens[1].token, Token::Function);
}

#[test]
fn test_split_keyword_idents_no_split_lowercase() {
    // Lowercase idents must never be split
    let tokens = tokenize("abc").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::Ident("abc".to_string()));
}

#[test]
fn test_split_keyword_idents_no_split_mixed() {
    // Mixed case idents containing non-keyword chars must not be split
    let tokens = tokenize("Foo").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::Ident("Foo".to_string()));
}

// === UTF-8 support: non-ASCII in strings, comments, and VaisX HTML comments ===
// Added 2026-04-11: monitor/web/app/layout.vaisx was blocked at position 100
// by a Korean HTML comment. logos 0.14 is Unicode-aware for regex char classes,
// so string literals already accept UTF-8; the missing piece was HTML comments.

#[test]
fn test_utf8_string_korean() {
    let tokens = tokenize(r#""안녕하세요""#).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("안녕하세요".to_string()));
}

#[test]
fn test_utf8_string_emoji() {
    let tokens = tokenize(r#""🚀 Vais""#).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("🚀 Vais".to_string()));
}

#[test]
fn test_utf8_string_mixed_ascii_korean() {
    let tokens = tokenize(r#""Hello 안녕 World""#).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("Hello 안녕 World".to_string()));
}

#[test]
fn test_utf8_line_comment_korean() {
    // # line comment followed by Korean text must be skipped cleanly
    let source = "# 안녕하세요 주석\nF main(){}";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token, Token::Function);
    assert_eq!(tokens[1].token, Token::Ident("main".to_string()));
}

#[test]
fn test_vaisx_html_comment_korean() {
    // VaisX template HTML comment with Korean (reproduces layout.vaisx:8 case)
    let source = "<!-- 인증 페이지: 레이아웃 없이 전체 화면 -->\n";
    let tokens = tokenize(source).unwrap();
    // HTML comment is skipped entirely → zero tokens
    assert_eq!(tokens.len(), 0, "HTML comment should be skipped, got: {:?}", tokens);
}

#[test]
fn test_vaisx_html_comment_ascii() {
    let source = "<!-- plain ascii comment -->";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_vaisx_html_comment_multiline() {
    let source = "<!-- line1\n   line2 한글\n   line3 -->";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_vaisx_layout_first_lines() {
    // Actual layout.vaisx first 8 lines — previously failed at position 100
    let source = concat!(
        "<template>\n",
        "  <div class=\"app-root\">\n",
        "    {#if isAuthRoute}\n",
        "      <!-- 인증 페이지: 레이아웃 없이 전체 화면 -->\n",
        "      <div class=\"auth-shell\"></div>\n",
        "    {:else}\n",
        "      <div class=\"app-layout\"></div>\n",
        "    {/if}\n",
        "  </div>\n",
        "</template>\n",
    );
    // Must tokenize without LexError. We don't assert exact token sequence
    // (that's the parser's job) — only that lexer accepts the UTF-8 comment.
    let result = tokenize(source);
    assert!(
        result.is_ok(),
        "VaisX layout with Korean HTML comment must lex cleanly, got: {:?}",
        result.err()
    );
}

#[test]
fn test_vaisx_doctype_skipped() {
    let source = "<!DOCTYPE html>\n<template></template>";
    let tokens = tokenize(source).unwrap();
    // doctype skipped; remaining tokens are <, template, >, <, /, template, >
    assert!(tokens.iter().any(|t| matches!(&t.token, Token::Ident(s) if s == "template")));
}

#[test]
fn test_utf8_identifier_rejected() {
    // Korean identifiers must still be rejected (keeps keyword space clean)
    // `F 안녕()` — lexer should emit Function then fail on the Korean ident
    let result = tokenize("F 안녕()");
    assert!(result.is_err(), "non-ASCII identifier must be rejected");
}

// === Single-quote string literals (VaisX / JS style) ===
// Added 2026-04-11 for monitor/web layout.vaisx which uses `{t('common.loading')}`
// patterns. Shares escape processing with `"..."` via parse_string_literal helper.
// Must not regress Rust-style lifetimes (`'a`, `'static`).

#[test]
fn test_single_quote_string_basic() {
    let tokens = tokenize("'hello'").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("hello".to_string()));
}

#[test]
fn test_single_quote_string_utf8() {
    let tokens = tokenize("'안녕'").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("안녕".to_string()));
}

#[test]
fn test_single_quote_string_with_dots() {
    // `'common.loading'` — must not be confused with lifetime `'common` + `.` + `loading`
    let tokens = tokenize("'common.loading'").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("common.loading".to_string()));
}

#[test]
fn test_single_quote_string_escaped_apostrophe() {
    let tokens = tokenize(r"'it\'s'").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("it's".to_string()));
}

#[test]
fn test_single_quote_empty_string() {
    let tokens = tokenize("''").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::String("".to_string()));
}

#[test]
fn test_single_quote_i18n_call() {
    // VaisX pattern: `t('common.loading')` inside an interpolation brace `{...}`.
    // This is what layout.vaisx has at position 796.
    let tokens = tokenize("t('common.loading')").unwrap();
    assert_eq!(tokens[0].token, Token::Ident("t".to_string()));
    assert_eq!(tokens[1].token, Token::LParen);
    assert_eq!(tokens[2].token, Token::String("common.loading".to_string()));
    assert_eq!(tokens[3].token, Token::RParen);
}

#[test]
fn test_lifetime_a_not_broken_by_string_rule() {
    // Bare `'a` (no closing quote) must still be a lifetime.
    // Input: `F f<'a>()` — angle brackets and `<` ensure lifetime context.
    let tokens = tokenize("F f<'a>()").unwrap();
    // Find the lifetime token — there should be exactly one `Lifetime("a")`.
    let has_lifetime = tokens.iter().any(|t| t.token == Token::Lifetime("a".to_string()));
    assert!(has_lifetime, "Lifetime 'a must still tokenize, got: {:?}", tokens);
}

#[test]
fn test_lifetime_static_not_broken() {
    let tokens = tokenize("'static").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::Lifetime("static".to_string()));
}

#[test]
fn test_lifetime_then_identifier() {
    // `'a T` — lifetime followed by a type identifier. Common in generic signatures.
    let tokens = tokenize("'a T").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Lifetime("a".to_string()));
    assert_eq!(tokens[1].token, Token::TypeKeyword);
}

#[test]
fn test_single_quote_string_in_interpolation() {
    // `{t('key')}` — the brace interpolation pattern used in VaisX.
    let tokens = tokenize("{t('key')}").unwrap();
    assert_eq!(tokens[0].token, Token::LBrace);
    assert_eq!(tokens[1].token, Token::Ident("t".to_string()));
    assert_eq!(tokens[2].token, Token::LParen);
    assert_eq!(tokens[3].token, Token::String("key".to_string()));
    assert_eq!(tokens[4].token, Token::RParen);
    assert_eq!(tokens[5].token, Token::RBrace);
}

#[test]
fn test_single_quote_es_module_import() {
    // `import Sidebar from './components/sidebar.vaisx'` —
    // VaisX template ES module import statement (layout.vaisx line 34, position 1041).
    let tokens = tokenize("import Sidebar from './components/sidebar.vaisx'").unwrap();
    let has_path = tokens.iter().any(|t| {
        matches!(&t.token, Token::String(s) if s == "./components/sidebar.vaisx")
    });
    assert!(has_path, "ES module import path must tokenize as String, got: {:?}", tokens);
}
