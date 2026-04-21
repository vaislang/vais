# Vais — Post-Compiler Drive: Cascade & Vec Completion

> **버전**: 2026-04-21 신규 드라이브 시작
> **이전 드라이브**: `ROADMAP-compiler-drive.md` ("문법 + 컴파일러 100%", 11/11 완료, 커밋 `e1697c14`)
> **이전 아카이브**: `ROADMAP-archive.md` (Phase 0 ~ 6.31 히스토리)

---

## 드라이브 목적

직전 드라이브가 구조적 컴파일러 gap 11개 (A.1~A.3, B.1~B.6, C.1~C.2) 를 닫았다. 이 드라이브의 목적은 두 가지:

1. **그 fix 들이 vaisdb 에 실제로 cascade 했는지 측정 + 남은 blocker 분류.**
   baseline vaisdb=237/261 이 B.1/B.4 덕에 올라갔을 가능성이 크지만 측정 안 된 상태.
2. **B.4 의 실제 활용을 막고 있는 Vec&lt;T&gt; 리터럴 lowering gap 해소.**
   `v: Vec&lt;T&gt; := [...]` 가 Vec 구조체의 data/len/cap/elem_size 중 `data` 만 초기화 →
   `v[1]` 이 garbage. 이 드라이브의 B.4 write-through 와 세트로 마무리 필요.

드라이브 3번 작업은 1번 측정 결과에 따라 정한다 (세부는 D.3 블록 참조).

---

## Baseline (2026-04-21 드라이브 시작 시점 실측)

Commit `e1697c14` 기준 `./scripts/check-integrity.sh`:

| 항목 | 숫자 | 비고 |
|------|------|------|
| compiler_syntax | 200/200 | 직전 드라이브 완료 |
| compiler_stages | 14/14 | 1 #[ignore] B7 known bug |
| std_files | 82/82 | 100% |
| living_spec | 112/112 | B.1/B.2/B.3 reproducer 추가 완료 |
| phase158 strict | 18/18 | 100% |
| vaisdb_files | 237/261 | 90.8% — **이 드라이브에서 측정/개선** |
| vaisc e2e | 2625/0/1 | B.5 defer edge case 3 추가 완료 |
| vaisc integration | 147/147 | 100% |

이 baseline 이 모든 작업의 **regression floor**. 1-file 감소도 허용하지 않음 (단, vaisdb 는 이 드라이브에서 **상승**이 목표 — 감소만 regression).

---

## 설계 원칙

1. **측정 후 결정**: D.1 은 측정 전용. 결과를 보고 D.3 의 정체를 정한다. 측정 없이 추측 금지.
2. **CLAUDE.md rule 3/4 엄수**: 수정 전 baseline 기록, regression 1건이라도 발생 시 즉시 revert.
3. **Scope 제한**: stdlib 확장 / Phase 4.x SCOPED 심층 구현 / 문서 튜토리얼은 이 드라이브 범위 밖.
4. **매트릭스 동기화**: `LANGUAGE_SPEC.md` Construct Status Matrix 를 작업 완료 시 갱신.

---

## Current Tasks (2026-04-21)

mode: auto
iteration: 0
max_iterations: 15

### Phase D — Post-compiler follow-up

- [ ] D.1 — vaisdb cascade 측정 + blocker 분류 (Opus direct, measurement)
  target: 현재 vaisdb 237/261 의 남은 24개 failure 를 실측 분류.
  approach:
    1. 직전 드라이브 fix 들이 반영된 현 바이너리로 `./scripts/check-integrity.sh` 의
       vaisdb 항목을 재실행 + 상세 로그 수집. 실측 N/261 확정.
    2. 237 에서 상승 했으면 (예: 248/261) 어떤 파일이 cascade 되었는지 diff. 어떤
       B.x fix 가 가장 큰 영향이었는지 기록.
    3. 남은 failure 각각에 대해:
       (a) 컴파일러 gap (C00x codegen/TC error 포함)
       (b) vaisdb 자체 코드 버그 (Vais 코드가 잘못됨)
       (c) 회피 불가능한 Phase 4.x SCOPED 의존
       으로 분류.
    4. (a)/(b)/(c) 별 개수 + 대표 예시 3개씩 을 `docs/vaisdb-cascade-survey.md` 에 기록.
  [완료 기준]:
    - vaisdb 실측 숫자 기록 (237/261 이상)
    - 남은 failure 의 (a)/(b)/(c) 분류 완료
    - `docs/vaisdb-cascade-survey.md` 작성
    - compiler baseline 감소 0건
  blocker: D.3 선택이 D.1 결과에 의존 (blocks D.3).

