//! Phase 9 (ROADMAP #9): Runtime intrinsic duplicate `declare`/`define` guard
//!
//! Before ROADMAP #9, a program that re-declared a runtime intrinsic via
//! `X F __store_byte(ptr: i64, value: i64) -> i64` (which std/http.vais and
//! friends do) would produce both:
//!   - `declare void @__store_byte(i64, i64)` — from the extern registration
//!   - `define void @__store_byte(i64 %ptr, i64 %val) { … }` — from
//!     `generate_helper_functions`
//! in the **same** main module. LLVM rejects that as a symbol redefinition,
//! which is why `vais-apps/signature/build.sh` had a python3 post-processing
//! step to strip the redundant declares.
//!
//! This test pins the fix: when the main module emits the helper function
//! body, the corresponding `declare` must be suppressed so the IR contains
//! exactly one definition and zero declarations for each runtime intrinsic.
//!
//! The related Phase 158 CI gate in `phase158_type_strict.rs` guards the
//! `VAIS_TC_NONFATAL` removal; the two gates are independent.

use crate::helpers::compile_to_ir;

/// Count how many lines in `ir` look like `{prefix} ... @{name}(...)`.
/// Matches the shape of LLVM IR function declarations/definitions:
///   - `declare i64 @strlen(i64)` — prefix "declare", name "strlen"
///   - `define void @__store_byte(i64 %ptr, i64 %val) {` — prefix "define",
///     name "__store_byte"
/// The intervening `{ret_type}` is skipped by matching the prefix at the
/// start of the trimmed line and `@{name}(` anywhere after.
fn count_prefix_at(ir: &str, prefix: &str, name: &str) -> usize {
    let at_name = format!("@{}(", name);
    ir.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with(&format!("{} ", prefix)) && trimmed.contains(&at_name)
        })
        .count()
}

/// Program that actively calls the runtime intrinsics we care about. Because
/// the std modules re-declare these as `X F __store_byte(...)`, the extern
/// registration path is triggered even for a minimal user program.
fn intrinsic_exerciser_source() -> &'static str {
    r#"
N F __store_byte(ptr: i64, value: i64) -> i64
N F __load_byte(ptr: i64) -> i64

F main() -> i64 {
    buf := 0  # would be a heap alloc in real code; we just need the symbol reference
    R buf
}
"#
}

#[test]
fn e2e_phase9_no_duplicate_store_byte() {
    let ir = compile_to_ir(intrinsic_exerciser_source())
        .expect("ROADMAP #9 fixture should compile to IR");
    let defines = count_prefix_at(&ir, "define", "__store_byte");
    let declares = count_prefix_at(&ir, "declare", "__store_byte");
    // Post-fix invariant: `generate_helper_functions()` emits the body in the
    // main module, and the extern declaration for `__store_byte` (triggered by
    // the user's `N F __store_byte(...)` declaration) must be suppressed by
    // `is_runtime_intrinsic` in the extern-decl loop.
    assert_eq!(
        defines, 1,
        "Expected exactly one `define @__store_byte` in main-module IR, got {}",
        defines
    );
    assert_eq!(
        declares, 0,
        "Expected zero `declare @__store_byte` — helper function define is emitted in the same module, so the declare is redundant (ROADMAP #9). Got {}",
        declares
    );
}

#[test]
fn e2e_phase9_no_duplicate_load_byte() {
    let ir = compile_to_ir(intrinsic_exerciser_source())
        .expect("ROADMAP #9 fixture should compile to IR");
    let defines = count_prefix_at(&ir, "define", "__load_byte");
    let declares = count_prefix_at(&ir, "declare", "__load_byte");
    assert_eq!(
        defines, 1,
        "Expected exactly one `define @__load_byte` in main-module IR, got {}",
        defines
    );
    assert_eq!(
        declares, 0,
        "Expected zero `declare @__load_byte` (ROADMAP #9), got {}",
        declares
    );
}

/// Genuine extern C functions (not runtime intrinsics) must still receive a
/// `declare` line. This guards against the fix over-filtering.
#[test]
fn e2e_phase9_real_externs_still_declared() {
    let source = r#"
N F strlen(s: i64) -> i64

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_ir(source).expect("extern C declaration should compile");
    let declares = count_prefix_at(&ir, "declare", "strlen");
    assert!(
        declares >= 1,
        "Expected at least one `declare @strlen(...)` line, got {}\n--- IR head ---\n{}",
        declares,
        &ir.chars().take(800).collect::<String>()
    );
}
