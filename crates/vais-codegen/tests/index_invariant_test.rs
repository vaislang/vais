//! index invariant verification (ADR 0001 §1 R2).
//!
//! Invariant (Phase Ω compound-assign fix):
//!     "For every emitted `getelementptr <elem-ty>, <elem-ty>* <base>, i64 <idx>`
//!      that originates from an indexed assignment (`arr[i] = v` or
//!      `arr[i] op= v`), <elem-ty> is the actual element LLVM type derived
//!      from the array's ResolvedType — not an `i64` fallback when the
//!      array's resolved type is in fact a Vec/Slice/Array of a different
//!      element type."
//!
//! These tests block regressions of the compound-assign element-type
//! resolution in `expr_helpers_assign.rs`. Simple assignment (`arr[i] = v`)
//! already routes through the AccessKind enum and is correct; compound
//! assignment (`arr[i] += v`) used to fall through a `_ => llvm_type_of(...)`
//! catch-all that erased the element type to `i64`, producing IR like
//! `getelementptr i64, i64* %vec_ptr, i64 %idx` even when the Vec held
//! `Vec<u8>` elements (a `{ i8*, i64 }` fat-ptr struct).
//!
//! Currently covers:
//!   - compound assign on Vec<i64> (basic — must keep emitting i64 GEP)
//!   - compound assign on Vec<i32> (regression: was sometimes i64-erased)
//!
//! Direct repro of the vaisdb classes that exposed the bug:
//!   - node.ll:1848 — Vec<Vec<u8>> push of a slice element
//!   - key.ll:1128  — Vec<&[u8]> indexing (fat-ptr-of-fat-ptr)
//! These two are read-paths (not compound assign), but share the same
//! element-type erasure family. Their direct fixes live in
//! expr_helpers_data.rs / call-arg coercion; this test guards the
//! compound-assign cousin.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ir(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed:\n{}\nErr: {}", source, e))
}

/// Find every `getelementptr` line and return (elem_ty, full_line).
/// Skips runtime helpers (function names starting with `__` or libc names)
/// because they hand-roll IR.
fn find_geps(ir: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut in_helper = false;
    for line in ir.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("define ") {
            in_helper = rest
                .find('@')
                .and_then(|i| rest.get(i + 1..))
                .map(|n| n.starts_with("__") || n.starts_with("llvm."))
                .unwrap_or(false);
        }
        if in_helper {
            continue;
        }
        // Match: `  %tN = getelementptr <ty>, <ty>* <base>, i64 <idx>`
        if let Some(idx) = trimmed.find("= getelementptr ") {
            let rhs = &trimmed[idx + "= getelementptr ".len()..];
            // Element type is the token before the first comma.
            let comma = rhs.find(',').unwrap_or(rhs.len());
            let elem_ty = rhs[..comma].trim().to_string();
            out.push((elem_ty, trimmed.to_string()));
        }
    }
    out
}

/// Return true if any GEP in `ir` has element type exactly equal to
/// `expected_elem`. Used as a positive assertion: "the function we just
/// compiled must have produced at least one GEP with this element type".
fn ir_has_gep_with_elem(ir: &str, expected_elem: &str) -> bool {
    find_geps(ir)
        .iter()
        .any(|(elem, _)| elem == expected_elem)
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "PASSES with stash@{0} phaseO_compound_assign_fix applied. \
            Currently negative test — verifies bug exists. Will be enabled \
            in Pillar 1.3 (codegen indexing 4-path 통합). See ROADMAP iter 74 \
            stash decision (vaisdb -3 regression caused revert)."]
fn index_invariant_compound_assign_i32_keeps_i32_elem_type() {
    // Vec<i32> compound assign — the GEP chain that walks Vec.data must
    // step by i32 (4 bytes), not by i64 (8 bytes). If the catch-all
    // fallback in expr_helpers_assign.rs reverts and emits `i64` here,
    // the store will write past the element boundary and corrupt the
    // adjacent slot.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            v: Vec<i32> := mut [1, 2, 3];
            v[0] += 10;
            R 0;
        }
        "#,
    );
    // The compound-assign emit path must use elem_llvm_ty = i32, so
    // there must be at least one `getelementptr i32, i32* ...` line.
    assert!(
        ir_has_gep_with_elem(&ir, "i32"),
        "Vec<i32> compound assign lost element type — no `getelementptr i32` found.\nIR:\n{}",
        ir
    );
}

