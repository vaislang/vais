# Vais 코어 언어 설계 원칙

**버전:** 1.0.0
**날짜:** 2026-01-12

---

## 개요

이 문서는 Vais 코어 언어의 설계 원칙을 정의합니다.
코어는 **절대 변경되어서는 안 되는** 부분이므로 신중하게 설계해야 합니다.

---

## 코어 철학

### 1. 최소하지만 완전함

```
코어에 포함:
✅ 기본 타입 (i, f, s, b, [T], {K:V}, ?T)
✅ 기본 연산자 (+, -, *, /, %, ==, !=, <, >, 등)
✅ 제어 흐름 (조건식, 루프 표현식)
✅ 함수 정의/호출
✅ 모듈 시스템 기본
✅ 에러 처리 기본

코어에 미포함:
❌ 파일 I/O (std.io)
❌ 네트워킹 (std.net)
❌ 데이터 포맷 (std.json)
❌ 고급 자료구조 (std.collections)
❌ 비동기 (std.async)
```

### 2. 직교 설계

각 기능은 독립적이고 조합 가능해야 합니다.

```vais
# 모든 것이 표현식
result = if cond { a } else { b }  # 조건문은 표현식
value = { let x = 1; x + 2 }       # 블록은 표현식

# 연산자는 일관되게 동작
[1,2,3].@(_*2)      # 배열에 map
"abc".@(_.up)       # 문자열에 map (각 문자)
{a:1,b:2}.@(_*2)    # 맵 값에 map
```

### 3. 암묵적보다 명시적

```vais
# 타입은 명시하거나 추론 가능
add(a, b) = a + b           # OK: 사용처에서 추론
add(a:i, b:i) = a + b       # OK: 명시적 타입

# 부수 효과는 명시적
fn pure_fn(x) = x * 2       # 순수 함수
fn io_fn(path) = !{         # ! = 부수 효과 있음
    read_file(path)
}
```

### 4. 예측 가능한 성능

```vais
# 비용이 명확해야 함
arr.@(_*2)           # O(n) - 선형
arr.sort             # O(n log n) - 명확함
arr.?(_>0).@(_*2)    # O(n) - 단일 패스로 최적화 가능

# 숨겨진 비용 없음
# (Python처럼 리스트의 `in`은 O(n), 셋은 O(1)인 경우와 달리)
```

---

## 타입 시스템

### 원시 타입

| 타입 | 전체 이름 | 설명 | 예시 |
|------|----------|------|------|
| `i` | int | 64비트 부호 있는 정수 | `42`, `-1` |
| `i32` | int32 | 32비트 부호 있는 정수 | `42i32` |
| `i64` | int64 | 64비트 부호 있는 정수 | `42i64` |
| `f` | float | 64비트 부동소수점 | `3.14` |
| `f32` | float32 | 32비트 부동소수점 | `3.14f32` |
| `s` | string | UTF-8 문자열 | `"hello"` |
| `b` | bool | 불리언 | `true`, `false` |
| `void` | void | 값 없음 | - |

### 복합 타입

| 타입 | 설명 | 예시 |
|------|------|------|
| `[T]` | T의 배열 | `[1, 2, 3]` |
| `{K:V}` | K에서 V로의 맵 | `{"a": 1, "b": 2}` |
| `(T1, T2, ...)` | 튜플 | `(1, "a", true)` |
| `?T` | 옵셔널 T | `some(42)`, `nil` |
| `!T` | 결과 (T 또는 에러) | `ok(42)`, `err("failed")` |

### 사용자 정의 타입

```vais
# 구조체
type User = {
    name: s,
    age: i,
    email: ?s
}

# 열거형
type Status =
    | Pending
    | Active(since: i)
    | Inactive(reason: s)

# 타입 별칭
type UserId = i
type UserMap = {s: User}
```

### 제네릭

```vais
# 제네릭 함수
fn first<T>(arr: [T]) -> ?T = arr.nth(0)

# 제네릭 타입
type Result<T, E> =
    | Ok(value: T)
    | Err(error: E)

# 제약 조건
fn sum<T: Numeric>(arr: [T]) -> T = arr./+
```

### 트레이트

```vais
# 트레이트 정의
trait Eq {
    fn eq(self, other: Self) -> b
    fn neq(self, other: Self) -> b = !self.eq(other)
}

trait Ord: Eq {
    fn cmp(self, other: Self) -> i  # -1, 0, 1
    fn lt(self, other: Self) -> b = self.cmp(other) < 0
    fn gt(self, other: Self) -> b = self.cmp(other) > 0
}

trait Mappable<T> {
    fn map<U>(self, f: T -> U) -> Self<U>
}

# 트레이트 구현
impl Eq for User {
    fn eq(self, other) = self.name == other.name
}
```

