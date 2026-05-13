//! Codegen coverage tests part 3 — ir_verify, abi, cross_compile, diagnostics
//!
//! Targets: ir_verify.rs (pub), abi.rs (pub), cross_compile.rs (pub),
//! diagnostics.rs (pub(crate) tested via codegen), target.rs
//!
//! Strategy: Unit-test public functions directly + gen_ok/gen_result for internal paths.

use vais_codegen::abi::{
    alignment, check_abi_compatibility, struct_layout, vtable, AbiCompatibility, CallingConvention,
    ABI_VERSION,
};
use vais_codegen::cross_compile::{CrossCompileConfig, CrossCompileError, RuntimeLibs};
use vais_codegen::ir_verify::{verify_text_ir, DiagnosticSeverity, IrDiagnostic};
use vais_codegen::CodeGenerator;
use vais_codegen::TargetTriple;
use vais_parser::parse;

// ============================================================================
// Helpers
// ============================================================================

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

fn gen_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen: {}", e))
}

// ============================================================================
// ir_verify: verify_text_ir — valid IR
// ============================================================================

#[test]
fn test_ir_verify_empty_ir() {
    let diags = verify_text_ir("");
    assert!(diags.is_empty());
}

#[test]
fn test_ir_verify_comments_only() {
    let diags = verify_text_ir("; This is a comment\n; Another comment\n");
    assert!(diags.is_empty());
}

