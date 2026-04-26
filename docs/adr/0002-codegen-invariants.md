# ADR 0002 — Codegen Invariants (4 클래스)

**Status**: Draft (iter 80, 2026-04-26)
**Decision driver**: Phase Ω Pillar 1.0. ADR 0001 §1이 정의한 "근본 fix" 게이트 (R1+R2+R3)를 codegen 4 클래스에 구체화한다.
**Depends on**: [ADR 0001](0001-root-cause-definition.md) — "근본 해결"의 공식 정의.

---

## 배경 (Context)

ADR 0001은 "근본 fix"의 정의 (R1 invariant + R2 차단 테스트 + R3 same-class audit)를 명문화했다. 그러나 codegen에서 무엇이 invariant인지는 명세화되지 않은 채, iter 74~79의 시도 동안 산출이 흩어져 누적되었다:

| 산출 | 위치 | invariant statement |
|---|---|---|
| `ret_invariant_test.rs` (iter 74, 7cfc5caf) | crates/vais-codegen/tests | "Every emitted `ret <ty> <val>` instruction has val's LLVM type == <ty>." |
| `call_arg_invariant_test.rs` (iter 74, 041685e6 + db44f364) | 동일 | "When a function parameter expects `{ i8*, i64 }` and arg is `%Vec*`, call site emits Vec→fat-ptr coercion." |
| `index_invariant_test.rs` (iter 74) | 동일 | "Compound-assign GEP `<elem-ty>` is the actual element type, not i64 fallback." |

**문제**: 이 3개 테스트가 같은 ADR 0001 §1 R2 의무를 따르지만, **무엇이 codegen invariant의 전체 집합인지는 정의되지 않았다**. 즉:
- 어떤 새 invariant를 추가해야 하는가?
- 각 invariant가 충족되었는지 어떻게 확증하는가?
- "i64 fallback"이 모든 곳에서 금지인가, 특정 클래스만인가?

iter 78 baseline 측정 (5 vaisdb test, 32 clang errors)은 이 결여의 직접 증거다:
- slice coerce miss × 9건 (call-arg + ret + index의 세 구역에 분산)
- ptr↔i64 × 4건 (call-arg 또는 var-load의 일부)
- integer constant non-integer × 5건 (별도 클래스 — 본 ADR이 5번째 invariant로 추가할 후보)
- undefined `@to_vec` × 1건 (resolved_function_sigs 누락 — 별도 클래스)

**관찰**: 32 errors가 "4-5 클래스에 cluster"되며, 클래스별 invariant가 정의되면 fix 우선순위가 명확해진다.

---

## 결정 (Decision)

본 ADR은 codegen이 보장해야 하는 **4 클래스 invariant**를 정의한다. 각 invariant는 ADR 0001 §1의 R1+R2+R3 의무를 따른다.

### Class 1 — `ret` elem-ty (LANDED, iter 74)

#### R1 — Invariant
> "codegen이 emit하는 모든 `ret <ty> <val>` 명령은 val의 LLVM 타입이 함수 시그니처의 ret `<ty>`와 정확히 일치한다."

#### R2 — 차단 테스트
- 위치: `crates/vais-codegen/tests/ret_invariant_test.rs`
- 5 case (iter 74):
  - i32 → i64 widening
  - float → double widening
  - `%Vec*` → `{ i8*, i64 }` (fat-ptr 변환)
  - i64 → fat-ptr (gated)
  - i64 → struct (gated)
- 추가 case 필요: function_gen/codegen.rs ~20 사이트, string_ops.rs 6, emit.rs 1, stmt_visitor poll/async 2 — 각 마이그레이션 시 case 추가 의무

#### R3 — Same-Class Audit
- grep target: `crates/vais-codegen/src/` 내 `ret <` emit 사이트
- 통합 helper: `coerce_ret_value` (iter 74 7cfc5caf 신설)
- audit 의무: 새 ret emit 사이트 추가 시 `coerce_ret_value` 호출 확증

### Class 2 — `index/store` elem-ty (PARTIAL, iter 74 R2 5 case)

#### R1 — Invariant
> "codegen이 emit하는 모든 `getelementptr <elem-ty>, <elem-ty>* <base>, i64 <idx>` (배열 인덱싱) 및 `store <ty> <val>, <ty>* <ptr>` (인덱스 어사인)에서, `<elem-ty>` / `<ty>`는 base의 ResolvedType으로부터 도출된 실제 element LLVM 타입이며, ResolvedType이 Vec/Slice/Array의 element를 명시할 때 i64 fallback을 사용하지 않는다."

#### R2 — 차단 테스트
- 위치: `crates/vais-codegen/tests/index_invariant_test.rs`
- 현재 5 case (iter 74):
  - compound assign on Vec<i64> (basic)
  - compound assign on Vec<i32> (regression: i64-erased)
