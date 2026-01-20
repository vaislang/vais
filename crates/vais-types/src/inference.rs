//! Type inference logic for the Vais type system
//!
//! This module contains the type inference algorithms including unification,
//! substitution, and fresh type variable generation.

use std::collections::HashMap;
use vais_ast::{Expr, Spanned};
use crate::types::{ResolvedType, TypeError, TypeResult, FunctionSig, GenericInstantiation};
use crate::TypeChecker;

impl TypeChecker {
    /// Unify two types
    pub(crate) fn unify(&mut self, expected: &ResolvedType, found: &ResolvedType) -> TypeResult<()> {
        let expected = self.apply_substitutions(expected);
        let found = self.apply_substitutions(found);

        if expected == found {
            return Ok(());
        }

        match (&expected, &found) {
            // Type variables can unify with anything
            (ResolvedType::Var(id), t) | (t, ResolvedType::Var(id)) => {
                self.substitutions.insert(*id, t.clone());
                Ok(())
            }
            // Unknown type unifies with anything (used as placeholder)
            (ResolvedType::Unknown, _) | (_, ResolvedType::Unknown) => Ok(()),
            // Generic type parameters match with any type (type erasure)
            (ResolvedType::Generic(_), _) | (_, ResolvedType::Generic(_)) => Ok(()),
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.unify(a, b),
            (ResolvedType::Optional(a), ResolvedType::Optional(b)) => self.unify(a, b),
            (ResolvedType::Result(a), ResolvedType::Result(b)) => self.unify(a, b),
            (ResolvedType::Ref(a), ResolvedType::Ref(b)) => self.unify(a, b),
            (ResolvedType::RefMut(a), ResolvedType::RefMut(b)) => self.unify(a, b),
            (ResolvedType::Pointer(a), ResolvedType::Pointer(b)) => self.unify(a, b),
            (ResolvedType::Range(a), ResolvedType::Range(b)) => self.unify(a, b),
            (ResolvedType::Future(a), ResolvedType::Future(b)) => self.unify(a, b),
            (ResolvedType::Tuple(a), ResolvedType::Tuple(b)) if a.len() == b.len() => {
                for (ta, tb) in a.iter().zip(b.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn {
                    params: pa,
                    ret: ra,
                },
                ResolvedType::Fn {
                    params: pb,
                    ret: rb,
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            // Named types with generics
            (
                ResolvedType::Named { name: na, generics: ga },
                ResolvedType::Named { name: nb, generics: gb },
            ) if na == nb && ga.len() == gb.len() => {
                for (ta, tb) in ga.iter().zip(gb.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            // Allow implicit integer type conversions (widening and narrowing)
            (a, b) if Self::is_integer_type(a) && Self::is_integer_type(b) => Ok(()),
            _ => Err(TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
                span: None,
            }),
        }
    }

    /// Check if type is an integer type
    pub(crate) fn is_integer_type(ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
        )
    }

    /// Apply substitutions to a type
    pub(crate) fn apply_substitutions(&self, ty: &ResolvedType) -> ResolvedType {
        match ty {
            ResolvedType::Var(id) => {
                if let Some(subst) = self.substitutions.get(id) {
                    self.apply_substitutions(subst)
                } else {
                    ty.clone()
                }
            }
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.apply_substitutions(t)).collect())
            }
            ResolvedType::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                ret: Box::new(self.apply_substitutions(ret)),
            },
            _ => ty.clone(),
        }
    }

    /// Create a fresh type variable
    pub(crate) fn fresh_type_var(&self) -> ResolvedType {
        let id = self.next_type_var.get();
        self.next_type_var.set(id + 1);
        ResolvedType::Var(id)
    }

    /// Substitute generic type parameters with concrete types
    pub(crate) fn substitute_generics(&self, ty: &ResolvedType, substitutions: &HashMap<String, ResolvedType>) -> ResolvedType {
        match ty {
            ResolvedType::Generic(name) => {
                substitutions.get(name).cloned().unwrap_or_else(|| ty.clone())
            }
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Map(key, value) => ResolvedType::Map(
                Box::new(self.substitute_generics(key, substitutions)),
                Box::new(self.substitute_generics(value, substitutions)),
            ),
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.substitute_generics(t, substitutions)).collect())
            }
            ResolvedType::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.substitute_generics(p, substitutions)).collect(),
                ret: Box::new(self.substitute_generics(ret, substitutions)),
            },
            ResolvedType::Named { name, generics } => ResolvedType::Named {
                name: name.clone(),
                generics: generics.iter().map(|g| self.substitute_generics(g, substitutions)).collect(),
            },
            _ => ty.clone(),
        }
    }

    /// Check a generic function call, inferring type arguments from actual arguments
    pub(crate) fn check_generic_function_call(
        &mut self,
        sig: &FunctionSig,
        args: &[Spanned<Expr>],
    ) -> TypeResult<ResolvedType> {
        // Check argument count
        if sig.params.len() != args.len() {
            return Err(TypeError::ArgCount {
                expected: sig.params.len(),
                got: args.len(),
                span: None,
            });
        }

        // Create fresh type variables for each generic parameter
        let generic_substitutions: HashMap<String, ResolvedType> = sig
            .generics
            .iter()
            .map(|param| (param.clone(), self.fresh_type_var()))
            .collect();

        // Check each argument and unify with parameter type
        for ((_, param_type, _), arg) in sig.params.iter().zip(args) {
            let arg_type = self.check_expr(arg)?;
            // Substitute generic parameters with type variables in the parameter type
            let expected_type = self.substitute_generics(param_type, &generic_substitutions);
            self.unify(&expected_type, &arg_type)?;
        }

        // Apply substitutions to infer concrete generic types
        let inferred_type_args: Vec<_> = sig
            .generics
            .iter()
            .map(|param| {
                let ty = generic_substitutions.get(param)
                    .expect("Internal compiler error: generic parameter should exist in substitutions map");
                self.apply_substitutions(ty)
            })
            .collect();

        // Record the generic instantiation if all type arguments are concrete
        let all_concrete = inferred_type_args.iter().all(|t| !matches!(t, ResolvedType::Var(_)));
        if all_concrete {
            let inst = GenericInstantiation::function(&sig.name, inferred_type_args.clone());
            self.add_instantiation(inst);
        }

        // Substitute generics in the return type
        let return_type = self.substitute_generics(&sig.ret, &generic_substitutions);
        let resolved_return = self.apply_substitutions(&return_type);

        // For async functions, wrap the return type in Future
        if sig.is_async {
            Ok(ResolvedType::Future(Box::new(resolved_return)))
        } else {
            Ok(resolved_return)
        }
    }

    /// Infer generic type arguments from a parameter type and an argument type.
    /// This is used to match a generic parameter with an actual argument.
    #[allow(dead_code)]
    pub(crate) fn infer_type_arg(
        &mut self,
        param_type: &ResolvedType,
        arg_type: &ResolvedType,
        type_args: &mut HashMap<String, ResolvedType>,
    ) -> TypeResult<()> {
        match (param_type, arg_type) {
            (ResolvedType::Generic(name), concrete) => {
                if let Some(existing) = type_args.get(name) {
                    // Check that the inferred type is consistent
                    if existing != concrete {
                        return Err(TypeError::Mismatch {
                            expected: existing.to_string(),
                            found: concrete.to_string(),
                            span: None,
                        });
                    }
                } else {
                    type_args.insert(name.clone(), concrete.clone());
                }
                Ok(())
            }
            (ResolvedType::Array(inner_param), ResolvedType::Array(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Pointer(inner_param), ResolvedType::Pointer(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Ref(inner_param), ResolvedType::Ref(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::RefMut(inner_param), ResolvedType::RefMut(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Optional(inner_param), ResolvedType::Optional(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Result(inner_param), ResolvedType::Result(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Future(inner_param), ResolvedType::Future(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (
                ResolvedType::Named { name: name_p, generics: generics_p },
                ResolvedType::Named { name: name_a, generics: generics_a },
            ) if name_p == name_a && generics_p.len() == generics_a.len() => {
                for (gp, ga) in generics_p.iter().zip(generics_a.iter()) {
                    self.infer_type_arg(gp, ga, type_args)?;
                }
                Ok(())
            }
            (ResolvedType::Tuple(types_p), ResolvedType::Tuple(types_a))
                if types_p.len() == types_a.len() =>
            {
                for (tp, ta) in types_p.iter().zip(types_a.iter()) {
                    self.infer_type_arg(tp, ta, type_args)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn { params: params_p, ret: ret_p },
                ResolvedType::Fn { params: params_a, ret: ret_a },
            ) if params_p.len() == params_a.len() => {
                for (pp, pa) in params_p.iter().zip(params_a.iter()) {
                    self.infer_type_arg(pp, pa, type_args)?;
                }
                self.infer_type_arg(ret_p, ret_a, type_args)
            }
            // Non-generic types don't contribute to type argument inference
            _ => Ok(()),
        }
    }
}
