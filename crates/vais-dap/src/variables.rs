//! Variable Management
//!
//! Handles variable reference allocation, scopes, and variable expansion.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::protocol::types::{Scope, ScopePresentationHint, Variable, VariablePresentationHint};

/// Evaluation context for expression evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluateContext {
    Watch,
    Hover,
    Repl,
    Clipboard,
}

/// Manages variable references for scopes and expandable variables
#[derive(Debug, Default)]
pub struct VariableManager {
    /// Next variable reference ID
    next_ref: AtomicI64,
    /// Mapping from variable reference to its info
    ref_map: HashMap<i64, VariableRefInfo>,
    /// Cached variables per reference
    cached_variables: HashMap<i64, Vec<CachedVariable>>,
}

/// Information about what a variable reference points to
#[derive(Debug, Clone)]
pub enum VariableRefInfo {
    /// A scope (locals, arguments, registers)
    Scope {
        frame_id: i64,
        scope_type: ScopeKind,
    },
    /// A structured variable that can be expanded
    Variable {
        frame_id: i64,
        parent_ref: i64,
        name: String,
        path: String, // Full expression path for evaluation
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Locals,
    Arguments,
    Registers,
    Globals,
}

impl ScopeKind {
    pub fn name(&self) -> &'static str {
        match self {
            ScopeKind::Locals => "Locals",
            ScopeKind::Arguments => "Arguments",
            ScopeKind::Registers => "Registers",
            ScopeKind::Globals => "Globals",
        }
    }

    pub fn presentation_hint(&self) -> ScopePresentationHint {
        match self {
            ScopeKind::Locals => ScopePresentationHint::Locals,
            ScopeKind::Arguments => ScopePresentationHint::Arguments,
            ScopeKind::Registers => ScopePresentationHint::Registers,
            ScopeKind::Globals => ScopePresentationHint::Locals,
        }
    }

    pub fn is_expensive(&self) -> bool {
        matches!(self, ScopeKind::Registers | ScopeKind::Globals)
    }
}

/// Cached variable data
#[derive(Debug, Clone)]
pub struct CachedVariable {
    pub name: String,
    pub value: String,
    pub type_name: Option<String>,
    pub variables_reference: i64,
    pub has_children: bool,
    pub memory_reference: Option<String>,
}

impl VariableManager {
    pub fn new() -> Self {
        Self {
            next_ref: AtomicI64::new(1),
            ref_map: HashMap::new(),
            cached_variables: HashMap::new(),
        }
    }

    fn next_ref(&self) -> i64 {
        self.next_ref.fetch_add(1, Ordering::SeqCst)
    }

    /// Clear all cached data (call when execution resumes)
    pub fn invalidate(&mut self) {
        self.ref_map.clear();
        self.cached_variables.clear();
    }

    /// Create scopes for a frame
    pub fn create_scopes(&mut self, frame_id: i64) -> Vec<Scope> {
        let mut scopes = Vec::new();

        for scope_kind in [
            ScopeKind::Locals,
            ScopeKind::Arguments,
            ScopeKind::Registers,
        ] {
            let ref_id = self.next_ref();

            self.ref_map.insert(
                ref_id,
                VariableRefInfo::Scope {
                    frame_id,
                    scope_type: scope_kind,
                },
            );

            scopes.push(Scope {
                name: scope_kind.name().to_string(),
                presentation_hint: Some(scope_kind.presentation_hint()),
                variables_reference: ref_id,
                named_variables: None,
                indexed_variables: None,
                expensive: Some(scope_kind.is_expensive()),
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            });
        }

        scopes
    }

    /// Create scopes for a frame with Globals instead of Registers
    pub fn create_scopes_with_globals(&mut self, frame_id: i64) -> Vec<Scope> {
        let mut scopes = Vec::new();

        for scope_kind in [
            ScopeKind::Locals,
            ScopeKind::Arguments,
            ScopeKind::Globals,
        ] {
            let ref_id = self.next_ref();

            self.ref_map.insert(
                ref_id,
                VariableRefInfo::Scope {
                    frame_id,
                    scope_type: scope_kind,
                },
            );

            scopes.push(Scope {
                name: scope_kind.name().to_string(),
                presentation_hint: Some(scope_kind.presentation_hint()),
                variables_reference: ref_id,
                named_variables: None,
                indexed_variables: None,
                expensive: Some(scope_kind.is_expensive()),
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            });
        }

        scopes
    }

