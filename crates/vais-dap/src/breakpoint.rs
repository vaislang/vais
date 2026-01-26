//! Breakpoint Management
//!
//! Handles breakpoint storage, verification, and resolution.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::protocol::types::{Breakpoint, Source, SourceBreakpoint, FunctionBreakpoint};

/// Manages breakpoints for a debug session
#[derive(Debug, Default)]
pub struct BreakpointManager {
    /// Next breakpoint ID
    next_id: AtomicI64,
    /// Source breakpoints indexed by file path
    source_breakpoints: HashMap<String, Vec<ManagedBreakpoint>>,
    /// Function breakpoints indexed by function name
    function_breakpoints: HashMap<String, ManagedBreakpoint>,
    /// Exception breakpoints
    exception_filters: Vec<String>,
}

/// Internal breakpoint representation
#[derive(Debug, Clone)]
pub struct ManagedBreakpoint {
    pub id: i64,
    pub verified: bool,
    pub line: i64,
    pub column: Option<i64>,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
    pub log_message: Option<String>,
    pub address: Option<u64>,
    pub function_name: Option<String>,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicI64::new(1),
            source_breakpoints: HashMap::new(),
            function_breakpoints: HashMap::new(),
            exception_filters: Vec::new(),
        }
    }

    fn next_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Set breakpoints for a source file, replacing any existing ones
    pub fn set_source_breakpoints(
        &mut self,
        source: &Source,
        breakpoints: &[SourceBreakpoint],
    ) -> Vec<ManagedBreakpoint> {
        let path = source.path.clone().unwrap_or_default();

        let managed: Vec<ManagedBreakpoint> = breakpoints.iter().map(|bp| {
            ManagedBreakpoint {
                id: self.next_id(),
                verified: false, // Will be verified by debugger
                line: bp.line,
                column: bp.column,
                condition: bp.condition.clone(),
                hit_condition: bp.hit_condition.clone(),
                log_message: bp.log_message.clone(),
                address: None,
                function_name: None,
            }
        }).collect();

        self.source_breakpoints.insert(path, managed.clone());
        managed
    }

    /// Set function breakpoints, replacing any existing ones
    pub fn set_function_breakpoints(
        &mut self,
        breakpoints: &[FunctionBreakpoint],
    ) -> Vec<ManagedBreakpoint> {
        self.function_breakpoints.clear();

        breakpoints.iter().map(|bp| {
            let managed = ManagedBreakpoint {
                id: self.next_id(),
                verified: false,
                line: 0,
                column: None,
                condition: bp.condition.clone(),
                hit_condition: bp.hit_condition.clone(),
                log_message: None,
                address: None,
                function_name: Some(bp.name.clone()),
            };
            self.function_breakpoints.insert(bp.name.clone(), managed.clone());
            managed
        }).collect()
    }

    /// Set exception filters
    pub fn set_exception_filters(&mut self, filters: Vec<String>) {
        self.exception_filters = filters;
    }

    /// Get all source breakpoints for a file
    pub fn get_source_breakpoints(&self, path: &str) -> Option<&Vec<ManagedBreakpoint>> {
        self.source_breakpoints.get(path)
    }

    /// Get all function breakpoints
    pub fn get_function_breakpoints(&self) -> impl Iterator<Item = &ManagedBreakpoint> {
        self.function_breakpoints.values()
    }

    /// Get exception filters
    pub fn get_exception_filters(&self) -> &[String] {
        &self.exception_filters
    }

    /// Mark a breakpoint as verified
    pub fn verify_breakpoint(&mut self, id: i64, address: u64, actual_line: Option<i64>) {
        for breakpoints in self.source_breakpoints.values_mut() {
            for bp in breakpoints.iter_mut() {
                if bp.id == id {
                    bp.verified = true;
                    bp.address = Some(address);
                    if let Some(line) = actual_line {
                        bp.line = line;
                    }
                    return;
                }
            }
        }

        for bp in self.function_breakpoints.values_mut() {
            if bp.id == id {
                bp.verified = true;
                bp.address = Some(address);
                return;
            }
        }
    }

    /// Find breakpoint by address
    pub fn find_by_address(&self, address: u64) -> Option<&ManagedBreakpoint> {
        for breakpoints in self.source_breakpoints.values() {
            for bp in breakpoints {
                if bp.address == Some(address) {
                    return Some(bp);
                }
            }
        }

        for bp in self.function_breakpoints.values() {
            if bp.address == Some(address) {
                return Some(bp);
            }
        }

        None
    }

    /// Convert managed breakpoint to DAP breakpoint
    pub fn to_dap_breakpoint(&self, bp: &ManagedBreakpoint, source: Option<Source>) -> Breakpoint {
        Breakpoint {
            id: Some(bp.id),
            verified: bp.verified,
            message: if bp.verified { None } else { Some("Pending".to_string()) },
            source,
            line: Some(bp.line),
            column: bp.column,
            end_line: None,
            end_column: None,
            instruction_reference: bp.address.map(|a| format!("0x{:x}", a)),
            offset: None,
        }
    }

    /// Check if breakpoint condition is met
    pub fn should_break(&self, bp: &ManagedBreakpoint, hit_count: u64) -> bool {
        // Check hit condition
        if let Some(ref hit_cond) = bp.hit_condition {
            if let Ok(threshold) = hit_cond.parse::<u64>() {
                if hit_count < threshold {
                    return false;
                }
            }
        }

        // Note: Actual condition evaluation happens in the debugger
        true
    }

    /// Get log message if this is a logpoint
    pub fn get_log_message<'a>(&self, bp: &'a ManagedBreakpoint) -> Option<&'a str> {
        bp.log_message.as_deref()
    }

    /// Clear all breakpoints
    pub fn clear_all(&mut self) {
        self.source_breakpoints.clear();
        self.function_breakpoints.clear();
        self.exception_filters.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_source_breakpoints() {
        let mut manager = BreakpointManager::new();

        let source = Source {
            path: Some("/test/file.vais".to_string()),
            ..Default::default()
        };

        let bps = vec![
            SourceBreakpoint { line: 10, column: None, condition: None, hit_condition: None, log_message: None },
            SourceBreakpoint { line: 20, column: Some(5), condition: Some("x > 0".to_string()), hit_condition: None, log_message: None },
        ];

        let managed = manager.set_source_breakpoints(&source, &bps);

        assert_eq!(managed.len(), 2);
        assert_eq!(managed[0].line, 10);
        assert_eq!(managed[1].line, 20);
        assert_eq!(managed[1].column, Some(5));
        assert_eq!(managed[1].condition, Some("x > 0".to_string()));
    }

    #[test]
    fn test_verify_breakpoint() {
        let mut manager = BreakpointManager::new();

        let source = Source {
            path: Some("/test/file.vais".to_string()),
            ..Default::default()
        };

        let bps = vec![
            SourceBreakpoint { line: 10, column: None, condition: None, hit_condition: None, log_message: None },
        ];

        let managed = manager.set_source_breakpoints(&source, &bps);
        let id = managed[0].id;

        assert!(!managed[0].verified);

        manager.verify_breakpoint(id, 0x1234, Some(11));

        let bps = manager.get_source_breakpoints("/test/file.vais").unwrap();
        assert!(bps[0].verified);
        assert_eq!(bps[0].address, Some(0x1234));
        assert_eq!(bps[0].line, 11);
    }
}
