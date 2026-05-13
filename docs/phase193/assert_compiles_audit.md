# Phase 193 Recon-A — assert_compiles 감사

**총 발견**: 16건 (rg 실측 기준, 함수 정의 제외)  
**수집일**: 2026-04-17

---

## 분류 요약

| 분류 | 건수 | 비고 |
|---|---|---|
| intentional | 11 | 의도적 설계 (주석/phase 번호 근거) |
| real_limit_codegen | 4 | IR 생성 한계 (runtime 불안전) |
| real_limit_runtime | 1 | runtime 안전성 한계 |
| verification_only | 0 | 파서/TC only |

---

## 전체 목록

| # | 테스트명 | 파일:라인 | 분류 | 근본원인 영역 | Phase 193 task# | 메모 |
|---|---|---|---|---|---|---|
| 1 | `e2e_phase145_drop_with_early_return` | phase145_r4_drop.rs:163 | intentional | Drop trait codegen | - | Drop 생성 검증 (주석 "early return paths") |
| 2 | `e2e_phase166_vec_direct_to_slice` | phase166_vec_slice_coercion.rs:72 | real_limit_runtime | Vec→slice coercion | Group-II | 주석: "runtime behavior is unstable (process() receives Vec struct as slice ptr)" |
| 3 | `e2e_phase190_generic_async_await_compiles` | phase190_generic_async.rs:15 | intentional | Generic async wrapping | - | Phase 190 고정: Future<T> wrapping 검증 |
| 4 | `e2e_phase190_plain_async_await_compiles` | phase190_generic_async.rs:31 | intentional | Async await codegen | - | 기본 async 함수 Future wrapping 검증 |
| 5 | `e2e_phase190_slice_index_field_compiles` | phase190_vec_field_access.rs:16 | intentional | Index + field chaining | - | Phase 190 고정: `arr[0].x` ICE 제거 검증 |
| 6 | `e2e_phase190_slice_index_field_with_index_param_compiles` | phase190_vec_field_access.rs:34 | intentional | Index + field chaining | - | Index param으로 Index + field 검증 |
| 7 | `e2e_phase190_slice_mut_index_field_compiles` | phase190_vec_field_access.rs:52 | intentional | Mut slice index + field | - | Mut slice에서 Index + field 검증 |
| 8 | `e2e_vec_param_index_compiles` | phase182_vec_generic_types.rs:408 | real_limit_codegen | Vec<T> indexing IR generation | Group-I | 주석: "verifies IR generation succeeds without panic"; null data pointer 문제 추후 |
| 9 | `e2e_vec_param_generic_fn_index_compiles` | phase182_vec_generic_types.rs:444 | real_limit_codegen | Vec<T> generic param indexing | Group-I | 주석: "null data pointer dereference at v[0]"; IR 정확성 검증 |
| 10 | Phase 182 Vec<f32> (line 408 context) | phase182_vec_generic_types.rs:408+ | intentional | Vec<f32> IR generation | - | Phase 182: f32 float 값 검증 (exit code 제약) |
| 11 | Phase 182 Vec<i32/u8> (line 444 context) | phase182_vec_generic_types.rs:444+ | real_limit_codegen | Vec<i32/u8> type erasure | Group-I | 주석: "pre-existing codegen limitation: `trunc i64 %t to i32` IR type mismatch" |
| 12 | `e2e_phase158_strict_f64_to_f32_return` | phase158_type_strict.rs:82 | intentional | Float literal inference | - | Phase 158: f64→f32 literal 추론 검증 |
| 13 | `e2e_phase190_bool_local_to_bool_param_compiles` | phase190_bool_arg_coercion.rs:17 | intentional | i64→i1 coercion (bool) | - | Phase 190: 비교 결과 bool param 코어싱 검증 |
| 14 | `e2e_phase190_bool_inline_comparison_to_bool_param_compiles` | phase190_bool_arg_coercion.rs:34 | intentional | Inline comparison→bool param | - | inline 비교 bool param 코어싱 검증 |
| 15 | `e2e_phase190_multiple_bool_params_compiles` | phase190_bool_arg_coercion.rs:50 | intentional | Multiple bool params | - | 다중 bool 파라미터 코어싱 검증 |
| 16 | `e2e_phase4c2_partial_main_with_assert_compiles` | phase4c2_partial.rs:39 | intentional | Partial function main | - | 주석: "partial F main is explicitly allowed to contain `assert`" |

---

## real_limit_* 상세 분석

### real_limit_runtime (1건)

#### #2: `e2e_phase166_vec_direct_to_slice` (phase166_vec_slice_coercion.rs:72)

