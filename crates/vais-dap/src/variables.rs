//! Variable Management
//!
//! Handles variable reference allocation, scopes, and variable expansion.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::protocol::types::{Scope, Variable, VariablePresentationHint, ScopePresentationHint};

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

        for scope_kind in [ScopeKind::Locals, ScopeKind::Arguments, ScopeKind::Registers] {
            let ref_id = self.next_ref();

            self.ref_map.insert(ref_id, VariableRefInfo::Scope {
                frame_id,
                scope_type: scope_kind,
            });

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
    pub fn cache_variables(&mut self, ref_id: i64, frame_id: i64, raw_vars: Vec<RawVariable>) -> Vec<Variable> {
        let mut cached = Vec::with_capacity(raw_vars.len());
        let mut dap_vars = Vec::with_capacity(raw_vars.len());

        for raw in raw_vars {
            // Allocate reference if variable has children
            let child_ref = if raw.has_children {
                let new_ref = self.next_ref();

                // Get the path for this variable
                let path = if let Some(VariableRefInfo::Variable { path: parent_path, .. }) =
                    self.ref_map.get(&ref_id)
                {
                    format!("{}.{}", parent_path, raw.name)
                } else {
                    raw.name.clone()
                };

                self.ref_map.insert(new_ref, VariableRefInfo::Variable {
                    frame_id,
                    parent_ref: ref_id,
                    name: raw.name.clone(),
                    path,
                });

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

            let presentation_hint = raw.type_name.as_ref().map(|t| {
                VariablePresentationHint {
                    kind: Some(type_to_kind(t)),
                    attributes: None,
                    visibility: None,
                    lazy: None,
                }
            });

            dap_vars.push(Variable {
                name: raw.name.clone(),
                value: raw.value,
                var_type: raw.type_name,
                presentation_hint,
                evaluate_name: Some(raw.name),
                variables_reference: child_ref,
                named_variables: if raw.has_children { raw.named_children } else { None },
                indexed_variables: if raw.has_children { raw.indexed_children } else { None },
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

    if lower.contains("int") || lower.contains("i32") || lower.contains("i64") ||
       lower.contains("u32") || lower.contains("u64") || lower.contains("isize") ||
       lower.contains("usize") || lower.contains("float") || lower.contains("f32") ||
       lower.contains("f64") || lower.contains("bool") || lower.contains("*") ||
       lower.contains("ptr") || lower.contains("ref")
    {
        "property".to_string()
    } else if lower.contains("str") || lower.contains("string") ||
       lower.contains("vec") || lower.contains("array") || lower.contains("[")
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
        assert!(manager.get_ref_info(scopes[0].variables_reference).is_some());
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
}
