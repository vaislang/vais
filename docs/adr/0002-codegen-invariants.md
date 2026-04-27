# ADR 0002 — Codegen Invariants (4 클래스) + AI Multi-Session Protocol

**Status**: Accepted (iter 80, 2026-04-26 — 사용자 명시 승인)
**Decision driver**: Phase Ω Pillar 1.0. ADR 0001 §1이 정의한 "근본 fix" 게이트 (R1+R2+R3)를 codegen 4 클래스에 구체화한다. **AI multi-session 협업 안전성도 본 ADR scope** — 기계 검증 가능한 audit + Self-Audit Checklist + Anti-Patterns + Iter Entry Spec + Rollback Trigger.
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

#### R3 — Same-Class Audit (기계 검증)
- 통합 helper: `coerce_ret_value` (iter 74 7cfc5caf 신설)
- audit 명령:
  ```bash
  grep -rn '"\s*ret ' crates/vais-codegen/src/ \
    | grep -v 'coerce_ret_value\|test\|//\|Binary file' \
    | wc -l
  ```
- **baseline (iter 80)**: 152 사이트
- **Phase Ω 종료 목표**: 0 (모든 ret emit이 `coerce_ret_value` 경유)
- audit 의무: 신규 ret emit 사이트 추가 시 `coerce_ret_value` 호출 확증. 매 P1.x iter 종료 시 baseline 카운트 갱신

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

#### R3 — Same-Class Audit (기계 검증)
- 통합 helper 후보: `resolve_index_access(arr_ty) -> (elem_llvm, AccessKind, elem_resolved)` (P1.3에서 신설 예정)
- audit 명령:
  ```bash
  # GEP emit 사이트 (4 path: data read / simple write / compound write / inkwell)
  grep -rn 'getelementptr' crates/vais-codegen/src/ \
    | grep -v 'test\|//\|Binary' | wc -l
  # store emit 사이트
  grep -rn '"\s*store ' crates/vais-codegen/src/ \
    | grep -v 'test\|//\|Binary' | wc -l
  ```
- **baseline (iter 80)**: GEP 160 사이트 / store 164 사이트
- **Phase Ω 종료 목표**: 단일 `resolve_index_access` helper로 GEP+store 모두 수렴 (직접 emit ≤ 5 사이트)
- audit 의무: P1.3 LANDED 시점에 4 path 모두 단일 helper. 매 P1.x iter 종료 시 baseline 갱신
- vaisdb 패턴 cover 의무 (iter 78 baseline 기반):
  - node.ll:1848 — Vec<Vec<u8>> push of slice element
  - key.ll:1128 — Vec<&[u8]> indexing (fat-ptr-of-fat-ptr)

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

#### R3 — Same-Class Audit (기계 검증)
- 통합 helper 후보: `coerce_call_arg(param_ty, arg_val) -> Value` (P1.4 Type-Tagged IR Builder에서 흡수)
- audit 명령:
  ```bash
  grep -rn '"\s*call ' crates/vais-codegen/src/ \
    | grep -v 'coerce_call_arg\|test\|//\|Binary' | wc -l
  ```
- **baseline (iter 80)**: 86 사이트
- **Phase Ω 종료 목표**: 0 (모든 call emit이 `coerce_call_arg` 경유)
- 보조 grep (수동 register_temp_type 카운트):
  ```bash
  grep -rn 'register_temp_type\|record_emitted_type' crates/vais-codegen/src/ | wc -l
  ```
- **baseline (iter 80)**: 334 호출 (ADR 0001 측정 329와 일치)
- **Phase Ω 종료 목표**: < 50 (자동 등록 인프라 흡수, P1.4)

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

#### R3 — Same-Class Audit (기계 검증)
- audit 명령:
  ```bash
  # 좁은 패턴: '_ => "i64"' fallback
  grep -rn '_ => "i64"' crates/vais-codegen/src/ | wc -l
  # 넓은 패턴: ResolvedType::Var/Unknown match arm
  grep -rn 'ResolvedType::Var\|ResolvedType::Unknown' crates/vais-codegen/src/ | wc -l
  ```
- **baseline (iter 80)**: 좁은 fallback 7 / Var·Unknown match arm 53
- **Phase Ω 종료 목표**: 좁은 fallback 0 / Var·Unknown 도달 시 panic (Generic/ConstGeneric만 fallback 허용)
- 알려진 fix 사이트 (iter 74 인계):
  - `crates/vais-codegen/src/types/conversion.rs:360-366`
  - `crates/vais-codegen/src/expr_helpers_data.rs:484-489`
