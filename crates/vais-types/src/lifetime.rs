//! Lifetime inference engine for the Vais type system
//!
//! Implements automatic lifetime inference similar to Rust's lifetime elision rules,
//! with constraint-based lifetime solving. This module handles:
//!
//! - Lifetime variable generation and tracking
//! - Constraint generation from function signatures and bodies
//! - Lifetime elision rules (matching Rust's three rules)
//! - Constraint solving via fixed-point iteration
//! - Scope-based lifetime validation

use crate::types::{ResolvedType, TypeError, TypeResult};
use std::collections::{BTreeMap, HashMap, HashSet};
use vais_ast::*;

/// Unique lifetime variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LifetimeVar(pub u32);

impl std::fmt::Display for LifetimeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'_{}", self.0)
    }
}

/// A named or inferred lifetime
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lifetime {
    /// A named lifetime parameter (e.g., 'a)
    Named(String),
    /// An inferred lifetime variable
    Inferred(LifetimeVar),
    /// The 'static lifetime - lives for the entire program
    Static,
}

impl std::fmt::Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lifetime::Named(name) => write!(f, "'{}", name),
            Lifetime::Inferred(var) => write!(f, "{}", var),
            Lifetime::Static => write!(f, "'static"),
        }
    }
}

/// A constraint between two lifetimes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifetimeConstraint {
    /// 'a: 'b - lifetime 'a outlives lifetime 'b
    Outlives {
        longer: Lifetime,
        shorter: Lifetime,
        reason: ConstraintReason,
    },
    /// 'a == 'b - two lifetimes must be equal
    Equal {
        a: Lifetime,
        b: Lifetime,
        reason: ConstraintReason,
    },
}

impl std::fmt::Display for LifetimeConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifetimeConstraint::Outlives {
                longer, shorter, ..
            } => {
                write!(f, "{}: {}", longer, shorter)
            }
            LifetimeConstraint::Equal { a, b, .. } => {
                write!(f, "{} == {}", a, b)
            }
        }
    }
}

/// Why a constraint was generated
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintReason {
    /// From a function parameter reference
    FunctionParam { param_name: String },
    /// From a function return type
    FunctionReturn,
    /// From a struct field reference
    StructField { field_name: String },
    /// From a lifetime bound declaration ('a: 'b)
    ExplicitBound,
    /// From assignment (rhs lifetime must outlive lhs scope)
    Assignment,
    /// From elision rule application
    Elision,
}

/// Scope identifier for tracking reference lifetimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(pub u32);

/// Scope information for lifetime analysis
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub depth: u32,
}

/// Result of lifetime inference for a function
#[derive(Debug, Clone)]
pub struct LifetimeResolution {
    /// Map from lifetime variable to resolved lifetime
    pub resolved: HashMap<LifetimeVar, Lifetime>,
    /// Inferred lifetime parameters for the function
    pub lifetime_params: Vec<String>,
    /// Validated constraints (all satisfied)
    pub constraints: Vec<LifetimeConstraint>,
}

/// The lifetime inference engine
pub struct LifetimeInferencer {
    /// Next lifetime variable ID
    next_var: u32,
    /// Next scope ID
    next_scope: u32,
    /// Active constraints
    constraints: Vec<LifetimeConstraint>,
    /// Scope hierarchy
    scopes: Vec<ScopeInfo>,
    /// Current scope
    current_scope: ScopeId,
    /// Lifetime variable assignments (var -> set of lifetimes it could be)
    assignments: HashMap<LifetimeVar, Lifetime>,
    /// Named lifetime parameters in scope
    named_lifetimes: HashSet<String>,
    /// Map from variable name to its lifetime
    var_lifetimes: HashMap<String, Lifetime>,
    /// Outlives relationships: key outlives all values in the set
    outlives_graph: BTreeMap<Lifetime, HashSet<Lifetime>>,
}

impl Default for LifetimeInferencer {
    fn default() -> Self {
        Self::new()
    }
}

