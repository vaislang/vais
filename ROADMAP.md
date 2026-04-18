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
  strategy: sequential (Phase 0.1 is foundational — design-impl inseparable, Opus direct)
  opus_direct: 0.1 — COMPILER_STAGES.md defines the contract for all later phases; can't be delegated.

### Phase 0 — Baseline & Integrity Matrix

- [x] 1. COMPILER_STAGES.md 작성 (Opus direct) ✅ 2026-04-19
  detail: `docs/COMPILER_STAGES.md` 작성. 6단계 정의 + 에러 코드 레지스트리 + 6-stage consolidated table + bash OK helpers + 10개 known bugs(B1-B10) 분류 및 target phase 매핑.
  changes: docs/COMPILER_STAGES.md (360 lines). Bash helper fns 실증 (tc/codegen/build/run 전부 expected exit code 일치).
  phase158: 18/18 green.
- [ ] 2. Integrity test matrix 스켈레톤 (impl-sonnet) [blockedBy: 1]
  detail: `crates/vaisc/tests/integrity/` 아래 3개 모듈 생성:
    - `compiler_syntax.rs` — 문법 포인트별 smoke (현재 최소 30개 stub, Phase 1에서 200+ 확장)
    - `compiler_stages.rs` — 각 stage gate (parse-only, tc-only, codegen-only, full-build)
    - `ecosystem_health.rs` — std/*.vais 개별 빌드 + vaisdb src/*.vais 개별 빌드 (baseline 측정용)
  완료 기준:
  - cargo test -p vaisc --test integrity 실행 가능
  - 각 테스트가 실패 시 어떤 파일의 어떤 단계가 실패했는지 출력
- [ ] 3. Baseline 측정 및 ROADMAP 기록 (Opus direct) [blockedBy: 2]
  detail: matrix 전체 실행하여 baseline 숫자 획득. ROADMAP `## Baseline (YYYY-MM-DD)` 섹션에 고정.
  완료 기준:
  - compiler_syntax: N/M, compiler_stages: N/M, ecosystem_health std: N/M, vaisdb: N/M 전부 기록
  - 모든 숫자가 실제 실행 결과와 일치
- [ ] 4. CI 스크립트 (impl-sonnet) [blockedBy: 2]
  detail: `scripts/check-integrity.sh` — 전체 matrix 실행, 어떤 테스트라도 실패 시 exit 1. Cargo alias `cargo integrity`.
  완료 기준:
  - 스크립트 실행 가능, Phase 0 baseline 상태에서 exit 0
  - Cargo alias 동작

### Phase 1 — 언어 문법 확정

- [ ] 5. LANGUAGE_SPEC.md 초안 (Opus direct) [blockedBy: 3]
  detail: `docs/LANGUAGE_SPEC.md` — Vais 전체 문법 레퍼런스. Single-char keyword, declarations (F/S/X/W/T/U/C/EN/O), statements (I/EL/L/LW/LF/M/R/B/C/D), operators, literals, types (primitives + Vec/HashMap/Option/Result/Tuple/Ref/RefMut/Slice/SliceMut/Pointer), patterns, closures. 각 construct의 grammar EBNF + 예제.
  완료 기준:
  - 800줄+, 모든 lexer keyword가 문서화
  - 각 construct에 `✓ 지원` 또는 `✗ 미지원 — 이유` 명시
  - CLAUDE.md의 기존 quick-ref와 일관
- [ ] 6. Parser 정합성 테스트 확장 (impl-sonnet) [blockedBy: 5]
  detail: `compiler_syntax.rs`를 30 → 200+로 확장. Spec의 모든 construct에 positive test 1개 + negative test 1개. LW, `} ! {` 같은 애매 문법 처리 결정 → 테스트에 반영.
  완료 기준:
  - 테스트 200+, 전체 green
  - 기존 phase158 18/18 green 유지
- [ ] 7. Lexer keyword 고정 + 에러 메시지 (impl-sonnet) [blockedBy: 6]
  detail: spec에 명시되지 않은 식별자가 keyword처럼 쓰이면 명확한 "not a keyword, did you mean X?" 에러. `partial`/`pure`/`partial` 등 최근 추가된 keyword는 spec에 포함.
  완료 기준:
  - 신규 keyword 모두 LEXER_KEYWORDS.md에 목록
  - negative test 동작 확인

### Phase 2 — Type system 정합성

- [ ] 8. Unification rules 문서화 (Opus direct) [blockedBy: 7]
  detail: `docs/TYPE_SYSTEM.md` — 모든 ResolvedType pair 조합의 unify 동작을 표로 명시. Phase 326 bridge (Named↔Optional/Result), auto-deref 규칙, generic instantiation 순서, fresh type var 생성 시점.
  완료 기준:
  - 500줄+, 주요 결정 사항 표 형태
  - 결정마다 현재 코드 위치 (파일:line) 참조
- [ ] 9. Cross-file impl dispatch 설계 & 구현 (Opus direct) [blockedBy: 8]
  detail: 현재 `S Parser`가 parser.vais에 있고 `X Parser { F parse_select }`가 parser_select.vais에 있을 때, parser.vais 단독 빌드 시 parse_select가 안 보이는 문제. 결정: (a) circular import를 benign cycle로 허용 vs (b) `#[extend]` 어노테이션 도입 vs (c) 모든 impl을 한 파일로 통합. 결정 근거 포함 문서화.
  완료 기준:
  - 결정사항 TYPE_SYSTEM.md에 기록
  - 구현 및 관련 테스트 추가
  - 기존 `test_circular_import_detection` 업데이트 또는 의도 명시
- [ ] 10. Option<&T> / Ref pattern binding 정합성 (Opus direct) [blockedBy: 9]
  detail: `Some(r) => r.field` 케이스에서 r이 &T / T 중 어느 것으로 bind되는지 결정하고 통일. role.vais `get_role_id` 같은 케이스가 정상 동작하도록.
  완료 기준:
  - 결정사항 문서화, reproducer 테스트 추가, 패스
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
