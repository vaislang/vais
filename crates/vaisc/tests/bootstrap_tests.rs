//! Bootstrap verification tests for the Vais self-hosting compiler
//!
//! These tests verify that the self-hosting compiler source files
//! can be compiled by the Rust-based compiler (Stage 0 → Stage 1 path).
//! Full Stage 2/3 verification requires the bootstrap-verify.sh script.

use std::path::Path;

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// ============================================================================
// Helper functions for IR generation
// ============================================================================

/// Compile Vais source code to LLVM IR string, returning the IR or an error.
fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("selfhost_test");
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

/// Returns the absolute path to a selfhost file given its relative name.
fn selfhost_path(file: &str) -> String {
    let project_root = env!("CARGO_MANIFEST_DIR");
    format!("{}/../..", project_root)
        .parse::<std::path::PathBuf>()
        .unwrap()
        .join("selfhost")
        .join(file)
        .to_string_lossy()
        .to_string()
}

// ============================================================================
// Existing tests (unchanged)
// ============================================================================

/// Verify selfhost source files exist
#[test]
fn selfhost_source_files_exist() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let required_files = [
        "main_entry.vais",
        "constants.vais",
        "stringbuffer_s1.vais",
        "lexer_s1.vais",
        "helpers_s1.vais",
        "parser_s1.vais",
        "codegen_s1.vais",
        "runtime.c",
        "bootstrap_test.vais",
    ];

    for file in &required_files {
        let path = selfhost_dir.join(file);
        assert!(
            path.exists(),
            "Required selfhost file missing: {}",
            path.display()
        );
    }
}

/// Verify bootstrap verification script exists and is executable
#[test]
fn bootstrap_script_exists() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts/bootstrap-verify.sh");

    assert!(
        script.exists(),
        "Bootstrap verification script missing: {}",
        script.display()
    );
}

/// Verify selfhost main_entry.vais can be tokenized by the Rust compiler
#[test]
fn selfhost_main_entry_tokenizes() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let source = std::fs::read_to_string(selfhost_dir.join("main_entry.vais"))
        .expect("Failed to read main_entry.vais");

    // Verify it can be tokenized (basic sanity check)
    let result = vais_lexer::tokenize(&source);
    assert!(
        result.is_ok(),
        "Failed to tokenize main_entry.vais: {:?}",
        result.err()
    );
}

/// Verify selfhost constants.vais can be tokenized
#[test]
fn selfhost_constants_tokenizes() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let source = std::fs::read_to_string(selfhost_dir.join("constants.vais"))
        .expect("Failed to read constants.vais");

    let result = vais_lexer::tokenize(&source);
    assert!(
        result.is_ok(),
        "Failed to tokenize constants.vais: {:?}",
        result.err()
    );
}

/// Verify runtime.c compiles (syntax check)
#[test]
fn selfhost_runtime_c_valid() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let source =
        std::fs::read_to_string(selfhost_dir.join("runtime.c")).expect("Failed to read runtime.c");

    // Basic validity checks
    assert!(source.contains("#include <stdint.h>"));
    assert!(source.contains("load_i64"));
    assert!(source.contains("store_i64"));
    assert!(source.contains("vais_gc_alloc"));
}

// ============================================================================
// Stage 1 files IR generation tests
// ============================================================================

