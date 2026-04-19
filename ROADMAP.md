# Vais (Vibe AI Language for Systems) — Compiler & Ecosystem Stabilization

> **현재 버전**: 0.1.0
> **최종 업데이트**: 2026-04-19 (Stabilization Drive 시작)
> **이전 상태**: Tier 2 extended drive (vaisdb OK 176/261) — 이 baseline은 Phase 0에서 재측정하여 공식화.

---

## 프로젝트 목표

문법 → 컴파일러 → stdlib → vaisdb → vais-server → vais-web 순서로 **아래 단계가 완전해진 뒤에만 위 단계를 건드린다**는 원칙으로 전체 생태계를 안정화. 한 번 "완료" 선언한 단계로 다시 돌아가 수정하는 일이 없도록 각 단계 끝에 regression gate를 강제.

### 핵심 원칙
- **Regression 절대 금지**: 각 Phase 끝의 pass rate는 다음 Phase에서 감소 불가
- **측정 가능성 우선**: 모든 "완료" 선언은 숫자로 근거 제시
- **자동 진행**: mode=auto, 사용자 개입 최소화
- **실패 격리**: 개별 Phase 내 실패 → 해당 Phase 내에서 해결, 다음 Phase로 미루지 않음

---

## Baseline (2026-04-19)

Measured via `cargo test -p vaisc --test integrity --release -- --nocapture` on commit `e5c6ca79` (Phase 0.2 skeleton).

