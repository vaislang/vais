//! Type checking errors

use thiserror::Error;
use vais_ast::Span;

use super::utils::find_similar_name;

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
            TypeError::BorrowConflict {
                existing_borrow_at,
                existing_is_mut,
                ..
            } => {
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
            TypeError::DanglingReference {
                source_defined_at, ..
            } => {
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
