use super::extract::*;
use super::html::html_escape;
use super::*;

use vais_parser::parse;

#[test]
fn test_extract_doc_comments() {
    let source = r#"
/// This is a function
/// that adds two numbers
F add(a:i64,b:i64)->i64=a+b
"#;
    let lines: Vec<&str> = source.lines().collect();
    let docs = extract_doc_comments(&lines, 50); // Position of F
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0], "This is a function");
    assert_eq!(docs[1], "that adds two numbers");
}

#[test]
fn test_parse_examples() {
    let docs = vec![
        "This is a test".to_string(),
        "```vais".to_string(),
        "F test()->i64=42".to_string(),
        "```".to_string(),
    ];
    let examples = parse_examples(&docs);
    assert_eq!(examples.len(), 1);
    assert!(examples[0].contains("F test()->i64=42"));
}

#[test]
fn test_extract_function_doc() {
    let source = "F add(a:i64,b:i64)->i64=a+b";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Function(f) = &ast.items[0].node {
        let doc = extract_function_doc(f, vec!["Adds two numbers".to_string()]);
        assert_eq!(doc.name, "add");
        assert_eq!(doc.params.len(), 2);
        assert_eq!(doc.docs.len(), 1);
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_extract_struct_doc() {
    let source = "S Point{x:i64,y:i64}";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Struct(s) = &ast.items[0].node {
        let doc = extract_struct_doc(s, vec!["A 2D point".to_string()]);
        assert_eq!(doc.name, "Point");
        assert_eq!(doc.docs.len(), 1);
    } else {
        panic!("Expected struct item");
    }
}

#[test]
fn test_html_escape() {
    assert_eq!(html_escape("<test>"), "&lt;test&gt;");
    assert_eq!(html_escape("a & b"), "a &amp; b");
    assert_eq!(html_escape("\"quote\""), "&quot;quote&quot;");
}

// ── Additional doc_gen tests ──

#[test]
fn test_html_escape_single_quote() {
    assert_eq!(html_escape("it's"), "it&#39;s");
}

#[test]
fn test_html_escape_empty() {
    assert_eq!(html_escape(""), "");
}

#[test]
fn test_html_escape_no_special_chars() {
    assert_eq!(html_escape("hello world"), "hello world");
}

#[test]
fn test_html_escape_multiple_special() {
    assert_eq!(
        html_escape("<a href=\"test\">foo & bar</a>"),
        "&lt;a href=&quot;test&quot;&gt;foo &amp; bar&lt;/a&gt;"
    );
}

#[test]
fn test_parse_examples_empty() {
    let docs: Vec<String> = vec![];
    let examples = parse_examples(&docs);
    assert!(examples.is_empty());
}

#[test]
fn test_parse_examples_no_code_blocks() {
    let docs = vec![
        "This is documentation".to_string(),
        "with multiple lines".to_string(),
    ];
    let examples = parse_examples(&docs);
    assert!(examples.is_empty());
}

#[test]
fn test_parse_examples_multiple_blocks() {
    let docs = vec![
        "Example 1:".to_string(),
        "```vais".to_string(),
        "F foo()->i64=1".to_string(),
        "```".to_string(),
        "Example 2:".to_string(),
        "```vais".to_string(),
        "F bar()->i64=2".to_string(),
        "```".to_string(),
    ];
    let examples = parse_examples(&docs);
    assert_eq!(examples.len(), 2);
}

#[test]
fn test_parse_examples_generic_code_fence() {
    let docs = vec![
        "```".to_string(),
        "some code".to_string(),
        "```".to_string(),
    ];
    let examples = parse_examples(&docs);
    assert_eq!(examples.len(), 1);
    assert!(examples[0].contains("some code"));
}

#[test]
fn test_extract_doc_comments_no_docs() {
    let source = "F main()->i64=0";
    let lines: Vec<&str> = source.lines().collect();
    let docs = extract_doc_comments(&lines, 0);
    assert!(docs.is_empty());
}

#[test]
fn test_extract_doc_comments_hash_style() {
    let source = "# A vais-style comment\nF main()->i64=0";
    let lines: Vec<&str> = source.lines().collect();
    let docs = extract_doc_comments(&lines, 24); // Position of F
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0], "A vais-style comment");
}

#[test]
fn test_extract_doc_comments_skips_separator_lines() {
    let source = "# Real comment\n# ============\nF main()->i64=0";
    let lines: Vec<&str> = source.lines().collect();
    let docs = extract_doc_comments(&lines, 30);
    // Separator lines (all = or -) should be skipped
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0], "Real comment");
}

