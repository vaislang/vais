//! Differential fuzz target: MIR interpreter vs native (LLVM + clang) execution.
//!
//! # Stage 0 — Scaffold
//!
//! This target establishes the scaffold for differential testing between the
//! Vais MIR reference interpreter and the native LLVM/clang backend. The full
//! diff comparison logic is in place; two execution paths are gated behind
//! explicit `unimplemented!` markers that document the remaining gaps for stage 1.
//!
//! ## Execution model
//!
//! ```text
//! fuzz input (VaisProgram)
//!     │
//!     ├─ parse + type-check
//!     │       │
//!     │       ├─ [Path A] lower to MIR → interpret "main()"
//!     │       │          returns MirValue or short-circuits on error
//!     │       │
//!     │       └─ [Path B] codegen LLVM IR → tempfile → clang → exec
//!     │                  returns (exit_code, stdout) or short-circuits
//!     │
//!     └─ compare: if both succeed AND differ → panic (libFuzzer finding)
//! ```
//!
//! ## Stage 0 gaps (to be filled in stage 1+)
//!
//! ### Gap A — MIR interpreter "main" entry point
//!
//! The MIR interpreter (`vais_mir::interpreter::interpret_function`) can call
//! any named function inside a `MirModule` and returns a `MirValue`.  It does
//! NOT produce an exit-code or capture stdout.  Bridging this to the
//! (exit_code, stdout) model used by the native path requires:
//!   1. The lowered MIR to contain a `main` body (function named `"main"`).
//!   2. A thin shim that maps `MirValue::Int(n)` → exit-code `n` and
//!      captures any MIR-level I/O side-effects into a fake stdout buffer.
//!      (Currently the interpreter has no I/O model.)
//!
//! Until that shim exists, Path A is an `unimplemented!("stage 1+")` stub.
//!
//! ### Gap B — native execution (clang + tempfile + process::Command)
//!
//! The native path must:
//!   1. Write the LLVM IR text to a temp file (e.g. via `tempfile` crate).
//!   2. Shell out to `clang` to compile to a binary.
//!   3. Run the binary and capture (exit_code, stdout).
//!
//! In a `cargo-fuzz` / libFuzzer environment this is intentionally avoided
//! because spawning external processes inside a fuzzer loop breaks the
//! libFuzzer fork-server model and introduces enormous latency.  The
//! recommended stage 1 approach is to use a dedicated corpus-replay harness
//! (not libFuzzer) for the native path, or to restrict the native path to a
//! JIT backend (vais-jit / Cranelift) that runs in-process.
//!
//! Until an in-process execution model is available, Path B is an
//! `unimplemented!("stage 1+")` stub.
//!
//! ## How to advance to stage 1
//!
//! 1. Implement `mir_run_main(mir_module) -> Result<RunOutput, MirRunError>`
//!    in `vais-mir/src/interpreter.rs` (or a new `runner.rs`).  The function
//!    must produce `RunOutput { exit_code: i64, stdout: String }`.
//! 2. Implement `native_run(ir_text, source_name) -> Result<RunOutput, NativeRunError>`
//!    using `vais-jit` (in-process Cranelift) to avoid the external-process
//!    problem described above.
//! 3. Replace both `unimplemented!` blocks below with calls to those functions.
//! 4. Remove the `#[allow(unreachable_code)]` attributes from the diff function.

//! Vais fuzz core library.  Houses the structured-input types and the
//! `compare_paths` differential check used by `fuzz_targets/
//! fuzz_mir_native_diff.rs`.  Splitting the logic into a library crate
//! lets `cargo test` reach `#[test]` functions defined here, which is
//! impossible from inside the libFuzzer-style binary (per
//! STEP17_FINDINGS F-MIR-02).

use arbitrary::{Arbitrary, Unstructured};

// ─── Structured input (mirrored from fuzz_full_pipeline) ────────────────────

/// Structured input for more intelligent fuzzing.
/// Identical to the `VaisProgram` type in `fuzz_full_pipeline.rs`; inlined here
/// so this target is self-contained.
#[derive(Debug, Arbitrary)]
pub struct VaisProgram {
    items: Vec<VaisItem>,
}

#[derive(Debug, Arbitrary)]
pub enum VaisItem {
    Function(FunctionDef),
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug, Arbitrary)]
pub struct FunctionDef {
    name: SmallString,
    params: Vec<(SmallString, VaisType)>,
    ret_type: VaisType,
    body: VaisExpr,
}

#[derive(Debug, Arbitrary)]
pub struct StructDef {
    name: SmallString,
    fields: Vec<(SmallString, VaisType)>,
}

#[derive(Debug, Arbitrary)]
pub struct EnumDef {
    name: SmallString,
    variants: Vec<SmallString>,
}

