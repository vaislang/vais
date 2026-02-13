//! Core type definitions for the Vais type system
//!
//! This module contains the fundamental type definitions used throughout
//! the type checker, including resolved types, type errors, and type signatures.

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;

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

/// Type checking error
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch {
        expected: String,
        found: String,
        span: Option<Span>,
    },

    #[error("Undefined variable: {name}")]
    UndefinedVar {
        name: String,
        span: Option<Span>,
        suggestion: Option<String>,
    },

    #[error("Undefined type: {name}")]
    UndefinedType {
        name: String,
        span: Option<Span>,
        suggestion: Option<String>,
    },

    #[error("Undefined function: {name}")]
    UndefinedFunction {
        name: String,
        span: Option<Span>,
        suggestion: Option<String>,
    },

    #[error("Cannot call non-function type: {0}")]
    NotCallable(String, Option<Span>),

    #[error("Wrong number of arguments: expected {expected}, got {got}")]
    ArgCount {
        expected: usize,
        got: usize,
        span: Option<Span>,
    },

    #[error("Cannot infer type")]
    CannotInfer,

    #[error("Duplicate definition: {0}")]
    Duplicate(String, Option<Span>),

    #[error("Cannot assign to immutable variable: {0}")]
    ImmutableAssign(String, Option<Span>),

    #[error("Non-exhaustive match: missing patterns {0}")]
    NonExhaustiveMatch(String, Option<Span>),

    #[error("Unreachable pattern at arm {0}")]
    UnreachablePattern(usize, Option<Span>),

    #[error("Effect mismatch: function declared as {declared} but has effect {actual}")]
    EffectMismatch {
        declared: String,
        actual: String,
        span: Option<Span>,
    },

    #[error("Pure function cannot call impure function: {callee} has effects {effects}")]
    PurityViolation {
        callee: String,
        effects: String,
        span: Option<Span>,
    },

    #[error("Linear type violation: variable '{var_name}' must be used exactly once, but was used {actual_uses} times")]
    LinearTypeViolation {
        var_name: String,
        expected_uses: usize,
        actual_uses: usize,
        defined_at: Option<Span>,
    },

    #[error("Affine type violation: variable '{var_name}' can be used at most once, but was used {actual_uses} times")]
    AffineTypeViolation {
        var_name: String,
        actual_uses: usize,
        defined_at: Option<Span>,
    },

    #[error("Move after use: variable '{var_name}' was already moved")]
    MoveAfterUse {
        var_name: String,
        first_use_at: Option<Span>,
        move_at: Option<Span>,
    },

    #[error("Dependent type predicate must be a boolean expression, found {found}")]
    DependentPredicateNotBool { found: String, span: Option<Span> },

    #[error("Dependent type refinement violation: predicate '{predicate}' may not hold")]
    RefinementViolation {
        predicate: String,
        span: Option<Span>,
    },

    #[error("Lifetime inference failed for function '{function_name}': cannot determine output lifetime with {input_count} input lifetimes (use explicit lifetime annotations)")]
    LifetimeElisionFailed {
        function_name: String,
        input_count: usize,
        span: Option<Span>,
    },

    #[error("Lifetime '{lifetime_name}' does not live long enough: cannot outlive 'static")]
    LifetimeOutlivesStatic {
        lifetime_name: String,
        span: Option<Span>,
    },

    #[error(
        "Reference lifetime {reference_lifetime} outlives referent lifetime {referent_lifetime}"
    )]
    LifetimeTooShort {
        reference_lifetime: String,
        referent_lifetime: String,
        span: Option<Span>,
    },

    #[error("Use of moved value: variable '{var_name}' was moved")]
    UseAfterMove {
        var_name: String,
        moved_at: Option<Span>,
        use_at: Option<Span>,
    },

    #[error(
        "Use of partially moved value: variable '{var_name}' has moved fields: {moved_fields:?}"
    )]
    UseAfterPartialMove {
        var_name: String,
        moved_fields: Vec<String>,
        use_at: Option<Span>,
    },

    #[error("Cannot assign to '{var_name}' while it is borrowed")]
    AssignWhileBorrowed {
        var_name: String,
        borrow_at: Option<Span>,
        assign_at: Option<Span>,
        is_mut_borrow: bool,
    },

    #[error("Cannot borrow '{var_name}' after it was moved")]
    BorrowAfterMove {
        var_name: String,
        moved_at: Option<Span>,
        borrow_at: Option<Span>,
    },

    #[error("Cannot borrow '{var_name}': conflicting borrows")]
    BorrowConflict {
        var_name: String,
        existing_borrow_at: Option<Span>,
        new_borrow_at: Option<Span>,
        existing_is_mut: bool,
        new_is_mut: bool,
    },

    #[error("Cannot mutably borrow immutable variable '{var_name}'")]
    MutBorrowOfImmutable {
        var_name: String,
        borrow_at: Option<Span>,
    },

    #[error(
        "Dangling reference: '{ref_var}' references '{source_var}' which does not live long enough"
    )]
    DanglingReference {
        ref_var: String,
        source_var: String,
        ref_scope_depth: u32,
        source_scope_depth: u32,
        ref_at: Option<Span>,
        source_defined_at: Option<Span>,
    },

    #[error("Cannot return reference to local variable '{var_name}'")]
    ReturnLocalRef {
        var_name: String,
        return_at: Option<Span>,
        defined_at: Option<Span>,
    },

    #[error("No field `{field}` on type `{type_name}`")]
    NoSuchField {
        field: String,
        type_name: String,
        suggestion: Option<String>,
        span: Option<Span>,
    },

    #[error("Extern function `{name}` has unexpected return type: expected `{expected}`, found `{found}`")]
    ExternSignatureMismatch {
        name: String,
        expected: String,
        found: String,
        span: Option<Span>,
    },

    #[error("Cannot infer type of {kind} '{name}' in function '{context}'")]
    InferFailed {
        kind: String,
        name: String,
        context: String,
        span: Option<Span>,
        suggestion: Option<String>,
    },
}

