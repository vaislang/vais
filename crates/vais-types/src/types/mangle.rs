//! Name mangling for generic types and functions

use super::resolved::ResolvedType;

/// Mangle a generic name with type arguments
pub fn mangle_name(base: &str, type_args: &[ResolvedType]) -> String {
    if type_args.is_empty() {
        base.to_string()
    } else {
        let args_str = type_args
            .iter()
            .map(mangle_type)
            .collect::<Vec<_>>()
            .join("_");
        format!("{}${}", base, args_str)
    }
}

/// Mangle a generic name with both type and const arguments
pub fn mangle_name_with_consts(
    base: &str,
    type_args: &[ResolvedType],
    const_args: &[(String, i64)],
) -> String {
    let mut parts = Vec::new();
    for ty in type_args {
        parts.push(mangle_type(ty));
    }
    for (_, val) in const_args {
        parts.push(format!("c{}", val));
    }
    if parts.is_empty() {
        base.to_string()
    } else {
        format!("{}${}", base, parts.join("_"))
    }
}

/// Mangle a single type for use in mangled names
pub fn mangle_type(ty: &ResolvedType) -> String {
    match ty {
        ResolvedType::I8 => "i8".to_string(),
        ResolvedType::I16 => "i16".to_string(),
        ResolvedType::I32 => "i32".to_string(),
        ResolvedType::I64 => "i64".to_string(),
        ResolvedType::I128 => "i128".to_string(),
        ResolvedType::U8 => "u8".to_string(),
        ResolvedType::U16 => "u16".to_string(),
        ResolvedType::U32 => "u32".to_string(),
        ResolvedType::U64 => "u64".to_string(),
        ResolvedType::U128 => "u128".to_string(),
        ResolvedType::F32 => "f32".to_string(),
        ResolvedType::F64 => "f64".to_string(),
        ResolvedType::Bool => "bool".to_string(),
        ResolvedType::Str => "str".to_string(),
        ResolvedType::Unit => "unit".to_string(),
        ResolvedType::Named { name, generics } => {
            if generics.is_empty() {
                name.clone()
            } else {
                let args = generics
                    .iter()
                    .map(mangle_type)
                    .collect::<Vec<_>>()
                    .join("_");
                format!("{}_{}", name, args)
            }
        }
        ResolvedType::Array(inner) => format!("arr_{}", mangle_type(inner)),
        ResolvedType::Pointer(inner) => format!("ptr_{}", mangle_type(inner)),
        ResolvedType::Ref(inner) => format!("ref_{}", mangle_type(inner)),
        ResolvedType::RefMut(inner) => format!("refmut_{}", mangle_type(inner)),
        ResolvedType::Slice(inner) => format!("slice_{}", mangle_type(inner)),
        ResolvedType::SliceMut(inner) => format!("slicemut_{}", mangle_type(inner)),
        ResolvedType::Optional(inner) => format!("opt_{}", mangle_type(inner)),
        ResolvedType::Result(ok, err) => format!("res_{}_{}", mangle_type(ok), mangle_type(err)),
        ResolvedType::Future(inner) => format!("fut_{}", mangle_type(inner)),
        ResolvedType::Tuple(types) => {
            let args = types.iter().map(mangle_type).collect::<Vec<_>>().join("_");
            format!("tup_{}", args)
        }
        ResolvedType::Fn { params, ret, .. } => {
            let params_str = params.iter().map(mangle_type).collect::<Vec<_>>().join("_");
            format!("fn_{}_{}", params_str, mangle_type(ret))
        }
        ResolvedType::Generic(name) => name.clone(),
        ResolvedType::ConstGeneric(name) => format!("cg_{}", name),
        ResolvedType::ConstArray { element, size } => {
            let size_str = match size.try_evaluate() {
                Some(n) => format!("{}", n),
                None => "dyn".to_string(),
            };
            format!("arr{}_{}", size_str, mangle_type(element))
        }
        ResolvedType::Var(id) => format!("v{}", id),
        ResolvedType::Vector { element, lanes } => format!("vec{}_{}", lanes, mangle_type(element)),
        ResolvedType::HigherKinded { name, arity } => format!("hkt{}_{}", arity, name),
        _ => "unknown".to_string(),
    }
}