#[derive(Debug, Arbitrary)]
pub enum VaisType {
    I64,
    F64,
    Bool,
    Str,
    Unit,
    Array(Box<VaisType>),
    Option(Box<VaisType>),
}

#[derive(Debug, Arbitrary)]
pub enum VaisExpr {
    Literal(i32),
    BoolLit(bool),
    Var(SmallString),
    BinOp(Box<VaisExpr>, BinOp, Box<VaisExpr>),
    If(Box<VaisExpr>, Box<VaisExpr>, Box<VaisExpr>),
    Block(Vec<VaisStmt>, Box<VaisExpr>),
}

#[derive(Debug, Arbitrary)]
pub enum VaisStmt {
    Let(SmallString, VaisExpr),
}

#[derive(Debug, Arbitrary)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Eq,
    And,
    Or,
}

/// Small string to avoid huge allocations.
#[derive(Debug)]
pub struct SmallString(String);

impl<'a> Arbitrary<'a> for SmallString {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(1..=15)?;
        let first = u.int_in_range(b'a'..=b'z')? as char;
        let rest: String = (0..len - 1)
            .map(|_| {
                let c = u.int_in_range(b'a'..=b'z').unwrap_or(b'a');
                c as char
            })
            .collect();
        Ok(SmallString(format!("{}{}", first, rest)))
    }
}

// ─── Source rendering (mirrored from fuzz_full_pipeline) ────────────────────

impl VaisProgram {
    pub fn to_source(&self) -> String {
        let mut source = String::new();
        for item in &self.items {
            source.push_str(&item.to_source());
            source.push('\n');
        }
        source
    }
}

impl VaisItem {
    pub fn to_source(&self) -> String {
        match self {
            VaisItem::Function(f) => f.to_source(),
            VaisItem::Struct(s) => s.to_source(),
            VaisItem::Enum(e) => e.to_source(),
        }
    }
}

impl FunctionDef {
    pub fn to_source(&self) -> String {
        let params: Vec<String> = self
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name.0, ty.to_source()))
            .collect();
        format!(
            "F {}({}) -> {} {{ {} }}",
            self.name.0,
            params.join(", "),
            self.ret_type.to_source(),
            self.body.to_source()
        )
    }
}

impl StructDef {
    pub fn to_source(&self) -> String {
        let fields: Vec<String> = self
            .fields
            .iter()
            .map(|(name, ty)| format!("{}: {}", name.0, ty.to_source()))
            .collect();
        format!("S {} {{ {} }}", self.name.0, fields.join(", "))
    }
}

impl EnumDef {
    pub fn to_source(&self) -> String {
        if self.variants.is_empty() {
            format!("E {} {{ Empty }}", self.name.0)
        } else {
            let variants: Vec<&str> = self.variants.iter().map(|v| v.0.as_str()).collect();
            format!("E {} {{ {} }}", self.name.0, variants.join(", "))
        }
    }
}

impl VaisType {
    pub fn to_source(&self) -> String {
        match self {
            VaisType::I64 => "i64".to_string(),
            VaisType::F64 => "f64".to_string(),
            VaisType::Bool => "bool".to_string(),
            VaisType::Str => "str".to_string(),
            VaisType::Unit => "()".to_string(),
            VaisType::Array(inner) => format!("[{}]", inner.to_source()),
            VaisType::Option(inner) => format!("Option<{}>", inner.to_source()),
        }
    }
}

impl VaisExpr {
    pub fn to_source(&self) -> String {
        match self {
            VaisExpr::Literal(n) => n.to_string(),
            VaisExpr::BoolLit(b) => b.to_string(),
            VaisExpr::Var(s) => s.0.clone(),
            VaisExpr::BinOp(l, op, r) => {
                format!("({} {} {})", l.to_source(), op.to_source(), r.to_source())
            }
            VaisExpr::If(cond, then, else_) => {
                format!(
                    "I {} {{ {} }} E {{ {} }}",
                    cond.to_source(),
                    then.to_source(),
                    else_.to_source()
                )
            }
            VaisExpr::Block(stmts, expr) => {
                let stmts_str: Vec<String> = stmts.iter().map(|s| s.to_source()).collect();
                if stmts_str.is_empty() {
                    expr.to_source()
                } else {
                    format!("{}\n{}", stmts_str.join("\n"), expr.to_source())
                }
            }
        }
    }
}

impl VaisStmt {
    pub fn to_source(&self) -> String {
        match self {
            VaisStmt::Let(name, expr) => format!("{} := {}", name.0, expr.to_source()),
        }
    }
}

