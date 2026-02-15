//! Tests for type_to_llvm caching functionality

#[cfg(test)]
mod tests {
    use crate::CodeGenerator;
    use vais_types::ResolvedType;

    #[test]
    fn test_type_to_llvm_cache_basic_types() {
        let codegen = CodeGenerator::new("test");

        // Test basic types work correctly (they use fast path, no cache)
        let i32_1 = codegen.type_to_llvm(&ResolvedType::I32);
        let i32_2 = codegen.type_to_llvm(&ResolvedType::I32);

        assert_eq!(i32_1, "i32");
        assert_eq!(i32_2, "i32");
        assert_eq!(i32_1, i32_2);

        // Primitive types use fast path and don't populate cache
        let cache_size = codegen.type_to_llvm_cache.borrow().len();
        assert_eq!(
            cache_size, 0,
            "Primitive types should use fast path and not populate cache"
        );
    }

    #[test]
    fn test_type_to_llvm_cache_composite_types() {
        let codegen = CodeGenerator::new("test");

        // Test composite types are cached
        let ptr_i32_1 = codegen.type_to_llvm(&ResolvedType::Pointer(Box::new(ResolvedType::I32)));
        let ptr_i32_2 = codegen.type_to_llvm(&ResolvedType::Pointer(Box::new(ResolvedType::I32)));

        assert_eq!(ptr_i32_1, "i32*");
        assert_eq!(ptr_i32_2, "i32*");

        // Cache should have entries - verify caching is working
        let cache_size = codegen.type_to_llvm_cache.borrow().len();
        assert!(
            cache_size >= 1,
            "Cache should have at least 1 entry for pointer type"
        );
    }

    #[test]
    fn test_type_to_llvm_cache_all_basic_types() {
        let codegen = CodeGenerator::new("test");

        let test_types = vec![
            (ResolvedType::I8, "i8"),
            (ResolvedType::I16, "i16"),
            (ResolvedType::I32, "i32"),
            (ResolvedType::I64, "i64"),
            (ResolvedType::I128, "i128"),
            (ResolvedType::U8, "i8"),
            (ResolvedType::U16, "i16"),
            (ResolvedType::U32, "i32"),
            (ResolvedType::U64, "i64"),
            (ResolvedType::U128, "i128"),
            (ResolvedType::F32, "float"),
            (ResolvedType::F64, "double"),
            (ResolvedType::Bool, "i1"),
            (ResolvedType::Str, "i8*"),
            (ResolvedType::Unit, "void"),
        ];

        for (ty, expected) in test_types {
            let result = codegen.type_to_llvm(&ty);
            assert_eq!(
                result, expected,
                "Type {:?} should convert to {}",
                ty, expected
            );
        }

        // All primitive types use fast path and don't populate cache
        let cache = codegen.type_to_llvm_cache.borrow();
        assert!(
            cache.is_empty(),
            "Primitive types should use fast path and not populate cache"
        );
    }

    #[test]
    fn test_type_to_llvm_cache_performance() {
        let codegen = CodeGenerator::new("test");

        // First call - not cached
        let start = std::time::Instant::now();
        let _ = codegen.type_to_llvm(&ResolvedType::I32);
        let first_duration = start.elapsed();

        // Second call - should be cached (much faster)
        let start = std::time::Instant::now();
        let _ = codegen.type_to_llvm(&ResolvedType::I32);
        let second_duration = start.elapsed();

        // While we can't guarantee exact timing, cached lookups should be comparable or faster
        // The key point is that both should complete very quickly
        assert!(
            first_duration.as_nanos() + second_duration.as_nanos() < 1_000_000,
            "Both operations should be very fast (< 1ms total)"
        );
    }

    #[test]
    fn test_type_to_llvm_cache_nested_types() {
        let codegen = CodeGenerator::new("test");

        // Create a deeply nested type
        let nested_type = ResolvedType::Array(Box::new(ResolvedType::Pointer(Box::new(
            ResolvedType::Array(Box::new(ResolvedType::I32)),
        ))));

        let result1 = codegen.type_to_llvm(&nested_type);
        let result2 = codegen.type_to_llvm(&nested_type);

        assert_eq!(
            result1, result2,
            "Same nested types should produce same LLVM representation"
        );
        // Array wraps a pointer to Array wraps i32 -> i32***
        assert_eq!(
            result1, "i32***",
            "Nested array-pointer-array-i32 should produce i32***"
        );

        // Multiple calls should use cache
        let cache = codegen.type_to_llvm_cache.borrow();
        assert!(
            !cache.is_empty(),
            "Cache should have entries for nested types"
        );
    }

    #[test]
    fn test_type_to_llvm_different_types_same_representation() {
        let codegen = CodeGenerator::new("test");

        // Both signed and unsigned 8-bit types should map to i8
        let i8_result = codegen.type_to_llvm(&ResolvedType::I8);
        let u8_result = codegen.type_to_llvm(&ResolvedType::U8);

        assert_eq!(i8_result, "i8");
        assert_eq!(u8_result, "i8");
    }

    #[test]
    fn test_type_to_llvm_cache_named_types() {
        let codegen = CodeGenerator::new("test");

        // Test named types (structs)
        let named_type = ResolvedType::Named {
            name: "MyStruct".to_string(),
            generics: vec![],
        };

        let result1 = codegen.type_to_llvm(&named_type);
        let result2 = codegen.type_to_llvm(&named_type);

        assert_eq!(result1, "%MyStruct");
        assert_eq!(result2, "%MyStruct");
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_type_to_llvm_cache_generic_types() {
        let codegen = CodeGenerator::new("test");

        // Test generic types
        let generic_type = ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![ResolvedType::I32],
        };

        let result1 = codegen.type_to_llvm(&generic_type);
        let result2 = codegen.type_to_llvm(&generic_type);

        assert_eq!(result1, result2);
        // Result should start with % for struct type
        assert!(
            result1.starts_with("%"),
            "Generic struct type should produce a struct reference"
        );
    }

    #[test]
    fn test_type_to_llvm_cache_isolation() {
        let codegen1 = CodeGenerator::new("test1");
        let codegen2 = CodeGenerator::new("test2");

        // Each CodeGenerator should have its own cache
        // Use complex types that actually populate the cache (not primitives)
        let complex_type1 = ResolvedType::Pointer(Box::new(ResolvedType::I32));
        let complex_type2 = ResolvedType::Array(Box::new(ResolvedType::I64));

        let _result1 = codegen1.type_to_llvm(&complex_type1);
        let _result2 = codegen2.type_to_llvm(&complex_type2);

        let cache1_size = codegen1.type_to_llvm_cache.borrow().len();
        let cache2_size = codegen2.type_to_llvm_cache.borrow().len();

        // Both should have entries, and caches are independent
        assert!(cache1_size > 0, "codegen1 cache should have entries for complex type");
        assert!(cache2_size > 0, "codegen2 cache should have entries for complex type");
    }
}