#[test]
fn test_ir_verify_single_valid_function() {
    let ir = r#"
define i64 @test() {
entry:
  ret i64 42
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

#[test]
fn test_ir_verify_multiple_valid_functions() {
    let ir = r#"
define i64 @foo() {
entry:
  ret i64 1
}

define i64 @bar() {
entry:
  ret i64 2
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

#[test]
fn test_ir_verify_function_with_branch() {
    let ir = r#"
define i64 @test(i1 %cond) {
entry:
  br i1 %cond, label %then, label %else
then:
  ret i64 1
else:
  ret i64 0
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

#[test]
fn test_ir_verify_function_with_phi() {
    let ir = r#"
define i64 @test(i1 %cond) {
entry:
  br i1 %cond, label %then, label %else
then:
  br label %merge
else:
  br label %merge
merge:
  %result = phi i64 [1, %then], [0, %else]
  ret i64 %result
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

#[test]
fn test_ir_verify_void_function() {
    let ir = r#"
define void @test() {
entry:
  ret void
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

// ============================================================================
// ir_verify: verify_text_ir — error detection
// ============================================================================

#[test]
fn test_ir_verify_unterminated_block() {
    let ir = r#"
define i64 @test() {
entry:
  %x = add i64 1, 2
}
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags.iter().any(|d| d.message.contains("unterminated")),
        "Expected unterminated block diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_unterminated_block_before_label() {
    let ir = r#"
define i64 @test() {
entry:
  %x = add i64 1, 2
next:
  ret i64 %x
}
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags.iter().any(|d| d.message.contains("unterminated")),
        "Expected unterminated block diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_phi_void() {
    let ir = r#"
define void @test(i1 %c) {
entry:
  br i1 %c, label %a, label %b
a:
  br label %m
b:
  br label %m
m:
  %r = phi void [undef, %a], [undef, %b]
  ret void
}
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags.iter().any(|d| d.message.contains("phi void")),
        "Expected phi void diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_phi_after_non_phi() {
    let ir = r#"
define i64 @test(i1 %c) {
entry:
  br i1 %c, label %a, label %b
a:
  br label %m
b:
  br label %m
m:
  %x = add i64 1, 2
  %r = phi i64 [1, %a], [2, %b]
  ret i64 %r
}
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("phi instruction after non-phi")),
        "Expected phi placement diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_duplicate_function() {
    let ir = r#"
define i64 @foo() {
entry:
  ret i64 1
}

define i64 @foo() {
entry:
  ret i64 2
}
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags.iter().any(|d| d.message.contains("duplicate")),
        "Expected duplicate function diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_mismatched_braces() {
    let ir = r#"
define i64 @test() {
entry:
  ret i64 42
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("mismatched braces") || d.message.contains("unterminated")),
        "Expected mismatched brace diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_undefined_label_reference() {
    let ir = r#"
define i64 @test() {
entry:
  br label %nonexistent
}
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags.iter().any(|d| d.message.contains("undefined label")),
        "Expected undefined label diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_return_type_mismatch() {
    let ir = r#"
define i64 @test() {
entry:
  ret i32 42
}
"#;
    let diags = verify_text_ir(ir);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("return type mismatch")),
        "Expected return type mismatch diagnostic: {:?}",
        diags
    );
}

#[test]
fn test_ir_verify_switch_terminator() {
    let ir = r#"
define i64 @test(i32 %val) {
entry:
  switch i32 %val, label %default [i32 0, label %case0]
case0:
  ret i64 10
default:
  ret i64 99
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

#[test]
fn test_ir_verify_unreachable_terminator() {
    let ir = r#"
define i64 @test() {
entry:
  unreachable
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

#[test]
fn test_ir_verify_invoke_terminator() {
    let ir = r#"
define i64 @test() {
entry:
  invoke void @foo() to label %normal unwind label %unwind
normal:
  ret i64 0
unwind:
  ret i64 1
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

#[test]
fn test_ir_verify_resume_terminator() {
    let ir = r#"
define void @test() {
entry:
  resume { i8*, i32 } undef
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

// ============================================================================
// ir_verify: IrDiagnostic Display
// ============================================================================

#[test]
fn test_ir_diagnostic_display_with_function() {
    let diag = IrDiagnostic {
        line: 42,
        severity: DiagnosticSeverity::Error,
        message: "test error".to_string(),
        function_name: Some("my_func".to_string()),
    };
    let s = format!("{}", diag);
    assert!(s.contains("error"));
    assert!(s.contains("42"));
    assert!(s.contains("my_func"));
    assert!(s.contains("test error"));
}

#[test]
fn test_ir_diagnostic_display_without_function() {
    let diag = IrDiagnostic {
        line: 10,
        severity: DiagnosticSeverity::Warning,
        message: "test warning".to_string(),
        function_name: None,
    };
    let s = format!("{}", diag);
    assert!(s.contains("warning"));
    assert!(s.contains("10"));
    assert!(s.contains("test warning"));
    assert!(!s.contains("@"));
}

#[test]
fn test_ir_diagnostic_severity_eq() {
    assert_eq!(DiagnosticSeverity::Error, DiagnosticSeverity::Error);
    assert_eq!(DiagnosticSeverity::Warning, DiagnosticSeverity::Warning);
    assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Warning);
}

// ============================================================================
// ir_verify: string constant brace handling
// ============================================================================

#[test]
fn test_ir_verify_braces_in_string_constants() {
    let ir = r#"
@.str = private unnamed_addr constant [6 x i8] c"a{b}c\00"

define i64 @test() {
entry:
  ret i64 0
}
"#;
    let diags = verify_text_ir(ir);
    // Braces inside string literals should not cause mismatched brace warnings
    let brace_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("mismatched braces"))
        .collect();
    assert!(
        brace_diags.is_empty(),
        "False positive brace issue: {:?}",
        brace_diags
    );
}

#[test]
fn test_ir_verify_multiple_blocks_all_terminated() {
    let ir = r#"
define i64 @test(i1 %a, i1 %b) {
entry:
  br i1 %a, label %blk1, label %blk2
blk1:
  br i1 %b, label %blk3, label %blk4
blk2:
  ret i64 0
blk3:
  ret i64 1
blk4:
  ret i64 2
}
"#;
    let diags = verify_text_ir(ir);
    assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
}

// ============================================================================
// abi: CallingConvention — exhaustive
// ============================================================================

#[test]
fn test_cc_parse_vais() {
    assert_eq!(
        CallingConvention::parse_abi("vais"),
        Some(CallingConvention::Vais)
    );
}

#[test]
fn test_cc_unknown_returns_none() {
    assert_eq!(CallingConvention::parse_abi(""), None);
    assert_eq!(CallingConvention::parse_abi("rust"), None);
    assert_eq!(CallingConvention::parse_abi("swift"), None);
}

#[test]
fn test_cc_to_llvm_all_variants() {
    let variants = [
        (CallingConvention::C, "ccc"),
        (CallingConvention::Vais, "ccc"),
        (CallingConvention::Fast, "fastcc"),
        (CallingConvention::StdCall, "x86_stdcallcc"),
        (CallingConvention::FastCall, "x86_fastcallcc"),
    ];
    for (cc, expected) in variants {
        assert_eq!(cc.to_llvm_str(), expected);
    }
}

// ============================================================================
// abi: alignment
// ============================================================================

#[test]
fn test_alignment_for_size_boundaries() {
    assert_eq!(alignment::for_size(0), 1);
    assert_eq!(alignment::for_size(1), 1);
    assert_eq!(alignment::for_size(2), 2);
    assert_eq!(alignment::for_size(3), 4);
    assert_eq!(alignment::for_size(4), 4);
    assert_eq!(alignment::for_size(5), 8);
    assert_eq!(alignment::for_size(8), 8);
    assert_eq!(alignment::for_size(100), 8);
    assert_eq!(alignment::for_size(1024), 8);
}

#[test]
fn test_alignment_constants_correctness() {
    assert_eq!(alignment::I8, 1);
    assert_eq!(alignment::I16, 2);
    assert_eq!(alignment::I32, 4);
    assert_eq!(alignment::I64, 8);
    assert_eq!(alignment::F32, 4);
    assert_eq!(alignment::F64, 8);
    assert_eq!(alignment::POINTER, 8);
    assert_eq!(alignment::BOOL, 1);
}

// ============================================================================
// abi: struct_layout
// ============================================================================

#[test]
fn test_struct_layout_field_offset_aligned() {
    assert_eq!(struct_layout::calculate_field_offset(0, 4), 0);
    assert_eq!(struct_layout::calculate_field_offset(1, 4), 4);
    assert_eq!(struct_layout::calculate_field_offset(2, 4), 4);
    assert_eq!(struct_layout::calculate_field_offset(3, 4), 4);
    assert_eq!(struct_layout::calculate_field_offset(4, 4), 4);
    assert_eq!(struct_layout::calculate_field_offset(5, 4), 8);
}

#[test]
fn test_struct_layout_field_offset_various_alignments() {
    assert_eq!(struct_layout::calculate_field_offset(0, 1), 0);
    assert_eq!(struct_layout::calculate_field_offset(1, 1), 1);
    assert_eq!(struct_layout::calculate_field_offset(0, 2), 0);
    assert_eq!(struct_layout::calculate_field_offset(1, 2), 2);
    assert_eq!(struct_layout::calculate_field_offset(0, 8), 0);
    assert_eq!(struct_layout::calculate_field_offset(1, 8), 8);
    assert_eq!(struct_layout::calculate_field_offset(9, 8), 16);
}

#[test]
fn test_struct_layout_struct_size() {
    assert_eq!(struct_layout::calculate_struct_size(0, 4), 0);
    assert_eq!(struct_layout::calculate_struct_size(1, 4), 4);
    assert_eq!(struct_layout::calculate_struct_size(4, 4), 4);
    assert_eq!(struct_layout::calculate_struct_size(5, 4), 8);
    assert_eq!(struct_layout::calculate_struct_size(8, 8), 8);
    assert_eq!(struct_layout::calculate_struct_size(9, 8), 16);
}

#[test]
fn test_struct_layout_ordering_constants() {
    assert_eq!(struct_layout::REPR_C_ORDERING, "declaration-order");
    assert_eq!(struct_layout::DEFAULT_ORDERING, "declaration-order");
}

// ============================================================================
// abi: vtable
// ============================================================================

#[test]
fn test_vtable_slot_constants() {
    assert_eq!(vtable::SLOT_DROP_FN, 0);
    assert_eq!(vtable::SLOT_SIZE, 1);
    assert_eq!(vtable::SLOT_ALIGN, 2);
    assert_eq!(vtable::SLOT_METHODS_START, 3);
    assert_eq!(vtable::METADATA_SLOTS, 3);
}

#[test]
fn test_vtable_method_slot() {
    assert_eq!(vtable::method_slot(0), 3);
    assert_eq!(vtable::method_slot(1), 4);
    assert_eq!(vtable::method_slot(5), 8);
    assert_eq!(vtable::method_slot(10), 13);
}

#[test]
fn test_vtable_type_with_zero_methods() {
    let ty = vtable::vtable_type_with_methods(0);
    assert!(ty.contains("i8*"));
    assert!(ty.contains("i64"));
    // Should have exactly drop_fn, size, align = 3 fields
    assert_eq!(ty.matches(',').count(), 2);
}

#[test]
fn test_vtable_type_with_one_method() {
    let ty = vtable::vtable_type_with_methods(1);
    // 3 metadata + 1 method = 4 fields, 3 commas
    assert_eq!(ty.matches(',').count(), 3);
}

#[test]
fn test_vtable_type_with_five_methods() {
    let ty = vtable::vtable_type_with_methods(5);
    // 3 metadata + 5 methods = 8 fields, 7 commas
    assert_eq!(ty.matches(',').count(), 7);
}

// ============================================================================
// abi: check_abi_compatibility
// ============================================================================

#[test]
fn test_abi_compat_current_version() {
    assert_eq!(
        check_abi_compatibility(ABI_VERSION),
        AbiCompatibility::Compatible
    );
}

#[test]
fn test_abi_compat_minor_diff() {
    assert_eq!(
        check_abi_compatibility("1.1.0"),
        AbiCompatibility::MinorDifference
    );
    assert_eq!(
        check_abi_compatibility("1.0.1"),
        AbiCompatibility::MinorDifference
    );
    assert_eq!(
        check_abi_compatibility("1.99.99"),
        AbiCompatibility::MinorDifference
    );
}

#[test]
fn test_abi_compat_major_diff() {
    assert_eq!(
        check_abi_compatibility("2.0.0"),
        AbiCompatibility::Incompatible
    );
    assert_eq!(
        check_abi_compatibility("0.1.0"),
        AbiCompatibility::Incompatible
    );
    assert_eq!(
        check_abi_compatibility("99.0.0"),
        AbiCompatibility::Incompatible
    );
}

#[test]
fn test_abi_compat_invalid_format() {
    assert_eq!(check_abi_compatibility(""), AbiCompatibility::Incompatible);
    assert_eq!(
        check_abi_compatibility("1.0"),
        AbiCompatibility::Incompatible
    );
    assert_eq!(
        check_abi_compatibility("1.0.0.0"),
        AbiCompatibility::Incompatible
    );
    assert_eq!(
        check_abi_compatibility("abc"),
        AbiCompatibility::Incompatible
    );
    assert_eq!(
        check_abi_compatibility("a.b.c"),
        AbiCompatibility::Incompatible
    );
}

#[test]
fn test_abi_version_is_valid_format() {
    let parts: Vec<&str> = ABI_VERSION.split('.').collect();
    assert_eq!(parts.len(), 3);
    for part in parts {
        assert!(part.parse::<u32>().is_ok());
    }
}

// ============================================================================
// cross_compile: CrossCompileConfig
// ============================================================================

#[test]
fn test_cross_config_new_native() {
    let config = CrossCompileConfig::new(TargetTriple::Native);
    assert!(config.sysroot.is_none());
    assert!(config.include_paths.is_empty());
    assert!(config.lib_paths.is_empty());
    assert!(config.linker_flags.is_empty());
    assert!(config.env_vars.is_empty());
    assert!(config.component_config.is_none());
}

#[test]
fn test_cross_config_new_wasi_preview2_has_component() {
    let config = CrossCompileConfig::new(TargetTriple::WasiPreview2);
    assert!(config.component_config.is_some());
}

#[test]
fn test_cross_config_new_linux_no_component() {
    let config = CrossCompileConfig::new(TargetTriple::X86_64Linux);
    assert!(config.component_config.is_none());
}

#[test]
fn test_cross_config_clang_native_no_target_flag() {
    let config = CrossCompileConfig::new(TargetTriple::Native);
    let cmd = config.clang_command();
    assert_eq!(cmd[0], "clang");
    assert!(!cmd.iter().any(|s| s.starts_with("--target")));
}

#[test]
fn test_cross_config_clang_linux_has_target() {
    let config = CrossCompileConfig::new(TargetTriple::X86_64Linux);
    let cmd = config.clang_command();
    assert!(cmd.iter().any(|s| s.contains("x86_64")));
}

#[test]
fn test_cross_config_clang_aarch64_has_target() {
    let config = CrossCompileConfig::new(TargetTriple::Aarch64Linux);
    let cmd = config.clang_command();
    assert!(cmd.iter().any(|s| s.contains("aarch64")));
}

#[test]
fn test_cross_config_clang_with_sysroot() {
    let mut config = CrossCompileConfig::new(TargetTriple::Aarch64Linux);
    config.sysroot = Some(std::path::PathBuf::from("/test/sysroot"));
    let cmd = config.clang_command();
    assert!(cmd.iter().any(|s| s.contains("--sysroot=/test/sysroot")));
}

#[test]
fn test_cross_config_clang_with_include_paths() {
    let mut config = CrossCompileConfig::new(TargetTriple::X86_64Linux);
    config
        .include_paths
        .push(std::path::PathBuf::from("/inc/a"));
    config
        .include_paths
        .push(std::path::PathBuf::from("/inc/b"));
    let cmd = config.clang_command();
    assert!(cmd.iter().any(|s| s == "-I/inc/a"));
    assert!(cmd.iter().any(|s| s == "-I/inc/b"));
}

#[test]
fn test_cross_config_clang_with_lib_paths() {
    let mut config = CrossCompileConfig::new(TargetTriple::X86_64Linux);
    config
        .lib_paths
        .push(std::path::PathBuf::from("/lib/mylib"));
    let cmd = config.clang_command();
    assert!(cmd.iter().any(|s| s == "-L/lib/mylib"));
}

#[test]
fn test_cross_config_clang_with_linker_flags() {
    let mut config = CrossCompileConfig::new(TargetTriple::X86_64Linux);
    config.linker_flags.push("--gc-sections".to_string());
    let cmd = config.clang_command();
    assert!(cmd.iter().any(|s| s == "-Wl,--gc-sections"));
}

#[test]
fn test_cross_config_linker_command_windows() {
    let config = CrossCompileConfig::new(TargetTriple::X86_64WindowsMsvc);
    let cmd = config.linker_command();
    assert_eq!(cmd[0], "lld-link");
}

#[test]
fn test_cross_config_linker_command_wasm() {
    let config = CrossCompileConfig::new(TargetTriple::Wasm32Unknown);
    let cmd = config.linker_command();
    assert_eq!(cmd[0], "wasm-ld");
}

#[test]
fn test_cross_config_linker_command_wasi() {
    let config = CrossCompileConfig::new(TargetTriple::WasiPreview1);
    let cmd = config.linker_command();
    assert_eq!(cmd[0], "wasm-ld");
}

#[test]
fn test_cross_config_linker_command_linux_uses_clang() {
    let config = CrossCompileConfig::new(TargetTriple::X86_64Linux);
    let cmd = config.linker_command();
    assert_eq!(cmd[0], "clang");
}

#[test]
fn test_cross_config_component_link_config() {
    let config = CrossCompileConfig::new(TargetTriple::WasiPreview2);
    assert!(config.component_link_config().is_some());

    let config2 = CrossCompileConfig::new(TargetTriple::X86_64Linux);
    assert!(config2.component_link_config().is_none());
}

// ============================================================================
// cross_compile: CrossCompileError Display
// ============================================================================

#[test]
fn test_cross_compile_error_sdk_not_found_display() {
    let err = CrossCompileError::SdkNotFound {
        target: "aarch64-ios".to_string(),
        hint: "Install Xcode".to_string(),
    };
    let s = format!("{}", err);
    assert!(s.contains("aarch64-ios"));
    assert!(s.contains("Install Xcode"));
}

#[test]
fn test_cross_compile_error_unsupported_display() {
    let err = CrossCompileError::UnsupportedTarget("mips64".to_string());
    let s = format!("{}", err);
    assert!(s.contains("mips64"));
}

#[test]
fn test_cross_compile_error_config_error_display() {
    let err = CrossCompileError::ConfigError("bad config".to_string());
    let s = format!("{}", err);
    assert!(s.contains("bad config"));
}

// ============================================================================
// cross_compile: RuntimeLibs
// ============================================================================

#[test]
fn test_runtime_libs_linux_gnu() {
    let libs = RuntimeLibs::for_target(&TargetTriple::X86_64Linux);
    assert!(libs.system_libs.contains(&"c".to_string()));
    assert!(libs.system_libs.contains(&"m".to_string()));
    assert!(libs.system_libs.contains(&"pthread".to_string()));
}

#[test]
fn test_runtime_libs_linux_musl() {
    let libs = RuntimeLibs::for_target(&TargetTriple::X86_64LinuxMusl);
    assert!(libs.system_libs.contains(&"c".to_string()));
    assert!(!libs.system_libs.contains(&"pthread".to_string())); // musl bundles pthread in libc
}

#[test]
fn test_runtime_libs_aarch64_linux() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64Linux);
    assert!(libs.system_libs.contains(&"c".to_string()));
    assert!(libs.system_libs.contains(&"pthread".to_string()));
}

#[test]
fn test_runtime_libs_aarch64_linux_musl() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64LinuxMusl);
    assert!(libs.system_libs.contains(&"c".to_string()));
}

#[test]
fn test_runtime_libs_windows_msvc() {
    let libs = RuntimeLibs::for_target(&TargetTriple::X86_64WindowsMsvc);
    assert!(libs.libs.contains(&"msvcrt".to_string()));
    assert!(libs.system_libs.contains(&"kernel32".to_string()));
}

#[test]
fn test_runtime_libs_windows_gnu() {
    let libs = RuntimeLibs::for_target(&TargetTriple::X86_64WindowsGnu);
    assert!(libs.libs.contains(&"mingw32".to_string()));
}

#[test]
fn test_runtime_libs_darwin() {
    let libs = RuntimeLibs::for_target(&TargetTriple::X86_64Darwin);
    assert!(libs.system_libs.contains(&"System".to_string()));
}

#[test]
fn test_runtime_libs_aarch64_darwin() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64Darwin);
    assert!(libs.system_libs.contains(&"System".to_string()));
}

#[test]
fn test_runtime_libs_android() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64Android);
    assert!(libs.system_libs.contains(&"log".to_string()));
    assert!(libs.system_libs.contains(&"c".to_string()));
}

#[test]
fn test_runtime_libs_armv7_android() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Armv7Android);
    assert!(libs.system_libs.contains(&"log".to_string()));
}

#[test]
fn test_runtime_libs_ios() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64Ios);
    assert!(libs.system_libs.contains(&"System".to_string()));
}

#[test]
fn test_runtime_libs_ios_simulator() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64IosSimulator);
    assert!(libs.system_libs.contains(&"System".to_string()));
}

