//! Type substitution functions

use std::collections::HashMap;

use super::resolved::{ConstBinOp, ResolvedConst, ResolvedType};

/// Maximum recursion depth for type substitution to prevent stack overflow
/// on circular or deeply nested type references.
const MAX_SUBSTITUTE_DEPTH: usize = 64;

/// Substitute generic type parameters with concrete types
pub fn substitute_type(
    ty: &ResolvedType,
    substitutions: &HashMap<String, ResolvedType>,
) -> ResolvedType {
    substitute_type_impl(ty, substitutions, 0)
}

fn substitute_type_impl(
    ty: &ResolvedType,
    substitutions: &HashMap<String, ResolvedType>,
    depth: usize,
) -> ResolvedType {
    if depth > MAX_SUBSTITUTE_DEPTH {
        return ty.clone();
    }
    match ty {
        ResolvedType::Generic(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        ResolvedType::Named { name, generics } => {
            // HKT application: if name itself is a substitution target (e.g., F<A> where F=Vec),
            // replace the constructor name and recurse into generic args.
            // NOTE: This HKT application logic is mirrored in inference.rs::substitute_generics().
            // Any changes here must be synchronized with that function.
            if let Some(subst) = substitutions.get(name) {
                if !generics.is_empty() {
                    // F<A> where F→Vec, A→i64 becomes Vec<i64>
                    let concrete_name = match subst {
                        ResolvedType::Named {
                            name: concrete, ..
                        }
                        | ResolvedType::HigherKinded {
                            name: concrete, ..
                        } => concrete.clone(),
                        _ => name.clone(),
                    };
                    let new_generics: Vec<ResolvedType> = generics
                        .iter()
                        .map(|g| substitute_type_impl(g, substitutions, depth + 1))
                        .collect();
                    return ResolvedType::Named {
                        name: concrete_name,
                        generics: new_generics,
                    };
                } else {
                    // No generics applied — direct substitution (e.g., bare F)
                    return subst.clone();
                }
            }

            // Early return if no generics to recurse into
            if generics.is_empty() {
                return ty.clone();
            }

            // Check if any generic parameter changed
            let mut changed = false;
            let new_generics: Vec<ResolvedType> = generics
                .iter()
                .map(|g| {
                    let subst = substitute_type_impl(g, substitutions, depth + 1);
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
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Array(Box::new(new_inner))
        }
        ResolvedType::Pointer(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Pointer(Box::new(new_inner))
        }
        ResolvedType::Ref(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Ref(Box::new(new_inner))
        }
        ResolvedType::RefMut(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::RefMut(Box::new(new_inner))
        }
        ResolvedType::Slice(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Slice(Box::new(new_inner))
        }
        ResolvedType::SliceMut(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::SliceMut(Box::new(new_inner))
        }
        ResolvedType::Optional(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Optional(Box::new(new_inner))
        }
        ResolvedType::Result(ok, err) => {
            let new_ok = substitute_type_impl(ok, substitutions, depth + 1);
            let new_err = substitute_type_impl(err, substitutions, depth + 1);
            if ok.as_ref() == &new_ok && err.as_ref() == &new_err {
                return ty.clone();
            }
            ResolvedType::Result(Box::new(new_ok), Box::new(new_err))
        }
        ResolvedType::Future(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
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
                    let subst = substitute_type_impl(t, substitutions, depth + 1);
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
                    let subst = substitute_type_impl(p, substitutions, depth + 1);
                    if !changed && p != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();
            let new_ret = substitute_type_impl(ret, substitutions, depth + 1);
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
            let new_element = substitute_type_impl(element, substitutions, depth + 1);
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
            let new_element = substitute_type_impl(element, substitutions, depth + 1);
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
        // HigherKinded: substitute if a mapping exists
        ResolvedType::HigherKinded { name, .. } => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        ResolvedType::Map(k, v) => {
            let new_k = substitute_type_impl(k, substitutions, depth + 1);
            let new_v = substitute_type_impl(v, substitutions, depth + 1);
            if k.as_ref() == &new_k && v.as_ref() == &new_v {
                return ty.clone();
            }
            ResolvedType::Map(Box::new(new_k), Box::new(new_v))
        }
        ResolvedType::Range(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Range(Box::new(new_inner))
        }
        ResolvedType::FnPtr {
            params,
            ret,
            is_vararg,
            effects,
        } => {
            let mut changed = false;
            let new_params: Vec<ResolvedType> = params
                .iter()
                .map(|p| {
                    let subst = substitute_type_impl(p, substitutions, depth + 1);
                    if !changed && p != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();
            let new_ret = substitute_type_impl(ret, substitutions, depth + 1);
            if !changed && ret.as_ref() != &new_ret {
                changed = true;
            }

            if !changed {
                return ty.clone();
            }

            ResolvedType::FnPtr {
                params: new_params,
                ret: Box::new(new_ret),
                is_vararg: *is_vararg,
                effects: effects.clone(),
            }
        }
        ResolvedType::DynTrait {
            trait_name,
            generics,
        } => {
            let mut changed = false;
            let new_generics: Vec<ResolvedType> = generics
                .iter()
                .map(|g| {
                    let subst = substitute_type_impl(g, substitutions, depth + 1);
                    if !changed && g != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();

            if !changed {
                return ty.clone();
            }

            ResolvedType::DynTrait {
                trait_name: trait_name.clone(),
                generics: new_generics,
            }
        }
        ResolvedType::ImplTrait { bounds: _ } => {
            // Bounds are String trait names, no type substitution needed
            ty.clone()
        }
        ResolvedType::Associated {
            base,
            trait_name,
            assoc_name,
            generics,
        } => {
            let new_base = substitute_type_impl(base, substitutions, depth + 1);
            let mut changed = base.as_ref() != &new_base;

            let new_generics: Vec<ResolvedType> = generics
                .iter()
                .map(|g| {
                    let subst = substitute_type_impl(g, substitutions, depth + 1);
                    if !changed && g != &subst {
                        changed = true;
                    }
                    subst
                })
                .collect();

            if !changed {
                return ty.clone();
            }

            ResolvedType::Associated {
                base: Box::new(new_base),
                trait_name: trait_name.clone(),
                assoc_name: assoc_name.clone(),
                generics: new_generics,
            }
        }
        ResolvedType::Lazy(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Lazy(Box::new(new_inner))
        }
        ResolvedType::Linear(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Linear(Box::new(new_inner))
        }
        ResolvedType::Affine(inner) => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::Affine(Box::new(new_inner))
        }
        ResolvedType::Dependent {
            var_name,
            base,
            predicate,
        } => {
            let new_base = substitute_type_impl(base, substitutions, depth + 1);
            if base.as_ref() == &new_base {
                return ty.clone();
            }
            ResolvedType::Dependent {
                var_name: var_name.clone(),
                base: Box::new(new_base),
                predicate: predicate.clone(),
            }
        }
        ResolvedType::RefLifetime { lifetime, inner } => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::RefLifetime {
                lifetime: lifetime.clone(),
                inner: Box::new(new_inner),
            }
        }
        ResolvedType::RefMutLifetime { lifetime, inner } => {
            let new_inner = substitute_type_impl(inner, substitutions, depth + 1);
            if inner.as_ref() == &new_inner {
                return ty.clone();
            }
            ResolvedType::RefMutLifetime {
                lifetime: lifetime.clone(),
                inner: Box::new(new_inner),
            }
        }
        ResolvedType::Lifetime(_) => {
            // Lifetime parameters are not substituted
            ty.clone()
        }
        // Primitives pass through unchanged
        ResolvedType::I8
        | ResolvedType::I16
        | ResolvedType::I32
        | ResolvedType::I64
        | ResolvedType::I128
        | ResolvedType::U8
        | ResolvedType::U16
        | ResolvedType::U32
        | ResolvedType::U64
        | ResolvedType::U128
        | ResolvedType::F32
        | ResolvedType::F64
        | ResolvedType::Bool
        | ResolvedType::Str
        | ResolvedType::Unit
        | ResolvedType::Unknown
        | ResolvedType::Never
        | ResolvedType::Var(_) => ty.clone(),
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
        ResolvedConst::Negate(inner) => {
            let new_inner = substitute_const_values(inner, const_subs);
            if let Some(val) = new_inner.try_evaluate() {
                if let Some(neg_val) = val.checked_neg() {
                    ResolvedConst::Value(neg_val)
                } else {
                    ResolvedConst::Negate(Box::new(new_inner))
                }
            } else {
                ResolvedConst::Negate(Box::new(new_inner))
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
                    ConstBinOp::Mod => {
                        if r == 0 {
                            None
                        } else {
                            l.checked_rem(r)
                        }
                    }
                    ConstBinOp::BitAnd => Some(l & r),
                    ConstBinOp::BitOr => Some(l | r),
                    ConstBinOp::BitXor => Some(l ^ r),
                    ConstBinOp::Shl => Some(l.wrapping_shl(r as u32)),
                    ConstBinOp::Shr => Some(l.wrapping_shr(r as u32)),
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