impl TypeError {
    /// Get the span associated with this error, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            TypeError::Mismatch { span, .. } => *span,
            TypeError::UndefinedVar { span, .. } => *span,
            TypeError::UndefinedType { span, .. } => *span,
            TypeError::UndefinedFunction { span, .. } => *span,
            TypeError::NotCallable(_, span) => *span,
            TypeError::ArgCount { span, .. } => *span,
            TypeError::CannotInfer => None,
            TypeError::Duplicate(_, span) => *span,
            TypeError::ImmutableAssign(_, span) => *span,
            TypeError::NonExhaustiveMatch(_, span) => *span,
            TypeError::UnreachablePattern(_, span) => *span,
            TypeError::EffectMismatch { span, .. } => *span,
            TypeError::PurityViolation { span, .. } => *span,
            TypeError::LinearTypeViolation { defined_at, .. } => *defined_at,
            TypeError::AffineTypeViolation { defined_at, .. } => *defined_at,
            TypeError::MoveAfterUse { move_at, .. } => *move_at,
            TypeError::DependentPredicateNotBool { span, .. } => *span,
            TypeError::RefinementViolation { span, .. } => *span,
            TypeError::LifetimeElisionFailed { span, .. } => *span,
            TypeError::LifetimeOutlivesStatic { span, .. } => *span,
            TypeError::LifetimeTooShort { span, .. } => *span,
            TypeError::UseAfterMove { use_at, .. } => *use_at,
            TypeError::UseAfterPartialMove { use_at, .. } => *use_at,
            TypeError::AssignWhileBorrowed { assign_at, .. } => *assign_at,
            TypeError::BorrowAfterMove { borrow_at, .. } => *borrow_at,
            TypeError::BorrowConflict { new_borrow_at, .. } => *new_borrow_at,
            TypeError::MutBorrowOfImmutable { borrow_at, .. } => *borrow_at,
            TypeError::DanglingReference { ref_at, .. } => *ref_at,
            TypeError::ReturnLocalRef { return_at, .. } => *return_at,
            TypeError::NoSuchField { span, .. } => *span,
            TypeError::ExternSignatureMismatch { span, .. } => *span,
            TypeError::InferFailed { span, .. } => *span,
        }
    }


    /// Get secondary spans with labels for multi-location errors
    pub fn secondary_spans(&self) -> Vec<(Span, String)> {
        match self {
            TypeError::UseAfterMove { moved_at, .. } => {
                if let Some(span) = moved_at {
                    vec![(*span, "value moved here".to_string())]
                } else {
                    vec![]
                }
            }
            TypeError::BorrowConflict { existing_borrow_at, existing_is_mut, .. } => {
                if let Some(span) = existing_borrow_at {
                    let label = if *existing_is_mut {
                        "first mutable borrow occurs here"
                    } else {
                        "first borrow occurs here"
                    };
                    vec![(*span, label.to_string())]
                } else {
                    vec![]
                }
            }
            TypeError::AssignWhileBorrowed { borrow_at, .. } => {
                if let Some(span) = borrow_at {
                    vec![(*span, "borrow occurs here".to_string())]
                } else {
                    vec![]
                }
            }
            TypeError::DanglingReference { source_defined_at, .. } => {
                if let Some(span) = source_defined_at {
                    vec![(*span, "source variable defined here".to_string())]
                } else {
                    vec![]
                }
            }
            TypeError::BorrowAfterMove { moved_at, .. } => {
                if let Some(span) = moved_at {
                    vec![(*span, "value moved here".to_string())]
                } else {
                    vec![]
                }
            }
            TypeError::ReturnLocalRef { defined_at, .. } => {
                if let Some(span) = defined_at {
                    vec![(*span, "local variable defined here".to_string())]
                } else {
                    vec![]
                }
            }
            TypeError::MoveAfterUse { first_use_at, .. } => {
                if let Some(span) = first_use_at {
                    vec![(*span, "first use occurs here".to_string())]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
    /// Get the error code for this error
    pub fn error_code(&self) -> &str {
        match self {
            TypeError::Mismatch { .. } => "E001",
            TypeError::UndefinedVar { .. } => "E002",
            TypeError::UndefinedType { .. } => "E003",
            TypeError::UndefinedFunction { .. } => "E004",
            TypeError::NotCallable(..) => "E005",
            TypeError::ArgCount { .. } => "E006",
            TypeError::CannotInfer => "E007",
            TypeError::Duplicate(..) => "E008",
            TypeError::ImmutableAssign(..) => "E009",
            TypeError::NonExhaustiveMatch(..) => "E010",
            TypeError::UnreachablePattern(..) => "E011",
            TypeError::EffectMismatch { .. } => "E012",
            TypeError::PurityViolation { .. } => "E013",
            TypeError::LinearTypeViolation { .. } => "E014",
            TypeError::AffineTypeViolation { .. } => "E015",
            TypeError::MoveAfterUse { .. } => "E016",
            TypeError::DependentPredicateNotBool { .. } => "E017",
            TypeError::RefinementViolation { .. } => "E018",
            TypeError::LifetimeElisionFailed { .. } => "E019",
            TypeError::LifetimeOutlivesStatic { .. } => "E020",
            TypeError::LifetimeTooShort { .. } => "E021",
            TypeError::UseAfterMove { .. } => "E022",
            TypeError::UseAfterPartialMove { .. } => "E023",
            TypeError::AssignWhileBorrowed { .. } => "E024",
            TypeError::BorrowAfterMove { .. } => "E025",
            TypeError::BorrowConflict { .. } => "E026",
            TypeError::MutBorrowOfImmutable { .. } => "E027",
            TypeError::DanglingReference { .. } => "E028",
            TypeError::ReturnLocalRef { .. } => "E029",
            TypeError::NoSuchField { .. } => "E030",
            TypeError::ExternSignatureMismatch { .. } => "E031",
            TypeError::InferFailed { .. } => "E032",
        }
    }

    /// Get a helpful message for this error
    pub fn help(&self) -> Option<String> {
        match self {
            TypeError::Mismatch { expected, found, .. } => {
                // Try to suggest a similar type name using Levenshtein distance
                let known_types = [
                    "i8", "i16", "i32", "i64", "i128",
                    "u8", "u16", "u32", "u64", "u128",
                    "f32", "f64", "bool", "str", "()",
                ];
                let suggestion = find_similar_name(found, known_types.iter().copied())
                    .filter(|s| s == expected);
                if let Some(sug) = suggestion {
                    Some(format!("did you mean `{}`?", sug))
                } else if expected == "i64" && found == "Str" {
                    Some("consider converting the string to a number".to_string())
                } else if expected.starts_with('i') || expected.starts_with('u') || expected.starts_with('f') {
                    Some("try using a type cast or conversion function".to_string())
                } else {
                    None
                }
            }
            TypeError::UndefinedVar { name, suggestion, .. } => {
                if let Some(sug) = suggestion {
                    Some(format!("did you mean '{}'?", sug))
                } else {
                    Some(format!("variable '{}' not found in this scope", name))
                }
            }
            TypeError::UndefinedType { name, suggestion, .. } => {
                if let Some(sug) = suggestion {
                    Some(format!("did you mean '{}'?", sug))
                } else {
                    Some(format!("type '{}' not found in this scope", name))
                }
            }
            TypeError::UndefinedFunction { name, suggestion, .. } => {
                if let Some(sug) = suggestion {
                    Some(format!("did you mean '{}'?", sug))
                } else {
                    Some(format!("function '{}' not found in this scope", name))
                }
            }
            TypeError::ImmutableAssign(name, _) => {
                Some(format!("consider declaring '{}' as mutable: '{}: mut Type'", name, name))
            }
            TypeError::EffectMismatch { declared, .. } => {
                Some(format!("remove the '{}' annotation or fix the function body", declared))
            }
            TypeError::PurityViolation { callee, .. } => {
                Some(format!("wrap '{}' call in an unsafe block or remove the pure annotation", callee))
            }
            TypeError::LinearTypeViolation { var_name, expected_uses, actual_uses, .. } => {
                if *actual_uses == 0 {
                    Some(format!("linear variable '{}' must be used exactly {} time(s), consider using it or marking it as affine", var_name, expected_uses))
                } else {
                    Some(format!("linear variable '{}' was used {} time(s), consider splitting usage or copying", var_name, actual_uses))
                }
            }
            TypeError::AffineTypeViolation { var_name, .. } => {
                Some(format!("affine variable '{}' can only be used once, consider cloning if multiple uses are needed", var_name))
            }
            TypeError::MoveAfterUse { var_name, .. } => {
                Some(format!("variable '{}' was already moved, consider cloning before the first use", var_name))
            }
            TypeError::LifetimeElisionFailed { input_count, .. } => {
                if *input_count > 1 {
                    Some("add an explicit lifetime parameter to the return type, e.g., -> &'a T".to_string())
                } else {
                    Some("add explicit lifetime annotations to clarify the relationship".to_string())
                }
            }
            TypeError::LifetimeOutlivesStatic { lifetime_name, .. } => {
                Some(format!("consider using 'static instead of '{}', or restructure to avoid the 'static requirement", lifetime_name))
            }
            TypeError::LifetimeTooShort { .. } => {
                Some("the reference must not outlive the data it refers to".to_string())
            }
            TypeError::UseAfterMove { var_name, .. } => {
                Some(format!("variable '{}' was moved and can no longer be used; consider cloning it before the move", var_name))
            }
            TypeError::UseAfterPartialMove { var_name, moved_fields, .. } => {
                Some(format!("variable '{}' has partially moved fields {:?}; consider cloning before moving individual fields", var_name, moved_fields))
            }
            TypeError::AssignWhileBorrowed { var_name, is_mut_borrow, .. } => {
                if *is_mut_borrow {
                    Some(format!("cannot assign to '{}' while it is mutably borrowed; the borrow must end before assignment", var_name))
                } else {
                    Some(format!("cannot assign to '{}' while it is borrowed; the borrow must end before assignment", var_name))
                }
            }
            TypeError::BorrowAfterMove { var_name, .. } => {
                Some(format!("cannot borrow '{}' because it was already moved; consider cloning before the move", var_name))
            }
            TypeError::BorrowConflict { var_name, existing_is_mut, new_is_mut, .. } => {
                if *existing_is_mut {
                    Some(format!("cannot borrow '{}' because it is already mutably borrowed; mutable borrows are exclusive", var_name))
                } else if *new_is_mut {
                    Some(format!("cannot mutably borrow '{}' because it is already immutably borrowed", var_name))
                } else {
                    Some(format!("conflicting borrows on '{}'", var_name))
                }
            }
            TypeError::MutBorrowOfImmutable { var_name, .. } => {
                Some(format!("consider declaring '{}' as mutable: `V mut {}`", var_name, var_name))
            }
            TypeError::DanglingReference { ref_var, source_var, .. } => {
                Some(format!(
                    "reference '{}' outlives '{}'; consider:\n  \
                     1. Move the data to an outer scope so it lives long enough\n  \
                     2. Clone the data instead of borrowing\n  \
                     3. Use an owned type instead of a reference",
                    ref_var, source_var
                ))
            }
            TypeError::ReturnLocalRef { var_name, .. } => {
                Some(format!(
                    "cannot return a reference to local variable '{}' because it is dropped at the end of the function; consider:\n  \
                     1. Return an owned value instead of a reference\n  \
                     2. Take the value as a parameter with a lifetime annotation\n  \
                     3. Use 'static data or a heap-allocated type (Box, Rc)",
                    var_name
                ))
            }
            TypeError::NoSuchField { field, type_name, suggestion, .. } => {
                if let Some(sug) = suggestion {
                    Some(format!("did you mean `{}`?", sug))
                } else {
                    Some(format!("type `{}` has no field named `{}`", type_name, field))
                }
            }
            TypeError::ExternSignatureMismatch { name, expected, found, .. } => {
                Some(format!(
                    "extern function `{}` should return `{}` (pointer), found `{}`",
                    name, expected, found
                ))
            }
            TypeError::InferFailed { suggestion, .. } => {
                suggestion.clone()
            }
            TypeError::NotCallable(type_name, _) => {
                Some(format!("expression of type `{}` is not callable; only functions and closures can be called", type_name))
            }
            TypeError::ArgCount { expected, got, .. } => {
                if *expected == 0 {
                    Some(format!("this function takes no arguments but {} {} supplied", got, if *got == 1 { "was" } else { "were" }))
                } else {
                    Some(format!("this function takes {} argument{} but {} {} supplied", expected, if *expected == 1 { "" } else { "s" }, got, if *got == 1 { "was" } else { "were" }))
                }
            }
            TypeError::CannotInfer => {
                Some("add explicit type annotation to resolve ambiguity".to_string())
            }
            TypeError::Duplicate(name, _) => {
                Some(format!("the name `{}` is already defined in this scope; consider renaming one of the definitions", name))
            }
            TypeError::NonExhaustiveMatch(missing, _) => {
                Some(format!("ensure all possible values are covered; add pattern{} for {} or use `_ =>` as a wildcard", if missing.contains(',') { "s" } else { "" }, missing))
            }
            TypeError::UnreachablePattern(arm, _) => {
                Some(format!("pattern at arm {} will never be matched because previous patterns already cover all cases; consider removing it", arm))
            }
            TypeError::DependentPredicateNotBool { found, .. } => {
                Some(format!("refinement predicates must evaluate to `bool`, but this expression has type `{}`", found))
            }
            TypeError::RefinementViolation { predicate, .. } => {
                Some(format!("the value does not satisfy the refinement predicate `{}`; ensure the value meets the constraint", predicate))
            }
            _ => None,
        }
    }

    /// Get the localized title for this error
    pub fn localized_title(&self) -> String {
        let key = format!("type.{}.title", self.error_code());
        vais_i18n::get_simple(&key)
    }

    /// Get the localized message for this error
    pub fn localized_message(&self) -> String {
        let key = format!("type.{}.message", self.error_code());
        match self {
            TypeError::Mismatch {
                expected, found, ..
            } => vais_i18n::get(&key, &[("expected", expected), ("found", found)]),
            TypeError::UndefinedVar { name, .. } => vais_i18n::get(&key, &[("name", name)]),
            TypeError::UndefinedType { name, .. } => vais_i18n::get(&key, &[("name", name)]),
            TypeError::UndefinedFunction { name, .. } => vais_i18n::get(&key, &[("name", name)]),
            TypeError::NotCallable(type_name, _) => vais_i18n::get(&key, &[("type", type_name)]),
            TypeError::ArgCount { expected, got, .. } => vais_i18n::get(
                &key,
                &[
                    ("expected", &expected.to_string()),
                    ("got", &got.to_string()),
                ],
            ),
            TypeError::CannotInfer => vais_i18n::get_simple(&key),
            TypeError::Duplicate(name, _) => vais_i18n::get(&key, &[("name", name)]),
            TypeError::ImmutableAssign(name, _) => vais_i18n::get(&key, &[("name", name)]),
            TypeError::NonExhaustiveMatch(patterns, _) => {
                vais_i18n::get(&key, &[("patterns", patterns)])
            }
            TypeError::UnreachablePattern(arm, _) => {
                vais_i18n::get(&key, &[("arm", &arm.to_string())])
            }
            TypeError::EffectMismatch {
                declared, actual, ..
            } => vais_i18n::get(&key, &[("declared", declared), ("actual", actual)]),
            TypeError::PurityViolation {
                callee, effects, ..
            } => vais_i18n::get(&key, &[("callee", callee), ("effects", effects)]),
            TypeError::LinearTypeViolation {
                var_name,
                expected_uses,
                actual_uses,
                ..
            } => vais_i18n::get(
                &key,
                &[
                    ("var_name", var_name),
                    ("expected_uses", &expected_uses.to_string()),
                    ("actual_uses", &actual_uses.to_string()),
                ],
            ),
            TypeError::AffineTypeViolation {
                var_name,
                actual_uses,
                ..
            } => vais_i18n::get(
                &key,
                &[
                    ("var_name", var_name),
                    ("actual_uses", &actual_uses.to_string()),
                ],
            ),
            TypeError::MoveAfterUse { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::DependentPredicateNotBool { found, .. } => {
                vais_i18n::get(&key, &[("found", found)])
            }
            TypeError::RefinementViolation { predicate, .. } => {
                vais_i18n::get(&key, &[("predicate", predicate)])
            }
            TypeError::LifetimeElisionFailed {
                function_name,
                input_count,
                ..
            } => vais_i18n::get(
                &key,
                &[
                    ("function_name", function_name),
                    ("input_count", &input_count.to_string()),
                ],
            ),
            TypeError::LifetimeOutlivesStatic { lifetime_name, .. } => {
                vais_i18n::get(&key, &[("lifetime_name", lifetime_name)])
            }
            TypeError::LifetimeTooShort {
                reference_lifetime,
                referent_lifetime,
                ..
            } => vais_i18n::get(
                &key,
                &[
                    ("reference_lifetime", reference_lifetime),
                    ("referent_lifetime", referent_lifetime),
                ],
            ),
            TypeError::UseAfterMove { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::UseAfterPartialMove { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::AssignWhileBorrowed { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::BorrowAfterMove { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::BorrowConflict { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::MutBorrowOfImmutable { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::DanglingReference {
                ref_var,
                source_var,
                ..
            } => vais_i18n::get(&key, &[("ref_var", ref_var), ("source_var", source_var)]),
            TypeError::ReturnLocalRef { var_name, .. } => {
                vais_i18n::get(&key, &[("var_name", var_name)])
            }
            TypeError::NoSuchField {
                field, type_name, ..
            } => vais_i18n::get(&key, &[("field", field), ("type_name", type_name)]),
            TypeError::ExternSignatureMismatch {
                name,
                expected,
                found,
                ..
            } => vais_i18n::get(
                &key,
                &[("name", name), ("expected", expected), ("found", found)],
            ),
            TypeError::InferFailed {
                kind,
                name,
                context,
                ..
            } => vais_i18n::get(
                &key,
                &[("kind", kind), ("name", name), ("context", context)],
            ),
        }
    }

    /// Get the localized help message for this error
    pub fn localized_help(&self) -> Option<String> {
        let key = format!("type.{}.help", self.error_code());
        if vais_i18n::has_key(&key) {
            Some(match self {
                TypeError::UndefinedVar { name, .. } => vais_i18n::get(&key, &[("name", name)]),
                TypeError::UndefinedFunction { name, .. } => {
                    vais_i18n::get(&key, &[("name", name)])
                }
                TypeError::ImmutableAssign(name, _) => vais_i18n::get(&key, &[("name", name)]),
                _ => vais_i18n::get_simple(&key),
            })
        } else {
            None
        }
    }
}

/// Type checking result
pub type TypeResult<T> = Result<T, TypeError>;

/// Resolved const value for const generics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedConst {
    /// Concrete integer value
    Value(i64),
    /// Unresolved const parameter
    Param(String),
    /// Binary operation (for type display/error messages)
    BinOp {
        op: ConstBinOp,
        left: Box<ResolvedConst>,
        right: Box<ResolvedConst>,
    },
}

impl ResolvedConst {
    /// Try to evaluate to a concrete value
    pub fn try_evaluate(&self) -> Option<i64> {
        match self {
            ResolvedConst::Value(n) => Some(*n),
            ResolvedConst::Param(_) => None,
            ResolvedConst::BinOp { op, left, right } => {
                let l = left.try_evaluate()?;
                let r = right.try_evaluate()?;
                Some(match op {
                    ConstBinOp::Add => l.checked_add(r)?,
                    ConstBinOp::Sub => l.checked_sub(r)?,
                    ConstBinOp::Mul => l.checked_mul(r)?,
                    ConstBinOp::Div => {
                        if r == 0 {
                            return None;
                        }
                        l.checked_div(r)?
                    }
                })
            }
        }
    }
}

impl std::fmt::Display for ResolvedConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedConst::Value(n) => write!(f, "{}", n),
            ResolvedConst::Param(name) => write!(f, "{}", name),
            ResolvedConst::BinOp { op, left, right } => write!(f, "({} {} {})", left, op, right),
        }
    }
}

/// Const binary operation for const expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstBinOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl std::fmt::Display for ConstBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstBinOp::Add => write!(f, "+"),
            ConstBinOp::Sub => write!(f, "-"),
            ConstBinOp::Mul => write!(f, "*"),
            ConstBinOp::Div => write!(f, "/"),
        }
    }
}

/// Resolved type in the type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedType {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Str,
    Unit,

    // Compound types
    Array(Box<ResolvedType>),
    /// Const-sized array: `[T; N]` where N is a const expression
    ConstArray {
        element: Box<ResolvedType>,
        size: ResolvedConst,
    },
    Map(Box<ResolvedType>, Box<ResolvedType>),
    Tuple(Vec<ResolvedType>),
    Optional(Box<ResolvedType>),
    Result(Box<ResolvedType>, Box<ResolvedType>),
    Pointer(Box<ResolvedType>),
    Ref(Box<ResolvedType>),
    RefMut(Box<ResolvedType>),
    /// Immutable slice: `&[T]` — fat pointer (ptr, len)
    Slice(Box<ResolvedType>),
    /// Mutable slice: `&mut [T]` — fat pointer (ptr, len)
    SliceMut(Box<ResolvedType>),
    Range(Box<ResolvedType>),
    Future(Box<ResolvedType>),

    // Function type (with optional effect annotation)
    Fn {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
        /// Effect set for this function type (None = infer)
        effects: Option<Box<EffectSet>>,
    },

    // Function pointer type (for C FFI callbacks)
    FnPtr {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
        is_vararg: bool,
        /// Effect set for this function pointer (None = total effects)
        effects: Option<Box<EffectSet>>,
    },

    // Named type (struct/enum)
    Named {
        name: String,
        generics: Vec<ResolvedType>,
    },

    // Type variable for inference
    Var(usize),

    // Generic type parameter (e.g., T in F foo<T>)
    Generic(String),

    // Const generic parameter (e.g., N in F foo<const N: u64>)
    ConstGeneric(String),

    // Unknown/Error type
    Unknown,

    // Never type - represents a type that never returns (e.g., return, break, continue)
    // This type unifies with any other type
    Never,

    // SIMD vector type: <lanes x element_type>
    // e.g., Vector { element: F32, lanes: 4 } -> <4 x float>
    Vector {
        element: Box<ResolvedType>,
        lanes: u32,
    },

    /// Dynamic trait object: `dyn Trait` or `dyn Trait<T>`
    /// Stored as a fat pointer: (vtable*, data*)
    /// Used for runtime polymorphism via vtable-based dispatch.
    DynTrait {
        trait_name: String,
        generics: Vec<ResolvedType>,
    },

    /// Associated type: `<T as Trait>::Item` or unresolved `Self::Item`
    /// GAT support: `<T as Trait>::Item<'a, i64>` with generic arguments
    /// After resolution, this becomes the concrete type
    Associated {
        /// Base type (T in <T as Trait>::Item)
        base: Box<ResolvedType>,
        /// Trait name (None if using Self::Item syntax)
        trait_name: Option<String>,
        /// Associated type name (Item)
        assoc_name: String,
        /// GAT generic arguments (e.g., ['a, i64] in Self::Item<'a, i64>)
        generics: Vec<ResolvedType>,
    },

    /// Linear type wrapper - must be used exactly once
    Linear(Box<ResolvedType>),

    /// Affine type wrapper - can be used at most once
    Affine(Box<ResolvedType>),

    /// Dependent type (Refinement type): a type refined by a predicate
    /// Example: `{n: i64 | n > 0}` represents positive integers
    /// The predicate is stored as a string representation for display/comparison
    Dependent {
        /// The bound variable name
        var_name: String,
        /// The base type being refined
        base: Box<ResolvedType>,
        /// The predicate expression (stored as string for comparison)
        predicate: String,
    },

    /// Reference with explicit lifetime: `&'a T`
    RefLifetime {
        lifetime: String,
        inner: Box<ResolvedType>,
    },

    /// Mutable reference with explicit lifetime: `&'a mut T`
    RefMutLifetime {
        lifetime: String,
        inner: Box<ResolvedType>,
    },

    /// Lifetime parameter (e.g., 'a, 'static)
    Lifetime(String),

    /// Lazy type: `Lazy<T>` - Deferred evaluation thunk
    /// Contains a closure that computes T when forced, and caches the result.
    Lazy(Box<ResolvedType>),
}

impl ResolvedType {
    pub fn is_numeric(&self) -> bool {
        match self {
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128
            | ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128
            | ResolvedType::F32
            | ResolvedType::F64
            | ResolvedType::Generic(_) // Generics are assumed to support numeric ops
            | ResolvedType::Var(_) // Type variables might resolve to numeric
            | ResolvedType::Unknown => true, // Unknown might be numeric
            // Wrapper types delegate to inner type
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.is_numeric(),
            ResolvedType::Dependent { base, .. } => base.is_numeric(),
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128
            | ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128 => true,
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.is_integer(),
            ResolvedType::Dependent { base, .. } => base.is_integer(),
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            ResolvedType::F32 | ResolvedType::F64 => true,
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.is_float(),
            ResolvedType::Dependent { base, .. } => base.is_float(),
            _ => false,
        }
    }

    /// Get the base type, unwrapping any refinement wrappers (Linear, Affine, Dependent)
    pub fn base_type(&self) -> &ResolvedType {
        match self {
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => inner.base_type(),
            ResolvedType::Dependent { base, .. } => base.base_type(),
            _ => self,
        }
    }
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::I8 => write!(f, "i8"),
            ResolvedType::I16 => write!(f, "i16"),
            ResolvedType::I32 => write!(f, "i32"),
            ResolvedType::I64 => write!(f, "i64"),
            ResolvedType::I128 => write!(f, "i128"),
            ResolvedType::U8 => write!(f, "u8"),
            ResolvedType::U16 => write!(f, "u16"),
            ResolvedType::U32 => write!(f, "u32"),
            ResolvedType::U64 => write!(f, "u64"),
            ResolvedType::U128 => write!(f, "u128"),
            ResolvedType::F32 => write!(f, "f32"),
            ResolvedType::F64 => write!(f, "f64"),
            ResolvedType::Bool => write!(f, "bool"),
            ResolvedType::Str => write!(f, "str"),
            ResolvedType::Unit => write!(f, "()"),
            ResolvedType::Array(t) => write!(f, "[{}]", t),
            ResolvedType::ConstArray { element, size } => write!(f, "[{}; {}]", element, size),
            ResolvedType::Map(k, v) => write!(f, "[{}:{}]", k, v),
            ResolvedType::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            ResolvedType::Optional(t) => write!(f, "{}?", t),
            ResolvedType::Result(t, e) => write!(f, "Result<{}, {}>", t, e),
            ResolvedType::Pointer(t) => write!(f, "*{}", t),
            ResolvedType::Ref(t) => write!(f, "&{}", t),
            ResolvedType::RefMut(t) => write!(f, "&mut {}", t),
            ResolvedType::Slice(t) => write!(f, "&[{}]", t),
            ResolvedType::SliceMut(t) => write!(f, "&mut [{}]", t),
            ResolvedType::Range(t) => write!(f, "Range<{}>", t),
            ResolvedType::Future(t) => write!(f, "Future<{}>", t),
            ResolvedType::Fn {
                params,
                ret,
                effects,
            } => {
                if let Some(effects) = effects {
                    if !effects.is_pure() {
                        write!(f, "{} ", effects)?;
                    }
                }
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::FnPtr {
                params,
                ret,
                is_vararg,
                effects,
            } => {
                if let Some(effects) = effects {
                    if !effects.is_pure() {
                        write!(f, "{} ", effects)?;
                    }
                }
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                if *is_vararg {
                    if !params.is_empty() {
                        write!(f, ",")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::Named { name, generics } => {
                write!(f, "{}", name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Var(id) => write!(f, "?{}", id),
            ResolvedType::Generic(name) => write!(f, "{}", name),
            ResolvedType::ConstGeneric(name) => write!(f, "const {}", name),
            ResolvedType::Unknown => write!(f, "?"),
            ResolvedType::Never => write!(f, "!"),
            ResolvedType::Vector { element, lanes } => write!(f, "Vec{}x{}", lanes, element),
            ResolvedType::DynTrait {
                trait_name,
                generics,
            } => {
                write!(f, "dyn {}", trait_name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => {
                if let Some(trait_name) = trait_name {
                    write!(f, "<{} as {}>::{}", base, trait_name, assoc_name)?;
                } else {
                    write!(f, "{}::{}", base, assoc_name)?;
                }
                // Display GAT parameters if present
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Linear(inner) => write!(f, "linear {}", inner),
            ResolvedType::Affine(inner) => write!(f, "affine {}", inner),
            ResolvedType::Dependent {
                var_name,
                base,
                predicate,
            } => {
                write!(f, "{{{}: {} | {}}}", var_name, base, predicate)
            }
            ResolvedType::RefLifetime { lifetime, inner } => {
                write!(f, "&'{} {}", lifetime, inner)
            }
            ResolvedType::RefMutLifetime { lifetime, inner } => {
                write!(f, "&'{} mut {}", lifetime, inner)
            }
            ResolvedType::Lifetime(name) => write!(f, "'{}", name),
            ResolvedType::Lazy(inner) => write!(f, "Lazy<{}>", inner),
        }
    }
}

/// Contract clause for formal verification (requires/ensures)
#[derive(Debug, Clone)]
pub struct ContractClause {
    /// Original expression string for error messages
    pub expr_str: String,
    /// Source span for error reporting
    pub span: Span,
}

/// Contract specification for Design by Contract
#[derive(Debug, Clone, Default)]
pub struct ContractSpec {
    /// Preconditions (requires clauses)
    pub requires: Vec<ContractClause>,
    /// Postconditions (ensures clauses)
    pub ensures: Vec<ContractClause>,
}

impl ContractSpec {
    /// Check if the contract specification has any clauses
    pub fn is_empty(&self) -> bool {
        self.requires.is_empty() && self.ensures.is_empty()
    }
}

// ============================================================================
// Effect System
// ============================================================================

/// Effect kinds representing different types of side effects
///
/// The effect system tracks what side effects a function may have,
/// enabling purity checking, optimization, and formal verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Pure - no observable side effects
    /// Can be safely memoized, reordered, or eliminated
    Pure,

    /// Read - reads from shared/global state
    /// Can be reordered with other reads
    Read,

    /// Write - writes to shared/global state
    /// Cannot be reordered with reads or writes
    Write,

    /// Allocate - allocates memory (heap allocation)
    /// Generally side-effect free but may fail
    Alloc,

    /// IO - performs input/output operations
    /// Console, file system, network
    IO,

    /// Async - may suspend execution
    /// Async/await, yield, sleep
    Async,

    /// Panic - may panic or abort
    /// unwrap, assert, divide by zero
    Panic,

    /// NonDet - non-deterministic (random, time)
    /// Different results on each call
    NonDet,

    /// Unsafe - performs unsafe operations
    /// Raw pointer dereference, FFI calls
    Unsafe,

    /// Diverge - may not terminate
    /// Infinite loops, recursion without base case
    Diverge,
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Pure => write!(f, "pure"),
            Effect::Read => write!(f, "read"),
            Effect::Write => write!(f, "write"),
            Effect::Alloc => write!(f, "alloc"),
            Effect::IO => write!(f, "io"),
            Effect::Async => write!(f, "async"),
            Effect::Panic => write!(f, "panic"),
            Effect::NonDet => write!(f, "nondet"),
            Effect::Unsafe => write!(f, "unsafe"),
            Effect::Diverge => write!(f, "diverge"),
        }
    }
}

/// Effect set - represents the combination of effects a function may have
///
/// Effect sets form a lattice where:
/// - Bottom (Pure) ⊆ All effects
/// - Top (All effects) is the supremum
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectSet {
    /// Set of individual effects
    effects: std::collections::HashSet<Effect>,
}

impl EffectSet {
    /// Create a new empty effect set (pure)
    pub fn pure() -> Self {
        Self {
            effects: std::collections::HashSet::new(),
        }
    }

    /// Create an effect set with a single effect
    pub fn single(effect: Effect) -> Self {
        let mut effects = std::collections::HashSet::new();
        if effect != Effect::Pure {
            effects.insert(effect);
        }
        Self { effects }
    }

    /// Create an effect set with multiple effects
    pub fn from_effects(effects: impl IntoIterator<Item = Effect>) -> Self {
        let mut set = std::collections::HashSet::new();
        for effect in effects {
            if effect != Effect::Pure {
                set.insert(effect);
            }
        }
        Self { effects: set }
    }

    /// Check if this is a pure (empty) effect set
    pub fn is_pure(&self) -> bool {
        self.effects.is_empty()
    }

    /// Check if this effect set is read-only (no writes, IO, etc.)
    pub fn is_readonly(&self) -> bool {
        !self.effects.contains(&Effect::Write)
            && !self.effects.contains(&Effect::IO)
            && !self.effects.contains(&Effect::Alloc)
    }

    /// Check if this effect set contains a specific effect
    pub fn contains(&self, effect: Effect) -> bool {
        if effect == Effect::Pure {
            return self.effects.is_empty();
        }
        self.effects.contains(&effect)
    }

    /// Add an effect to this set
    pub fn add(&mut self, effect: Effect) {
        if effect != Effect::Pure {
            self.effects.insert(effect);
        }
    }

    /// Union two effect sets (combines all effects)
    pub fn union(&self, other: &EffectSet) -> EffectSet {
        EffectSet {
            effects: self.effects.union(&other.effects).copied().collect(),
        }
    }

    /// Intersection of two effect sets
    pub fn intersection(&self, other: &EffectSet) -> EffectSet {
        EffectSet {
            effects: self.effects.intersection(&other.effects).copied().collect(),
        }
    }

    /// Check if this effect set is a subset of another
    pub fn is_subset_of(&self, other: &EffectSet) -> bool {
        self.effects.is_subset(&other.effects)
    }

    /// Get all effects in this set
    pub fn effects(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter()
    }

    /// Create common effect sets
    pub fn io() -> Self {
        Self::from_effects([Effect::IO, Effect::Panic])
    }

    pub fn alloc() -> Self {
        Self::from_effects([Effect::Alloc, Effect::Panic])
    }

    pub fn read_write() -> Self {
        Self::from_effects([Effect::Read, Effect::Write])
    }

    pub fn total() -> Self {
        Self::from_effects([
            Effect::Read,
            Effect::Write,
            Effect::Alloc,
            Effect::IO,
            Effect::Async,
            Effect::Panic,
            Effect::NonDet,
            Effect::Unsafe,
            Effect::Diverge,
        ])
    }
}

impl std::fmt::Display for EffectSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.effects.is_empty() {
            write!(f, "pure")
        } else {
            let effects: Vec<_> = self.effects.iter().map(|e| e.to_string()).collect();
            write!(f, "{{{}}}", effects.join(", "))
        }
    }
}

impl std::hash::Hash for EffectSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash in a deterministic order
        let mut effects: Vec<_> = self.effects.iter().collect();
        effects.sort_by_key(|e| format!("{:?}", e));
        for effect in effects {
            effect.hash(state);
        }
    }
}

/// Function effect annotation - how effects are declared/inferred
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EffectAnnotation {
    /// No annotation - infer from body
    #[default]
    Infer,
    /// Explicitly declared as pure
    Pure,
    /// Explicitly declared with specific effects
    Declared(EffectSet),
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub generics: Vec<String>,
    pub generic_bounds: HashMap<String, Vec<String>>, // generic name -> trait bounds
    pub params: Vec<(String, ResolvedType, bool)>,    // (name, type, is_mut)
    pub ret: ResolvedType,
    pub is_async: bool,
    pub is_vararg: bool, // true for variadic C functions (printf, etc.)
    /// Number of required parameters (those without default values)
    /// If None, all parameters are required (backward compatible)
    pub required_params: Option<usize>,
    /// Contract specification for formal verification (requires/ensures)
    pub contracts: Option<ContractSpec>,
    /// Effect annotation - declared or inferred effects
    pub effect_annotation: EffectAnnotation,
    /// Inferred effects (populated during type checking)
    pub inferred_effects: Option<EffectSet>,
}

impl Default for FunctionSig {
    fn default() -> Self {
        Self {
            name: String::new(),
            generics: vec![],
            generic_bounds: HashMap::new(),
            params: vec![],
            ret: ResolvedType::Unit,
            is_async: false,
            is_vararg: false,
            required_params: None,
            contracts: None,
            effect_annotation: EffectAnnotation::Infer,
            inferred_effects: None,
        }
    }
}

impl FunctionSig {
    /// Return the minimum number of required arguments
    pub fn min_args(&self) -> usize {
        self.required_params.unwrap_or(self.params.len())
    }
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
    pub field_order: Vec<String>, // Preserves declaration order for tuple literal syntax
    pub methods: HashMap<String, FunctionSig>,
    pub repr_c: bool, // true if #[repr(C)] attribute is present
}

/// Enum variant field types
#[derive(Debug, Clone)]
pub enum VariantFieldTypes {
    /// Unit variant (no fields)
    Unit,
    /// Tuple variant with positional fields
    Tuple(Vec<ResolvedType>),
    /// Struct variant with named fields
    Struct(HashMap<String, ResolvedType>),
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: HashMap<String, VariantFieldTypes>,
    pub methods: HashMap<String, FunctionSig>,
}

/// Union definition (untagged, C-style)
/// All fields share the same memory location (offset 0).
/// No runtime tag - caller is responsible for knowing which field is active.
#[derive(Debug, Clone)]
pub struct UnionDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
}

/// Linearity mode for linear type system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Linearity {
    /// Unrestricted (can be used any number of times)
    #[default]
    Unrestricted,
    /// Linear (must be used exactly once)
    Linear,
    /// Affine (can be used at most once - dropped without use is OK)
    Affine,
}