- [ ] D.2 — Vec<T> literal lowering + Vec::new/push inkwell dispatch (Opus direct)
  **배경**: B.4 (2026-04-21 직전 드라이브) 가 `v[i].field = x` write-through 를 구현했지만
  Vec 리터럴 `v: Vec<T> := [...]` 가 data slot 만 채우고 len/cap/elem_size 를 초기화
  하지 않아 `v[1]` 가 garbage. 또한 `Vec::new()` / `v.push(x)` 는 inkwell 에서 C002.
  target:
    - `crates/vais-codegen/src/inkwell/gen_aggregate.rs::generate_array` (Vec 리터럴)
    - `crates/vais-codegen/src/inkwell/gen_expr/call.rs` 또는 신규 `builtins/vec_methods.rs`
      (Vec::new, push, pop, len, ...)
  approach:
    1. `v: Vec<T> := [a, b, c]` 타입 annotation 감지 시 generate_array 가
       `{ data: ptr as i64, len: 3, cap: 3, elem_size: sizeof(T) }` 완전 초기화.
       heap-alloc 대신 stack alloca 사용 (기존 배열 리터럴과 일관, O(1)).
    2. `Vec::new()` (static method call) 를 inkwell 에서 `{ 0, 0, 0, sizeof(T) }` 로.
       sizeof(T) 는 T 를 어떻게 얻느냐가 관건 — StaticMethodCall 의 type_name 에서
       generic T 추출 → `type_mapper.map_type(T).get_store_size()` 로 결정.
    3. `v.push(x)` 는 runtime realloc 필요 → 먼저 **fixed-cap** 버전으로 구현
       (cap 초과 시 panic/abort). dynamic realloc 은 Phase D.2b 로 분리.
    4. B.2 의 str method dispatch 패턴 재사용: receiver 가 `Named { name: "Vec", ... }`
       일 때 inline intrinsic lookup.
  [완료 기준]:
    - `v: Vec<i64> := [10, 20, 30]; v[1]` → 20 (현재 garbage)
    - `v: Vec<Point> := [Point{x:1,...}, Point{x:3,...}]; v[0].x = 99; v[0].x + v[1].x`
      → 99 + 3 = 102
    - `v: Vec<i64> = Vec::new(); v.push(5); v[0]` → 5
    - LIVING_SPEC 에 재현 3건 추가 (`vec_literal_init`, `vec_struct_idx_assign`, `vec_new_push`)
    - e2e 4+ 추가 pass
    - compiler baseline 유지 (vaisdb 는 상승 가능)
    - LANGUAGE_SPEC Matrix L262 `Vec<Struct>[i].field =` Run 컬럼 ◐→✓ 업그레이드
  scope 한계: push 는 fixed-cap. dynamic realloc (realloc 호출) 은 Phase D.2b.
  blockedBy: D.1 (D.1 의 측정 결과가 D.2 범위를 확정 — 만약 vaisdb failure 의 대다수가
    Vec-관련이면 D.2 가 첫 우선순위 후보였음을 실측으로 확인; 아니면 D.2 는 그대로 두고
    D.3 을 다른 타깃으로).

- [ ] D.3 — TBD from D.1 findings (모델 TBD) [blockedBy: D.1, D.2]
  **이 작업은 D.1 측정 결과로 정의됨**. D.1 완료 후 이 블록을 채운다.
  후보 (D.1 결과에 따라 선택):
    (α) vaisdb 에 남은 (b) "vaisdb 자체 버그" 가 많으면 → vaisdb 코드 cleanup drive 로
        이 ROADMAP 스코프 밖으로 이관. D.3 는 "이 드라이브 완료 선언" 으로 축소.
    (β) vaisdb 에 남은 (a) "compiler gap" 중 공통 패턴이 있으면 → 그 패턴을 묶어
        D.3 로 만듬 (예: HashMap<K, V> 메서드 dispatch, Slice write-back, 등).
    (γ) 남은 것이 전부 (c) SCOPED 의존이면 → D.3 는 "SCOPED 범위 외 문서 업데이트" 만.

### Phase E — 드라이브 완료

- [ ] E.1 — 드라이브 완료 선언 + 다음 드라이브 후보 제안 (Opus direct) [blockedBy: D.3]
  target: 이 ROADMAP 을 archive 로, 다음 드라이브 후보를 README/ROADMAP 초안에 기록.
  [완료 기준]:
    - vaisdb 최종 실측 숫자 반영
    - LANGUAGE_SPEC Matrix 최종 상태 반영
    - 다음 드라이브 후보 3개 이상 기록 (stdlib 확장, vaisdb 자체 cleanup, Phase 4.x 심층,
      etc.)
    - baseline 유지

progress: 0/4 (0%)

---

## Gate 기준

- **Phase D 완료 조건**: D.1~D.3 ✓, vaisdb 숫자 확정, compiler baseline 감소 0건.
- **Phase E 완료 조건**: 완료 선언 commit + 다음 드라이브 제안.
- **즉시 revert 조건**: 각 task 후 `./scripts/check-integrity.sh` 에서 1-file regression 감지 시
  (단, vaisdb 의 **상승**은 regression 아님).

---

## Archive / 별도 드라이브

- **직전 드라이브 (문법+컴파일러 100%)**: `ROADMAP-compiler-drive.md`. 11/11 완료. 커밋 `e1697c14`.
- **이전 아카이브**: `ROADMAP-archive.md` (Phase 0~6.31 히스토리).
- **stdlib 확장 / Phase 4.x SCOPED 심층 / 문서 튜토리얼**: 이 드라이브 스코프 밖. 별도 세션.