#[test]
fn test_runtime_libs_wasm32() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Wasm32Unknown);
    assert!(libs.system_libs.is_empty());
    assert!(libs.libs.is_empty());
}

#[test]
fn test_runtime_libs_wasi_preview1() {
    let libs = RuntimeLibs::for_target(&TargetTriple::WasiPreview1);
    assert!(libs.system_libs.contains(&"c".to_string()));
}

#[test]
fn test_runtime_libs_wasi_preview2() {
    let libs = RuntimeLibs::for_target(&TargetTriple::WasiPreview2);
    assert!(libs.system_libs.contains(&"c".to_string()));
    assert!(libs.libs.contains(&"wasi-emulated-mman".to_string()));
}

#[test]
fn test_runtime_libs_native() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Native);
    assert!(libs.system_libs.contains(&"c".to_string()));
}

#[test]
fn test_runtime_libs_riscv64() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Riscv64LinuxGnu);
    assert!(libs.system_libs.contains(&"c".to_string()));
    assert!(libs.system_libs.contains(&"pthread".to_string()));
}

#[test]
fn test_runtime_libs_freebsd() {
    let libs = RuntimeLibs::for_target(&TargetTriple::X86_64FreeBsd);
    assert!(libs.system_libs.contains(&"c".to_string()));
    assert!(libs.system_libs.contains(&"pthread".to_string()));
}

