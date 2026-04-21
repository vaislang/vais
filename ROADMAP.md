# Vais — 문법 + 컴파일러 100% 완성 Roadmap

> **버전**: 2026-04-20 재작성
> **이전 ROADMAP**: `ROADMAP-archive.md` (Phase 0 ~ 6.31 히스토리)
> **범위**: **문법 + 컴파일러** 100% 완성만. stdlib/vaisdb/생태계는 별도 드라이브.
> **현재 커밋**: `cde15d44` (archive 이동 직전 상태)

---

## Baseline (2026-04-20 재작성 시점 실측)

Commit `cde15d44` 기준 `./scripts/check-integrity.sh`:

| 항목 | 숫자 | 비고 |
|------|------|------|
| compiler_syntax | 199/199 (0 ignored) | Survey A 실측, ROADMAP "14 ignored" 는 stale |
| compiler_stages | 14/14 | 1 #[ignore] B7 known bug |
| std_files | 82/82 | 100% |
| living_spec | 101/101 | 100% |
| phase158 strict | 18/18 | 100% |
| vaisdb_files | 237/261 | 90.8% (이 ROADMAP 범위 밖 — 별도 드라이브) |
| vais-types unit | 355/356 | 1 pre-existing fail (`phase156::test_try_on_non_result_errors`) |
| vaisc e2e | 2622/0/1 | 1 ignored (`e2e_str_as_bytes`, 하네스 한계 Phase 6.31) |
| vaisc integration | 147/147 | 100% |
| vais-parser | 868/870 | 2 pre-existing fail (세션 외) |

이 baseline 이 모든 Phase 의 **regression floor**. 1-file regression 도 허용하지 않음.

---

## 설계 원칙

1. **실측 우선**: 모든 "완료" 판정은 최소 재현 + `vaisc check`/`vaisc build` 실제 실행으로 검증.
2. **CLAUDE.md rule 3/4 엄수**: 컴파일러 수정 전 baseline 기록, regression 1건이라도 발생 시 즉시 revert.
3. **Scope 제한**: 이 ROADMAP 은 **문법 + 컴파일러 완성** 만. stdlib API 확장, vaisdb 수정, 문서 튜토리얼 은 별도 드라이브.
4. **매트릭스 동기화**: `LANGUAGE_SPEC.md` Construct Status Matrix 가 single source of truth. 구현 완료 시 ◐ → ✓ 승격 필수.
5. **실제로 작동하는 것 먼저 마크**: Survey 에서 "ROADMAP 은 미완이나 실제 작동" 확인된 항목은 먼저 문서 동기화만으로 ✓.

---

## 4개 Survey 발견 요약 (2026-04-20)

**Survey A (parser/lexer)**:
- compiler_syntax 199 active / 0 ignored (ROADMAP "14 ignored" stale)
- removed keyword 완전 제거 확인 (spawn/lazy/force 잔여 참조 없음)
- Phase 1.x 대부분 파서 완성

**Survey B (TC/codegen)**:
- Phase 2.10 (Option match-arm 재포장): **실제 작동** ← ROADMAP 이 stale
- Phase 4.19~4.23 (linear/comptime/dyn/yield/move): **모두 build 성공** ← SCOPED 마크 stale
- Phase 3.13 (parse_f64/char_at): **실제 미완** — E002, C002
- Phase 3.14 (Vec<Struct>[i].field=): **실제 미완** — Vec<T> method specialization gap

**Survey C (dead code)**:
- **삭제 후보 0건**. 제거된 키워드/구버전 구현 모두 깔끔히 마이그레이션됨.
- 33개 `#[allow(dead_code)]` 모두 정당한 사유 (예약/부분사용/테스트).

