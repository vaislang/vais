//! Lifetime validation and public API entry points.

use super::checker::BorrowChecker;
use super::*;

/// Check a MIR body for borrow errors.
pub fn check_body(body: &Body) -> Vec<BorrowError> {
    // Apply lifetime elision rules before checking
    let body_with_elision = apply_lifetime_elision(body);
    let mut checker = BorrowChecker::new(&body_with_elision);
    checker.check()
}

/// Apply lifetime elision rules to a body.
///
/// Rule 1: If there is exactly one input lifetime in parameters,
/// and the return type has a lifetime, infer that they are the same.
fn apply_lifetime_elision(body: &Body) -> Body {
    let mut new_body = body.clone();

    // Skip if explicit bounds already exist
    if !new_body.lifetime_bounds.is_empty() {
        return new_body;
    }

    // Collect lifetimes from parameters
    let mut param_lifetimes = Vec::new();
    for param_ty in &new_body.params {
        if let Some(lt) = extract_lifetime(param_ty) {
            param_lifetimes.push(lt);
        }
    }

    // Collect lifetimes from return type
    let return_lifetimes = extract_all_lifetimes(&new_body.return_type);

    // Rule 1: Single input lifetime -> all output lifetimes are the same
    if param_lifetimes.len() == 1 && !return_lifetimes.is_empty() {
        let input_lt = &param_lifetimes[0];
        for output_lt in &return_lifetimes {
            if input_lt != output_lt {
                // Add bound: output_lt: input_lt (output must outlive input)
                new_body
                    .lifetime_bounds
                    .push((output_lt.clone(), vec![input_lt.clone()]));
            }
        }
    }

    new_body
}

/// Extract the first lifetime from a type.
fn extract_lifetime(ty: &MirType) -> Option<String> {
    match ty {
        MirType::RefLifetime { lifetime, .. } | MirType::RefMutLifetime { lifetime, .. } => {
            Some(lifetime.clone())
        }
        MirType::Tuple(elems) => {
            for elem in elems {
                if let Some(lt) = extract_lifetime(elem) {
                    return Some(lt);
                }
            }
            None
        }
        _ => None,
    }
}

/// Extract all lifetimes from a type.
fn extract_all_lifetimes(ty: &MirType) -> Vec<String> {
    let mut lifetimes = Vec::new();
    collect_lifetimes(ty, &mut lifetimes);
    lifetimes
}

fn collect_lifetimes(ty: &MirType, lifetimes: &mut Vec<String>) {
    match ty {
        MirType::RefLifetime { lifetime, inner } | MirType::RefMutLifetime { lifetime, inner } => {
            lifetimes.push(lifetime.clone());
            collect_lifetimes(inner, lifetimes);
        }
        MirType::Tuple(elems) => {
            for elem in elems {
                collect_lifetimes(elem, lifetimes);
            }
        }
        MirType::Array(elem) | MirType::Pointer(elem) | MirType::Ref(elem) => {
            collect_lifetimes(elem, lifetimes);
        }
        MirType::Function { params, ret } => {
            for param in params {
                collect_lifetimes(param, lifetimes);
            }
            collect_lifetimes(ret, lifetimes);
        }
        _ => {}
    }
}

/// Check all bodies in a MIR module for borrow errors.
pub fn check_module(module: &MirModule) -> Vec<BorrowError> {
    let mut all_errors = Vec::new();

    for body in &module.bodies {
        let errors = check_body(body);
        all_errors.extend(errors);
    }

    all_errors
}