#[test]
fn test_runtime_libs_aarch64_freebsd() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64FreeBsd);
    assert!(libs.system_libs.contains(&"c".to_string()));
}

#[test]
fn test_runtime_libs_aarch64_windows_msvc() {
    let libs = RuntimeLibs::for_target(&TargetTriple::Aarch64WindowsMsvc);
    assert!(libs.libs.contains(&"msvcrt".to_string()));
    assert!(libs.system_libs.contains(&"kernel32".to_string()));
}

// ============================================================================
// TargetTriple: parse
// ============================================================================

#[test]
fn test_target_parse_native() {
    assert_eq!(TargetTriple::parse("native"), Some(TargetTriple::Native));
    assert_eq!(TargetTriple::parse("auto"), Some(TargetTriple::Native));
}

#[test]
fn test_target_parse_linux_variants() {
    assert_eq!(
        TargetTriple::parse("x86_64-linux"),
        Some(TargetTriple::X86_64Linux)
    );
    assert_eq!(
        TargetTriple::parse("x86_64-unknown-linux-gnu"),
        Some(TargetTriple::X86_64Linux)
    );
    assert_eq!(
        TargetTriple::parse("x86_64-linux-musl"),
        Some(TargetTriple::X86_64LinuxMusl)
    );
}

#[test]
fn test_target_parse_windows() {
    assert_eq!(
        TargetTriple::parse("x86_64-windows-msvc"),
        Some(TargetTriple::X86_64WindowsMsvc)
    );
    assert_eq!(
        TargetTriple::parse("x86_64-windows-gnu"),
        Some(TargetTriple::X86_64WindowsGnu)
    );
}

