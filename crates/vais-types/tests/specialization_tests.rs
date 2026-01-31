//! Tests for trait specialization and negative implementations

use vais_types::specialization::*;

#[test]
fn test_no_overlap_between_concrete_impls_different_types() {
    // Two concrete impls for different types should not overlap
    let impl1 = TraitImplInfo {
        trait_name: "Display".to_string(),
        impl_type: ImplTargetType::Concrete("i64".to_string()),
        is_negative: false,
        methods: vec!["display_i64".to_string()],
    };

    let impl2 = TraitImplInfo {
        trait_name: "Display".to_string(),
        impl_type: ImplTargetType::Concrete("f64".to_string()),
        is_negative: false,
        methods: vec!["display_f64".to_string()],
    };

    let result = check_impl_overlap(&[impl1], &impl2);
    assert!(result.is_none(), "Concrete impls for different types should not overlap");
}

#[test]
fn test_overlap_detection_blanket_and_concrete() {
    // Blanket impl overlaps with concrete impl for the same trait
    let blanket = TraitImplInfo {
        trait_name: "Clone".to_string(),
        impl_type: ImplTargetType::Blanket("T".to_string()),
        is_negative: false,
        methods: vec!["clone_generic".to_string()],
    };

    let concrete = TraitImplInfo {
        trait_name: "Clone".to_string(),
        impl_type: ImplTargetType::Concrete("MyType".to_string()),
        is_negative: false,
        methods: vec!["clone_mytype".to_string()],
    };

    let result = check_impl_overlap(&[blanket], &concrete);
    assert!(result.is_some(), "Blanket and concrete impls should overlap");

    if let Some(OverlapError::ConflictingImpls { trait_name, .. }) = result {
        assert_eq!(trait_name, "Clone");
    } else {
        panic!("Expected ConflictingImpls error");
    }
}

#[test]
fn test_overlap_detection_generic_and_concrete() {
    // Generic impl with bounds overlaps with concrete impl
    let generic = TraitImplInfo {
        trait_name: "Display".to_string(),
        impl_type: ImplTargetType::Generic("T".to_string(), vec!["Debug".to_string()]),
        is_negative: false,
        methods: vec!["display_debug".to_string()],
    };

    let concrete = TraitImplInfo {
        trait_name: "Display".to_string(),
        impl_type: ImplTargetType::Concrete("i64".to_string()),
        is_negative: false,
        methods: vec!["display_i64".to_string()],
    };

    let result = check_impl_overlap(&[generic], &concrete);
    assert!(result.is_some(), "Generic and concrete impls should overlap");
}

#[test]
fn test_specialization_concrete_wins_over_generic() {
    // When resolving, concrete impl should be selected over generic
    let concrete = TraitImplInfo {
        trait_name: "Foo".to_string(),
        impl_type: ImplTargetType::Concrete("i64".to_string()),
        is_negative: false,
        methods: vec!["concrete_method".to_string()],
    };

    let generic = TraitImplInfo {
        trait_name: "Foo".to_string(),
        impl_type: ImplTargetType::Generic("T".to_string(), vec!["Bar".to_string()]),
        is_negative: false,
        methods: vec!["generic_method".to_string()],
    };

    let blanket = TraitImplInfo {
        trait_name: "Foo".to_string(),
        impl_type: ImplTargetType::Blanket("T".to_string()),
        is_negative: false,
        methods: vec!["blanket_method".to_string()],
    };

    let impls = vec![blanket, generic, concrete.clone()];
    let result = resolve_specialization(&impls, "i64");

    assert!(result.is_some());
    let resolved = result.unwrap();
    assert_eq!(resolved.methods, vec!["concrete_method"]);
    assert!(matches!(resolved.impl_type, ImplTargetType::Concrete(_)));
}

#[test]
fn test_specialization_bounded_wins_over_unbounded() {
    // Bounded generic should be selected over unbounded blanket impl
    let bounded = TraitImplInfo {
        trait_name: "ToString".to_string(),
        impl_type: ImplTargetType::Generic("T".to_string(), vec!["Display".to_string()]),
        is_negative: false,
        methods: vec!["to_string_display".to_string()],
    };

    let unbounded = TraitImplInfo {
        trait_name: "ToString".to_string(),
        impl_type: ImplTargetType::Blanket("T".to_string()),
        is_negative: false,
        methods: vec!["to_string_default".to_string()],
    };

    let impls = vec![unbounded, bounded.clone()];
    let result = resolve_specialization(&impls, "SomeType");

    assert!(result.is_some());
    let resolved = result.unwrap();
    assert_eq!(resolved.methods, vec!["to_string_display"]);
}

