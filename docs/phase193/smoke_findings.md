# Phase 193 Recon-C — 실전 스모크 S1~S5 발굴 결과

**수집일**: 2026-04-17  
**baseline commit**: 4066ab9d (Phase 192 complete)  
**E2E 상태 (baseline)**: 2596/0/0, clippy 0/0  
**방법**: examples/phase193_smoke/ 에 S1~S5 스모크 작성 후 `./target/release/vaisc <f>.vais` + `./<f>` 실행

---

## 스모크 결과 요약

| # | 이름 | 시나리오 | 컴파일 | 런타임 | 판정 |
|---|---|---|---|---|---|
| S1 | struct + str baseline | `S{name: str, age: i64}` 생성/접근 | ✅ | ✅ | PASS |
| S2 | Vec<Struct{str}> + match | `Vec<Person>.push + .get + match` | ❌ | — | FAIL (type inference) |
| S2b | Vec<Struct{str}> no-get | push + len만 | ❌ | — | FAIL (symbol) |
| S3 | closure over Vec<Struct{str}> | S2 + closure | ❌ | — | FAIL (type inference) |
| S3b | closure str capture | `greeting := "X"; \|n: str\| puts("{greeting}, {n}")` | ❌ | — | FAIL (capture) |
| S3c | closure int capture | `n := 42; \|\| puts("{n}")` | ❌ | — | FAIL (capture) |
| S4 | async + struct{str} | `A F f(p: Person)` + `.await` | ✅ | ✅ | PASS |
| S5 | complete path | (의존성 때문에 미실행) | — | — | BLOCKED |

**통과 2건 / 실패 5건 / 블록 1건** — 핵심 실전 시나리오 대부분 현 시점에서 불가.

---

## 신규 발굴 구멍 (Phase 193 승격 대상)

### R-1: `Vec_with_capacity` 심볼 정의 누락 (REGRESSION, 심각)

**증상**: `Vec.with_capacity(N)` 호출이 포함된 소스를 fresh 컴파일(캐시 삭제 후)하면 clang이 `use of undefined value '@Vec_with_capacity'`로 거부.

**재현**:
```bash
rm -rf examples/.vais-cache
./target/release/vaisc examples/simple_vec_test.vais
# → error: clang compilation failed for module 'vec':
#   examples/.vais-cache/vec.ll:333:19: error: use of undefined value '@Vec_with_capacity'
```

**주의**: 기존 캐시 바이너리 `./examples/simple_vec_test`는 정상 실행(exit 142). 즉 **한 번 빌드된 바이너리는 동작**하므로 개발자가 평소 개발 중에는 놓치기 쉬운 회귀. E2E 2596/0/0이 녹색인 이유도 여기 있음(테스트 캐시/격리 경로에서 우회).

**원인 추정**: Phase 192 #1 Group A (commit e260c893) "stdlib Vec elem_size patch 가드" 변경 중 stdlib `Vec.with_capacity` body가 경우에 따라 emit 스킵되도록 바뀐 것으로 의심. 정확한 bisect 필요.

**영향 범위**: `Vec.with_capacity`를 호출하는 모든 example 파일 (최소 `simple_vec_test.vais`, `generic_vec_test.vais`, `phase193_smoke/S2*`). 사용자가 hello world 넘어서면 즉시 만나는 회귀.

**권장 처리**: **Group-I 최우선**. task #4 블록의 첫 번째 수정 대상.

**수정 포인트 후보**:
- `crates/vais-codegen/src/module_gen/instantiations.rs` (stdlib Vec emit 경로)
- `crates/vais-codegen/src/expr_helpers_call/method_call.rs` (elem_size patch 가드가 with_capacity 자체를 건너뛰는지)
- bisect: `git bisect` 범위 = `39922874..4066ab9d` (Phase 192 세 커밋 + 직전)

---

### R-2: Closure가 외부 변수를 캡처하지 못함 (REGRESSION, 심각)

**증상**: 외부 스코프의 변수를 클로저에서 참조하면 `error[C001] Undefined variable`.

**재현**:
```vais
F main() -> i64 {
    n := 42
    show := || puts("n = {n}")   # → "Undefined variable: n"
    show()
    0
}
```

**비교 (통과하는 케이스)**:
```vais
# E2E execution_tests::exec_closure_capture_multiple 통과
F main() -> i64 {
    a := 10
    b := 20
    f := |x| a + b + x   # ← 람다 parameter 있음 + 표현식 반환
    f(12)                # 42
}
```

**차이 가설**: 
- 0-param 클로저(`||`) + **문자열 보간(`{n}`)** 내부에서의 capture가 TC 스코프에 보이지 않음
- 또는 `puts(...)` call statement 내부에서 쓰이면 capture 분석이 누락

**원인 추정**: Phase 191 #4 commit f29993d7 "closure str capture clone-on-capture" 이후 **capture 전파**가 일부 경로에서 누락됐을 가능성. clone-on-capture가 `puts` 호출 경로를 경유하는 capture 분석을 우회하는 것으로 의심.

**권장 처리**: **Group-III 최우선 재정의**. Recon-A에서는 Group-III 0건이었으나 이 발굴로 최소 1건 확정.