**Survey D (LANGUAGE_SPEC matrix)**:
- 49 constructs: ✓ 35 / ◐ 14 / ✗ 0
- ◐ 중 절반은 사실상 작동 (거짓 부정): E, pure F, partial F, unsafe{}, dyn, linear/affine 등 — 매트릭스 업데이트만 필요
- 진짜 미완 (comptime eval, Vec<Struct>[i].field=, defer edge cases, cross-file impl split)
- 매트릭스 vs CODEGEN_FEATURES.md 불일치: cross-file impl split

---

## Current Tasks (2026-04-20)

mode: auto (문법 + 컴파일러 100% drive)
iteration: 4
max_iterations: 20
  strategy: A.1 단독 시작 (A.1/A.2만 unblocked, A.1은 측정+문서, A.2는 5 construct 독립 측정) → sequential
  opus_direct: A.1 — 측정 근거 → 매트릭스 판정이 분리 불가능한 evaluator 루프
  strategy iteration 2: A.2 순차 (A.3 가 A.2 결과에 blockedBy). 5 construct 각각 최소 재현 → 매트릭스 판정.
  opus_direct: A.2 — 측정 + 판정 루프. impl-sonnet 위임 시 "build 성공 vs codegen gap" 구분을 놓칠 리스크.
  strategy iteration 3: A.3 단독 (cross-file X impl 2 파일 재현 → LANGUAGE_SPEC vs CODEGEN_FEATURES 일치화).
  opus_direct: A.3 — 실측 결과가 2 문서의 어느 쪽을 정답으로 세울지 판정 루프. 2 파일, 소규모.
  strategy iteration 4: B.1 단독. 1단계 research (CODEGEN_FEATURES 전수 조사 + TC✓/codegen✗ 후보 목록) → 2단계 판정 (TC error 로 거부할 construct 선정) → 3단계 구현. 각 단계 Opus direct.
  opus_direct: B.1 — 매트릭스 조사 결과가 어떤 construct 를 거부 대상으로 삼을지 판정과 직결. 위임 시 의도 분리 불가.
  strategy iteration 4 note (2026-04-21): B.1 재정의. 실측 결과 "TC-pass/codegen-fail" 4개 중 (1)(2)(4)는 B.4/B.2 스코프 이미 포함, (3)은 독립 버그 아니라 Option/Result lowering 하나의 뿌리. B.1 → "Optional/Result lowering 근본 수정" 으로 변경 (사용자 승인). Explore agent 진단 완료: call.rs:183-246의 하드코딩 i64 가 원인. 구현은 /clear 후 새 세션.
  context_checkpoint: B.1 research 단계 완료. 구현은 다음 세션에서 fresh context 로 시작. 본 세션 3 iteration (A.1/A.2/A.3) + B.1 research = 문서 동기화 + 진단 완료. 컴파일러 코드 수정 0.
  strategy iteration 3 (2026-04-21 fresh session): B.1 단독 (B.2~B.5 모두 blockedBy B.1) → sequential. Research 완료 상태 — 수정 타겟 확정 (call.rs:183-246, generator.rs expected-type helper, gen_match.rs:915-1095). Opus direct — 매트릭스 lowering 설계와 구현 분리 불가 (expected-type 전파 방식 결정 필요).
  opus_direct: B.1 — LLVM aggregate layout 일관성 설계. hardcoded i64 제거 시 어떤 context path 로 expected-type 을 읽을지 (TC expected_types map 참조 vs call-site 전달) 판정 = 구현. CLAUDE.md rule 3/4 엄수 (baseline syntax=200 std=82/82 vaisdb=237/261 phase158=18/18, regression 1건 즉시 revert).
  context_checkpoint iteration 3 (2026-04-21): B.1 구현 완료 + 커밋 (`a7fa3ff8`). 본 세션은 research 없이 시작해서 진단·구현·검증·커밋 모두 1 iteration 내 완료. 5개 pending (B.2, B.3, B.4, B.5, B.6) unblocked. **컨텍스트 보호** 차원에서 /clear 후 fresh session 권장 — ROADMAP.md 가 full state source. 다음 세션은 B.2 부터 (impl-sonnet 위임 적합) 또는 B.6 부터 (Opus direct, B.1 연장선).
  strategy iteration 4 (2026-04-21 autonomous loop fire): B.2 단독 (impl-sonnet 위임) → sequential background. B.3/B.4/B.5/B.6 도 unblocked 이지만 (1) B.3/B.6 은 Opus direct — 현 세션 컨텍스트 여유 있을 때 다음 iteration 으로 유보, (2) B.4/B.5 는 text-IR 코드젠의 서로 다른 파일을 만지지만 같은 backend 이므로 병렬 위험, (3) B.2 는 runtime/string_ops.rs 에 국한. B.2 완료 후 재평가.
  opus_direct: 이 iteration 은 없음 (B.2 는 impl-sonnet). parse_f64 (strtod wrapper) + char_at (GEP+load) 모두 design 단순 — Sonnet 적합.
  iteration 4 outcome (2026-04-21): 두 번의 agent 위임 모두 research 단계에서 turn-cap truncation (a8884e59, a71db40d — worktree 자동 정리 → 변경 0). Opus direct 로 재조사한 결과 B.2 scope 이 잘못 추정됨: inkwell 은 string method dispatch 자체가 없다 (`generate_method_call` at gen_aggregate.rs:699 은 순수 TypeName_method lookup). 텍스트-IR 의 string_ops.rs 는 live path 아님 (VAIS_SINGLE_MODULE=1 deprecated → C005). B.2 재정의 후 pending 상환, 다음 iteration 시작 시 fresh session 으로 Opus direct design + impl-sonnet 구현 분리 수행 권장.
  stuck_recovery iteration 4: B.2 task in_progress → pending 복구. 사유: agent 2회 truncation + scope 재정의 필요. retry counter 미소비 (구조적 원인 — scope 추정 오류).

