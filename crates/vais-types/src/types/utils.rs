//! Utility functions for type system

/// Calculate Levenshtein distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, val) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *val = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Find the most similar name from a list of candidates
/// Returns None if no name is similar enough (distance > threshold)
pub fn find_similar_name<'a>(
    name: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let name_lower = name.to_lowercase();
    let max_distance = std::cmp::max(2, name.len() / 3); // Allow ~1/3 of chars to be different

    let mut best_match: Option<(String, usize)> = None;

    for candidate in candidates {
        let candidate_lower = candidate.to_lowercase();
        let distance = levenshtein_distance(&name_lower, &candidate_lower);

        if distance <= max_distance {
            if let Some((_, best_dist)) = &best_match {
                if distance < *best_dist {
                    best_match = Some((candidate.to_string(), distance));
                }
            } else {
                best_match = Some((candidate.to_string(), distance));
            }
        }
    }

    best_match.map(|(name, _)| name)
}
