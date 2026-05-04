use std::fs;
use std::path::{Path, PathBuf};

use vais_mir::interpreter::{
    interpret_function, interpret_function_with_io, InterpreterRunOutput, MirValue,
};
use vais_mir::lower::lower_module_checked;
use vais_mir::validate::validate_module;

#[test]
fn interpreter_runs_arithmetic_return() {
    let value = interpret_source("F main() -> i64 = 40 + 2", "main");
    assert_eq!(value, MirValue::Int(42));
}

#[test]
fn interpreter_runs_direct_call_and_branch() {
    let source = r#"
        F add(a: i64, b: i64) -> i64 {
            a + b
        }

        F main() -> i64 {
            x: i64 := add(20, 22)
            I x != 42 { R 1 }
            R 0
        }
    "#;
    let value = interpret_source(source, "main");
    assert_eq!(value, MirValue::Int(0));
}

#[test]
fn interpreter_runs_strict_core_fixture_subset() {
    for (path, expected) in [
        (
            "tests/core/positive/functions/basic_return.vais",
            MirValue::Int(42),
        ),
        (
            "tests/core/positive/functions/call_and_block.vais",
            MirValue::Int(0),
        ),
        (
            "tests/core/positive/primitives/int_bool_string.vais",
            MirValue::Int(0),
        ),
        (
            "tests/core/positive/control/if_else_while.vais",
            MirValue::Int(0),
        ),
        ("tests/core/positive/structs/point.vais", MirValue::Int(0)),
        (
            "tests/core/positive/enums/color_match.vais",
            MirValue::Int(0),
        ),
        (
            "tests/core/positive/enums/option_match.vais",
            MirValue::Int(0),
        ),
        (
            "tests/core/positive/collections/vec_i64.vais",
            MirValue::Int(0),
        ),
    ] {
        let source_path = compiler_root().join(path);
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", source_path.display(), e));
        let value = interpret_source(&source, "main");
        assert_eq!(value, expected, "unexpected interpreter result for {path}");
    }
}

fn interpret_source(source: &str, function: &str) -> MirValue {
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering failed");
    validate_module(&mir).expect("MIR validation failed");
    interpret_function(&mir, function, vec![]).expect("MIR interpretation failed")
}

fn compiler_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to resolve compiler root")
}

// ── Step 17 stage 5a: stdout-capturing entry point tests ────────────────────

/// `interpret_function_with_io` returns exit_code=42 for `R 42`.
#[test]
fn interpret_with_io_int_return_maps_to_exit_code() {
    let source = "F main() -> i64 = 42";
    let module = vais_parser::parse(source).expect("parse");
    let mir = lower_module_checked(&module).expect("lower");
    validate_module(&mir).expect("validate");
    let out: InterpreterRunOutput =
        interpret_function_with_io(&mir, "main", vec![]).expect("interpret");
    assert_eq!(out.exit_code, 42);
    assert_eq!(out.stdout, "");
    assert_eq!(out.return_value, MirValue::Int(42));
}

/// 8-bit truncation matches POSIX exit semantics.
#[test]
fn interpret_with_io_truncates_exit_code_to_8_bits() {
    let source = "F main() -> i64 = 257";
    let module = vais_parser::parse(source).expect("parse");
    let mir = lower_module_checked(&module).expect("lower");
    validate_module(&mir).expect("validate");
    let out = interpret_function_with_io(&mir, "main", vec![]).expect("interpret");
    assert_eq!(out.exit_code, 1, "257 & 0xFF = 1");
}

/// Backward compatibility: `interpret_function` still rejects calls to
/// nonexistent function names. The print builtin intercept fires ONLY on
/// the `_with_io` entry point (which sets the stdout sink); the bare
/// entry leaves the sink None so try_intercept_builtin returns None.
///
/// We exercise this via a direct call to a fake function name (no MIR
/// body) — the bare entry must error.
#[test]
fn bare_interpret_function_rejects_unknown_function_name() {
    let source = "F main() -> i64 = 0";
    let module = vais_parser::parse(source).expect("parse");
    let mir = lower_module_checked(&module).expect("lower");
    validate_module(&mir).expect("validate");
    // Call a function name that has no body (and is not `main`).
    let result = interpret_function(&mir, "definitely_not_a_function", vec![]);
    assert!(
        result.is_err(),
        "bare interpret_function must error on unknown function; got {:?}",
        result
    );
}