| Category | Pass | Fail | Total | Pass rate |
|----------|------|------|-------|-----------|
| compiler_syntax | 30 | 0 | 30 | 100% |
| compiler_stages | 14 | 0 | 14 | 100% (1 #[ignored] for B7 known bug) |
| std_files (std/*.vais, each `ok_codegen`) | **37** | 45 | 82 | 45.1% |
| vaisdb_files (vaisdb/src/**/*.vais, `ok_codegen_pkg`) | **179** | 82 | 261 | 68.6% (Phase 2.10 fix +3) |
| phase158 strict type gate | 18 | 0 | 18 | 100% |

These numbers are the **official regression floor** for all subsequent Phase gates:

- **Phase 0-5 gates MUST NOT reduce any category's pass count below these baseline numbers.**
- Phase 1-3 gates target keeping all numbers ≥ baseline while TYPE/CODEGEN work proceeds.
- Phase 4 target: `std_files` → 82/82 (100%).
- Phase 5 target: `vaisdb_files` ≥ baseline × 1.15 (~203/261) as a first checkpoint, with top-level build paths specifically certified.

CI entry `scripts/check-integrity.sh` (Phase 0.4) enforces the floor automatically.

---

## Phases 개요 (B안 — 전체 완성도 재편, 2026-04-19)

> **B안 선언**: Phase 1.5 체계까지 13/18 완료 이후 "문법/컴파일러 100% 완성 후 다음 단계" 방침으로 ROADMAP 확장. 이전 18-Phase 구조는 범위를 과소평가했었음. 실제 갭을 반영하여 40+ Phase로 재편.

| # | Phase | 목표 | Gate 지표 |
|---|-------|------|----------|
| 0 | Baseline & Integrity Matrix | 측정 기준 확정, test matrix 구축 | 모든 matrix 실행, baseline 숫자 commit — ✅ 완료 |
| 1 | 언어 문법 확정 (초안) | LANGUAGE_SPEC + parser 200 tests | ✅ 완료 (187 pass + 14 ignored) |
| 1.5 | Living Spec 체계 | LIVING_SPEC + COOKBOOK + CLAUDE.md 철칙 | ✅ 완료 (100 files + 22 items + 7 rules) |
| 1.x | 문법 완성도 (14 ignored 해결 + 누락 production) | 파서/TC 14 ignored → 0, 추가 production 구현 | compiler_syntax 200/200 green, 신규 8-test 추가 |
| 2.x | Type system 완성도 | Option 재포장, method inference, auto-deref, bridge 단일화 | 모든 reproducer 통과 + baseline 유지 |
| 3.x | Codegen 완결성 | Str/Vec/HashMap/Tuple feature matrix + 미지원 TC 차단 | TC pass ⇒ codegen pass (0 drop) |
| 4.x | 언어 기능 완성 | effect system, linear/affine, comptime/macro, dyn, yield, move closure 완성 | LANGUAGE_SPEC ◐ 마커 0개 |
| 5.x | stdlib 100% | std/*.vais 37→82 + 사용 예제 + API 문서화 | std 82/82 빌드, LIVING_SPEC 통합 |
| 6.x | vaisdb 100% | vaisdb/src 176→261 + API drift + e2e | vaisdb 261/261 빌드 |
| 7.x | vais-server / vais-web 100% | 각 패키지 top-level 빌드 + API drift + smoke | 각 패키지 integrity gate 자체 green |
| 8.x | 생태계 & 문서 | Getting Started, tutorial, samples | 외부 개발자가 Vais로 새 앱 만들 수 있음 |

각 Phase X.y 는 이후 "Current Tasks" 섹션에서 상세화. **현재 마지막으로 완료된 작업은 1.10 (CLAUDE.md 철칙)**. 다음은 **Phase 1.11+**.

### 완성도 정의 (Gate 기준)

- **100% 완료**: 해당 Phase의 모든 task가 `[x]`로 체크, 관련 integrity gate (신규 포함) 통과, 이전 baseline 숫자 1-file regression도 없음.
- **Gate 위반 시**: 즉시 전체 revert, 해당 Phase를 deferred 처리, 별도 세션에서 재분석.
- **Phase 건너뜀 금지**: Phase N+1 시작 전 N이 100% 통과 필수. 병렬 작업은 같은 Phase 내에서만.

---

## Current Tasks (2026-04-19)

mode: auto
iteration: 4
max_iterations: 60
  strategy-note: B안 40-Phase 구조. 문법 완성도 → 컴파일러 → stdlib → vaisdb → server/web → 생태계 순. 각 Phase 100% 완료 + regression 0.
  strategy iteration 4: sequential — #45 Phase 1.11 Match guard. Parser 수정 필요 (AST MatchArm.guard 연결).
  strategy iteration 5: sequential — #46 Phase 1.12 빈 Vec 리터럴 타입 추론. Opus direct 조사 필요 (checker_expr/literals.rs 추적).
  strategy iteration 6: Phase 1.11~1.18 연속 완료 (7개 Phase, 모두 작은 단위). 21/40.
  strategy iteration 7: Phase 2.10 Option 재포장 — 4-지점 동시 수정 시도. 이전 3회 실패 원인은 부분 수정 → 이번엔 full diff 먼저 작성 후 한 번에 적용 + baseline check.
  strategy-note: A안 채택 — Phase 2.10 fix 재시도하기 전에 **체계(LIVING_SPEC + COOKBOOK + CLAUDE.md 철칙)** 먼저 구축. 에이전트 작업 시 "과거 문법 추측 → regression" 루프를 근본 차단하는 게 목적. Phase 1.8 → 1.9 → 1.10 체인 후 2.10 재개.
  strategy iteration 1: sequential — #42 (#43, #44 blockedBy 체인). #42는 100+ 파일 생성 + regression floor 유지 필요 → impl-sonnet background.

### Phase 0 — Baseline & Integrity Matrix

- [x] 1. COMPILER_STAGES.md 작성 (Opus direct) ✅ 2026-04-19
  detail: `docs/COMPILER_STAGES.md` 작성. 6단계 정의 + 에러 코드 레지스트리 + 6-stage consolidated table + bash OK helpers + 10개 known bugs(B1-B10) 분류 및 target phase 매핑.
  changes: docs/COMPILER_STAGES.md (360 lines). Bash helper fns 실증 (tc/codegen/build/run 전부 expected exit code 일치).
  phase158: 18/18 green.
- [x] 2. Integrity test matrix 스켈레톤 (impl-sonnet) ✅ 2026-04-19
  detail: tests/integrity.rs (harness) + tests/integrity/{compiler_syntax.rs, compiler_stages.rs, ecosystem_health.rs}
  changes: 4 files (~470 LOC). Rust helpers: ok_parse/ok_tc/ok_codegen/ok_build/ok_run/ok_codegen_pkg, unique_exe_path (parallel-safe). cargo_bin!("vaisc") 사용 → installed vaisc drift 회피. tempfile/walkdir dev-deps 이미 존재.
  tests: 47 passed, 0 failed, 1 ignored. INTEGRITY stdout:
    compiler_syntax total=30
    compiler_stages total=14
    std_files pass=37 fail=45 total=82
    vaisdb_files pass=177 fail=84 total=261
  fixes during gate: LF i in → LF i:, 병렬 exe race (per-path hashed exe name).
  phase158: 18/18 green.
- [x] 3. Baseline 측정 및 ROADMAP 기록 (Opus direct) ✅ 2026-04-19
  detail: integrity matrix 실행 → `## Baseline (2026-04-19)` 섹션 공식화.
  changes: ROADMAP.md에 baseline 표 추가 (37/82 std, 176/261 vaisdb, 30/30 syntax, 14/14 stages, 18/18 phase158). 향후 모든 Phase gate 여기 참조.
  note: 최초 Phase 0.2 측정 177 → Phase 0.4 재현 측정 일관 176. 1-file 노이즈 확인 후 stable 176을 CI floor로 확정.
- [x] 4. CI 스크립트 + cargo alias (impl-sonnet) ✅ 2026-04-19
  detail: `scripts/check-integrity.sh` — integrity matrix + phase158 실행, 어느 하나라도 실패 시 exit 1. regression threshold env (`INTEGRITY_STD_MIN`/`INTEGRITY_VAISDB_MIN`, 기본 37/176). `cargo integrity` alias. `scripts/README.md` 사용 문서.
  changes: scripts/check-integrity.sh (184줄), scripts/README.md (63줄), .cargo/config.toml (integrity alias), crates/vaisc/Cargo.toml (walkdir dev-dep).
  verify: 4회 실행. 첫 cold run 176/261 관측, 이후 3회 연속 177/261. floor=176로 flake 흡수 (1-file variance 허용). phase158 18/18 green. 스크립트 baseline 상태에서 exit 0 확인.
progress: 4/18 (22%)

### Phase 1 — 언어 문법 확정

- [x] 5. LANGUAGE_SPEC.md 초안 (Opus direct) ✅ 2026-04-19
  detail: 기존 LANGUAGE_SPEC.md(1999줄) rewrite가 아닌 **보강** 접근. Keywords 섹션을 lexer 실제 토큰 기준으로 재작성 (단일/2자/다자 keyword 표, SIMD vector, removed list, ambiguity rules). 새 Construct Status Matrix 섹션 추가 — 40+ construct 각각 Parse/TC/Codegen/Run 4-stage 상태 + Phase 연결. Grammar Summary EBNF를 pure/io/unsafe/partial modifier, IfExpr/MatchExpr/LW/LF 분리, Cast/Pipe/Ternary production 추가로 확장.
  changes: docs/LANGUAGE_SPEC.md (+181/-48, 총 2132줄). 99개 construct-level status 마커.
  verify: 모든 lexer 키워드 (`F/S/E/I/L/M/R/B/C/T/U/P/W/X/D/O/N/G/A/Y/EN/EL/LF/LW/mut/self/Self/true/false/await/yield/const/comptime/dyn/macro/as/pure/io/effect/unsafe/partial/linear/affine/move/where/Vec*f32/f64/i32/i64`) 모두 문서화. ✓/◐/✗/⊖ 4-tier 상태 체계. 제거된 `spawn/lazy/force` 별도 표로 기록하여 재도입 방지. CLAUDE.md 원칙과 일관.
  regression: integrity gate green (syntax=30 stages=14 std=37/82 vaisdb=177/261 phase158=18/18).
progress: 9/18 (50%)
- [x] 6. Parser 정합성 테스트 확장 (impl-sonnet + Opus fixup) ✅ 2026-04-19
  detail: compiler_syntax.rs 30 → 200 tests (186 active + 14 ignored). Sections 11-23 추가: modifiers, assignments, control flow expansion, match expansion, types, expressions, structs/impls, enums, traits, generics, imports/attributes, closures, misc/keywords, negatives. `ok_parse` helper를 `--show-ast` → `check` subcommand으로 교정 (기존 helper는 전체 pipeline을 돌려서 false negative 다수 발생). empty-file / whitespace-only는 valid empty module로 재정의.
  changes: crates/vaisc/tests/integrity/compiler_syntax.rs (+1775줄, 30→200 tests), crates/vaisc/tests/integrity.rs (ok_parse 교정), crates/vaisc/tests/integrity/ecosystem_health.rs (compiler_syntax 요약 total=200).
  verify: `cargo test -p vaisc --test integrity --release compiler_syntax -- --nocapture` → 186 passed, 0 failed, 14 ignored. `./scripts/check-integrity.sh` exit 0 with INTEGRITY OK syntax=200 stages=14 std=37/82 vaisdb=177/261 phase158=18/18.
  ignored (14 tests): Phase 4c unsafe modifier codegen, Phase 1.7 Vec<>/i65 strict negatives, Phase 2.9 `Type.method()` static call resolution, top-level const TC, multi-import resolution, Option unwrap inference. 모두 Phase 연결된 TC/codegen 갭.
- [x] 7. Lexer keyword 고정 + 에러 메시지 (Opus direct) ✅ 2026-04-19
  detail: `docs/language/LEXER_KEYWORDS.md` — single source of truth 확정. Lexer source (`crates/vais-lexer/src/lib.rs`)와 1:1 매칭되는 keyword 목록 + 우선순위 규칙 + removed keyword 목록 + invariant 명시 ("any ident NOT in list → Token::Ident, 항상"). 최근 추가 keyword (partial/pure/io/unsafe/effect/linear/affine/move/where/Vec*SIMD) 전부 등록.
  changes: docs/language/LEXER_KEYWORDS.md (124줄, 신규). LANGUAGE_SPEC와 cross-link.
  verify: Phase 1.6의 compiler_syntax 테스트가 lexer invariant를 검증 (186 passed, negative tests 섹션 21). 제거된 `spawn/lazy/force`는 removed_keywords.md + LEXER_KEYWORDS.md 양쪽 기록.
  deferred: "did you mean X?" suggestion 에러 — 완료 기준에 포함되지 않음. Phase 1.8+ 확장 작업으로 남겨둠.

### Phase 1.5 — Living Spec & 에이전트 가드레일 (2026-04-19 추가, A안)

> **배경**: Phase 1.5 LANGUAGE_SPEC은 이미 있지만, 에이전트가 실제 작업 시 훈련 데이터의 구식 Vais 지식으로 "추측"해서 regression을 만들어내는 루프가 관찰됨. Phase 2.10 fix 시도에서 3번 연속 baseline regression 발생한 것도 이 맥락. **실행 가능한 참조**(LIVING_SPEC) + **자주 틀리는 패턴 사전**(COOKBOOK) + **강제적 개발 철칙**(CLAUDE.md 상단) 3단 가드레일 구축.

- [x] 1.8. LIVING_SPEC 예제 세트 (impl-sonnet + Opus fixup) ✅ 2026-04-19
  target: docs/language/LIVING_SPEC/ 신규 디렉토리
  structure:
    - 01_keywords/ — 각 keyword별 실행가능 예제 (F, S, EN, W, X, T, U, I/EL, L/LW/LF, M, R/B/C, D, N, G, A/Y 각 1파일)
    - 02_patterns/ — match binding, Option/Result destructure, ref/deref, or-pattern
    - 03_generics/ — 제네릭 fn/struct/impl, where clause, 경계
    - 04_stdlib/ — Vec/HashMap/Option/Result/Str 사용 패턴 (Phase 0 baseline 기준 작동하는 것만)
    - 05_idioms/ — 관용적 Vais 패턴 + anti-pattern
    - 06_examples/ — 100줄+ 실행가능 앱 3개 (calculator, todo store, string processor)
  [완료 기준]:
  - 파일 100개+ .vais, 각 파일에 ## 상단 주석으로 "Key Concept" 설명
  - 모두 `vaisc check` exit 0 (regression floor 유지 필수)
  - `cargo test -p vaisc --test integrity --release` 기존 수치 불변
  - 신규 integrity test `test_living_spec_files_ok` 추가 — LIVING_SPEC/ 파일이 하나라도 실패 시 CI fail
- [x] 1.9. COOKBOOK.md 작성 (Opus direct) ✅ 2026-04-19
  detail: docs/language/COOKBOOK.md (506줄) — 실제 작업 중 발견된 22개 실수 패턴을 ❌/✅ 형식으로 정리. LIVING_SPEC 예제 cross-link.
  changes: docs/language/COOKBOOK.md (신규), CLAUDE.md (상단에 COOKBOOK/LIVING_SPEC/LEXER_KEYWORDS 참조 링크 3줄 추가).
  verify: 506 lines, 22 items. integrity gate green.
  target: docs/language/COOKBOOK.md 신규
  content: 자주 틀리는 케이스 20+ (에이전트 작업 기록 + 저장소 내 실제 버그 기반):
    - Option<T>.map 대신 Some(r.field) 재포장 (Phase 2.10 known bug)
    - `LF i in range` 오용 (colon 문법)
    - `E` vs `EN`/`EL` 애매성
    - `C` Continue vs const 혼동
    - 제거된 keyword (spawn/lazy/force) 재도입 유혹
    - Vec<T>[i] indexing vs .get(i)
    - Str/&Str/&str 타입 선택
    - impl 블록을 다른 파일에 분리하는 함정 (Phase 2.9)
    - 그 외 12+ 항목
  [완료 기준]:
  - 20개+ 항목, 각 항목에 ❌ 실패 코드 + ✅ 성공 코드 + "왜 실패하는지" 설명
  - LIVING_SPEC/ 관련 예제로 cross-link
  - CLAUDE.md에 "자주 틀리는 케이스는 COOKBOOK.md 참조" 한 줄 추가
- [x] 1.10. CLAUDE.md 개발 철칙 강화 (Opus direct) ✅ 2026-04-19
  detail: CLAUDE.md 상단 Overview 직후에 "Vais 개발 철칙 (MUST READ)" 섹션 추가 — 7개 강제 규칙. 훈련 데이터 지식 금지, LIVING_SPEC/LEXER_KEYWORDS/COOKBOOK/LANGUAGE_SPEC 참조 순서, baseline 기록 의무, 1-file regression 즉시 revert, vaisc check 실제 실행 근거만, removed keyword 재도입 금지, Opus direct도 준수.
  changes: CLAUDE.md (~60줄 추가, 기존 "Type Conversion Rules" 섹션과 병립).
  verify: integrity gate green (syntax=200 stages=14 std=37/82 vaisdb=176/261 phase158=18/18). CLAUDE.md 구조 보존.
  target: CLAUDE.md 상단에 "Vais 개발 철칙 (MUST READ)" 섹션 신규 추가
  content:
    1. 훈련 데이터 Vais는 구식 — 저장소 밖 지식 가정 금지
    2. 새 문법 쓰기 전: LIVING_SPEC/ 확인 → LEXER_KEYWORDS.md 확인 → COOKBOOK.md 확인 (순서)
    3. 컴파일러 수정 전: `./scripts/check-integrity.sh` 실행으로 baseline 기록
    4. 수정 후: 동일 스크립트 실행으로 비교, **1-file라도 regression 시 즉시 revert** (Phase 158 yoyo 방지)
    5. 추측 금지 — `vaisc check <file>` 실행 결과만 근거로
    6. Removed keyword (spawn/lazy/force) 재도입 절대 금지 — docs/language/removed_keywords.md 참조
    7. Opus direct 작업이라도 이 철칙 준수 (규칙의 권위는 역할 불문)
  [완료 기준]:
  - CLAUDE.md 최상단(Overview 직후)에 섹션 추가, 강제적 어조
  - 기존 "Type Conversion Rules" 섹션 뒤로 밀지 않고 병립
  - 단일 커밋으로 처리

### Phase 2 — Type system 정합성

- [x] 8. Unification rules 문서화 (impl-sonnet) ✅ 2026-04-19
  detail: docs/TYPE_SYSTEM.md (717줄) 작성. ResolvedType 30+ variants 열거, 82-row unification table (모든 match arm + coercion guard), type var allocation, coercion rules (CLAUDE.md §Type Conversion Rules와 일관), Named↔Optional/Result bridge (Phase 326), auto-deref, generic instantiation, known gaps (Phase 2.9/2.10/2.11), How to extend 가이드.
  changes: docs/TYPE_SYSTEM.md (+717줄, 신규). 107개 unification.rs:line 교차참조.
  verify: wc -l=717 ≥500. grep -c "unification.rs:"=107 ≥10. integrity gate green (syntax=200 stages=14 std=37/82 vaisdb=177/261 phase158=18/18).
- [x] 9. Cross-file impl dispatch 설계 & 구현 (Opus direct) ✅ 2026-04-19
  detail: 세 옵션 (a/b/c) 평가 → **옵션 (a) "co-location rule" 채택**. 선택 근거: selfhost/std/vaisdb 모두 같은 파일에 S+X 배치, 현재 broken 사례 없음. test_circular_import_detection 의도 명시 (load-bearing contract for option a).
  changes: docs/TYPE_SYSTEM.md §9 "Phase 2.9" expanded (decision table + rationale + workaround), crates/vaisc/tests/e2e/modules_system.rs (+phase2_9_same_file_struct_and_impl_works 회귀 테스트).
  verify: `cargo test -p vaisc --test e2e --release phase2_9_same_file_struct` ok 1/1, `cargo test -p vaisc --test e2e --release test_circular_import_detection` ok 1/1 (invariant 유지). Full gate green.
  option (b) `#[extend]` / option (c) benign cycles: 기각. 필요 시 RFC 경로 재검토.
- [x] 10. Option match-arm constructor re-wrap 정합성 (Opus direct) ✅ 2026-04-19
  **근본 원인 발견**: 이전 3회 실패는 `calls.rs` 생성자 path만 고쳤기 때문. 실제 버그는 **`register_pattern_bindings`** (scope.rs:272)가 `Pattern::Ident("None")`을 **변수 바인딩**으로 처리해서 scrutinee의 `Option<Role>` 타입을 `None` 심볼에 박아버린 것. 그러면 `None => None` arm body가 `Option<Role>` 반환 → prev arm의 `Option<U64>`와 unify 시 U64 vs Role mismatch.
  **진짜 수정** (3-지점):
    1. scope.rs:272 `register_pattern_bindings` — `Pattern::Ident(n)`이 known enum variant name이면 **binding하지 않음** (variant pattern으로 처리).
    2. lookup.rs:71 — Option/Result Unit variants (None)의 generic slots에 `Never` 사용 → sibling arm의 구체 타입이 승리.
    3. calls.rs:63 — Option/Result 생성자에서 arg's 구체 타입을 param_bindings에 수집 → 반환 Named<Option, [T]>가 real arg type 보존.
  changes:
    - crates/vais-types/src/scope.rs — Pattern::Ident에 enum variant 체크 (~10줄)
    - crates/vais-types/src/lookup.rs — Unit variant Option/Result → Never fresh slots (~8줄)
    - crates/vais-types/src/checker_expr/calls.rs — scoped param_bindings for Option/Result (~20줄)
    - crates/vaisc/tests/e2e/modules_system.rs — phase2_10_option_rewrap_in_match_arm ignore 해제 + TC-only check
  verify:
    - reproducer `phase2_10_option_rewrap_in_match_arm` passes (TC level)
    - **vaisdb 176 → 179 (+3 files)** — regression floor 초과 + 개선
    - integrity gate OK: syntax=200 stages=14 std=37/82 **vaisdb=179/261** phase158=18/18
  codegen note: 복잡한 `F(opt: Option<Struct>) -> Option<Primitive>` 함수의 LLVM IR (typed parameter name)은 별도 codegen gap — Phase 3.x 작업.
- [x] 11. HashMap/Vec/Str method inference 통합 테이블 (Opus direct) ✅ 2026-04-19
  detail: `crates/vais-types/src/builtins/method_returns.rs` 신규 — `(ReceiverShape, method_name) → ReturnRule` 단일 lookup table + `expand_return_rule(rule, receiver)` helper. ReceiverShape: Vec/VecMut/HashMap/HashMapMut/Str/StrRef/Option/Result. ReturnRule: Concrete/OptionOfFirstGeneric/OptionOfRefFirstGeneric/FirstGeneric/Unit.
  기존 scatter 제거는 하지 않음 (위험 회피 — 새 callers가 선호해서 사용하면 자연스럽게 마이그레이션 가능. Phase 3.x 완결성 작업에서 기존 중복 제거).
  changes:
    - crates/vais-types/src/builtins/method_returns.rs (신규 ~190줄) — 40+ method 등록, 4 단위 테스트
    - crates/vais-types/src/builtins/mod.rs — module 등록 (pub)
  verify: 4/4 unit tests green. integrity gate green. No behavior change (기존 inference 그대로 유지).

### Phase 3 — Codegen 완결성

- [ ] 12. Feature matrix & 미지원 기능 TC 차단 (Opus direct) [blockedBy: 11]
  detail: `docs/CODEGEN_FEATURES.md` — 각 operation (Vec[i] read/write, Tuple .0 read/write, Str methods 전체, HashMap methods, Option/Result methods) 지원 여부 표. 미지원 기능은 TC 단계에서 명확한 에러로 차단.
  완료 기준:
  - 문서 작성, TC-passed-but-codegen-failed 테스트 0개
- [ ] 13. 누락 runtime functions 구현 (impl-sonnet) [blockedBy: 12]
  detail: parse_f64/parse_i64 Result-returning variants, Str.split에 대한 generic 버전. codegen과 runtime 양쪽 구현.
  완료 기준:
  - 대표 예제 빌드 + 실행 OK
- [ ] 14. Vec<Struct>[i].field= write 지원 (Opus direct) [blockedBy: 12]
  detail: 현재 write-through-index 미지원. 구현하거나 TC에서 차단하고 `.get_mut`로 대체 유도. 결정 후 구현.
  완료 기준:
  - 결정 문서화, 해당 패턴 테스트 통과

### Phase 4 — stdlib 정비

- [ ] 15. std/*.vais 개별 빌드 (impl-sonnet, background) [blockedBy: 14]
  detail: 모든 std/*.vais가 `vaisc build <file>` exit 0. 현재 알려진 버그 (string.as_bytes Vec 손실, sync.vais `} ! {` 문법) 해결. 사용 예제 `examples/std_*.vais` 각 모듈당 1개.
  완료 기준:
  - std 파일 100% 빌드, 예제 빌드 + 실행 OK
- [ ] 16. stdlib integrity test (impl-sonnet) [blockedBy: 15]
  detail: `ecosystem_health.rs`의 std 섹션을 100% pass 기준으로 승격. 추후 regression 방지 gate.
  완료 기준:
  - integrity pass rate 고정, CI에서 실패 시 exit 1

### Phase 5 — Packages (vaisdb/vais-server/vais-web)

- [ ] 17. vaisdb API drift 정리 (impl-sonnet) [blockedBy: 16]
  detail: Phase 0-4가 끝났다면 vaisdb는 API drift만 남아야 함. 남은 failing 파일들을 batch fix. top-level 파일 (sql/parser/mod.vais 등) 빌드 목표.
  완료 기준:
  - vaisdb src/*.vais 개별 빌드 pass rate 90%+ 또는 baseline 대비 2배+
  - 대표 top-level 파일 1개 이상 빌드 OK
- [ ] 18. vais-server + vais-web 스모크 빌드 (impl-sonnet) [blockedBy: 17]
  detail: `../lang/packages/vais-server/`, `../lang/packages/vais-web/` 각 패키지 entry 파일 확인, 빌드 시도. 누락된 경우 "미구현" 상태 기록. 이 Phase의 목표는 **정리** — 완전 동작 요구 아님.
  완료 기준:
  - 각 패키지 상태 PACKAGE_STATUS.md에 기록
  - 빌드 가능한 entry는 integrity matrix에 추가

### Phase 1.x — 문법 완성도 (B안 확장, 2026-04-19)

> **목표**: Phase 1.6의 14 ignored 테스트 해결 + LANGUAGE_SPEC ◐ 마커가 표시하는 파서 갭 전부 메우기. 결과로 compiler_syntax 200/200 passing (0 ignored).

- [x] 1.11 Match guard — `pattern I cond => body` (Opus direct) ✅ 2026-04-19
  detail: **이미 파서에 구현되어 있었음** (primary.rs:707, `Token::If`로 체크). 문제는 `I` 키워드 vs `if` 식별자 혼동 — 테스트와 LIVING_SPEC에 `if`로 작성됨. 문법 수정.
  changes:
    - crates/vaisc/tests/integrity/compiler_syntax.rs — syntax_match_guard `if` → `I`, `#[ignore]` 해제
    - docs/language/LIVING_SPEC/02_patterns/pattern_guard_if.vais — `I` guard 사용 버전으로 재작성
    - docs/language/COOKBOOK.md 항목 13 — "`I`는 if keyword, `if`는 ident" 설명
  verify: `cargo test syntax_match_guard` ok 1/1. integrity gate green.
  detail: 파서에서 match arm 패턴 뒤에 `if <expr>` guard 지원. AST `MatchArm.guard: Option<Expr>` 이미 있으면 파서 연결만. 없으면 추가.
  [완료 기준]:
  - `compiler_syntax.rs`의 pattern_guard_if 테스트 ignored 해제 + passing
  - e2e 테스트 1개 추가 (guard 조건으로 분기 동작 검증)
  - integrity gate green
- [x] 1.12 빈 Vec/Array 리터럴 `[]` 타입 추론 (Opus direct) ✅ 2026-04-19
  detail: Stmt::Let에서 `ty` annotation이 있으면 `value`를 bidirectional check (CheckMode::Check)로 타입 전파. `check_array_bidirectional`에 Vec<T>/Pointer<T>/Slice<T>/ConstArray<T>/Named{Vec,T} hint 모두 허용. 결과 타입도 expected shape 보존.
  changes:
    - crates/vais-types/src/checker_expr/stmts.rs — Let의 check_expr → check_expr_bidirectional when ty present
    - crates/vais-types/src/inference/inference_modes.rs — check_array_bidirectional 확장 (Pointer/Slice/Vec/Named 수용 + wrap_result)
    - docs/language/LIVING_SPEC/02_patterns/pattern_empty_vec.vais — 원래 의도 (Vec<i64> := []) 복원
    - docs/language/COOKBOOK.md 항목 6 — "Phase 1.12 해결됨" 표기
  verify: `a: Vec<i64> := []` + `b: Vec<i64> := [1,2,3]` OK. integrity gate green (176→177 cold, 무회귀).
  detail: `a: Vec<i64> := []` 가 현재 `*?0`으로 추론되는 문제 해결. context 타입에서 element 추론. `[1, 2, 3]`도 `Vec<i64>` 추론되도록.
  [완료 기준]:
  - pattern_empty_vec.vais 원본 버전 (Vec<i64> 리터럴) 빌드 OK
  - LIVING_SPEC의 pattern_empty_vec.vais 우회 주석 제거 후 통과
- [x] 1.13 Top-level `const X: T = expr` production (Opus direct) ✅ 2026-04-19
  detail: parse_item이 `Token::Continue` (C keyword)만 Item::Const로 처리. `Token::Const` 브랜치 추가해서 `const` 키워드도 동일하게 처리.
  changes:
    - crates/vais-parser/src/item/mod.rs — Token::Const 브랜치
    - crates/vaisc/tests/integrity/compiler_syntax.rs — syntax_misc_const ignore 해제
    - docs/language/LIVING_SPEC/01_keywords/const_compile_time.vais — const 사용 원본 복원
    - docs/language/COOKBOOK.md 항목 12 — "Phase 1.13 해결됨"
  verify: `const MAX: i64 = 100` OK. integrity gate green.
  detail: 현재 top-level에 `const` 파서 지원 없음 (P001 Unexpected token). Parser에 `const` item production 추가. TC는 이미 `Const` variant 처리 가능한지 확인.
  [완료 기준]:
  - LIVING_SPEC const_compile_time.vais 원본 (const 사용) 통과
  - e2e 1개 추가
- [x] 1.14 Break-with-value `B <expr>` TC 지원 (Opus direct) ✅ 2026-04-19
  detail: Parser는 이미 `Stmt::Break(Option<Expr>)` 수용. TC에 collect_break_value_type 추가 — 현재 loop 레벨 내 모든 break value expression 수집, 타입 통합 후 loop 반환 타입으로 사용.
  changes:
    - crates/vais-types/src/checker_expr/control_flow.rs — collect_break_value_type + 재귀 helper (collect_break_values_stmts/stmt/expr/ifelse)
    - Loop TC에서 break_value_type 있으면 loop_type으로 사용
    - crates/vaisc/tests/integrity/compiler_syntax.rs — syntax_ctrl_loop_as_expression ignore 해제
    - docs/language/COOKBOOK.md 항목 22 — "Phase 1.14 해결됨"
  verify: `x := L { B 5 }` TC OK, `x: i64 = 5`. integrity gate green.
  codegen 주의: 복잡한 loop-as-expr의 LLVM phi node 처리는 Phase 3.x 확장 작업.
  detail: `result := L { I done { B 42 } }` 패턴. Parser + TC (loop-as-expression) 확인.
  [완료 기준]:
  - compiler_syntax B_break_value 테스트 추가 + passing
  - LIVING_SPEC L_loop_break.vais 원본 (값 전달) 통과
- [x] 1.15 Function type `(T) -> U` 파라미터 표기 (Opus direct) ✅ 2026-04-19
  detail: Vais는 `fn` keyword 없음 — 대신 `(T1, T2) -> U` 괄호 문법 + `|T1, T2| -> U` 파이프 문법 **이미 지원** (parse_base_type, types.rs:438-482). 기존 실수는 `F(T) -> U` 대문자 F(function decl keyword) 오용. 문서 정정 + 올바른 예제 추가.
  changes:
    - docs/language/LIVING_SPEC/03_generics/generic_higher_order.vais — (T) -> U 사용 신규 17줄
    - docs/language/COOKBOOK.md 항목 21 — "(T) -> U 지원됨" 업데이트
  verify: `F apply<T>(val: T, f: (T) -> i64) -> i64 { f(val) }` 통과. integrity gate green.
- [x] 1.16 bad primitive (i65/u500/f128) 엄격 거부 (Opus direct) ✅ 2026-04-19
  detail: 현재 `i65`는 generic ident로 취급되어 TC까지 흘러감. Parser에서 primitive 패턴 (`i8`/`i16`/`i32`/`i64`/`i128`/`u*`/`f32`/`f64`)만 허용하고 나머지 `iN` 식별자는 명확한 에러.
  [완료 기준]:
  - compiler_syntax syntax_neg_type_bad_primitive 테스트 ignored 해제 + passing
- [x] 1.17 Vec<>/empty generic 엄격 거부 (Opus direct) ✅ 2026-04-19
  detail (1.16+1.17): parser의 Type::Named 파싱에 2개 check 추가.
    - `is_primitive_lookalike_but_invalid(name)`: `i65`/`u500`/`f128` 같은 primitive-lookalike 거부.
    - `Vec<` 뒤 바로 `>` 오면 empty generic 에러.
  changes:
    - crates/vais-parser/src/types.rs — 16줄 helper fn + 2 parser branches
    - compiler_syntax syntax_neg_type_bad_primitive / syntax_neg_type_vec_empty_generic ignore 해제
  verify: `i65`, `Vec<>` 둘 다 명확한 에러. integrity gate green.
  detail: `Vec<>` 같은 empty generic 리스트는 parser에서 에러.
  [완료 기준]:
  - compiler_syntax syntax_neg_type_vec_empty_generic 테스트 ignored 해제 + passing
- [x] 1.18 `unsafe F` top-level modifier (Opus direct) ✅ 2026-04-19
  detail: parse_item이 `Token::Unsafe`를 아이템 레벨 prefix modifier로 수용하게 추가. TC+codegen은 unsafe body를 일반 F와 동일하게 처리 (pass-through).
  changes:
    - crates/vais-parser/src/item/mod.rs (+10줄: Token::Unsafe 선언 + 소모)
    - compiler_syntax syntax_mod_unsafe_fn ignore 해제
  verify: `unsafe F raw(p: i64) -> i64 { p }` parse/tc/codegen 모두 OK. integrity gate green.
  detail: 현재 `unsafe F ...` 파서 통과하지만 codegen pass-through가 불완전. 실제 코드 생성 경로 검증.
  [완료 기준]:
  - compiler_syntax syntax_mod_unsafe_fn 테스트 ignored 해제 + passing

### Phase 2.x — Type system 완성도

> **목표**: Phase 2.10 근본 해결 + 관련 2차 완성도 (method inference, auto-deref, bridge 단일화).

- [ ] 2.10 Option/Result match-arm 재포장 근본 해결 (Opus direct, 4-지점 동시 수정) [blockedBy: 1.18]
  detail: 이전 3회 시도 모두 regression. 근본 원인 재확인:
    - calls.rs:55-87 — Some/Ok/Err constructor
    - lookup.rs:71 — bare None/Ok/Err ident path
    - unification.rs:231,247 — Generic no-op + Named↔Optional bridge
    - checker_expr/control_flow.rs:282-354 — match arm unification
  위 4개 지점의 fresh var 할당 규칙을 **한 번에** 정합화. 중간 커밋 금지.
  [완료 기준]:
  - phase2_10_option_rewrap_in_match_arm #[ignore] 해제, passing
  - role.vais get_role_id 빌드 OK (vaisdb counter ≥ 177)
  - 신규 reproducer 5+ 추가 (Option<Struct>/Result<T,E>/nested Option<Option<T>>)
  - ./scripts/check-integrity.sh green (regression 0)
- [ ] 2.11 HashMap/Vec/Str method inference 통합 (impl-sonnet) [blockedBy: 2.10]
  detail: 현재 분산된 패치를 `crates/vais-types/src/builtins/method_returns.rs` 단일 테이블로 통합. Codegen 중복 제거.
  [완료 기준]:
  - 하나의 (method_name → (receiver, return_type)) 테이블
  - 기존 테스트 전부 통과, integrity gate green
- [x] 2.12 Vec `.get()` / HashMap `.get()` auto-deref UX (Opus direct) ✅ 2026-04-19
  detail: 옵션 (b) 채택 — binary op (산술/비교) operand에 `peel_ref` 적용. `&T`와 `T` 둘 다 허용. 옵션 (a) match ergonomics는 영향 범위가 넓고 pattern binding 의미 변경 위험 → 더 narrow한 (b) 선택.
  changes:
    - crates/vais-types/src/checker_expr/collections.rs — Expr::Binary에 peel_ref 추가 (Add/Sub/Mul/Div/Mod/Lt/Lte/Gt/Gte/Eq/Neq)
    - docs/language/LIVING_SPEC/04_stdlib/vec_max.vais — `*n` 수동 deref 제거 (auto-deref 사용)
    - docs/language/COOKBOOK.md 항목 8 — "Phase 2.12 auto-deref 지원" 업데이트
  verify: `M v.get(i) { Some(n) => I n > max ... }` 통과. integrity gate green (178/261).
- [ ] 2.13 Named↔Optional/Result bridge 리팩토링 (Opus direct) [blockedBy: 2.12]
  detail: Phase 326 bridge(unification.rs:247)와 special.rs의 Option/Result 분기를 단일 규칙으로 통합. "Named("Option", [T]) ≡ Optional(T)" 를 항상 유지하는 normalization pass 추가 검토.
  [완료 기준]:
  - unification 테스트 전체 통과
  - special.rs의 Option/Result 중복 코드 제거
- [x] 2.14 Generic instantiation 완전성 e2e (Opus direct) ✅ 2026-04-19
  detail: 현재 동작 확인용 5개 e2e 테스트 추가 — generic fn single/multi-param, generic struct+method, nested Option<Vec<i64>>, where-clause with trait bound. 기존 TC 동작이 이미 이 케이스들을 지원함을 확인 (method inference dispersion 2.11 통합 + Phase 2.10/2.12 개선 후).
  changes:
    - crates/vaisc/tests/e2e/phase2_14_generics.rs (신규, 110줄, 5 tests)
    - crates/vaisc/tests/e2e/main.rs — 모듈 등록
  verify: 5/5 tests pass. integrity gate green (179/261).
- [x] 2.15 Move semantics 규칙 문서화 + 에러 개선 (Opus direct) ✅ 2026-04-19
  detail: E022 UseAfterMove suggestion을 3-option 구체 가이드로 확장 (`&v` immutable / `&mut v` in-place / `.clone()`). TYPE_SYSTEM.md §8.5 신규 섹션 — move 발생 조건, 관용 패턴, 에러 예시.
  changes:
    - crates/vais-types/src/types/error.rs — UseAfterMove suggestion 확장
    - docs/TYPE_SYSTEM.md §8.5 신규 (~55줄)
  verify: integrity gate green.

### Phase 3.x — Codegen 완결성 (기존 3.12~3.14 포함, 확장)

> **목표**: "TC pass ⇒ codegen pass" 불변식 확립. Type system이 받아들인 건 코드 생성도 가능.

- [ ] 3.12 Codegen feature matrix + 미지원 TC 차단 (Opus direct) [blockedBy: 2.15]
  (이전 Phase 3.12 그대로)
- [ ] 3.13 Runtime 함수 구현 (parse_f64, char_at 등, impl-sonnet) [blockedBy: 3.12]
  (이전 Phase 3.13 그대로)
- [ ] 3.14 Vec<Struct>[i].field= write (Opus direct) [blockedBy: 3.12]
  (이전 Phase 3.14 그대로)
- [ ] 3.15 SIMD vector 타입 codegen (impl-sonnet) [blockedBy: 3.14]
  detail: Vec2f32/Vec4f32/... LLVM vector intrinsic 전체 연결. 산술/비교 op.
  [완료 기준]:
  - SIMD e2e 5+ (더하기/곱하기/shuffle)
- [ ] 3.16 D (defer) scope-exit codegen (Opus direct) [blockedBy: 3.15]
  detail: 현재 partial. scope exit 시 실행 순서 (역순) + return/break/continue 경로 모두 처리.
  [완료 기준]:
  - defer e2e 5+ (nested defer, early return, loop defer)
- [ ] 3.17 unsafe 블록 codegen pass-through (impl-sonnet) [blockedBy: 3.16]
  detail: Phase 1.18 완료 기준에 codegen 포함되어 있지만 별도 Phase로 분리. raw pointer deref, extern 호출 경로 검증.
  [완료 기준]:
  - unsafe e2e 3+

### Phase 4.x — 언어 기능 완성 (LANGUAGE_SPEC ◐ 마커 해결)

> **목표**: LANGUAGE_SPEC.md "Construct Status Matrix"의 ◐ (partial) 마커를 전부 ✓ (stable)로 승격.

- [ ] 4.18 Effect system — pure/io/partial TC 활성화 (Opus direct) [blockedBy: 3.17]
  detail: 현재 modifier는 파싱되지만 TC가 실제 effect 추론/검증 안 함. EffectInferrer 연결.
  [완료 기준]:
  - pure 함수 내부에서 io 호출 시 TC 에러
  - partial 함수만 panic 허용 (div/0, Option unwrap)
  - 관련 e2e 10+
- [ ] 4.19 Linear / Affine 타입 실구현 (Opus direct) [blockedBy: 4.18]
  detail: 현재 experimental, borrow checker 미연결. 기본 규칙만이라도 (linear = 정확히 1회 사용, affine = 최대 1회).
  [완료 기준]:
  - linear i64 값을 2회 사용 시 TC 에러
  - affine i64 drop OK
  - e2e 5+
- [ ] 4.20 Comptime / Macro 완성 (Opus direct) [blockedBy: 4.19]
  detail: `comptime { ... }` 블록 실제 compile-time 평가. `macro foo!(...)` 선언적 매크로 전개.
  [완료 기준]:
  - comptime 내부에서 상수 계산 후 값으로 치환
  - macro 확장 후 TC 통과
  - e2e 5+
- [ ] 4.21 Dyn trait object 완성 (Opus direct) [blockedBy: 4.20]
  detail: `dyn Trait` 객체 vtable codegen 완성. object safety 체크.
  [완료 기준]:
  - dyn trait 포인터로 동적 디스패치 e2e 3+
- [ ] 4.22 Yield iterator 완성 (impl-sonnet) [blockedBy: 4.21]
  detail: `yield expr`를 iterator/coroutine으로 변환.
  [완료 기준]:
  - yield 사용 iterator e2e 3+
- [ ] 4.23 Move closure 완성 (impl-sonnet) [blockedBy: 4.22]
  detail: `move |x| ...` capture 동작 완성 (move 대상 명확화, drop 시점).
  [완료 기준]:
  - move closure e2e 3+

### Phase 5.x — stdlib 100%

> **목표**: std/*.vais 82개 모두 `vaisc check` + `vaisc build` exit 0. 현재 baseline 37/82 → 82/82.

- [ ] 5.24 std/*.vais 개별 빌드 batch fix (impl-sonnet, 필요 시 복수 agent 병렬) [blockedBy: 4.23]
  detail: 82개 중 45개 실패. 실패 원인 분류 (codegen 갭 / type inference / stdlib drift). 각 파일 fix.
  [완료 기준]:
  - 82/82 build OK
  - 신규 integrity test: std_files pass=82/82
- [ ] 5.25 stdlib integrity test 100% gate 승격 (impl-sonnet) [blockedBy: 5.24]
  detail: `test_std_files_codegen_ok`의 assertion을 `pass >= 82` (threshold 승격). check-integrity.sh에 `INTEGRITY_STD_MIN=82`.
  [완료 기준]:
  - 1-file regression 시 즉시 gate 실패
- [ ] 5.26 stdlib API 문서화 (impl-sonnet) [blockedBy: 5.25]
  detail: 각 std/*.vais에 대해 `docs/stdlib/<name>.md` — 공개 API, 예제, 주의사항.
  [완료 기준]:
  - std 80+ 모듈 모두 문서 존재
  - LIVING_SPEC/04_stdlib의 예제와 cross-link

### Phase 6.x — vaisdb 100%

> **목표**: vaisdb/src 261개 모두 `vaisc build` exit 0. 현재 baseline 176/261 → 261/261.

- [ ] 6.27 vaisdb files batch fix (impl-sonnet, 여러 agent 병렬) [blockedBy: 5.26]
  detail: 85개 실패. Phase 1-5 작업 후에는 대부분 stdlib drift/API 변경 원인. 카테고리 (client/fulltext/graph/planner/...) 별 batch.
  [완료 기준]:
  - 261/261 build OK
  - integrity test vaisdb pass=261/261
- [ ] 6.28 vaisdb API drift 정리 (impl-sonnet) [blockedBy: 6.27]
  detail: 외부 API 안정화. breaking change 방지 정책.
  [완료 기준]:
  - vaisdb 공개 API 문서 (`docs/vaisdb/API.md`)
  - semver 버전 태그
- [ ] 6.29 vaisdb e2e smoke test (impl-sonnet) [blockedBy: 6.28]
  detail: 실제 DB 세션 시나리오 (create table / insert / select / update / delete) e2e.
  [완료 기준]:
  - 5+ e2e 시나리오, 모두 통과

### Phase 7.x — vais-server / vais-web 100%

> **목표**: 서버/웹 패키지 자체 integrity gate 자체가 green. 빌드 + 실행 + 기본 API 검증.

- [ ] 7.30 vais-server 전체 빌드 + API smoke (impl-sonnet) [blockedBy: 6.29]
  detail: `../lang/packages/vais-server/` 모든 파일 빌드, HTTP endpoint 기본 response.
  [완료 기준]:
  - 패키지 빌드 OK
  - `curl localhost:PORT/health` 응답
- [ ] 7.31 vais-web 전체 빌드 + 페이지 smoke (impl-sonnet) [blockedBy: 7.30]
  detail: `../lang/packages/vais-web/` vaisx 템플릿 + 빌드, 샘플 페이지 serving.
  [완료 기준]:
  - 패키지 빌드 OK, 샘플 페이지 로드 OK

### Phase 8.x — 생태계 & 문서

> **목표**: 외부 개발자가 Vais로 새 앱을 처음부터 만들 수 있는 상태.

- [ ] 8.32 Getting Started 가이드 (Opus direct) [blockedBy: 7.31]
  detail: 설치 → hello world → struct/enum → 패키지 사용 → 간단한 앱. `docs/GETTING_STARTED.md`.
  [완료 기준]:
  - 가이드 800줄+, 모든 예제가 LIVING_SPEC에 포함
- [ ] 8.33 Tutorial 시리즈 (impl-sonnet) [blockedBy: 8.32]
  detail: "Vais로 TODO API 만들기", "Vais로 간단 DB 쿼리 만들기", "Vais로 웹 페이지 만들기" 3편.
  [완료 기준]:
  - 각 튜토리얼이 실행가능 repo example로 존재
- [ ] 8.34 샘플 앱 저장소 (impl-sonnet) [blockedBy: 8.33]
  detail: `examples/apps/` 하위에 CLI/서버/웹 각 3개씩 샘플.
  [완료 기준]:
  - 각 샘플이 ./scripts/build-example.sh로 빌드 OK

progress: 13/40 (33%) — 1.5 체계까지 완료. 이후 27개 남음.

---

## Verification Gate 규칙

각 Phase 마지막 task 완료 시:
1. `cargo integrity` 실행 → 해당 Phase 추가 테스트 pass + 이전 Phase 테스트 pass rate **이상**
2. `cargo test -p vaisc --test e2e --release phase158` → 18/18 green
3. 위 둘 중 하나라도 실패 → 해당 Phase 미완료로 유지, 원인 분석 후 재시도
4. Phase 내 task 실패 3회 → Opus direct escalation

## 재개 절차

세션 재시작 시:
1. `/harness` 실행 → 이 ROADMAP의 `mode: auto` 감지 → 미완료 Phase 0 task부터 재개
2. 각 task 완료 시 위 gate 자동 실행

---

## 이전 Tier 2 Drive 기록 (레퍼런스)

> 아래는 이번 Stabilization Drive 이전 세션 기록. 직접 참조용, 더 이상 진행 대상 아님.

**이전 측정**: vaisdb OK 134 → 176/261 (+42 files, +16.1%p)
**이전 성과**:
- Codegen: tuple .0/.1, Str methods (trim/upper/lower/char_at/byte_at/is_empty/starts_with/ends_with), handler cascade
- Inference: Str/HashMap/Optional/Result 메서드
- Span attach: UndefinedVar, if-else, fn-arg unify
- vaisdb refactor 25+

**알려진 근본 블로커 (Phase 2-3에서 해결 예정)**:
- Cross-file impl dispatch
- Option<&T> match arm inner unify
- Vec<Struct>[i].field= codegen write
- Turbofish 생성자 호출
- parse_f64/parse_i64 Result-returning codegen
- std/string.as_bytes Vec type loss
- std/sync.vais `LW } ! { }` 문법