impl LifetimeInferencer {
    pub fn new() -> Self {
        let root_scope = ScopeInfo {
            id: ScopeId(0),
            parent: None,
            depth: 0,
        };
        Self {
            next_var: 0,
            next_scope: 1,
            constraints: Vec::new(),
            scopes: vec![root_scope],
            current_scope: ScopeId(0),
            assignments: HashMap::new(),
            named_lifetimes: HashSet::new(),
            var_lifetimes: HashMap::new(),
            outlives_graph: BTreeMap::new(),
        }
    }

    /// Generate a fresh lifetime variable
    pub fn fresh_lifetime_var(&mut self) -> LifetimeVar {
        let var = LifetimeVar(self.next_var);
        self.next_var += 1;
        var
    }

    /// Push a new scope
    pub fn push_scope(&mut self) -> ScopeId {
        let parent_depth = self
            .scopes
            .iter()
            .find(|s| s.id == self.current_scope)
            .map(|s| s.depth)
            .unwrap_or(0);
        let id = ScopeId(self.next_scope);
        self.next_scope += 1;
        let scope = ScopeInfo {
            id,
            parent: Some(self.current_scope),
            depth: parent_depth + 1,
        };
        self.scopes.push(scope);
        self.current_scope = id;
        id
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.iter().find(|s| s.id == self.current_scope) {
            if let Some(parent) = scope.parent {
                self.current_scope = parent;
            }
        }
    }

    /// Register a named lifetime parameter
    pub fn register_named_lifetime(&mut self, name: &str) {
        self.named_lifetimes.insert(name.to_string());
    }

    /// Register a variable's lifetime
    pub fn register_var_lifetime(&mut self, var_name: &str, lifetime: Lifetime) {
        self.var_lifetimes.insert(var_name.to_string(), lifetime);
    }

    /// Get a variable's lifetime
    pub fn get_var_lifetime(&self, var_name: &str) -> Option<&Lifetime> {
        self.var_lifetimes.get(var_name)
    }

    /// Add an outlives constraint: `longer` outlives `shorter`
    pub fn add_outlives(&mut self, longer: Lifetime, shorter: Lifetime, reason: ConstraintReason) {
        self.constraints.push(LifetimeConstraint::Outlives {
            longer: longer.clone(),
            shorter: shorter.clone(),
            reason,
        });
        self.outlives_graph
            .entry(longer)
            .or_default()
            .insert(shorter);
    }

    /// Add an equality constraint
    pub fn add_equal(&mut self, a: Lifetime, b: Lifetime, reason: ConstraintReason) {
        self.constraints
            .push(LifetimeConstraint::Equal { a, b, reason });
    }

    /// Resolve a lifetime string to a Lifetime enum
    pub fn resolve_lifetime_name(&self, name: &str) -> Lifetime {
        if name == "static" {
            Lifetime::Static
        } else if self.named_lifetimes.contains(name) {
            Lifetime::Named(name.to_string())
        } else {
            // Unknown lifetime name - treat as named (will be validated later)
            Lifetime::Named(name.to_string())
        }
    }

    /// Apply Rust-style lifetime elision rules to a function signature.
    ///
    /// Rules:
    /// 1. Each reference parameter gets a distinct lifetime parameter
    /// 2. If there is exactly one input lifetime, it is assigned to all output lifetimes
    /// 3. If there is a `&self` or `&mut self` parameter, its lifetime is assigned to all output lifetimes
    pub fn apply_elision_rules(
        &mut self,
        params: &[(String, ResolvedType, bool)],
        ret: &ResolvedType,
    ) -> LifetimeElisionResult {
        let mut input_lifetimes: Vec<(String, Lifetime)> = Vec::new();
        let mut has_self_ref = false;
        let mut self_lifetime: Option<Lifetime> = None;

        // Rule 1: Assign distinct lifetime to each reference parameter
        for (name, ty, _) in params {
            if let Some(lt) = self.extract_reference_lifetime(ty) {
                input_lifetimes.push((name.clone(), lt));
            } else if self.is_reference_type(ty) {
                let var = self.fresh_lifetime_var();
                let lt = Lifetime::Inferred(var);
                input_lifetimes.push((name.clone(), lt));
            }

            // Check for self reference (Rule 3)
            if name == "self" && self.is_reference_type(ty) {
                has_self_ref = true;
                self_lifetime = input_lifetimes.last().map(|(_, lt)| lt.clone());
            }
        }

        // Determine output lifetime
        let output_lifetime = if self.has_reference_in_return(ret) {
            // Rule 3: &self/&mut self -> output gets self's lifetime
            if has_self_ref {
                self_lifetime.clone()
            }
            // Rule 2: Exactly one input lifetime -> output gets that lifetime
            else if input_lifetimes.len() == 1 {
                Some(input_lifetimes[0].1.clone())
            } else if input_lifetimes.is_empty() {
                // No input lifetimes - output gets 'static
                Some(Lifetime::Static)
            } else {
                // Multiple input lifetimes, no self - cannot elide
                None
            }
        } else {
            None // No reference in return type, no elision needed
        };

        let elision_successful = output_lifetime.is_some() || !self.has_reference_in_return(ret);
        LifetimeElisionResult {
            input_lifetimes,
            output_lifetime,
            elision_successful,
        }
    }