    /// Get info about a variable reference
    pub fn get_ref_info(&self, ref_id: i64) -> Option<&VariableRefInfo> {
        self.ref_map.get(&ref_id)
    }

    /// Cache variables for a reference
    pub fn cache_variables(
        &mut self,
        ref_id: i64,
        frame_id: i64,
        raw_vars: Vec<RawVariable>,
    ) -> Vec<Variable> {
        let mut cached = Vec::with_capacity(raw_vars.len());
        let mut dap_vars = Vec::with_capacity(raw_vars.len());

        for raw in raw_vars {
            // Allocate reference if variable has children
            let child_ref = if raw.has_children {
                let new_ref = self.next_ref();

                // Get the path for this variable
                let path = if let Some(VariableRefInfo::Variable {
                    path: parent_path, ..
                }) = self.ref_map.get(&ref_id)
                {
                    format!("{}.{}", parent_path, raw.name)
                } else {
                    raw.name.clone()
                };

                self.ref_map.insert(
                    new_ref,
                    VariableRefInfo::Variable {
                        frame_id,
                        parent_ref: ref_id,
                        name: raw.name.clone(),
                        path,
                    },
                );

                new_ref
            } else {
                0
            };

            cached.push(CachedVariable {
                name: raw.name.clone(),
                value: raw.value.clone(),
                type_name: raw.type_name.clone(),
                variables_reference: child_ref,
                has_children: raw.has_children,
                memory_reference: raw.memory_reference.clone(),
            });

            let presentation_hint = raw.type_name.as_ref().map(|t| VariablePresentationHint {
                kind: Some(type_to_kind(t)),
                attributes: None,
                visibility: None,
                lazy: None,
            });

            dap_vars.push(Variable {
                name: raw.name.clone(),
                value: raw.value,
                var_type: raw.type_name,
                presentation_hint,
                evaluate_name: Some(raw.name),
                variables_reference: child_ref,
                named_variables: if raw.has_children {
                    raw.named_children
                } else {
                    None
                },
                indexed_variables: if raw.has_children {
                    raw.indexed_children
                } else {
                    None
                },
                memory_reference: raw.memory_reference,
            });
        }

        self.cached_variables.insert(ref_id, cached);
        dap_vars
    }

    /// Get cached variables for a reference
    pub fn get_cached_variables(&self, ref_id: i64) -> Option<&Vec<CachedVariable>> {
        self.cached_variables.get(&ref_id)
    }

    /// Get the evaluation path for a variable reference
    pub fn get_evaluation_path(&self, ref_id: i64) -> Option<String> {
        match self.ref_map.get(&ref_id)? {
            VariableRefInfo::Variable { path, .. } => Some(path.clone()),
            VariableRefInfo::Scope { .. } => None,
        }
    }

    /// Evaluate an expression in the given context
    /// Returns a Variable representation of the result
    pub fn evaluate_expression(
        &mut self,
        expression: &str,
        frame_id: i64,
        context: EvaluateContext,
    ) -> Variable {
        // Try to resolve the expression from cached variables
        if let Some(cached_var) = self.resolve_path(expression, frame_id) {
            let formatted_value = self.format_for_context(cached_var, context);

            Variable {
                name: expression.to_string(),
                value: formatted_value,
                var_type: cached_var.type_name.clone(),
                presentation_hint: cached_var.type_name.as_ref().map(|t| {
                    VariablePresentationHint {
                        kind: Some(type_to_kind(t)),
                        attributes: None,
                        visibility: None,
                        lazy: None,
                    }
                }),
                evaluate_name: Some(expression.to_string()),
                variables_reference: cached_var.variables_reference,
                named_variables: None,
                indexed_variables: None,
                memory_reference: cached_var.memory_reference.clone(),
            }
        } else {
            // Expression not found in cache, return unevaluated
            Variable {
                name: expression.to_string(),
                value: format!("<not available: {}>", expression),
                var_type: None,
                presentation_hint: None,
                evaluate_name: Some(expression.to_string()),
                variables_reference: 0,
                named_variables: None,
                indexed_variables: None,
                memory_reference: None,
            }
        }
    }

    /// Find a variable by name in all cached scopes for a frame
    fn find_variable_by_name(&self, name: &str, frame_id: i64) -> Option<&CachedVariable> {
        // Iterate through all scope references for this frame
        for (ref_id, ref_info) in &self.ref_map {
            if let VariableRefInfo::Scope {
                frame_id: fid,
                scope_type: _,
            } = ref_info
            {
                if *fid == frame_id {
                    // Check cached variables for this scope
                    if let Some(cached_vars) = self.cached_variables.get(ref_id) {
                        if let Some(var) = cached_vars.iter().find(|v| v.name == name) {
                            return Some(var);
                        }
                    }
                }
            }
        }
        None
    }

