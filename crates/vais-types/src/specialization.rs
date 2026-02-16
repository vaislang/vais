//! Negative trait implementations and trait specialization
//!
//! This module provides:
//! - Negative impl tracking: marking that a type does NOT implement a trait
//! - Impl overlap detection: identifying when two impl blocks could apply to the same type
//! - Specialization rules: selecting the most specific impl when overlap occurs

/// Represents a negative trait implementation
/// Example: `impl !Send for MyType` means MyType explicitly does NOT implement Send
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeImpl {
    pub trait_name: String,
    pub type_name: String,
}

/// Information about a trait implementation for overlap detection and specialization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitImplInfo {
    pub trait_name: String,
    pub impl_type: ImplTargetType,
    pub is_negative: bool,
    pub methods: Vec<String>,
}

/// The target type of an impl block, categorized by specificity
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImplTargetType {
    /// Concrete type: impl Foo for i64
    /// Most specific - applies to exactly one type
    Concrete(String),

    /// Generic with bounds: impl<T: Bar> Foo for T
    /// More specific than unbounded - applies to types satisfying bounds
    Generic(String, Vec<String>),

    /// Blanket impl: impl<T> Foo for T
    /// Least specific - applies to all types
    Blanket(String),
}

/// Errors that can occur during overlap detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlapError {
    /// Two positive impls conflict
    ConflictingImpls {
        trait_name: String,
        type1: String,
        type2: String,
    },

    /// A negative impl conflicts with a positive impl
    NegativeImplConflict {
        trait_name: String,
        type_name: String,
    },
}

impl std::fmt::Display for OverlapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverlapError::ConflictingImpls {
                trait_name,
                type1,
                type2,
            } => {
                write!(
                    f,
                    "Conflicting implementations of trait '{}' for types '{}' and '{}'",
                    trait_name, type1, type2
                )
            }
            OverlapError::NegativeImplConflict {
                trait_name,
                type_name,
            } => {
                write!(
                    f,
                    "Negative impl conflicts with positive impl: trait '{}' for type '{}'",
                    trait_name, type_name
                )
            }
        }
    }
}

impl std::error::Error for OverlapError {}

/// Check if two impl blocks overlap (could both apply to the same type)
///
/// Two impls overlap if there exists a type that both could apply to.
/// Examples:
/// - `impl Foo for i64` and `impl<T> Foo for T` overlap (both apply to i64)
/// - `impl Foo for i64` and `impl Foo for f64` do NOT overlap (disjoint types)
/// - `impl<T: Bar> Foo for T` and `impl<T> Foo for T` overlap (all Bar types)
pub fn check_impl_overlap(
    existing_impls: &[TraitImplInfo],
    new_impl: &TraitImplInfo,
) -> Option<OverlapError> {
    for existing in existing_impls {
        // Only check overlap for the same trait
        if existing.trait_name != new_impl.trait_name {
            continue;
        }

        // Check for negative impl conflict
        if existing.is_negative != new_impl.is_negative {
            if let Some(_overlap_type) = types_overlap(&existing.impl_type, &new_impl.impl_type) {
                return Some(OverlapError::NegativeImplConflict {
                    trait_name: existing.trait_name.clone(),
                    type_name: format_impl_type(&existing.impl_type),
                });
            }
        }

        // Check for positive impl overlap (only if both are positive)
        if !existing.is_negative
            && !new_impl.is_negative
            && types_overlap(&existing.impl_type, &new_impl.impl_type).is_some()
        {
            // Overlap is allowed if one impl is more specific (specialization)
            // But we still return the error with information about the overlap
            // The caller can decide whether to allow it based on specialization rules
            return Some(OverlapError::ConflictingImpls {
                trait_name: existing.trait_name.clone(),
                type1: format_impl_type(&existing.impl_type),
                type2: format_impl_type(&new_impl.impl_type),
            });
        }
    }

    None
}

/// Determine if two impl target types overlap
/// Returns Some(type_name) if they overlap, None otherwise
fn types_overlap(type1: &ImplTargetType, type2: &ImplTargetType) -> Option<String> {
    use ImplTargetType::*;

    match (type1, type2) {
        // Same concrete types always overlap
        (Concrete(t1), Concrete(t2)) if t1 == t2 => Some(t1.clone()),

        // Different concrete types never overlap
        (Concrete(_), Concrete(_)) => None,

        // Concrete type overlaps with any generic/blanket
        (Concrete(t), Generic(_, _)) | (Generic(_, _), Concrete(t)) => Some(t.clone()),
        (Concrete(t), Blanket(_)) | (Blanket(_), Concrete(t)) => Some(t.clone()),

        // Generic and blanket always overlap (blanket includes everything)
        (Generic(_, _), Blanket(_)) | (Blanket(_), Generic(_, _)) => Some("T".to_string()),

        // Two generics or two blankets overlap
        (Generic(_, _), Generic(_, _)) => Some("T".to_string()),
        (Blanket(_), Blanket(_)) => Some("T".to_string()),
    }
}

