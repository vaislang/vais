use super::*;
use super::extract::*;
use super::html::html_escape;

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