#[test]
fn index_invariant_compound_assign_i32_no_i64_gep_on_vec() {
    // Stronger regression check: the bug pattern was `getelementptr i64,
    // i64* %some_vec_ptr` when the user wrote `vec_of_i32[i] += x`.
    // After the fix the compound-assign path goes through Vec's data
    // pointer (loaded as i64, inttoptr to i32*, GEP by i32). The final
    // index GEP must therefore be `getelementptr i32, ...`, never
    // `getelementptr i64, ...` on the user's array.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            v: Vec<i32> := mut [10, 20, 30];
            v[1] += 5;
            R 0;
        }
        "#,
    );
    // Find any GEP whose element type is `i64` AND whose base operand
    // looks like a typed pointer to user memory (heuristic: the `, i64*
    // %tN` operand pair). Vec internals (loading data slot from a
    // `{ i64, i64, i64, i64, i64 }` struct) are allowed to GEP through
    // i64 — those have `, %Vec$i32* %v` or struct-typed base instead.
    let geps = find_geps(&ir);
    let bad: Vec<&(String, String)> = geps
        .iter()
        .filter(|(elem, line)| {
            elem == "i64"
                // Heuristic: the user-array bug emits `, i64* %tN, i64 %tM`
                // (typed i64* base from inttoptr fallback). Internal Vec
                // struct GEPs use a typed `%Vec$T*` base instead.
                && line.contains(", i64* %")
                && line.contains(", i64 %")
        })
        .collect();
    assert!(
        bad.is_empty(),
        "Vec<i32> compound assign emitted `getelementptr i64, i64* %t...` — \
         element-type erasure regression:\n{:#?}\nIR:\n{}",
        bad,
        ir
    );
}

#[test]
#[ignore = "PASSES with stash@{0} phaseO_compound_assign_fix applied. \
            Currently negative test — Vec<i64> compound assign emits no GEP \
            without the fix. Will be enabled in Pillar 1.3."]
fn index_invariant_compound_assign_i64_still_emits_i64_gep() {
    // Sanity check: when the element type genuinely is i64, the GEP
    // *should* be `getelementptr i64, i64* ...`. This catches the case
    // where someone over-corrects and breaks the i64 path.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            v: Vec<i64> := mut [1, 2, 3];
            v[0] += 100;
            R 0;
        }
        "#,
    );
    assert!(
        ir_has_gep_with_elem(&ir, "i64"),
        "Vec<i64> compound assign should still produce an `i64` GEP somewhere:\nIR:\n{}",
        ir
    );
}

#[test]
fn index_invariant_simple_assign_vec_i32_already_correct() {
    // Sanity check: simple assign on Vec<i32> already routes through
    // AccessKind::VecData and emits `getelementptr i32, ...`. Including
    // it here documents the contract that compound assign must match.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            v: Vec<i32> := mut [1, 2, 3];
            v[0] = 42;
            R 0;
        }
        "#,
    );
    assert!(
        ir_has_gep_with_elem(&ir, "i32"),
        "Vec<i32> simple assign baseline regression — no `getelementptr i32` found.\nIR:\n{}",
        ir
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Pillar 1.1 보강 (iter 85, ADR 0002 R2 Class 2): vaisdb 패턴 cover
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn index_invariant_simple_assign_vec_u8_emits_i8_gep() {
    // vaisdb 패턴: byte buffer 단순 indexed write.
    // Vec<u8> simple assign이 i8 GEP를 emit해야 한다 (i64 fallback 금지).
    // 본 case는 simple-assign path (AccessKind::VecData)로 이미 정상 동작.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            buf: Vec<u8> := mut [0, 0, 0];
            buf[0] = 65;
            R 0;
        }
        "#,
    );
    assert!(
        ir_has_gep_with_elem(&ir, "i8"),
        "Vec<u8> simple assign should emit `getelementptr i8, ...`. \
         Without it, byte-level write writes 8 bytes (clobber adjacent slots).\nIR:\n{}",
        ir
    );
}

#[test]
fn index_invariant_simple_read_vec_u8_emits_i8_gep() {
    // vaisdb 패턴: byte buffer 단순 read.
    // Vec<u8> indexed read도 i8 GEP를 emit해야 한다.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            buf: Vec<u8> := mut [10, 20, 30];
            x := buf[1];
            x as i64
        }
        "#,
    );
    assert!(
        ir_has_gep_with_elem(&ir, "i8"),
        "Vec<u8> indexed read should emit `getelementptr i8, ...`.\nIR:\n{}",
        ir
    );
}

