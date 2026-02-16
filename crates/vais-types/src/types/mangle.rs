//! Name mangling for generic types and functions

use super::resolved::ResolvedType;

/// Mangle a generic name with type arguments
pub fn mangle_name(base: &str, type_args: &[ResolvedType]) -> String {
    if type_args.is_empty() {
        base.to_string()
    } else {
        let mut buf = String::new();
        buf.push_str(base);
        buf.push('$');
        for (i, ty) in type_args.iter().enumerate() {
            if i > 0 {
                buf.push('_');
            }
            mangle_type_into(ty, &mut buf);
        }
        buf
    }
}

/// Mangle a generic name with both type and const arguments
pub fn mangle_name_with_consts(
    base: &str,
    type_args: &[ResolvedType],
    const_args: &[(String, i64)],
) -> String {
    if type_args.is_empty() && const_args.is_empty() {
        base.to_string()
    } else {
        let mut buf = String::new();
        buf.push_str(base);
        buf.push('$');
        let mut first = true;
        for ty in type_args {
            if !first {
                buf.push('_');
            }
            first = false;
            mangle_type_into(ty, &mut buf);
        }
        for (_, val) in const_args {
            if !first {
                buf.push('_');
            }
            first = false;
            buf.push('c');
            buf.push_str(&val.to_string());
        }
        buf
    }
}

/// Mangle a single type for use in mangled names
pub fn mangle_type(ty: &ResolvedType) -> String {
    let mut buf = String::new();
    mangle_type_into(ty, &mut buf);
    buf
}

/// Internal helper: mangle a type into a provided buffer
fn mangle_type_into(ty: &ResolvedType, buf: &mut String) {
    match ty {
        ResolvedType::I8 => buf.push_str("i8"),
        ResolvedType::I16 => buf.push_str("i16"),
        ResolvedType::I32 => buf.push_str("i32"),
        ResolvedType::I64 => buf.push_str("i64"),
        ResolvedType::I128 => buf.push_str("i128"),
        ResolvedType::U8 => buf.push_str("u8"),
        ResolvedType::U16 => buf.push_str("u16"),
        ResolvedType::U32 => buf.push_str("u32"),
        ResolvedType::U64 => buf.push_str("u64"),
        ResolvedType::U128 => buf.push_str("u128"),
        ResolvedType::F32 => buf.push_str("f32"),
        ResolvedType::F64 => buf.push_str("f64"),
        ResolvedType::Bool => buf.push_str("bool"),
        ResolvedType::Str => buf.push_str("str"),
        ResolvedType::Unit => buf.push_str("unit"),
        ResolvedType::Named { name, generics } => {
            buf.push_str(name);
            if !generics.is_empty() {
                buf.push('_');
                for (i, ty) in generics.iter().enumerate() {
                    if i > 0 {
                        buf.push('_');
                    }
                    mangle_type_into(ty, buf);
                }
            }
        }
        ResolvedType::Array(inner) => {
            buf.push_str("arr_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::Pointer(inner) => {
            buf.push_str("ptr_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::Ref(inner) => {
            buf.push_str("ref_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::RefMut(inner) => {
            buf.push_str("refmut_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::Slice(inner) => {
            buf.push_str("slice_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::SliceMut(inner) => {
            buf.push_str("slicemut_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::Optional(inner) => {
            buf.push_str("opt_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::Result(ok, err) => {
            buf.push_str("res_");
            mangle_type_into(ok, buf);
            buf.push('_');
            mangle_type_into(err, buf);
        }
        ResolvedType::Future(inner) => {
            buf.push_str("fut_");
            mangle_type_into(inner, buf);
        }
        ResolvedType::Tuple(types) => {
            buf.push_str("tup_");
            for (i, ty) in types.iter().enumerate() {
                if i > 0 {
                    buf.push('_');
                }
                mangle_type_into(ty, buf);
            }
        }
        ResolvedType::Fn { params, ret, .. } => {
            buf.push_str("fn_");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    buf.push('_');
                }
                mangle_type_into(param, buf);
            }
            buf.push('_');
            mangle_type_into(ret, buf);
        }
        ResolvedType::Generic(name) => buf.push_str(name),
        ResolvedType::ConstGeneric(name) => {
            buf.push_str("cg_");
            buf.push_str(name);
        }
        ResolvedType::ConstArray { element, size } => {
            buf.push_str("arr");
            match size.try_evaluate() {
                Some(n) => buf.push_str(&n.to_string()),
                None => buf.push_str("dyn"),
            }
            buf.push('_');
            mangle_type_into(element, buf);
        }
        ResolvedType::Var(id) => {
            buf.push('v');
            buf.push_str(&id.to_string());
        }
        ResolvedType::Vector { element, lanes } => {
            buf.push_str("vec");
            buf.push_str(&lanes.to_string());
            buf.push('_');
            mangle_type_into(element, buf);
        }
        ResolvedType::HigherKinded { name, arity } => {
            buf.push_str("hkt");
            buf.push_str(&arity.to_string());
            buf.push('_');
            buf.push_str(name);
        }
        _ => buf.push_str("unknown"),
    }
}