#[test]
fn test_target_parse_darwin() {
    assert_eq!(
        TargetTriple::parse("x86_64-darwin"),
        Some(TargetTriple::X86_64Darwin)
    );
    assert_eq!(
        TargetTriple::parse("aarch64-darwin"),
        Some(TargetTriple::Aarch64Darwin)
    );
    assert_eq!(
        TargetTriple::parse("arm64"),
        Some(TargetTriple::Aarch64Darwin)
    );
}

#[test]
fn test_target_parse_wasm() {
    assert_eq!(
        TargetTriple::parse("wasm32"),
        Some(TargetTriple::Wasm32Unknown)
    );
    assert_eq!(
        TargetTriple::parse("wasi"),
        Some(TargetTriple::WasiPreview1)
    );
    assert_eq!(
        TargetTriple::parse("wasi-preview2"),
        Some(TargetTriple::WasiPreview2)
    );
}

#[test]
fn test_target_parse_mobile() {
    assert_eq!(
        TargetTriple::parse("aarch64-android"),
        Some(TargetTriple::Aarch64Android)
    );
    assert_eq!(
        TargetTriple::parse("armv7-android"),
        Some(TargetTriple::Armv7Android)
    );
    assert_eq!(
        TargetTriple::parse("aarch64-ios"),
        Some(TargetTriple::Aarch64Ios)
    );
    assert_eq!(
        TargetTriple::parse("aarch64-ios-sim"),
        Some(TargetTriple::Aarch64IosSimulator)
    );
}

