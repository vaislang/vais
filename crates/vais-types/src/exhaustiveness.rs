//! Pattern Exhaustiveness Checking for Vais
//!
//! This module implements exhaustiveness checking for match expressions.
//! It ensures that all possible values of the matched expression are covered
//! by at least one pattern arm.
//!
//! The algorithm is based on the "Warnings for pattern matching" paper by
//! Luc Maranget, adapted for Vais's type system.

use crate::ResolvedType;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use vais_ast::{Literal, MatchArm, Pattern, Spanned};

/// Result of exhaustiveness check
#[derive(Debug, Clone)]
pub struct ExhaustivenessResult {
    /// Whether the match is exhaustive
    pub is_exhaustive: bool,
    /// Missing patterns (for error reporting)
    pub missing_patterns: Vec<String>,
    /// Unreachable arms (for warning)
    pub unreachable_arms: Vec<usize>,
}

/// Pattern space representation for exhaustiveness checking
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSpace {
    /// Matches everything
    Any,
    /// Matches nothing (empty set)
    Empty,
    /// Integer value
    Int(i64),
    /// Integer range [start, end] (inclusive)
    IntRange(i64, i64),
    /// Boolean value
    Bool(bool),
    /// String value
    String(String),
    /// Wildcard (matches all)
    Wildcard,
    /// Constructor with fields (for enums/structs)
    Constructor {
        name: String,
        fields: Vec<PatternSpace>,
    },
    /// Or pattern (union of spaces)
    Or(Vec<PatternSpace>),
    /// Negation (everything except this)
    Not(Box<PatternSpace>),
}

/// Exhaustiveness checker
pub struct ExhaustivenessChecker {
    /// Enum definitions for looking up variants
    enums: HashMap<String, Vec<String>>,
    /// Cache for exhaustiveness check results
    /// Key: (type hash, patterns hash) -> ExhaustivenessResult
    cache: HashMap<(u64, u64), ExhaustivenessResult>,
}