---

## 표현식

### 모든 것이 표현식

```vais
# 조건문은 표현식
x = if a > b { a } else { b }

# 블록은 표현식
y = {
    let temp = compute()
    temp * 2
}

# match는 표현식
result = match status {
    Pending => "waiting",
    Active(s) => "active since " + s.str,
    Inactive(r) => "inactive: " + r
}
```

### 연산자 표현식

```vais
# 산술
a + b, a - b, a * b, a / b, a % b

# 비교
a == b, a != b, a < b, a > b, a <= b, a >= b

# 논리
a && b, a || b, !a

# 컬렉션
arr.@expr      # map
arr.?expr      # filter
arr./op        # reduce
a @ arr        # contains (in)
a..b           # range
#arr           # length
```

### 체인 표현식

```vais
# 메서드 체이닝
users
    .?(_.active)
    .@(_.name)
    .sort
    .take(10)

# 파이프 연산자 (대안)
users
    |> filter(_.active)
    |> map(_.name)
    |> sort
    |> take(10)
```

### 람다 표현식

```vais
# 암묵적 파라미터 (_)
arr.@(_*2)              # 각 요소 * 2
arr.?(_.age > 18)       # 나이로 필터

# 명시적 파라미터
arr.@(x => x * 2)
arr.?((u) => u.age > 18)

# 여러 파라미터
zip(a, b).@((x, y) => x + y)
```

---

## 함수

### 함수 정의

```vais
# 기본
add(a, b) = a + b

# 타입 포함
add(a: i, b: i) -> i = a + b

# 여러 줄
process(data) = {
    let cleaned = clean(data)
    let validated = validate(cleaned)
    transform(validated)
}

# 기본값 파라미터
greet(name, greeting = "Hello") = greeting + ", " + name

# 이름 있는 파라미터 (호출 시)
create_user(name: "John", age: 30)
```

### 재귀

```vais
# 이름 있는 재귀
fact(n) = if n < 2 { 1 } else { n * fact(n-1) }

# $로 자기 재귀
fact(n) = n < 2 ? 1 : n * $(n-1)

# 꼬리 재귀 (최적화됨)
fact(n, acc = 1) = n < 2 ? acc : $(n-1, n*acc)
```

### 고차 함수

```vais
# 함수를 파라미터로
apply(f, x) = f(x)
apply(_*2, 5)  # 10

# 함수를 반환값으로
multiplier(n) = (x) => x * n
double = multiplier(2)
double(5)  # 10

# 합성
compose(f, g) = (x) => f(g(x))
```

---

## 제어 흐름

### 조건문

```vais
# If 표현식
result = if cond { then_val } else { else_val }

# 삼항 (축약형)
result = cond ? then_val : else_val

# 체인
category =
    age < 13 ? "child" :
    age < 20 ? "teen" :
    age < 60 ? "adult" :
    "senior"
```

### 패턴 매칭

```vais
# Match 표현식
match value {
    0 => "zero",
    1 => "one",
    n if n < 0 => "negative",
    _ => "other"
}

# 구조 분해
match user {
    {name, age} if age >= 18 => "adult: " + name,
    {name, _} => "minor: " + name
}

# Option 매칭
match maybe_value {
    some(v) => process(v),
    nil => default_value
}
```

### 반복

```vais
# For each (표현식)
for x in arr { process(x) }

# 인덱스 포함
for (i, x) in arr.enum {
    print(i.str + ": " + x.str)
}

# 컬렉션 연산 (권장)
arr.@(process)           # map
arr.?(predicate)         # filter
arr./(acc, x => ...)     # fold
```

---

## 에러 처리

### Option 타입 (?T)

```vais
# 생성
maybe_val: ?i = some(42)
nothing: ?i = nil

# 사용
val = maybe_val ? default    # 기본값으로 unwrap
val = maybe_val!             # unwrap 또는 panic

# 체이닝
result = maybe_val
    .@(_*2)           # option에 map
    .?(_.> 0)         # option 필터
    ? 0               # nil이면 기본값
```

### Result 타입 (!T)

```vais
# 생성
success: !i = ok(42)
failure: !i = err("something went wrong")

# ?로 사용
fn read_data(path) -> !s = {
    content = read_file(path)?    # 에러 전파
    parsed = parse(content)?      # 에러 전파
    ok(parsed)
}

# 패턴 매칭
match result {
    ok(v) => process(v),
    err(e) => handle_error(e)
}
```

### 에러 전파

```vais
# ? 연산자
fn process() -> !Result = {
    a = step1()?        # 에러면 조기 반환
    b = step2(a)?       # 에러면 조기 반환
    ok(finalize(b))
}

# try 블록
fn safe_process() -> !Result = try {
    a = step1()
    b = step2(a)
    finalize(b)
}
```