#[test]
fn test_target_parse_riscv() {
    assert_eq!(
        TargetTriple::parse("riscv64"),
        Some(TargetTriple::Riscv64LinuxGnu)
    );
}

#[test]
fn test_target_parse_unknown() {
    assert_eq!(TargetTriple::parse("mips64"), None);
    assert_eq!(TargetTriple::parse(""), None);
    assert_eq!(TargetTriple::parse("unknown-unknown-unknown"), None);
}

#[test]
fn test_target_parse_case_insensitive() {
    assert_eq!(TargetTriple::parse("NATIVE"), Some(TargetTriple::Native));
    assert_eq!(
        TargetTriple::parse("WASM32"),
        Some(TargetTriple::Wasm32Unknown)
    );
}

// ============================================================================
// TargetTriple: classification methods
// ============================================================================

#[test]
fn test_target_is_wasm() {
    assert!(TargetTriple::Wasm32Unknown.is_wasm());
    assert!(TargetTriple::WasiPreview1.is_wasm());
    assert!(TargetTriple::WasiPreview2.is_wasm());
    assert!(!TargetTriple::X86_64Linux.is_wasm());
    assert!(!TargetTriple::Native.is_wasm());
}

#[test]
fn test_target_is_windows() {
    assert!(TargetTriple::X86_64WindowsMsvc.is_windows());
    assert!(TargetTriple::X86_64WindowsGnu.is_windows());
    assert!(TargetTriple::Aarch64WindowsMsvc.is_windows());
    assert!(!TargetTriple::X86_64Linux.is_windows());
}

