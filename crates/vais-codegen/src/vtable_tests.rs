//! Tests for VTable generation and dynamic dispatch

#[cfg(test)]
mod tests {
    use crate::{vtable::*, CodeGenerator};
    use std::collections::HashMap;
    use vais_types::{ResolvedType, TraitDef, TraitMethodSig};

    fn create_drawable_trait() -> TraitDef {
        let mut methods = HashMap::new();

        methods.insert(
            "draw".to_string(),
            TraitMethodSig {
                name: "draw".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::Unit,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        methods.insert(
            "area".to_string(),
            TraitMethodSig {
                name: "area".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::I64,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        TraitDef {
            name: "Drawable".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        }
    }

    #[test]
    fn test_vtable_generator_new() {
        let gen = VtableGenerator::new();
        assert_eq!(gen.get_vtables().count(), 0);
    }

    #[test]
    fn test_vtable_generation_with_all_methods() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_drawable_trait();

        let mut impls = HashMap::new();
        impls.insert("draw".to_string(), "Circle_draw".to_string());
        impls.insert("area".to_string(), "Circle_area".to_string());

        let vtable = gen.generate_vtable("Circle", &trait_def, &impls).expect("vtable generation failed");

        assert_eq!(vtable.trait_name, "Drawable");
        assert_eq!(vtable.impl_type, "Circle");
        assert_eq!(vtable.methods.len(), 2);

        // Verify method names are correctly mapped
        let method_map: HashMap<_, _> = vtable.methods.iter().cloned().collect();
        assert_eq!(method_map.get("draw"), Some(&"Circle_draw".to_string()));
        assert_eq!(method_map.get("area"), Some(&"Circle_area".to_string()));
    }

    #[test]
    fn test_vtable_generation_with_missing_method() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_drawable_trait();

        // Only implement 'draw', not 'area'
        let mut impls = HashMap::new();
        impls.insert("draw".to_string(), "Square_draw".to_string());

        let result = gen.generate_vtable("Square", &trait_def, &impls);

        // Missing required method should result in compile-time error
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Trait `Drawable` method `area` not implemented for type `Square`"));
    }

    #[test]
    fn test_vtable_caching() {
        let mut gen = VtableGenerator::new();
        let mut trait_def = create_drawable_trait();

        // Add default implementations to avoid errors
        for method_sig in trait_def.methods.values_mut() {
            method_sig.has_default = true;
        }

        let impls = HashMap::new();

        let vtable1 = gen.generate_vtable("Triangle", &trait_def, &impls).expect("vtable generation failed");
        let vtable2 = gen.generate_vtable("Triangle", &trait_def, &impls).expect("vtable generation failed");

        assert_eq!(vtable1.global_name, vtable2.global_name);
        assert_eq!(gen.get_vtables().count(), 1);
    }

    #[test]
    fn test_vtable_struct_type() {
        let trait_def = create_drawable_trait();
        let vtable_type = VtableGenerator::vtable_struct_type(&trait_def);

        // Should contain drop, size, align, and 2 method pointers
        assert!(vtable_type.starts_with("{ "));
        assert!(vtable_type.ends_with(" }"));
        assert!(vtable_type.contains("i8*")); // drop
        assert!(vtable_type.contains("i64")); // size and align
    }

    #[test]
    fn test_vtable_global_generation() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_drawable_trait();

        let mut impls = HashMap::new();
        impls.insert("draw".to_string(), "Rectangle_draw".to_string());
        impls.insert("area".to_string(), "Rectangle_area".to_string());

        let vtable = gen.generate_vtable("Rectangle", &trait_def, &impls).expect("vtable generation failed");
        let global_ir = gen.generate_vtable_global(&vtable, &trait_def, 16, 8);

        assert!(global_ir.contains("@vtable_Rectangle_Drawable"));
        assert!(global_ir.contains("internal constant"));
    }

    #[test]
    fn test_trait_object_type() {
        assert_eq!(TRAIT_OBJECT_TYPE, "{ i8*, i8* }");
    }

    #[test]
    fn test_trait_object_creation() {
        let gen = VtableGenerator::new();
        let _trait_def = create_drawable_trait();

        // Create mock vtable info
        let vtable_info = VtableInfo {
            trait_name: "Drawable".to_string(),
            impl_type: "Circle".to_string(),
            global_name: "@vtable_Circle_Drawable".to_string(),
            methods: vec![
                ("draw".to_string(), "Circle_draw".to_string()),
                ("area".to_string(), "Circle_area".to_string()),
            ],
        };

        let mut counter = 0;
        let (ir, result) =
            gen.create_trait_object("%circle_val", "i64", &vtable_info, &mut counter);

        // Should allocate memory and create fat pointer
        assert!(ir.contains("@malloc"));
        assert!(ir.contains("insertvalue"));
        assert!(result.starts_with("%trait_obj_"));
    }

    #[test]
    fn test_dynamic_call_generation() {
        let gen = VtableGenerator::new();
        let trait_def = create_drawable_trait();

        let mut counter = 0;
        let (ir, _result) = gen.generate_dynamic_call(
            "%dyn_drawable",
            0, // draw method index
            &[],
            "void",
            &trait_def,
            &mut counter,
        );

        // Should extract vtable, load function pointer, and call
        assert!(ir.contains("extractvalue"));
        assert!(ir.contains("getelementptr"));
        assert!(ir.contains("call"));
    }

    #[test]
    fn test_codegen_vtable_integration() {
        let mut gen = CodeGenerator::new("test");

        let trait_def = create_drawable_trait();
        gen.register_trait(trait_def);

        let mut method_impls = HashMap::new();
        method_impls.insert("draw".to_string(), "Circle_draw".to_string());
        method_impls.insert("area".to_string(), "Circle_area".to_string());

        gen.register_trait_impl("Circle", "Drawable", method_impls);

        let vtable = gen.get_or_generate_vtable("Circle", "Drawable");
        assert!(vtable.is_ok());

        let vtable = vtable.unwrap();
        assert_eq!(vtable.trait_name, "Drawable");
        assert_eq!(vtable.impl_type, "Circle");
    }

    #[test]
    fn test_codegen_generate_vtable_globals() {
        let mut gen = CodeGenerator::new("test");

        let trait_def = create_drawable_trait();
        gen.register_trait(trait_def);

        let mut method_impls = HashMap::new();
        method_impls.insert("draw".to_string(), "Ellipse_draw".to_string());
        method_impls.insert("area".to_string(), "Ellipse_area".to_string());

        gen.register_trait_impl("Ellipse", "Drawable", method_impls);

        // Generate vtable first
        let _ = gen.get_or_generate_vtable("Ellipse", "Drawable");

        let globals = gen.generate_vtable_globals();
        assert!(globals.contains("@vtable_Ellipse_Drawable"));
    }

    #[test]
    fn test_dyn_trait_size() {
        let gen = CodeGenerator::new("test");
        let dyn_type = ResolvedType::DynTrait {
            trait_name: "Drawable".to_string(),
            generics: vec![],
        };

        assert_eq!(gen._type_size(&dyn_type), 16); // Fat pointer: 8 + 8
    }

    #[test]
    fn test_multiple_traits_same_type() {
        let mut gen = VtableGenerator::new();

        // Create two different traits
        let trait1 = create_drawable_trait();

        let mut methods2 = HashMap::new();
        methods2.insert(
            "describe".to_string(),
            TraitMethodSig {
                name: "describe".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::Str,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        let trait2 = TraitDef {
            name: "Describable".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods: methods2,
        };

        // Same type implements both traits
        let impls1 = HashMap::from([
            ("draw".to_string(), "Widget_draw".to_string()),
            ("area".to_string(), "Widget_area".to_string()),
        ]);
        let impls2 = HashMap::from([("describe".to_string(), "Widget_describe".to_string())]);

        let vtable1 = gen.generate_vtable("Widget", &trait1, &impls1).expect("vtable1 generation failed");
        let vtable2 = gen.generate_vtable("Widget", &trait2, &impls2).expect("vtable2 generation failed");

        // Should generate two different vtables
        assert_ne!(vtable1.global_name, vtable2.global_name);
        assert_eq!(gen.get_vtables().count(), 2);
    }

    /// Create a trait with async methods for testing
    fn create_async_service_trait() -> TraitDef {
        let mut methods = HashMap::new();

        // Async method: fetch returns Future<i64>
        methods.insert(
            "fetch".to_string(),
            TraitMethodSig {
                name: "fetch".to_string(),
                params: vec![
                    (
                        "self".to_string(),
                        ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                        false,
                    ),
                    ("url".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                has_default: false,
                is_async: true,
                is_const: false,
            },
        );

        // Sync method
        methods.insert(
            "name".to_string(),
            TraitMethodSig {
                name: "name".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::Str,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        TraitDef {
            name: "Service".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        }
    }

    #[test]
    fn test_async_trait_vtable_generation() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_async_service_trait();

        let mut impls = HashMap::new();
        impls.insert("fetch".to_string(), "HttpService_fetch".to_string());
        impls.insert("name".to_string(), "HttpService_name".to_string());

        let vtable = gen.generate_vtable("HttpService", &trait_def, &impls).expect("vtable generation failed");

        assert_eq!(vtable.trait_name, "Service");
        assert_eq!(vtable.impl_type, "HttpService");
        assert_eq!(vtable.methods.len(), 2);

        let method_map: HashMap<_, _> = vtable.methods.iter().cloned().collect();
        assert_eq!(
            method_map.get("fetch"),
            Some(&"HttpService_fetch".to_string())
        );
        assert_eq!(
            method_map.get("name"),
            Some(&"HttpService_name".to_string())
        );
    }

    #[test]
    fn test_async_trait_vtable_struct_type() {
        let trait_def = create_async_service_trait();
        let vtable_type = VtableGenerator::vtable_struct_type(&trait_def);

        // Async method 'fetch' should have i64 return (Future handle)
        // Sync method 'name' should have i64 return (Str mapped to i64)
        assert!(vtable_type.starts_with("{ "));
        assert!(vtable_type.contains("i8*")); // drop
        assert!(vtable_type.contains("i64")); // size/align and method returns
    }

    #[test]
    fn test_async_trait_vtable_global() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_async_service_trait();

        let mut impls = HashMap::new();
        impls.insert("fetch".to_string(), "HttpService_fetch".to_string());
        impls.insert("name".to_string(), "HttpService_name".to_string());

        let vtable = gen.generate_vtable("HttpService", &trait_def, &impls).expect("vtable generation failed");
        let global_ir = gen.generate_vtable_global(&vtable, &trait_def, 8, 8);

        assert!(global_ir.contains("@vtable_HttpService_Service"));
        assert!(global_ir.contains("internal constant"));
        assert!(global_ir.contains("@HttpService_fetch"));
        assert!(global_ir.contains("@HttpService_name"));
    }

    #[test]
    fn test_async_trait_dynamic_call() {
        let gen = VtableGenerator::new();
        let trait_def = create_async_service_trait();

        let mut counter = 0;
        // Call async method 'fetch' through dynamic dispatch
        let (ir, result) = gen.generate_dynamic_call(
            "%dyn_service",
            0, // fetch method index
            &["%url_arg".to_string()],
            "i64", // async methods return i64 (Future handle)
            &trait_def,
            &mut counter,
        );

        assert!(ir.contains("extractvalue"));
        assert!(ir.contains("getelementptr"));
        assert!(ir.contains("call"));
        assert!(!result.is_empty()); // Should have a result (Future handle)
    }

    #[test]
    fn test_mixed_async_sync_trait() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_async_service_trait();

        let mut impls = HashMap::new();
        impls.insert("fetch".to_string(), "MyService_fetch".to_string());
        impls.insert("name".to_string(), "MyService_name".to_string());

        let vtable = gen.generate_vtable("MyService", &trait_def, &impls).expect("vtable generation failed");

        // Both methods should be present
        assert_eq!(vtable.methods.len(), 2);

        // Verify vtable global generation works with mixed methods
        let global_ir = gen.generate_vtable_global(&vtable, &trait_def, 8, 8);
        assert!(global_ir.contains("@vtable_MyService_Service"));
    }

    #[test]
    fn test_async_trait_codegen_integration() {
        let mut gen = CodeGenerator::new("test_async");

        let trait_def = create_async_service_trait();
        gen.register_trait(trait_def);

        let mut method_impls = HashMap::new();
        method_impls.insert("fetch".to_string(), "ApiClient_fetch".to_string());
        method_impls.insert("name".to_string(), "ApiClient_name".to_string());

        gen.register_trait_impl("ApiClient", "Service", method_impls);

        let vtable = gen.get_or_generate_vtable("ApiClient", "Service");
        assert!(vtable.is_ok());

        let vtable = vtable.unwrap();
        assert_eq!(vtable.trait_name, "Service");
        assert_eq!(vtable.impl_type, "ApiClient");
    }

    #[test]
    fn test_empty_trait() {
        let mut gen = VtableGenerator::new();

        let empty_trait = TraitDef {
            name: "Empty".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods: HashMap::new(),
        };

        let impls = HashMap::new();
        let vtable = gen.generate_vtable("AnyType", &empty_trait, &impls).expect("vtable generation failed");

        assert_eq!(vtable.methods.len(), 0);
    }
}
