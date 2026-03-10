use super::*;
use crate::CodeGenerator;
use vais_types::ResolvedType;

#[test]
fn test_tuple_sizeof_sums_elements() {
    let gen = CodeGenerator::new("test");
    // (i8, i8) should be 2 bytes, not 16
    let tuple_type = ResolvedType::Tuple(vec![ResolvedType::I8, ResolvedType::I8]);
    assert_eq!(gen.compute_sizeof(&tuple_type), 2);
}

#[test]
fn test_tuple_alignof_takes_max() {
    let gen = CodeGenerator::new("test");
    // (i8, i32) should have alignment of 4 (from i32)
    let tuple_type = ResolvedType::Tuple(vec![ResolvedType::I8, ResolvedType::I32]);
    assert_eq!(gen.compute_alignof(&tuple_type), 4);
}

#[test]
fn test_struct_sizeof_sums_fields() {
    let mut gen = CodeGenerator::new("test");
    // Struct with two i8 fields
    gen.types.structs.insert(
        "Point2D".to_string(),
        StructInfo {
            _name: "Point2D".to_string(),
            fields: vec![
                ("x".to_string(), ResolvedType::I8),
                ("y".to_string(), ResolvedType::I8),
            ],
            _repr_c: false,
            _invariants: vec![],
        },
    );
    let struct_type = ResolvedType::Named {
        name: "Point2D".to_string(),
        generics: vec![],
    };
    assert_eq!(gen.compute_sizeof(&struct_type), 2);
}

#[test]
fn test_struct_alignof_takes_max_field() {
    let mut gen = CodeGenerator::new("test");
    // Struct with i8 and i32 fields
    gen.types.structs.insert(
        "MixedStruct".to_string(),
        StructInfo {
            _name: "MixedStruct".to_string(),
            fields: vec![
                ("a".to_string(), ResolvedType::I8),
                ("b".to_string(), ResolvedType::I32),
                ("c".to_string(), ResolvedType::I16),
            ],
            _repr_c: false,
            _invariants: vec![],
        },
    );
    let struct_type = ResolvedType::Named {
        name: "MixedStruct".to_string(),
        generics: vec![],
    };
    // Size: 1 + 4 + 2 = 7
    assert_eq!(gen.compute_sizeof(&struct_type), 7);
    // Alignment: max(1, 4, 2) = 4
    assert_eq!(gen.compute_alignof(&struct_type), 4);
}

#[test]
fn test_primitive_types() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_sizeof(&ResolvedType::I8), 1);
    assert_eq!(gen.compute_alignof(&ResolvedType::I8), 1);
    assert_eq!(gen.compute_sizeof(&ResolvedType::I32), 4);
    assert_eq!(gen.compute_alignof(&ResolvedType::I32), 4);
    assert_eq!(gen.compute_sizeof(&ResolvedType::I64), 8);
    assert_eq!(gen.compute_alignof(&ResolvedType::I64), 8);
    assert_eq!(gen.compute_sizeof(&ResolvedType::I128), 16);
    assert_eq!(gen.compute_alignof(&ResolvedType::I128), 16);
}

// ========== format_llvm_float ==========

#[test]
fn test_format_llvm_float_zero() {
    let result = format_llvm_float(0.0);
    assert!(result.contains("e+00") || result.contains("e-00"));
}

#[test]
fn test_format_llvm_float_positive() {
    let result = format_llvm_float(1.0);
    assert!(result.contains("1.000000e+00"));
}

#[test]
fn test_format_llvm_float_negative() {
    let result = format_llvm_float(-1.0);
    assert!(result.contains("-1.000000e+00"));
}

#[test]
fn test_format_llvm_float_large() {
    let result = format_llvm_float(100.0);
    assert!(result.contains("e+02"));
}

#[test]
fn test_format_llvm_float_small() {
    let result = format_llvm_float(0.001);
    assert!(result.contains("e-03"));
}

#[test]
fn test_format_llvm_float_pi() {
    let result = format_llvm_float(std::f64::consts::PI);
    assert!(result.starts_with("3.14159"));
    assert!(result.contains("e+00"));
}

#[test]
fn test_format_llvm_float_negative_exponent() {
    let result = format_llvm_float(0.01);
    assert!(result.contains("e-"));
}

