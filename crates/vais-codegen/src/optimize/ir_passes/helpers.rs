//! Helper utilities for IR passes.

/// Extract function name from define statement
pub(super) fn extract_function_name(line: &str) -> Option<String> {
    // Pattern: define TYPE @function_name(...)
    if let Some(at_pos) = line.find('@') {
        let after_at = &line[at_pos + 1..];
        if let Some(paren_pos) = after_at.find('(') {
            return Some(after_at[..paren_pos].to_string());
        }
    }
    None
}
