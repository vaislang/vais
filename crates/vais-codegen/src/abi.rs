//! ABI (Application Binary Interface) specification for Vais
//!
//! This module defines the stable ABI for Vais, including:
//! - Struct layout rules
//! - Function calling conventions
//! - VTable format
//! - ABI version compatibility checking
//!
//! The ABI version follows semantic versioning (major.minor.patch):
//! - Major: Breaking ABI changes (incompatible)
//! - Minor: Backwards-compatible additions
//! - Patch: Bug fixes that don't affect ABI

/// Current ABI version for Vais
///
/// Format: "major.minor.patch"
/// - Major version changes indicate breaking ABI changes
/// - Minor version changes indicate backwards-compatible additions
/// - Patch version changes indicate bug fixes that don't affect the ABI
pub const ABI_VERSION: &str = "1.0.0";

/// Result of ABI compatibility check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiCompatibility {
    /// Fully compatible - same major version
    Compatible,
    /// Minor differences - different minor/patch but same major version
    MinorDifference,
    /// Incompatible - different major version
    Incompatible,
}

/// Function calling convention
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    /// Standard C calling convention (cdecl) - used for FFI
    C,
    /// Vais default calling convention (currently same as C)
    Vais,
    /// LLVM fast calling convention - used for internal calls
    Fast,
    /// x86 stdcall convention (callee cleans stack)
    StdCall,
    /// x86 fastcall convention (first two args in registers)
    FastCall,
    /// Platform-dependent system convention (stdcall on Windows, cdecl elsewhere)
    System,
}

impl CallingConvention {
    /// Convert calling convention to LLVM IR string
    pub fn to_llvm_str(self) -> &'static str {
        match self {
            CallingConvention::C => "ccc",
            CallingConvention::Vais => "ccc", // Currently same as C
            CallingConvention::Fast => "fastcc",
            CallingConvention::StdCall => "x86_stdcallcc",
            CallingConvention::FastCall => "x86_fastcallcc",
            CallingConvention::System => {
                // On Windows x86, use stdcall; elsewhere, use C
                #[cfg(all(target_os = "windows", target_arch = "x86"))]
                return "x86_stdcallcc";
                #[cfg(not(all(target_os = "windows", target_arch = "x86")))]
                return "ccc";
            }
        }
    }

    /// Get the calling convention from a string (ABI string from extern block)
    pub fn parse_abi(s: &str) -> Option<Self> {
        match s {
            "C" | "ccc" => Some(CallingConvention::C),
            "Vais" | "vais" => Some(CallingConvention::Vais),
            "Fast" | "fastcc" => Some(CallingConvention::Fast),
            "stdcall" => Some(CallingConvention::StdCall),
            "fastcall" => Some(CallingConvention::FastCall),
            "system" => Some(CallingConvention::System),
            _ => None,
        }
    }
}

/// Type alignment rules
///
/// Defines the alignment requirements for primitive types in bytes
pub mod alignment {
    /// i8 alignment (1 byte)
    pub const I8: usize = 1;
    /// i16 alignment (2 bytes)
    pub const I16: usize = 2;
    /// i32 alignment (4 bytes)
    pub const I32: usize = 4;
    /// i64 alignment (8 bytes)
    pub const I64: usize = 8;
    /// f32 alignment (4 bytes)
    pub const F32: usize = 4;
    /// f64 alignment (8 bytes)
    pub const F64: usize = 8;
    /// Pointer alignment (8 bytes on 64-bit targets)
    pub const POINTER: usize = 8;
    /// Bool alignment (1 byte)
    pub const BOOL: usize = 1;

    /// Get alignment for a type size (power of 2)
    pub fn for_size(size: usize) -> usize {
        if size <= 1 {
            1
        } else if size <= 2 {
            2
        } else if size <= 4 {
            4
        } else {
            8
        }
    }
}

/// Struct layout specification
pub mod struct_layout {
    /// Struct layout rules
    ///
    /// # `repr(C)` structs
    /// - Fields are laid out in declaration order
    /// - Alignment follows C rules (compatible with C ABIs)
    /// - Padding is inserted to maintain alignment
    /// - Total size is rounded up to alignment
    ///
    /// # Normal structs (default)
    /// - Fields are laid out in declaration order
    /// - Alignment follows LLVM's natural alignment rules
    /// - No reordering optimization currently performed
    /// - Compatible with LLVM's struct layout
    pub const REPR_C_ORDERING: &str = "declaration-order";
    pub const DEFAULT_ORDERING: &str = "declaration-order";

    /// Calculate struct field offset with padding
    pub fn calculate_field_offset(current_offset: usize, field_align: usize) -> usize {
        // Align to field alignment
        current_offset.div_ceil(field_align) * field_align
    }

