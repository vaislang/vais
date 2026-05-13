//! Error message suggestion utilities for code generation diagnostics
//!
//! Contains edit distance calculation, symbol suggestion, and type conversion hints.

// Error Message Suggestion Utilities
// ============================================================================

/// Calculate the Levenshtein edit distance between two strings
pub(crate) fn edit_distance(a: &str, b: &str) -> usize {
    let len_a = a.len();
    let len_b = b.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    // Create a matrix for dynamic programming
    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    // Initialize first column and row
    for (i, row) in matrix.iter_mut().enumerate().take(len_a + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len_b + 1) {
        *cell = j;
    }

    // Fill the matrix
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    matrix[i + 1][j] + 1, // insertion
                ),
                matrix[i][j] + cost, // substitution
            );
        }
    }

    matrix[len_a][len_b]
}

/// Find similar symbols from a list of candidates and return suggestions
///
/// Returns up to `max_suggestions` candidates sorted by edit distance.
/// Only includes candidates within a reasonable edit distance threshold.
pub(crate) fn suggest_similar(
    name: &str,
    candidates: &[&str],
    max_suggestions: usize,
) -> Vec<String> {
    // Calculate max distance based on name length
    // Short names (1-3 chars): max 1 edit, medium (4-7): max 2, long: max 3
    let max_distance = if name.len() <= 3 {
        1
    } else if name.len() <= 7 {
        2
    } else {
        3
    };

    let mut suggestions: Vec<(String, usize)> = candidates
        .iter()
        .map(|&candidate| {
            // Check for case-insensitive match first
            if candidate.eq_ignore_ascii_case(name) {
                (candidate.to_string(), 0)
            } else {
                let distance = edit_distance(name, candidate);
                (candidate.to_string(), distance)
            }
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .collect();

    // Sort by distance, then alphabetically
    suggestions.sort_by(|a, b| match a.1.cmp(&b.1) {
        std::cmp::Ordering::Equal => a.0.cmp(&b.0),
        other => other,
    });

    // Take top suggestions
    suggestions
        .into_iter()
        .take(max_suggestions)
        .map(|(name, _)| name)
        .collect()
}

/// Format a "did you mean" suggestion string
pub(crate) fn format_did_you_mean(suggestions: &[String]) -> String {
    match suggestions.len() {
        0 => String::new(),
        1 => format!(". Did you mean `{}`?", suggestions[0]),
        2 => format!(
            ". Did you mean `{}` or `{}`?",
            suggestions[0], suggestions[1]
        ),
        _ => {
            let first_two = suggestions[0..2].join("`, `");
            format!(". Did you mean `{}`, or `{}`?", first_two, suggestions[2])
        }
    }
}

/// Suggest type conversion hints based on common type mismatches
#[cfg(test)]
pub(crate) fn suggest_type_conversion(expected: &str, found: &str) -> String {
    // Common numeric conversions
    if expected.starts_with('i') && found.starts_with('f') {
        return format!(". Consider using `as {}` for explicit conversion", expected);
    }
    if expected.starts_with('f') && found.starts_with('i') {
        return format!(". Consider using `as {}` for explicit conversion", expected);
    }

    // Integer size conversions
    if expected.starts_with('i') && found.starts_with('i') && expected != found {
        return format!(". Consider using `as {}` to convert", expected);
    }

    // Float size conversions
    if expected.starts_with('f') && found.starts_with('f') && expected != found {
        return format!(". Consider using `as {}` to convert", expected);
    }

    // String conversions
    if expected == "String" && found == "&str" {
        return ". Consider using `.to_string()` or `.into()`".to_string();
    }
    if expected == "&str" && found == "String" {
        return ". Consider using `.as_str()` or `&`".to_string();
    }

    // Bool to integer
    if expected.starts_with('i') && found == "bool" {
        return format!(
            ". Consider using `as {}` to convert boolean to integer",
            expected
        );
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== edit_distance ==========

    #[test]
    fn test_edit_distance_identical() {
        assert_eq!(edit_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_edit_distance_empty() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", ""), 3);
        assert_eq!(edit_distance("", "xyz"), 3);
    }

    #[test]
    fn test_edit_distance_one_char_diff() {
        assert_eq!(edit_distance("cat", "bat"), 1);
        assert_eq!(edit_distance("cat", "cats"), 1);
        assert_eq!(edit_distance("cats", "cat"), 1);
    }

    #[test]
    fn test_edit_distance_completely_different() {
        assert_eq!(edit_distance("abc", "xyz"), 3);
    }

    #[test]
    fn test_edit_distance_symmetric() {
        assert_eq!(
            edit_distance("kitten", "sitting"),
            edit_distance("sitting", "kitten")
        );
    }

    #[test]
    fn test_edit_distance_transposition() {
        assert_eq!(edit_distance("ab", "ba"), 2); // swap = delete + insert
    }

    // ========== suggest_similar ==========

    #[test]
    fn test_suggest_similar_exact() {
        let candidates = &["foo", "bar", "baz"];
        let result = suggest_similar("foo", candidates, 3);
        assert_eq!(result, vec!["foo"]);
    }

    #[test]
    fn test_suggest_similar_close() {
        let candidates = &["print_i64", "print_f64", "puts"];
        let result = suggest_similar("print_i65", candidates, 3);
        assert!(result.contains(&"print_i64".to_string()));
    }

    #[test]
    fn test_suggest_similar_none() {
        let candidates = &["foo", "bar"];
        let result = suggest_similar("completely_different_name", candidates, 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_suggest_similar_case_insensitive() {
        let candidates = &["Print", "puts"];
        let result = suggest_similar("print", candidates, 3);
        assert!(result.contains(&"Print".to_string()));
    }

    #[test]
    fn test_suggest_similar_max_suggestions() {
        let candidates = &["aa", "ab", "ac", "ad"];
        let result = suggest_similar("aa", candidates, 2);
        assert!(result.len() <= 2);
    }

    #[test]
    fn test_suggest_similar_empty_candidates() {
        let candidates: &[&str] = &[];
        let result = suggest_similar("foo", candidates, 3);
        assert!(result.is_empty());
    }

    // ========== format_did_you_mean ==========

    #[test]
    fn test_format_did_you_mean_empty() {
        assert_eq!(format_did_you_mean(&[]), "");
    }

    #[test]
    fn test_format_did_you_mean_one() {
        let suggestions = vec!["foo".to_string()];
        assert_eq!(format_did_you_mean(&suggestions), ". Did you mean `foo`?");
    }

    #[test]
    fn test_format_did_you_mean_two() {
        let suggestions = vec!["foo".to_string(), "bar".to_string()];
        let result = format_did_you_mean(&suggestions);
        assert!(result.contains("foo"));
        assert!(result.contains("bar"));
        assert!(result.contains("or"));
    }

    #[test]
    fn test_format_did_you_mean_three() {
        let suggestions = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = format_did_you_mean(&suggestions);
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains("c"));
    }

    // ========== suggest_type_conversion ==========

    #[test]
    fn test_suggest_type_conversion_float_to_int() {
        let result = suggest_type_conversion("i64", "f64");
        assert!(result.contains("as i64"));
    }

    #[test]
    fn test_suggest_type_conversion_int_to_float() {
        let result = suggest_type_conversion("f64", "i64");
        assert!(result.contains("as f64"));
    }

    #[test]
    fn test_suggest_type_conversion_int_sizes() {
        let result = suggest_type_conversion("i64", "i32");
        assert!(result.contains("as i64"));
    }

    #[test]
    fn test_suggest_type_conversion_float_sizes() {
        let result = suggest_type_conversion("f64", "f32");
        assert!(result.contains("as f64"));
    }

    #[test]
    fn test_suggest_type_conversion_string_to_str() {
        let result = suggest_type_conversion("&str", "String");
        assert!(result.contains("as_str"));
    }

    #[test]
    fn test_suggest_type_conversion_str_to_string() {
        let result = suggest_type_conversion("String", "&str");
        assert!(result.contains("to_string"));
    }

    #[test]
    fn test_suggest_type_conversion_bool_to_int() {
        let result = suggest_type_conversion("i64", "bool");
        assert!(result.contains("boolean to integer"));
    }

    #[test]
    fn test_suggest_type_conversion_same_type() {
        let result = suggest_type_conversion("i64", "i64");
        assert!(result.is_empty());
    }

    #[test]
    fn test_suggest_type_conversion_no_suggestion() {
        let result = suggest_type_conversion("Vec", "HashMap");
        assert!(result.is_empty());
    }
}
