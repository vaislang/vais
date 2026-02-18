//! Format type expressions

use super::*;

impl Formatter {
    /// Format a type
    pub(crate) fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::Named { name, generics } => {
                if generics.is_empty() {
                    name.to_string()
                } else {
                    let gens: Vec<String> =
                        generics.iter().map(|g| self.format_type(&g.node)).collect();
                    format!("{}<{}>", name, gens.join(", "))
                }
            }
            Type::Array(inner) => format!("[{}]", self.format_type(&inner.node)),
            Type::ConstArray { element, size } => {
                format!("[{}; {}]", self.format_type(&element.node), size)
            }
            Type::Map(key, value) => {
                format!(
                    "[{}:{}]",
                    self.format_type(&key.node),
                    self.format_type(&value.node)
                )
            }
            Type::Tuple(types) => {
                let ts: Vec<String> = types.iter().map(|t| self.format_type(&t.node)).collect();
                format!("({})", ts.join(", "))
            }
            Type::Optional(inner) => format!("{}?", self.format_type(&inner.node)),
            Type::Result(inner) => format!("{}!", self.format_type(&inner.node)),
            Type::Pointer(inner) => format!("*{}", self.format_type(&inner.node)),
            Type::Ref(inner) => format!("&{}", self.format_type(&inner.node)),
            Type::RefMut(inner) => format!("&mut {}", self.format_type(&inner.node)),
            Type::Slice(inner) => format!("&[{}]", self.format_type(&inner.node)),
            Type::SliceMut(inner) => format!("&mut [{}]", self.format_type(&inner.node)),
            Type::Fn { params, ret } => {
                let ps: Vec<String> = params.iter().map(|p| self.format_type(&p.node)).collect();
                format!("({}) -> {}", ps.join(", "), self.format_type(&ret.node))
            }
            Type::Unit => "()".to_string(),
            Type::Infer => "_".to_string(),
            Type::DynTrait {
                trait_name,
                generics,
            } => {
                if generics.is_empty() {
                    format!("dyn {}", trait_name)
                } else {
                    let gens: Vec<String> =
                        generics.iter().map(|g| self.format_type(&g.node)).collect();
                    format!("dyn {}<{}>", trait_name, gens.join(", "))
                }
            }
            Type::FnPtr {
                params,
                ret,
                is_vararg,
            } => {
                let ps: Vec<String> = params.iter().map(|p| self.format_type(&p.node)).collect();
                let vararg_str = if *is_vararg { ", ..." } else { "" };
                format!(
                    "fn({}{}) -> {}",
                    ps.join(", "),
                    vararg_str,
                    self.format_type(&ret.node)
                )
            }
            Type::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => {
                let base_str = if let Some(tn) = trait_name {
                    format!(
                        "<{} as {}>::{}",
                        self.format_type(&base.node),
                        tn,
                        assoc_name
                    )
                } else {
                    format!("{}::{}", self.format_type(&base.node), assoc_name)
                };
                // Add GAT generic arguments if present
                if generics.is_empty() {
                    base_str
                } else {
                    let gen_strs: Vec<String> =
                        generics.iter().map(|g| self.format_type(&g.node)).collect();
                    format!("{}<{}>", base_str, gen_strs.join(", "))
                }
            }
            Type::Linear(inner) => format!("linear {}", self.format_type(&inner.node)),
            Type::Affine(inner) => format!("affine {}", self.format_type(&inner.node)),
            Type::Dependent {
                var_name,
                base,
                predicate,
            } => {
                format!(
                    "{{{}: {} | {:?}}}",
                    var_name,
                    self.format_type(&base.node),
                    predicate.node
                )
            }
            Type::RefLifetime { lifetime, inner } => {
                format!("&'{} {}", lifetime, self.format_type(&inner.node))
            }
            Type::RefMutLifetime { lifetime, inner } => {
                format!("&'{} mut {}", lifetime, self.format_type(&inner.node))
            }
            Type::Lazy(inner) => {
                format!("Lazy<{}>", self.format_type(&inner.node))
            }
            Type::ImplTrait { bounds } => {
                let bs: Vec<&str> = bounds.iter().map(|b| b.node.as_str()).collect();
                format!("X {}", bs.join(" + "))
            }
        }
    }
}