// ========== LocalVar ==========

#[test]
fn test_local_var_param() {
    let var = LocalVar::param(ResolvedType::I64, "%arg0");
    assert!(var.is_param());
    assert!(!var.is_ssa());
    assert!(!var.is_alloca());
    assert_eq!(var.llvm_name, "%arg0");
}

#[test]
fn test_local_var_ssa() {
    let var = LocalVar::ssa(ResolvedType::Bool, "%t0");
    assert!(!var.is_param());
    assert!(var.is_ssa());
    assert!(!var.is_alloca());
}

#[test]
fn test_local_var_alloca() {
    let var = LocalVar::alloca(ResolvedType::F64, "%x.addr");
    assert!(!var.is_param());
    assert!(!var.is_ssa());
    assert!(var.is_alloca());
}

#[test]
fn test_local_var_type_preserved() {
    let var = LocalVar::param(ResolvedType::Str, "%s");
    assert_eq!(var.ty, ResolvedType::Str);
}

#[test]
fn test_local_var_kind_enum() {
    assert_eq!(LocalVarKind::Param, LocalVarKind::Param);
    assert_ne!(LocalVarKind::Param, LocalVarKind::Ssa);
    assert_ne!(LocalVarKind::Ssa, LocalVarKind::Alloca);
}

#[test]
fn test_local_var_clone() {
    let var = LocalVar::ssa(ResolvedType::I32, "%val");
    let cloned = var.clone();
    assert_eq!(cloned.llvm_name, var.llvm_name);
    assert_eq!(cloned.ty, var.ty);
    assert!(cloned.is_ssa());
}

// ========== More sizeof/alignof ==========

#[test]
fn test_sizeof_unsigned_types() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_sizeof(&ResolvedType::U8), 1);
    assert_eq!(gen.compute_sizeof(&ResolvedType::U16), 2);
    assert_eq!(gen.compute_sizeof(&ResolvedType::U32), 4);
    assert_eq!(gen.compute_sizeof(&ResolvedType::U64), 8);
    assert_eq!(gen.compute_sizeof(&ResolvedType::U128), 16);
}

#[test]
fn test_sizeof_float_types() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_sizeof(&ResolvedType::F32), 4);
    assert_eq!(gen.compute_sizeof(&ResolvedType::F64), 8);
}

#[test]
fn test_sizeof_bool() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_sizeof(&ResolvedType::Bool), 1);
    assert_eq!(gen.compute_alignof(&ResolvedType::Bool), 1);
}

#[test]
fn test_sizeof_str() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_sizeof(&ResolvedType::Str), 16); // fat pointer { i8*, i64 }
}

#[test]
fn test_sizeof_unit() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_sizeof(&ResolvedType::Unit), 0);
    assert_eq!(gen.compute_alignof(&ResolvedType::Unit), 1);
}

#[test]
fn test_sizeof_pointer() {
    let gen = CodeGenerator::new("test");
    let ptr = ResolvedType::Pointer(Box::new(ResolvedType::I8));
    assert_eq!(gen.compute_sizeof(&ptr), 8);
    assert_eq!(gen.compute_alignof(&ptr), 8);
}

#[test]
fn test_sizeof_ref() {
    let gen = CodeGenerator::new("test");
    let r = ResolvedType::Ref(Box::new(ResolvedType::I64));
    assert_eq!(gen.compute_sizeof(&r), 8);
}

#[test]
fn test_sizeof_ref_mut() {
    let gen = CodeGenerator::new("test");
    let r = ResolvedType::RefMut(Box::new(ResolvedType::I64));
    assert_eq!(gen.compute_sizeof(&r), 8);
}

#[test]
fn test_sizeof_array() {
    let gen = CodeGenerator::new("test");
    let arr = ResolvedType::Array(Box::new(ResolvedType::I64));
    assert_eq!(gen.compute_sizeof(&arr), 8); // pointer to heap
}

#[test]
fn test_sizeof_optional() {
    let gen = CodeGenerator::new("test");
    let opt = ResolvedType::Optional(Box::new(ResolvedType::I64));
    assert_eq!(gen.compute_sizeof(&opt), 8);
}

#[test]
fn test_sizeof_result() {
    let gen = CodeGenerator::new("test");
    let res = ResolvedType::Result(Box::new(ResolvedType::I64), Box::new(ResolvedType::Str));
    assert_eq!(gen.compute_sizeof(&res), 8);
}

