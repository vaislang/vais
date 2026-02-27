//! Module-level type checking: check_module, registration, and generics.

use std::collections::{HashMap, HashSet};

use vais_ast::*;

use super::TypeChecker;
use crate::object_safety;
use crate::ownership;
use crate::traits::TraitImpl;
use crate::traits::{AssociatedTypeDef, TraitDef, TraitMethodSig};
use crate::types::{
    self, EffectAnnotation, EnumDef, FunctionSig, ResolvedType, StructDef, TypeError, TypeResult,
    UnionDef, VariantFieldTypes,
};

mod registration;
mod traits;
mod validation;

/// Saved state of generic type parameters, for restoring after processing.
pub(crate) struct SavedGenericState {
    /// Generic type parameter names (e.g., ["T", "U"])
    pub generics: Vec<String>,
    /// Trait bounds per generic (e.g., {"T": ["Clone", "Debug"]})
    pub bounds: HashMap<String, Vec<String>>,
    /// Const generic values (e.g., {"N": ResolvedType::I64})
    pub const_generics: HashMap<String, ResolvedType>,
    /// Higher-kinded type generics with their arity (e.g., {"F": 1} for F<_>)
    pub hkt_generics: HashMap<String, usize>,
}

/// Extract HKT parameter names and arities from AST generic parameters.
pub(crate) fn extract_hkt_params(generics: &[vais_ast::GenericParam]) -> HashMap<String, usize> {
    generics
        .iter()
        .filter_map(|g| {
            if let vais_ast::GenericParamKind::HigherKinded { arity, .. } = &g.kind {
                Some((g.name.node.clone(), *arity))
            } else {
                None
            }
        })
        .collect()
}