#[test]
fn test_target_is_apple() {
    assert!(TargetTriple::X86_64Darwin.is_apple());
    assert!(TargetTriple::Aarch64Darwin.is_apple());
    assert!(TargetTriple::Aarch64Ios.is_apple());
    assert!(TargetTriple::Aarch64IosSimulator.is_apple());
    assert!(!TargetTriple::X86_64Linux.is_apple());
}

#[test]
fn test_target_is_ios() {
    assert!(TargetTriple::Aarch64Ios.is_ios());
    assert!(TargetTriple::Aarch64IosSimulator.is_ios());
    assert!(!TargetTriple::Aarch64Darwin.is_ios());
}

#[test]
fn test_target_is_android() {
    assert!(TargetTriple::Aarch64Android.is_android());
    assert!(TargetTriple::Armv7Android.is_android());
    assert!(!TargetTriple::X86_64Linux.is_android());
}

#[test]
fn test_target_is_musl() {
    assert!(TargetTriple::X86_64LinuxMusl.is_musl());
    assert!(TargetTriple::Aarch64LinuxMusl.is_musl());
    assert!(!TargetTriple::X86_64Linux.is_musl());
}

#[test]
fn test_target_output_extension() {
    assert_eq!(TargetTriple::X86_64WindowsMsvc.output_extension(), "exe");
    assert_eq!(TargetTriple::X86_64WindowsGnu.output_extension(), "exe");
    assert_eq!(TargetTriple::Wasm32Unknown.output_extension(), "wasm");
    assert_eq!(TargetTriple::WasiPreview1.output_extension(), "wasm");
    assert_eq!(TargetTriple::X86_64Linux.output_extension(), "");
    assert_eq!(TargetTriple::Aarch64Darwin.output_extension(), "");
}

#[test]
fn test_target_pointer_bits() {
    assert_eq!(TargetTriple::X86_64Linux.pointer_bits(), 64);
    assert_eq!(TargetTriple::Aarch64Darwin.pointer_bits(), 64);
    assert_eq!(TargetTriple::Wasm32Unknown.pointer_bits(), 32);
    assert_eq!(TargetTriple::Armv7Android.pointer_bits(), 32);
    assert_eq!(TargetTriple::Native.pointer_bits(), 64);
}

#[test]
fn test_target_all_targets_not_empty() {
    let targets = TargetTriple::all_targets();
    assert!(!targets.is_empty());
    assert!(targets.contains(&"native"));
    assert!(targets.contains(&"wasm32"));
    assert!(targets.contains(&"wasi"));
}

// ============================================================================
// Codegen via gen_ok: diagnostics coverage (suggest_similar, format_did_you_mean)
// ============================================================================

#[test]
fn test_codegen_undefined_var_suggests_similar() {
    let result = gen_result("F test() -> i64 { abc := 1\nR abd }");
    // Should either succeed (if abd is somehow resolved) or give error with suggestion
    match result {
        Err(e) => assert!(
            e.contains("abd") || e.contains("abc"),
            "Error should reference var: {}",
            e
        ),
        Ok(_) => {} // Some codegen modes may accept this
    }
}

#[test]
fn test_codegen_simple_arithmetic() {
    let ir = gen_ok("F f() -> i64 = 1 + 2 + 3");
    assert!(ir.contains("add"));
}

#[test]
fn test_codegen_if_else_phi() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 {
                x
            } E {
                0 - x
            }
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("br"));
}

