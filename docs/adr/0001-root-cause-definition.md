# ADR 0001 — "근본 해결"의 공식 정의

**Status**: Accepted
**Date**: 2026-04-26
**Decision driver**: 사용자 — "몇 달 동안 계속 진행하다 다시 뜯어고치고가 반복인 것 같아. 어떻게 해결해야 이런 일이 다시 발생하지 않을까?"

---

## 배경 (Context)

vais 컴파일러 개발에서 동일 클래스 버그가 다른 사이트에서 반복적으로 재발하는 패턴이 누적되어 왔다. 정량 측정값 (2026-04-26 실측):

| 지표 | 값 | 의미 |
|---|---:|---|
| codegen LOC | 70,530 | 단일 crate가 70K 거대 |
| if-coerce 분기 산발 | **165** | 165곳에 ad-hoc 타입 변환 |
| 수동 `register_temp_type` | **329** | 329곳에 수동 타입 등록 |
| 수동 `bitcast` emit | **77** | 77곳에 산발 변환 |
| 수동 `insertvalue` emit | **53** | 53곳에 산발 fat-ptr 구성 |
| 수동 `inttoptr`/`ptrtoint` | **139** | 139곳에 포인터/정수 변환 |
| Phase 158 coercion 토글 | **5회** | 같은 결정을 5번 번복 |
| Phase 17 (invariant 시도) | `stopped (unknown)` | invariant 도입 시도가 cascade로 실패 |
| Master Roadmap pivot | iter 64 | "trust-building"으로 pivot했으나 iter 68~73에 다시 사이트 fix 회귀 |

**관찰된 패턴**:
1. 사이트 A에서 버그 발견 → 사이트 A에 if-coerce 분기 추가 → "근본 fix" 라벨
2. 같은 클래스 버그가 사이트 B에서 발견 → 사이트 B에도 if-coerce 추가 → 또 "근본 fix" 라벨
3. ... 165회 반복
4. 어느 시점에 invariant 시도 (Phase 17) → cascade → revert → stopped
5. 다시 1번으로 회귀

이 패턴이 종결되지 않는 핵심 이유: **"근본 해결"의 정의가 합의되지 않아, 사이트 fix와 진짜 invariant fix가 같은 라벨로 불림**.

---

## 결정 (Decision)

이 문서로 "근본 해결"과 "사이트 fix"의 정의를 공식화한다. 이후 모든 vais 컴파일러 PR/Phase는 이 정의를 따른다.

### 1. 근본 fix (Root-Cause Fix) — 정의

다음 **3개 요건을 모두 충족**해야 "근본 fix"로 분류된다:

#### 요건 R1 — Invariant 명시
fix가 보장하는 **invariant**를 한 문장으로 명시한다.

> 형식: "이 fix 이후, [어떤 조건]에서 [어떤 속성]이 항상 성립한다."
>
> 예 (좋음): "이 fix 이후, codegen이 emit하는 모든 `ret` 명령은 value의 LLVM 타입이 함수 시그니처의 ret 타입과 일치한다."
>
> 예 (나쁨): "ret path에서 Vec→fat-ptr 변환이 추가되었다." ← 이건 fix 동작 설명, invariant 아님.

#### 요건 R2 — 차단 테스트
**그 invariant가 깨지면 반드시 fail하는** 테스트를 추가한다.

- 단일 사례 fix가 아니라 invariant 자체를 검증
- grep-based audit 또는 codegen-output assertion 형태 권장
- e2e 테스트 1개로 부족 — invariant violation을 직접 감지해야 함

> 예 (좋음): "codegen 후 emit된 모든 IR 파일에 `ret { i8*, i64 } %X` 패턴 발생 시 X의 emitted_type이 `{ i8*, i64 }`임을 grep + AST로 검증하는 테스트"
>
> 예 (나쁨): "test_btree.vais가 컴파일 통과한다" ← 단일 사례, 다른 사이트 누락 검증 못 함.

#### 요건 R3 — Same-Class Audit
같은 클래스 버그가 **다른 모든 사이트에서도 해결됨**을 증명한다.

