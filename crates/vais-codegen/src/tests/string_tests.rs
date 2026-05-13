use crate::diagnostics::{edit_distance, suggest_type_conversion};
use crate::{format_did_you_mean, suggest_similar};

#[test]
fn test_edit_distance() {
    assert_eq!(edit_distance("", ""), 0);
    assert_eq!(edit_distance("hello", "hello"), 0);
    assert_eq!(edit_distance("hello", "hallo"), 1);
    assert_eq!(edit_distance("hello", "hell"), 1);
    assert_eq!(edit_distance("hello", "helloo"), 1);
    assert_eq!(edit_distance("kitten", "sitting"), 3);
    assert_eq!(edit_distance("saturday", "sunday"), 3);
}

#[test]
fn test_suggest_similar() {
    let candidates = vec!["count", "counter", "account", "mount", "county"];

    // Exact case-insensitive match should be prioritized
    let suggestions = suggest_similar("COUNT", &candidates, 3);
    assert_eq!(suggestions[0], "count");

    // Close matches
    let suggestions = suggest_similar("countr", &candidates, 3);
    assert!(suggestions.contains(&"counter".to_string()));
    assert!(suggestions.contains(&"count".to_string()));

    // Should limit to max_suggestions
    let suggestions = suggest_similar("cont", &candidates, 2);
    assert!(suggestions.len() <= 2);

    // No matches if too far
    let suggestions = suggest_similar("xyz", &candidates, 3);
    assert!(suggestions.is_empty());
}

#[test]
fn test_format_did_you_mean() {
    assert_eq!(format_did_you_mean(&[]), "");
    assert_eq!(
        format_did_you_mean(&["foo".to_string()]),
        ". Did you mean `foo`?"
    );
    assert_eq!(
        format_did_you_mean(&["foo".to_string(), "bar".to_string()]),
        ". Did you mean `foo` or `bar`?"
    );
    assert_eq!(
        format_did_you_mean(&["foo".to_string(), "bar".to_string(), "baz".to_string()]),
        ". Did you mean `foo`, `bar`, or `baz`?"
    );
}

#[test]
fn test_suggest_type_conversion() {
    // Numeric conversions
    assert!(suggest_type_conversion("i64", "f64").contains("as i64"));
    assert!(suggest_type_conversion("f64", "i64").contains("as f64"));
    assert!(suggest_type_conversion("i32", "i64").contains("as i32"));

    // String conversions
    assert!(suggest_type_conversion("String", "&str").contains(".to_string()"));
    assert!(suggest_type_conversion("&str", "String").contains(".as_str()"));

    // Bool to int
    assert!(suggest_type_conversion("i64", "bool").contains("as i64"));

    // No suggestion for unrelated types
    assert_eq!(suggest_type_conversion("Vec", "HashMap"), "");
}
