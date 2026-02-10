//! Breakpoint Management
//!
//! Handles breakpoint storage, verification, and resolution.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::protocol::types::{Breakpoint, FunctionBreakpoint, Source, SourceBreakpoint};

/// Hit counter for tracking breakpoint hits
#[derive(Debug, Default)]
pub struct HitCounter {
    counts: HashMap<i64, u64>,  // breakpoint_id -> hit_count
}

impl HitCounter {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Increment hit count for a breakpoint and return the new count
    pub fn increment(&mut self, bp_id: i64) -> u64 {
        let count = self.counts.entry(bp_id).or_insert(0);
        *count += 1;
        *count
    }

    /// Get current hit count for a breakpoint
    pub fn get(&self, bp_id: i64) -> u64 {
        self.counts.get(&bp_id).copied().unwrap_or(0)
    }

    /// Reset hit count for a specific breakpoint
    pub fn reset(&mut self, bp_id: i64) {
        self.counts.remove(&bp_id);
    }

    /// Reset all hit counts
    pub fn reset_all(&mut self) {
        self.counts.clear();
    }
}

/// Hit condition operators
#[derive(Debug, Clone, PartialEq)]
pub enum HitConditionOp {
    Equal(u64),        // "= N" or just "N"
    GreaterEqual(u64), // ">= N"
    Greater(u64),      // "> N"
    Multiple(u64),     // "% N" (every Nth hit)
}

/// Parse hit condition string into an operator
pub fn parse_hit_condition(cond: &str) -> Option<HitConditionOp> {
    let cond = cond.trim();

    if let Some(rest) = cond.strip_prefix(">=") {
        rest.trim().parse::<u64>().ok().map(HitConditionOp::GreaterEqual)
    } else if let Some(rest) = cond.strip_prefix('>') {
        rest.trim().parse::<u64>().ok().map(HitConditionOp::Greater)
    } else if let Some(rest) = cond.strip_prefix('%') {
        rest.trim().parse::<u64>().ok().map(HitConditionOp::Multiple)
    } else if let Some(rest) = cond.strip_prefix('=') {
        rest.trim().parse::<u64>().ok().map(HitConditionOp::Equal)
    } else {
        cond.parse::<u64>().ok().map(HitConditionOp::Equal)
    }
}

/// Evaluate hit condition
pub fn evaluate_hit_condition(op: &HitConditionOp, hit_count: u64) -> bool {
    match op {
        HitConditionOp::Equal(n) => hit_count == *n,
        HitConditionOp::GreaterEqual(n) => hit_count >= *n,
        HitConditionOp::Greater(n) => hit_count > *n,
        HitConditionOp::Multiple(n) => *n > 0 && hit_count.is_multiple_of(*n),
    }
}