#[test]
fn test_sizeof_unknown_named() {
    let gen = CodeGenerator::new("test");
    let named = ResolvedType::Named {
        name: "UnknownType".to_string(),
        generics: vec![],
    };
    assert_eq!(gen.compute_sizeof(&named), 8); // default
}

#[test]
fn test_alignof_unsigned_types() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_alignof(&ResolvedType::U8), 1);
    assert_eq!(gen.compute_alignof(&ResolvedType::U16), 2);
    assert_eq!(gen.compute_alignof(&ResolvedType::U32), 4);
    assert_eq!(gen.compute_alignof(&ResolvedType::U64), 8);
    assert_eq!(gen.compute_alignof(&ResolvedType::U128), 16);
}

#[test]
fn test_alignof_float_types() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_alignof(&ResolvedType::F32), 4);
    assert_eq!(gen.compute_alignof(&ResolvedType::F64), 8);
}

#[test]
fn test_alignof_str() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.compute_alignof(&ResolvedType::Str), 8);
}

#[test]
fn test_alignof_empty_tuple() {
    let gen = CodeGenerator::new("test");
    let tuple = ResolvedType::Tuple(vec![]);
    assert_eq!(gen.compute_alignof(&tuple), 8); // default when max is None
}

// ========== type_to_llvm ==========

#[test]
fn test_type_to_llvm_primitives() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.type_to_llvm(&ResolvedType::I8), "i8");
    assert_eq!(gen.type_to_llvm(&ResolvedType::I16), "i16");
    assert_eq!(gen.type_to_llvm(&ResolvedType::I32), "i32");
    assert_eq!(gen.type_to_llvm(&ResolvedType::I64), "i64");
    assert_eq!(gen.type_to_llvm(&ResolvedType::I128), "i128");
    assert_eq!(gen.type_to_llvm(&ResolvedType::U8), "i8");
    assert_eq!(gen.type_to_llvm(&ResolvedType::U16), "i16");
    assert_eq!(gen.type_to_llvm(&ResolvedType::U32), "i32");
    assert_eq!(gen.type_to_llvm(&ResolvedType::U64), "i64");
    assert_eq!(gen.type_to_llvm(&ResolvedType::U128), "i128");
    assert_eq!(gen.type_to_llvm(&ResolvedType::F32), "float");
    assert_eq!(gen.type_to_llvm(&ResolvedType::F64), "double");
    assert_eq!(gen.type_to_llvm(&ResolvedType::Bool), "i1");
    assert_eq!(gen.type_to_llvm(&ResolvedType::Str), "{ i8*, i64 }");
    assert_eq!(gen.type_to_llvm(&ResolvedType::Unit), "void");
}

#[test]
fn test_type_to_llvm_pointer() {
    let gen = CodeGenerator::new("test");
    let ptr = ResolvedType::Pointer(Box::new(ResolvedType::I8));
    assert_eq!(gen.type_to_llvm(&ptr), "i8*");
}

#[test]
fn test_type_to_llvm_array() {
    let gen = CodeGenerator::new("test");
    let arr = ResolvedType::Array(Box::new(ResolvedType::I64));
    assert_eq!(gen.type_to_llvm(&arr), "i64*");
}

#[test]
fn test_type_to_llvm_ref() {
    let gen = CodeGenerator::new("test");
    let r = ResolvedType::Ref(Box::new(ResolvedType::I32));
    assert_eq!(gen.type_to_llvm(&r), "i32*");
}

#[test]
fn test_type_to_llvm_ref_mut() {
    let gen = CodeGenerator::new("test");
    let r = ResolvedType::RefMut(Box::new(ResolvedType::F64));
    assert_eq!(gen.type_to_llvm(&r), "double*");
}

#[test]
fn test_type_to_llvm_range() {
    let gen = CodeGenerator::new("test");
    let range = ResolvedType::Range(Box::new(ResolvedType::I64));
    assert_eq!(gen.type_to_llvm(&range), "{ i64, i64, i1 }");
}

#[test]
fn test_type_to_llvm_tuple() {
    let gen = CodeGenerator::new("test");
    let tuple = ResolvedType::Tuple(vec![ResolvedType::I32, ResolvedType::F64]);
    assert_eq!(gen.type_to_llvm(&tuple), "{ i32, double }");
}