---

## 모듈 시스템

### 모듈 정의

```vais
# file: math.vais
mod math

# Public 기본: pub
pub pi = 3.14159265359

pub sin(x: f) -> f = ...
pub cos(x: f) -> f = ...

# Private (내보내지 않음)
priv helper() = ...
```

### 임포트

```vais
# 모듈 임포트
use math

# 특정 항목 임포트
use math.{sin, cos, pi}

# 별칭으로 임포트
use math.sin as sine

# 전체 임포트 (비권장)
use math.*
```

### 가시성

```vais
pub     # public (누구나 사용 가능)
priv    # private (같은 모듈만)
pub(pkg)  # 패키지-private (같은 패키지)
```

---

## 메모리 모델

### 소유권 (단순화)

```vais
# 값은 기본적으로 복사 (원시 타입)
a = 5
b = a    # 복사
a = 10   # b는 여전히 5

# 컬렉션은 참조 카운팅
arr1 = [1, 2, 3]
arr2 = arr1    # 공유 참조
arr1.push(4)   # 둘 다 [1,2,3,4] 봄

# 필요할 때 명시적 clone
arr2 = arr1.clone()  # 깊은 복사
```

### 가변성

```vais
# 기본적으로 불변
x = 5
x = 10    # OK: 재바인딩

# 가변 바인딩
mut counter = 0
counter += 1    # OK: 변경

# 가변 파라미터
fn increment(mut x: i) = {
    x += 1
    x
}
```

---

## 동시성 원시 타입

### Spawn

```vais
use std.async.spawn

# 태스크 생성
handle = spawn {
    expensive_computation()
}

# 결과 대기
result = handle.await
```

### 채널

```vais
use std.async.{channel, send, recv}

# 채널 생성
(tx, rx) = channel()

# 전송
spawn { tx.send(42) }

# 수신
value = rx.recv()
```

### Async/Await

```vais
# 비동기 함수
async fn fetch_data(url: s) -> !s = {
    response = http.get(url).await?
    ok(response.body)
}

# Await
data = fetch_data("https://api.example.com").await?
```

---

## 향후를 위한 예약어

다음 기능들은 코어에 예약되어 있지만 초기 버전에서는 구현되지 않습니다:

```
# 매크로 (향후)
macro define! { ... }

# 컴파일 타임 계산 (향후)
const fn compile_time() = ...

# 이펙트 시스템 (향후)
fn pure_fn() -> T pure = ...
fn io_fn() -> T io = ...

# 선형 타입 (향후)
fn consume(owned x: Resource) = ...
```

---

## 설계 결정 로그

### 왜 람다 파라미터에 `_`를 사용하나?

```
고려한 옵션:
1. x => x * 2       (장황함)
2. \x -> x * 2      (Haskell 스타일, 낯설음)
3. |x| x * 2        (Rust 스타일)
4. _ * 2            (선택: 간결, Scala에서 익숙함)

결정: _는 더 짧고 함수형 프로그래밍에서 자주 사용됨.
```

### 왜 `if-else` 대신 `?:`를 사용하나?

```
고려한 옵션:
1. if cond then a else b    (장황함)
2. if(cond, a, b)           (함수 스타일)
3. cond ? a : b             (선택: C 계열에서 익숙함)

결정: 삼항 연산자가 단순한 경우 더 간결함.
     복잡한 경우 if-else 블록 사용 가능.
```

### 왜 map/filter/reduce에 `.@` `.?` `./`를 사용하나?

```
고려한 옵션:
1. map(arr, fn)             (함수: 6 토큰)
2. arr.map(fn)              (메서드: 5 토큰)
3. arr >> map(fn)           (파이프: 5 토큰)
4. arr.@(fn)                (선택: 4 토큰)

결정: 연산자 형태가 가장 간결하면서도 읽기 쉬움.
     점 접두사가 컬렉션 연산을 시각적으로 그룹화함.
```

### 왜 클래스가 없나?

```
이유:
- 클래스는 데이터와 동작을 복잡하게 혼합함
- 트레이트 + 구조체가 동일한 기능을 더 명확한 분리로 제공
- 함수형 스타일이 AI 생성에 더 자연스러움
- 더 단순한 멘탈 모델
```

---

## 요약

Vais 코어 설계 원칙:

| 원칙 | 설명 |
|------|------|
| 최소 | 코어에는 필수 기능만 |
| 직교 | 기능들이 독립적으로 조합됨 |
| 명시적 | 암묵적 동작 최소화 |
| 표현식 | 모든 것이 값을 반환 |
| 타입 안전 | 컴파일 타임 타입 검사 |
| 확장 가능 | 코어 변경 없이 확장 가능 |

**코어가 안정되면, 생태계가 성장합니다.**
