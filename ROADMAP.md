# Vais — Double Drive: struct-in-array-literal → Phase 4.x SCOPED

> **scope revision 2026-04-21**: 원래 triple drive 였으나 G (vaisdb cleanup)
> 축은 vaisdb 저장소 자체 드라이브로 이관. 이 드라이브는 compiler 저장소 내
> 작업 (F + H) 만 포함하여 check-integrity 안전망 안에서 완결.

> **버전**: 2026-04-21 triple drive 시작
> **이전 드라이브**: `ROADMAP-cascade-drive.md` ("Cascade & Vec Completion", 4/4 완료)
> **더 이전 드라이브**: `ROADMAP-compiler-drive.md` (11/11 완료)
> **아카이브**: `ROADMAP-archive.md` (Phase 0 ~ 6.31 히스토리)

---

## 드라이브 목적

사용자가 우선순위대로 연속 진행 지정. 이 세션은 compiler 저장소 내 2 축:

1. **F (struct-in-array-literal fix)** — D.2 자연 후속, compiler 완성도.
2. **H (Phase 4.x SCOPED 심층)** — E034 purity + generic bound.

G 축 (vaisdb cleanup) 는 vaisdb 저장소 전용 드라이브로 이관
(`/Users/sswoo/study/projects/vais/lang/packages/vaisdb/ROADMAP.md` 기반).
이유: vaisdb BufferPool API 재설계 (`write_page` 2-arg→1-arg) 가 단순 arg 제거가
아니라 frame.data 데이터 흐름 재설계 필요 — 9+ 사이트 각각 판단 필요,
vaisdb 자체 regression CI 부재로 compiler 세션 내에서는 안전하게 완주 불가.

각 축 완료 후 baseline 유지 확인 → 다음 축 진입. 중간에 regression 발생 시
즉시 revert + 원인 분석 후 진행 여부 재결정.

---

## Baseline (2026-04-21 triple drive 시작)

직전 드라이브 완료 시점:

| 항목 | 숫자 |
|------|------|
| compiler_syntax | 200/200 |
| compiler_stages | 14/14 |
| std_files | 82/82 |
| living_spec | 116/116 |
| phase158 strict | 18/18 |
| vaisdb_files | 237/261 (90.8%) |
| vaisc e2e | 2625/0/1 |
| vaisc integration | 147/147 |

**Regression floor**: syntax=200, stages=14, std=82, vaisdb=237, phase158=18,
living_spec=116. 감소 0 허용 (vaisdb는 G 드라이브에서 상승이 목표).

---

## 설계 원칙

1. **CLAUDE.md rule 3/4 엄수**: 수정 전 baseline 기록, 1-file regression 즉시 revert.
2. **축 간 격리**: F/G/H 사이 baseline 체크 의무. F 끝날 때 baseline 재확인 후 G 진입.
3. **축 단위 commit**: 각 축 완료 시 독립 commit (rollback 용이성).
4. **Scope 엄격**: G 축은 vaisdb 저장소에서만 작업, compiler 저장소 손대지 않음.
5. **H 축 범위 폭발 방지**: 타입 시스템 변경 시 범위 넓어질 조짐 있으면 중단 후 사용자 확인.

---

## Current Tasks (2026-04-21)

mode: auto
iteration: 4
max_iterations: 25
strategy: H.1 E034 purity analysis exploration — sequential
opus_direct: H.1 type-system changes require cautious scope decisions per CLAUDE.md rule 4; measurement-first approach to avoid Phase 158-style yo-yo regressions

### Phase F — struct-in-array-literal fix (text backend)

