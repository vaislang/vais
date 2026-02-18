//! Validation of dyn Trait object safety.

use super::*;

impl TypeChecker {
    /// Validate object safety for dyn Trait types
    pub(crate) fn validate_dyn_trait_object_safety(&mut self, ty: &ResolvedType) {
        match ty {
            ResolvedType::DynTrait {
                trait_name,
                generics,
            } => {
                if let Some(trait_def) = self.traits.get(trait_name) {
                    if let Err(violations) = object_safety::check_object_safety(trait_def) {
                        let mut error_msg = format!(
                            "trait `{}` cannot be used as a trait object (not object-safe)",
                            trait_name
                        );
                        for violation in &violations {
                            error_msg.push_str(&format!("\n  - {}", violation.description()));
                        }
                        self.warnings.push(error_msg);
                    }
                }
                // Check generics recursively
                for gen in generics {
                    self.validate_dyn_trait_object_safety(gen);
                }
            }
            // Recursively check compound types
            ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Pointer(inner)
            | ResolvedType::Array(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Range(inner) => {
                self.validate_dyn_trait_object_safety(inner);
            }
            ResolvedType::Result(ok, err) => {
                self.validate_dyn_trait_object_safety(ok);
                self.validate_dyn_trait_object_safety(err);
            }
            ResolvedType::ConstArray { element, .. } => {
                self.validate_dyn_trait_object_safety(element);
            }
            ResolvedType::Map(k, v) => {
                self.validate_dyn_trait_object_safety(k);
                self.validate_dyn_trait_object_safety(v);
            }
            ResolvedType::Tuple(types) => {
                for t in types {
                    self.validate_dyn_trait_object_safety(t);
                }
            }
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                for p in params {
                    self.validate_dyn_trait_object_safety(p);
                }
                self.validate_dyn_trait_object_safety(ret);
            }
            ResolvedType::Named { generics, .. } => {
                for g in generics {
                    self.validate_dyn_trait_object_safety(g);
                }
            }
            _ => {}
        }
    }
}
