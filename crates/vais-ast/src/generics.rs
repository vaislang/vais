//! Generic parameters, variance, and where predicates

use crate::ast_types::Type;
use crate::infrastructure::Spanned;

/// Variance annotation for generic type parameters
/// Controls subtyping relationship between parameterized types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variance {
    /// Invariant (default): T is neither covariant nor contravariant
    /// Container<A> has no subtyping relation with Container<B>
    #[default]
    Invariant,
    /// Covariant (+T): if A <: B then Container<A> <: Container<B>
    /// Used for read-only/producer types (e.g., Iterator, Supplier)
    Covariant,
    /// Contravariant (-T): if A <: B then Container<B> <: Container<A>
    /// Used for write-only/consumer types (e.g., Predicate, Consumer)
    Contravariant,
}

/// Generic parameter kind - either a type parameter, const parameter, lifetime parameter,
/// or higher-kinded type parameter
#[derive(Debug, Clone, PartialEq)]
pub enum GenericParamKind {
    /// Type parameter with optional trait bounds (e.g., T, T: Display + Clone)
    Type { bounds: Vec<Spanned<String>> },
    /// Const parameter with a type (e.g., const N: u64)
    Const { ty: Spanned<Type> },
    /// Lifetime parameter (e.g., 'a, 'static)
    Lifetime {
        /// Lifetime bounds (e.g., 'a: 'b means 'a outlives 'b)
        bounds: Vec<String>,
    },
    /// Higher-kinded type parameter (e.g., F<_> or F<_, _>)
    /// Represents a type constructor that takes `arity` type arguments.
    /// Example: `F<_>` has arity 1, `F<_, _>` has arity 2
    HigherKinded {
        /// Number of type arguments this constructor takes
        arity: usize,
        /// Optional trait bounds on the type constructor
        bounds: Vec<Spanned<String>>,
    },
}

/// Generic parameter with optional trait bounds and variance annotation
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Spanned<String>,
    pub bounds: Vec<Spanned<String>>, // Trait constraints (e.g., T: Display + Clone) - kept for backward compatibility
    pub kind: GenericParamKind,
    pub variance: Variance, // Variance annotation: Invariant (default), Covariant (+), Contravariant (-)
}

impl GenericParam {
    /// Create a type generic parameter (backward compatible constructor)
    pub fn new_type(name: Spanned<String>, bounds: Vec<Spanned<String>>) -> Self {
        Self {
            name,
            bounds: bounds.clone(),
            kind: GenericParamKind::Type { bounds },
            variance: Variance::Invariant,
        }
    }

    /// Create a type generic parameter with variance annotation
    pub fn new_type_with_variance(
        name: Spanned<String>,
        bounds: Vec<Spanned<String>>,
        variance: Variance,
    ) -> Self {
        Self {
            name,
            bounds: bounds.clone(),
            kind: GenericParamKind::Type { bounds },
            variance,
        }
    }

    /// Create a const generic parameter
    pub fn new_const(name: Spanned<String>, ty: Spanned<Type>) -> Self {
        Self {
            name,
            bounds: vec![],
            kind: GenericParamKind::Const { ty },
            variance: Variance::Invariant,
        }
    }

    /// Create a lifetime generic parameter (e.g., 'a)
    pub fn new_lifetime(name: Spanned<String>, bounds: Vec<String>) -> Self {
        Self {
            name,
            bounds: vec![],
            kind: GenericParamKind::Lifetime { bounds },
            variance: Variance::Invariant,
        }
    }

    /// Create a higher-kinded type parameter (e.g., F<_>)
    pub fn new_higher_kinded(
        name: Spanned<String>,
        arity: usize,
        bounds: Vec<Spanned<String>>,
    ) -> Self {
        Self {
            name,
            bounds: vec![],
            kind: GenericParamKind::HigherKinded { arity, bounds },
            variance: Variance::Invariant,
        }
    }

    /// Check if this is a const generic parameter
    pub fn is_const(&self) -> bool {
        matches!(self.kind, GenericParamKind::Const { .. })
    }

    /// Check if this is a higher-kinded type parameter
    pub fn is_higher_kinded(&self) -> bool {
        matches!(self.kind, GenericParamKind::HigherKinded { .. })
    }

    /// Check if this parameter is covariant
    pub fn is_covariant(&self) -> bool {
        matches!(self.variance, Variance::Covariant)
    }

    /// Check if this parameter is contravariant
    pub fn is_contravariant(&self) -> bool {
        matches!(self.variance, Variance::Contravariant)
    }
}

/// Where clause predicate: `T: Display + Clone`
#[derive(Debug, Clone, PartialEq)]
pub struct WherePredicate {
    /// The type being constrained (usually a generic parameter name)
    pub ty: Spanned<String>,
    /// Trait bounds on this type
    pub bounds: Vec<Spanned<String>>,
}