- **P1.1에서 ≥10 case로 확장 의무** (iter 83):
  - struct field Vec<T> compound assign
  - method chain 결과 indexing
  - ref/&mut through compound assign
  - **Vec<&[u8]> indexing read (key.ll:1128 패턴)**
  - **Vec<Vec<u8>> indexing read (node.ll:1848 패턴)**

#### R3 — Same-Class Audit
- grep target: `getelementptr` / `store` emit 4 path
  - `expr_helpers_data.rs` (read)
  - `expr_helpers_assign.rs` (simple write)
  - `expr_helpers_assign.rs` (compound write)
  - `inkwell/` (별도 backend)
- 통합 helper 후보: `resolve_index_access(arr_ty) -> (elem_llvm, AccessKind, elem_resolved)` (P1.3에서 신설 예정)
- audit 의무: P1.3 LANDED 시점에 4 path 모두 단일 helper로 수렴

### Class 3 — `call-arg` coerce (PARTIAL, iter 74 structural guard)

#### R1 — Invariant
> "codegen이 emit하는 모든 `call <ret-ty> <fn>(<args>...)`에서, 각 `<arg>`의 LLVM 타입은 callee의 함수 시그니처가 요구하는 param 타입과 정확히 일치한다. 불일치 시 call 직전에 명시적 coerce를 emit한다 (예: Vec→fat-ptr는 load+load+2x insertvalue)."

#### R2 — 차단 테스트
- 위치: `crates/vais-codegen/tests/call_arg_invariant_test.rs`
- 현재 case:
  - method call에서 param이 Slice이고 arg가 `&Vec<T>` (LANDED 041685e6)
  - `vec_of_vec_indexing_loses_element_type` (`#[ignore]`, P1.2 LANDED 시 활성화)
- 추가 case 필요:
  - 외부 trait dispatch 사이트
  - generic function instantiation 시 param/arg generic erasure 시점
  - tuple/struct unpacking → call

#### R3 — Same-Class Audit
- grep target: `call ` emit 사이트 (~329 사이트 중 method_call.rs / function_call.rs / trait_dispatch.rs 등)
- 통합 helper 후보: `coerce_call_arg(param_ty, arg_val) -> Value` (P1.4 Type-Tagged IR Builder에서 흡수)

### Class 4 — `var-to-llvm` (NEW)

#### R1 — Invariant
> "codegen이 ResolvedType 값을 LLVM 타입으로 매핑할 때, ResolvedType::Var(_) / ResolvedType::Unknown을 만나면 i64 fallback을 사용하지 않고 컴파일을 중단한다 (panic! 또는 hard error). i64 fallback은 ResolvedType::Generic(monomorphization 미완성) 또는 ResolvedType::ConstGeneric에서만 허용된다."

#### R2 — 차단 테스트
- 위치 (신규): `crates/vais-codegen/tests/var_to_llvm_invariant_test.rs` (P1.0 후속, iter 81+)
- case 후보:
  - TC inference 누락 → ResolvedType::Var이 codegen 도달 시 즉시 panic
  - Unknown 타입 (TC 에러 후 codegen 진입 시) → 즉시 panic
  - Generic 타입은 codegen에서 i64 fallback 허용 + Warning emit (현 동작)
  - ConstGeneric도 동일

#### R3 — Same-Class Audit
- grep target:
  - `crates/vais-codegen/src/types/conversion.rs:360-366` (현 fallback 사이트)
  - `crates/vais-codegen/src/expr_helpers_data.rs:484-489` (Named/Unknown/Generic → "i64")
  - `_ => "i64"` 패턴 전수 grep
- audit 의무: `_ => "i64"` 가 ResolvedType 매칭에 있으면 (a) Generic/ConstGeneric만 허용 (b) 다른 케이스는 panic 또는 에러 전환

---

## R1+R2+R3 충족 매트릭스 (iter 80 baseline)

| Class | R1 stated | R2 test exists | R2 case sufficient | R3 audit done | Status |
|---|---|---|---|---|---|
| 1. ret elem-ty | ✅ | ✅ ret_invariant_test.rs | ⚠ 5/40+ sites | ⚠ partial (1/30+ migrated) | **partial-LANDED** |
| 2. index/store | ✅ | ✅ index_invariant_test.rs | ⚠ 5 → ≥10 needed (P1.1) | ❌ 4 path 미통합 (P1.3) | **partial-LANDED** |
| 3. call-arg | ✅ | ✅ call_arg_invariant_test.rs | ⚠ 1 active + 1 ignored | ❌ ~329 site 미통합 (P1.4) | **partial-LANDED** |
| 4. var-to-llvm | ✅ (이 ADR) | ❌ 신설 필요 (P1.0 후속) | — | ❌ 미수행 | **NEW (proposed)** |