#[test]
fn index_invariant_simple_assign_vec_i32_no_compound() {
    // vaisdb 패턴 보강: Vec<i32> simple assign (compound 아님)이 i32 GEP 정상.
    // Pillar 1.3 indexing 4-path 통합 시 simple/compound 양쪽 path가
    // 일관성 있어야 함을 명문화.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            v: Vec<i32> := mut [1, 2, 3];
            v[0] = 42;
            v[1] = 99;
            R 0;
        }
        "#,
    );
    let geps = find_geps(&ir);
    // 기대: 적어도 i32 GEP 1개 이상 (write 위치마다 1개)
    let i32_count = geps.iter().filter(|(e, _)| e == "i32").count();
    assert!(
        i32_count >= 1,
        "Vec<i32> simple assign should emit at least one `getelementptr i32, ...`. \
         Found {} i32 GEPs.\nIR:\n{}",
        i32_count, ir
    );
}

#[test]
#[ignore = "vaisdb node.ll:1848 패턴 (Vec<Vec<u8>> push of slice). \
            현재 codegen에서 inner Vec<u8> erasure로 fail. \
            Pillar 1.2 (TC inference Var 해소) + Pillar 1.3 (indexing 4-path) \
            완료 후 PASS 예정. ADR 0002 Class 2/3 cross-cutting."]
fn index_invariant_vec_of_vec_u8_outer_push_emits_struct_gep() {
    // vaisdb node.ll:1848의 reduced repro:
    // Vec<Vec<u8>> 의 outer indexing은 inner Vec<u8> struct ({i8*, i64})를
    // 가져오기 위해 `getelementptr {i8*, i64}, ...` 또는 `%Vec$u8` typed GEP
    // 가 필요. i64 erasure → 실패.
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            outer: Vec<Vec<u8>> := mut [];
            inner: Vec<u8> := mut [1, 2, 3];
            outer.push(inner);
            R 0;
        }
        "#,
    );
    // 기대: Vec<u8> 또는 fat-ptr struct GEP 존재 (정확한 형태는 implementation
    // detail이지만 i64 erasure는 금지)
    let geps = find_geps(&ir);
    let bad: Vec<&(String, String)> = geps
        .iter()
        .filter(|(elem, line)| {
            elem == "i64"
                && line.contains(", i64* %")
                && line.contains(", i64 %")
        })
        .collect();
    assert!(
        bad.is_empty(),
        "Vec<Vec<u8>> push emitted i64 GEP on user pointer — element type erasure:\n{:#?}",
        bad
    );
}

#[test]
#[ignore = "vaisdb key.ll:1128 패턴 (&[&[u8]] indexing). \
            fat-ptr-of-fat-ptr indexing은 Pillar 1.3 helper 통합 후 명확한 \
            GEP 형태 보장 가능. 현재는 i64 fallback path가 살아있음."]
fn index_invariant_slice_of_slices_u8_indexing() {
    // vaisdb key.ll:1128의 reduced repro:
    // &[&[u8]] (slice of byte slices) indexing은 outer fat-ptr extract 후
    // inner fat-ptr GEP를 거쳐야 함. 양쪽 모두 i64 erasure 금지.
    let ir = gen_ir(
        r#"
        F process(parts: &[&[u8]]) -> i64 {
            first := parts[0];
            R first.len() as i64;
        }
        F main() -> i64 {
            R 0;
        }
        "#,
    );
    let geps = find_geps(&ir);
    let bad: Vec<&(String, String)> = geps
        .iter()
        .filter(|(elem, line)| {
            elem == "i64"
                && line.contains(", i64* %")
                && line.contains(", i64 %")
        })
        .collect();
    assert!(
        bad.is_empty(),
        "&[&[u8]] indexing emitted i64 GEP — fat-ptr-of-fat-ptr erasure:\n{:#?}",
        bad
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Helper sanity
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn find_geps_detects_synthetic_mismatch() {
    // Sanity check on the helper itself — given hand-crafted IR with a
    // known-bad i64 GEP on a typed pointer base, the filter must catch it.
    let bad_ir = r#"
define i64 @bad() {
entry:
  %v = alloca i64
  %idx = add i64 0, 1
  %elem_ptr = getelementptr i64, i64* %v, i64 %idx
  ret i64 0
}
"#;
    let geps = find_geps(bad_ir);
    let bad: Vec<&(String, String)> = geps
        .iter()
        .filter(|(elem, line)| {
            elem == "i64" && line.contains(", i64* %") && line.contains(", i64 %")
        })
        .collect();
    assert!(
        !bad.is_empty(),
        "GEP filter failed to detect synthetic bad pattern:\n{}",
        bad_ir
    );
}