    /// Resolve a dotted path expression (e.g., "obj.field.sub")
    fn resolve_path(&self, path: &str, frame_id: i64) -> Option<&CachedVariable> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        // Find the base variable
        let mut current_var = self.find_variable_by_name(parts[0], frame_id)?;

        // Navigate through the path
        for part in &parts[1..] {
            // Need to find child variables
            if current_var.variables_reference == 0 {
                return None; // No children
            }

            // Look for the child in cached variables
            let cached_children = self.cached_variables.get(&current_var.variables_reference)?;
            current_var = cached_children.iter().find(|v| v.name == *part)?;
        }

        Some(current_var)
    }

    /// Format a variable for the given context
    fn format_for_context(&self, var: &CachedVariable, context: EvaluateContext) -> String {
        match context {
            EvaluateContext::Hover => {
                // Include type information
                if let Some(type_name) = &var.type_name {
                    format!("{} ({})", var.value, type_name)
                } else {
                    var.value.clone()
                }
            }
            EvaluateContext::Clipboard => {
                // Value only, no type
                var.value.clone()
            }
            _ => var.value.clone(),
        }
    }
}

/// Raw variable from debugger
#[derive(Debug, Clone)]
pub struct RawVariable {
    pub name: String,
    pub value: String,
    pub type_name: Option<String>,
    pub has_children: bool,
    pub named_children: Option<i64>,
    pub indexed_children: Option<i64>,
    pub memory_reference: Option<String>,
}