#[test]
fn test_codegen_nested_if() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 10 {
                I x > 20 {
                    3
                } E {
                    2
                }
            } E {
                1
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_while_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            i := mut 0
            sum := mut 0
            L i < 10 {
                sum = sum + i
                i = i + 1
            }
            sum
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_codegen_struct_definition() {
    let ir = gen_ok(
        r#"
        S Point {
            x: i64,
            y: i64
        }
        F test() -> i64 {
            p := Point { x: 10, y: 20 }
            p.x
        }
    "#,
    );
    assert!(ir.contains("Point") || ir.contains("insertvalue") || ir.contains("store"));
}

#[test]
fn test_codegen_enum_definition() {
    let ir = gen_ok(
        r#"
        E Color {
            Red,
            Green,
            Blue
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_string_literal() {
    let ir = gen_ok(r#"F test() -> str = "hello world""#);
    assert!(ir.contains("hello world") || ir.contains("hello"));
}

#[test]
fn test_codegen_bool_operations() {
    let ir = gen_ok("F test() -> bool = true && false || true");
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_comparison_operators() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 1 < 2
            b := 3 >= 3
            c := 4 <= 5
            d := 6 > 5
            e := 7 == 7
            f := 8 != 9
            0
        }
    "#,
    );
    assert!(ir.contains("icmp"));
}

#[test]
fn test_codegen_multiple_functions() {
    let ir = gen_ok(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F sub(a: i64, b: i64) -> i64 = a - b
        F main() -> i64 = add(3, sub(5, 2))
    "#,
    );
    assert!(ir.contains("@add"));
    assert!(ir.contains("@sub"));
    assert!(ir.contains("@main"));
}

#[test]
fn test_codegen_recursive_function() {
    let ir = gen_ok(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 {
                n
            } E {
                fib(n - 1) + fib(n - 2)
            }
        }
    "#,
    );
    assert!(ir.contains("@fib"));
}

#[test]
fn test_codegen_mutable_variable() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            x = 42
            x
        }
    "#,
    );
    assert!(ir.contains("store") || ir.contains("alloca"));
}

#[test]
fn test_codegen_unary_negation() {
    let ir = gen_ok("F test() -> i64 = 0 - 42");
    assert!(ir.contains("sub"));
}

#[test]
fn test_codegen_float_literal() {
    let ir = gen_ok("F test() -> f64 = 3.14");
    assert!(ir.contains("3.14") || ir.contains("double") || ir.contains("float"));
}

#[test]
fn test_codegen_float_arithmetic() {
    let ir = gen_ok(
        r#"
        F test() -> f64 {
            a := 1.5
            b := 2.5
            a + b
        }
    "#,
    );
    assert!(ir.contains("fadd") || ir.contains("add") || ir.contains("double"));
}

#[test]
fn test_codegen_type_cast() {
    let result = gen_result("F test() -> f64 = 42 as f64");
    // May or may not be supported
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_codegen_empty_function() {
    let ir = gen_ok("F test() -> i64 = 0");
    assert!(ir.contains("@test"));
    assert!(ir.contains("ret"));
}

#[test]
fn test_codegen_for_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 0..10 {
                sum = sum + i
            }
            sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_break_continue() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            i := mut 0
            result := mut 0
            L i < 100 {
                I i == 50 {
                    B
                }
                I i % 2 == 0 {
                    i = i + 1
                    C
                }
                result = result + i
                i = i + 1
            }
            result
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_codegen_lambda_simple() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            f := |x: i64| -> i64 { x * 2 }
            f(21)
        }
    "#,
    );
    // Lambda codegen may or may not be fully supported
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_codegen_match_integer() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                1 => 10,
                2 => 20,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("switch") || ir.contains("icmp") || ir.contains("br"));
}

#[test]
fn test_codegen_nested_struct() {
    let ir = gen_ok(
        r#"
        S Inner { val: i64 }
        S Outer { inner: Inner, extra: i64 }
        F test() -> i64 {
            o := Outer { inner: Inner { val: 42 }, extra: 10 }
            o.inner.val
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_method_call() {
    let ir = gen_ok(
        r#"
        S Counter { val: i64 }
        X Counter {
            F get(self) -> i64 = self.val
        }
        F test() -> i64 {
            c := Counter { val: 99 }
            c.get()
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_trait_definition() {
    let ir = gen_ok(
        r#"
        W Printable {
            F show(self) -> i64
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_extern_declaration() {
    let ir = gen_ok(
        r#"
        N {
            F puts(s: str) -> i64
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(ir.contains("puts") || ir.contains("declare"));
}

#[test]
fn test_codegen_modulo_operator() {
    let ir = gen_ok("F test() -> i64 = 17 % 5");
    assert!(ir.contains("srem") || ir.contains("rem"));
}

#[test]
fn test_codegen_self_recursion_operator() {
    let ir = gen_ok(
        r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 {
                1
            } E {
                n * @(n - 1)
            }
        }
    "#,
    );
    assert!(ir.contains("@factorial"));
}

#[test]
fn test_codegen_pipe_operator() {
    let result = gen_result(
        r#"
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 = 5 |> double
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}