### Phase A — 문서 동기화 (Opus direct, 먼저 실행)

> **배경**: Survey B/D 에서 "실제 작동하는데 ROADMAP/매트릭스는 미완으로 표시" 된 항목 다수 발견. 코드 수정 없이 문서만 정정하면 완성도가 실제로 올라감. 이 Phase 는 baseline 유지 + 문서-실측 일치만 확인.

- [x] A.1 — Phase 2.10 Option match-arm 재포장: 실측 작동 확인 + 문서 업데이트 ✅ 2026-04-21
  실측 결과 (Opus direct):
    - 기존 e2e `phase2_10_option_rewrap_in_match_arm` (TC-only assertion): ✓ passing (이미 #[ignore] 해제된 상태)
    - 신규 reproducer 3 건 작성, LIVING_SPEC 에 TC-only 로 추가:
      * `docs/language/LIVING_SPEC/02_patterns/phase2_10_option_struct_rewrap.vais`
      * `docs/language/LIVING_SPEC/02_patterns/phase2_10_result_te_rewrap.vais`
      * `docs/language/LIVING_SPEC/02_patterns/phase2_10_nested_option_flatten.vais`
      → 세 건 모두 `vaisc check` ✓. LIVING_SPEC 101 → **104** 유지, 모두 pass.
    - 세 건 `vaisc build` 는 C004 `LLVM error: Aggregate extract index out of range` 실패.
      → Phase 2.10 closure 는 **TC-level 만**. Codegen 은 CODEGEN_FEATURES.md L171 의
        `F f(opt: Option<Struct>) -> Option<Primitive>` Phase 3.14/3.15 gap 로 귀결.
        Survey B 의 "실제 작동" 주장은 TC 한정. **이 결론은 사용자 AskUserQuestion 으로 승인.**
  문서 동기화:
    - CODEGEN_FEATURES.md L173: "Resolved Phase 2.10" → "**TC resolved** Phase 2.10. Codegen 은 L171 Phase 3.14/3.15 gap" 로 정정.
    - LANGUAGE_SPEC.md: Phase 2.10 matrix 엔트리 없음 — 업데이트 불필요.
  baseline 유지: syntax=200, stages=14, std=82/82, vaisdb=237/261, phase158=18/18 (check-integrity.sh 재실행 OK).
  카스케이드 메모 (Phase B 로 이월): 이 codegen gap 은 B.4 (Phase 3.14 Vec<Struct>[i].field=) 의
    lowering 작업과 지근 거리. B.4 내부 혹은 별도 B.6 으로 `Option<Struct>` 파라미터 lowering 을
    다루면 vaisdb cascade 가능성 있음. B.1 의 "TC ✓/codegen ✗" 매트릭스 전수 조사에서 재확인.

- [x] A.2 — Phase 4.19~4.23 SCOPED 재평가: 실측 작동 확인 + 매트릭스 정정 ✅ 2026-04-21
  실측 (Opus direct, `vaisc check` + `vaisc build` + run):
    - linear/affine: Parse ✓ / TC ✓ / Codegen ✓ / Run ✓ — use-count 강제만 Phase 4.19 SCOPED.
    - comptime (function-body `F f()->T = comptime { ... }`): Parse ✓ / TC ✓ / Codegen ✓ / Run ✓ (integer/conditional).
      * const-initializer `const N:i64 = comptime {...}` 은 여전히 Parse error → Phase B.3 대상 (확인).
    - dyn Trait: Parse ✓ / TC ✓ / Codegen ✓ / Run ✓ (basic method dispatch. full vtable 은 Phase 4.21 SCOPED).
    - yield: Parse ✓ / TC ✓ / Codegen ✓ / Run ✓ — simplified semantics (full coroutine desugar Phase 4.22 SCOPED).
    - move closure: Parse ✓ / TC ✓ / Codegen ✓ / Run ✓ — drop-on-move tracking Phase 4.23 SCOPED.
  신규 reproducer 5 건 `docs/language/LIVING_SPEC/01_keywords/` 추가:
    linear_affine_annotation.vais, comptime_function_body.vais, dyn_trait_param.vais,
    yield_in_loop.vais, move_closure_capture.vais. 모두 TC pass.
  LIVING_SPEC 104 → **109**, 모두 pass.
  LANGUAGE_SPEC.md 매트릭스 (L254/265/267-269) + keyword status 테이블 (L174-177, L185-187) 동기화:
    - Move closure / Dyn / Yield / Linear / Affine: ◐◐ → ✓✓ with SCOPED note
    - Comptime: Parse ◐ (function-body ✓ / const-init ✗ B.3), TC ✓, Codegen ✓, Run ✓
  baseline 유지: syntax=200, stages=14, std=82/82, vaisdb=237/261, phase158=18/18.

- [x] A.3 — Cross-file impl split 불일치 해결 ✅ 2026-04-21
  실측: cross-file X impl 분리 재현 (`/tmp/a3/shape_def.vais` + `shape_impl.vais`)
    → `U shape_def::Square` 후 `X Square: Shape { F area(self) { self.side * ... } }`
    는 E030 "No such field 'side' on Square" 로 TC 실패. Cross-file split 은 실제로 작동
    **하지 않음**. TYPE_SYSTEM.md §Phase 2.9 decision (a) 와 일치.
  정답: CODEGEN_FEATURES.md L93 `✗` 가 맞음. LANGUAGE_SPEC.md L231 의 `◐ cross-file
    dispatch: Phase 2.9` 는 "진행 중" 뉘앙스로 오인 소지 → **명시적 disallowed** 로 갱신.
  changes:
    - docs/LANGUAGE_SPEC.md L231 Impl block row: "✓ (same-file). Cross-file `X`/`S`
      split intentionally **disallowed** — Phase 2.9 decision (a), TYPE_SYSTEM.md §Phase 2.9.
      Workaround: co-locate `X` with `S`."
    - CODEGEN_FEATURES.md L93 기존 `✗` + "Phase 2.9 decision (a)" 주석 유지 (변경 없음).
  baseline 유지: syntax=200 std=82/82 vaisdb=237/261 phase158=18/18.

### Phase B — 실제 미완 구현 (Opus direct / impl-sonnet)

> **배경**: Survey 결과 실제로 구현 필요한 항목은 Phase 3.12 / 3.13 / 3.14 + comptime eval + defer edge case, 총 5개. 순차 진행. 각각 regression gate 준수.

- [x] B.1 — **Optional/Result lowering 근본 수정** (Opus direct, inkwell backend) ✅ 2026-04-21
  **구현 완료 (2026-04-21, fresh session iteration 3)**:
  - `crates/vais-codegen/src/inkwell/gen_types.rs` L95-108: `Option<T>` / `Result<T,E>`
    named-generic AST → `ResolvedType::Optional` / `Result` (was slipping into
    generic `Named` branch → `%Option = type opaque`).
  - `crates/vais-codegen/src/inkwell/types.rs` L285-306: `Optional(T)` / `Result(T,E)`
    LLVM layout pinned to `{ i8, i64 }` regardless of T. Matches gen_types.rs
    try/unwrap comments and user-enum ABI.
  - `crates/vais-codegen/src/inkwell/gen_expr/call.rs`:
    * L182-192: Some/Ok/Err 셋을 `build_option_result_ctor(tag, args, prefix)` 에 위임.
    * L767-883: new `build_option_result_ctor` + `pack_enum_payload_i64` helpers
      — struct 인자를 size 따라 (≤8B stack-alloca bitcast / >8B malloc+store+ptrtoint)
      i64 slot 에 pack. 이전엔 `coerce_to_i64` 가 struct 에 대해 const 0 을 반환 (silent drop).
  - `crates/vais-codegen/src/inkwell/gen_match.rs`:
    * New `match_scrutinee_type_stack` field (generator.rs). `generate_match` 가
      scrutinee 의 ResolvedType 을 push 하고 arm 처리 후 pop.
    * New `option_result_payload_struct_type()` helper 가 scrutinee 에서 Some/Ok/Err
      payload 의 LLVM StructType 을 복원 — Named struct, nested Option/Result, Tuple 지원.
    * Variant 디코드 경로가 `enum_variant_struct_types` lookup 실패 시 scrutinee 기반
      struct-type 복원으로 fallback.
    * `var_struct_types` 태깅은 named struct payload 에 대해서만.

  **검증**:
    - docs/language/LIVING_SPEC/02_patterns/phase2_10_option_struct_rewrap.vais:
      `vaisc build` ✓ → exit=42 ✓
    - docs/language/LIVING_SPEC/02_patterns/phase2_10_result_te_rewrap.vais:
      `vaisc build` ✓ → exit=7 ✓
    - docs/language/LIVING_SPEC/02_patterns/phase2_10_nested_option_flatten.vais:
      `vaisc build` ✓ → exit=100 ✓ (nested Option<Option<i64>> 평탄화)
    - Baseline 완전 유지: syntax=200 stages=14 std=82/82 vaisdb=237/261 phase158=18/18
      (check-integrity.sh 수정 전/후 동일).
    - vais-types unit 1238/1 (1 pre-existing fail, baseline 과 일치).
    - vaisc integration 147/147 ✓, execution 25/25 ✓, error_message 115/115 ✓.

  **스코프 제한 (Phase 6 으로 이월)**:
  - 텍스트-IR 백엔드 (`crates/vais-codegen/src/` non-inkwell) 는 독립 구현
    (type_inference.rs L460-495 monomorphization via `Option$T`). 동일 gap 존재 가능성
    있으나 B.1 스코프 외. 본 세션 변경은 inkwell 백엔드 (vaisc build 기본값) 에만 적용.
  - Loop-break-Option 완성 기준 항목: `generate_break` 가 현재 "loop-with-value"
    미구현 (gen_stmt.rs:1059 "In a full implementation, this would be used").
    이는 Option lowering 문제가 아니라 별개의 loop-expression 기능 gap. → **B.6 으로 분리**.

  **카스케이드 메모**: 이 fix 가 유사 패턴 (Option<Vec<T>>, Result<HashMap<K,V>, E> 등)
  에서 cascade unlock 가능. C.1 re-survey 시 확인.

- [ ] B.2 — Phase 3.13 string runtime intrinsics (Opus direct) [blockedBy: B.1]
  **재정의 (2026-04-21 iteration 4)**: 원래 "parse_f64/char_at 추가" 였으나 실측 결과
  **inkwell backend 이 string method dispatch 자체가 없음**. `generate_method_call`
  (`crates/vais-codegen/src/inkwell/gen_aggregate.rs:699`) 은 단순 `TypeName_method`
  함수 lookup 만 한다 — `s.parse_i64()` 조차 C002 fail. 텍스트-IR 의
  `string_ops.rs::generate_string_method_call` (line 256) 은 inline impl 이 있지만
  `VAIS_SINGLE_MODULE=1` 은 deprecated → C005. 즉 inkwell 이 유일 backend.

  구현 실측 (2026-04-21):
    - `s.parse_i64()` → inkwell C002 (parse_i64 도 미구현)
    - `s.char_at(1)` → inkwell C002
    - `s.parse_f64()` → inkwell C002
    - 텍스트-IR 의 기존 구현은 live path 아님.

  두 agent 위임 시도 모두 research 단계에서 turn-cap truncation (agent-a8884e59,
  agent-a71db40d). Research burden 큼 → Opus direct 로 fresh session 에서.

  target: `crates/vais-codegen/src/inkwell/gen_aggregate.rs:699` (generate_method_call)
    에 string-method 특수 케이스 삽입. 또는 `crates/vais-codegen/src/inkwell/builtins/`
    하위에 신규 `string_methods.rs` 추가하여 dispatch 조립.

  approach (Opus direct, 다음 세션 권장):
    1. inkwell `generate_method_call` 에서 receiver 가 `str` (ResolvedType::Str 이거나
       recv_val 이 `{ i8*, i64 }` fat-pointer struct) 인지 체크.
    2. str method dispatch 테이블:
       - `char_at(i)` → GEP + load i8 + zext to i64 (가장 단순)
       - `parse_f64()` → strtod extern + Result<f64, str> packing.
         * B.1 의 `build_option_result_ctor` helper 재사용: payload f64 → bitcast to i64.
         * Err branch 는 static "parse error" string 반환.
       - `len()` 은 이미 L711 처리됨 (fat-pointer extract field 1).
       - (선택) parse_i64, parse_u64, parse_i32, parse_u32: TC 는 signature 제공,
         codegen 만 가하면 완성. strtoll/strtoull extern.
    3. e2e 테스트 추가 (inkwell backend 로 검증):
       - 방법 1: `docs/language/LIVING_SPEC/` 에 재현 파일 추가 (LIVING_SPEC 은 `vaisc check` 기반이라 build 테스트는 별도 필요).
       - 방법 2: `crates/vaisc/tests/integrity/` 에 integration_tests 와 유사 패턴으로 inkwell-build+run 테스트 추가.

  [완료 기준]:
    - `s.parse_f64()` / `s.char_at(i)` / `s.parse_i64()` 모두 `vaisc build` 통과 + 런타임 정상.
    - e2e 테스트 3+ (parse_f64 ok/err, char_at). inkwell backend 경로.
    - baseline: syntax=200 std=82/82 vaisdb=237/261 phase158=18/18 (현재 수준 유지 / 향상).
    - CODEGEN_FEATURES.md L176-177 업데이트 (parse_* 해결 표시).
    - vaisdb cascade 측정 (의무 아님).

  blocker note: agent 위임 2회 truncation 경험. 다음 시도 시 **design step 을 Opus direct 로**,
  **implementation step 만 impl-sonnet 으로 분리 위임** 권장.

- [ ] B.3 — comptime {} 표현식 evaluation (Opus direct, research-heavy) [blockedBy: B.1]
  target: crates/vais-parser/src/expr/primary.rs (comptime block parse) + crates/vais-types/src/comptime/ (evaluator)
  approach:
    1. 재현: `const N: i64 = comptime { 5 + 3 }` parse error 위치 특정
    2. Parser 에 comptime block 이 const initializer 에서 허용되도록
    3. TC/codegen 에서 comptime block 을 constant-fold — 최소: integer 산술만
  scope limitation: 최소 integer 산술만 (+/-/*/Mod). 복잡한 comptime loop/if 는 추후 phase.
  [완료 기준]:
    - `const N: i64 = comptime { 5 + 3 }` 통과, N == 8
    - 매트릭스 `comptime {}` ◐ → 최소한 "integer arithmetic ✓, complex ◐"
    - baseline 유지

- [ ] B.4 — Phase 3.14 Vec<Struct>[i].field= write-through (Opus direct) [blockedBy: B.1]
  target: crates/vais-codegen/src/ (index-assign + Vec<Struct> specialization)
  approach:
    1. 재현: `v: Vec<Point> := ...; v[0].x = 10` 의 실패 지점 확정
    2. GEP 체인 구현: `v.data + i*sizeof(T) + offsetof(field)` → store
    3. Vec<T> method specialization gap 해결 (Survey B 의 "systematically 누락" 패턴)
  [완료 기준]:
    - `v[i].field = value` e2e 3+ pass
    - LANGUAGE_SPEC Matrix L262 `Vec<Struct>[i].field =` ◐ → ✓
    - baseline 유지
    - vaisdb C003 에러 중 이 패턴 cascade 측정

- [ ] B.5 — D (defer) edge case 완성 (Opus direct) [blockedBy: B.1]
  target: crates/vais-codegen/src/stmt_visitor.rs + control_flow.rs (early-return/break/nested loop 시 defer 실행)
  approach: Survey D 에 의하면 basic scope-exit OK. early return + break 시 defer LIFO 순서 보장이 미완.
    1. 재현: 3 케이스 (early return / break / nested loop break)
    2. 각각 defer 가 LIFO 로 실행되도록 codegen 수정
    3. phase3_16_defer e2e 이미 4/4 passing → 이 작업은 **추가** 테스트
  [완료 기준]:
    - phase3_16_defer_edge_cases e2e 3+ 추가 pass
    - 매트릭스 `Defer (D)` ◐ → ✓
    - baseline 유지

- [ ] B.6 — Loop-with-value (`B expr` → loop-expression) codegen (Opus direct) [blockedBy: B.1]
  **배경**: B.1 구현 중 발견. `generate_break` (gen_stmt.rs:1058-1061) 가 break-value 를
  평가한 뒤 **무조건 버린다** ("In a full implementation, this would be used"). 즉
  `F f() -> Option<Point> { L { B Some(p) } }` 패턴은 타입체커 통과 후 코드젠이 값을
  drop 하고 loop 은 unit 반환 → 런타임에 None-branch 로 떨어짐. Option/Result 에 국한된
  lowering 이슈가 아니므로 B.1 에서 분리.
  target: crates/vais-codegen/src/inkwell/gen_stmt.rs (generate_break, generate_condition_loop,
    generate_while_loop, generate_range_for_loop) + LoopContext 확장.
  approach:
    1. LoopContext 에 `break_value_slot: Option<PointerValue<'ctx>>` 추가.
    2. Loop 진입 시 첫 break-value 타입을 peek 하거나 scrutinee/expected-type 로 결정 →
       alloca 할당. 다수 break 는 같은 slot 에 store.
    3. `generate_break(Some(val))`: val → coerce to slot 타입 → store → branch to break_block.
    4. Loop exit: alloca 에서 load 후 loop-expression 값으로 반환. 값 없는 loop 은 unit.
  [완료 기준]:
    - `L { B 42 }` → 42 (i64)
    - `F f() -> Option<i64> { L { B Some(42) } }` → Some(42) (B.1 재검증)
    - `F g() -> Option<Point> { ... L { B Some(p) } }` → Some(Point) (struct payload)
    - 기존 값 없는 break 는 동일 동작 (regression 0)
    - e2e 3+ 추가 pass

### Phase C — 매트릭스 최종 정합 + 100% 선언

- [ ] C.1 — LANGUAGE_SPEC Construct Status Matrix 전수 재검증 (Opus direct, measurement) [blockedBy: B.5]
  target: 49 construct 모두 Parse/TC/Codegen/Run 컬럼 실측으로 재검증
  approach: 각 construct 최소 재현 → 4단계 결과 기록 → 매트릭스 갱신
  [완료 기준]:
    - 매트릭스 ✗ 개수 0 유지 (원래 0)
    - ◐ 개수 최소화 (Phase 4.x SCOPED 중 진짜 못 구현한 것만)
    - Gate rule 준수: "every ◐ has a planned Phase in ROADMAP.md"
    - Survey 결과와 일치

- [ ] C.2 — 100% 완성 선언 + stdlib/vaisdb drive 로 인계 (Opus direct) [blockedBy: C.1]
  target: ROADMAP.md 최종 업데이트, 다음 드라이브 (Phase 5.x stdlib 또는 6.x vaisdb) 시작 조건 명시
  [완료 기준]:
    - LANGUAGE_SPEC Matrix 에서 코어 언어 ✓ 달성 (SCOPED-maintained ◐ 는 예외 문서화)
    - compiler_syntax/std/living_spec/phase158 모두 100%
    - e2e 2622+ 유지
    - vaisdb 현재 숫자 그대로 (이 드라이브는 건드리지 않음)
    - 다음 드라이브 제안 (stdlib 확장 또는 vaisdb cleanup) ROADMAP 에 기록

progress: 4/11 (36%) — A.1, A.2, A.3, B.1 완료. B.6 신규 추가.

---

## Archive / 별도 드라이브

- **Phase 0 ~ 6.31 히스토리**: `ROADMAP-archive.md` 참조
- **stdlib 확장 (구 Phase 5.x)**: 이 드라이브 이후 별도 세션
- **vaisdb 24 failures (구 Phase 6.32/6.33)**: 컴파일러 완성 후 재측정 → cascade 확인 → 남은 blocker 로 새 Phase
- **vais-server/vais-web (구 Phase 7.x)**: 장기 별도 드라이브
- **문서/튜토리얼/생태계 (구 Phase 8.x)**: 장기 별도 드라이브
- **Phase 4.x 심층 구현** (linear borrow-checker full, macro expansion engine, yield coroutine desugar 등 500줄+): Survey B 에 의하면 기본 작동은 함. 심층 완성은 별도 집중 세션.

---

## Gate 기준

- **Phase A 완료 조건**: 3개 task ✓, LANGUAGE_SPEC Matrix 일치, 코드 변경 0 (문서만), baseline 유지.
- **Phase B 완료 조건**: 5개 task ✓, e2e 2624+ (2622 baseline + B.2 신규 2), 매트릭스 ◐ 감소 증명, baseline 유지.
- **Phase C 완료 조건**: 매트릭스 전수 재검증 완료, 100% 선언 가능.
- **즉시 revert 조건**: 각 task 후 `./scripts/check-integrity.sh` 에서 1-file regression 감지 시.