impl BinOp {
    pub fn to_source(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Eq => "==",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

// ─── Differential comparison types ──────────────────────────────────────────

/// The normalized output of one execution path, used for diffing.
#[derive(Debug, PartialEq, Eq)]
pub struct RunOutput {
    /// Process exit code (or interpreter return value cast to i64).
    exit_code: i64,
    /// Captured stdout as a string.
    stdout: String,
}

/// Outcome from running one execution path.
// `Ok` is dead code at stage 0 (both execution paths are stubbed with
// `NotImplemented`).  It will become live once stage 1 wires real runners.
#[allow(dead_code)]
pub enum PathOutcome {
    /// The input was invalid for this path (parse/TC/lowering/codegen error).
    /// Not a finding — short-circuit OK.
    InputInvalid,
    /// Path succeeded and produced a result.
    Ok(RunOutput),
    /// This execution path is not yet implemented (stage 1+ gap).
    /// Not a finding — short-circuit OK.
    NotImplemented,
}

// ─── Path A: MIR interpreter ─────────────────────────────────────────────────

/// Attempt to run the program via the MIR reference interpreter.
///
/// # Stage 0 gap
///
/// The MIR interpreter (`vais_mir::interpreter::interpret_function`) calls a
/// named function and returns a `MirValue`, not a `(exit_code, stdout)` pair.
/// The interpreter has no I/O side-effect model.  A shim that:
///   - calls `interpret_function(module, "main", vec![])`,
///   - maps `MirValue::Int(n) -> exit_code = n`, and
///   - provides a fake stdout capture for print-like builtins
/// must be added in stage 1 before this path produces real diffs.
///
/// The `unimplemented!` is reached at runtime and causes `PathOutcome::NotImplemented`
/// to be returned, so the differential check short-circuits without a finding.
pub fn run_mir_path(source: &str) -> PathOutcome {
    use vais_mir::interpreter::{interpret_function, MirValue};
    use vais_mir::lower::lower_module_checked;
    use vais_parser::parse;
    use vais_types::TypeChecker;

    // Step 1: parse
    let module = match parse(source) {
        Ok(m) => m,
        Err(_) => return PathOutcome::InputInvalid,
    };

    // Step 2: type-check
    let mut checker = TypeChecker::new();
    if checker.check_module(&module).is_err() {
        return PathOutcome::InputInvalid;
    }

    // Step 3: lower to MIR (strict — reject semantic-loss placeholders)
    let mir_module = match lower_module_checked(&module) {
        Ok(m) => m,
        Err(_) => return PathOutcome::InputInvalid,
    };

    // Step 4: interpret "main"
    // STAGE 1: bridge MirValue → (exit_code, stdout). The interpreter has no
    // I/O model yet, so stdout is always empty; integer return is mapped to
    // the program's exit code (8-bit truncated to match POSIX exit semantics).
    // Stage 2+ extends this when the MIR interpreter gains print intrinsics.
    match interpret_function(&mir_module, "main", vec![]) {
        Ok(MirValue::Int(n)) => PathOutcome::Ok(RunOutput {
            // Truncate to 8-bit so this matches what the native side will
            // produce when its `exit(n)` reaches the OS.
            exit_code: n & 0xFF,
            stdout: String::new(),
        }),
        Ok(MirValue::Unit) => {
            // `main` returning Unit is treated like `main` returning 0.
            PathOutcome::Ok(RunOutput { exit_code: 0, stdout: String::new() })
        }
        Ok(_) => {
            // Non-integer / non-unit return — input is outside the
            // differential-test envelope (we cannot map it to an exit code).
            PathOutcome::InputInvalid
        }
        Err(_) => {
            // Interpreter errored (unsupported MIR construct, step limit, etc.)
            // — treat as "input not in scope for this oracle" rather than as
            // a finding. Stage 2+ may upgrade specific error classes (e.g.
            // div-by-zero) into hard findings.
            PathOutcome::InputInvalid
        }
    }
}

// ─── Path B: native execution (LLVM IR + clang) ─────────────────────────────

/// Attempt to run the program via the native LLVM/clang backend.
///
/// # Stage 0 gap
///
/// Running natively inside a libFuzzer loop requires either:
///   (a) an in-process JIT (vais-jit / Cranelift) — recommended for stage 1, or
///   (b) write LLVM IR to a tempfile, shell out to clang, run the binary.
///
/// Option (b) breaks the libFuzzer fork-server model and is prohibitively slow
/// (~seconds per iteration).  Option (a) requires wiring `vais-jit` into the
/// fuzz crate.  Until one of those is done, this path returns `NotImplemented`.
#[allow(unreachable_code, unused_variables)]
pub fn run_native_path(source: &str) -> PathOutcome {
    use vais_codegen::CodeGenerator;
    use vais_parser::parse;
    use vais_types::TypeChecker;

    // Step 1: parse
    let module = match parse(source) {
        Ok(m) => m,
        Err(_) => return PathOutcome::InputInvalid,
    };

    // Step 2: type-check
    let mut checker = TypeChecker::new();
    if checker.check_module(&module).is_err() {
        return PathOutcome::InputInvalid;
    }

    // Step 3: codegen LLVM IR (text backend)
    let mut gen = CodeGenerator::new("fuzz_mir_native_diff");
    let _ir = match gen.generate_module(&module) {
        Ok(ir) => ir,
        Err(_) => return PathOutcome::InputInvalid,
    };

    // STAGE 1 GAP: wire vais-jit (Cranelift in-process) or tempfile+clang.
    // Replace this unimplemented! once an in-process execution model is available.
    return PathOutcome::NotImplemented;

    #[allow(unreachable_code)]
    {
        // Placeholder — dead code showing the intended stage 1 call site:
        //
        //   let output = vais_jit::run_ir_text(&_ir)?;
        //   PathOutcome::Ok(RunOutput {
        //       exit_code: output.exit_code,
        //       stdout: output.stdout,
        //   })
        PathOutcome::NotImplemented
    }
}

// ─── Differential comparison ─────────────────────────────────────────────────

/// Compare MIR interpreter output against native execution output.
///
/// Returns `Ok(())` if:
/// - either path could not process the input (invalid Vais program), or
/// - either path is not yet implemented (stage 0 stubs), or
/// - both paths succeed and produce identical output.
///
/// Panics if both paths succeed and produce DIFFERENT output.  libFuzzer
/// records a panic as a crash finding.
pub fn compare_paths(source: &str) {
    let mir_result = run_mir_path(source);
    let native_result = run_native_path(source);

    match (mir_result, native_result) {
        // Both implemented and both succeeded — check for divergence.
        (PathOutcome::Ok(mir_out), PathOutcome::Ok(native_out)) => {
            if mir_out != native_out {
                panic!(
                    "MIR/native diff detected!\n\
                     source:\n{}\n\
                     MIR    exit={} stdout={:?}\n\
                     native exit={} stdout={:?}",
                    source,
                    mir_out.exit_code,
                    mir_out.stdout,
                    native_out.exit_code,
                    native_out.stdout,
                );
            }
        }
        // Any other combination: invalid input or not-yet-implemented path.
        // Not a finding.
        _ => {}
    }
}

// ─── libFuzzer entry point lives in fuzz_targets/fuzz_mir_native_diff.rs ────
// The thin binary calls `compare_paths` for each iteration. This library
// keeps the comparison logic and structured input types reachable from
// `cargo test --lib -p vais-fuzz`, addressing STEP17_FINDINGS F-MIR-02.


// ─── Verifying this scaffold ────────────────────────────────────────────────
//
// Because `fuzz_target!` expands to a libFuzzer-provided `main`, running
// `cargo test --bin fuzz_mir_native_diff` launches libFuzzer on the binary
// rather than executing `#[test]` functions. There is therefore no
// in-binary unit-test surface; the deterministic-input checks that lived
// here in earlier drafts are removed.
//
// To run a single deterministic input through `compare_paths`, use a
// short-lived corpus with cargo-fuzz:
//
//   cd compiler/fuzz
//   mkdir -p corpus/fuzz_mir_native_diff
//   echo 'F main() -> i64 { 42 }' > corpus/fuzz_mir_native_diff/seed
//   cargo fuzz run fuzz_mir_native_diff -- -runs=1
//
// Stage 1 status: Path A (MIR interpret) is wired to the real
// `interpret_function` and does map `MirValue::Int(n)` to an exit code.
// Path B (native LLVM/clang execution) is still `NotImplemented` — Stage 2+
// wires `vais-jit` (Cranelift in-process) so the differential check has
// two concrete sides to compare. Until then the runner short-circuits
// every iteration on the native side and no diff finding can land.

// ─── Unit tests (reachable under `cargo test --lib -p vais-fuzz`) ───────────
//
// These tests fix STEP17_FINDINGS F-MIR-02: the original fuzz binary's
// in-binary `#[test]` functions were unreachable because libFuzzer's main
// hijacks the cargo-test runner. Moving the logic into this library crate
// lets cargo test run normally.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_paths_simple_main_does_not_panic() {
        // Both paths short-circuit cleanly: Path A interprets and returns
        // an exit code; Path B is NotImplemented. Either way, no panic.
        compare_paths("F main() -> i64 { 0 }");
    }

    #[test]
    fn compare_paths_invalid_source_does_not_panic() {
        // Parser rejects → both paths return InputInvalid → no finding,
        // no panic.
        compare_paths("this is not valid vais");
    }

    #[test]
    fn compare_paths_empty_does_not_panic() {
        compare_paths("");
    }

    #[test]
    fn run_output_eq_basic() {
        let a = RunOutput { exit_code: 0, stdout: String::new() };
        let b = RunOutput { exit_code: 0, stdout: String::new() };
        let c = RunOutput { exit_code: 1, stdout: String::new() };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