    /// Calculate total struct size with tail padding
    pub fn calculate_struct_size(fields_end: usize, struct_align: usize) -> usize {
        // Round up to struct alignment
        fields_end.div_ceil(struct_align) * struct_align
    }
}

pub mod vtable {
    //! VTable layout specification
    //!
    //! VTable structure format:
    //! ```text
    //! struct VTable {
    //!     drop_fn: void(i8*)*,  // Slot 0: Destructor function pointer
    //!     size: i64,            // Slot 1: Size of concrete type in bytes
    //!     align: i64,           // Slot 2: Alignment of concrete type in bytes
    //!     method_0: fn*,        // Slot 3: First method
    //!     method_1: fn*,        // Slot 4: Second method
    //!     ...                   // Additional methods in declaration order
    //! }
    //! ```

    /// VTable slot indices
    pub const SLOT_DROP_FN: usize = 0;
    pub const SLOT_SIZE: usize = 1;
    pub const SLOT_ALIGN: usize = 2;
    pub const SLOT_METHODS_START: usize = 3;

    /// Get the slot index for a method by its position
    pub fn method_slot(method_index: usize) -> usize {
        SLOT_METHODS_START + method_index
    }

    /// LLVM type for VTable with N methods
    pub fn vtable_type_with_methods(num_methods: usize) -> String {
        let mut fields = vec![
            "i8*".to_string(), // drop_fn
            "i64".to_string(), // size
            "i64".to_string(), // align
        ];

        // Add method slots
        for _ in 0..num_methods {
            fields.push("i8*".to_string()); // Generic function pointer
        }

        format!("{{ {} }}", fields.join(", "))
    }

    /// Number of metadata slots before methods
    pub const METADATA_SLOTS: usize = 3;
}