- grep으로 동일 패턴 사이트 전수 조사
- 각 사이트가 (a) fix 적용됨 또는 (b) invariant에 의해 자동 해결됨 또는 (c) fix 대상 아님임을 확증
- "0 hits" 또는 "전수 매핑 결과" 보고 의무

> 예: `grep -rn "ret { i8\*, i64 }" crates/vais-codegen/src/` → 5건 발견 → 각 건이 invariant 적용 후 안전함 file:line으로 확증.

### 2. 사이트 fix (Site Fix) — 정의

위 R1~R3 중 **하나라도 충족 못하면** 사이트 fix로 분류한다. 사이트 fix는 다음 의무를 진다:

#### 요건 S1 — "임시" 라벨
- 코드에 `// TEMP-SITE-FIX(adr-0001):` 주석 의무
- 이 주석 없이 ad-hoc coercion 분기 추가 금지

#### 요건 S2 — 추적 issue
- "이 클래스의 근본 fix"를 추적하는 issue/Phase entry 의무
- 사이트 fix 추가 시 그 추적 issue에 사이트 추가 기록

#### 요건 S3 — 만료 기한
- 사이트 fix는 다음 Phase 종료까지 유효
- Phase 종료 시 (a) 근본 fix로 승격 또는 (b) 새 Phase로 추적 이관

### 3. Phase 종료 게이트 변경

기존 Phase 종료 기준:
- "이슈 해결 + E2E 0 regression"

새 Phase 종료 기준:
- (1) **이슈 해결 + E2E 0 regression** (기존)
- (2) **+ 추가된 모든 사이트 fix가 추적 issue에 기록됨**
- (3) **+ Phase 시작 시 명시한 invariant가 충족됨** (Phase 시작 시 invariant 명시 의무 — CLAUDE.md 규칙 11)
- (4) **+ same-class audit grep 결과 보고됨** (R3와 동일)

### 4. ROADMAP 표기 의무

ROADMAP에 task 추가 시 다음 형식 권장:

```markdown
- [ ] N. <subject>
  type: root-cause-fix | site-fix
  invariant: <R1 명시 — root-cause-fix만>
  test: <R2 차단 테스트 위치 — root-cause-fix만>
  audit: <R3 grep 결과 — root-cause-fix만>
  tracker: <추적 issue — site-fix만>
  expires: <Phase 명 — site-fix만>
```

---

## 결과 (Consequences)

### 긍정
- "이번엔 근본 fix"가 명확한 게이트를 통과해야만 그 라벨을 받음 → 자기기만 차단
- 사이트 fix는 명시적으로 "임시"로 추적됨 → 누적 확인 가능
- Phase 종료 게이트가 invariant 충족을 강제 → Phase 17 같은 무한 stopped 차단

### 부정
- 단순 fix도 R1~R3 작성 의무 → 초기 오버헤드 증가
- 추적 issue 관리 부담
- "정확한 invariant 명시"가 어려운 경우 사이트 fix로 fallback해야 함

### 위험 완화
- 사이트 fix는 **금지가 아님** — 명시적 라벨링과 추적만 의무
- "복잡한 fix는 모두 근본이어야 함"이 아니라 "근본이라고 부르려면 게이트 통과 의무"
- ADR 자체도 ADR로 변경 가능 (ADR 0002로 갱신 시)

---

## 적용 시점

- **2026-04-26 (이 문서 채택일)부터 모든 신규 codegen PR에 적용**
- 기존 165개 사이트 fix는 retro-active 분류 작업 별도 (Pillar 1 시 일괄 흡수 예정)
- CLAUDE.md 규칙 8~12로 사전 차단 메커니즘 추가 (별도 commit)

---

## 적용 범위 (2026-04-26 추가, P4.1 retrospective 백로그 1번)

본 ADR (R1+R2+R3 게이트)의 적용 영역과 면제 영역을 명시한다. P4.2에서 std E009
38건 mechanical fix가 ADR 게이트 적용 의무 모호 영역으로 드러난 것이 동기.

### 의무 적용 (R1+R2+R3 모두)