impl Linearity {
    /// Check if this linearity requires tracking
    pub fn requires_tracking(&self) -> bool {
        matches!(self, Linearity::Linear | Linearity::Affine)
    }

    /// Check if this linearity allows dropping without use
    pub fn allows_drop_without_use(&self) -> bool {
        matches!(self, Linearity::Unrestricted | Linearity::Affine)
    }

    /// Check if a use count is valid for this linearity
    pub fn is_valid_use_count(&self, count: usize) -> bool {
        match self {
            Linearity::Unrestricted => true,
            Linearity::Linear => count == 1,
            Linearity::Affine => count <= 1,
        }
    }
}

/// Variable info (internal to type checker).
/// Reserved for linear/affine type tracking.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct VarInfo {
    pub(crate) ty: ResolvedType,
    pub(crate) is_mut: bool,
    pub(crate) linearity: Linearity,
    pub(crate) use_count: usize,
    /// Span where the variable was defined (for error messages)
    pub(crate) defined_at: Option<vais_ast::Span>,
}

/// Generic instantiation tracking for monomorphization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericInstantiation {
    /// Base name of the generic item (function or struct)
    pub base_name: String,
    /// Concrete type arguments
    pub type_args: Vec<ResolvedType>,
    /// Concrete const arguments (name -> value)
    pub const_args: Vec<(String, i64)>,
    /// Mangled name for code generation
    pub mangled_name: String,
    /// Kind of instantiation
    pub kind: InstantiationKind,
}