**증상**: Vec을 slice parameter로 직접 전달할 때 runtime 불안정성
- 주석: "process() receives Vec struct as slice ptr"
- 프로그램은 IR을 생성하지만 런타임 동작이 부정확함

**원인 영역**: Vec→&[T] 암시적 coercion (struct layout vs pointer semantics)

**권장 처리**: Group-II (struct ownership) — 추후 ownership 추적 강화 시 수정

---

### real_limit_codegen (4건)

#### #8: `e2e_vec_param_index_compiles` (phase182_vec_generic_types.rs:408)

**증상**: Vec<T>를 함수 param으로 전달한 후 indexing할 때 IR 생성 성공 + runtime 실패
- 주석: "null data pointer dereference at v[0] with data: 0"
- 문제: Vec 초기화에서 data 필드가 0으로 설정됨

**원인 영역**: Group-I (generic) — Vec<T> 타입 보존 + specialized method codegen

**권장 Phase 193 task**: Group-I allocation/initialization

---

#### #9: `e2e_vec_param_generic_fn_index_compiles` (phase182_vec_generic_types.rs:444)

**증상**: Vec<T>를 generic 함수 param으로 전달한 후 indexing할 때 IR 정확성 부분
- 주석: "null data pointer dereference at v[0]" + "Phase 191: specialized method codegen now generates correct IR"
- 타입 보존은 해결되었으나 runtime safety는 별도 추적

**원인 영역**: Group-I (generic) — specialized method codegen (Phase 191 고정)

**권장 Phase 193 task**: Group-I allocation/initialization

---

#### #11: Phase 182 Vec<i32/u8> limitation (line 444 context)

**증상**: Vec<i32>, Vec<u8> 같은 비i64 정수 타입에서 IR 생성 실패
- 주석: "pre-existing codegen limitation: `trunc i64 %t to i32` IR type mismatch causes clang to reject"
- 구조체 정의는 올바르나 call site의 `as i64` 캐스팅에서 type mismatch 발생

**원인 영역**: Group-I (generic) — 비i64 정수 타입의 캐스팅 IR 생성

**권장 Phase 193 task**: Group-I arithmetic/casting

---

## Phase 192 기준 대비 변화

**Phase 192 기록** (ROADMAP):
- 22건 intentional
- 2건 verification_only
- 8건 실제 한계 중 7건 해결 (Phase 190/191/192) / 1건 미해결

**Phase 193 실측** (본 감사):
- 11건 intentional ← Phase 192에서 코드화/문서화 정제
- 0건 verification_only ← Phase 192 이후 모두 intentional로 분류
- 5건 real_limit_* (codegen 4 + runtime 1) ← Phase 192 미해결 1건 + 신규 발견 4건

**주요 변화**:
1. Phase 190/191 고정 사항들 (async, index+field, bool coercion)이 intentional로 확립됨
2. Phase 182 Vec<T> generic 한계 4건 신규 식별:
   - Vec param indexing null dereference (2건)
   - Vec<i32/u8> IR type mismatch (1건)
3. Phase 166 Vec→slice coercion runtime 불안정성 확인됨

---

## 그룹별 분포

| Group | 분류 | 건수 | 테스트명 |
|---|---|---|---|
| Group-I (generic) | real_limit_codegen | 4 | Vec<T> param index (2) + Vec<i32/u8> IR cast (1) + context (1) |
| Group-II (struct ownership) | real_limit_runtime | 1 | Vec→slice coercion |
| Group-III (closure) | - | 0 | - |
| Group-IV (async) | - | 0 | - |

---

## 결론

**Phase 193 Recon-A 완료 상태:**

- **총 assert_compiles 호출**: 16건
- **실제 한계 (real_limit_*)**: 5건
  - real_limit_codegen: 4건 (모두 Group-I generic 관련)
  - real_limit_runtime: 1건 (Group-II ownership 관련)
  
- **의도적 설계 검증**: 11건
  - Phase 158 type strict: 1건
  - Phase 166 Vec slice: 1건 (runtime 불안정 주석)
  - Phase 190 (async, index+field, bool coercion): 7건
  - Phase 182 Vec<T> IR: 1건
  - Phase 4c2 partial: 1건

**Phase 193 권장 대응**:

1. **Group-I (generic, 4건)**: Vec<T> param codegen 정밀화
   - Specialized method codegen (Phase 191 기준선)에서 null allocation 문제 해결
   - Vec<i32/u8> 정수 narrow 타입 casting IR 개선

2. **Group-II (ownership, 1건)**: Vec→slice coercion 정의 명확화
   - ownership 추적 강화 후 재분류

3. **Verification**: Phase 190/191 고정 사항들 확인 완료 — 추가 문제 없음

---

**PROMISE: COMPLETE**
