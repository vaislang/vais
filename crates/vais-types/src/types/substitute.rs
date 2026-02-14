//! Type substitution functions

use std::collections::HashMap;

use super::resolved::{ConstBinOp, ResolvedConst, ResolvedType};

/// Substitute generic type parameters with concrete types
pub fn substitute_type(
    ty: &ResolvedType,
    substitutions: &HashMap<String, ResolvedType>,
) -> ResolvedType {
    match ty {
        ResolvedType::Generic(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        ResolvedType::Named { name, generics } => {
            // Early return if no substitution needed and no generics to recurse into
            if generics.is_empty() && !substitutions.contains_key(name) {
                return ty.clone();
            }

            // Check if any generic parameter changed
            let mut changed = false;
            let new_generics: Vec<ResolvedType> = generics
                .iter()
                .map(|g| {
                    let subst = substitute_type(g, substitutions);
                    if !changed && g != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();

            // If no changes, return clone of original
            if !changed {
                return ty.clone();
            }

            ResolvedType::Named {
                name: name.clone(),
                generics: new_generics,
            }
        }
        ResolvedType::Array(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Array(Box::new(new_inner))
        }
        ResolvedType::Pointer(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Pointer(Box::new(new_inner))
        }
        ResolvedType::Ref(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Ref(Box::new(new_inner))
        }
        ResolvedType::RefMut(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::RefMut(Box::new(new_inner))
        }
        ResolvedType::Slice(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Slice(Box::new(new_inner))
        }
        ResolvedType::SliceMut(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::SliceMut(Box::new(new_inner))
        }
        ResolvedType::Optional(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Optional(Box::new(new_inner))
        }
        ResolvedType::Result(ok, err) => {
            let new_ok = substitute_type(ok, substitutions);
            let new_err = substitute_type(err, substitutions);
            if ok.as_ref() == &new_ok && err.as_ref() == &new_err {
                return ty.clone();
            }
            ResolvedType::Result(Box::new(new_ok), Box::new(new_err))
        }
        ResolvedType::Future(inner) => {
            let new_inner = substitute_type(inner, substitutions);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Future(Box::new(new_inner))
        }
        ResolvedType::Tuple(types) => {
            let mut changed = false;
            let new_types: Vec<ResolvedType> = types
                .iter()
                .map(|t| {
                    let subst = substitute_type(t, substitutions);
                    if !changed && t != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();

            if !changed {
                return ty.clone();
            }
            ResolvedType::Tuple(new_types)
        }
        ResolvedType::Fn {
            params,
            ret,
            effects,
        } => {
            let mut changed = false;
            let new_params: Vec<ResolvedType> = params
                .iter()
                .map(|p| {
                    let subst = substitute_type(p, substitutions);
                    if !changed && p != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();
            let new_ret = substitute_type(ret, substitutions);
            if !changed && ret.as_ref() != &new_ret {
                changed = true;
            }

            if !changed {
                return ty.clone();
            }

            ResolvedType::Fn {
                params: new_params,
                ret: Box::new(new_ret),
                effects: effects.clone(),
            }
        }
        ResolvedType::Vector { element, lanes } => {
            let new_element = substitute_type(element, substitutions);
            if element.as_ref() == &new_element {
                return ty.clone();
            }
            ResolvedType::Vector {
                element: Box::new(new_element),
                lanes: *lanes,
            }
        }
        ResolvedType::ConstGeneric(name) => {
            // Const generics can be substituted if a mapping exists
            substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| ty.clone())
        }
        ResolvedType::ConstArray { element, size } => {
            let new_element = substitute_type(element, substitutions);
            // Substitute const parameter names in size expression
            let new_size = substitute_const(size, substitutions);

            // Check if anything changed
            if element.as_ref() == &new_element && size == &new_size {
                return ty.clone();
            }

            ResolvedType::ConstArray {
                element: Box::new(new_element),
                size: new_size,
            }
        }
        // Primitives and other types pass through unchanged
        _ => ty.clone(),
    }
}

/// Substitute const parameter names in a ResolvedConst expression
pub fn substitute_const(
    c: &ResolvedConst,
    _substitutions: &HashMap<String, ResolvedType>,
) -> ResolvedConst {
    // For now, const substitution happens through const_substitutions map
    c.clone()
}

/// Substitute const parameters with concrete values in a ResolvedConst expression
pub fn substitute_const_values(
    c: &ResolvedConst,
    const_subs: &HashMap<String, i64>,
) -> ResolvedConst {
    match c {
        ResolvedConst::Value(_) => c.clone(),
        ResolvedConst::Param(name) => {
            if let Some(&val) = const_subs.get(name) {
                ResolvedConst::Value(val)
            } else {
                c.clone()
            }
        }
        ResolvedConst::BinOp { op, left, right } => {
            let new_left = substitute_const_values(left, const_subs);
            let new_right = substitute_const_values(right, const_subs);
            // Try to evaluate if both are now concrete
            if let (Some(l), Some(r)) = (new_left.try_evaluate(), new_right.try_evaluate()) {
                let result = match op {
                    ConstBinOp::Add => l.checked_add(r),
                    ConstBinOp::Sub => l.checked_sub(r),
                    ConstBinOp::Mul => l.checked_mul(r),
                    ConstBinOp::Div => {
                        if r == 0 {
                            None
                        } else {
                            l.checked_div(r)
                        }
                    }
                };
                if let Some(val) = result {
                    return ResolvedConst::Value(val);
                }
            }
            ResolvedConst::BinOp {
                op: *op,
                left: Box::new(new_left),
                right: Box::new(new_right),
            }
        }
    }
}