#[test]
fn test_negative_impl_conflict_with_positive() {
    // Negative impl for a type should conflict with positive impl for same type
    let negative = TraitImplInfo {
        trait_name: "Send".to_string(),
        impl_type: ImplTargetType::Concrete("RcCell".to_string()),
        is_negative: true,
        methods: vec![],
    };

    let positive = TraitImplInfo {
        trait_name: "Send".to_string(),
        impl_type: ImplTargetType::Concrete("RcCell".to_string()),
        is_negative: false,
        methods: vec![],
    };

    let result = check_impl_overlap(&[negative], &positive);
    assert!(result.is_some());

    if let Some(OverlapError::NegativeImplConflict { trait_name, type_name }) = result {
        assert_eq!(trait_name, "Send");
        assert_eq!(type_name, "RcCell");
    } else {
        panic!("Expected NegativeImplConflict error");
    }
}

#[test]
fn test_negative_impl_blocks_trait_check() {
    // Type with negative impl should not be considered to implement the trait
    let mut registry = ImplRegistry::new();

    let negative_impl = TraitImplInfo {
        trait_name: "Send".to_string(),
        impl_type: ImplTargetType::Concrete("MyNonSendType".to_string()),
        is_negative: true,
        methods: vec![],
    };

    registry.register_impl(negative_impl).unwrap();

    assert!(!registry.type_implements_trait("MyNonSendType", "Send"));
}

#[test]
fn test_multiple_impls_different_specificity_levels() {
    // Test resolution with three levels of specificity
    let concrete = TraitImplInfo {
        trait_name: "Serialize".to_string(),
        impl_type: ImplTargetType::Concrete("String".to_string()),
        is_negative: false,
        methods: vec!["serialize_string".to_string()],
    };

    let bounded = TraitImplInfo {
        trait_name: "Serialize".to_string(),
        impl_type: ImplTargetType::Generic(
            "T".to_string(),
            vec!["Display".to_string(), "Clone".to_string()],
        ),
        is_negative: false,
        methods: vec!["serialize_displayable".to_string()],
    };

    let unbounded = TraitImplInfo {
        trait_name: "Serialize".to_string(),
        impl_type: ImplTargetType::Blanket("T".to_string()),
        is_negative: false,
        methods: vec!["serialize_any".to_string()],
    };

    // For String, should pick concrete
    let impls = vec![unbounded.clone(), bounded.clone(), concrete.clone()];
    let result = resolve_specialization(&impls, "String");
    assert_eq!(result.unwrap().methods, vec!["serialize_string"]);

    // For other types, should pick bounded over unbounded
    let result = resolve_specialization(&impls, "CustomType");
    assert_eq!(result.unwrap().methods, vec!["serialize_displayable"]);
}

#[test]
fn test_no_overlap_when_types_completely_different() {
    // Verify that different concrete types don't overlap
    let impl1 = TraitImplInfo {
        trait_name: "Hash".to_string(),
        impl_type: ImplTargetType::Concrete("Vec".to_string()),
        is_negative: false,
        methods: vec![],
    };

    let impl2 = TraitImplInfo {
        trait_name: "Hash".to_string(),
        impl_type: ImplTargetType::Concrete("HashMap".to_string()),
        is_negative: false,
        methods: vec![],
    };

    let result = check_impl_overlap(&[impl1], &impl2);
    assert!(result.is_none());
}