**수정 포인트 후보**:
- `crates/vais-codegen/src/closure.rs` (capture 수집 경로)
- `crates/vais-types/src/checker_expr.rs` (closure body 내부 name resolution)
- `crates/vais-parser/src/expr.rs` (string interpolation의 `{name}` 파싱이 scope annotation 남기는지)

---

### R-3: Function parameter의 pipe-style closure type 구문 파싱 실패

**증상**: `F apply_n(f: |i64| -> i64, ...)` 선언이 `found Pipe, expected type name`으로 파싱 실패.

**재현**:
```bash
./target/release/vaisc examples/closure_counter.vais
# → error at line 6 col 14:
#   F apply_n(f: |i64| -> i64, x: i64, n: i64) -> i64 {
#                ^ found Pipe, expected type name
```

**비교**: E2E `exec_closure_basic`는 `(i64) -> i64` (괄호) 표기만 사용. Pipe 표기는 여전히 같은 소스의 6번째 줄에서 파싱 실패.

**예상 원인**: 파서의 type 파싱에서 pipe-closure type을 수용하지 않음. 주석 CLAUDE.md에는 `|T|` closure 구문이 명시돼 있지만 **함수 parameter type 위치**에서는 파서가 거부.

**권장 처리**: Group-III 에 포함 (closure 생태계). parser 수정 범위.

**수정 포인트 후보**:
- `crates/vais-parser/src/types.rs` (type 파서 시작 지점에 pipe-closure 분기 추가)

---

### R-4: Vec<Struct> `.get()` 후 `Some(p)` 매치에서 p 타입 미전파

**증상**: 
```vais
opt := v.get(i)           # v: Vec<Person>
M opt {
    Some(p) => p.age,     # → "no field 'age' on type '?'"
    None => 0
}
```

**가설**: `Vec<T>::get`의 반환 `Option<T>`에서 `T = Person` specialization이 match arm의 `Some(p)` 바인딩에 전파 안 됨.

**권장 처리**: **Group-I 에 포함** (generic specialization 잔여). Recon-A #8/#9와 같은 영역.

**수정 포인트 후보**:
- `crates/vais-types/src/checker_expr.rs` (match arm의 pattern binding type inference)
- `crates/vais-types/src/inference.rs` (generic return type substitution)

---

## 통과 관찰 (baseline 유지 확인)

- **S1** (struct + str): 완전 정상. Phase 191 string ownership 모델 안정.
- **S4** (async + struct + str, `.await` 후 field access): 완전 정상. commit 39922874의 async poll 수정이 효과적.

---

## Recon-A/B와 중복 제거 후 최종 Phase 193 수정 대상

| ID | 영역 | 출처 | task |
|---|---|---|---|
| A1 | Vec<T> param indexing null data ptr | Recon-A #8/#9 | #4 Group-I |
| A2 | Vec<i32/u8> trunc i64 to i32 mismatch | Recon-A #11 | #4 Group-I |
| B1 | Vec→slice coercion runtime | Recon-A #2 | #5 Group-II |
| **C1** | **Vec_with_capacity emit 누락 (R-1)** | Recon-C | **#4 Group-I (최우선)** |
| **C2** | **Closure outer capture 실패 (R-2)** | Recon-C | **#6 Group-III (최우선)** |
| **C3** | **Pipe closure type parse fail (R-3)** | Recon-C | #6 Group-III |
| **C4** | **Vec.get()→Some(p) type 미전파 (R-4)** | Recon-C | #4 Group-I |

총 **7건 실제 한계** (Recon-A 5건 + Recon-C 신규 발굴 4건 중 중복 제거 2건 — R-1/R-4는 A1/A2와 메커니즘 유사하므로 같은 Group에 묶음).

**Recon-B still_open 1건**(Apr 17)은 위 C1/C2 중 하나가 그 세션의 build_test_fail 원인으로 추정 → 추가 task 불필요.

---

## Phase 193 task 재라우팅

| task | 기존 스코프 | 재정의 스코프 |
|---|---|---|
| #4 Group-I | generic specialization 잔여 (A1, A2) | + C1, C4 = **4건** |
| #5 Group-II | struct ownership/drop (B1) | 변동 없음 = **1건** |
| #6 Group-III | closure str capture | **C2, C3 = 2건 (신규 scope)** |
| #7 Group-IV | async poll/state machine | **0건 → task delete 또는 보호 테스트만** |

**권고**: task #7은 현재 발견된 구멍이 없으므로 **"async regression guard" 테스트 추가**로 축소 (S4 + 변형 3~4건을 E2E에 추가). 수정 대상 없음.

---

## 엔지니어링 메타 관찰

1. **E2E 녹색인데 실 사용 회귀**: `exec_closure_capture_multiple`는 통과하지만 `closure_counter.vais` example은 파서 레벨에서 실패. E2E가 **파싱 가능한 하위 집합**만 커버. Phase 193 Final Gate에 **examples/ 전수 rebuild 테스트** 추가 필요.
2. **Cache 은닉**: `.vais-cache/` 삭제 없이 재빌드하면 기존 바이너리로 "성공"처럼 보이는 UX 함정. CI에서는 캐시 없이 돌아야 함.
3. **Recon-A 커버리지 부족**: 읽기만으로는 C1/C2/C3 같은 파서/emit 회귀 놓침. 실전 스모크가 결정적이었음.

---

PROMISE: COMPLETE