/// Kind of generic instantiation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstantiationKind {
    Function,
    Struct,
    Method { struct_name: String },
}

impl GenericInstantiation {
    /// Create a new function instantiation
    pub fn function(base_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let mangled = mangle_name(base_name, &type_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args: Vec::new(),
            mangled_name: mangled,
            kind: InstantiationKind::Function,
        }
    }

    /// Create a new function instantiation with const generic arguments
    pub fn function_with_consts(
        base_name: &str,
        type_args: Vec<ResolvedType>,
        const_args: Vec<(String, i64)>,
    ) -> Self {
        let mangled = mangle_name_with_consts(base_name, &type_args, &const_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args,
            mangled_name: mangled,
            kind: InstantiationKind::Function,
        }
    }

    /// Create a new struct instantiation
    pub fn struct_type(base_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let mangled = mangle_name(base_name, &type_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args: Vec::new(),
            mangled_name: mangled,
            kind: InstantiationKind::Struct,
        }
    }

    /// Create a new struct instantiation with const generic arguments
    pub fn struct_type_with_consts(
        base_name: &str,
        type_args: Vec<ResolvedType>,
        const_args: Vec<(String, i64)>,
    ) -> Self {
        let mangled = mangle_name_with_consts(base_name, &type_args, &const_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            const_args,
            mangled_name: mangled,
            kind: InstantiationKind::Struct,
        }
    }