- **codegen Rust 코드**: `crates/vais-codegen/`, `crates/vais-types/checker_*` 등 컴파일러 핵심
- **TC inference / unification**: `crates/vais-types/src/inference/` 등 타입 추론 변경
- **새 codegen path 추가** 또는 **기존 path semantics 변경**
- **ADR 0002 4 클래스 invariant 영향**: ret / index-store / call-arg / var-to-llvm

### 면제 (R1+R2+R3 의무 없음, 단 commit message에 분류만 명시)

- **`.vais` source-only 변경**: std/, lang/packages/*/src/, examples/, tutorial/
  - 예: P4.2 std E009 mut 38건 일괄 추가
  - 이유: codegen invariant와 무관, .vais source 정합성 회복은 mechanical
  - 의무: commit message에 "ADR 0001 분류: source-only (codegen 무영향)" 명시
- **문서 / 정책 / spec**: docs/, README, CLAUDE.md, ADR 자체
  - 의무: ADR 변경 시 reference link만 명시
- **CI / 인프라 / scripts**: .github/workflows/, scripts/, Cargo.toml 의존성
  - 의무: 차단력 (regression detection) 평가 명시
- **테스트 추가**: tests/, benches/ — 단 codegen invariant 변경 동반 시 R2로 분류

### 경계 사례 — 사용자 판단

- **.vais → Rust binding 변경** (vais-node, vais-python): 면제 (downstream binding)
- **CLAUDE.md 규칙 추가**: 정책으로 면제, 단 동반 ADR 변경 시 함께 LANDED
- **stdlib 함수 신규 (.vais)**: 면제, 단 codegen이 builtin dispatch 처리 시 codegen 측 audit 의무

### Why
P4.2 audit이 std E009 38건을 발견했을 때 "ADR 0001 적용 의무인가?"가 결정 모호.
실제로 std E009 fix는 codegen 무영향 mechanical 변경이라 R1+R2+R3 부담 정당화
어려움. 본 절 추가로 미래 audit/PR 결정이 명확해짐.

---

## 참고

- 정량 분석 출처: `lang/packages/vaisdb/ROADMAP.md` iter 74 entry (2026-04-26)
- 4-Pillar 안정화 제안: 동 문서, Phase Ω 섹션
- 기존 CLAUDE.md 규칙 1~7: 사후 가드레일 (regression 후 대응) — 본 ADR + 규칙 8~12는 사전 가드레일

---

## Retrospective (2026-04-26, Phase Ω P4.1, iter 83)

ADR 0001 LANDED 이후 ~9시간 / 15 compiler commits / 8 lang commits 의 적용 효과 평가.

### 정량 측정

**측정 윈도우**: c683bd42 (ADR 0001 LANDED, iter 74) → d0a4c89a (iter 82, 본 retrospective 시점)

| 지표 | 값 | 평가 |
|---|---:|---|
| 신규 commits (ADR LANDED 이후) | 15 | — |
| ADR 0001 명시 인용 commits | 13/15 (87%) | ✅ 강한 채택 |
| `revert` 메시지 commits | 0 | ✅ Phase 158 5회 토글 패턴 부재 |
| `TEMP-SITE-FIX(adr-0001)` 주석 사이트 | 0 | ✅ site-fix 누적 부재 (또는 모두 root-cause로 라벨) |
| stash (cascade 시 보관) | 1 (`phaseO_compound_assign_fix`) | ✅ 규칙 적용 (iter 74 fix 시도가 vaisdb 깨짐 → revert 대신 stash) |
| 신규 R1+R2+R3 충족 invariant | 2 (ret + call-arg) | ✅ ret_invariant_test (5 case) + call_arg_invariant_test (3 case) |
| ADR 신규 (0002) | 1 | ✅ 0001 게이트를 codegen 4 클래스로 확장 |

### 질적 관찰

#### 1. R3 게이트가 cascade 차단에 성공
- iter 74: stash@{0} `phaseO_compound_assign_fix`가 vaisdb 3건 regression 유발 → R3 audit 미충족 → CLAUDE 규칙 4 (즉시 revert)에 따라 stash 보관, 진행 차단
- 결과: Phase 17/Phase 158 같은 "임시 fix가 main에 유입되어 cascade" 패턴 차단
- 직접 효과: 본 retrospective 시점 main HEAD에 cascade 흔적 0건

#### 2. 단일 coerce point 패턴 도입 성공 (Mini Pillar 1)
- 7cfc5caf: `coerce_ret_value` 신설 → c0d5bd31, 628674ec, 2ab0a421로 3 사이트 migration
- ret_invariant_test 5/5 PASS 유지 (각 migration commit마다 cargo test로 검증)
- ad-hoc if-coerce 165 → 162 (3 사이트 흡수, ADR 0002 Pillar 1.4가 나머지 처리 예정)

#### 3. ADR 0002 (codegen invariants)로 0001을 4 클래스로 정밀화
- 0001은 "근본 fix 정의"만 명시 (보편 게이트), 0002는 codegen에 한해 4 클래스 invariant + R3 grep+카운트 baseline 명시
- iter 80 baseline: ret 152 / GEP 160 / store 164 / call 86 / register 334 / Var fallback 7 / Var match 53
- iter 81 P4.2: 위 7 baseline 모두 0 변동 확증 → R3 게이트가 측정 가능

#### 4. AI multi-session protocol 흡수 (ADR 0002)
- 0001은 게이트 정의만, 0002는 AI agent의 multi-session work을 위한 5 강화 흡수:
  - Self-Audit Checklist 9 항목 (R1+R2+R3 + verify-cargo/integrity/vaisdb + recon)
  - AI Failure Mode Anti-Patterns 9건 (A1~A9, commit/iter trace)
  - Iter Entry Point Spec yaml block
  - Rollback Trigger T1~T5 (T5 = Self-Audit NO 안전망)
- iter 81 P4.2가 이 protocol을 첫 적용 → R3 baseline 7/7 정확 확증으로 protocol 가치 입증

### 부정적 관찰 (정책 한계)

#### 1. R1+R2+R3 부담 평가
- 본 retrospective 측정 윈도우의 13 ADR-cited commits에서 R1+R2+R3 모두 명시한 것은 4건 (db44f364, 1b99766c, 7cfc5caf, 041685e6)
- 나머지 9건은 R3 또는 R2만 부분 명시 (refactor migration commits는 R2 PASS만 인용)
- 평가: refactor commits는 R1 invariant가 prior commit에서 이미 정의됨을 가정 → 명시 부담 감소가 정당
- 정책 보강 권장 없음 (현 운영 적정)

#### 2. 0001만으로는 codegen 외 영역 커버 불충분
- 본 P4.2 발견: std E009 38건은 codegen이 아닌 .vais source 정합성 이슈
- 0001 R1+R2+R3는 codegen invariant 강화에는 효과적이지만 "codebase 정합성" 영역은 별도
- 0001 적용 영역: codegen / TC fix만 명확히 정의, std/vaisdb source 변경은 ADR 적용 의무 모호
- 정책 보강 권장: ADR 0001에 "적용 범위" 절 추가 (codegen + TC 핵심, source-only 변경은 면제)

### 결론

**ADR 0001 + ADR 0002 + CLAUDE 규칙 8~12 + Phase Ω Pillar 3+2의 통합 효과**:
- ad-hoc fix 누적 차단: ✅ 0건 신규 site-fix label, stash로 cascade 차단
- invariant 명시 의무: ✅ 87% adoption rate (13/15)
- R3 cascade audit: ✅ 1건 cascade 사전 차단 (stash@{0})
- 측정 가능 baseline: ✅ ADR 0002로 7항 grep+카운트 명시

**개선 제안 (미래 ADR 0003+ 후보)**:
1. 0001 §"적용 범위" 절 추가 — codegen + TC 외부 (std/vaisdb source) 변경의 ADR 적용 의무 명시
2. R1 명시 부담 감소 — refactor commits는 prior commit의 R1 인용 허용 (관행화)
3. retrospective 의무화 (P4.1 본건이 첫 사례) — 매 N iter (예: 10 iter) 자동 재평가

**본 retrospective의 결정**:
- 정책 변경 미권장 (현 ADR 0001+0002 효과 충분히 검증)
- 개선 제안 1번 (적용 범위 명시)은 별도 ADR 0003 후보로 등록 (백로그)
- ADR 0001 Status: **Accepted (sustained)** — retrospective 통과