#[test]
fn test_impl_registry_workflow() {
    // Test complete workflow: register impls, check traits, resolve impls
    let mut registry = ImplRegistry::new();

    // Register a blanket impl for Clone
    let clone_blanket = TraitImplInfo {
        trait_name: "Clone".to_string(),
        impl_type: ImplTargetType::Blanket("T".to_string()),
        is_negative: false,
        methods: vec!["clone".to_string()],
    };

    assert!(registry.register_impl(clone_blanket).is_ok());

    // Verify any type implements Clone
    assert!(registry.type_implements_trait("i64", "Clone"));
    assert!(registry.type_implements_trait("MyType", "Clone"));

    // Try to register overlapping concrete impl - should fail
    let clone_concrete = TraitImplInfo {
        trait_name: "Clone".to_string(),
        impl_type: ImplTargetType::Concrete("i64".to_string()),
        is_negative: false,
        methods: vec!["clone_i64".to_string()],
    };

    assert!(registry.register_impl(clone_concrete).is_err());
}

#[test]
fn test_impl_registry_different_traits_same_type() {
    // Same type can implement different traits without conflict
    let mut registry = ImplRegistry::new();

    let clone_impl = TraitImplInfo {
        trait_name: "Clone".to_string(),
        impl_type: ImplTargetType::Concrete("MyType".to_string()),
        is_negative: false,
        methods: vec!["clone".to_string()],
    };

    let debug_impl = TraitImplInfo {
        trait_name: "Debug".to_string(),
        impl_type: ImplTargetType::Concrete("MyType".to_string()),
        is_negative: false,
        methods: vec!["debug".to_string()],
    };

    assert!(registry.register_impl(clone_impl).is_ok());
    assert!(registry.register_impl(debug_impl).is_ok());

    assert!(registry.type_implements_trait("MyType", "Clone"));
    assert!(registry.type_implements_trait("MyType", "Debug"));
}

#[test]
fn test_resolve_impl_returns_most_specific() {
    let mut registry = ImplRegistry::new();

    let concrete = TraitImplInfo {
        trait_name: "Eq".to_string(),
        impl_type: ImplTargetType::Concrete("i64".to_string()),
        is_negative: false,
        methods: vec!["eq_i64".to_string()],
    };

    registry.register_impl(concrete).unwrap();

    let resolved = registry.resolve_impl("i64", "Eq");
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().methods, vec!["eq_i64"]);

    // Non-existent type should return None
    let resolved = registry.resolve_impl("f64", "Eq");
    assert!(resolved.is_none());
}

#[test]
fn test_overlap_between_two_blanket_impls() {
    // Two blanket impls for the same trait should overlap
    let blanket1 = TraitImplInfo {
        trait_name: "Default".to_string(),
        impl_type: ImplTargetType::Blanket("T".to_string()),
        is_negative: false,
        methods: vec!["default1".to_string()],
    };

    let blanket2 = TraitImplInfo {
        trait_name: "Default".to_string(),
        impl_type: ImplTargetType::Blanket("U".to_string()),
        is_negative: false,
        methods: vec!["default2".to_string()],
    };

    let result = check_impl_overlap(&[blanket1], &blanket2);
    assert!(result.is_some());
}

#[test]
fn test_overlap_between_two_generics() {
    // Two generic impls with different bounds still overlap
    let generic1 = TraitImplInfo {
        trait_name: "From".to_string(),
        impl_type: ImplTargetType::Generic("T".to_string(), vec!["Clone".to_string()]),
        is_negative: false,
        methods: vec!["from1".to_string()],
    };

    let generic2 = TraitImplInfo {
        trait_name: "From".to_string(),
        impl_type: ImplTargetType::Generic("T".to_string(), vec!["Copy".to_string()]),
        is_negative: false,
        methods: vec!["from2".to_string()],
    };

    let result = check_impl_overlap(&[generic1], &generic2);
    assert!(result.is_some());
}

#[test]
fn test_negative_impl_with_blanket() {
    // Negative concrete impl should conflict with blanket positive impl
    let negative = TraitImplInfo {
        trait_name: "Send".to_string(),
        impl_type: ImplTargetType::Concrete("Rc".to_string()),
        is_negative: true,
        methods: vec![],
    };

    let blanket = TraitImplInfo {
        trait_name: "Send".to_string(),
        impl_type: ImplTargetType::Blanket("T".to_string()),
        is_negative: false,
        methods: vec![],
    };

    let result = check_impl_overlap(&[negative], &blanket);
    assert!(result.is_some());
    assert!(matches!(result.unwrap(), OverlapError::NegativeImplConflict { .. }));
}