#[test]
fn test_type_to_llvm_slice() {
    let gen = CodeGenerator::new("test");
    let slice = ResolvedType::Slice(Box::new(ResolvedType::I64));
    assert_eq!(gen.type_to_llvm(&slice), "{ i8*, i64 }");
}

#[test]
fn test_type_to_llvm_slice_mut() {
    let gen = CodeGenerator::new("test");
    let slice = ResolvedType::SliceMut(Box::new(ResolvedType::I32));
    assert_eq!(gen.type_to_llvm(&slice), "{ i8*, i64 }");
}

#[test]
fn test_type_to_llvm_optional() {
    let gen = CodeGenerator::new("test");
    let opt = ResolvedType::Optional(Box::new(ResolvedType::I64));
    assert_eq!(gen.type_to_llvm(&opt), "{ i8, i64 }");
}

#[test]
fn test_type_to_llvm_result() {
    let gen = CodeGenerator::new("test");
    let res = ResolvedType::Result(Box::new(ResolvedType::I32), Box::new(ResolvedType::Str));
    assert_eq!(gen.type_to_llvm(&res), "{ i8, i32 }");
}

#[test]
fn test_type_to_llvm_future() {
    let gen = CodeGenerator::new("test");
    let future = ResolvedType::Future(Box::new(ResolvedType::I64));
    assert_eq!(gen.type_to_llvm(&future), "i64");
}

#[test]
fn test_type_to_llvm_never() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.type_to_llvm(&ResolvedType::Never), "void");
}

#[test]
fn test_type_to_llvm_lazy() {
    let gen = CodeGenerator::new("test");
    let lazy = ResolvedType::Lazy(Box::new(ResolvedType::I64));
    assert_eq!(gen.type_to_llvm(&lazy), "{ i1, i64, i8* }");
}

#[test]
fn test_type_to_llvm_vector() {
    let gen = CodeGenerator::new("test");
    let vec = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    assert_eq!(gen.type_to_llvm(&vec), "<4 x float>");
}

#[test]
fn test_type_to_llvm_fn_ptr() {
    let gen = CodeGenerator::new("test");
    let fn_ptr = ResolvedType::FnPtr {
        params: vec![ResolvedType::I64, ResolvedType::I64],
        ret: Box::new(ResolvedType::I64),
        is_vararg: false,
        effects: None,
    };
    assert_eq!(gen.type_to_llvm(&fn_ptr), "i64(i64, i64)*");
}

#[test]
fn test_type_to_llvm_fn_ptr_vararg() {
    let gen = CodeGenerator::new("test");
    let fn_ptr = ResolvedType::FnPtr {
        params: vec![ResolvedType::Pointer(Box::new(ResolvedType::I8))],
        ret: Box::new(ResolvedType::I32),
        is_vararg: true,
        effects: None,
    };
    assert_eq!(gen.type_to_llvm(&fn_ptr), "i32(i8*, ...)*");
}

#[test]
fn test_type_to_llvm_caching() {
    let gen = CodeGenerator::new("test");
    // Call twice - second should use cache
    let result1 = gen.type_to_llvm(&ResolvedType::I64);
    let result2 = gen.type_to_llvm(&ResolvedType::I64);
    assert_eq!(result1, result2);
    assert_eq!(result1, "i64");
}

#[test]
fn test_type_to_llvm_linear_transparent() {
    let gen = CodeGenerator::new("test");
    let linear = ResolvedType::Linear(Box::new(ResolvedType::I32));
    assert_eq!(gen.type_to_llvm(&linear), "i32");
}

#[test]
fn test_type_to_llvm_affine_transparent() {
    let gen = CodeGenerator::new("test");
    let affine = ResolvedType::Affine(Box::new(ResolvedType::F64));
    assert_eq!(gen.type_to_llvm(&affine), "double");
}

#[test]
fn test_type_to_llvm_ref_dyn_trait_fat_pointer() {
    let gen = CodeGenerator::new("test");
    let dyn_ref = ResolvedType::Ref(Box::new(ResolvedType::DynTrait {
        trait_name: "Display".to_string(),
        generics: vec![],
    }));
    // &dyn Trait should be a fat pointer, not pointer-to-fat-pointer
    let result = gen.type_to_llvm(&dyn_ref);
    assert!(result.contains("i8*"));
}