    /// Create a new method instantiation
    pub fn method(struct_name: &str, method_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let base = format!("{}_{}", struct_name, method_name);
        let mangled = mangle_name(&base, &type_args);
        Self {
            base_name: method_name.to_string(),
            type_args,
            const_args: Vec::new(),
            mangled_name: mangled,
            kind: InstantiationKind::Method {
                struct_name: struct_name.to_string(),
            },
        }
    }
}

/// Mangle a generic name with type arguments
pub fn mangle_name(base: &str, type_args: &[ResolvedType]) -> String {
    if type_args.is_empty() {
        base.to_string()
    } else {
        let args_str = type_args
            .iter()
            .map(mangle_type)
            .collect::<Vec<_>>()
            .join("_");
        format!("{}${}", base, args_str)
    }
}

/// Mangle a generic name with both type and const arguments
pub fn mangle_name_with_consts(
    base: &str,
    type_args: &[ResolvedType],
    const_args: &[(String, i64)],
) -> String {
    let mut parts = Vec::new();
    for ty in type_args {
        parts.push(mangle_type(ty));
    }
    for (_, val) in const_args {
        parts.push(format!("c{}", val));
    }
    if parts.is_empty() {
        base.to_string()
    } else {
        format!("{}${}", base, parts.join("_"))
    }
}