#[test]
fn test_resolve_specialization_empty_impls() {
    // Resolving with no impls should return None
    let impls: Vec<TraitImplInfo> = vec![];
    let result = resolve_specialization(&impls, "SomeType");
    assert!(result.is_none());
}

#[test]
fn test_resolve_specialization_only_negative_impls() {
    // Negative impls should be filtered out during resolution
    let negative = TraitImplInfo {
        trait_name: "Send".to_string(),
        impl_type: ImplTargetType::Concrete("MyType".to_string()),
        is_negative: true,
        methods: vec![],
    };

    let impls = vec![negative];
    let result = resolve_specialization(&impls, "MyType");
    assert!(result.is_none(), "Negative impls should not be resolved");
}

#[test]
fn test_specificity_ordering() {
    // Test that specificity values are correctly ordered
    let concrete = ImplTargetType::Concrete("i64".to_string());
    let bounded = ImplTargetType::Generic("T".to_string(), vec!["Clone".to_string()]);
    let unbounded = ImplTargetType::Generic("T".to_string(), vec![]);
    let blanket = ImplTargetType::Blanket("T".to_string());

    use vais_types::specialization::*;

    // Access specificity through resolution (indirect test)
    let impl_concrete = TraitImplInfo {
        trait_name: "Test".to_string(),
        impl_type: concrete,
        is_negative: false,
        methods: vec!["a".to_string()],
    };

    let impl_bounded = TraitImplInfo {
        trait_name: "Test".to_string(),
        impl_type: bounded,
        is_negative: false,
        methods: vec!["b".to_string()],
    };

    let impl_unbounded = TraitImplInfo {
        trait_name: "Test".to_string(),
        impl_type: unbounded,
        is_negative: false,
        methods: vec!["c".to_string()],
    };

    let impl_blanket = TraitImplInfo {
        trait_name: "Test".to_string(),
        impl_type: blanket,
        is_negative: false,
        methods: vec!["d".to_string()],
    };

    // Concrete should win
    let impls = vec![
        impl_blanket.clone(),
        impl_unbounded.clone(),
        impl_bounded.clone(),
        impl_concrete.clone(),
    ];
    let result = resolve_specialization(&impls, "i64");
    assert_eq!(result.unwrap().methods, vec!["a"]);
}

#[test]
fn test_impl_registry_get_all_impls() {
    let mut registry = ImplRegistry::new();

    let impl1 = TraitImplInfo {
        trait_name: "Clone".to_string(),
        impl_type: ImplTargetType::Concrete("i64".to_string()),
        is_negative: false,
        methods: vec![],
    };

    let impl2 = TraitImplInfo {
        trait_name: "Debug".to_string(),
        impl_type: ImplTargetType::Concrete("f64".to_string()),
        is_negative: false,
        methods: vec![],
    };

    registry.register_impl(impl1).unwrap();
    registry.register_impl(impl2).unwrap();

    assert_eq!(registry.impls().len(), 2);
}

#[test]
fn test_impl_registry_get_negative_impls() {
    let mut registry = ImplRegistry::new();

    let neg_impl = TraitImplInfo {
        trait_name: "Send".to_string(),
        impl_type: ImplTargetType::Concrete("MyType".to_string()),
        is_negative: true,
        methods: vec![],
    };

    registry.register_impl(neg_impl).unwrap();

    assert_eq!(registry.negative_impls().len(), 1);
    assert_eq!(registry.negative_impls()[0].trait_name, "Send");
    assert_eq!(registry.negative_impls()[0].type_name, "MyType");
}

#[test]
fn test_format_error_messages() {
    let conflict_err = OverlapError::ConflictingImpls {
        trait_name: "Clone".to_string(),
        type1: "i64".to_string(),
        type2: "T".to_string(),
    };

    let msg = format!("{}", conflict_err);
    assert!(msg.contains("Clone"));
    assert!(msg.contains("i64"));
    assert!(msg.contains("T"));

    let neg_err = OverlapError::NegativeImplConflict {
        trait_name: "Send".to_string(),
        type_name: "Rc".to_string(),
    };

    let msg = format!("{}", neg_err);
    assert!(msg.contains("Send"));
    assert!(msg.contains("Rc"));
    assert!(msg.contains("Negative"));
}
