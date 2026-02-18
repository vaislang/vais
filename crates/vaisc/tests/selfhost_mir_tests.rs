//! Selfhost MIR optimization module compilation tests.
//! Verifies that all MIR-related .vais files in selfhost/ generate valid LLVM IR.
//!
//! Note: Most MIR files depend on constants and functions defined in mir.vais (the core
//! types module) and on each other. Because the vaisc test pipeline compiles each file in
//! isolation, those inter-module references are unresolved and the type-checker reports
//! UndefinedVar / UndefinedFunction errors. Tests that hit this limitation are marked
//! #[ignore] with a description of the missing symbol.

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Compile Vais source to LLVM IR string, returning the IR or an error message.
fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("selfhost_mir_test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen
        .generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

/// Compile a .vais file from disk to LLVM IR.
fn compile_file_to_ir(path: &str) -> Result<String, String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    compile_to_ir(&source)
}

// ---------------------------------------------------------------------------
// selfhost/mir.vais — MIR core types
// This file defines all fundamental MIR constants and types used by every
// other MIR module, and compiles standalone without external dependencies.
// ---------------------------------------------------------------------------
#[test]
fn selfhost_mir_core_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_alias.vais — alias analysis
// Requires: STMT_MIR_ASSIGN and other MIR opcode constants from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references STMT_MIR_ASSIGN from mir.vais"]
fn selfhost_mir_alias_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_alias.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_alias.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_analysis.vais — dominance / loop analysis
// Requires: TERM_GOTO and other terminator constants from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references TERM_GOTO from mir.vais"]
fn selfhost_mir_analysis_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_analysis.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_analysis.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_bounds.vais — value range analysis
// Requires: mir_dominance_analysis from mir_analysis.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references mir_dominance_analysis from mir_analysis.vais"]
fn selfhost_mir_bounds_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_bounds.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_bounds.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_vectorize.vais — loop vectorization
// Requires: mir_dominance_analysis from mir_analysis.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references mir_dominance_analysis from mir_analysis.vais"]
fn selfhost_mir_vectorize_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_vectorize.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_vectorize.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_layout.vais — struct layout optimization
// Requires: STMT_MIR_ASSIGN and other MIR opcode constants from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references STMT_MIR_ASSIGN from mir.vais"]
fn selfhost_mir_layout_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_layout.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_layout.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_optimizer.vais — optimization coordinator
// Requires: STMT_MIR_ASSIGN from mir.vais plus functions from mir_alias,
// mir_bounds, mir_vectorize, and mir_layout.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references STMT_MIR_ASSIGN from mir.vais and functions from mir_alias/bounds/vectorize/layout"]
fn selfhost_mir_optimizer_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_optimizer.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_optimizer.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_borrow.vais — borrow checking
// Requires: mir_liveness_analysis from mir_analysis.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references mir_liveness_analysis from mir_analysis.vais"]
fn selfhost_mir_borrow_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_borrow.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_borrow.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_builder.vais — MIR construction
// Requires: mir_local_decl and other constructor functions from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references mir_local_decl from mir.vais"]
fn selfhost_mir_builder_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_builder.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_builder.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_lower.vais — AST→MIR lowering
// Requires: BINOP_ADD and other binary operator constants from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references BINOP_ADD from mir.vais"]
fn selfhost_mir_lower_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_lower.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_lower.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_emit_llvm.vais — MIR→LLVM emission
// Requires: MIR_TY_I8 and other MIR type constants from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references MIR_TY_I8 from mir.vais"]
fn selfhost_mir_emit_llvm_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_emit_llvm.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_emit_llvm.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_main.vais — MIR module coordinator
// Requires: Lexer::new from lexer.vais (and transitively all MIR modules).
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references Lexer::new from lexer.vais and functions from all MIR submodules"]
fn selfhost_mir_main_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_main.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_main.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_test_borrow.vais — borrow checker tests
// Requires: mir_local_decl and other constructor functions from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references mir_local_decl from mir.vais"]
fn selfhost_mir_test_borrow_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_test_borrow.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_test_borrow.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// selfhost/mir_test_opt.vais — optimization tests
// Requires: STMT_MIR_ASSIGN and other MIR opcode constants from mir.vais.
// Cannot compile standalone; needs multi-module compilation support.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "cross-module dependency: references STMT_MIR_ASSIGN from mir.vais"]
fn selfhost_mir_test_opt_compiles() {
    let result = compile_file_to_ir("../../selfhost/mir_test_opt.vais");
    assert!(
        result.is_ok(),
        "selfhost/mir_test_opt.vais failed to compile to IR: {}",
        result.unwrap_err()
    );
}