#[test]
fn test_extract_function_doc_no_docs() {
    let source = "F greet()->i64=42";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Function(f) = &ast.items[0].node {
        let doc = extract_function_doc(f, vec![]);
        assert_eq!(doc.name, "greet");
        assert!(doc.docs.is_empty());
        assert!(doc.examples.is_empty());
        assert_eq!(doc.kind, DocKind::Function);
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_extract_function_doc_with_return_type() {
    let source = "F add(a:i64,b:i64)->i64=a+b";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Function(f) = &ast.items[0].node {
        let doc = extract_function_doc(f, vec![]);
        assert!(doc.returns.is_some());
        assert!(doc.signature.contains("F add"));
        assert!(doc.signature.contains("i64"));
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_extract_function_doc_pub() {
    let source = "P F greet()->i64=42";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Function(f) = &ast.items[0].node {
        let doc = extract_function_doc(f, vec![]);
        assert_eq!(doc.visibility, Visibility::Public);
        assert!(doc.signature.starts_with("P "));
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_extract_function_doc_private() {
    let source = "F helper()->i64=1";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Function(f) = &ast.items[0].node {
        let doc = extract_function_doc(f, vec![]);
        assert_eq!(doc.visibility, Visibility::Private);
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_extract_enum_doc() {
    let source = "E Color { Red, Green, Blue }";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Enum(e) = &ast.items[0].node {
        let doc = extract_enum_doc(e, vec!["Color enum".to_string()]);
        assert_eq!(doc.name, "Color");
        assert_eq!(doc.kind, DocKind::Enum);
        assert!(doc.signature.contains("E Color"));
        assert!(doc.signature.contains("Red"));
    } else {
        panic!("Expected enum item");
    }
}

#[test]
fn test_extract_enum_doc_pub() {
    let source = "P E Status { Ok, Error }";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Enum(e) = &ast.items[0].node {
        let doc = extract_enum_doc(e, vec![]);
        assert_eq!(doc.visibility, Visibility::Public);
        assert!(doc.signature.starts_with("P "));
    } else {
        panic!("Expected enum item");
    }
}

#[test]
fn test_extract_documentation_full() {
    let source = r#"/// A simple function
F add(a:i64,b:i64)->i64=a+b

S Point{x:i64,y:i64}

E Color{Red,Green,Blue}
"#;
    let ast = parse(source).unwrap();
    let doc = extract_documentation(std::path::Path::new("test.vais"), &ast, source);

    assert_eq!(doc.name, "test");
    assert_eq!(doc.items.len(), 3);
}

#[test]
fn test_extract_documentation_empty_source() {
    let source = "";
    let ast = parse(source).unwrap();
    let doc = extract_documentation(std::path::Path::new("empty.vais"), &ast, source);

    assert_eq!(doc.name, "empty");
    assert!(doc.items.is_empty());
}

#[test]
fn test_doc_kind_equality() {
    assert_eq!(DocKind::Function, DocKind::Function);
    assert_eq!(DocKind::Struct, DocKind::Struct);
    assert_eq!(DocKind::Enum, DocKind::Enum);
    assert_eq!(DocKind::Trait, DocKind::Trait);
    assert_eq!(DocKind::Constant, DocKind::Constant);
    assert_eq!(DocKind::ExternFunction, DocKind::ExternFunction);
    assert_ne!(DocKind::Function, DocKind::Struct);
}

#[test]
fn test_visibility_equality() {
    assert_eq!(Visibility::Public, Visibility::Public);
    assert_eq!(Visibility::Private, Visibility::Private);
    assert_ne!(Visibility::Public, Visibility::Private);
}

#[test]
fn test_extract_function_doc_with_examples() {
    let source = "F greet()->i64=42";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Function(f) = &ast.items[0].node {
        let doc = extract_function_doc(
            f,
            vec![
                "Greets the user".to_string(),
                "```vais".to_string(),
                "greet()".to_string(),
                "```".to_string(),
            ],
        );
        assert_eq!(doc.examples.len(), 1);
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_extract_struct_doc_pub() {
    let source = "P S Config{debug:bool}";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Struct(s) = &ast.items[0].node {
        let doc = extract_struct_doc(s, vec![]);
        assert_eq!(doc.visibility, Visibility::Public);
        assert!(doc.signature.starts_with("P "));
    } else {
        panic!("Expected struct item");
    }
}

#[test]
fn test_extract_struct_doc_signature_contains_fields() {
    let source = "S Point{x:i64,y:i64}";
    let ast = parse(source).unwrap();

    if let vais_ast::Item::Struct(s) = &ast.items[0].node {
        let doc = extract_struct_doc(s, vec![]);
        assert!(doc.signature.contains("x: i64"));
        assert!(doc.signature.contains("y: i64"));
    } else {
        panic!("Expected struct item");
    }
}