/// Result of recording a breakpoint hit
#[derive(Debug, PartialEq)]
pub enum HitResult {
    Break,          // Break execution
    Skip,           // Condition not met, continue
    Log(String),    // Logpoint, log message and continue
}

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
    /// Hit counter for tracking breakpoint hits
    hit_counter: HitCounter,
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
            hit_counter: HitCounter::new(),
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

        let managed: Vec<ManagedBreakpoint> = breakpoints
            .iter()
            .map(|bp| {
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
            })
            .collect();

        self.source_breakpoints.insert(path, managed.clone());
        managed
    }

    /// Set function breakpoints, replacing any existing ones
    pub fn set_function_breakpoints(
        &mut self,
        breakpoints: &[FunctionBreakpoint],
    ) -> Vec<ManagedBreakpoint> {
        self.function_breakpoints.clear();

        breakpoints
            .iter()
            .map(|bp| {
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
                self.function_breakpoints
                    .insert(bp.name.clone(), managed.clone());
                managed
            })
            .collect()
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

        self.function_breakpoints
            .values()
            .find(|&bp| bp.address == Some(address))
            .map(|v| v as _)
    }

    /// Convert managed breakpoint to DAP breakpoint
    pub fn to_dap_breakpoint(&self, bp: &ManagedBreakpoint, source: Option<Source>) -> Breakpoint {
        Breakpoint {
            id: Some(bp.id),
            verified: bp.verified,
            message: if bp.verified {
                None
            } else {
                Some("Pending".to_string())
            },
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
        self.hit_counter.reset_all();
    }

    /// Record a breakpoint hit and evaluate conditions
    pub fn record_hit(&mut self, bp: &ManagedBreakpoint) -> HitResult {
        let count = self.hit_counter.increment(bp.id);

        // Check hit condition
        if let Some(ref hit_cond) = bp.hit_condition {
            if let Some(op) = parse_hit_condition(hit_cond) {
                if !evaluate_hit_condition(&op, count) {
                    return HitResult::Skip;
                }
            }
        }

        // Check log message (logpoint)
        if let Some(ref msg) = bp.log_message {
            return HitResult::Log(msg.clone());
        }

        HitResult::Break
    }

    /// Get current hit count for a breakpoint
    pub fn get_hit_count(&self, bp_id: i64) -> u64 {
        self.hit_counter.get(bp_id)
    }

    /// Reset hit count for a specific breakpoint
    pub fn reset_hit_count(&mut self, bp_id: i64) {
        self.hit_counter.reset(bp_id);
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
            SourceBreakpoint {
                line: 10,
                column: None,
                condition: None,
                hit_condition: None,
                log_message: None,
            },
            SourceBreakpoint {
                line: 20,
                column: Some(5),
                condition: Some("x > 0".to_string()),
                hit_condition: None,
                log_message: None,
            },
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

        let bps = vec![SourceBreakpoint {
            line: 10,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
        }];

        let managed = manager.set_source_breakpoints(&source, &bps);
        let id = managed[0].id;

        assert!(!managed[0].verified);

        manager.verify_breakpoint(id, 0x1234, Some(11));

        let bps = manager.get_source_breakpoints("/test/file.vais").unwrap();
        assert!(bps[0].verified);
        assert_eq!(bps[0].address, Some(0x1234));
        assert_eq!(bps[0].line, 11);
    }

    #[test]
    fn test_hit_counter_increment() {
        let mut counter = HitCounter::new();

        assert_eq!(counter.get(1), 0);
        assert_eq!(counter.increment(1), 1);
        assert_eq!(counter.increment(1), 2);
        assert_eq!(counter.increment(1), 3);
        assert_eq!(counter.get(1), 3);

        assert_eq!(counter.increment(2), 1);
        assert_eq!(counter.get(2), 1);

        counter.reset(1);
        assert_eq!(counter.get(1), 0);

        counter.increment(2);
        counter.reset_all();
        assert_eq!(counter.get(2), 0);
    }

    #[test]
    fn test_parse_hit_condition_equal() {
        assert_eq!(parse_hit_condition("5"), Some(HitConditionOp::Equal(5)));
        assert_eq!(parse_hit_condition("= 5"), Some(HitConditionOp::Equal(5)));
        assert_eq!(parse_hit_condition("  =  10  "), Some(HitConditionOp::Equal(10)));
        assert_eq!(parse_hit_condition("invalid"), None);
    }

    #[test]
    fn test_parse_hit_condition_greater() {
        assert_eq!(parse_hit_condition("> 3"), Some(HitConditionOp::Greater(3)));
        assert_eq!(parse_hit_condition(">5"), Some(HitConditionOp::Greater(5)));
        assert_eq!(parse_hit_condition(">= 3"), Some(HitConditionOp::GreaterEqual(3)));
        assert_eq!(parse_hit_condition(">=10"), Some(HitConditionOp::GreaterEqual(10)));
        assert_eq!(parse_hit_condition("  >=  7  "), Some(HitConditionOp::GreaterEqual(7)));
    }

    #[test]
    fn test_parse_hit_condition_multiple() {
        assert_eq!(parse_hit_condition("% 10"), Some(HitConditionOp::Multiple(10)));
        assert_eq!(parse_hit_condition("%5"), Some(HitConditionOp::Multiple(5)));
        assert_eq!(parse_hit_condition("  %  3  "), Some(HitConditionOp::Multiple(3)));
    }

    #[test]
    fn test_evaluate_hit_condition() {
        // Equal
        assert!(evaluate_hit_condition(&HitConditionOp::Equal(5), 5));
        assert!(!evaluate_hit_condition(&HitConditionOp::Equal(5), 4));
        assert!(!evaluate_hit_condition(&HitConditionOp::Equal(5), 6));

        // Greater
        assert!(evaluate_hit_condition(&HitConditionOp::Greater(3), 4));
        assert!(!evaluate_hit_condition(&HitConditionOp::Greater(3), 3));
        assert!(!evaluate_hit_condition(&HitConditionOp::Greater(3), 2));

        // GreaterEqual
        assert!(evaluate_hit_condition(&HitConditionOp::GreaterEqual(3), 4));
        assert!(evaluate_hit_condition(&HitConditionOp::GreaterEqual(3), 3));
        assert!(!evaluate_hit_condition(&HitConditionOp::GreaterEqual(3), 2));

        // Multiple
        assert!(evaluate_hit_condition(&HitConditionOp::Multiple(10), 10));
        assert!(evaluate_hit_condition(&HitConditionOp::Multiple(10), 20));
        assert!(evaluate_hit_condition(&HitConditionOp::Multiple(10), 100));
        assert!(!evaluate_hit_condition(&HitConditionOp::Multiple(10), 5));
        assert!(!evaluate_hit_condition(&HitConditionOp::Multiple(10), 15));

        // Multiple with 0 should always return false
        assert!(!evaluate_hit_condition(&HitConditionOp::Multiple(0), 10));
    }

    #[test]
    fn test_record_hit_break() {
        let mut manager = BreakpointManager::new();

        let bp = ManagedBreakpoint {
            id: 1,
            verified: true,
            line: 10,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
            address: Some(0x1000),
            function_name: None,
        };

        // First hit should break
        assert_eq!(manager.record_hit(&bp), HitResult::Break);
        assert_eq!(manager.get_hit_count(1), 1);

        // Second hit should also break
        assert_eq!(manager.record_hit(&bp), HitResult::Break);
        assert_eq!(manager.get_hit_count(1), 2);
    }

    #[test]
    fn test_record_hit_skip() {
        let mut manager = BreakpointManager::new();

        let bp = ManagedBreakpoint {
            id: 1,
            verified: true,
            line: 10,
            column: None,
            condition: None,
            hit_condition: Some(">= 3".to_string()),
            log_message: None,
            address: Some(0x1000),
            function_name: None,
        };

        // First two hits should skip
        assert_eq!(manager.record_hit(&bp), HitResult::Skip);
        assert_eq!(manager.get_hit_count(1), 1);

        assert_eq!(manager.record_hit(&bp), HitResult::Skip);
        assert_eq!(manager.get_hit_count(1), 2);

        // Third hit should break
        assert_eq!(manager.record_hit(&bp), HitResult::Break);
        assert_eq!(manager.get_hit_count(1), 3);

        // Fourth hit should also break
        assert_eq!(manager.record_hit(&bp), HitResult::Break);
        assert_eq!(manager.get_hit_count(1), 4);
    }

    #[test]
    fn test_record_hit_logpoint() {
        let mut manager = BreakpointManager::new();

        let bp = ManagedBreakpoint {
            id: 1,
            verified: true,
            line: 10,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: Some("Value: {x}".to_string()),
            address: Some(0x1000),
            function_name: None,
        };

        // Should return log message
        assert_eq!(manager.record_hit(&bp), HitResult::Log("Value: {x}".to_string()));
        assert_eq!(manager.get_hit_count(1), 1);

        // Second hit should also log
        assert_eq!(manager.record_hit(&bp), HitResult::Log("Value: {x}".to_string()));
        assert_eq!(manager.get_hit_count(1), 2);
    }

    #[test]
    fn test_record_hit_multiple_condition() {
        let mut manager = BreakpointManager::new();

        let bp = ManagedBreakpoint {
            id: 1,
            verified: true,
            line: 10,
            column: None,
            condition: None,
            hit_condition: Some("% 3".to_string()),
            log_message: None,
            address: Some(0x1000),
            function_name: None,
        };

        // First two hits should skip
        assert_eq!(manager.record_hit(&bp), HitResult::Skip);
        assert_eq!(manager.record_hit(&bp), HitResult::Skip);

        // Third hit should break (multiple of 3)
        assert_eq!(manager.record_hit(&bp), HitResult::Break);

        // Fourth and fifth should skip
        assert_eq!(manager.record_hit(&bp), HitResult::Skip);
        assert_eq!(manager.record_hit(&bp), HitResult::Skip);

        // Sixth hit should break (multiple of 3)
        assert_eq!(manager.record_hit(&bp), HitResult::Break);

        assert_eq!(manager.get_hit_count(1), 6);
    }

    #[test]
    fn test_reset_hit_count() {
        let mut manager = BreakpointManager::new();

        let bp = ManagedBreakpoint {
            id: 1,
            verified: true,
            line: 10,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
            address: Some(0x1000),
            function_name: None,
        };

        manager.record_hit(&bp);
        manager.record_hit(&bp);
        assert_eq!(manager.get_hit_count(1), 2);

        manager.reset_hit_count(1);
        assert_eq!(manager.get_hit_count(1), 0);

        manager.record_hit(&bp);
        assert_eq!(manager.get_hit_count(1), 1);
    }
}