/// Check ABI version compatibility
///
/// Compares the provided version string against the current ABI_VERSION
/// and returns the compatibility status.
///
/// # Arguments
///
/// * `version` - Version string to check (format: "major.minor.patch")
///
/// # Returns
///
/// - `AbiCompatibility::Compatible` - Same major version
/// - `AbiCompatibility::MinorDifference` - Different minor/patch, same major
/// - `AbiCompatibility::Incompatible` - Different major version or invalid format
///
/// # Examples
///
/// ```
/// use vais_codegen::abi::{check_abi_compatibility, AbiCompatibility, ABI_VERSION};
///
/// assert_eq!(check_abi_compatibility(ABI_VERSION), AbiCompatibility::Compatible);
/// assert_eq!(check_abi_compatibility("1.1.0"), AbiCompatibility::MinorDifference);
/// assert_eq!(check_abi_compatibility("2.0.0"), AbiCompatibility::Incompatible);
/// ```
pub fn check_abi_compatibility(version: &str) -> AbiCompatibility {
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some((major, minor, patch))
    };

    let current = match parse_version(ABI_VERSION) {
        Some(v) => v,
        None => return AbiCompatibility::Incompatible,
    };

    let other = match parse_version(version) {
        Some(v) => v,
        None => return AbiCompatibility::Incompatible,
    };

    if current.0 != other.0 {
        // Different major version - incompatible
        AbiCompatibility::Incompatible
    } else if current == other {
        // Exact match
        AbiCompatibility::Compatible
    } else {
        // Same major, different minor/patch
        AbiCompatibility::MinorDifference
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_version_format() {
        // Ensure ABI_VERSION is in correct format
        assert!(ABI_VERSION.split('.').count() == 3);
        let parts: Vec<&str> = ABI_VERSION.split('.').collect();
        assert!(parts[0].parse::<u32>().is_ok());
        assert!(parts[1].parse::<u32>().is_ok());
        assert!(parts[2].parse::<u32>().is_ok());
    }

    #[test]
    fn test_compatibility_same_version() {
        assert_eq!(
            check_abi_compatibility(ABI_VERSION),
            AbiCompatibility::Compatible
        );
    }

    #[test]
    fn test_compatibility_minor_difference() {
        assert_eq!(
            check_abi_compatibility("1.1.0"),
            AbiCompatibility::MinorDifference
        );
        assert_eq!(
            check_abi_compatibility("1.0.1"),
            AbiCompatibility::MinorDifference
        );
        assert_eq!(
            check_abi_compatibility("1.999.999"),
            AbiCompatibility::MinorDifference
        );
    }

    #[test]
    fn test_compatibility_incompatible() {
        assert_eq!(
            check_abi_compatibility("2.0.0"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("0.1.0"),
            AbiCompatibility::Incompatible
        );
    }

    #[test]
    fn test_compatibility_invalid_format() {
        assert_eq!(
            check_abi_compatibility("1.0"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("invalid"),
            AbiCompatibility::Incompatible
        );
        assert_eq!(
            check_abi_compatibility("1.0.0.0"),
            AbiCompatibility::Incompatible
        );
    }

    #[test]
    fn test_calling_convention_to_llvm() {
        assert_eq!(CallingConvention::C.to_llvm_str(), "ccc");
        assert_eq!(CallingConvention::Vais.to_llvm_str(), "ccc");
        assert_eq!(CallingConvention::Fast.to_llvm_str(), "fastcc");
        assert_eq!(CallingConvention::StdCall.to_llvm_str(), "x86_stdcallcc");
        assert_eq!(CallingConvention::FastCall.to_llvm_str(), "x86_fastcallcc");
        // System is platform-dependent
        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        assert_eq!(CallingConvention::System.to_llvm_str(), "x86_stdcallcc");
        #[cfg(not(all(target_os = "windows", target_arch = "x86")))]
        assert_eq!(CallingConvention::System.to_llvm_str(), "ccc");
    }

    #[test]
    fn test_calling_convention_from_str() {
        assert_eq!(
            CallingConvention::parse_abi("C"),
            Some(CallingConvention::C)
        );
        assert_eq!(
            CallingConvention::parse_abi("ccc"),
            Some(CallingConvention::C)
        );
        assert_eq!(
            CallingConvention::parse_abi("Vais"),
            Some(CallingConvention::Vais)
        );
        assert_eq!(
            CallingConvention::parse_abi("Fast"),
            Some(CallingConvention::Fast)
        );
        assert_eq!(
            CallingConvention::parse_abi("fastcc"),
            Some(CallingConvention::Fast)
        );
        assert_eq!(
            CallingConvention::parse_abi("stdcall"),
            Some(CallingConvention::StdCall)
        );
        assert_eq!(
            CallingConvention::parse_abi("fastcall"),
            Some(CallingConvention::FastCall)
        );
        assert_eq!(
            CallingConvention::parse_abi("system"),
            Some(CallingConvention::System)
        );
        assert_eq!(CallingConvention::parse_abi("invalid"), None);
    }

    #[test]
    fn test_alignment_constants() {
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
        assert_eq!(alignment::for_size(0), 1);
        assert_eq!(alignment::for_size(1), 1);
        assert_eq!(alignment::for_size(2), 2);
        assert_eq!(alignment::for_size(3), 4);
        assert_eq!(alignment::for_size(4), 4);
        assert_eq!(alignment::for_size(5), 8);
        assert_eq!(alignment::for_size(8), 8);
        assert_eq!(alignment::for_size(16), 8);
    }

    #[test]
    fn test_struct_field_offset() {
        // Field at offset 0 with align 4 -> 0
        assert_eq!(struct_layout::calculate_field_offset(0, 4), 0);

        // Field at offset 1 with align 4 -> 4
        assert_eq!(struct_layout::calculate_field_offset(1, 4), 4);

        // Field at offset 5 with align 8 -> 8
        assert_eq!(struct_layout::calculate_field_offset(5, 8), 8);

        // Field at offset 8 with align 8 -> 8 (already aligned)
        assert_eq!(struct_layout::calculate_field_offset(8, 8), 8);
    }

    #[test]
    fn test_struct_size_calculation() {
        // Size 12 with align 4 -> 12 (already aligned)
        assert_eq!(struct_layout::calculate_struct_size(12, 4), 12);

        // Size 13 with align 4 -> 16
        assert_eq!(struct_layout::calculate_struct_size(13, 4), 16);

        // Size 5 with align 8 -> 8
        assert_eq!(struct_layout::calculate_struct_size(5, 8), 8);

        // Size 16 with align 8 -> 16 (already aligned)
        assert_eq!(struct_layout::calculate_struct_size(16, 8), 16);
    }

    #[test]
    fn test_vtable_slot_indices() {
        assert_eq!(vtable::SLOT_DROP_FN, 0);
        assert_eq!(vtable::SLOT_SIZE, 1);
        assert_eq!(vtable::SLOT_ALIGN, 2);
        assert_eq!(vtable::SLOT_METHODS_START, 3);
        assert_eq!(vtable::METADATA_SLOTS, 3);
    }

    #[test]
    fn test_vtable_method_slot() {
        assert_eq!(vtable::method_slot(0), 3);
        assert_eq!(vtable::method_slot(1), 4);
        assert_eq!(vtable::method_slot(2), 5);
    }

    #[test]
    fn test_vtable_type_with_methods() {
        // VTable with 0 methods
        assert_eq!(vtable::vtable_type_with_methods(0), "{ i8*, i64, i64 }");

        // VTable with 1 method
        assert_eq!(
            vtable::vtable_type_with_methods(1),
            "{ i8*, i64, i64, i8* }"
        );

        // VTable with 2 methods
        assert_eq!(
            vtable::vtable_type_with_methods(2),
            "{ i8*, i64, i64, i8*, i8* }"
        );
    }
}