impl TypeChecker {
    /// Type checks a complete module.
    ///
    /// Performs two-pass type checking:
    /// 1. First pass: Collect all type definitions (functions, structs, enums, traits)
    /// 2. Second pass: Type check all function bodies and implementations
    ///
    /// # Arguments
    ///
    /// * `module` - The parsed AST module to type check
    ///
    /// # Returns
    ///
    /// Ok(()) if type checking succeeds, or a TypeError on failure.
    pub fn check_module(&mut self, module: &Module) -> TypeResult<()> {
        // First pass: collect all type definitions
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => self.register_struct(s)?,
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::TypeAlias(t) => self.register_type_alias(t)?,
                Item::TraitAlias(ta) => self.register_trait_alias(ta)?,
                Item::Use(_use_stmt) => {
                    // Use statements are handled at the compiler level (AST merging)
                    // by the time we reach type checking, all imports are already resolved
                }
                Item::Trait(t) => self.register_trait(t)?,
                Item::Impl(impl_block) => {
                    // Register impl methods to the target type
                    self.register_impl(impl_block)?;
                }
                Item::Macro(_) => {
                    // Macro definitions are handled at the expansion phase
                    // before type checking
                }
                Item::Error { .. } => {
                    // Error nodes from recovery mode are skipped during type checking.
                    // They represent parsing failures that have already been reported.
                }
                Item::ExternBlock(ext) => {
                    // Register extern functions
                    for func in &ext.functions {
                        self.register_extern_function(func)?;
                    }
                }
                Item::Const(const_def) => {
                    // Register constant with its type
                    let const_type = self.resolve_type(&const_def.ty.node);
                    self.constants
                        .insert(const_def.name.node.clone(), const_type);
                }
                Item::Global(_global_def) => {
                    // Global variable definitions
                    // Type checking happens during code generation
                }
            }
        }

        // Second pass: check function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    let result = self.check_function(f);
                    self.try_or_collect(result)?;
                }
                Item::Impl(impl_block) => {
                    // Check impl method bodies
                    // Get struct generics if the target is a struct
                    let struct_generics = match &impl_block.target_type.node {
                        Type::Named { name, .. } => {
                            // Look up the struct definition to get its generics
                            self.structs
                                .get(name)
                                .map(|s| {
                                    s.generics
                                        .iter()
                                        .map(|g| {
                                            GenericParam::new_type(
                                                Spanned::new(g.clone(), Span::default()),
                                                vec![],
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        }
                        _ => vec![],
                    };
                    // Also include impl-level generics
                    let mut all_generics = struct_generics;
                    all_generics.extend_from_slice(&impl_block.generics);

                    for method in &impl_block.methods {
                        let result = self.check_impl_method(
                            &impl_block.target_type.node,
                            &method.node,
                            &all_generics,
                        );
                        self.try_or_collect(result)?;
                    }
                }
                _ => {}
            }
        }

        // Third pass: ownership and borrow checking (skip imported items)
        if let Some(strict) = self.ownership_check_mode {
            let mut ownership_checker = ownership::OwnershipChecker::new_collecting();
            // Only check ownership for items from the current file, not imported modules
            let local_module =
                if self.imported_item_count > 0 && self.imported_item_count < module.items.len() {
                    Module {
                        items: module.items[self.imported_item_count..].to_vec(),
                        modules_map: None,
                    }
                } else {
                    module.clone()
                };
            // Run ownership check in collecting mode (never fails, collects all errors)
            let _ = ownership_checker.check_module(&local_module);
            let ownership_errors = ownership_checker.take_errors();

            if !ownership_errors.is_empty() {
                if strict {
                    // Strict mode: return first error
                    return Err(ownership_errors.into_iter().next().unwrap());
                } else {
                    // Warn mode: add to warnings
                    for err in &ownership_errors {
                        self.warnings.push(format!("[ownership] {}", err));
                    }
                }
            }
        }

        Ok(())
    }

    /// Set current generics with their bounds for type resolution
    pub(crate) fn set_generics(&mut self, generics: &[GenericParam]) -> SavedGenericState {
        let prev_generics = std::mem::replace(
            &mut self.current_generics,
            generics.iter().map(|g| &g.name.node).cloned().collect(),
        );
        let prev_bounds = std::mem::replace(
            &mut self.current_generic_bounds,
            generics
                .iter()
                .map(|g| {
                    let mut expanded_bounds = Vec::new();
                    // For HKT params, use the bounds from their kind
                    let raw_bounds = match &g.kind {
                        GenericParamKind::HigherKinded { bounds, .. } => bounds,
                        _ => &g.bounds,
                    };
                    for b in raw_bounds {
                        if let Some(alias_bounds) = self.trait_aliases.get(&b.node) {
                            expanded_bounds.extend(alias_bounds.iter().cloned());
                        } else {
                            expanded_bounds.push(b.node.clone());
                        }
                    }
                    (g.name.node.clone(), expanded_bounds)
                })
                .collect(),
        );
        // Track const generic parameters with their types
        // Collect first to avoid borrow conflict with self.resolve_type
        let new_const_generics: HashMap<String, ResolvedType> = generics
            .iter()
            .filter_map(|g| {
                if let GenericParamKind::Const { ty } = &g.kind {
                    Some((g.name.node.clone(), self.resolve_type(&ty.node)))
                } else {
                    None
                }
            })
            .collect();
        let prev_const_generics =
            std::mem::replace(&mut self.current_const_generics, new_const_generics);

        // Track HKT generic parameters with their arity
        let new_hkt_generics: HashMap<String, usize> = generics
            .iter()
            .filter_map(|g| {
                if let GenericParamKind::HigherKinded { arity, .. } = &g.kind {
                    Some((g.name.node.clone(), *arity))
                } else {
                    None
                }
            })
            .collect();
        let prev_hkt_generics = std::mem::replace(&mut self.current_hkt_generics, new_hkt_generics);

        SavedGenericState {
            generics: prev_generics,
            bounds: prev_bounds,
            const_generics: prev_const_generics,
            hkt_generics: prev_hkt_generics,
        }
    }

    /// Restore previous generics
    pub(crate) fn restore_generics(&mut self, saved: SavedGenericState) {
        self.current_generics = saved.generics;
        self.current_generic_bounds = saved.bounds;
        self.current_const_generics = saved.const_generics;
        self.current_hkt_generics = saved.hkt_generics;
    }

    /// Merge where clause bounds into current generic bounds.
    ///
    /// Where clause predicates provide additional trait bounds on generic parameters
    /// that supplement the inline bounds in the generic parameter list.
    ///
    /// # Arguments
    ///
    /// * `where_clause` - The where clause predicates to merge
    ///
    /// # Example
    ///
    /// ```vais
    /// F foo<T>(x: T) where T: Display + Clone { ... }
    /// ```
    ///
    /// The where clause bounds (Display, Clone) are merged into the generic bounds for T.
    pub(crate) fn merge_where_clause(&mut self, where_clause: &[WherePredicate]) {
        for predicate in where_clause {
            let bounds = self
                .current_generic_bounds
                .entry(predicate.ty.node.clone())
                .or_default();
            for b in &predicate.bounds {
                // Expand trait aliases in where clause bounds
                if let Some(alias_bounds) = self.trait_aliases.get(&b.node) {
                    for ab in alias_bounds {
                        if !bounds.contains(ab) {
                            bounds.push(ab.clone());
                        }
                    }
                } else if !bounds.contains(&b.node) {
                    bounds.push(b.node.clone());
                }
            }
        }
    }

    /// Extract contract specification from function attributes.
    ///
    /// Parses requires/ensures/invariant attributes and builds a ContractSpec.
    /// NOTE: Contract expression type-checking is done in check_function() (checker_fn.rs)
    /// for requires (before body) and ensures (after body with 'return' in scope).
    /// This method only extracts the contract clauses without re-checking types.
    pub(crate) fn extract_contracts(
        &mut self,
        f: &Function,
    ) -> TypeResult<Option<types::ContractSpec>> {
        use types::{ContractClause, ContractSpec};

        let mut spec = ContractSpec::default();

        for attr in &f.attributes {
            match attr.name.as_str() {
                "requires" | "ensures" => {
                    if let Some(expr) = &attr.expr {
                        // Contract expressions are already type-checked in check_function()
                        // (requires before body, ensures after body with 'return' in scope).
                        // Here we only extract the clause for storage in FunctionSig.
                        let clause = ContractClause {
                            expr_str: attr.args.first().cloned().unwrap_or_default(),
                            span: expr.span,
                        };

                        if attr.name == "requires" {
                            spec.requires.push(clause);
                        } else {
                            spec.ensures.push(clause);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(if spec.is_empty() { None } else { Some(spec) })
    }
}
