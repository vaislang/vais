//! Type conversion and formatting helpers for LSP type resolution

use super::LspType;
use vais_ast::Type;

/// Convert AST Type to LspType
pub(crate) fn ast_type_to_lsp(ty: &Type) -> LspType {
    match ty {
        Type::Named { name, generics } => match name.as_str() {
            "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "f32" | "f64" | "bool" | "str" | "isize" | "usize" | "char" => {
                LspType::Primitive(name.clone())
            }
            "Option" => {
                let inner = generics
                    .first()
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                LspType::Optional(Box::new(inner))
            }
            "Result" => {
                let ok = generics
                    .first()
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                let err = generics
                    .get(1)
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                LspType::Result(Box::new(ok), Box::new(err))
            }
            "Vec" => {
                let inner = generics
                    .first()
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                LspType::Array(Box::new(inner))
            }
            _ => LspType::Named(name.clone()),
        },
        Type::Array(inner) => LspType::Array(Box::new(ast_type_to_lsp(&inner.node))),
        Type::Tuple(types) => {
            let inner: Vec<LspType> = types.iter().map(|t| ast_type_to_lsp(&t.node)).collect();
            LspType::Tuple(inner)
        }
        Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
            let param_types: Vec<LspType> =
                params.iter().map(|p| ast_type_to_lsp(&p.node)).collect();
            let ret_type = Box::new(ast_type_to_lsp(&ret.node));
            LspType::Function {
                params: param_types,
                ret: ret_type,
            }
        }
        Type::Optional(inner) => LspType::Optional(Box::new(ast_type_to_lsp(&inner.node))),
        Type::Result(inner) => LspType::Result(
            Box::new(ast_type_to_lsp(&inner.node)),
            Box::new(LspType::Unknown),
        ),
        Type::Ref(inner) | Type::RefMut(inner) => ast_type_to_lsp(&inner.node),
        Type::Pointer(inner) => ast_type_to_lsp(&inner.node),
        Type::Slice(inner) | Type::SliceMut(inner) => {
            LspType::Array(Box::new(ast_type_to_lsp(&inner.node)))
        }
        Type::Unit => LspType::Unit,
        Type::Infer => LspType::Unknown,
        _ => LspType::Unknown,
    }
}

/// Format AST Type as display string
pub(crate) fn format_type(ty: &Type) -> String {
    match ty {
        Type::Named { name, generics } => {
            if generics.is_empty() {
                name.clone()
            } else {
                let gen_strs: Vec<String> = generics.iter().map(|g| format_type(&g.node)).collect();
                format!("{}<{}>", name, gen_strs.join(", "))
            }
        }
        Type::Array(inner) => format!("[{}]", format_type(&inner.node)),
        Type::Tuple(types) => {
            let strs: Vec<String> = types.iter().map(|t| format_type(&t.node)).collect();
            format!("({})", strs.join(", "))
        }
        Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
            let param_strs: Vec<String> = params.iter().map(|p| format_type(&p.node)).collect();
            format!(
                "fn({}) -> {}",
                param_strs.join(", "),
                format_type(&ret.node)
            )
        }
        Type::Unit => "()".to_string(),
        Type::Infer => "_".to_string(),
        Type::Pointer(inner) => format!("*{}", format_type(&inner.node)),
        Type::Ref(inner) => format!("&{}", format_type(&inner.node)),
        Type::RefMut(inner) => format!("&mut {}", format_type(&inner.node)),
        Type::Slice(inner) => format!("&[{}]", format_type(&inner.node)),
        Type::SliceMut(inner) => format!("&mut [{}]", format_type(&inner.node)),
        Type::Optional(inner) => format!("{}?", format_type(&inner.node)),
        Type::Result(inner) => format!("{}!", format_type(&inner.node)),
        _ => format!("{:?}", ty),
    }
}

/// Parse a simple type string back into LspType (for method return types)
pub(crate) fn parse_type_string(s: &str) -> LspType {
    let s = s.trim();
    match s {
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32"
        | "f64" | "bool" | "str" | "isize" | "usize" | "char" => LspType::Primitive(s.to_string()),
        "()" => LspType::Unit,
        _ if s.starts_with("Option<") => LspType::Optional(Box::new(LspType::Unknown)),
        _ if s.starts_with("Result<") => {
            LspType::Result(Box::new(LspType::Unknown), Box::new(LspType::Unknown))
        }
        _ if s.starts_with("Vec<") || s.starts_with('[') => {
            LspType::Array(Box::new(LspType::Unknown))
        }
        _ if s.starts_with("fn(") => LspType::Function {
            params: vec![],
            ret: Box::new(LspType::Unknown),
        },
        _ => LspType::Named(s.to_string()),
    }
}
