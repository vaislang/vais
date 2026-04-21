# Vais — Next Drive Candidates

> **상태**: 신규 드라이브 선택 대기
> **직전 드라이브**: `ROADMAP-cascade-drive.md` ("Cascade & Vec Completion",
>   4/4 완료 2026-04-21, commit `38d9a1f3`)
> **더 이전 드라이브**: `ROADMAP-compiler-drive.md` (11/11 완료, commit `e1697c14`)
> **아카이브**: `ROADMAP-archive.md` (Phase 0 ~ 6.31 히스토리)

---

## 현재 baseline (2026-04-21)

직전 드라이브 종료 시점:

| 항목 | 숫자 |
|------|------|
| compiler_syntax | 200/200 |
| compiler_stages | 14/14 (1 #[ignore] B7) |
| std_files | 82/82 |
| living_spec | 116/116 (D.2 reproducer 3개 포함) |
| phase158 strict | 18/18 |
| vaisdb_files | 237/261 (90.8%) |
| vaisc e2e | 2625/0/1 |
| vaisc integration | 147/147 |

모든 다음 드라이브는 이 baseline을 **regression floor**로 유지해야 한다
(CLAUDE.md rule 4).

---

## 다음 드라이브 후보

세 축을 제안. 사용자가 하나를 고르면 그 드라이브로 `ROADMAP.md` 재작성.

### 후보 1 — vaisdb cleanup drive (가장 높은 vaisdb 수치 상승 기대)

**목적**: `docs/vaisdb-cascade-survey.md` §3.2 / §5 에 정리된 20개 vaisdb
code bugs 를 vaisdb 저장소 쪽에서 정리.

**특징**:
- **vaisdb 저장소 작업** (이 컴파일러 저장소 밖, `/Users/sswoo/study/projects/vais/lang/packages/vaisdb`).
- 패턴이 매우 반복적 (`write_page` arg 1 vs 2, `get_node`/`insert_node` 없음 등).
- 예상 산출: vaisdb 237/261 → 250+/261.

**리스크**: 컴파일러 저장소 밖이라 이 harness/check-integrity 체계의 보호가
덜 적용됨. vaisdb 자체 CI 가 필요.

**추천 조건**: vaisdb 쪽에 시간 투자 OK, integration 수치가 우선순위일 때.

### 후보 2 — struct-in-array-literal text-backend fix (D.2 자연 후속)

**목적**: `[Point{x:1}, Point{x:2}]` 같은 struct literal 을 포함하는 배열
리터럴 이 text-backend 에서 `store %Point <ptr>` 를 emit 해서 clang 에 거부됨.
D.2 에서 발견한 실제 compiler gap.

**특징**:
- `crates/vais-codegen/src/expr_helpers_data.rs::generate_array_expr` 개선.
- 원소 타입이 Named 일 때 struct 값 load → struct store 패턴으로 분기.
- `Vec<Struct> := [Struct{...}, ...]` end-to-end 완성 → LANGUAGE_SPEC Matrix
  L262 `Vec<Struct>[i].field =` 완전 ✓.

**vaisdb 레버리지**: 현재 0 (아무 파일도 이 패턴 안 씀). stdlib 확장이나 미래
예제에 유용.

**리스크**: 낮음. 단일 함수 변경, 기존 케이스에는 영향 없음.

**추천 조건**: 컴파일러 완성도 추구, 작은 스코프의 드라이브.

### 후보 3 — Phase 4.x SCOPED 심층 (E034 purity analysis, generic bounds)

**목적**: `docs/vaisdb-cascade-survey.md` §3.3 의 (c) 버킷 — E034 "total
function may panic" 정확도, generic bound propagation 재검증.

**특징**:
- `vais-types/src/checker_fn.rs` + 관련 `partial` keyword 전파 논리 개선.
- `crates/vais-types/src/inference.rs` 의 trait bound 확인 개선.

**vaisdb 레버리지**: +4 files (manager/window/bulk/cow). 이 파일들은 vaisdb
bug 도 섞여 있어 순수 컴파일러 fix 만으로는 완전 unblock 안 될 수도 있음.

**리스크**: 중간. Phase 4.x 는 원래 SCOPED 로 분류된 영역 — 부분 구현은
보류된 이유가 있음. 규모 증가 위험.

**추천 조건**: 타입 시스템 심층 기능에 투자할 의사가 있을 때.

---

## 선택 방식

사용자가 1/2/3 또는 "다른 축" 을 지정하면 이 파일을 해당 드라이브의
실행 ROADMAP 으로 재작성.

## 드라이브 외 단발성 과제 (언제든 가능)

- **`Vec::new()` 정의 추가** (`std/vec.vais`) — 현재 `with_capacity(0)` 만
  가능. ≤ 10 줄 fix.
- **examples/ 디렉토리 정리** — Phase 4.x 관련 deprecated 예제 제거.
- **LANGUAGE_SPEC.md Matrix** 의 `◐` 행들 재검증 (한 번 훑기).

---

## 작업 시작 시

드라이브 선택 후:
1. 이 파일을 새 드라이브 ROADMAP 으로 재작성 (`mode: pending` 또는 `auto`,
   구체적 tasks, baseline, 설계 원칙).
2. 현재 baseline 재측정 (`./scripts/check-integrity.sh`) → 고정.
3. `harness-init` 또는 `harness-plan` 이 나머지 처리.