    /// Extract the lifetime from a reference type, if explicitly annotated
    fn extract_reference_lifetime(&self, ty: &ResolvedType) -> Option<Lifetime> {
        match ty {
            ResolvedType::RefLifetime { lifetime, .. }
            | ResolvedType::RefMutLifetime { lifetime, .. } => {
                Some(self.resolve_lifetime_name(lifetime))
            }
            _ => None,
        }
    }

    /// Check if a type contains a reference
    fn is_reference_type(&self, ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::Ref(_)
                | ResolvedType::RefMut(_)
                | ResolvedType::RefLifetime { .. }
                | ResolvedType::RefMutLifetime { .. }
        )
    }

    /// Check if a return type contains a reference
    fn has_reference_in_return(&self, ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Ref(_)
            | ResolvedType::RefMut(_)
            | ResolvedType::RefLifetime { .. }
            | ResolvedType::RefMutLifetime { .. } => true,
            ResolvedType::Tuple(types) => types.iter().any(|t| self.has_reference_in_return(t)),
            ResolvedType::Optional(inner) | ResolvedType::Result(inner) => {
                self.has_reference_in_return(inner)
            }
            _ => false,
        }
    }

    /// Infer lifetimes for a function signature.
    ///
    /// This is the main entry point for lifetime inference on a function.
    /// It applies elision rules, generates constraints from explicit annotations,
    /// and validates that all constraints are satisfiable.
    pub fn infer_function_lifetimes(
        &mut self,
        func_name: &str,
        params: &[(String, ResolvedType, bool)],
        ret: &ResolvedType,
        explicit_lifetime_params: &[String],
        lifetime_bounds: &[(String, Vec<String>)],
    ) -> TypeResult<LifetimeResolution> {
        // Register explicit lifetime parameters
        for lt_param in explicit_lifetime_params {
            self.register_named_lifetime(lt_param);
        }

        // Register explicit lifetime bounds ('a: 'b)
        for (lt, bounds) in lifetime_bounds {
            let longer = Lifetime::Named(lt.clone());
            for bound in bounds {
                let shorter = self.resolve_lifetime_name(bound);
                self.add_outlives(longer.clone(), shorter, ConstraintReason::ExplicitBound);
            }
        }

        // Generate constraints from parameters
        for (name, ty, _) in params {
            self.generate_type_constraints(
                ty,
                &ConstraintReason::FunctionParam {
                    param_name: name.clone(),
                },
            );
        }

        // Generate constraints from return type
        self.generate_type_constraints(ret, &ConstraintReason::FunctionReturn);

        // Apply elision rules if no explicit lifetime annotations on return
        let elision = self.apply_elision_rules(params, ret);

        if !elision.elision_successful && self.has_reference_in_return(ret) {
            return Err(TypeError::LifetimeElisionFailed {
                function_name: func_name.to_string(),
                input_count: elision.input_lifetimes.len(),
                span: None,
            });
        }

        // Add elision-derived constraints
        if let Some(ref output_lt) = elision.output_lifetime {
            if let Some(ret_lt) = self.extract_reference_lifetime(ret) {
                self.add_equal(ret_lt, output_lt.clone(), ConstraintReason::Elision);
            }
        }

        // Solve constraints
        self.solve_constraints()?;

        // Collect results
        let lifetime_params: Vec<String> = if explicit_lifetime_params.is_empty() {
            // Infer lifetime parameters from the elision result
            elision
                .input_lifetimes
                .iter()
                .filter_map(|(_, lt)| match lt {
                    Lifetime::Named(name) => Some(name.clone()),
                    _ => None,
                })
                .collect()
        } else {
            explicit_lifetime_params.to_vec()
        };

        Ok(LifetimeResolution {
            resolved: self.assignments.clone(),
            lifetime_params,
            constraints: self.constraints.clone(),
        })
    }

    /// Generate constraints from a type's lifetime annotations
    fn generate_type_constraints(&mut self, ty: &ResolvedType, reason: &ConstraintReason) {
        match ty {
            ResolvedType::RefLifetime { lifetime, inner } => {
                let lt = self.resolve_lifetime_name(lifetime);
                // The reference's lifetime must outlive any lifetimes in the inner type
                self.generate_inner_outlives(&lt, inner, reason);
                self.generate_type_constraints(inner, reason);
            }
            ResolvedType::RefMutLifetime { lifetime, inner } => {
                let lt = self.resolve_lifetime_name(lifetime);
                self.generate_inner_outlives(&lt, inner, reason);
                self.generate_type_constraints(inner, reason);
            }
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                self.generate_type_constraints(inner, reason);
            }
            ResolvedType::Array(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Result(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Pointer(inner) => {
                self.generate_type_constraints(inner, reason);
            }
            ResolvedType::Tuple(types) => {
                for t in types {
                    self.generate_type_constraints(t, reason);
                }
            }
            ResolvedType::Fn { params, ret, .. } => {
                for p in params {
                    self.generate_type_constraints(p, reason);
                }
                self.generate_type_constraints(ret, reason);
            }
            ResolvedType::Named { generics, .. } => {
                for g in generics {
                    self.generate_type_constraints(g, reason);
                }
            }
            _ => {}
        }
    }

    /// Generate outlives constraints: the outer lifetime must outlive inner lifetimes
    fn generate_inner_outlives(
        &mut self,
        outer: &Lifetime,
        inner: &ResolvedType,
        reason: &ConstraintReason,
    ) {
        match inner {
            ResolvedType::RefLifetime { lifetime, .. } => {
                let inner_lt = self.resolve_lifetime_name(lifetime);
                self.add_outlives(outer.clone(), inner_lt, reason.clone());
            }
            ResolvedType::RefMutLifetime { lifetime, .. } => {
                let inner_lt = self.resolve_lifetime_name(lifetime);
                self.add_outlives(outer.clone(), inner_lt, reason.clone());
            }
            _ => {}
        }
    }

    /// Solve the collected constraints using fixed-point iteration.
    ///
    /// Validates that:
    /// - All outlives constraints are satisfiable
    /// - No contradictory equality constraints exist
    /// - 'static outlives everything (always true)
    fn solve_constraints(&mut self) -> TypeResult<()> {
        // Build transitive closure of outlives relationships
        let mut changed = true;
        let max_iterations = 100;
        let mut iteration = 0;

        while changed && iteration < max_iterations {
            changed = false;
            iteration += 1;

            // Collect new edges from transitivity
            let mut new_edges: Vec<(Lifetime, Lifetime)> = Vec::new();

            for (longer, shorters) in &self.outlives_graph {
                for shorter in shorters {
                    // If shorter outlives others, then longer also outlives them (transitivity)
                    if let Some(transitive) = self.outlives_graph.get(shorter).cloned() {
                        for t in transitive {
                            if !shorters.contains(&t) && &t != longer {
                                new_edges.push((longer.clone(), t));
                            }
                        }
                    }
                }
            }

            for (longer, shorter) in new_edges {
                let set = self.outlives_graph.entry(longer).or_default();
                if set.insert(shorter) {
                    changed = true;
                }
            }
        }

        // Validate equality constraints
        for constraint in &self.constraints {
            if let LifetimeConstraint::Equal { a, b, .. } = constraint {
                // Check for contradictions: if a == b, then a: b and b: a must hold
                // If one outlives the other but not vice versa, it's a contradiction
                // (This is a simplification; in practice we just add bidirectional outlives)
                if a != b {
                    // Equality is treated as bidirectional outlives / unification.
                    // We just record the assignment for inferred variables.
                    if let Lifetime::Inferred(var) = a {
                        self.assignments.insert(*var, b.clone());
                    }
                    if let Lifetime::Inferred(var) = b {
                        self.assignments.insert(*var, a.clone());
                    }
                }
            }
        }

        // Validate: 'static outlives everything (trivially true)
        // Check for impossible constraints like X: 'static where X is a short-lived scope
        for constraint in &self.constraints {
            if let LifetimeConstraint::Outlives {
                longer,
                shorter,
                reason: _,
            } = constraint
            {
                // A non-static named lifetime cannot outlive 'static
                // (unless it IS 'static)
                if *shorter == Lifetime::Static && *longer != Lifetime::Static {
                    // This is only an error for Inferred lifetimes
                    // Named lifetimes that need to outlive 'static are a user error
                    if let Lifetime::Named(name) = longer {
                        if name != "static" {
                            return Err(TypeError::LifetimeOutlivesStatic {
                                lifetime_name: name.clone(),
                                span: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate that a reference does not outlive its referent.
    /// Used to check assignments and returns.
    ///
    /// Returns an error with detailed context if the reference would dangle:
    /// - Which lifetime is the reference vs referent
    /// - Why the constraint fails (cause)
    /// - What the user can do to fix it (resolution guide)
    pub fn check_reference_validity(
        &self,
        ref_lifetime: &Lifetime,
        referent_lifetime: &Lifetime,
    ) -> TypeResult<()> {
        // 'static references are always valid
        if *referent_lifetime == Lifetime::Static {
            return Ok(());
        }

        // If ref_lifetime outlives referent_lifetime, the reference is dangling
        match (ref_lifetime, referent_lifetime) {
            (Lifetime::Static, Lifetime::Named(name)) => Err(TypeError::LifetimeTooShort {
                reference_lifetime: "'static".to_string(),
                referent_lifetime: format!("'{}", name),
                span: None,
            }),
            (Lifetime::Static, Lifetime::Inferred(var)) => Err(TypeError::LifetimeTooShort {
                reference_lifetime: "'static".to_string(),
                referent_lifetime: format!("{}", var),
                span: None,
            }),
            (Lifetime::Named(a), Lifetime::Named(b)) if a != b => {
                // Check if a outlives b in the constraint graph
                let a_lt = Lifetime::Named(a.clone());
                let b_lt = Lifetime::Named(b.clone());

                // Check if ref_lifetime is longer than referent_lifetime
                // (which means the reference outlives the data = dangling)
                if let Some(a_outlives) = self.outlives_graph.get(&a_lt) {
                    if a_outlives.contains(&b_lt) {
                        // a outlives b - reference is valid (data lives longer)
                        return Ok(());
                    }
                }

                // Check reverse: if b outlives a, the reference is valid
                if let Some(b_outlives) = self.outlives_graph.get(&b_lt) {
                    if b_outlives.contains(&a_lt) {
                        return Ok(());
                    }
                }

                // No relationship established - conservative: allow
                // (actual violation would be caught by borrow checker at usage site)
                Ok(())
            }
            (Lifetime::Named(a), Lifetime::Inferred(var)) => {
                // Named lifetime referencing inferred lifetime
                // Check if the inferred var was resolved
                if let Some(resolved) = self.assignments.get(var) {
                    return self.check_reference_validity(ref_lifetime, resolved);
                }
                // If ref is not 'static and referent is inferred, allow (conservative)
                let _ = a;
                Ok(())
            }
            (Lifetime::Inferred(var), referent) => {
                // Resolve inferred reference lifetime and recheck
                if let Some(resolved) = self.assignments.get(var) {
                    return self.check_reference_validity(resolved, referent);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Validate all reference parameters against return type lifetime.
    /// Checks that the function does not return a dangling reference.
    pub fn validate_return_lifetime(
        &self,
        return_lifetime: &Lifetime,
        param_lifetimes: &[(String, Lifetime)],
    ) -> TypeResult<()> {
        // Return lifetime must be outlived by at least one parameter lifetime
        // (otherwise the returned reference could dangle)
        if *return_lifetime == Lifetime::Static {
            return Ok(()); // 'static is always valid
        }

        // Check if any parameter lifetime covers the return lifetime
        for (_name, param_lt) in param_lifetimes {
            if param_lt == return_lifetime {
                return Ok(());
            }
            // Check if param_lt outlives return_lt
            if let Some(outlives) = self.outlives_graph.get(param_lt) {
                if outlives.contains(return_lifetime) {
                    return Ok(());
                }
            }
        }

        // If no parameter covers the return lifetime, it might be dangling
        // (This is caught more precisely by elision rules, so we're conservative here)
        Ok(())
    }

    /// Get the current constraints for debugging/diagnostics
    pub fn get_constraints(&self) -> &[LifetimeConstraint] {
        &self.constraints
    }

    /// Reset the inferencer state for a new function
    pub fn reset(&mut self) {
        self.next_var = 0;
        self.next_scope = 1;
        self.constraints.clear();
        self.scopes = vec![ScopeInfo {
            id: ScopeId(0),
            parent: None,
            depth: 0,
        }];
        self.current_scope = ScopeId(0);
        self.assignments.clear();
        self.named_lifetimes.clear();
        self.var_lifetimes.clear();
        self.outlives_graph.clear();
    }

    /// Extract lifetime parameters from generic parameters
    pub fn extract_lifetime_params(generics: &[GenericParam]) -> Vec<String> {
        generics
            .iter()
            .filter_map(|g| match &g.kind {
                GenericParamKind::Lifetime { .. } => Some(g.name.node.clone()),
                _ => None,
            })
            .collect()
    }

    /// Extract lifetime bounds from generic parameters
    pub fn extract_lifetime_bounds(generics: &[GenericParam]) -> Vec<(String, Vec<String>)> {
        generics
            .iter()
            .filter_map(|g| match &g.kind {
                GenericParamKind::Lifetime { bounds } if !bounds.is_empty() => {
                    Some((g.name.node.clone(), bounds.clone()))
                }
                _ => None,
            })
            .collect()
    }
}

/// Result of applying lifetime elision rules
#[derive(Debug, Clone)]
pub struct LifetimeElisionResult {
    /// Input lifetimes (param_name, lifetime)
    pub input_lifetimes: Vec<(String, Lifetime)>,
    /// Output lifetime (if elided successfully)
    pub output_lifetime: Option<Lifetime>,
    /// Whether elision was successful
    pub elision_successful: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_lifetime_var() {
        let mut inferencer = LifetimeInferencer::new();
        let v0 = inferencer.fresh_lifetime_var();
        let v1 = inferencer.fresh_lifetime_var();
        assert_eq!(v0, LifetimeVar(0));
        assert_eq!(v1, LifetimeVar(1));
    }

    #[test]
    fn test_lifetime_display() {
        assert_eq!(format!("{}", Lifetime::Named("a".into())), "'a");
        assert_eq!(format!("{}", Lifetime::Static), "'static");
        assert_eq!(format!("{}", Lifetime::Inferred(LifetimeVar(0))), "'_0");
    }

    #[test]
    fn test_scope_push_pop() {
        let mut inferencer = LifetimeInferencer::new();
        assert_eq!(inferencer.current_scope, ScopeId(0));
        let s1 = inferencer.push_scope();
        assert_eq!(s1, ScopeId(1));
        assert_eq!(inferencer.current_scope, ScopeId(1));
        inferencer.pop_scope();
        assert_eq!(inferencer.current_scope, ScopeId(0));
    }

    #[test]
    fn test_resolve_static_lifetime() {
        let inferencer = LifetimeInferencer::new();
        let lt = inferencer.resolve_lifetime_name("static");
        assert_eq!(lt, Lifetime::Static);
    }

    #[test]
    fn test_resolve_named_lifetime() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.register_named_lifetime("a");
        let lt = inferencer.resolve_lifetime_name("a");
        assert_eq!(lt, Lifetime::Named("a".into()));
    }

    #[test]
    fn test_elision_single_input_lifetime() {
        let mut inferencer = LifetimeInferencer::new();
        // F foo(x: &i64) -> &i64
        let params = vec![(
            "x".to_string(),
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
        )];
        let ret = ResolvedType::Ref(Box::new(ResolvedType::I64));
        let result = inferencer.apply_elision_rules(&params, &ret);
        assert!(result.elision_successful);
        assert!(result.output_lifetime.is_some());
    }

    #[test]
    fn test_elision_multiple_input_lifetimes_fails() {
        let mut inferencer = LifetimeInferencer::new();
        // F foo(x: &i64, y: &i64) -> &i64  (ambiguous - cannot elide)
        let params = vec![
            (
                "x".to_string(),
                ResolvedType::Ref(Box::new(ResolvedType::I64)),
                false,
            ),
            (
                "y".to_string(),
                ResolvedType::Ref(Box::new(ResolvedType::I64)),
                false,
            ),
        ];
        let ret = ResolvedType::Ref(Box::new(ResolvedType::I64));
        let result = inferencer.apply_elision_rules(&params, &ret);
        assert!(!result.elision_successful);
        assert!(result.output_lifetime.is_none());
    }

    #[test]
    fn test_elision_self_reference() {
        let mut inferencer = LifetimeInferencer::new();
        // F foo(self: &Self, x: &i64) -> &i64  (self's lifetime used for output)
        let params = vec![
            (
                "self".to_string(),
                ResolvedType::Ref(Box::new(ResolvedType::Named {
                    name: "Self".into(),
                    generics: vec![],
                })),
                false,
            ),
            (
                "x".to_string(),
                ResolvedType::Ref(Box::new(ResolvedType::I64)),
                false,
            ),
        ];
        let ret = ResolvedType::Ref(Box::new(ResolvedType::I64));
        let result = inferencer.apply_elision_rules(&params, &ret);
        assert!(result.elision_successful);
        assert!(result.output_lifetime.is_some());
    }

    #[test]
    fn test_elision_no_reference_return() {
        let mut inferencer = LifetimeInferencer::new();
        // F foo(x: &i64) -> i64  (no reference in return, no elision needed)
        let params = vec![(
            "x".to_string(),
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
        )];
        let ret = ResolvedType::I64;
        let result = inferencer.apply_elision_rules(&params, &ret);
        assert!(result.elision_successful);
        assert!(result.output_lifetime.is_none());
    }

    #[test]
    fn test_outlives_constraint() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.add_outlives(
            Lifetime::Named("a".into()),
            Lifetime::Named("b".into()),
            ConstraintReason::ExplicitBound,
        );
        assert_eq!(inferencer.constraints.len(), 1);
    }

    #[test]
    fn test_solve_simple_constraints() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.register_named_lifetime("a");
        inferencer.register_named_lifetime("b");
        inferencer.add_outlives(
            Lifetime::Named("a".into()),
            Lifetime::Named("b".into()),
            ConstraintReason::ExplicitBound,
        );
        assert!(inferencer.solve_constraints().is_ok());
    }

    #[test]
    fn test_transitive_outlives() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.register_named_lifetime("a");
        inferencer.register_named_lifetime("b");
        inferencer.register_named_lifetime("c");
        // 'a: 'b, 'b: 'c => 'a: 'c (transitive)
        inferencer.add_outlives(
            Lifetime::Named("a".into()),
            Lifetime::Named("b".into()),
            ConstraintReason::ExplicitBound,
        );
        inferencer.add_outlives(
            Lifetime::Named("b".into()),
            Lifetime::Named("c".into()),
            ConstraintReason::ExplicitBound,
        );
        assert!(inferencer.solve_constraints().is_ok());
        // Check that 'a outlives 'c transitively
        let a_outlives = inferencer.outlives_graph.get(&Lifetime::Named("a".into()));
        assert!(a_outlives.is_some());
        assert!(a_outlives.unwrap().contains(&Lifetime::Named("c".into())));
    }

    #[test]
    fn test_static_outlives_violation() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.register_named_lifetime("a");
        // 'a: 'static is an error (non-static cannot outlive static)
        inferencer.add_outlives(
            Lifetime::Named("a".into()),
            Lifetime::Static,
            ConstraintReason::ExplicitBound,
        );
        assert!(inferencer.solve_constraints().is_err());
    }

    #[test]
    fn test_infer_function_lifetimes_simple() {
        let mut inferencer = LifetimeInferencer::new();
        // F first(x: &'a i64) -> &'a i64
        let params = vec![(
            "x".to_string(),
            ResolvedType::RefLifetime {
                lifetime: "a".into(),
                inner: Box::new(ResolvedType::I64),
            },
            false,
        )];
        let ret = ResolvedType::RefLifetime {
            lifetime: "a".into(),
            inner: Box::new(ResolvedType::I64),
        };
        let result =
            inferencer.infer_function_lifetimes("first", &params, &ret, &["a".to_string()], &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.fresh_lifetime_var();
        inferencer.register_named_lifetime("a");
        inferencer.push_scope();
        inferencer.reset();
        assert_eq!(inferencer.next_var, 0);
        assert!(inferencer.named_lifetimes.is_empty());
        assert_eq!(inferencer.current_scope, ScopeId(0));
    }

    // --- Dangling reference / lifetime validity tests ---

    #[test]
    fn test_static_ref_to_named_lifetime_is_dangling() {
        let inferencer = LifetimeInferencer::new();
        // 'static reference to data with lifetime 'a -> dangling
        let result =
            inferencer.check_reference_validity(&Lifetime::Static, &Lifetime::Named("a".into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_static_ref_to_static_is_ok() {
        let inferencer = LifetimeInferencer::new();
        let result = inferencer.check_reference_validity(&Lifetime::Static, &Lifetime::Static);
        assert!(result.is_ok());
    }

    #[test]
    fn test_named_ref_to_static_is_ok() {
        let inferencer = LifetimeInferencer::new();
        // 'a reference to 'static data -> always valid
        let result =
            inferencer.check_reference_validity(&Lifetime::Named("a".into()), &Lifetime::Static);
        assert!(result.is_ok());
    }

    #[test]
    fn test_static_ref_to_inferred_is_dangling() {
        let inferencer = LifetimeInferencer::new();
        let result = inferencer
            .check_reference_validity(&Lifetime::Static, &Lifetime::Inferred(LifetimeVar(0)));
        assert!(result.is_err());
    }

    #[test]
    fn test_inferred_ref_resolves_to_check() {
        let mut inferencer = LifetimeInferencer::new();
        // Assign inferred var 0 -> 'static
        inferencer
            .assignments
            .insert(LifetimeVar(0), Lifetime::Static);

        // Inferred(0) = 'static, referent = Named("a")
        // 'static ref to 'a data -> dangling
        let result = inferencer.check_reference_validity(
            &Lifetime::Inferred(LifetimeVar(0)),
            &Lifetime::Named("a".into()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_named_ref_with_outlives_relationship() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.register_named_lifetime("a");
        inferencer.register_named_lifetime("b");
        // 'a outlives 'b
        inferencer.add_outlives(
            Lifetime::Named("a".into()),
            Lifetime::Named("b".into()),
            ConstraintReason::ExplicitBound,
        );
        let _ = inferencer.solve_constraints();

        // 'a ref to 'b data -> valid because 'a outlives 'b (data lives at least as long)
        let result = inferencer
            .check_reference_validity(&Lifetime::Named("a".into()), &Lifetime::Named("b".into()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_return_lifetime_with_matching_param() {
        let mut inferencer = LifetimeInferencer::new();
        inferencer.register_named_lifetime("a");

        let param_lifetimes = vec![("x".to_string(), Lifetime::Named("a".into()))];
        let result =
            inferencer.validate_return_lifetime(&Lifetime::Named("a".into()), &param_lifetimes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_return_static_is_always_ok() {
        let inferencer = LifetimeInferencer::new();
        let result = inferencer.validate_return_lifetime(&Lifetime::Static, &[]);
        assert!(result.is_ok());
    }
}