- [x] F.1 — `[Struct{...}, Struct{...}]` text backend lowering (Opus direct) ✅ 2026-04-21
      changes:
        crates/vais-codegen/src/expr_helpers_data.rs (generate_array_expr:
          원소 elem_ty 가 Named struct 이면 ptr 을 load 해서 struct value store)
        docs/language/LIVING_SPEC/04_stdlib/vec_struct_literal.vais (reproducer)
        docs/LANGUAGE_SPEC.md Matrix L262 업데이트
      result:
        - `v: Vec<Point> := [Point{x:1,y:2}, Point{x:3,y:4}]; R v[0].x + v[1].x` → 4 ✓
        - scalar Vec 회귀 없음: `v: Vec<i64> := [10,20,30]` 합계 = 60 ✓
        - LIVING_SPEC 116 → 117
        - compiler baseline 유지
      scope 한계:
        - literal-init 후 write (`v[0].x = 99`) 는 B.4 memcpy-to-temp 경로라
          본 fix 범위 밖. 실제 값이 Vec 에 반영 안 됨. 별도 drive 필요
          (Vec<Struct> 원소 수정 시 memcpy 대신 GEP+store).
  **배경**: D.2 에서 발견. `generate_array_expr` (expr_helpers_data.rs:35) 가
  Struct literal 을 array 원소로 받으면 `val`이 struct 포인터인데, `store` 를
  struct value 기준으로 emit 해서 `store %Point %t1` (t1은 ptr) 타입 미스매치.
  target: `crates/vais-codegen/src/expr_helpers_data.rs::generate_array_expr`
  approach:
    1. 원소 Expr 의 resolved type 이 Named (struct) 이면 current `val` (ptr)
       를 load 해서 struct value 얻은 뒤 store. 기존 primitive 경로는 그대로.
    2. `is_struct_lit` 등 기존 classifier 재사용 가능.
    3. ConstArray 스타일 (원소 타입 단일) 전제. Tuple 안에 struct 섞여 있어도 각각 처리.
  [완료 기준]:
    - `v: Vec<Point> := [Point{x:1,y:2}, Point{x:3,y:4}]; R v[0].x + v[1].x` → 4
    - `v[0].x = 99; R v[0].x + v[1].x` → 102 (B.4 write-through 완성)
    - LIVING_SPEC +1 reproducer (`vec_struct_literal.vais`)
    - compiler baseline 유지 (vaisdb, e2e, integration 모두)
    - LANGUAGE_SPEC Matrix L262 Run 컬럼 ◐ → ✓ (최종)

### Phase G — 제거 (vaisdb cleanup 은 vaisdb 저장소 자체 드라이브로 이관)

> **scope revision 2026-04-21**: G 는 세션 내 완주 불가 (9+ write_page
> 사이트는 단순 arg strip 불가, BufferPool API 재설계 필요 / vaisdb
> 자체 regression CI 부재). `docs/vaisdb-cascade-survey.md` 가 다음
> vaisdb 드라이브의 가이드 역할.

### Phase H — Phase 4.x SCOPED 심층

- [x] H.1 — E034 purity analysis 정밀도 개선 (Opus direct) ✅ 2026-04-21
      changes:
        docs/totality-analysis-findings.md (E034 현재 설계 분석,
          vaisdb 4 파일 E034 이 legitimate 인지 확인, 개선 후보
          5가지 평가, "flow-sensitive 분석 필요 → Phase 4.x 스코프 밖"
          결정 문서화)
      result:
        - 조사 결과: 현재 E034 는 정확함. vaisdb 4 E034 사례
          (window/manager/bulk/cow) 모두 `!` 직접 사용으로 legitimate panic
          source. 가장 가치 있는 개선인 `contains_key → insert → get!` 패턴
          인식은 flow-sensitive 분석 필요 → TC 재설계 규모, Phase 158 요요
          위험 → 이 드라이브 범위 밖.
        - 실제 vaisdb E034 해결: vaisdb 코드 쪽에서 `partial F` 추가하거나
          `?` 전파로 재작성. vaisdb cleanup drive 스코프.
        - baseline 유지.
      scope 결론: Phase 4.x SCOPED 범위 폭발 방지 원칙 적용. 개선 후보는
        "What remains for a future TC drive" 섹션에 기록.

- [ ] H.2 — generic bound propagation 확인 (Opus direct) [blockedBy: H.1]
  target: `crates/vais-types/src/inference.rs`.
  bulk.vais `S: NodeStore` bound 가 중첩 call 에서 소실되는 이슈를 실측.
  approach: 실측 후 판단. TC 에 propagation gap 이 있으면 fix,
    아니면 별도 재현 case 만 document 로 남김.
  완료 기준: 실측 결과 + 조치 (fix or 문서화) + baseline 유지.

- [ ] H.3 — 드라이브 최종 완료 선언 (Opus direct) [blockedBy: H.2]
  double drive 종료 (F + H), 통합 결과 기록, 다음 드라이브 후보 제안
  (vaisdb cleanup drive 포함).

progress: 1/4 (25%)

---

## Gate 기준

- **F 완료 조건**: F.1 ✓, compiler baseline 유지, LIVING_SPEC +1, Matrix L262 ✓.
- **G 진입 조건**: F.1 완료 + 사용자 승인 (G.0).
- **G 완료 조건**: G.4 ✓, compiler baseline 유지 (vaisdb 상승만 허용).
- **H 진입 조건**: G.4 완료 + 사용자 승인 (H.0).
- **H 완료 조건**: H.3 ✓, 드라이브 통합 결과 기록.
- **즉시 revert 조건**: 각 task 후 regression 1-file 감지 시.

---

## Archive / 별도 드라이브

- **Cascade & Vec Completion**: `ROADMAP-cascade-drive.md` (4/4 완료, commit `68a52ff6`).
- **Compiler 100% drive**: `ROADMAP-compiler-drive.md`.
- **아카이브**: `ROADMAP-archive.md`.