- audit 의무: `_ => "i64"`가 ResolvedType 매칭에 있으면 (a) Generic/ConstGeneric만 허용 (b) 다른 케이스는 panic 또는 에러 전환

---

## AI Multi-Session Protocol (강화 2~5)

본 ADR은 인간 개발자뿐 아니라 **Claude Code multi-session AI 협업**도 일급 사용자로 본다. AI multi-session에는 컨텍스트 유실, 자기기만 라벨링, 검증 루프 단축, 사이트 fix 회귀 등 인간과 다른 실패 모드가 있다. 다음 4 절이 AI 안전망이다.

---

### Self-Audit Checklist (매 Pillar 1 iter 종료 전 의무)

본 ADR을 따르는 fix를 LANDED하기 전, 다음 9개 Y/N 체크. **하나라도 NO → LANDED 금지 + 사용자 escalation 의무** (단순 site-fix로 재분류는 가능):

```
[ ] (R1)  commit message에 4 클래스 중 어느 invariant인지 명시?
[ ] (R2)  대응 *_invariant_test.rs에 case 추가됨?
[ ] (R2)  추가 case가 fix 적용 전 fail / 후 pass 양방향 확증?
[ ] (R3)  해당 클래스 R3 audit 명령 실행 → 카운트 기록?
[ ] (R3)  카운트가 줄거나 같음 (절대 늘지 않음)?
[ ] (verify-cargo)     cargo test --workspace ≥ 2625?
[ ] (verify-integrity) ./scripts/check-integrity.sh INTEGRITY OK?
[ ] (verify-vaisdb)    ./scripts/vaisdb-regression.sh --all 합계 ≤ 9?
[ ] (recon)            memory/ROADMAP의 LOC 추정과 실제 wc -l 일치?
```

NO 발생 시 ROADMAP에 `audit_fail: <항목 + 사유>` 기록 의무.

---

### AI Failure Mode Anti-Patterns (avoid)

본 ADR을 따르는 multi-session AI는 다음 패턴을 **절대 반복하지 않는다**. 각 anti-pattern은 실제 발생 commit/iter에 trace됨.

| # | Anti-pattern | 회피 의무 | Trace |
|---|---|---|---|
| A1 | memory의 LOC/카운트 추정을 신뢰하고 wc -l 생략 | 항상 실측 cross-check | memory `feedback_recon_mandatory`, iter 4 "280 panic 30배 과대" |
| A2 | research-haiku idle_notification 후 텍스트 미수신 시 무한 대기 | Opus direct 전환 또는 background 재spawn | iter 76 P3.1+P2.1 사례 |
| A3 | cargo test PASS만 보고 LANDED | integrity + vaisdb 3축 의무 | CLAUDE 규칙 4, Phase 2.10 |
| A4 | import 기반 정적 분석으로 cascade 위험 판정 | standalone build cross-check 의무 | iter 78 false-negative (recon 5 후보 → 실제 32 errors) |
| A5 | "R2 case 5개 PASS = invariant 충족" 결론 | vaisdb 실제 패턴 cover 별도 확증 | iter 74 compound assign REVERTED |
| A6 | 이전 iter 결정 반전 시 ROADMAP 사유 미기록 | 반전 사유 명시 의무 | Phase 158 5회 토글 |
| A7 | iter 종료 시 "다음 세션에 결정" 무한 deferral | CLAUDE 규칙 12 (1주 결정 의무) | Phase 16/17 stopped(unknown) |
| A8 | foreground research-haiku spawn 후 텍스트 도달 안 함 | background로 spawn 또는 Opus direct | iter 76 동일 사례 |
| A9 | 검증 build 시 cache nuke 생략 | force-rebuild + cache nuke 항상 | iter 73 cache-state illusion |

새 anti-pattern 발견 시 본 절 갱신 의무 (다음 iter 시작 시).

---

### Iter Entry Point Spec (multi-session 견고성)

매 Pillar 1 iter는 다음 형식의 entry block으로 시작 의무 (lang/packages/vaisdb/ROADMAP.md):