**Phase Ω 완성 상태** (목표):
- 4/4 R1+R2+R3 모두 ✅
- iter 78 baseline 32 errors → 0 errors (Pillar 1 종료 시)
- ADR 0002 LANDED 후 신규 codegen if-coerce 분기는 4 클래스 중 하나에 분류 의무

---

## 단계적 도입 plan (P1.0 → P1.4)

| iter | task | invariant | 산출 |
|------|------|-----------|------|
| 80~82 | P1.0 본 ADR 채택 | 4 클래스 명세 | docs/adr/0002 LANDED |
| 83 | P1.1 index test 보강 | Class 2 R2 case ≥10 | tests/index_invariant_test.rs 확장 |
| 84~88 | P1.2 TC Var unify | Class 4 R3 (수백 사이트) | vais-types/checker_expr/calls.rs:291 fix |
| 89~93 | P1.3 indexing 4-path 통합 | Class 2 R3 | resolve_index_access helper 신설 |
| 94~100 | P1.4 Type-Tagged IR Builder | 4 클래스 모두 R3 자동화 | write_ir! → emit_call/emit_ret/emit_gep typed wrapper |

**총 ~6주 multi-session**. 매 iter 종료 시 검증 의무 (cargo test --workspace ≥ 2625, integrity vaisdb ≥ 261, std ≥ 82).

---

## CLAUDE.md 규칙과의 연결

규칙 8 (codegen 새 if-coerce 분기 추가 시 ADR 0001 분류 의무)는 본 ADR로 구체화된다:

- "근본 fix"로 분류 시 → 4 클래스 중 하나에 매핑 + R1/R2/R3 의무
- 4 클래스 외 새 invariant 발견 시 → ADR 0003 신설 + CLAUDE.md 규칙 8 갱신

규칙 11 (Phase 시작 시 invariant 명시)는 본 ADR이 적용 사례:
- Pillar 1 invariant (각 sub-Pillar): 본 ADR 4 클래스 중 어느 R3을 충족시키는가
- exit_audit: cargo + integrity + 4 클래스 R2 차단 테스트 PASS

---

## 결과 (Consequences)

### 긍정
- 4 클래스 명세화 → 신규 codegen 분기 분류 명확
- iter 78 baseline 32 errors → 클래스별 cluster 식별 가능 → fix 우선순위 도출
- ADR 0001 R3 의무가 추상이 아니라 구체 클래스에 binding됨
- Phase Ω Pillar 1 (P1.1~P1.4)이 본 ADR을 단일 spec으로 참조

### 부정
- Class 4 (var-to-llvm)는 panic 도입이라 cascade 위험 (R2 테스트 + R3 audit 없이는 cargo test --workspace 깨질 수 있음)
- Type-Tagged IR Builder (P1.4)는 ~7주 작업. ADR 채택이 그 commitment 묶음
- 5번째 클래스 (integer constant, undefined fn) 후보가 iter 78 baseline에서 발견되었으나 본 ADR 미포함 — 별도 ADR 0003 또는 본 ADR 갱신

### 위험 완화
- Class 4는 R2/R3 차단 테스트 LANDED 후에만 production code path에 panic 도입
- 단계적 도입 plan에 따라 multi-session 분산
- 각 iter 종료 시 cargo + integrity + vaisdb 3축 검증 의무 (ROADMAP 위험 회피 원칙 4)

---

## 적용 시점

- **2026-04-26 (본 문서 채택일)**: ADR 0002 Draft 상태. 사용자 review 후 Accepted 전환
- **iter 81~ Accepted 후**: 신규 codegen if-coerce 분기는 본 ADR 4 클래스 중 하나에 분류 의무 (CLAUDE.md 규칙 8 강화)
- **Pillar 1 종료 후**: 본 ADR 가 Pillar 1의 exit audit 기준

---

## 참고

- ADR 0001: 근본 해결의 공식 정의 (R1+R2+R3 게이트)
- iter 74 산출 commits: 7cfc5caf (coerce_ret_value), 1b99766c (ret_invariant_test), 041685e6 (call-arg structural guard), db44f364 (call_arg_invariant_test)
- iter 78 baseline: lang/packages/vaisdb/ROADMAP.md "iter 78 strategy + 결과" 섹션
- Phase Ω 4-Pillar 구조: ROADMAP.md "🎯 Phase Ω" 섹션
- CLAUDE.md 규칙 8~12: 사전 가드레일

---

## 변경 이력

| 일자 | iter | 변경 | 작성자 |
|------|------|------|--------|
| 2026-04-26 | 80 | Draft 작성 | Opus direct (P1.0) |
