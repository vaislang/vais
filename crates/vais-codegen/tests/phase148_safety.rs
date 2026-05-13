//! Phase 148 safety hardening tests — match codegen void/Unit phi fix.
//!
//! Verifies that match expressions where arms produce void/Unit results
//! do not generate `phi void` instructions (invalid LLVM IR).

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

// ============================================================================
// match void/Unit arm — phi void fix
// ============================================================================

/// Match arms returning i64 values must still use phi node (non-void path unaffected).
#[test]
fn test_match_non_void_arms_uses_phi() {
    let ir = gen_ok(
        r#"
        F do_a() -> i64 { R 1 }
        F do_b() -> i64 { R 2 }
        F main() -> i64 {
            x := 1
            result := M x {
                1 => do_a(),
                _ => do_b()
            }
            R result
        }
    "#,
    );
    // Non-void arms must produce a phi node
    assert!(
        ir.contains("phi i64"),
        "Expected 'phi i64' in IR for non-void match arms, got:\n{}",
        ir
    );
    // Must NOT contain phi void
    assert!(
        !ir.contains("phi void"),
        "Unexpected 'phi void' in IR:\n{}",
        ir
    );
}

/// Integer switch-style match (all literal arms) also must not produce phi void.
#[test]
fn test_match_switch_style_non_void() {
    let ir = gen_ok(
        r#"
        F main() -> i64 {
            x := 2
            result := M x {
                1 => 10,
                2 => 20,
                _ => 0
            }
            R result
        }
    "#,
    );
    assert!(
        !ir.contains("phi void"),
        "Unexpected 'phi void' in switch-style match IR:\n{}",
        ir
    );
}

/// Match where the resolved type is Unit — should use void_placeholder_ir, not phi.
#[test]
fn test_match_unit_arms_no_phi_void() {
    // Arms that call Unit-returning functions.
    // The codegen should emit "add i64 0, 0 ; void/Unit placeholder" instead of "phi void".
    let ir = gen_ok(
        r#"
        F side_effect_a() { }
        F side_effect_b() { }
        F run(x: i64) {
            M x {
                1 => side_effect_a(),
                _ => side_effect_b()
            }
        }
        F main() -> i64 {
            run(1)
            R 0
        }
    "#,
    );
    // Must NOT contain phi void
    assert!(
        !ir.contains("phi void"),
        "Unexpected 'phi void' in Unit-arm match IR:\n{}",
        ir
    );
}

/// Verify the void placeholder comment is present when Unit arms are used.
#[test]
fn test_match_unit_arms_has_placeholder_comment() {
    let ir = gen_ok(
        r#"
        F noop() { }
        F run(x: i64) {
            M x {
                1 => noop(),
                _ => noop()
            }
        }
        F main() -> i64 {
            run(1)
            R 0
        }
    "#,
    );
    // void_placeholder_ir emits this comment
    assert!(
        ir.contains("void/Unit placeholder"),
        "Expected 'void/Unit placeholder' comment in Unit-arm match IR, got:\n{}",
        ir
    );
}