```yaml
iter: <N>
prerequisite_check:
  - [ ] /tmp/vais-lib/std symlink 존재
  - [ ] ~/.cargo/bin/vaisc 실행 가능 (vaisc --version 출력)
  - [ ] cargo test --workspace ≥ <prev_baseline>
  - [ ] ./scripts/vaisdb-regression.sh --all 합계 ≤ <prev_baseline>
class_in_focus: <1|2|3|4>
sub_invariant: <한 줄 — 본 iter에서 추가/강화하는 sub-property>
r2_target_test: crates/vais-codegen/tests/<class>_invariant_test.rs
r3_grep_command: <복사 가능 명령 — 본 ADR에서 인용>
r3_baseline_count: <iter 시작 시점 측정값>
expected_landing: <"draft" | "test-added" | "fix-LANDED" | "audit-PASS">
session_scope: <"recon" | "fix" | "verification">
multi_session_marker: <"continuing iter N-1" | "new iter">
```

iter 종료 시 동일 block + actual 결과 + 다음 iter entry로 갱신. ROADMAP 산문 흩어짐 방지.

---

### Iter Retrospective (의무, P4.3 LANDED iter 84)

매 iter 종료 시 ROADMAP `lang/packages/vaisdb/ROADMAP.md`에 다음 형식의 retrospective 절 추가 의무:

```markdown
### iter <N> strategy + 결과 (YYYY-MM-DD, <task-id> LANDED)
- task: <id> <subject> (<단독|병렬>, 위험 <0~10/10>, 추정 <시간>)
- strategy: <Opus direct | impl-sonnet | research-haiku> (<이유>)
- 산출 (<repo> commit `<hash>`):
  - <변경 1>
  - <변경 2>
- ADR 0001 분류: <root-cause-fix | site-fix | N/A>
- ADR 0002 분류: <Class 1|2|3|4 | N/A> (codegen 변경 시)
- 검증: cargo test <count> / check-integrity <std>/<vaisdb> / vaisdb-regression <count>
- 다음 task 후보 (iter <N+1>+): <옵션 A> / <옵션 B>
- 본 iter commits <count>: <repo> `<hash>` ...
```

**의무 항목**:
1. 매 iter LANDED 시 ROADMAP에 위 형식 절 추가 (즉시, iter close 전)
2. iter close 시 mode/iteration/max_iterations 갱신
3. multi-session 종료 시 별도 "iter N~M 세션 종료" 절 추가 (iter 76~80 패턴)
4. retrospective 누락 시 다음 iter entry 차단 (lint 권장 — tooling 미구현, manual 게이트)

**Why**: iter 76~83에서 자발적 관행화. 본 P4.3에서 정책 명문화로 채택. ROADMAP 산문 흩어짐 방지 + multi-session 인계 안전.

**적용 시점**: 본 ADR 갱신 (P4.3 LANDED) 이후 모든 iter.

---

### Rollback Trigger (정량)

다음 중 하나라도 충족 시 **즉시 git revert + stash 보관 + ROADMAP 기록 의무**. CLAUDE 규칙 4번을 본 ADR에 binding.

| Trigger | 검출 명령 | 임계값 |
|---|---|---|
| T1. cargo test 카운트 감소 | `cargo test --workspace 2>&1 \| grep -E 'test result:'` | -1 라도 trigger |
| T2. integrity 감소 | `./scripts/check-integrity.sh \| tail -3` | std/vaisdb 카운트 -1 |
| T3. vaisdb regression 증가 | `./scripts/vaisdb-regression.sh --all` | 합계 errors > 9 |
| T4. clippy 신규 에러 | `cargo clippy --workspace --exclude vais-python --exclude vais-node` | error 신규 (warning OK) |
| T5. Self-Audit Checklist NO | (위 9개 중 하나) | 1개라도 NO |

Rollback 후 의무:
1. **stash 보관** (drop 금지) — 학습 자료
2. ROADMAP에 `rollback: <iter N>. <trigger>. 영향: <카운트>` 기록
3. 같은 fix 재시도 시 Self-Audit Checklist 처음부터 + 추가 case 보강
4. **3회 연속 rollback** → 사용자 escalation (단일 사이트 fix 금지, Pillar 재설계 검토)

T5 "Self-Audit NO"가 안전망 — 단순 panic/unreachable 추가가 T1~T4에서 silent하게 통과해도 T5에서 차단.

---

## R1+R2+R3 충족 매트릭스 (iter 80 baseline)

| Class | R1 | R2 test | R2 case | R3 baseline | R3 목표 | Status |
|---|---|---|---|---:|---:|---|
| 1. ret elem-ty | ✅ | ✅ ret_invariant_test.rs | ⚠ 5/40+ | 152 ret emit | 0 (helper 경유) | **partial-LANDED** |
| 2. index/store | ✅ | ✅ index_invariant_test.rs | ⚠ 5→≥10 (P1.1) | 160 GEP / 164 store | helper ≤ 5 사이트 | **partial-LANDED** |
| 3. call-arg | ✅ | ✅ call_arg_invariant_test.rs | ⚠ 1 active+1 ignored | 86 call / 334 register | call 0 / register < 50 | **partial-LANDED** |
| 4. var-to-llvm | ✅ (이 ADR) | ❌ 신설 (P1.0 후속) | — | 7 fallback / 53 match | 0 fallback / panic 도달 | **NEW (proposed)** |

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

