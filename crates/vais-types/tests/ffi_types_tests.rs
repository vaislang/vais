//! Type checking tests for FFI features

use vais_types::{TypeChecker, ResolvedType};
use vais_ast::*;

#[test]
fn test_function_pointer_type() {
    let mut checker = TypeChecker::new();

    // Register a function pointer type
    let fn_ptr_type = ResolvedType::FnPtr {
        params: vec![ResolvedType::I32, ResolvedType::I32],
        ret: Box::new(ResolvedType::I64),
        is_vararg: false,
        effects: None,
    };

    // Check display
    assert_eq!(format!("{}", fn_ptr_type), "fn(i32,i32)->i64");
}

#[test]
fn test_vararg_function_pointer_type() {
    let fn_ptr_type = ResolvedType::FnPtr {
        params: vec![ResolvedType::Pointer(Box::new(ResolvedType::I8))],
        ret: Box::new(ResolvedType::I32),
        is_vararg: true,
        effects: None,
    };

    let display_str = format!("{}", fn_ptr_type);
    assert!(display_str.contains("..."));
    assert!(display_str.contains("i8"));
}

#[test]
fn test_repr_c_struct() {
    use std::collections::HashMap;

    let mut fields = HashMap::new();
    fields.insert("x".to_string(), ResolvedType::I32);
    fields.insert("y".to_string(), ResolvedType::I32);

    let struct_def = vais_types::StructDef {
        name: "Point".to_string(),
        generics: vec![],
        fields,
        methods: HashMap::new(),
        repr_c: true,
    };

    assert!(struct_def.repr_c);
}

#[test]
fn test_vararg_function_signature() {
    use std::collections::HashMap;

    let func_sig = vais_types::FunctionSig {
        name: "printf".to_string(),
        generics: vec![],
        generic_bounds: HashMap::new(),
        params: vec![
            ("fmt".to_string(), ResolvedType::Pointer(Box::new(ResolvedType::I8)), false),
        ],
        ret: ResolvedType::I32,
        is_async: false,
        is_vararg: true,
        required_params: None,
        contracts: None,
        effect_annotation: vais_types::EffectAnnotation::Infer,
        inferred_effects: None,
    };

    assert!(func_sig.is_vararg);
    assert_eq!(func_sig.params.len(), 1);
}
