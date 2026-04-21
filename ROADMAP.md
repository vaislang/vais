# Vais — Triple Drive: struct-in-array-literal → vaisdb cleanup → Phase 4.x SCOPED

> **버전**: 2026-04-21 triple drive 시작
> **이전 드라이브**: `ROADMAP-cascade-drive.md` ("Cascade & Vec Completion", 4/4 완료)
> **더 이전 드라이브**: `ROADMAP-compiler-drive.md` (11/11 완료)
> **아카이브**: `ROADMAP-archive.md` (Phase 0 ~ 6.31 히스토리)

---

## 드라이브 목적

사용자가 우선순위대로 연속 진행 지정. 3개 축을 순차적으로 완주:

1. **F (struct-in-array-literal fix)** — D.2 자연 후속, compiler 완성도.
2. **G (vaisdb cleanup)** — vaisdb 237/261 개선. 저장소 이동 필요.
3. **H (Phase 4.x SCOPED 심층)** — E034 purity + generic bound.

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
iteration: 1
max_iterations: 25
strategy: F.1 only unblocked — sequential
opus_direct: F.1 codegen-design inseparable — array-literal struct element handling extends D.2's text-backend synthesis pattern; CLAUDE.md rule 4 regression floor requires incremental check

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

### Phase G — vaisdb cleanup drive

- [ ] G.0 — 사용자 확인: vaisdb 저장소 작업 승인 (Opus direct) [blockedBy: F.1]
  **배경**: G 축은 `/Users/sswoo/study/projects/vais/lang/packages/vaisdb` 로
  이동해서 작업. 이 compiler 저장소의 check-integrity 보호 체계가 적용 안 됨.
  F.1 완료 후 사용자에게 다시 확인 (AskUserQuestion):
    - vaisdb cleanup 을 이 compiler 저장소의 harness 로 계속 진행?
    - 아니면 이 드라이브는 F 만 끝내고 H 로 건너뛸지?
    - 아니면 여기서 종료?

- [ ] G.1 — `write_page` arg-count cleanup (9+ 사이트) [blockedBy: G.0]
  target files (vaisdb):
    `sql/catalog/manager.vais` (3), `sql/executor/dml.vais` (3+),
    `graph/mod.vais` (1), `vector/mod.vais` (1+)
  approach: `BufferPool.write_page(frame, flushed)` 2-arg 콜 → 1-arg 로 수정.
  실제 signature 는 vaisdb 쪽에서 확인 후 결정.

- [ ] G.2 — `store.get_node` / `insert_node` / `delete_node` undefined [blockedBy: G.1]
  target files (vaisdb):
    `vector/hnsw/wal.vais` (4+), `vector/hnsw/delete.vais` (3)
  approach: `NodeStore` trait 의 실제 메서드명 확인 → 콜 사이트 수정.

- [ ] G.3 — 기타 vaisdb bugs (lock_node, missing fields, arg counts) [blockedBy: G.2]
  target files (vaisdb):
    `graph/mod.vais`, `rag/mod.vais`, `fulltext/mod.vais`, `vector/mod.vais`,
    `vector/search.vais`, `sql/executor/join.vais`, `sql/executor/sort_agg.vais`,
    `sql/parser/mod.vais`, `sql/parser/parser_expr.vais`,
    `storage/btree/insert.vais`, `storage/txn/deadlock.vais`
  approach: 각 파일의 구체적 bug 를 `docs/vaisdb-cascade-survey.md` §3.2 참조.

- [ ] G.4 — vaisdb 통합 검증 [blockedBy: G.3]
  `./scripts/check-integrity.sh` vaisdb 파트 재실행. 목표 250+/261.
  compiler 저장소 baseline 동일 유지 확인.

### Phase H — Phase 4.x SCOPED 심층

- [ ] H.0 — 사용자 확인: H 진입 여부 (Opus direct) [blockedBy: G.4]
  **배경**: Phase 4.x SCOPED 는 범위 폭발 위험 최고. G 완료 후 vaisdb 상태,
  남은 예산, 사용자 의지 재확인 후 진입.

- [ ] H.1 — E034 purity analysis 정밀도 개선 [blockedBy: H.0]
  target: `crates/vais-types/src/checker_fn.rs` + `partial` keyword 전파.
  목적: `total function may panic` 오류가 실제 panic 가능성에 더 가깝게.
  완료 기준: vaisdb window/manager/bulk/cow 중 몇 개 unblock (측정 후 확정).

- [ ] H.2 — generic bound propagation 확인 [blockedBy: H.1]
  target: `crates/vais-types/src/inference.rs`.
  bulk.vais `S: NodeStore` bound 가 중첩 call 에서 소실되는 이슈.

- [ ] H.3 — 드라이브 최종 완료 선언 [blockedBy: H.2]
  triple drive 종료, 3개 축 통합 결과 기록, 다음 드라이브 후보 제안.

progress: 1/10 (10%)

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