impl ExhaustivenessChecker {
    pub fn new() -> Self {
        Self {
            enums: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    /// Clear the exhaustiveness check cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Compute hash for a type
    fn hash_type(ty: &ResolvedType) -> u64 {
        let mut hasher = DefaultHasher::new();
        format!("{:?}", ty).hash(&mut hasher);
        hasher.finish()
    }

    /// Compute hash for match arms patterns
    fn hash_patterns(arms: &[MatchArm]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for arm in arms {
            format!("{:?}", arm.pattern).hash(&mut hasher);
            if arm.guard.is_some() {
                "guard".hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Register enum variants for exhaustiveness checking
    pub fn register_enum(&mut self, name: &str, variants: Vec<String>) {
        self.enums.insert(name.to_string(), variants);
    }

    /// Check if a match expression is exhaustive (with caching)
    pub fn check_match(
        &mut self,
        scrutinee_type: &ResolvedType,
        arms: &[MatchArm],
    ) -> ExhaustivenessResult {
        // Check cache first
        let type_hash = Self::hash_type(scrutinee_type);
        let patterns_hash = Self::hash_patterns(arms);
        let cache_key = (type_hash, patterns_hash);

        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        // Perform the check
        let mut missing_patterns = Vec::new();
        let mut unreachable_arms = Vec::new();

        // Build the uncovered space
        let mut uncovered = self.type_to_pattern_space(scrutinee_type);

        // Track what each arm covers
        for (idx, arm) in arms.iter().enumerate() {
            // Skip arms with guards - they don't guarantee coverage
            if arm.guard.is_some() {
                continue;
            }

            let arm_space = self.pattern_to_space(&arm.pattern);

            // Check if this arm is reachable
            if self.is_empty(&self.intersect(&uncovered, &arm_space)) {
                unreachable_arms.push(idx);
            }

            // Subtract this arm's coverage from uncovered
            uncovered = self.subtract(&uncovered, &arm_space);
        }

        // Check what's still uncovered
        let is_exhaustive = self.is_empty(&uncovered);

        if !is_exhaustive {
            missing_patterns = self.space_to_patterns(&uncovered, scrutinee_type);
        }

        let result = ExhaustivenessResult {
            is_exhaustive,
            missing_patterns,
            unreachable_arms,
        };

        // Store in cache
        self.cache.insert(cache_key, result.clone());
        result
    }

    /// Convert a type to its full pattern space
    fn type_to_pattern_space(&self, ty: &ResolvedType) -> PatternSpace {
        match ty {
            ResolvedType::Bool => {
                PatternSpace::Or(vec![PatternSpace::Bool(true), PatternSpace::Bool(false)])
            }
            ResolvedType::I8 => PatternSpace::IntRange(i8::MIN as i64, i8::MAX as i64),
            ResolvedType::I16 => PatternSpace::IntRange(i16::MIN as i64, i16::MAX as i64),
            ResolvedType::I32 => PatternSpace::IntRange(i32::MIN as i64, i32::MAX as i64),
            ResolvedType::I64 => PatternSpace::IntRange(i64::MIN, i64::MAX),
            ResolvedType::U8 => PatternSpace::IntRange(0, u8::MAX as i64),
            ResolvedType::U16 => PatternSpace::IntRange(0, u16::MAX as i64),
            ResolvedType::U32 => PatternSpace::IntRange(0, u32::MAX as i64),
            ResolvedType::U64 => PatternSpace::IntRange(0, i64::MAX), // Approximation
            ResolvedType::Named { name, .. } => {
                // Check if it's an enum
                if let Some(variants) = self.enums.get(name) {
                    PatternSpace::Or(
                        variants
                            .iter()
                            .map(|v| PatternSpace::Constructor {
                                name: v.clone(),
                                fields: vec![],
                            })
                            .collect(),
                    )
                } else {
                    // Struct or unknown type - assume wildcard needed
                    PatternSpace::Any
                }
            }
            ResolvedType::Optional(inner) => {
                // Option is Some(T) | None
                PatternSpace::Or(vec![
                    PatternSpace::Constructor {
                        name: "Some".to_string(),
                        fields: vec![self.type_to_pattern_space(inner)],
                    },
                    PatternSpace::Constructor {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                ])
            }
            ResolvedType::Result(inner) => {
                // Result is Ok(T) | Err(E)
                PatternSpace::Or(vec![
                    PatternSpace::Constructor {
                        name: "Ok".to_string(),
                        fields: vec![self.type_to_pattern_space(inner)],
                    },
                    PatternSpace::Constructor {
                        name: "Err".to_string(),
                        fields: vec![PatternSpace::Any],
                    },
                ])
            }
            ResolvedType::Tuple(types) => PatternSpace::Constructor {
                name: "".to_string(), // Anonymous tuple
                fields: types
                    .iter()
                    .map(|t| self.type_to_pattern_space(t))
                    .collect(),
            },
            // Other types need wildcard
            _ => PatternSpace::Any,
        }
    }

    /// Convert a pattern to a pattern space
    fn pattern_to_space(&self, pattern: &Spanned<Pattern>) -> PatternSpace {
        match &pattern.node {
            Pattern::Wildcard => PatternSpace::Any,
            Pattern::Ident(_) => PatternSpace::Any, // Variable binding matches anything
            Pattern::Literal(lit) => match lit {
                Literal::Int(n) => PatternSpace::Int(*n),
                Literal::Bool(b) => PatternSpace::Bool(*b),
                Literal::Float(_) => PatternSpace::Any, // Float comparison is tricky
                Literal::String(s) => PatternSpace::String(s.clone()),
            },
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start_val = start
                    .as_ref()
                    .and_then(|p| {
                        if let Pattern::Literal(Literal::Int(n)) = &p.node {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(i64::MIN);

                let end_val = end
                    .as_ref()
                    .and_then(|p| {
                        if let Pattern::Literal(Literal::Int(n)) = &p.node {
                            Some(if *inclusive { *n } else { *n - 1 })
                        } else {
                            None
                        }
                    })
                    .unwrap_or(i64::MAX);

                PatternSpace::IntRange(start_val, end_val)
            }
            Pattern::Or(patterns) => {
                PatternSpace::Or(patterns.iter().map(|p| self.pattern_to_space(p)).collect())
            }
            Pattern::Variant { name, fields } => PatternSpace::Constructor {
                name: name.node.clone(),
                fields: fields.iter().map(|p| self.pattern_to_space(p)).collect(),
            },
            Pattern::Struct { name, fields } => PatternSpace::Constructor {
                name: name.node.clone(),
                fields: fields
                    .iter()
                    .map(|(_, opt_pat)| {
                        opt_pat
                            .as_ref()
                            .map(|p| self.pattern_to_space(p))
                            .unwrap_or(PatternSpace::Any)
                    })
                    .collect(),
            },
            Pattern::Tuple(patterns) => PatternSpace::Constructor {
                name: "".to_string(),
                fields: patterns.iter().map(|p| self.pattern_to_space(p)).collect(),
            },
        }
    }

    /// Check if a pattern space is empty
    fn is_empty(&self, space: &PatternSpace) -> bool {
        match space {
            PatternSpace::Empty => true,
            PatternSpace::Any | PatternSpace::Wildcard => false,
            PatternSpace::Int(_) | PatternSpace::Bool(_) | PatternSpace::String(_) => false,
            PatternSpace::IntRange(start, end) => start > end,
            PatternSpace::Constructor { fields, .. } => fields.iter().any(|f| self.is_empty(f)),
            PatternSpace::Or(spaces) => spaces.iter().all(|s| self.is_empty(s)),
            PatternSpace::Not(inner) => matches!(**inner, PatternSpace::Any),
        }
    }

    /// Intersect two pattern spaces
    fn intersect(&self, a: &PatternSpace, b: &PatternSpace) -> PatternSpace {
        match (a, b) {
            (PatternSpace::Empty, _) | (_, PatternSpace::Empty) => PatternSpace::Empty,
            (PatternSpace::Any, other) | (other, PatternSpace::Any) => other.clone(),
            (PatternSpace::Wildcard, other) | (other, PatternSpace::Wildcard) => other.clone(),

            (PatternSpace::Int(x), PatternSpace::Int(y)) => {
                if x == y {
                    PatternSpace::Int(*x)
                } else {
                    PatternSpace::Empty
                }
            }
            (PatternSpace::Int(x), PatternSpace::IntRange(start, end))
            | (PatternSpace::IntRange(start, end), PatternSpace::Int(x)) => {
                if *x >= *start && *x <= *end {
                    PatternSpace::Int(*x)
                } else {
                    PatternSpace::Empty
                }
            }
            (PatternSpace::IntRange(s1, e1), PatternSpace::IntRange(s2, e2)) => {
                let new_start = (*s1).max(*s2);
                let new_end = (*e1).min(*e2);
                if new_start <= new_end {
                    PatternSpace::IntRange(new_start, new_end)
                } else {
                    PatternSpace::Empty
                }
            }

            (PatternSpace::Bool(x), PatternSpace::Bool(y)) => {
                if x == y {
                    PatternSpace::Bool(*x)
                } else {
                    PatternSpace::Empty
                }
            }

            (PatternSpace::String(x), PatternSpace::String(y)) => {
                if x == y {
                    PatternSpace::String(x.clone())
                } else {
                    PatternSpace::Empty
                }
            }

            (
                PatternSpace::Constructor {
                    name: n1,
                    fields: f1,
                },
                PatternSpace::Constructor {
                    name: n2,
                    fields: f2,
                },
            ) => {
                if n1 == n2 && f1.len() == f2.len() {
                    let new_fields: Vec<_> = f1
                        .iter()
                        .zip(f2.iter())
                        .map(|(a, b)| self.intersect(a, b))
                        .collect();
                    if new_fields.iter().any(|f| self.is_empty(f)) {
                        PatternSpace::Empty
                    } else {
                        PatternSpace::Constructor {
                            name: n1.clone(),
                            fields: new_fields,
                        }
                    }
                } else {
                    PatternSpace::Empty
                }
            }

            (PatternSpace::Or(spaces), other) | (other, PatternSpace::Or(spaces)) => {
                let new_spaces: Vec<_> = spaces
                    .iter()
                    .map(|s| self.intersect(s, other))
                    .filter(|s| !self.is_empty(s))
                    .collect();
                if new_spaces.is_empty() {
                    PatternSpace::Empty
                } else if new_spaces.len() == 1 {
                    // SAFETY: length checked to be exactly 1
                    new_spaces
                        .into_iter()
                        .next()
                        .expect("length verified to be 1")
                } else {
                    PatternSpace::Or(new_spaces)
                }
            }

            _ => PatternSpace::Empty,
        }
    }

    /// Subtract pattern space b from a
    fn subtract(&self, a: &PatternSpace, b: &PatternSpace) -> PatternSpace {
        match (a, b) {
            (_, PatternSpace::Empty) => a.clone(),
            (PatternSpace::Empty, _) => PatternSpace::Empty,
            (_, PatternSpace::Any) | (_, PatternSpace::Wildcard) => PatternSpace::Empty,
            (PatternSpace::Any, _) | (PatternSpace::Wildcard, _) => {
                PatternSpace::Not(Box::new(b.clone()))
            }

            (PatternSpace::Int(x), PatternSpace::Int(y)) => {
                if x == y {
                    PatternSpace::Empty
                } else {
                    PatternSpace::Int(*x)
                }
            }
            (PatternSpace::Int(x), PatternSpace::IntRange(start, end)) => {
                if *x >= *start && *x <= *end {
                    PatternSpace::Empty
                } else {
                    PatternSpace::Int(*x)
                }
            }
            (PatternSpace::IntRange(s1, e1), PatternSpace::Int(x)) => {
                if *x < *s1 || *x > *e1 {
                    PatternSpace::IntRange(*s1, *e1)
                } else if *x == *s1 && *x == *e1 {
                    PatternSpace::Empty
                } else if *x == *s1 {
                    PatternSpace::IntRange(*s1 + 1, *e1)
                } else if *x == *e1 {
                    PatternSpace::IntRange(*s1, *e1 - 1)
                } else {
                    // Split into two ranges
                    PatternSpace::Or(vec![
                        PatternSpace::IntRange(*s1, *x - 1),
                        PatternSpace::IntRange(*x + 1, *e1),
                    ])
                }
            }
            (PatternSpace::IntRange(s1, e1), PatternSpace::IntRange(s2, e2)) => {
                // Complex range subtraction
                if *s2 > *e1 || *e2 < *s1 {
                    // No overlap
                    PatternSpace::IntRange(*s1, *e1)
                } else if *s2 <= *s1 && *e2 >= *e1 {
                    // Completely covered
                    PatternSpace::Empty
                } else if *s2 > *s1 && *e2 < *e1 {
                    // Hole in the middle
                    PatternSpace::Or(vec![
                        PatternSpace::IntRange(*s1, *s2 - 1),
                        PatternSpace::IntRange(*e2 + 1, *e1),
                    ])
                } else if *s2 <= *s1 {
                    // Covered from start
                    PatternSpace::IntRange(*e2 + 1, *e1)
                } else {
                    // Covered from end
                    PatternSpace::IntRange(*s1, *s2 - 1)
                }
            }

            (PatternSpace::Bool(x), PatternSpace::Bool(y)) => {
                if x == y {
                    PatternSpace::Empty
                } else {
                    PatternSpace::Bool(*x)
                }
            }

            (PatternSpace::Or(spaces), _) => {
                let new_spaces: Vec<_> = spaces
                    .iter()
                    .map(|s| self.subtract(s, b))
                    .filter(|s| !self.is_empty(s))
                    .collect();
                if new_spaces.is_empty() {
                    PatternSpace::Empty
                } else if new_spaces.len() == 1 {
                    // SAFETY: length checked to be exactly 1
                    new_spaces
                        .into_iter()
                        .next()
                        .expect("length verified to be 1")
                } else {
                    PatternSpace::Or(new_spaces)
                }
            }

            (_, PatternSpace::Or(spaces)) => {
                let mut result = a.clone();
                for space in spaces {
                    result = self.subtract(&result, space);
                }
                result
            }

            (
                PatternSpace::Constructor {
                    name: n1,
                    fields: f1,
                },
                PatternSpace::Constructor {
                    name: n2,
                    fields: f2,
                },
            ) => {
                if n1 != n2 || f1.len() != f2.len() {
                    a.clone()
                } else {
                    // Subtract field by field (complex case)
                    // For simplicity, if any field is completely subtracted, the whole is empty
                    let all_any = f2.iter().all(|f| matches!(f, PatternSpace::Any));
                    if all_any {
                        PatternSpace::Empty
                    } else {
                        // Partial subtraction - return the original minus covered parts
                        // This is an approximation; full implementation would need more work
                        a.clone()
                    }
                }
            }

            _ => a.clone(),
        }
    }

    /// Convert uncovered pattern space to human-readable patterns
    fn space_to_patterns(&self, space: &PatternSpace, _ty: &ResolvedType) -> Vec<String> {
        match space {
            PatternSpace::Empty => vec![],
            PatternSpace::Any | PatternSpace::Wildcard => vec!["_".to_string()],
            PatternSpace::Int(n) => vec![format!("{}", n)],
            PatternSpace::IntRange(start, end) => {
                if end - start <= 5 {
                    // List individual values for small ranges
                    (*start..=*end).map(|n| format!("{}", n)).collect()
                } else {
                    vec![format!("{}..={}", start, end)]
                }
            }
            PatternSpace::Bool(b) => vec![format!("{}", b)],
            PatternSpace::String(s) => vec![format!("\"{}\"", s)],
            PatternSpace::Constructor { name, fields } => {
                if fields.is_empty() {
                    vec![name.clone()]
                } else {
                    let field_patterns: Vec<String> = fields
                        .iter()
                        .map(|f| {
                            let p = self.space_to_patterns(f, &ResolvedType::Unknown);
                            if p.is_empty() {
                                "_".to_string()
                            } else {
                                p[0].clone()
                            }
                        })
                        .collect();
                    vec![format!("{}({})", name, field_patterns.join(", "))]
                }
            }
            PatternSpace::Or(spaces) => spaces
                .iter()
                .flat_map(|s| self.space_to_patterns(s, _ty))
                .collect(),
            PatternSpace::Not(inner) => {
                let inner_pats = self.space_to_patterns(inner, _ty);
                if inner_pats.is_empty() {
                    vec!["_".to_string()]
                } else {
                    vec![format!("(not {})", inner_pats.join(" | "))]
                }
            }
        }
    }
}

impl Default for ExhaustivenessChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_int_pattern(n: i64) -> Spanned<Pattern> {
        Spanned::new(Pattern::Literal(Literal::Int(n)), vais_ast::Span::default())
    }

    fn make_wildcard() -> Spanned<Pattern> {
        Spanned::new(Pattern::Wildcard, vais_ast::Span::default())
    }

    fn make_bool_pattern(b: bool) -> Spanned<Pattern> {
        Spanned::new(
            Pattern::Literal(Literal::Bool(b)),
            vais_ast::Span::default(),
        )
    }

    fn make_arm(pattern: Spanned<Pattern>) -> MatchArm {
        MatchArm {
            pattern,
            guard: None,
            body: Box::new(Spanned::new(
                vais_ast::Expr::Int(0),
                vais_ast::Span::default(),
            )),
        }
    }

    #[test]
    fn test_exhaustive_bool_match() {
        let mut checker = ExhaustivenessChecker::new();
        let arms = vec![
            make_arm(make_bool_pattern(true)),
            make_arm(make_bool_pattern(false)),
        ];
        let result = checker.check_match(&ResolvedType::Bool, &arms);
        assert!(result.is_exhaustive);
        assert!(result.missing_patterns.is_empty());
    }

    #[test]
    fn test_non_exhaustive_bool_match() {
        let mut checker = ExhaustivenessChecker::new();
        let arms = vec![make_arm(make_bool_pattern(true))];
        let result = checker.check_match(&ResolvedType::Bool, &arms);
        assert!(!result.is_exhaustive);
        assert!(result.missing_patterns.contains(&"false".to_string()));
    }

    #[test]
    fn test_exhaustive_with_wildcard() {
        let mut checker = ExhaustivenessChecker::new();
        let arms = vec![make_arm(make_int_pattern(0)), make_arm(make_wildcard())];
        let result = checker.check_match(&ResolvedType::I64, &arms);
        assert!(result.is_exhaustive);
    }

    #[test]
    fn test_non_exhaustive_int_match() {
        let mut checker = ExhaustivenessChecker::new();
        let arms = vec![make_arm(make_int_pattern(0)), make_arm(make_int_pattern(1))];
        let result = checker.check_match(&ResolvedType::I64, &arms);
        assert!(!result.is_exhaustive);
    }

    #[test]
    fn test_unreachable_arm() {
        let mut checker = ExhaustivenessChecker::new();
        let arms = vec![
            make_arm(make_wildcard()),
            make_arm(make_int_pattern(0)), // This is unreachable
        ];
        let result = checker.check_match(&ResolvedType::I64, &arms);
        assert!(result.is_exhaustive);
        assert_eq!(result.unreachable_arms, vec![1]);
    }
}
