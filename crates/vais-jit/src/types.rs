//! Type mapping between Vais types and Cranelift types.

use cranelift::prelude::*;
use vais_types::ResolvedType;

/// Maps Vais types to Cranelift types.
pub struct TypeMapper {
    pointer_type: Type,
}

impl TypeMapper {
    /// Creates a new type mapper for the given pointer width.
    pub fn new(pointer_type: Type) -> Self {
        Self { pointer_type }
    }

    /// Maps a Vais resolved type to a Cranelift type.
    pub fn map_type(&self, ty: &ResolvedType) -> Type {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 => types::I8,
            ResolvedType::I16 | ResolvedType::U16 => types::I16,
            ResolvedType::I32 | ResolvedType::U32 => types::I32,
            ResolvedType::I64 | ResolvedType::U64 => types::I64,
            ResolvedType::I128 | ResolvedType::U128 => types::I128,
            ResolvedType::F32 => types::F32,
            ResolvedType::F64 => types::F64,
            ResolvedType::Bool => types::I8,
            ResolvedType::Str => self.pointer_type,
            ResolvedType::Unit => types::I8, // Unit as single byte (placeholder)
            ResolvedType::Pointer(_) => self.pointer_type,
            ResolvedType::Ref(_) => self.pointer_type,
            ResolvedType::RefMut(_) => self.pointer_type,
            ResolvedType::Array(_) => self.pointer_type,
            ResolvedType::Map(_, _) => self.pointer_type,
            ResolvedType::Named { .. } => self.pointer_type,
            ResolvedType::Fn { .. } => self.pointer_type,
            ResolvedType::Optional(_) => self.pointer_type,
            ResolvedType::Result(_) => self.pointer_type,
            ResolvedType::Tuple(_) => self.pointer_type,
            ResolvedType::Range(_) => self.pointer_type,
            ResolvedType::Future(_) => self.pointer_type,
            ResolvedType::Generic(_) => {
                panic!("Unsubstituted generic type in JIT")
            }
            ResolvedType::Var(_) => {
                panic!("Unresolved type variable in JIT")
            }
            ResolvedType::Unknown => self.pointer_type,
            ResolvedType::Never => types::I64, // Never type should not occur in JIT, but default to i64
            ResolvedType::ConstArray { .. } => self.pointer_type, // Const arrays are represented as pointers
            ResolvedType::ConstGeneric(_) => {
                panic!("Unsubstituted const generic in JIT")
            }
            ResolvedType::Vector { element, lanes } => {
                // Map SIMD vector types to Cranelift vector types
                let elem_type = self.map_type(element);
                // Cranelift supports vector types like I8X16, I16X8, I32X4, I64X2, F32X4, F64X2
                match (elem_type, *lanes) {
                    (types::I8, 16) => types::I8X16,
                    (types::I16, 8) => types::I16X8,
                    (types::I32, 4) => types::I32X4,
                    (types::I32, 8) => types::I32X4, // Use I32X4 for 8-lane (two operations)
                    (types::I64, 2) => types::I64X2,
                    (types::I64, 4) => types::I64X2, // Use I64X2 for 4-lane
                    (types::F32, 2) => types::F32X4, // Use F32X4 for 2-lane
                    (types::F32, 4) => types::F32X4,
                    (types::F32, 8) => types::F32X4, // Use F32X4 for 8-lane (two operations)
                    (types::F64, 2) => types::F64X2,
                    (types::F64, 4) => types::F64X2, // Use F64X2 for 4-lane
                    _ => {
                        // Fallback: treat as pointer to array
                        self.pointer_type
                    }
                }
            }
        }
    }

    /// Returns the size of a type in bytes.
    pub fn size_of(&self, ty: &ResolvedType) -> u32 {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str | ResolvedType::Pointer(_) | ResolvedType::Ref(_) | ResolvedType::RefMut(_) => {
                if self.pointer_type == types::I64 {
                    8
                } else {
                    4
                }
            }
            ResolvedType::Unit => 0,
            ResolvedType::Vector { element, lanes } => {
                // Vector size = element size * lane count
                self.size_of(element) * (*lanes as u32)
            }
            _ => {
                if self.pointer_type == types::I64 {
                    8
                } else {
                    4
                }
            }
        }
    }

    /// Returns the pointer type for this target.
    pub fn pointer_type(&self) -> Type {
        self.pointer_type
    }

    /// Checks if a type is a floating point type.
    pub fn is_float(&self, ty: &ResolvedType) -> bool {
        matches!(ty, ResolvedType::F32 | ResolvedType::F64)
    }

    /// Checks if a type is a signed integer type.
    pub fn is_signed(&self, ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::I128
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mapping() {
        let mapper = TypeMapper::new(types::I64);

        assert_eq!(mapper.map_type(&ResolvedType::I8), types::I8);
        assert_eq!(mapper.map_type(&ResolvedType::I16), types::I16);
        assert_eq!(mapper.map_type(&ResolvedType::I32), types::I32);
        assert_eq!(mapper.map_type(&ResolvedType::I64), types::I64);
        assert_eq!(mapper.map_type(&ResolvedType::F32), types::F32);
        assert_eq!(mapper.map_type(&ResolvedType::F64), types::F64);
        assert_eq!(mapper.map_type(&ResolvedType::Bool), types::I8);
    }

    #[test]
    fn test_size_of() {
        let mapper = TypeMapper::new(types::I64);

        assert_eq!(mapper.size_of(&ResolvedType::I8), 1);
        assert_eq!(mapper.size_of(&ResolvedType::I16), 2);
        assert_eq!(mapper.size_of(&ResolvedType::I32), 4);
        assert_eq!(mapper.size_of(&ResolvedType::I64), 8);
        assert_eq!(mapper.size_of(&ResolvedType::Str), 8);
    }

    #[test]
    fn test_is_float() {
        let mapper = TypeMapper::new(types::I64);

        assert!(mapper.is_float(&ResolvedType::F32));
        assert!(mapper.is_float(&ResolvedType::F64));
        assert!(!mapper.is_float(&ResolvedType::I32));
        assert!(!mapper.is_float(&ResolvedType::Bool));
    }

    #[test]
    fn test_is_signed() {
        let mapper = TypeMapper::new(types::I64);

        assert!(mapper.is_signed(&ResolvedType::I8));
        assert!(mapper.is_signed(&ResolvedType::I32));
        assert!(!mapper.is_signed(&ResolvedType::U32));
        assert!(!mapper.is_signed(&ResolvedType::F64));
    }
}
