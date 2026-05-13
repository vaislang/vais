//! Phase 32 Language Feature Edge Cases
//!
//! Tests for Vais language features focusing on edge cases not covered
//! by phase32.rs (capture modes, where clauses, pattern alias) or
//! phase45.rs (lazy/force, comptime basic, union parse, defer parse,
//! global parse):
//! - Defer inside a loop body

use super::helpers::*;

// ==================== Phase 32: Language Feature Edge Cases ====================

// Note: defer_with_early_return covered by phase37_comptime_defer.rs (e2e_p37_defer_with_early_return_zero)
// Note: pipe_operator_basic/chained covered by phase37_pipe_string.rs (e2e_p37_pipe_single/triple_chain)
// Note: global_variable_read/arithmetic covered by phase37_union_const.rs (e2e_p37_global_single/multiple)
// Note: union_field_access covered by phase37_union_const.rs (e2e_p37_union_single_field)
// Note: comptime_in_function covered by phase37_comptime_defer.rs (e2e_p37_comptime_in_helper_function)

// ===== Defer: Inside Loop Body =====

#[test]
fn e2e_phase32_defer_in_loop() {
    // defer inside a L loop iteration — 3 iterations, n=3
    let source = r#"
F main() -> i64 {
    n := mut 0
    L i:0..3 {
        D { }
        n = n + 1
    }
    R n
}
"#;
    assert_exit_code(source, 3);
}
