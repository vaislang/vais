//! Tests for ABI specification and compatibility

#[cfg(test)]
mod tests {
    use crate::abi::*;

    // ============================================================================
    // ABI Version Compatibility Tests
    // ============================================================================

    #[test]
    fn test_abi_version_is_valid_semver() {
        // Verify ABI_VERSION follows semantic versioning format
        let parts: Vec<&str> = ABI_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "ABI version must have 3 parts (major.minor.patch)");

        for part in parts {
            assert!(
                part.parse::<u32>().is_ok(),
                "Each part of ABI version must be a valid u32: {}",
                part
            );
        }
    }

    #[test]
    fn test_compatibility_exact_match() {
        assert_eq!(
            check_abi_compatibility(ABI_VERSION),
            AbiCompatibility::Compatible
        );
    }

    #[test]
    fn test_compatibility_same_major() {
        // Same major version should be compatible or minor difference
        let result = check_abi_compatibility("1.0.0");
        assert!(
            matches!(result, AbiCompatibility::Compatible | AbiCompatibility::MinorDifference)
        );
    }

    #[test]
    fn test_compatibility_different_patch() {
        assert_eq!(
            check_abi_compatibility("1.0.1"),
            AbiCompatibility::MinorDifference
        );
        assert_eq!(
            check_abi_compatibility("1.0.99"),
            AbiCompatibility::MinorDifference
        );
    }

    #[test]
    fn test_compatibility_different_minor() {
        assert_eq!(
            check_abi_compatibility("1.1.0"),
            AbiCompatibility::MinorDifference
        );
        assert_eq!(
            check_abi_compatibility("1.99.0"),
            AbiCompatibility::MinorDifference
        );
    }

    #[test]
    fn test_compatibility_incompatible_major() {
        assert_eq!(
            check_abi_compatibility("2.0.0"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("0.1.0"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("99.0.0"),
            AbiCompatibility::Incompatible
        );
    }

    #[test]
    fn test_compatibility_invalid_version_format() {
        // Invalid formats should be incompatible
        assert_eq!(
            check_abi_compatibility("1.0"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("1"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("1.0.0.0"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("invalid"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("v1.0.0"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility(""),
            AbiCompatibility::Incompatible
        );
    }

    // ============================================================================
    // Calling Convention Tests
    // ============================================================================

    #[test]
    fn test_calling_convention_to_llvm_str() {
        assert_eq!(CallingConvention::C.to_llvm_str(), "ccc");
        assert_eq!(CallingConvention::Vais.to_llvm_str(), "ccc");
        assert_eq!(CallingConvention::Fast.to_llvm_str(), "fastcc");
    }

    #[test]
    fn test_calling_convention_from_str_valid() {
        assert_eq!(CallingConvention::parse_abi("C"), Some(CallingConvention::C));
        assert_eq!(CallingConvention::parse_abi("ccc"), Some(CallingConvention::C));
        assert_eq!(CallingConvention::parse_abi("Vais"), Some(CallingConvention::Vais));
        assert_eq!(CallingConvention::parse_abi("vais"), Some(CallingConvention::Vais));
        assert_eq!(CallingConvention::parse_abi("Fast"), Some(CallingConvention::Fast));
        assert_eq!(CallingConvention::parse_abi("fastcc"), Some(CallingConvention::Fast));
    }

    #[test]
    fn test_calling_convention_from_str_invalid() {
        assert_eq!(CallingConvention::parse_abi("invalid"), None);
        assert_eq!(CallingConvention::parse_abi(""), None);
        assert_eq!(CallingConvention::parse_abi("cdecl"), None);
        assert_eq!(CallingConvention::parse_abi("unknown"), None);
    }

    #[test]
    fn test_calling_convention_roundtrip() {
        // Note: C and Vais both map to "ccc", so they're not uniquely roundtrippable
        // This test checks that parsing llvm_str yields a valid convention
        assert_eq!(CallingConvention::parse_abi("ccc"), Some(CallingConvention::C));
        assert_eq!(CallingConvention::parse_abi("fastcc"), Some(CallingConvention::Fast));

        // Test that to_llvm_str produces parseable strings
        let c_str = CallingConvention::C.to_llvm_str();
        assert!(CallingConvention::parse_abi(c_str).is_some());

        let fast_str = CallingConvention::Fast.to_llvm_str();
        assert_eq!(CallingConvention::parse_abi(fast_str), Some(CallingConvention::Fast));
    }

    // ============================================================================
    // Alignment Tests
    // ============================================================================

    #[test]
    fn test_alignment_constants_are_powers_of_two() {
        fn is_power_of_two(n: usize) -> bool {
            n > 0 && (n & (n - 1)) == 0
        }

        assert!(is_power_of_two(alignment::I8));
        assert!(is_power_of_two(alignment::I16));
        assert!(is_power_of_two(alignment::I32));
        assert!(is_power_of_two(alignment::I64));
        assert!(is_power_of_two(alignment::F32));
        assert!(is_power_of_two(alignment::F64));
        assert!(is_power_of_two(alignment::POINTER));
        assert!(is_power_of_two(alignment::BOOL));
    }

    #[test]
    fn test_alignment_values() {
        assert_eq!(alignment::I8, 1);
        assert_eq!(alignment::I16, 2);
        assert_eq!(alignment::I32, 4);
        assert_eq!(alignment::I64, 8);
        assert_eq!(alignment::F32, 4);
        assert_eq!(alignment::F64, 8);
        assert_eq!(alignment::POINTER, 8);
        assert_eq!(alignment::BOOL, 1);
    }

    #[test]
    fn test_alignment_for_size() {
        // Zero and one byte
        assert_eq!(alignment::for_size(0), 1);
        assert_eq!(alignment::for_size(1), 1);

        // Two bytes
        assert_eq!(alignment::for_size(2), 2);

        // 3-4 bytes align to 4
        assert_eq!(alignment::for_size(3), 4);
        assert_eq!(alignment::for_size(4), 4);

        // 5-8 bytes align to 8
        assert_eq!(alignment::for_size(5), 8);
        assert_eq!(alignment::for_size(6), 8);
        assert_eq!(alignment::for_size(7), 8);
        assert_eq!(alignment::for_size(8), 8);

        // Larger sizes align to 8
        assert_eq!(alignment::for_size(16), 8);
        assert_eq!(alignment::for_size(100), 8);
        assert_eq!(alignment::for_size(1024), 8);
    }

    #[test]
    fn test_alignment_for_size_returns_power_of_two() {
        fn is_power_of_two(n: usize) -> bool {
            n > 0 && (n & (n - 1)) == 0
        }

        for size in 0..100 {
            let align = alignment::for_size(size);
            assert!(is_power_of_two(align), "Alignment for size {} is not power of two: {}", size, align);
        }
    }

    // ============================================================================
    // Struct Layout Tests
    // ============================================================================

    #[test]
    fn test_struct_field_offset_aligned() {
        // Already aligned offsets should stay the same
        assert_eq!(struct_layout::calculate_field_offset(0, 1), 0);
        assert_eq!(struct_layout::calculate_field_offset(0, 4), 0);
        assert_eq!(struct_layout::calculate_field_offset(0, 8), 0);
        assert_eq!(struct_layout::calculate_field_offset(4, 4), 4);
        assert_eq!(struct_layout::calculate_field_offset(8, 8), 8);
        assert_eq!(struct_layout::calculate_field_offset(16, 8), 16);
    }

    #[test]
    fn test_struct_field_offset_needs_padding() {
        // Test cases requiring padding
        assert_eq!(struct_layout::calculate_field_offset(1, 4), 4);
        assert_eq!(struct_layout::calculate_field_offset(2, 4), 4);
        assert_eq!(struct_layout::calculate_field_offset(3, 4), 4);
        assert_eq!(struct_layout::calculate_field_offset(5, 8), 8);
        assert_eq!(struct_layout::calculate_field_offset(9, 8), 16);
        assert_eq!(struct_layout::calculate_field_offset(10, 4), 12);
    }

    #[test]
    fn test_struct_field_offset_examples() {
        // Example: struct { i8, i32 } - i32 needs 4-byte alignment
        let offset_after_i8 = 1;
        assert_eq!(struct_layout::calculate_field_offset(offset_after_i8, 4), 4);

        // Example: struct { i8, i8, i64 } - i64 needs 8-byte alignment
        let offset_after_two_i8 = 2;
        assert_eq!(struct_layout::calculate_field_offset(offset_after_two_i8, 8), 8);

        // Example: struct { i32, i64 } - i64 needs 8-byte alignment
        let offset_after_i32 = 4;
        assert_eq!(struct_layout::calculate_field_offset(offset_after_i32, 8), 8);
    }

    #[test]
    fn test_struct_size_calculation_aligned() {
        // Already aligned sizes
        assert_eq!(struct_layout::calculate_struct_size(4, 4), 4);
        assert_eq!(struct_layout::calculate_struct_size(8, 8), 8);
        assert_eq!(struct_layout::calculate_struct_size(16, 8), 16);
        assert_eq!(struct_layout::calculate_struct_size(12, 4), 12);
    }

    #[test]
    fn test_struct_size_calculation_needs_padding() {
        // Sizes requiring tail padding
        assert_eq!(struct_layout::calculate_struct_size(1, 4), 4);
        assert_eq!(struct_layout::calculate_struct_size(5, 8), 8);
        assert_eq!(struct_layout::calculate_struct_size(9, 8), 16);
        assert_eq!(struct_layout::calculate_struct_size(13, 4), 16);
        assert_eq!(struct_layout::calculate_struct_size(17, 8), 24);
    }

    #[test]
    fn test_struct_size_calculation_examples() {
        // Example: struct { i8 } with align 1 -> size 1
        assert_eq!(struct_layout::calculate_struct_size(1, 1), 1);

        // Example: struct { i8, i32 } - fields end at offset 8, align to 4 -> size 8
        assert_eq!(struct_layout::calculate_struct_size(8, 4), 8);

        // Example: struct { i8, i8, i64 } - fields end at offset 16, align to 8 -> size 16
        assert_eq!(struct_layout::calculate_struct_size(16, 8), 16);

        // Example: struct { i32, i8 } - fields end at offset 5, align to 4 -> size 8
        assert_eq!(struct_layout::calculate_struct_size(5, 4), 8);
    }

    // ============================================================================
    // VTable Layout Tests
    // ============================================================================

    #[test]
    fn test_vtable_slot_indices() {
        assert_eq!(vtable::SLOT_DROP_FN, 0);
        assert_eq!(vtable::SLOT_SIZE, 1);
        assert_eq!(vtable::SLOT_ALIGN, 2);
        assert_eq!(vtable::SLOT_METHODS_START, 3);
    }

    #[test]
    fn test_vtable_metadata_slots() {
        assert_eq!(vtable::METADATA_SLOTS, 3);
        assert_eq!(vtable::METADATA_SLOTS, vtable::SLOT_METHODS_START);
    }

    #[test]
    fn test_vtable_method_slot_calculation() {
        assert_eq!(vtable::method_slot(0), 3);
        assert_eq!(vtable::method_slot(1), 4);
        assert_eq!(vtable::method_slot(2), 5);
        assert_eq!(vtable::method_slot(10), 13);
        assert_eq!(vtable::method_slot(100), 103);
    }

    #[test]
    fn test_vtable_method_slot_is_sequential() {
        for i in 0..20 {
            assert_eq!(vtable::method_slot(i), vtable::SLOT_METHODS_START + i);
        }
    }

    #[test]
    fn test_vtable_type_with_zero_methods() {
        let ty = vtable::vtable_type_with_methods(0);
        assert_eq!(ty, "{ i8*, i64, i64 }");

        // Count fields: drop_fn, size, align
        let field_count = ty.matches(',').count() + 1;
        assert_eq!(field_count, 3);
    }

    #[test]
    fn test_vtable_type_with_one_method() {
        let ty = vtable::vtable_type_with_methods(1);
        assert_eq!(ty, "{ i8*, i64, i64, i8* }");

        // Count fields: drop_fn, size, align, method_0
        let field_count = ty.matches(',').count() + 1;
        assert_eq!(field_count, 4);
    }

    #[test]
    fn test_vtable_type_with_multiple_methods() {
        let ty = vtable::vtable_type_with_methods(3);
        assert_eq!(ty, "{ i8*, i64, i64, i8*, i8*, i8* }");

        // Count fields: drop_fn, size, align, method_0, method_1, method_2
        let field_count = ty.matches(',').count() + 1;
        assert_eq!(field_count, 6);
    }

    #[test]
    fn test_vtable_type_field_count() {
        for num_methods in 0..10 {
            let ty = vtable::vtable_type_with_methods(num_methods);
            let field_count = ty.matches(',').count() + 1;
            assert_eq!(
                field_count,
                vtable::METADATA_SLOTS + num_methods,
                "VTable with {} methods has wrong field count",
                num_methods
            );
        }
    }

    #[test]
    fn test_vtable_type_format() {
        // Verify the type string is properly formatted LLVM struct type
        let ty = vtable::vtable_type_with_methods(2);
        assert!(ty.starts_with("{ "));
        assert!(ty.ends_with(" }"));
        assert!(ty.contains("i8*"));
        assert!(ty.contains("i64"));
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_realistic_struct_layout_i8_i32() {
        // struct { i8, i32 }
        let mut offset = 0;

        // i8 at offset 0
        offset += 1; // sizeof(i8)

        // i32 needs 4-byte alignment
        offset = struct_layout::calculate_field_offset(offset, alignment::I32);
        assert_eq!(offset, 4);
        offset += 4; // sizeof(i32)

        // Total size with tail padding (align to struct alignment = 4)
        let size = struct_layout::calculate_struct_size(offset, alignment::I32);
        assert_eq!(size, 8);
    }

    #[test]
    fn test_realistic_struct_layout_i8_i8_i64() {
        // struct { i8, i8, i64 }
        let mut offset = 0;

        // i8 at offset 0
        offset += 1;

        // i8 at offset 1
        offset += 1;

        // i64 needs 8-byte alignment
        offset = struct_layout::calculate_field_offset(offset, alignment::I64);
        assert_eq!(offset, 8);
        offset += 8; // sizeof(i64)

        // Total size with tail padding (align to 8)
        let size = struct_layout::calculate_struct_size(offset, alignment::I64);
        assert_eq!(size, 16);
    }

    #[test]
    fn test_realistic_struct_layout_i32_i8() {
        // struct { i32, i8 }
        let mut offset = 0;

        // i32 at offset 0
        offset += 4;

        // i8 at offset 4
        offset += 1;

        // Total size with tail padding (align to struct alignment = 4)
        let size = struct_layout::calculate_struct_size(offset, alignment::I32);
        assert_eq!(size, 8);
    }
}
