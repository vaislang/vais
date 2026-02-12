//! Vais Type → JavaScript type mapping

use vais_ast::Type;

/// Represents a JavaScript type for documentation/JSDoc purposes
#[derive(Debug, Clone, PartialEq)]
pub enum JsType {
    Number,
    BigInt,
    String,
    Boolean,
    Void,
    Array(Box<JsType>),
    Map(Box<JsType>, Box<JsType>),
    Object(String),
    Function(Vec<JsType>, Box<JsType>),
    Nullable(Box<JsType>),
    Any,
}

impl std::fmt::Display for JsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsType::Number => write!(f, "number"),
            JsType::BigInt => write!(f, "bigint"),
            JsType::String => write!(f, "string"),
            JsType::Boolean => write!(f, "boolean"),
            JsType::Void => write!(f, "void"),
            JsType::Array(inner) => write!(f, "Array<{inner}>"),
            JsType::Map(k, v) => write!(f, "Map<{k}, {v}>"),
            JsType::Object(name) => write!(f, "{name}"),
            JsType::Function(params, ret) => {
                let params_str: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "({}) => {ret}", params_str.join(", "))
            }
            JsType::Nullable(inner) => write!(f, "{inner} | null"),
            JsType::Any => write!(f, "any"),
        }
    }
}

/// Convert a Vais AST Type to a JS type annotation string (for JSDoc)
pub fn type_to_js(ty: &Type) -> String {
    match ty {
        Type::Named { name, generics: _ } => match name.as_str() {
            "i8" | "i16" | "i32" | "u8" | "u16" | "u32" | "f32" | "f64" => "number".to_string(),
            "i64" | "u64" | "i128" | "u128" => "number".to_string(),
            "bool" => "boolean".to_string(),
            "str" | "String" => "string".to_string(),
            _ => {
                name.clone()
            }
        },
        Type::Unit => "void".to_string(),
        Type::Array(inner) => format!("Array<{}>", type_to_js(&inner.node)),
        Type::ConstArray { element, .. } => format!("Array<{}>", type_to_js(&element.node)),
        Type::Map(k, v) => format!("Map<{}, {}>", type_to_js(&k.node), type_to_js(&v.node)),
        Type::Tuple(types) => {
            let parts: Vec<String> = types.iter().map(|t| type_to_js(&t.node)).collect();
            format!("[{}]", parts.join(", "))
        }
        Type::Optional(inner) => format!("{} | null", type_to_js(&inner.node)),
        Type::Result(inner) => type_to_js(&inner.node),
        Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
            let params_str: Vec<String> = params.iter().map(|p| type_to_js(&p.node)).collect();
            format!("({}) => {}", params_str.join(", "), type_to_js(&ret.node))
        }
        Type::Ref(inner) | Type::RefMut(inner) => type_to_js(&inner.node),
        Type::Pointer(inner) => type_to_js(&inner.node),
        Type::Lazy(inner) => format!("() => {}", type_to_js(&inner.node)),
        Type::DynTrait { trait_name, .. } => trait_name.clone(),
        Type::Infer => "any".to_string(),
        _ => "any".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_ast::Span;
    use vais_ast::Spanned;

    #[test]
    fn test_primitive_types() {
        let ty = Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        };
        assert_eq!(type_to_js(&ty), "number");

        let ty = Type::Named {
            name: "bool".to_string(),
            generics: vec![],
        };
        assert_eq!(type_to_js(&ty), "boolean");

        let ty = Type::Named {
            name: "str".to_string(),
            generics: vec![],
        };
        assert_eq!(type_to_js(&ty), "string");
    }

    #[test]
    fn test_array_type() {
        let ty = Type::Array(Box::new(Spanned::new(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            Span::new(0, 3),
        )));
        assert_eq!(type_to_js(&ty), "Array<number>");
    }

    #[test]
    fn test_optional_type() {
        let ty = Type::Optional(Box::new(Spanned::new(
            Type::Named {
                name: "str".to_string(),
                generics: vec![],
            },
            Span::new(0, 3),
        )));
        assert_eq!(type_to_js(&ty), "string | null");
    }

    #[test]
    fn test_unit_type() {
        assert_eq!(type_to_js(&Type::Unit), "void");
    }
}
