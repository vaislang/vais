//! Type checker implementation
//!
//! Main type checking logic for AOEL units.

use aoel_ast::{Expr, Unit};

use crate::error::{TypeCheckError, TypeCheckResult};
use crate::infer::TypeInferrer;
use crate::symbol::{find_similar_names, ScopeLevel, Symbol, SymbolKind, SymbolTable};
use crate::types::{is_bool_type, type_to_string};

/// Type checker for AOEL units
pub struct TypeChecker<'a> {
    /// The unit being checked
    unit: &'a Unit,
    /// Symbol table
    symbols: SymbolTable,
    /// Collected errors
    errors: Vec<TypeCheckError>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(unit: &'a Unit) -> Self {
        Self {
            unit,
            symbols: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    /// Run all type checking phases
    pub fn check(mut self) -> TypeCheckResult<()> {
        // Phase 1: Build symbol table and check for duplicates
        self.build_symbol_table();

        // Phase 1: Resolve references in expressions
        self.check_references();

        // Phase 1: Type check expressions
        self.check_expression_types();

        // Phase 2: Validate FLOW structure
        self.check_flow();

        // Phase 3: Check INTENT consistency
        self.check_intent_consistency();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    // =========================================================================
    // Phase 1: Symbol Table
    // =========================================================================

    fn build_symbol_table(&mut self) {
        // Register INPUT fields
        for field in &self.unit.input.fields {
            let symbol = Symbol::input_field(
                field.name.name.clone(),
                field.ty.clone(),
                field.span,
            );

            if let Some(existing) = self.symbols.define(ScopeLevel::Input, symbol) {
                self.errors.push(TypeCheckError::DuplicateField {
                    name: field.name.name.clone(),
                    first_span: existing.span,
                    duplicate_span: field.span,
                });
            }
        }

        // Register OUTPUT fields
        for field in &self.unit.output.fields {
            let symbol = Symbol::output_field(
                field.name.name.clone(),
                field.ty.clone(),
                field.span,
            );

            if let Some(existing) = self.symbols.define(ScopeLevel::Output, symbol) {
                self.errors.push(TypeCheckError::DuplicateField {
                    name: field.name.name.clone(),
                    first_span: existing.span,
                    duplicate_span: field.span,
                });
            }
        }

        // Register FLOW nodes
        for node in &self.unit.flow.nodes {
            let symbol = Symbol::flow_node(
                node.id.name.clone(),
                format!("{:?}", node.op_type),
                node.span,
            );

            if let Some(existing) = self.symbols.define(ScopeLevel::Flow, symbol) {
                self.errors.push(TypeCheckError::DuplicateNodeId {
                    id: node.id.name.clone(),
                    first_span: existing.span,
                    duplicate_span: node.span,
                });
            }
        }
    }

    // =========================================================================
    // Phase 1: Reference Checking
    // =========================================================================

    fn check_references(&mut self) {
        // Check CONSTRAINT expressions
        for constraint in &self.unit.constraint.constraints {
            self.check_expr_references(&constraint.expr);
        }

        // Check VERIFY expressions
        for entry in &self.unit.verify.entries {
            if let Some(ref expr) = entry.expr {
                self.check_expr_references(expr);
            }
        }

        // Check INPUT field constraints
        for field in &self.unit.input.fields {
            for constraint in &field.constraints {
                self.check_expr_references(&constraint.expr);
            }
        }

        // Check OUTPUT field constraints
        for field in &self.unit.output.fields {
            for constraint in &field.constraints {
                self.check_expr_references(&constraint.expr);
            }
        }

        // Check FLOW node parameters
        for node in &self.unit.flow.nodes {
            for param in &node.params {
                self.check_expr_references(&param.value);
            }
        }

        // Check FLOW edge conditions
        for edge in &self.unit.flow.edges {
            if let Some(ref cond) = edge.condition {
                self.check_expr_references(cond);
            }
        }
    }

    fn check_expr_references(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => {
                // Skip special keywords (reserved words and operators)
                let special_keywords = [
                    "input", "output", "INPUT", "OUTPUT",
                    // Arithmetic operators for NODE parameters
                    "ADD", "SUB", "MUL", "DIV", "MOD",
                    // Comparison operators
                    "EQ", "NEQ", "LT", "GT", "LTE", "GTE",
                    // Logical operators
                    "AND", "OR", "NOT", "XOR",
                    // Other reserved names
                    "VOID", "TRUE", "FALSE",
                ];
                if special_keywords.contains(&ident.name.as_str()) {
                    return;
                }

                // Check if it's a known symbol (built-in, input, output, or node)
                if !self.symbols.exists(&ident.name) {
                    let all_names = self.symbols.all_names();
                    self.errors.push(TypeCheckError::UndefinedReference {
                        name: ident.name.clone(),
                        span: ident.span,
                        suggestions: find_similar_names(&ident.name, &all_names, 2),
                    });
                }
            }
            Expr::FieldAccess(access) => {
                self.check_field_access(access);
            }
            Expr::Binary(bin) => {
                self.check_expr_references(&bin.left);
                self.check_expr_references(&bin.right);
            }
            Expr::Unary(un) => {
                self.check_expr_references(&un.operand);
            }
            Expr::Call(call) => {
                // Check function name
                if !self.symbols.is_builtin(&call.name.name) {
                    let all_names = self.symbols.all_names();
                    self.errors.push(TypeCheckError::UndefinedReference {
                        name: call.name.name.clone(),
                        span: call.name.span,
                        suggestions: find_similar_names(&call.name.name, &all_names, 2),
                    });
                }
                // Check arguments
                for arg in &call.args {
                    self.check_expr_references(arg);
                }
            }
            Expr::Index(idx) => {
                self.check_expr_references(&idx.base);
                self.check_expr_references(&idx.index);
            }
            Expr::Grouped(grp) => {
                self.check_expr_references(&grp.inner);
            }
            Expr::Literal(_) | Expr::ExternalRef(_) => {}
        }
    }

    fn check_field_access(&mut self, access: &aoel_ast::FieldAccess) {
        // Handle input.field and output.field patterns
        if let Expr::Ident(base) = &access.base {
            match base.name.as_str() {
                "input" | "INPUT" => {
                    if self
                        .symbols
                        .lookup_in_scope(ScopeLevel::Input, &access.field.name)
                        .is_none()
                    {
                        let available = self.symbols.names_in_scope(ScopeLevel::Input);
                        self.errors.push(TypeCheckError::InvalidFieldAccess {
                            base: "input".to_string(),
                            field: access.field.name.clone(),
                            span: access.span,
                            available_fields: available,
                        });
                    }
                }
                "output" | "OUTPUT" => {
                    if self
                        .symbols
                        .lookup_in_scope(ScopeLevel::Output, &access.field.name)
                        .is_none()
                    {
                        let available = self.symbols.names_in_scope(ScopeLevel::Output);
                        self.errors.push(TypeCheckError::InvalidFieldAccess {
                            base: "output".to_string(),
                            field: access.field.name.clone(),
                            span: access.span,
                            available_fields: available,
                        });
                    }
                }
                _ => {
                    // Check if it's a node reference (node.port)
                    if self
                        .symbols
                        .lookup_in_scope(ScopeLevel::Flow, &base.name)
                        .is_some()
                    {
                        // It's a node.port reference - allow any port name
                        // (we could validate port names based on OpType in the future)
                        return;
                    }

                    // Otherwise, recursively check the base
                    self.check_expr_references(&access.base);
                }
            }
        } else {
            // Nested field access - check recursively
            self.check_expr_references(&access.base);
        }
    }

    // =========================================================================
    // Phase 1: Expression Type Checking
    // =========================================================================

    fn check_expression_types(&mut self) {
        let inferrer = TypeInferrer::new(&self.symbols);

        // Check CONSTRAINT expressions are BOOL
        for constraint in &self.unit.constraint.constraints {
            match inferrer.infer(&constraint.expr) {
                Ok(ty) => {
                    if !is_bool_type(&ty) {
                        self.errors.push(TypeCheckError::NonBoolConstraint {
                            found: type_to_string(&ty),
                            span: constraint.span,
                        });
                    }
                }
                Err(e) => self.errors.push(e),
            }
        }

        // Check VERIFY expressions are BOOL
        for entry in &self.unit.verify.entries {
            if let Some(ref expr) = entry.expr {
                match inferrer.infer(expr) {
                    Ok(ty) => {
                        if !is_bool_type(&ty) {
                            self.errors.push(TypeCheckError::NonBoolVerify {
                                found: type_to_string(&ty),
                                span: entry.span,
                            });
                        }
                    }
                    Err(e) => self.errors.push(e),
                }
            }
        }

        // Check INPUT field constraints are BOOL
        for field in &self.unit.input.fields {
            for constraint in &field.constraints {
                match inferrer.infer(&constraint.expr) {
                    Ok(ty) => {
                        if !is_bool_type(&ty) {
                            self.errors.push(TypeCheckError::NonBoolConstraint {
                                found: type_to_string(&ty),
                                span: constraint.span,
                            });
                        }
                    }
                    Err(e) => self.errors.push(e),
                }
            }
        }
    }

    // =========================================================================
    // Phase 2: FLOW Validation
    // =========================================================================

    fn check_flow(&mut self) {
        let inferrer = TypeInferrer::new(&self.symbols);

        // Check edge sources and targets
        for edge in &self.unit.flow.edges {
            // Validate source
            if !self.is_valid_edge_endpoint(&edge.source) {
                if let Some(endpoint_name) = self.extract_endpoint_name(&edge.source) {
                    self.errors.push(TypeCheckError::InvalidEdgeSource {
                        name: endpoint_name,
                        span: edge.span,
                    });
                }
            }

            // Validate target
            if !self.is_valid_edge_endpoint(&edge.target) {
                if let Some(endpoint_name) = self.extract_endpoint_name(&edge.target) {
                    self.errors.push(TypeCheckError::InvalidEdgeTarget {
                        name: endpoint_name,
                        span: edge.span,
                    });
                }
            }

            // Check edge conditions are BOOL
            if let Some(ref cond) = edge.condition {
                match inferrer.infer(cond) {
                    Ok(ty) => {
                        if !is_bool_type(&ty) {
                            self.errors.push(TypeCheckError::NonBoolEdgeCondition {
                                found: type_to_string(&ty),
                                span: cond.span(),
                            });
                        }
                    }
                    Err(e) => self.errors.push(e),
                }
            }
        }
    }

    fn is_valid_edge_endpoint(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(ident) => {
                // Valid if it's a flow node
                self.symbols
                    .lookup_in_scope(ScopeLevel::Flow, &ident.name)
                    .is_some()
            }
            Expr::FieldAccess(access) => {
                if let Expr::Ident(base) = &access.base {
                    match base.name.as_str() {
                        // INPUT.field or OUTPUT.field
                        "INPUT" | "input" => self
                            .symbols
                            .lookup_in_scope(ScopeLevel::Input, &access.field.name)
                            .is_some(),
                        "OUTPUT" | "output" => self
                            .symbols
                            .lookup_in_scope(ScopeLevel::Output, &access.field.name)
                            .is_some(),
                        // node.port
                        _ => self
                            .symbols
                            .lookup_in_scope(ScopeLevel::Flow, &base.name)
                            .is_some(),
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn extract_endpoint_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(ident) => Some(ident.name.clone()),
            Expr::FieldAccess(access) => {
                let base_name = self.extract_endpoint_name(&access.base)?;
                Some(format!("{}.{}", base_name, access.field.name))
            }
            _ => None,
        }
    }

    // =========================================================================
    // Phase 3: INTENT Consistency
    // =========================================================================

    fn check_intent_consistency(&mut self) {
        // Check GOAL inputs reference existing INPUT fields
        for input_expr in &self.unit.intent.goal_spec.inputs {
            self.check_goal_reference(input_expr, true);
        }

        // Check GOAL outputs reference existing OUTPUT fields
        for output_expr in &self.unit.intent.goal_spec.outputs {
            self.check_goal_reference(output_expr, false);
        }
    }

    fn check_goal_reference(&mut self, expr: &Expr, is_input: bool) {
        match expr {
            Expr::Ident(ident) => {
                let scope = if is_input {
                    ScopeLevel::Input
                } else {
                    ScopeLevel::Output
                };
                let kind = if is_input { "input" } else { "output" };

                // Skip if it's just "input" or "output"
                if ident.name == "input" || ident.name == "output" {
                    return;
                }

                if self.symbols.lookup_in_scope(scope, &ident.name).is_none() {
                    self.errors.push(TypeCheckError::UndefinedGoalField {
                        field: ident.name.clone(),
                        kind,
                        span: ident.span,
                    });
                }
            }
            Expr::FieldAccess(access) => {
                // Handle input.field or output.field in GOAL
                if let Expr::Ident(base) = &access.base {
                    if (base.name == "input" || base.name == "INPUT") && is_input {
                        if self
                            .symbols
                            .lookup_in_scope(ScopeLevel::Input, &access.field.name)
                            .is_none()
                        {
                            self.errors.push(TypeCheckError::UndefinedGoalField {
                                field: access.field.name.clone(),
                                kind: "input",
                                span: access.span,
                            });
                        }
                    } else if (base.name == "output" || base.name == "OUTPUT") && !is_input {
                        if self
                            .symbols
                            .lookup_in_scope(ScopeLevel::Output, &access.field.name)
                            .is_none()
                        {
                            self.errors.push(TypeCheckError::UndefinedGoalField {
                                field: access.field.name.clone(),
                                kind: "output",
                                span: access.span,
                            });
                        }
                    }
                }
            }
            Expr::Grouped(grp) => {
                self.check_goal_reference(&grp.inner, is_input);
            }
            _ => {}
        }
    }
}