/// lexer_s1.vais depends on token.vais constants (TOK_INT etc.) which are not
/// in scope when compiled standalone — requires multi-module compilation.
#[test]
#[ignore = "lexer_s1.vais depends on token.vais symbols (TOK_INT etc.) unavailable in standalone compilation"]
fn bootstrap_stage1_lexer_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("lexer_s1.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// parser_s1.vais depends on token.vais symbols (token_get_kind etc.) which are
/// not in scope when compiled standalone — requires multi-module compilation.
#[test]
#[ignore = "parser_s1.vais depends on token.vais symbols unavailable in standalone compilation"]
fn bootstrap_stage1_parser_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("parser_s1.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// codegen_s1.vais depends on stringbuffer functions (sb_new etc.) which are
/// not in scope when compiled standalone — requires multi-module compilation.
#[test]
#[ignore = "codegen_s1.vais depends on stringbuffer symbols (sb_new etc.) unavailable in standalone compilation"]
fn bootstrap_stage1_codegen_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("codegen_s1.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stage1_helpers_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("helpers_s1.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stage1_stringbuffer_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("stringbuffer_s1.vais")).unwrap();
    assert!(!ir.is_empty());
}

// ============================================================================
// Core selfhost files IR generation tests
// ============================================================================

/// ast.vais has an ExprNode type mismatch that requires the full module context
/// to resolve — standalone compilation fails with a type error.
#[test]
#[ignore = "ast.vais has an ExprNode type mismatch that requires full module context to resolve"]
fn bootstrap_core_ast_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("ast.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// main.vais depends on parser_check and other cross-module symbols not present
/// in standalone compilation.
#[test]
#[ignore = "main.vais depends on parser_check and other cross-module symbols unavailable in standalone compilation"]
fn bootstrap_core_main_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("main.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_core_codegen_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("codegen.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_core_type_checker_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("type_checker.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_core_parser_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("parser.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_core_module_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("module.vais")).unwrap();
    assert!(!ir.is_empty());
}

// ============================================================================
// Selfhost stdlib files compilation tests
// ============================================================================

#[test]
fn bootstrap_stdlib_vec_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("vec.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stdlib_string_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("string.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// hashmap.vais depends on vec_new from vec.vais which is not in scope when
/// compiled standalone — requires multi-module compilation.
#[test]
#[ignore = "hashmap.vais depends on vec_new and other vec.vais symbols unavailable in standalone compilation"]
fn bootstrap_stdlib_hashmap_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("hashmap.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stdlib_option_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("option.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stdlib_print_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("print.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stdlib_span_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("span.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stdlib_token_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("token.vais")).unwrap();
    assert!(!ir.is_empty());
}

#[test]
fn bootstrap_stdlib_constants_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("constants.vais")).unwrap();
    assert!(!ir.is_empty());
}

// ============================================================================
// LSP/Tools compilation tests
// ============================================================================

/// lsp_handlers.vais depends on jb_new (JSON builder) from lsp_json.vais which
/// is not in scope when compiled standalone.
#[test]
#[ignore = "lsp_handlers.vais depends on jb_new and other lsp_json.vais symbols unavailable in standalone compilation"]
fn bootstrap_lsp_handlers_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("lsp_handlers.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// lsp_symbols.vais depends on item_get_kind and other cross-module symbols.
#[test]
#[ignore = "lsp_symbols.vais depends on item_get_kind and other cross-module symbols unavailable in standalone compilation"]
fn bootstrap_lsp_symbols_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("lsp_symbols.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// lsp_json.vais depends on sb_new (string buffer) from stringbuffer_s1.vais
/// which is not in scope when compiled standalone.
#[test]
#[ignore = "lsp_json.vais depends on sb_new (stringbuffer_s1.vais) unavailable in standalone compilation"]
fn bootstrap_lsp_json_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("lsp_json.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// lsp_main.vais depends on symtable_new and other multi-module symbols.
#[test]
#[ignore = "lsp_main.vais depends on symtable_new and other cross-module symbols unavailable in standalone compilation"]
fn bootstrap_lsp_main_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("lsp_main.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// fmt.vais depends on sb_new (string buffer) from stringbuffer_s1.vais
/// which is not in scope when compiled standalone.
#[test]
#[ignore = "fmt.vais depends on sb_new (stringbuffer_s1.vais) unavailable in standalone compilation"]
fn bootstrap_fmt_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("fmt.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// fmt_main.vais depends on lexer_new and other multi-module symbols.
#[test]
#[ignore = "fmt_main.vais depends on lexer_new and other cross-module symbols unavailable in standalone compilation"]
fn bootstrap_fmt_main_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("fmt_main.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// doc_gen.vais depends on sb_new (string buffer) from stringbuffer_s1.vais
/// which is not in scope when compiled standalone.
#[test]
#[ignore = "doc_gen.vais depends on sb_new (stringbuffer_s1.vais) unavailable in standalone compilation"]
fn bootstrap_doc_gen_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("doc_gen.vais")).unwrap();
    assert!(!ir.is_empty());
}

/// doc_gen_main.vais depends on lexer_new and other multi-module symbols.
#[test]
#[ignore = "doc_gen_main.vais depends on lexer_new and other cross-module symbols unavailable in standalone compilation"]
fn bootstrap_doc_gen_main_compiles() {
    let ir = compile_file_to_ir(&selfhost_path("doc_gen_main.vais")).unwrap();
    assert!(!ir.is_empty());
}
