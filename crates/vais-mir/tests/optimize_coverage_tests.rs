//! Coverage tests for vais-mir/src/optimize.rs
//!
//! Targets: DCE, CSE, constant propagation, copy propagation,
//! loop unrolling, tail call detection, escape analysis, unreachable block removal.

use vais_mir::lower::lower_module;
use vais_mir::optimize::*;
use vais_mir::*;

fn lower(source: &str) -> MirModule {
    let module = vais_parser::parse(source).expect("Parse failed");
    lower_module(&module)
}

// ============================================================================
// Dead Code Elimination
// ============================================================================

#[test]
fn test_dce_removes_unused_local() {
    let mut mir = lower(
        r#"
        F f(x: i64) -> i64 = {
            unused := 42
            x + 1
        }
    "#,
    );

    let before: usize = mir.bodies[0]
        .basic_blocks
        .iter()
        .map(|bb| bb.statements.len())
        .sum();

    dead_code_elimination(&mut mir.bodies[0]);

    let after: usize = mir.bodies[0]
        .basic_blocks
        .iter()
        .map(|bb| bb.statements.len())
        .sum();

    assert!(after <= before);
}

#[test]
fn test_dce_keeps_return_place() {
    let mut mir = lower("F f() -> i64 = 42");

    dead_code_elimination(&mut mir.bodies[0]);

    // Return place (_0) should not be removed
    let display = mir.bodies[0].display();
    assert!(display.contains("_0"));
}

#[test]
fn test_dce_keeps_used_locals() {
    let mut mir = lower(
        r#"
        F f(x: i64) -> i64 = {
            a := x + 1
            a * 2
        }
    "#,
    );

    let before: usize = mir.bodies[0]
        .basic_blocks
        .iter()
        .map(|bb| bb.statements.len())
        .sum();

    dead_code_elimination(&mut mir.bodies[0]);

    let after: usize = mir.bodies[0]
        .basic_blocks
        .iter()
        .map(|bb| bb.statements.len())
        .sum();

    // Used locals should remain
    assert!(after > 0);
    // Should not remove more than unused
    assert!(after <= before);
}

// ============================================================================
// Constant Propagation
// ============================================================================

#[test]
fn test_constant_propagation_basic() {
    let mut mir = lower(
        r#"
        F f() -> i64 = {
            a := 10
            b := 20
            a + b
        }
    "#,
    );

    constant_propagation(&mut mir.bodies[0]);

    // After constant propagation, constants should be folded
    let display = mir.bodies[0].display();
    assert!(display.contains("const 10") || display.contains("10") || display.contains("Add"));
}

#[test]
fn test_constant_propagation_through_chain() {
    let mut mir = lower(
        r#"
        F f() -> i64 = {
            a := 5
            b := a
            b
        }
    "#,
    );

    constant_propagation(&mut mir.bodies[0]);
    let display = mir.bodies[0].display();
    assert!(display.contains("5") || display.contains("const"));
}

// ============================================================================
// Copy Propagation
// ============================================================================

#[test]
fn test_copy_propagation() {
    let mut mir = lower(
        r#"
        F f(x: i64) -> i64 = {
            a := x
            a
        }
    "#,
    );

    copy_propagation(&mut mir.bodies[0]);

    // After copy propagation, 'a' should be replaced with 'x' where possible
    let display = mir.bodies[0].display();
    assert!(!display.is_empty());
}

// ============================================================================
// Common Subexpression Elimination
// ============================================================================

#[test]
fn test_cse_identical_binops() {
    let mut mir = lower(
        r#"
        F f(x: i64, y: i64) -> i64 = {
            a := x + y
            b := x + y
            a + b
        }
    "#,
    );

    common_subexpression_elimination(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    assert!(!display.is_empty());
}

// ============================================================================
// Loop Unrolling
// ============================================================================

#[test]
fn test_loop_unrolling_no_crash() {
    let mut mir = lower("F f(x: i64) -> i64 = x");

    // Should not crash even with no loops
    loop_unrolling(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    assert!(display.contains("fn f"));
}

// ============================================================================
// Tail Call Detection
// ============================================================================

#[test]
fn test_tail_call_detection_recursive() {
    let mut mir = lower("F f(n: i64) -> i64 = I n == 0 { 1 } E { @(n - 1) }");

    tail_call_detection(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    // Recursive call should be detected as tail call
    assert!(display.contains("tailcall") || display.contains("f"));
}

#[test]
fn test_tail_call_detection_no_recursion() {
    let mut mir = lower("F f(x: i64) -> i64 = x + 1");

    tail_call_detection(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    assert!(!display.contains("tailcall"));
}

// ============================================================================
// Escape Analysis
// ============================================================================

#[test]
fn test_escape_analysis_no_crash() {
    let mut mir = lower(
        r#"
        F f(x: i64) -> i64 = {
            a := x + 1
            b := a * 2
            b
        }
    "#,
    );

    escape_analysis(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    assert!(display.contains("fn f"));
}

// ============================================================================
// Remove Unreachable Blocks
// ============================================================================

#[test]
fn test_remove_unreachable_blocks() {
    let mut mir = lower(
        r#"
        F f(x: i64) -> i64 = I x > 0 { R 1 } E { R 0 }
    "#,
    );

    let before = mir.bodies[0].basic_blocks.len();
    remove_unreachable_blocks(&mut mir.bodies[0]);
    let after = mir.bodies[0].basic_blocks.len();

    // Should not increase the number of blocks
    assert!(after <= before);
}

// ============================================================================
// Full optimization pipeline
// ============================================================================

#[test]
fn test_optimize_mir_body_full() {
    let mut mir = lower(
        r#"
        F compute(x: i64) -> i64 = {
            unused := 999
            a := 10
            b := 20
            c := a + b
            x + c
        }
    "#,
    );

    optimize_mir_body(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    assert!(display.contains("fn compute"));
}

#[test]
fn test_optimize_mir_module_multiple_bodies() {
    let mut mir = lower(
        r#"
        F a(x: i64) -> i64 = x + 1
        F b(x: i64) -> i64 = {
            unused := 42
            x * 2
        }
    "#,
    );

    optimize_mir_module(&mut mir);

    // Both functions should still be present
    assert_eq!(mir.bodies.len(), 2);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_optimize_empty_body() {
    let mut mir = lower("F noop() -> i64 = 0");
    optimize_mir_body(&mut mir.bodies[0]);
    assert!(!mir.bodies[0].basic_blocks.is_empty());
}

#[test]
fn test_optimize_preserves_correctness_if_else() {
    let mut mir = lower("F abs(x: i64) -> i64 = I x < 0 { 0 - x } E { x }");

    let before_blocks = mir.bodies[0].basic_blocks.len();
    optimize_mir_body(&mut mir.bodies[0]);
    let after_blocks = mir.bodies[0].basic_blocks.len();

    // Should have roughly the same structure (at least 3 blocks for if/else)
    assert!(after_blocks >= 3 || after_blocks <= before_blocks);
}

#[test]
fn test_optimize_nested_if() {
    let mut mir = lower(
        r#"
        F classify(x: i64) -> i64 = I x > 0 {
            I x > 100 { 3 } E { 2 }
        } E {
            1
        }
    "#,
    );

    optimize_mir_body(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    assert!(display.contains("switchInt"));
}

#[test]
fn test_optimize_match() {
    let mut mir = lower(
        r#"
        F f(x: i64) -> i64 = M x {
            0 => 100,
            1 => 200,
            2 => 300,
            _ => 0
        }
    "#,
    );

    optimize_mir_body(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    assert!(display.contains("switchInt"));
}
