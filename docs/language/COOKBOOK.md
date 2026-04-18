# Vais COOKBOOK

> **목적**: 에이전트/개발자가 Vais 코드를 쓸 때 **실제로 자주 틀리는 패턴**을 모음. 매 항목은 ❌ 실패 코드 + ✅ 성공 코드 + **왜 실패하는지**를 포함. 각 항목은 `docs/language/LIVING_SPEC/` 의 실행가능 예제와 cross-link됨.
>
> **사용법**: 새 Vais 코드를 작성하기 전 관련 항목 확인. 작업 중 에러가 나면 에러 메시지를 이 문서에서 grep해서 해결법 탐색.
>
> **업데이트 규칙**: 실제 작업 중 발견된 실수 케이스만 추가. 가정으로 만들지 말 것. 모든 ✅ 예제는 `vaisc check` 통과를 확인한 뒤 커밋.

---

## 목차

1. [For-each 루프: `LF i: range` (not `in`)](#1-for-each-루프-lf-i-range-not-in)
2. [Else는 `EL` (not `else`)](#2-else는-el-not-else)
3. [`C`는 Continue (never const)](#3-c는-continue-never-const)
4. [Option match-arm `Some(r.field)` 재포장 금지](#4-option-match-arm-somerfield-재포장-금지)
5. [제거된 키워드 재도입 금지 (`spawn`, `lazy`, `force`)](#5-제거된-키워드-재도입-금지-spawn-lazy-force)
6. [빈 Vec 리터럴 `[]`은 타입 추론 안됨](#6-빈-vec-리터럴-은-타입-추론-안됨)
7. [Vec indexing: `v[i]` vs `v.get(i)`](#7-vec-indexing-vi-vs-vgeti)
8. [Vec `.get(i)`는 `Option<&T>` — deref 필요](#8-vec-geti는-optiont--deref-필요)
9. [Str / &Str / &str — 매개변수 타입 선택](#9-str--str--str--매개변수-타입-선택)
10. [Cross-file impl 분리 금지 (Phase 2.9)](#10-cross-file-impl-분리-금지-phase-29)
11. [`E` 단일 문자 키워드는 애매 — `EN`/`EL` 선호](#11-e-단일-문자-키워드는-애매--enel-선호)
12. [Top-level `const`는 파서 미지원 — `G` 사용](#12-top-level-const는-파서-미지원--g-사용)
13. [Match arm guard `if`는 미지원 — `I/EL`로 대체](#13-match-arm-guard-if는-미지원--iel로-대체)
14. [`Vec<T>` 매개변수는 move — 재사용 전 복제 고려](#14-vect-매개변수는-move--재사용-전-복제-고려)
15. [Self-recursion은 `@` 사용](#15-self-recursion은--사용)
16. [Bool↔int, int↔float 암시적 변환 금지 (Phase 158)](#16-boolint-intfloat-암시적-변환-금지-phase-158)
17. [Struct literal 필드 shorthand 지원됨](#17-struct-literal-필드-shorthand-지원됨)
18. [Ternary `cond ? a : b` 는 지원됨](#18-ternary-cond--a--b-는-지원됨)
19. [Pipe `\|>` 우선 — 중첩 함수 호출 대신](#19-pipe--우선--중첩-함수-호출-대신)
20. [String interp `"{name}"` — 연결 대신](#20-string-interp-name--연결-대신)
21. [함수 타입 `fn(T)->U` 표기법 (파라미터로 함수)](#21-함수-타입-fntu-표기법-파라미터로-함수)
22. [Break with value — 미지원](#22-break-with-value--미지원)

---

## 1. For-each 루프: `LF i: range` (not `in`)

```vais
# ❌ DON'T — 파서 오류
LF i in 0..10 { ... }

# ✅ DO
LF i: 0..10 { ... }
```

**왜 실패하는지**: Vais for-each는 콜론 `:`를 사용. `in` 키워드는 예약되지 않음 (Rust/Python 습관으로 자주 틀림).

**참조**: `LIVING_SPEC/01_keywords/LF_foreach_range.vais`

---

## 2. Else는 `EL` (not `else`)

```vais
# ❌ DON'T — 파서 오류, `else`는 식별자로 취급
I cond { a } else { b }

# ✅ DO
I cond { a } EL { b }

# ✅ DO (체인)
I c1 { a } EL I c2 { b } EL { c }
```

**왜 실패하는지**: Vais는 2-character unambiguous keyword `EL`을 예약. `else`는 lexer가 일반 ident로 분류.

**참조**: `LIVING_SPEC/01_keywords/I_EL_if_else.vais`, `I_EL_chained.vais`

---

## 3. `C`는 Continue (never const)

```vais
# ❌ DON'T — 컴파일 오류 (C는 Continue 토큰)
C MAX: i64 = 100

# ✅ DO — top-level 상수는 G 사용
G MAX: i64 = 100

# ✅ DO — loop의 continue는 C
LF i: 0..10 {
    I i % 2 == 0 { C }
    process(i)
}
```

**왜 실패하는지**: 단일 문자 `C`는 lexer에서 **오직 Continue 토큰**으로 매핑됨. `const` 키워드는 따로 존재하지만 top-level에서 파서 지원 미비 — `G` (global) 사용 권장.

**참조**: `LIVING_SPEC/01_keywords/const_compile_time.vais`, `B_break_continue.vais`, `G_global.vais`

---

## 4. Option match-arm `Some(r.field)` 재포장 금지

```vais
S Profile { age: i64 }

# ❌ DON'T — Phase 2.10 known bug: codegen/TC에서 타입 혼란
F get_age_opt(p: Option<Profile>) -> Option<i64> {
    M p {
        Some(r) => Some(r.age),
        None => None
    }
}

# ✅ DO — 값을 바로 반환하고 바깥에서 감싸거나 직접 사용
F get_age_or(p: Option<Profile>, def: i64) -> i64 {
    M p {
        Some(r) => r.age,
        None => def
    }
}
```

**왜 실패하는지**: `Some(arg)` 생성자가 fresh type var를 할당하는데 arg 타입과 연결이 안 됨 (Phase 2.10 deferred bug, `docs/TYPE_SYSTEM.md §9` 참조). 결과적으로 match arm의 scrutinee 타입이 반환 타입으로 잘못 흘러감.

**참조**: `LIVING_SPEC/02_patterns/match_option_map.vais`, `LIVING_SPEC/05_idioms/idiom_option_chaining.vais`

---

## 5. 제거된 키워드 재도입 금지 (`spawn`, `lazy`, `force`)

```vais
# ❌ DON'T — 모두 제거된 키워드, lexer 오류
spawn { ... }
lazy 42
force expr

# ✅ DO — 대체 패턴
# spawn → 런타임 task API (문서 참조)
# lazy / force → LazyCell-style stdlib
```

**왜 실패하는지**:
- `spawn`: Phase 195 (commit `12592076`)에서 제거. 런타임 task API로 대체.
- `lazy` / `force`: Phase 194 (commit `8c60c075`)에서 제거. stdlib LazyCell 타입으로 대체.

재도입 시 `docs/language/removed_keywords.md` 확인 후 RFC 필요.

**참조**: `docs/language/LEXER_KEYWORDS.md`, `docs/language/removed_keywords.md`

---

## 6. 빈 Vec 리터럴 `[]` — Phase 1.12 ✅ 해결됨

```vais
# ✅ DO — 타입 어노테이션 있으면 Vec<T>로 추론
a: Vec<i64> := []
b: Vec<i64> := [1, 2, 3]

# ✅ DO — Vec::new() 생성자 (타입 어노테이션 없이도 안전)
c: Vec<i64> = Vec::new()
```

**Phase 1.12 변경**: `Stmt::Let`가 `ty` hint를 bidirectional check로 전파. `[...]` 리터럴이 expected type (Array/Vec/Pointer/Slice/Named "Vec") 에 맞게 추론됨.

**참조**: `LIVING_SPEC/04_stdlib/vec_new_push.vais`, `LIVING_SPEC/02_patterns/pattern_empty_vec.vais`

---

## 7. Vec indexing: `v[i]` vs `v.get(i)`

```vais
v: Vec<i64> = Vec::new()
v.push(10)

# v[i] — 직접 indexing, bounds check 없음 (현재 구현에 따라 다름)
# v.get(i) — Option<&T> 반환, 안전

# ✅ 안전한 접근
M v.get(0) {
    Some(val) => *val,  # val은 &i64이므로 deref
    None => 0
}
```

**왜 실패하는지**: `.get(i)`는 `Option<&T>`를 반환. match arm에서 `Some(val)` 바인딩 시 `val: &T`이므로 `*val`로 역참조 필요.

**참조**: `LIVING_SPEC/04_stdlib/vec_get_by_index.vais`, `vec_max.vais`, `vec_sum.vais`

---

## 8. Vec `.get(i)`는 `Option<&T>` — deref 필요

```vais
v: Vec<i64> = Vec::new()
v.push(5)

# ❌ DON'T — `n`은 &i64, 정수 비교에 `expected numeric, found &i64`
M v.get(0) {
    Some(n) => I n > 0 { 1 } EL { 0 },
    None => 0
}

# ✅ DO — 명시적 deref
M v.get(0) {
    Some(n) => {
        cur := *n
        I cur > 0 { 1 } EL { 0 }
    },
    None => 0
}
```

**왜 실패하는지**: Vec의 `.get()`은 `&T` 반환 (데이터 복사 방지). 산술/비교 연산 전에 `*n`으로 역참조.

**참조**: `LIVING_SPEC/04_stdlib/vec_max.vais`

---

## 9. Str / &Str / &str — 매개변수 타입 선택

```vais
# ✅ DO — 일반 문자열 파라미터
F greet(name: str) -> i64 = name.len()

# ✅ DO — 빌림 참조 (값이 복제되지 않음)
F measure(s: &str) -> i64 = s.len()
```

**왜 실패하는지**: Vais는 `str` / `&str` 모두 지원. 차이는 주로 move/borrow 세만틱. 단순 읽기에는 `&str` 또는 `str` 둘 다 OK. 명확하지 않으면 `str`로 시작.

**참조**: `LIVING_SPEC/04_stdlib/str_len.vais`

---

## 10. Cross-file impl 분리 금지 (Phase 2.9)

```vais
# ❌ DON'T — S와 X를 다른 파일에 배치
# foo.vais:
S Counter { count: i64 }

# bar.vais:
X Counter { F bump(self) -> Counter { ... } }
# → foo.vais만 빌드 시 bump() 메서드 TC 실패

# ✅ DO — 같은 파일에 S와 X 배치
# counter.vais:
S Counter { count: i64 }
X Counter {
    F bump(self) -> Counter { Counter { count: self.count + 1 } }
}
```

**왜 실패하는지**: Phase 2.9 decision (option a) — 컴파일러는 단일 파일 단위 빌드 시 다른 파일의 X 블록을 자동으로 찾지 않음. 의도된 설계: 의존성 명시성 + circular import 방지.

**참조**: `LIVING_SPEC/05_idioms/idiom_impl_colocation.vais`, `docs/TYPE_SYSTEM.md §9 Phase 2.9`

---

## 11. `E` 단일 문자 키워드는 애매 — `EN`/`EL` 선호

```vais
# ⚠️ Legacy — `E` 단독은 컨텍스트에 따라 enum/else 양쪽 가능
E Color { Red, Green, Blue }
I cond { a } E { b }

# ✅ DO — 2-char unambiguous keywords
EN Color { Red, Green, Blue }
I cond { a } EL { b }
```

**왜 실패하는지**: `E`는 lexer priority 3, `EN`/`EL`은 priority 4 (higher). 새 코드에서는 unambiguous 버전 사용 권장. Phase 1.7에서 `E` 단독은 deprecation warning 대상.

**참조**: `docs/language/LEXER_KEYWORDS.md`, `docs/LANGUAGE_SPEC.md §Keywords`

---

## 12. Top-level `const` — Phase 1.13 ✅ 해결됨

```vais
# ✅ DO — const 선언 (Phase 1.13부터)
const MAX: i64 = 100

# ✅ DO — C 키워드도 동일 의미 (legacy)
C MIN: i64 = 0

# ✅ DO — 가변/불변 글로벌이 필요하면 G
G counter: i64 = 0
```

**Phase 1.13 변경**: `parse_item`이 `Token::Const`도 `Item::Const`로 라우팅. 기존 `C`와 동일하게 동작.

**참조**: `LIVING_SPEC/01_keywords/const_compile_time.vais`, `G_global.vais`

---

## 13. Match arm guard — `pattern I cond => body` (Vais uses `I`, not `if`)

```vais
# ❌ DON'T — `if`는 식별자, 파서 오류
M n {
    x if x < 0 => -1,
    0 => 0,
    _ => 1
}

# ✅ DO — Vais는 single-char `I` 키워드로 guard
F sign(n: i64) -> i64 {
    M n {
        x I x < 0 => -1,
        0 => 0,
        _ => 1
    }
}
```

**왜 실패하는지**: Vais의 `I`는 if keyword. match guard는 `pattern I <cond> => body` 문법. `if`는 소문자 identifier로 취급되어 파서가 "expected =>" 에러.

**참조**: `LIVING_SPEC/02_patterns/pattern_guard_if.vais`, Phase 1.11 ✅

---

## 14. `Vec<T>` 매개변수는 move — 재사용 전 복제 고려

```vais
F analyze(v: Vec<i64>) -> i64 { v.len() }
F sum(v: Vec<i64>) -> i64 { /* ... */ }

# ❌ DON'T — v가 analyze()로 이동 후 sum()에서 "use after move"
F main() -> i64 {
    v: Vec<i64> = Vec::new()
    v.push(1)
    a := analyze(v)
    b := sum(v)  # E022: use after move
    a + b
}

# ✅ DO — & 참조 사용, 또는 한 쪽에 한 번만 전달
F analyze_ref(v: &Vec<i64>) -> i64 { v.len() }
F sum_ref(v: &Vec<i64>) -> i64 { /* ... */ }

F main() -> i64 {
    v: Vec<i64> = Vec::new()
    v.push(1)
    a := analyze_ref(&v)
    b := sum_ref(&v)
    a + b
}
```

**왜 실패하는지**: Vais는 기본 move semantics. 값 타입 파라미터는 소유권 이동. `E022: use after move` 에러. 참조 `&T`로 빌림.

**참조**: `LIVING_SPEC/06_examples/example_string_processor.vais` (해결 패턴)

---

## 15. Self-recursion은 `@` 사용

```vais
# ❌ DON'T — 함수 이름 변경 시 수동 업데이트 필요
F fact(n: i64) -> i64 {
    I n <= 1 { 1 } EL { n * fact(n - 1) }
}

# ✅ DO — @ 자동 self-reference
F fact(n: i64) -> i64 {
    I n <= 1 { 1 } EL { n * @(n - 1) }
}
```

**왜 실패하는지**: Vais의 `@` 연산자는 현재 함수를 가리킴. 리팩토링 안전성 + 토큰 효율.

**참조**: `LIVING_SPEC/05_idioms/idiom_self_recursion_at.vais`

---

## 16. Bool↔int, int↔float 암시적 변환 금지 (Phase 158)

```vais
# ❌ DON'T — TC 오류
F wrong(b: bool) -> i64 = b  # bool은 i64로 자동 변환 안 됨
F wrong2(x: i64) -> f64 = x  # i64 → f64 암시적 변환 금지

# ✅ DO — `as`로 명시
F ok(b: bool) -> i64 = b as i64
F ok2(x: i64) -> f64 = x as f64
```

**왜 실패하는지**: Phase 158 요요 방지 — Rust-style strict coercion 확정. 허용되는 암시 변환은 정수 widening (i8→i16→i32→i64, u 계열 동일)과 float 리터럴 추론뿐. 나머지는 `as` 명시 필수.

**참조**: `CLAUDE.md §"Type Conversion Rules"`, `LIVING_SPEC/05_idioms/idiom_cast_explicit.vais`

---

## 17. Struct literal 필드 shorthand 지원됨

```vais
S Point { x: i64, y: i64 }

F make(x: i64, y: i64) -> Point {
    # ✅ DO — 명시적
    Point { x: x, y: y }

    # 주의: 일부 shorthand는 파서 지원 미비. 명시 형태 권장.
}
```

**왜 실패하는지**: Vais는 Rust-style shorthand를 일부 지원하나, 모호성 회피를 위해 명시 권장.

**참조**: `LIVING_SPEC/05_idioms/idiom_struct_lit_shorthand.vais`

---

## 18. Ternary `cond ? a : b` 는 지원됨

```vais
# ✅ DO — 간결한 분기
F abs(n: i64) -> i64 = n < 0 ? -n : n

# ✅ DO — I/EL과 동일 의미, 긴 블록은 I/EL 선호
F sign(n: i64) -> i64 = n > 0 ? 1 : n < 0 ? -1 : 0
```

**참고**: `cond ? a : b`는 Rust에 없지만 Vais는 채택. C/JS 습관 있는 사람에게 친숙.

**참조**: `LIVING_SPEC/05_idioms/idiom_ternary.vais`

---

## 19. Pipe `|>` 우선 — 중첩 함수 호출 대신

```vais
F add1(x: i64) -> i64 = x + 1
F double(x: i64) -> i64 = x * 2
F sq(x: i64) -> i64 = x * x

# ❌ DON'T — 안쪽부터 읽어야 함
result := sq(double(add1(3)))

# ✅ DO — 좌→우 흐름
result := 3 |> add1 |> double |> sq
```

**왜 권장되는지**: 데이터 변환 파이프라인은 좌→우 읽기가 자연스러움.

**참조**: `LIVING_SPEC/04_stdlib/pipe_operator.vais`, `LIVING_SPEC/05_idioms/idiom_pipe_transform.vais`

---

## 20. String interp `"{name}"` — 연결 대신

```vais
name := "world"

# ✅ DO — 보간
msg := "hello {name}"

# ❌ 주의 — 일부 언어의 `+` 연결은 Vais에서 타입 호환이 다름
# msg := "hello " + name  # 직접 연결 대신 보간 권장
```

**왜 권장되는지**: `"{expr}"` 구문은 문자열 + 표현식 혼합을 컴파일러가 안전하게 타입 체크.

**참조**: `LIVING_SPEC/04_stdlib/str_interp.vais`, `LIVING_SPEC/05_idioms/idiom_string_interp.vais`

---

## 21. 함수 타입 — `(T) -> U` 또는 `|T| -> U` (Phase 1.15 ✅)

```vais
# ❌ DON'T — `F` 대문자는 function declaration keyword, 타입 표기 아님
F apply<T>(val: T, f: F(T) -> i64) -> i64 = f(val)

# ✅ DO — 괄호 문법
F apply<T>(val: T, f: (T) -> i64) -> i64 { f(val) }

# ✅ DO — 파이프 문법 (클로저처럼)
F apply2<T>(val: T, f: |T| -> i64) -> i64 { f(val) }

F double(x: i64) -> i64 { x * 2 }
F main() -> i64 { apply(5, double) }  # 10
```

**Phase 1.15 확인**: parser `parse_base_type`는 `(T1, T2) -> U`와 `|T1, T2| -> U` 둘 다 `Type::Fn`으로 파싱. `F` 대문자는 function declaration keyword라서 타입 위치에 올 수 없음.

**참조**: `LIVING_SPEC/03_generics/generic_higher_order.vais`

---

## 22. Break with value — Phase 1.14 ✅ Parser + TC 지원

```vais
# ✅ DO — B <expr>로 loop-as-expression
result := L { B 42 }  # result: i64 = 42

# ✅ DO — 조건부 break
result := L {
    I done { B 42 } EL { 0 }
}
```

**Phase 1.14 변경**: Parser는 이미 `B <expr>` 허용. TC에 `collect_break_value_type`
helper 추가 — 현재 loop 레벨의 모든 `Stmt::Break(Some(e))`를 수집하고 타입 통합.

**주의**: 복잡한 loop-as-expression의 codegen (phi node 등)은 Phase 3.x 완결성
작업에서 추가 완성 필요. 단순 케이스는 이미 작동.

**참조**: `LIVING_SPEC/01_keywords/B_break_continue.vais`, `L_loop_break.vais`

---

## 기여 가이드

이 COOKBOOK 추가 항목:

1. **실제 작업 중 발견된 실수만** 추가. 가정으로 새 항목 만들지 말 것.
2. ❌ 실패 코드는 반드시 컴파일 오류 / TC 오류 메시지 포함 (주석으로).
3. ✅ 성공 코드는 `docs/language/LIVING_SPEC/`의 파일을 참조하거나 실제로 검증.
4. "왜 실패하는지"는 lexer/parser/TC 구현에 근거한 사실만 (추측 금지).
5. 변경 후 `./scripts/check-integrity.sh` 실행으로 regression 없음 확인.