- **2026-04-26 iter 80 (P1.0a Draft + P1.0b Accepted)**: 본 ADR 채택. AI multi-session protocol 4 절 흡수 (Self-Audit / Anti-Patterns / Iter Entry / Rollback)
- **iter 81~**: 신규 codegen if-coerce 분기는 본 ADR 4 클래스 중 하나에 분류 의무 (CLAUDE.md 규칙 8 강화)
- **iter 83 P1.1 시작 시점**: 본 ADR Self-Audit Checklist를 모든 P1.x iter LANDED 게이트로 적용
- **Pillar 1 종료 후**: 본 ADR가 Pillar 1의 exit audit 기준 (R1+R2+R3 4/4 ✅ + R3 카운트 모두 목표 도달)

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
| 2026-04-26 | 80 | Draft 작성 (P1.0a) | Opus direct |
| 2026-04-26 | 80 | Accepted 전환 + AI multi-session protocol 4 절 흡수 (P1.0b). R3 audit baseline 측정 (iter 80 실측 grep) | Opus direct (사용자 승인) |
| 2026-04-28 | 124 | §"Amendment — 단일 API 수렴 한계" 추가 (Pillar 4 Task #35) | Opus direct |

---

## Amendment — 단일 API 수렴 한계 (2026-04-28, iter 124, Pillar 4 Task #35)

본 ADR §"AI multi-session protocol" / Phase Ω 종료 invariant 3번 ("763개 산발 사이트가 단일 API로 수렴")의 강한 표현을 P1.4 학습 (iter 104~120, 17 iter)에 따라 다음과 같이 수정한다.

### 본질적 제약 (P1.4 17-iter 측정 결과)

ADR 0002 4 클래스 invariant + R1+R2+R3 게이트는 codegen action 단위 보장만 다룬다. **단일 wrapper API로 모든 사이트를 자동 흡수**하는 것은 wrapper 자체의 hidden cost (ADR 0003 R4 참조)로 본질적 제약이 있다.

P1.4가 763 사이트 (165 if-coerce + 329 register_temp + 77 bitcast + 53 insertvalue + 139 inttoptr/ptrtoint)에 대해 적용한 결과:
- **카테고리 A (단일 LLVM 타입)**: 1 사이트 (stmt_visitor.rs:708 ret-cast) wrapper migration LANDED. +0.6 file production impact.
- **카테고리 B (다양 타입)**: 3회 시도 모두 결정적 negative (iter 110/116/119). wrapper 사용 자체가 production impact 악화.
- **카테고리 C (구조적 단순)**: 위험 4/10 측정 후 결정 (개별 사이트 5-run 측정 의무).
- **카테고리 D (일반화 분기)**: sub-분류 필요, 자동화 부적절.

### 수정된 invariant 3

원안:
> "165 ad-hoc if-coerce + 329 수동 register_temp_type 산발 사이트가 단일 API로 수렴 (Pillar 1)"

수정안:
> "165 ad-hoc if-coerce + 329 수동 register_temp_type 산발 사이트는 **분류 체계 (카테고리 A/B/C/D)** 로 관리되며, 카테고리 A는 단일 wrapper API로 자동 흡수, 카테고리 B/C/D는 manual register/coerce 유지가 안전. wrapper migration은 ADR 0003 R4 (Hidden Cost Audit) 충족 의무."

### Self-Audit Checklist 보강

본 ADR §"Self-Audit Checklist (9 항목)"에 항목 10번 추가:
- **10. wrapper migration 시 ADR 0003 R4 적용 여부**: 본 commit이 wrapper migration이면 R4.1+R4.2 5-run 측정 결과 명시. R4.4 카테고리 분류 명시.

### Why
P1.4 17 iter 작업에서 "단일 API 수렴"의 강한 표현이 wrapper migration 무용 사이트에 대한 반복적 시도 (iter 110/116/119, 동일 패턴)를 유발했다. 본 amendment로 미래 P1.x 작업이 카테고리 분류 + R4 hidden cost audit을 사전에 적용하여 결정적 negative migration을 차단한다.