#[test]
fn test_type_to_llvm_ref_slice_fat_pointer() {
    let gen = CodeGenerator::new("test");
    let slice_ref = ResolvedType::Ref(Box::new(ResolvedType::Slice(Box::new(ResolvedType::I64))));
    // &[T] should be a fat pointer { i8*, i64 }, not pointer-to-fat-pointer
    assert_eq!(gen.type_to_llvm(&slice_ref), "{ i8*, i64 }");
}

// ========== get_integer_bits ==========

#[test]
fn test_get_integer_bits() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.get_integer_bits(&ResolvedType::I8), 8);
    assert_eq!(gen.get_integer_bits(&ResolvedType::U8), 8);
    assert_eq!(gen.get_integer_bits(&ResolvedType::I16), 16);
    assert_eq!(gen.get_integer_bits(&ResolvedType::U16), 16);
    assert_eq!(gen.get_integer_bits(&ResolvedType::I32), 32);
    assert_eq!(gen.get_integer_bits(&ResolvedType::U32), 32);
    assert_eq!(gen.get_integer_bits(&ResolvedType::I64), 64);
    assert_eq!(gen.get_integer_bits(&ResolvedType::U64), 64);
    assert_eq!(gen.get_integer_bits(&ResolvedType::I128), 128);
    assert_eq!(gen.get_integer_bits(&ResolvedType::U128), 128);
    assert_eq!(gen.get_integer_bits(&ResolvedType::F64), 0); // not integer
    assert_eq!(gen.get_integer_bits(&ResolvedType::Bool), 0);
}

// ========== get_integer_bits_from_val ==========

#[test]
fn test_get_integer_bits_from_val() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.get_integer_bits_from_val("%t0"), 64);
    assert_eq!(gen.get_integer_bits_from_val("42"), 64);
    assert_eq!(gen.get_integer_bits_from_val("-1"), 64);
    assert_eq!(gen.get_integer_bits_from_val("hello"), 0);
    assert_eq!(gen.get_integer_bits_from_val("null"), 0);
}

// ========== estimate_type_size ==========

#[test]
fn test_estimate_type_size_primitives() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.estimate_type_size("i1"), 1);
    assert_eq!(gen.estimate_type_size("i8"), 1);
    assert_eq!(gen.estimate_type_size("i16"), 2);
    assert_eq!(gen.estimate_type_size("i32"), 4);
    assert_eq!(gen.estimate_type_size("i64"), 8);
    assert_eq!(gen.estimate_type_size("i128"), 16);
    assert_eq!(gen.estimate_type_size("float"), 4);
    assert_eq!(gen.estimate_type_size("double"), 8);
    assert_eq!(gen.estimate_type_size("i8*"), 8);
}

#[test]
fn test_estimate_type_size_pointer() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.estimate_type_size("i64*"), 8);
    assert_eq!(gen.estimate_type_size("float*"), 8);
}

#[test]
fn test_estimate_type_size_named() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.estimate_type_size("%MyStruct"), 8);
}

#[test]
fn test_estimate_type_size_vector() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.estimate_type_size("<4 x float>"), 16); // 4 * 4
    assert_eq!(gen.estimate_type_size("<2 x double>"), 16); // 2 * 8
    assert_eq!(gen.estimate_type_size("<8 x i32>"), 32); // 8 * 4
}

#[test]
fn test_estimate_type_size_unknown() {
    let gen = CodeGenerator::new("test");
    assert_eq!(gen.estimate_type_size("something_unknown"), 8);
}

// ========== ClosureInfo ==========

#[test]
fn test_closure_info_clone() {
    let info = ClosureInfo {
        func_name: "__lambda_0".to_string(),
        captures: vec![("x".to_string(), "%t0".to_string())],
        is_ref_capture: false,
    };
    let cloned = info.clone();
    assert_eq!(cloned.func_name, "__lambda_0");
    assert_eq!(cloned.captures.len(), 1);
}

// ========== StructInfo ==========

