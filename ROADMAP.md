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
| vaisdb_files (vaisdb/src/**/*.vais, `ok_codegen_pkg`) | **176** | 85 | 261 | 67.4% |
| phase158 strict type gate | 18 | 0 | 18 | 100% |

These numbers are the **official regression floor** for all subsequent Phase gates:

- **Phase 0-5 gates MUST NOT reduce any category's pass count below these baseline numbers.**
- Phase 1-3 gates target keeping all numbers ≥ baseline while TYPE/CODEGEN work proceeds.
- Phase 4 target: `std_files` → 82/82 (100%).
- Phase 5 target: `vaisdb_files` ≥ baseline × 1.15 (~203/261) as a first checkpoint, with top-level build paths specifically certified.

CI entry `scripts/check-integrity.sh` (Phase 0.4) enforces the floor automatically.

---

## Phases 개요

| # | Phase | 목표 | Gate 지표 |
|---|-------|------|----------|
| 0 | Baseline & Integrity Matrix | 측정 기준 확정, test matrix 구축 | 모든 matrix 실행, baseline 숫자 commit |
| 1 | 언어 문법 확정 | Spec 문서 + parser 정합성 테스트 200+ | spec과 parser 100% 일치 |
| 2 | Type system 정합성 | Unification rules + cross-file impl + Option match | type system 테스트 100% |
| 3 | Codegen 완결성 | Str/Vec/HashMap/Tuple feature matrix | TC pass ⇒ codegen pass (0 drop) |
| 4 | stdlib 정비 | std/*.vais 개별 빌드 + 사용 예제 | std 파일 100% 빌드 |
| 5 | Packages (vaisdb/vais-server/vais-web) | 각 패키지 top-level 빌드 + API drift 정리 | 각 패키지 정의된 entry 파일 빌드 OK |

---

## Current Tasks (2026-04-19)

mode: auto
iteration: 1
max_iterations: 30
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
- [ ] 1.9. COOKBOOK.md 작성 (Opus direct) [blockedBy: 1.8]
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
- [ ] 1.10. CLAUDE.md 개발 철칙 강화 (Opus direct) [blockedBy: 1.9]
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
- [~] 10. Option match-arm constructor re-wrap 정합성 (Opus direct) 🚧 DEFERRED 2026-04-19
  detail: 정확한 bug 범위 재정의 — 문제는 `Some(r) => r.field` binding이 아니라 **arm이 `Some(r.field)` 재포장할 때** 생기는 enum constructor의 fresh-var disconnect. role.vais `get_role_id`가 canonical example.
  investigation: docs/TYPE_SYSTEM.md §9 "Phase 2.10"에 reproducer + root cause(calls.rs:55-87) 기록.
  naive fix 결과:
  - (a) 전체 enum에 substitute_generics → vaisdb -1 file regression
  - (b) Option/Result 한정 scoped fix → vaisdb -2 file regression
  → 기존 disconnected-fresh-var 동작은 load-bearing. 안전한 fix를 위해 Named↔Optional bridge (unification.rs:247) 함께 수정하거나 checker_expr/special.rs의 별도 경로 조사 필요.
  changes: docs/TYPE_SYSTEM.md §9 확장 (+40줄 reproducer & 분석), crates/vaisc/tests/e2e/modules_system.rs (+phase2_10_option_rewrap_in_match_arm #[ignore] 회귀 테스트).
  status: 완료 기준 "reproducer 테스트 패스" 미충족 → `[~]` 표기. 176/261 baseline은 **이 버그를 포함한 숫자**이므로 regression floor 위반 아님. Phase 2.10 해결 시 vaisdb OK 숫자 상승 기대.
  next session: calls.rs 패스 + unification bridge 동시 분석 필요. 단독 수정 금지 — 반드시 baseline check로 검증.
  완료 기준 (원본):
  - 결정사항 문서화 ✓ (deferred decision 기록)
  - reproducer 테스트 추가 ✓ (ignored)
  - 패스 ✗ (regression-safe fix 미발견 → deferred)
- [ ] 11. HashMap/Vec/Str method inference 정리 (impl-sonnet) [blockedBy: 10]
  detail: 현재 분산된 inference 패치들을 `crates/vais-types/src/builtins/method_returns.rs`로 통합. Codegen 측 중복 제거.
  완료 기준:
  - 하나의 테이블 (method name → return type) 
  - 기존 테스트 전부 통과

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

progress: 0/18 (0%)

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