/// Mangle a single type for use in mangled names
pub fn mangle_type(ty: &ResolvedType) -> String {
    match ty {
        ResolvedType::I8 => "i8".to_string(),
        ResolvedType::I16 => "i16".to_string(),
        ResolvedType::I32 => "i32".to_string(),
        ResolvedType::I64 => "i64".to_string(),
        ResolvedType::I128 => "i128".to_string(),
        ResolvedType::U8 => "u8".to_string(),
        ResolvedType::U16 => "u16".to_string(),
        ResolvedType::U32 => "u32".to_string(),
        ResolvedType::U64 => "u64".to_string(),
        ResolvedType::U128 => "u128".to_string(),
        ResolvedType::F32 => "f32".to_string(),
        ResolvedType::F64 => "f64".to_string(),
        ResolvedType::Bool => "bool".to_string(),
        ResolvedType::Str => "str".to_string(),
        ResolvedType::Unit => "unit".to_string(),
        ResolvedType::Named { name, generics } => {
            if generics.is_empty() {
                name.clone()
            } else {
                let args = generics
                    .iter()
                    .map(mangle_type)
                    .collect::<Vec<_>>()
                    .join("_");
                format!("{}_{}", name, args)
            }
        }
        ResolvedType::Array(inner) => format!("arr_{}", mangle_type(inner)),
        ResolvedType::Pointer(inner) => format!("ptr_{}", mangle_type(inner)),
        ResolvedType::Ref(inner) => format!("ref_{}", mangle_type(inner)),
        ResolvedType::RefMut(inner) => format!("refmut_{}", mangle_type(inner)),
        ResolvedType::Slice(inner) => format!("slice_{}", mangle_type(inner)),
        ResolvedType::SliceMut(inner) => format!("slicemut_{}", mangle_type(inner)),
        ResolvedType::Optional(inner) => format!("opt_{}", mangle_type(inner)),
        ResolvedType::Result(ok, err) => format!("res_{}_{}", mangle_type(ok), mangle_type(err)),
        ResolvedType::Future(inner) => format!("fut_{}", mangle_type(inner)),
        ResolvedType::Tuple(types) => {
            let args = types.iter().map(mangle_type).collect::<Vec<_>>().join("_");
            format!("tup_{}", args)
        }
        ResolvedType::Fn { params, ret, .. } => {
            let params_str = params.iter().map(mangle_type).collect::<Vec<_>>().join("_");
            format!("fn_{}_{}", params_str, mangle_type(ret))
        }
        ResolvedType::Generic(name) => name.clone(),
        ResolvedType::ConstGeneric(name) => format!("cg_{}", name),
        ResolvedType::ConstArray { element, size } => {
            let size_str = match size.try_evaluate() {
                Some(n) => format!("{}", n),
                None => "dyn".to_string(),
            };
            format!("arr{}_{}", size_str, mangle_type(element))
        }
        ResolvedType::Var(id) => format!("v{}", id),
        ResolvedType::Vector { element, lanes } => format!("vec{}_{}", lanes, mangle_type(element)),
        _ => "unknown".to_string(),
    }
}

