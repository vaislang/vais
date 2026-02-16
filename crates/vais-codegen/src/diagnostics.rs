//! Error message suggestion utilities for code generation diagnostics
//!
//! Contains edit distance calculation, symbol suggestion, and type conversion hints.

// Error Message Suggestion Utilities
// ============================================================================

/// Calculate the Levenshtein edit distance between two strings
#[cfg_attr(test, allow(dead_code))]
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
#[allow(dead_code)]
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