fn type_to_kind(type_name: &str) -> String {
    let lower = type_name.to_lowercase();

    if lower.contains("int")
        || lower.contains("i32")
        || lower.contains("i64")
        || lower.contains("u32")
        || lower.contains("u64")
        || lower.contains("isize")
        || lower.contains("usize")
        || lower.contains("float")
        || lower.contains("f32")
        || lower.contains("f64")
        || lower.contains("bool")
        || lower.contains("*")
        || lower.contains("ptr")
        || lower.contains("ref")
    {
        "property".to_string()
    } else if lower.contains("str")
        || lower.contains("string")
        || lower.contains("vec")
        || lower.contains("array")
        || lower.contains("[")
    {
        "data".to_string()
    } else if lower.contains("struct") || lower.contains("enum") {
        "class".to_string()
    } else {
        "property".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_scopes() {
        let mut manager = VariableManager::new();

        let scopes = manager.create_scopes(1);

        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].name, "Locals");
        assert_eq!(scopes[1].name, "Arguments");
        assert_eq!(scopes[2].name, "Registers");

        // Check that references are tracked
        assert!(manager
            .get_ref_info(scopes[0].variables_reference)
            .is_some());
    }

    #[test]
    fn test_cache_variables() {
        let mut manager = VariableManager::new();

        let scopes = manager.create_scopes(1);
        let locals_ref = scopes[0].variables_reference;

        let raw_vars = vec![
            RawVariable {
                name: "x".to_string(),
                value: "42".to_string(),
                type_name: Some("i64".to_string()),
                has_children: false,
                named_children: None,
                indexed_children: None,
                memory_reference: None,
            },
            RawVariable {
                name: "arr".to_string(),
                value: "[1, 2, 3]".to_string(),
                type_name: Some("Vec<i64>".to_string()),
                has_children: true,
                named_children: None,
                indexed_children: Some(3),
                memory_reference: Some("0x1234".to_string()),
            },
        ];

        let dap_vars = manager.cache_variables(locals_ref, 1, raw_vars);

        assert_eq!(dap_vars.len(), 2);
        assert_eq!(dap_vars[0].name, "x");
        assert_eq!(dap_vars[0].variables_reference, 0); // No children
        assert!(dap_vars[1].variables_reference > 0); // Has children

        // Check child reference is tracked
        let child_ref = dap_vars[1].variables_reference;
        let info = manager.get_ref_info(child_ref).unwrap();
        if let VariableRefInfo::Variable { name, .. } = info {
            assert_eq!(name, "arr");
        } else {
            panic!("Expected Variable ref info");
        }
    }

    #[test]
    fn test_evaluate_existing_variable() {
        let mut manager = VariableManager::new();

        let scopes = manager.create_scopes(1);
        let locals_ref = scopes[0].variables_reference;

        let raw_vars = vec![RawVariable {
            name: "x".to_string(),
            value: "42".to_string(),
            type_name: Some("i64".to_string()),
            has_children: false,
            named_children: None,
            indexed_children: None,
            memory_reference: None,
        }];

        manager.cache_variables(locals_ref, 1, raw_vars);

        let result = manager.evaluate_expression("x", 1, EvaluateContext::Repl);

        assert_eq!(result.name, "x");
        assert_eq!(result.value, "42");
        assert_eq!(result.var_type, Some("i64".to_string()));
        assert_eq!(result.variables_reference, 0);
    }

    #[test]
    fn test_evaluate_missing_variable() {
        let mut manager = VariableManager::new();

        let scopes = manager.create_scopes(1);
        let locals_ref = scopes[0].variables_reference;

        manager.cache_variables(locals_ref, 1, vec![]);

        let result = manager.evaluate_expression("missing", 1, EvaluateContext::Repl);

        assert_eq!(result.name, "missing");
        assert!(result.value.contains("not available"));
        assert_eq!(result.var_type, None);
        assert_eq!(result.variables_reference, 0);
    }

    #[test]
    fn test_evaluate_dotted_path() {
        let mut manager = VariableManager::new();

        let scopes = manager.create_scopes(1);
        let locals_ref = scopes[0].variables_reference;

        // Create parent variable
        let raw_vars = vec![RawVariable {
            name: "obj".to_string(),
            value: "MyStruct".to_string(),
            type_name: Some("MyStruct".to_string()),
            has_children: true,
            named_children: Some(1),
            indexed_children: None,
            memory_reference: None,
        }];

        let dap_vars = manager.cache_variables(locals_ref, 1, raw_vars);
        let obj_ref = dap_vars[0].variables_reference;

        // Cache child variable
        let child_vars = vec![RawVariable {
            name: "field".to_string(),
            value: "99".to_string(),
            type_name: Some("i32".to_string()),
            has_children: false,
            named_children: None,
            indexed_children: None,
            memory_reference: None,
        }];

        manager.cache_variables(obj_ref, 1, child_vars);

        let result = manager.evaluate_expression("obj.field", 1, EvaluateContext::Repl);

        assert_eq!(result.name, "obj.field");
        assert_eq!(result.value, "99");
        assert_eq!(result.var_type, Some("i32".to_string()));
    }

    #[test]
    fn test_evaluate_context_formatting() {
        let mut manager = VariableManager::new();

        let scopes = manager.create_scopes(1);
        let locals_ref = scopes[0].variables_reference;

        let raw_vars = vec![RawVariable {
            name: "x".to_string(),
            value: "42".to_string(),
            type_name: Some("i64".to_string()),
            has_children: false,
            named_children: None,
            indexed_children: None,
            memory_reference: None,
        }];

        manager.cache_variables(locals_ref, 1, raw_vars);

        // Test Hover context (includes type)
        let hover_result = manager.evaluate_expression("x", 1, EvaluateContext::Hover);
        assert_eq!(hover_result.value, "42 (i64)");

        // Test Clipboard context (value only)
        let clipboard_result = manager.evaluate_expression("x", 1, EvaluateContext::Clipboard);
        assert_eq!(clipboard_result.value, "42");

        // Test Repl context (default)
        let repl_result = manager.evaluate_expression("x", 1, EvaluateContext::Repl);
        assert_eq!(repl_result.value, "42");
    }

    #[test]
    fn test_create_scopes_with_globals() {
        let mut manager = VariableManager::new();

        let scopes = manager.create_scopes_with_globals(1);

        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].name, "Locals");
        assert_eq!(scopes[1].name, "Arguments");
        assert_eq!(scopes[2].name, "Globals");

        // Verify that Globals scope is marked as expensive
        assert_eq!(scopes[2].expensive, Some(true));

        // Check that references are tracked
        for scope in &scopes {
            assert!(manager.get_ref_info(scope.variables_reference).is_some());
        }

        // Verify the scope type is correct
        let globals_info = manager.get_ref_info(scopes[2].variables_reference).unwrap();
        if let VariableRefInfo::Scope {
            scope_type,
            frame_id,
        } = globals_info
        {
            assert_eq!(*scope_type, ScopeKind::Globals);
            assert_eq!(*frame_id, 1);
        } else {
            panic!("Expected Scope ref info for Globals");
        }
    }
}
