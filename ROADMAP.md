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
iteration: 2
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

- [ ] B.1 — **Optional/Result lowering 근본 수정** (Opus direct) [blockedBy: A.3]
  **재정의 (2026-04-21)**: 원래 "TC pass ⇒ codegen pass 불변식 확립" 이었으나 research 결과
  silent drop 실제 케이스가 대부분 **하나의 뿌리** — Option/Result 의 lowering gap — 에서 기인
  한다고 확인. TC 거부 막기보다 뿌리 수정이 근본 해결.

  실측 증거 (A.1 + B.1 research 단계):
    - `F f(opt: Option<Struct>) -> Option<Primitive>` + match-arm 재포장: C004 "Aggregate extract index out of range" (A.1 3건)
    - `L { ... B Some(42) }` loop-break-값이 `Option<i64>`: C004 동일 증상
    - `EN Status { OK, Failed(i64) }` 커스텀 enum 은 동일 구조에서 **정상 동작** → Optional-family 전용 lowering 경로에 gap.
    - Vec<Struct>[i].field = (B.4) 는 별개 root cause (index-assign). B.4 는 유지.

  target: crates/vais-codegen/src/inkwell/ — Option/Result 가 (a) 함수 파라미터,
    (b) 함수 반환, (c) 변수 바인딩, (d) match-arm 재포장, (e) loop-break-값, (f) 저장/로드
    의 **모든 경로**에서 일관된 LLVM aggregate 레이아웃으로 lowering.

  approach:
    1. research-haiku 로 현재 Optional lowering 경로 진단 — 어느 위치 에서 "fat pointer" vs
       "aggregate struct" 불일치가 발생하는가?
    2. 진단 리포트 받은 뒤 Opus direct 로 수정 범위 결정 (단일 함수 / 모듈 단위 / 여러 파일).
    3. CLAUDE.md rule 3/4 엄수: 수정 전 baseline 기록, 수정 후 regression 1건 발생 시 즉시 revert.
    4. 각 단계마다 A.1 의 3개 LIVING_SPEC 파일 (phase2_10_*.vais) 로 build + run 검증.
    5. 성공 시 추가로 loop-break-Option 재현 케이스를 e2e 테스트로 추가.

  **Research 완료 (2026-04-21, 본 세션 iteration 4)**:
    Explore agent 진단 결과 근본 원인 확정:
    - `Option<T>` 의 type-level lowering 은 `{ i8, %T }` 로 정확 (conversion.rs:338-346).
    - 그러나 `Some()`/`Ok()`/`Err()` 생성자가 payload field 를 **하드코딩 `i64`** 로 설정
      (`crates/vais-codegen/src/inkwell/gen_expr/call.rs:183-246`). `self.coerce_to_i64(v)?`
      로 모든 payload 를 i64 로 강제 coerce. 구조체/Optional/Vec 등 aggregate payload 에서
      type-system 기대 (`{ i8, %Role }`) vs 실제 IR (`{ i8, i64 }`) 불일치 → LLVM verifier
      C004 "Aggregate extract index out of range".
    - `ResolvedType::Optional(Box<T>)` vs `Named("Option", [T])` 이중 표현이 coexist
      (crates/vais-types/src/inference/option_result_bridge.rs). 둘 다 `type_to_llvm` 에서는
      같은 `{ i8, T }` 로 resolve 되지만 constructor 에서 T 를 읽지 않음.

  **수정 범위 (다음 세션에서 진행)**:
    1. `crates/vais-codegen/src/inkwell/gen_expr/call.rs:183-246`
       — `Some`/`Ok`/`Err` 생성자가 expected-type (Option<T>/Result<T,E>) 을 context 에서
         읽어 `{ i8, %T }` 를 동적으로 빌드. `coerce_to_i64` fallback 제거.
    2. `crates/vais-codegen/src/inkwell/generator.rs`
       — 현재 expression 의 expected type 을 query 할 수 있는 helper 추가
         (TC 의 `expected_types` map 을 codegen 이 참조하거나 call-site 에서 전달).
    3. `crates/vais-codegen/src/inkwell/gen_match.rs:915-1095`
       — `Pattern::Variant` 의 field extraction 이 실제 struct layout 에 맞도록 index/type 결정
         (현재도 i64 고정 가정이 있다면 수정).

  **다음 세션 준비물 (/clear 후 복원 시)**:
    - ROADMAP.md 의 본 B.1 섹션 (이 블록) 을 읽고 시작.
    - 실측 reproducer: `docs/language/LIVING_SPEC/02_patterns/phase2_10_option_struct_rewrap.vais`
      (+ result_te, nested_option 2건). 현재 TC ✓ / Build ✗ C004. 수정 후 Build ✓ 목표.
    - Baseline: syntax=200, stages=14, std=82/82, vaisdb=237/261, phase158=18/18 (본 iteration 시작 시 실측).
    - CLAUDE.md rule 3/4 규칙 엄수.

  [완료 기준]:
    - A.1 reproducer 3건 (Option<Struct>/Result<T,E>/nested Option) 모두 TC + Codegen + Run ✓
    - loop-break 의 Option<i64>/Option<Struct> 케이스도 Codegen + Run ✓
    - docs/CODEGEN_FEATURES.md L171/L173 및 L175 해당 행 업데이트 (resolved 표시)
    - baseline 유지: syntax=200, std=82/82, vaisdb≥237/261, phase158=18/18
    - vaisdb cascade 측정 (+N 기대, 의무는 아님)
    - (선택) 새 e2e 테스트 2+ 추가 — loop-break-Option, Option<Struct> param round-trip

- [ ] B.2 — Phase 3.13 runtime 함수 구현 (impl-sonnet) [blockedBy: B.1]
  target: crates/vais-codegen/src/function_gen/runtime.rs + string_ops.rs + builtins dispatch
  approach: parse_f64, char_at 두 intrinsic 구현.
    - parse_f64: `strtod(const char*, NULL)` extern + Vais wrapper. Return `Result<f64, str>` 또는 `Option<f64>`.
    - char_at: 이미 TC OK. codegen 에서 `getelementptr i8, i8* %str, i64 %idx; load i8` 으로 구현.
  [완료 기준]:
    - 각각 e2e 테스트 2+ (정상 + edge case)
    - std 82/82 유지, e2e 2624+ (기존 2622 + 신규 2)
    - baseline regression 0
    - vaisdb cascade 측정 (+N 기대, 의무는 아님)

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

progress: 0/10 (0%)

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