/// Format an impl target type for display in error messages
fn format_impl_type(impl_type: &ImplTargetType) -> String {
    match impl_type {
        ImplTargetType::Concrete(name) => name.clone(),
        ImplTargetType::Generic(param, bounds) if bounds.is_empty() => param.to_string(),
        ImplTargetType::Generic(param, bounds) => {
            format!("{}: {}", param, bounds.join(" + "))
        }
        ImplTargetType::Blanket(param) => param.clone(),
    }
}

/// Resolve which impl to use when multiple impls apply to a type
///
/// Uses specialization rules:
/// 1. Concrete types are most specific
/// 2. Generics with bounds are more specific than unbounded
/// 3. Unbounded generics (blanket impls) are least specific
///
/// Returns the most specific impl, or None if no impls apply
pub fn resolve_specialization<'a>(
    impls: &'a [TraitImplInfo],
    target_type: &str,
) -> Option<&'a TraitImplInfo> {
    let mut candidates: Vec<&TraitImplInfo> = impls
        .iter()
        .filter(|impl_info| {
            !impl_info.is_negative && type_matches(&impl_info.impl_type, target_type)
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Sort by specificity (most specific first)
    candidates.sort_by(|a, b| {
        let spec_a = get_specificity(&a.impl_type);
        let spec_b = get_specificity(&b.impl_type);
        spec_b.cmp(&spec_a) // Reverse order: higher specificity first
    });

    Some(candidates[0])
}

/// Check if an impl type matches a concrete target type
fn type_matches(impl_type: &ImplTargetType, target_type: &str) -> bool {
    match impl_type {
        ImplTargetType::Concrete(t) => t == target_type,
        ImplTargetType::Generic(_, _) => true, // Generics match all types (bounds checked elsewhere)
        ImplTargetType::Blanket(_) => true,    // Blanket impls match all types
    }
}

/// Get the specificity level of an impl type
/// Higher values = more specific
fn get_specificity(impl_type: &ImplTargetType) -> u32 {
    match impl_type {
        ImplTargetType::Concrete(_) => 3, // Most specific
        ImplTargetType::Generic(_, bounds) if !bounds.is_empty() => 2, // Bounded generic
        ImplTargetType::Generic(_, _) => 1, // Unbounded generic
        ImplTargetType::Blanket(_) => 0,  // Least specific
    }
}

/// Registry for tracking trait implementations including negative impls
#[derive(Debug, Clone, Default)]
pub struct ImplRegistry {
    impls: Vec<TraitImplInfo>,
    negative_impls: Vec<NegativeImpl>,
}

impl ImplRegistry {
    /// Create a new empty impl registry
    pub fn new() -> Self {
        Self {
            impls: Vec::new(),
            negative_impls: Vec::new(),
        }
    }

    /// Register a new trait implementation
    ///
    /// Returns an error if the impl conflicts with existing impls
    pub fn register_impl(&mut self, impl_info: TraitImplInfo) -> Result<(), OverlapError> {
        // Check for overlaps with existing impls
        if let Some(err) = check_impl_overlap(&self.impls, &impl_info) {
            // For now, we reject overlapping impls
            // In the future, we could allow them with explicit specialization
            return Err(err);
        }

        // If it's a negative impl, also track it separately
        if impl_info.is_negative {
            if let ImplTargetType::Concrete(ref type_name) = impl_info.impl_type {
                self.negative_impls.push(NegativeImpl {
                    trait_name: impl_info.trait_name.clone(),
                    type_name: type_name.clone(),
                });
            }
        }

        self.impls.push(impl_info);
        Ok(())
    }

    /// Check if a type implements a trait
    ///
    /// Returns true if there's a positive impl, false if there's a negative impl or no impl
    pub fn type_implements_trait(&self, type_name: &str, trait_name: &str) -> bool {
        // First check negative impls
        for neg in &self.negative_impls {
            if neg.type_name == type_name && neg.trait_name == trait_name {
                return false;
            }
        }

        // Then check for positive impls
        self.impls.iter().any(|impl_info| {
            impl_info.trait_name == trait_name
                && !impl_info.is_negative
                && type_matches(&impl_info.impl_type, type_name)
        })
    }

    /// Resolve which impl applies to a specific type
    ///
    /// Uses specialization rules to select the most specific impl
    pub fn resolve_impl(&self, type_name: &str, trait_name: &str) -> Option<&TraitImplInfo> {
        // Early return if no matching trait impls exist
        if !self.impls.iter().any(|impl_info| impl_info.trait_name == trait_name) {
            return None;
        }

        // Find the most specific impl
        let mut candidates: Vec<&TraitImplInfo> = self
            .impls
            .iter()
            .filter(|impl_info| {
                impl_info.trait_name == trait_name
                    && !impl_info.is_negative
                    && type_matches(&impl_info.impl_type, type_name)
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by specificity (most specific first)
        candidates.sort_by(|a, b| {
            let spec_a = get_specificity(&a.impl_type);
            let spec_b = get_specificity(&b.impl_type);
            spec_b.cmp(&spec_a) // Reverse order: higher specificity first
        });

        Some(candidates[0])
    }

    /// Get all registered impls
    pub fn impls(&self) -> &[TraitImplInfo] {
        &self.impls
    }

    /// Get all negative impls
    pub fn negative_impls(&self) -> &[NegativeImpl] {
        &self.negative_impls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_overlap_different_concrete_types() {
        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("f64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        assert!(check_impl_overlap(&[impl1], &impl2).is_none());
    }

    #[test]
    fn test_overlap_concrete_and_blanket() {
        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Blanket("T".to_string()),
            is_negative: false,
            methods: vec![],
        };

        assert!(check_impl_overlap(&[impl1], &impl2).is_some());
    }

    #[test]
    fn test_overlap_concrete_and_generic() {
        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Generic("T".to_string(), vec!["Bar".to_string()]),
            is_negative: false,
            methods: vec![],
        };

        assert!(check_impl_overlap(&[impl1], &impl2).is_some());
    }

    #[test]
    fn test_specialization_concrete_wins_over_generic() {
        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec!["concrete_method".to_string()],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Blanket("T".to_string()),
            is_negative: false,
            methods: vec!["generic_method".to_string()],
        };

        let impls = vec![impl1.clone(), impl2];
        let result = resolve_specialization(&impls, "i64");

        assert!(result.is_some());
        assert_eq!(result.unwrap().methods, vec!["concrete_method"]);
    }

    #[test]
    fn test_specialization_bounded_wins_over_unbounded() {
        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Generic("T".to_string(), vec!["Bar".to_string()]),
            is_negative: false,
            methods: vec!["bounded_method".to_string()],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Blanket("T".to_string()),
            is_negative: false,
            methods: vec!["unbounded_method".to_string()],
        };

        let impls = vec![impl1.clone(), impl2];
        let result = resolve_specialization(&impls, "SomeType");

        assert!(result.is_some());
        assert_eq!(result.unwrap().methods, vec!["bounded_method"]);
    }

    #[test]
    fn test_negative_impl_conflict() {
        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: true,
            methods: vec![],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        let error = check_impl_overlap(&[impl1], &impl2);
        assert!(error.is_some());
        assert!(matches!(
            error.unwrap(),
            OverlapError::NegativeImplConflict { .. }
        ));
    }

    #[test]
    fn test_impl_registry_basic() {
        let mut registry = ImplRegistry::new();

        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        assert!(registry.register_impl(impl1).is_ok());
        assert!(registry.type_implements_trait("i64", "Foo"));
        assert!(!registry.type_implements_trait("f64", "Foo"));
    }

    #[test]
    fn test_impl_registry_negative_impl() {
        let mut registry = ImplRegistry::new();

        let neg_impl = TraitImplInfo {
            trait_name: "Send".to_string(),
            impl_type: ImplTargetType::Concrete("MyType".to_string()),
            is_negative: true,
            methods: vec![],
        };

        assert!(registry.register_impl(neg_impl).is_ok());
        assert!(!registry.type_implements_trait("MyType", "Send"));
    }

    #[test]
    fn test_impl_registry_overlap_rejection() {
        let mut registry = ImplRegistry::new();

        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Blanket("T".to_string()),
            is_negative: false,
            methods: vec![],
        };

        assert!(registry.register_impl(impl1).is_ok());
        assert!(registry.register_impl(impl2).is_err());
    }

    #[test]
    fn test_impl_registry_resolve() {
        let mut registry = ImplRegistry::new();

        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec!["specific".to_string()],
        };

        assert!(registry.register_impl(impl1).is_ok());

        let resolved = registry.resolve_impl("i64", "Foo");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().methods, vec!["specific"]);
    }

    #[test]
    fn test_multiple_impls_different_specificity() {
        // This test demonstrates the ideal case where we would allow
        // multiple impls with different specificity levels
        let impl1 = TraitImplInfo {
            trait_name: "Display".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec!["display_i64".to_string()],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Display".to_string(),
            impl_type: ImplTargetType::Generic("T".to_string(), vec!["Debug".to_string()]),
            is_negative: false,
            methods: vec!["display_debug".to_string()],
        };

        let impl3 = TraitImplInfo {
            trait_name: "Display".to_string(),
            impl_type: ImplTargetType::Blanket("T".to_string()),
            is_negative: false,
            methods: vec!["display_default".to_string()],
        };

        // When resolving for i64, should pick the concrete impl
        let impls = vec![impl1.clone(), impl2.clone(), impl3.clone()];
        let result = resolve_specialization(&impls, "i64");
        assert!(result.is_some());
        assert_eq!(result.unwrap().methods, vec!["display_i64"]);

        // When resolving for a generic type, should pick bounded over unbounded
        let result = resolve_specialization(&impls, "SomeDebugType");
        assert!(result.is_some());
        assert_eq!(result.unwrap().methods, vec!["display_debug"]);
    }

    #[test]
    fn test_no_overlap_different_traits() {
        let impl1 = TraitImplInfo {
            trait_name: "Foo".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        let impl2 = TraitImplInfo {
            trait_name: "Bar".to_string(),
            impl_type: ImplTargetType::Concrete("i64".to_string()),
            is_negative: false,
            methods: vec![],
        };

        assert!(check_impl_overlap(&[impl1], &impl2).is_none());
    }
}