#[test]
fn test_generate_struct_type() {
    let mut gen = CodeGenerator::new("test");
    let info = StructInfo {
        _name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), ResolvedType::I64),
            ("y".to_string(), ResolvedType::I64),
        ],
        _repr_c: false,
        _invariants: vec![],
    };
    gen.types.structs.insert("Point".to_string(), info.clone());
    let def = gen.generate_struct_type("Point", &info);
    assert_eq!(def, "%Point = type { i64, i64 }");
}

#[test]
fn test_generate_struct_type_mixed() {
    let mut gen = CodeGenerator::new("test");
    let info = StructInfo {
        _name: "Mixed".to_string(),
        fields: vec![
            ("flag".to_string(), ResolvedType::Bool),
            ("value".to_string(), ResolvedType::F64),
            (
                "data".to_string(),
                ResolvedType::Pointer(Box::new(ResolvedType::I8)),
            ),
        ],
        _repr_c: false,
        _invariants: vec![],
    };
    gen.types.structs.insert("Mixed".to_string(), info.clone());
    let def = gen.generate_struct_type("Mixed", &info);
    assert_eq!(def, "%Mixed = type { i1, double, i8* }");
}

// ========== EnumInfo ==========

#[test]
fn test_generate_enum_type_unit_only() {
    let gen = CodeGenerator::new("test");
    let info = EnumInfo {
        name: "Color".to_string(),
        variants: vec![
            EnumVariantInfo {
                name: "Red".to_string(),
                _tag: 0,
                fields: EnumVariantFields::Unit,
            },
            EnumVariantInfo {
                name: "Green".to_string(),
                _tag: 1,
                fields: EnumVariantFields::Unit,
            },
            EnumVariantInfo {
                name: "Blue".to_string(),
                _tag: 2,
                fields: EnumVariantFields::Unit,
            },
        ],
    };
    let def = gen.generate_enum_type("Color", &info);
    assert_eq!(def, "%Color = type { i32 }");
}

#[test]
fn test_generate_enum_type_with_payload() {
    let gen = CodeGenerator::new("test");
    let info = EnumInfo {
        name: "Value".to_string(),
        variants: vec![
            EnumVariantInfo {
                name: "Int".to_string(),
                _tag: 0,
                fields: EnumVariantFields::Tuple(vec![ResolvedType::I64]),
            },
            EnumVariantInfo {
                name: "Float".to_string(),
                _tag: 1,
                fields: EnumVariantFields::Tuple(vec![ResolvedType::F64]),
            },
            EnumVariantInfo {
                name: "None".to_string(),
                _tag: 2,
                fields: EnumVariantFields::Unit,
            },
        ],
    };
    let def = gen.generate_enum_type("Value", &info);
    assert!(def.contains("i32")); // tag
    assert!(def.contains("Value"));
}

// ========== UnionInfo ==========

#[test]
fn test_generate_union_type() {
    let gen = CodeGenerator::new("test");
    let info = UnionInfo {
        _name: "Data".to_string(),
        fields: vec![
            ("i_val".to_string(), ResolvedType::I32),
            ("f_val".to_string(), ResolvedType::F64),
        ],
    };
    let def = gen.generate_union_type("Data", &info);
    // Should use the largest field type (f64 = double, 8 bytes)
    assert!(def.contains("%Data = type { double }"));
}

#[test]
fn test_generate_union_type_single_field() {
    let gen = CodeGenerator::new("test");
    let info = UnionInfo {
        _name: "Single".to_string(),
        fields: vec![("val".to_string(), ResolvedType::I64)],
    };
    let def = gen.generate_union_type("Single", &info);
    assert_eq!(def, "%Single = type { i64 }");
}

// ========== Sizeof with nested tuples ==========

#[test]
fn test_sizeof_nested_tuple() {
    let gen = CodeGenerator::new("test");
    let inner = ResolvedType::Tuple(vec![ResolvedType::I8, ResolvedType::I8]);
    let outer = ResolvedType::Tuple(vec![inner, ResolvedType::I32]);
    // (i8, i8) = 2 bytes, i32 = 4 bytes, total = 6
    assert_eq!(gen.compute_sizeof(&outer), 6);
}

#[test]
fn test_sizeof_empty_tuple() {
    let gen = CodeGenerator::new("test");
    let tuple = ResolvedType::Tuple(vec![]);
    assert_eq!(gen.compute_sizeof(&tuple), 0);
}
