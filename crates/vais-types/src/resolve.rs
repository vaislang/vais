//! Type resolution: AST Type → ResolvedType.

use vais_ast::*;

use super::TypeChecker;
use crate::types::{self, ResolvedType, TypeError, TypeResult};

impl TypeChecker {
    /// Resolve AST type to internal type
    pub(crate) fn resolve_type(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, generics } => {
                let resolved_generics: Vec<_> = generics
                    .iter()
                    .map(|g| self.resolve_type(&g.node))
                    .collect();

                match name.as_str() {
                    "i8" => ResolvedType::I8,
                    "i16" => ResolvedType::I16,
                    "i32" => ResolvedType::I32,
                    "i64" => ResolvedType::I64,
                    "i128" => ResolvedType::I128,
                    "u8" => ResolvedType::U8,
                    "u16" => ResolvedType::U16,
                    "u32" => ResolvedType::U32,
                    "u64" => ResolvedType::U64,
                    "u128" => ResolvedType::U128,
                    "f32" => ResolvedType::F32,
                    "f64" => ResolvedType::F64,
                    "bool" => ResolvedType::Bool,
                    "str" => ResolvedType::Str,
                    // SIMD Vector types
                    "Vec2f32" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::F32),
                        lanes: 2,
                    },
                    "Vec4f32" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::F32),
                        lanes: 4,
                    },
                    "Vec8f32" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::F32),
                        lanes: 8,
                    },
                    "Vec2f64" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::F64),
                        lanes: 2,
                    },
                    "Vec4f64" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::F64),
                        lanes: 4,
                    },
                    "Vec4i32" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::I32),
                        lanes: 4,
                    },
                    "Vec8i32" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::I32),
                        lanes: 8,
                    },
                    "Vec2i64" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::I64),
                        lanes: 2,
                    },
                    "Vec4i64" => ResolvedType::Vector {
                        element: Box::new(ResolvedType::I64),
                        lanes: 4,
                    },
                    _ => {
                        // Check if it's a generic type parameter
                        if self.current_generics.contains(name) {
                            ResolvedType::Generic(name.clone())
                        } else if let Some(alias) = self.type_aliases.get(name) {
                            alias.clone()
                        } else {
                            ResolvedType::Named {
                                name: name.clone(),
                                generics: resolved_generics,
                            }
                        }
                    }
                }
            }
            Type::Array(inner) => ResolvedType::Array(Box::new(self.resolve_type(&inner.node))),
            Type::ConstArray { element, size } => {
                let resolved_element = self.resolve_type(&element.node);
                let resolved_size = self.resolve_const_expr(size);
                ResolvedType::ConstArray {
                    element: Box::new(resolved_element),
                    size: resolved_size,
                }
            }
            Type::Map(key, value) => ResolvedType::Map(
                Box::new(self.resolve_type(&key.node)),
                Box::new(self.resolve_type(&value.node)),
            ),
            Type::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.resolve_type(&t.node)).collect())
            }
            Type::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.resolve_type(&inner.node)))
            }
            Type::Result(inner) => ResolvedType::Result(
                Box::new(self.resolve_type(&inner.node)),
                Box::new(ResolvedType::I64), // Default error type for backward compat
            ),
            Type::Pointer(inner) => ResolvedType::Pointer(Box::new(self.resolve_type(&inner.node))),
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.resolve_type(&inner.node))),
            Type::RefMut(inner) => ResolvedType::RefMut(Box::new(self.resolve_type(&inner.node))),
            Type::Slice(inner) => ResolvedType::Slice(Box::new(self.resolve_type(&inner.node))),
            Type::SliceMut(inner) => ResolvedType::SliceMut(Box::new(self.resolve_type(&inner.node))),
            Type::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.resolve_type(&p.node)).collect(),
                ret: Box::new(self.resolve_type(&ret.node)),
                effects: None,
            },
            Type::Unit => ResolvedType::Unit,
            Type::Infer => self.fresh_type_var(),
            Type::FnPtr {
                params,
                ret,
                is_vararg,
            } => {
                let resolved_params: Vec<_> =
                    params.iter().map(|p| self.resolve_type(&p.node)).collect();
                let resolved_ret = Box::new(self.resolve_type(&ret.node));
                ResolvedType::FnPtr {
                    params: resolved_params,
                    ret: resolved_ret,
                    is_vararg: *is_vararg,
                    effects: None,
                }
            }
            Type::DynTrait {
                trait_name,
                generics,
            } => {
                let resolved_generics: Vec<_> = generics
                    .iter()
                    .map(|g| self.resolve_type(&g.node))
                    .collect();
                ResolvedType::DynTrait {
                    trait_name: trait_name.clone(),
                    generics: resolved_generics,
                }
            }
            Type::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => {
                let resolved_base = self.resolve_type(&base.node);
                // Resolve GAT generic arguments
                let resolved_generics: Vec<ResolvedType> = generics
                    .iter()
                    .map(|g| self.resolve_type(&g.node))
                    .collect();
                // Try to resolve the associated type immediately if possible
                self.resolve_associated_type(
                    &resolved_base,
                    trait_name.as_deref(),
                    assoc_name,
                    &resolved_generics,
                )
            }
            Type::Linear(inner) => ResolvedType::Linear(Box::new(self.resolve_type(&inner.node))),
            Type::Affine(inner) => ResolvedType::Affine(Box::new(self.resolve_type(&inner.node))),
            Type::Dependent {
                var_name,
                base,
                predicate,
            } => {
                let resolved_base = self.resolve_type(&base.node);
                // Convert predicate expression to string for storage
                // The predicate is validated separately during type checking
                let predicate_str = format!("{:?}", predicate.node);
                ResolvedType::Dependent {
                    var_name: var_name.clone(),
                    base: Box::new(resolved_base),
                    predicate: predicate_str,
                }
            }
            Type::RefLifetime { lifetime, inner } => ResolvedType::RefLifetime {
                lifetime: lifetime.clone(),
                inner: Box::new(self.resolve_type(&inner.node)),
            },
            Type::RefMutLifetime { lifetime, inner } => ResolvedType::RefMutLifetime {
                lifetime: lifetime.clone(),
                inner: Box::new(self.resolve_type(&inner.node)),
            },
            Type::Lazy(inner) => ResolvedType::Lazy(Box::new(self.resolve_type(&inner.node))),
        }
    }

    /// Resolve an associated type to its concrete type
    /// Supports GAT (Generic Associated Types) with generic arguments
    pub(crate) fn resolve_associated_type(
        &self,
        base_ty: &ResolvedType,
        trait_name: Option<&str>,
        assoc_name: &str,
        gat_args: &[ResolvedType],
    ) -> ResolvedType {
        // Get the type name from base_ty
        let type_name = match base_ty {
            ResolvedType::Named { name, .. } => name.clone(),
            ResolvedType::Generic(name) => name.clone(),
            _ => {
                // Can't resolve, return as-is with GAT arguments
                return ResolvedType::Associated {
                    base: Box::new(base_ty.clone()),
                    trait_name: trait_name.map(|s| s.to_string()),
                    assoc_name: assoc_name.to_string(),
                    generics: gat_args.to_vec(),
                };
            }
        };

        // Find the trait impl for this type
        for impl_ in &self.trait_impls {
            if impl_.type_name == type_name {
                // If trait_name is specified, check it matches
                if let Some(tn) = trait_name {
                    if impl_.trait_name != tn {
                        continue;
                    }
                }
                // Check if this impl has the associated type
                if let Some(resolved) = impl_.associated_types.get(assoc_name) {
                    // For GAT, substitute generic parameters in the resolved type
                    if !gat_args.is_empty() {
                        // Get the trait definition to find GAT parameter names
                        if let Some(trait_def) = self.traits.get(&impl_.trait_name) {
                            if let Some(assoc_def) = trait_def.associated_types.get(assoc_name) {
                                // Build substitution map: GAT param name -> concrete type
                                let mut substitutions = std::collections::HashMap::new();
                                for (i, param_name) in assoc_def.generics.iter().enumerate() {
                                    if let Some(arg) = gat_args.get(i) {
                                        substitutions.insert(param_name.clone(), arg.clone());
                                    }
                                }
                                // Substitute GAT parameters in the resolved type
                                return crate::types::substitute_type(resolved, &substitutions);
                            }
                        }
                    }
                    return resolved.clone();
                }
            }
        }

        // If not found, check trait definition for default
        if let Some(trait_name) = trait_name {
            if let Some(trait_def) = self.traits.get(trait_name) {
                if let Some(assoc_def) = trait_def.associated_types.get(assoc_name) {
                    if let Some(default) = &assoc_def.default {
                        return default.clone();
                    }
                }
            }
        }

        // Return unresolved associated type with GAT arguments
        ResolvedType::Associated {
            base: Box::new(base_ty.clone()),
            trait_name: trait_name.map(|s| s.to_string()),
            assoc_name: assoc_name.to_string(),
            generics: gat_args.to_vec(),
        }
    }

    // Type inference methods have been moved to the inference module

    /// Validate a dependent type predicate.
    /// The predicate must be a boolean expression when the bound variable has the base type.
    pub fn validate_dependent_type(
        &mut self,
        var_name: &str,
        base: &ResolvedType,
        predicate: &Spanned<Expr>,
    ) -> TypeResult<()> {
        // Create a new scope for the bound variable
        self.push_scope();
        self.define_var(var_name, base.clone(), false);

        // Type check the predicate expression
        let pred_type = self.check_expr(predicate)?;

        // Remove the temporary scope
        self.pop_scope();

        // The predicate must evaluate to bool
        let applied_type = self.apply_substitutions(&pred_type);
        if applied_type != ResolvedType::Bool {
            return Err(TypeError::DependentPredicateNotBool {
                found: applied_type.to_string(),
                span: Some(predicate.span),
            });
        }

        Ok(())
    }

    /// Resolve a const expression from AST to internal representation
    pub(crate) fn resolve_const_expr(&self, expr: &vais_ast::ConstExpr) -> types::ResolvedConst {
        match expr {
            vais_ast::ConstExpr::Literal(n) => types::ResolvedConst::Value(*n),
            vais_ast::ConstExpr::Param(name) => types::ResolvedConst::Param(name.clone()),
            vais_ast::ConstExpr::BinOp { op, left, right } => {
                let resolved_left = self.resolve_const_expr(left);
                let resolved_right = self.resolve_const_expr(right);
                let resolved_op = match op {
                    vais_ast::ConstBinOp::Add => types::ConstBinOp::Add,
                    vais_ast::ConstBinOp::Sub => types::ConstBinOp::Sub,
                    vais_ast::ConstBinOp::Mul => types::ConstBinOp::Mul,
                    vais_ast::ConstBinOp::Div => types::ConstBinOp::Div,
                };

                // Try to evaluate if both sides are concrete values
                if let (Some(l), Some(r)) =
                    (resolved_left.try_evaluate(), resolved_right.try_evaluate())
                {
                    let result = match resolved_op {
                        types::ConstBinOp::Add => l.checked_add(r),
                        types::ConstBinOp::Sub => l.checked_sub(r),
                        types::ConstBinOp::Mul => l.checked_mul(r),
                        types::ConstBinOp::Div => {
                            if r != 0 {
                                l.checked_div(r)
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(value) = result {
                        return types::ResolvedConst::Value(value);
                    }
                }

                types::ResolvedConst::BinOp {
                    op: resolved_op,
                    left: Box::new(resolved_left),
                    right: Box::new(resolved_right),
                }
            }
        }
    }
}