/// Substitute generic type parameters with concrete types
pub fn substitute_type(
    ty: &ResolvedType,
    substitutions: &HashMap<String, ResolvedType>,
) -> ResolvedType {
    match ty {
        ResolvedType::Generic(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        ResolvedType::Named { name, generics } => {
            // Early return if no substitution needed and no generics to recurse into
            if generics.is_empty() && !substitutions.contains_key(name) {
                return ty.clone();
            }

            // Check if any generic parameter changed
            let mut changed = false;
            let new_generics: Vec<ResolvedType> = generics
                .iter()
                .map(|g| {
                    let subst = substitute_type(g, substitutions);
                    if !changed && g != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();

            // If no changes, return clone of original
            if !changed {
                return ty.clone();
            }

            ResolvedType::Named {
                name: name.clone(),
                generics: new_generics,
            }
        }
        ResolvedType::Array(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Array(Box::new(new_inner))
        }
        ResolvedType::Pointer(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Pointer(Box::new(new_inner))
        }
        ResolvedType::Ref(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Ref(Box::new(new_inner))
        }
        ResolvedType::RefMut(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::RefMut(Box::new(new_inner))
        }
        ResolvedType::Slice(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Slice(Box::new(new_inner))
        }
        ResolvedType::SliceMut(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::SliceMut(Box::new(new_inner))
        }
        ResolvedType::Optional(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Optional(Box::new(new_inner))
        }
        ResolvedType::Result(ok, err) => {
            let new_ok = substitute_type(ok, substitutions);
            let new_err = substitute_type(err, substitutions);
            if ok.as_ref() == &new_ok && err.as_ref() == &new_err {
                return ty.clone();
            }
            ResolvedType::Result(Box::new(new_ok), Box::new(new_err))
        }
        ResolvedType::Future(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Future(Box::new(new_inner))
        }
        ResolvedType::Tuple(types) => {
            let mut changed = false;
            let new_types: Vec<ResolvedType> = types
                .iter()
                .map(|t| {
                    let subst = substitute_type(t, substitutions);
                    if !changed && t != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();

            if !changed {
                return ty.clone();
            }
            ResolvedType::Tuple(new_types)
        }
        ResolvedType::Fn {
            params,
            ret,
            effects,
        } => {
            let mut changed = false;
            let new_params: Vec<ResolvedType> = params
                .iter()
                .map(|p| {
                    let subst = substitute_type(p, substitutions);
                    if !changed && p != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();
            let new_ret = substitute_type(ret, substitutions);
            if !changed && ret.as_ref() != &new_ret {
                changed = true;
            }

            if !changed {
                return ty.clone();
            }

            ResolvedType::Fn {
                params: new_params,
                ret: Box::new(new_ret),
                effects: effects.clone(),
            }
        }
        ResolvedType::Vector { element, lanes } => {
            let new_element = substitute_type(element, substitutions);
            if element.as_ref() == &new_element {
                return ty.clone();
            }
            ResolvedType::Vector {
                element: Box::new(new_element),
                lanes: *lanes,
            }
        }
        ResolvedType::ConstGeneric(name) => {
            // Const generics can be substituted if a mapping exists
            substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| ty.clone())
        }
        ResolvedType::ConstArray { element, size } => {
            let new_element = substitute_type(element, substitutions);
            // Substitute const parameter names in size expression
            let new_size = substitute_const(size, substitutions);

            // Check if anything changed
            if element.as_ref() == &new_element && size == &new_size {
                return ty.clone();
            }

            ResolvedType::ConstArray {
                element: Box::new(new_element),
                size: new_size,
            }
        }
        // Primitives and other types pass through unchanged
        _ => ty.clone(),
    }
}

/// Substitute const parameter names in a ResolvedConst expression
pub fn substitute_const(
    c: &ResolvedConst,
    _substitutions: &HashMap<String, ResolvedType>,
) -> ResolvedConst {
    // For now, const substitution happens through const_substitutions map
    c.clone()
}

/// Substitute const parameters with concrete values in a ResolvedConst expression
pub fn substitute_const_values(
    c: &ResolvedConst,
    const_subs: &HashMap<String, i64>,
) -> ResolvedConst {
    match c {
        ResolvedConst::Value(_) => c.clone(),
        ResolvedConst::Param(name) => {
            if let Some(&val) = const_subs.get(name) {
                ResolvedConst::Value(val)
            } else {
                c.clone()
            }
        }
        ResolvedConst::BinOp { op, left, right } => {
            let new_left = substitute_const_values(left, const_subs);
            let new_right = substitute_const_values(right, const_subs);
            // Try to evaluate if both are now concrete
            if let (Some(l), Some(r)) = (new_left.try_evaluate(), new_right.try_evaluate()) {
                let result = match op {
                    ConstBinOp::Add => l.checked_add(r),
                    ConstBinOp::Sub => l.checked_sub(r),
                    ConstBinOp::Mul => l.checked_mul(r),
                    ConstBinOp::Div => {
                        if r == 0 {
                            None
                        } else {
                            l.checked_div(r)
                        }
                    }
                };
                if let Some(val) = result {
                    return ResolvedConst::Value(val);
                }
            }
            ResolvedConst::BinOp {
                op: *op,
                left: Box::new(new_left),
                right: Box::new(new_right),
            }
        }
    }
}
