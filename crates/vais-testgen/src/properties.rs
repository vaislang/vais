//! Property definitions for property-based testing.

/// A property that a function should satisfy.
#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    /// The function should not crash/panic on the given inputs.
    DoesNotCrash,
    /// The function should return a non-zero value.
    ReturnsNonZero,
    /// The function's return value should be in [lo, hi].
    ReturnsInRange(i64, i64),
    /// f(f(x)) == f(x) — applying the function twice gives the same result.
    Idempotent,
    /// f(a, b) == f(b, a) — argument order doesn't matter.
    Commutative,
    /// Custom assertion string.
    Custom(String),
}

impl std::fmt::Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Property::DoesNotCrash => write!(f, "does_not_crash"),
            Property::ReturnsNonZero => write!(f, "returns_non_zero"),
            Property::ReturnsInRange(lo, hi) => write!(f, "returns_in_range({}, {})", lo, hi),
            Property::Idempotent => write!(f, "idempotent"),
            Property::Commutative => write!(f, "commutative"),
            Property::Custom(s) => write!(f, "custom({})", s),
        }
    }
}
