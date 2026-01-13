# Vais 문법 가이드

이 문서에서는 Vais 언어의 완전한 문법을 설명합니다.

## 목차

- [주석](#주석)
- [데이터 타입](#데이터-타입)
- [변수](#변수)
- [연산자](#연산자)
- [함수](#함수)
- [제어 흐름](#제어-흐름)
- [컬렉션](#컬렉션)
- [컬렉션 연산](#컬렉션-연산)
- [모듈](#모듈)
- [FFI](#ffi)
- [모범 사례](#모범-사례)

---

## 주석

```vais
// 한 줄 주석

/*
   여러 줄
   주석
*/
```

---

## 데이터 타입

### 기본 타입(Primitive Types)

| 타입 | 예제 | 설명 |
|------|------|------|
| `Int` | `42`, `-17`, `0` | 64비트 부호 있는 정수 |
| `Float` | `3.14`, `-0.5`, `1.0e10` | 64비트 부동소수점 |
| `Bool` | `true`, `false` | 불리언(Boolean) |
| `String` | `"hello"`, `'world'` | UTF-8 문자열 |
| `Nil` | `nil` | 널(Null)/None 값 |

### 컬렉션 타입(Collection Types)

| 타입 | 예제 | 설명 |
|------|------|------|
| `Array` | `[1, 2, 3]` | 순서가 있는 리스트 |
| `Map` | `{a: 1, b: 2}` | 키-값 쌍 |
| `Range` | `1..10` | 정수 범위 (끝 값 미포함) |

### 타입 어노테이션(Type Annotations) - 선택사항

```vais
// 타입 어노테이션이 있는 함수
add(a: Int, b: Int): Int = a + b

// 매개변수 타입
greet(name: String) = "Hello, " ++ name
```

---

## 변수

### 선언

```vais
// 기본적으로 불변(immutable)
x = 10
name = "Vais"
numbers = [1, 2, 3]
```

### 구조 분해(Destructuring)

```vais
// 배열 구조 분해
[first, second, ...rest] = [1, 2, 3, 4, 5]
// first = 1, second = 2, rest = [3, 4, 5]

// 구조체 구조 분해
{name, age} = {name: "Alice", age: 30}
```

---

## 연산자

### 산술 연산자(Arithmetic Operators)

| 연산자 | 설명 | 예제 |
|--------|------|------|
| `+` | 덧셈 | `5 + 3` -> `8` |
| `-` | 뺄셈 | `5 - 3` -> `2` |
| `*` | 곱셈 | `5 * 3` -> `15` |
| `/` | 나눗셈 | `10 / 3` -> `3` |
| `%` | 나머지 | `10 % 3` -> `1` |
| `-` | 부호 반전 (단항) | `-5` -> `-5` |

### 비교 연산자(Comparison Operators)

| 연산자 | 설명 | 예제 |
|--------|------|------|
| `==` | 같음 | `5 == 5` -> `true` |
| `!=` | 다름 | `5 != 3` -> `true` |
| `<` | 작음 | `3 < 5` -> `true` |
| `>` | 큼 | `5 > 3` -> `true` |
| `<=` | 작거나 같음 | `5 <= 5` -> `true` |
| `>=` | 크거나 같음 | `5 >= 3` -> `true` |

### 논리 연산자(Logical Operators)

| 연산자 | 설명 | 예제 |
|--------|------|------|
| `&&` | 논리 AND | `true && false` -> `false` |
| `\|\|` | 논리 OR | `true \|\| false` -> `true` |
| `!` | 논리 NOT | `!true` -> `false` |

### 문자열 및 컬렉션 연산자

| 연산자 | 설명 | 예제 |
|--------|------|------|
| `++` | 연결(Concatenation) | `"a" ++ "b"` -> `"ab"` |
| `#` | 길이 | `#[1,2,3]` -> `3` |

---

## 함수

### 기본 정의

```vais
// 단일 표현식 (중괄호 불필요)
add(a, b) = a + b

// 호출
result = add(10, 20)  // 30
```

### 자기 재귀(Self-Recursion)와 `$`

`$` 연산자는 현재 함수를 재귀적으로 호출합니다:

```vais
// 팩토리얼(Factorial)
factorial(n) = n < 2 ? 1 : n * $(n - 1)

factorial(5)  // 120

// 피보나치(Fibonacci)
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)

fib(10)  // 55
```

### 람다 표현식(Lambda Expression)

```vais
// 명시적 매개변수가 있는 람다
double = (x) => x * 2

// 람다 플레이스홀더(_)
triple = _ * 3

// 컬렉션 연산에서 사용
[1, 2, 3].@(_ * 2)         // [2, 4, 6]
[1, 2, 3].@((x) => x * 2)  // 동일한 결과
```

### 고차 함수(Higher-Order Functions)

```vais
// 함수를 반환하는 함수
make_adder(n) = (x) => x + n

add5 = make_adder(5)
add5(10)  // 15

// 함수를 매개변수로 받는 함수
apply(f, x) = f(x)
apply(_ * 2, 10)  // 20
```

### 공개 함수(Public Functions)

```vais
// 내보내기 가능한 공개 함수
pub add(a, b) = a + b

// 비공개 함수 (기본값)
helper(x) = x * 2
```

---

## 제어 흐름

### 삼항 표현식(Ternary Expression)

```vais
// condition ? then : else
max(a, b) = a > b ? a : b

// 중첩 삼항
sign(n) = n > 0 ? "positive" : n < 0 ? "negative" : "zero"
```

### if 표현식

```vais
// if-then-else 표현식
result = if x > 0 then "positive" else "non-positive"

// 여러 줄 (복잡한 로직용)
classify(n) = if n > 0 then
    "positive"
else if n < 0 then
    "negative"
else
    "zero"
```

---

## 컬렉션

### 배열(Arrays)

```vais
// 생성
empty = []
numbers = [1, 2, 3, 4, 5]
mixed = [1, "two", true, [3, 4]]

// 접근
first = numbers[0]      // 1
last = numbers[-1]      // 5 (음수 인덱스)

// 범위로 생성
one_to_ten = [1..11]    // [1, 2, 3, ..., 10]
```

### 맵/구조체(Maps/Structs)

```vais
// 생성
person = {name: "Alice", age: 30}

// 접근
person.name      // "Alice"
person["name"]   // "Alice"

// 중첩
data = {
    user: {name: "Bob", id: 123},
    items: [1, 2, 3]
}
data.user.name   // "Bob"
```

### 범위(Ranges)

```vais
// 배타적 범위 (끝 값 미포함)
1..5     // [1, 2, 3, 4]

// 컬렉션 연산과 함께 사용
[1..6].@(_ * 2)   // [2, 4, 6, 8, 10]
```

---

## 컬렉션 연산

Vais는 일반적인 함수형 연산을 위한 간결한 연산자를 제공합니다.

### Map `.@`

각 요소를 변환:

```vais
[1, 2, 3].@(_ * 2)           // [2, 4, 6]
[1, 2, 3].@((x) => x + 1)    // [2, 3, 4]

// 중첩 접근
users.@(_.name)              // 이름 추출
```

### Filter `.?`

조건에 맞는 요소 유지:

```vais
[1, 2, 3, 4, 5].?(_ > 2)         // [3, 4, 5]
[1, 2, 3, 4, 5].?(_ % 2 == 0)    // [2, 4]

// 객체 필터링
users.?(_.age >= 18)             // 성인 사용자
```

### Reduce `./`

요소들을 단일 값으로 접기(Fold):

```vais
// 합계: ./+
[1, 2, 3, 4, 5]./+(0, _ + _)     // 15

// 곱: ./*
[1, 2, 3, 4, 5]./*(1, _ * _)     // 120

// 커스텀 리듀스
[1, 2, 3]./(10, _ + _)           // 10 + 1 + 2 + 3 = 16
```

### 연산 체이닝(Chaining Operations)

```vais
// 필터 후 맵
[1..10].?(_ % 2 == 0).@(_ * _)   // [4, 16, 36, 64]

// 복잡한 파이프라인
data
    .?(_.active)          // 활성 항목 필터
    .@(_.value)           // 값 추출
    ./+(0, _ + _)         // 합계
```

---

## 모듈

### 가져오기(Importing)

```vais
// 특정 함수 가져오기
use math.{sin, cos, pi}

// 별칭으로 가져오기
use json.parse as json_parse

// 모두 가져오기 (권장하지 않음)
use utils.*
```

### 내보내기(Exporting)

```vais
// 공개 함수 (내보내기)
pub add(a, b) = a + b

pub calculate(x, y) = {
    sum: add(x, y),
    diff: x - y
}

// 비공개 헬퍼 (내보내지 않음)
helper(x) = x * 2
```

### 모듈 파일 구조

```
my-project/
├── vais.toml
├── src/
│   ├── main.vais
│   └── utils.vais
└── lib/
    └── math.vais
```

---

## FFI

### FFI 함수 선언

```vais
ffi "c" {
    fn abs(n: i32) -> i32
    fn sqrt(x: f64) -> f64
    fn pow(base: f64, exp: f64) -> f64
    fn strlen(s: cstr) -> i64
    fn getenv(key: cstr) -> cstr
}
```

### FFI 타입

| Vais 타입 | C 타입 | 설명 |
|-----------|--------|------|
| `i8`, `i16`, `i32`, `i64` | `int8_t` 등 | 부호 있는 정수 |
| `u8`, `u16`, `u32`, `u64` | `uint8_t` 등 | 부호 없는 정수 |
| `f32` | `float` | 32비트 부동소수점 |
| `f64` | `double` | 64비트 부동소수점 |
| `bool` | `bool` | 불리언 |
| `cstr` | `char*` | C 문자열 |
| `ptr` | `void*` | 일반 포인터 |
| `void` | `void` | 반환 값 없음 |

### FFI 함수 사용

```vais
ffi "c" {
    fn abs(n: i32) -> i32
    fn sqrt(x: f64) -> f64
}

print(abs(-42))      // 42
print(sqrt(16.0))    // 4.0
```

---

## 전체 예제

```vais
// 모듈 가져오기
use std.io.{read_file, write_file}

// 공개 함수들
pub factorial(n) = n < 2 ? 1 : n * $(n - 1)

pub fibonacci(n) = n < 2 ? n : $(n - 1) + $(n - 2)

pub quicksort(arr) =
    #arr <= 1 ? arr :
    let pivot = arr[0] in
    let less = arr[1..].?((_ < pivot)) in
    let greater = arr[1..].?((_ >= pivot)) in
    $(less) ++ [pivot] ++ $(greater)

// 메인 프로그램
numbers = [5, 2, 8, 1, 9, 3]
sorted = quicksort(numbers)

print("원본:", numbers)
print("정렬:", sorted)
print("팩토리얼:", [1..6].@(factorial(_)))
print("피보나치:", [0..10].@(fibonacci(_)))
```

---

## 모범 사례

1. **재귀에 `$` 사용** - 함수 이름보다 간결함
2. **람다에 `_` 사용** - 단순한 변환에 더 깔끔함
3. **연산 체이닝** - `.@`, `.?`, `./`를 조합하여 파이프라인 구성
4. **삼항 표현식 선호** - 단순한 조건문에 적합
5. **타입 어노테이션** - 복잡한 함수에 추가 (선택사항이지만 도움됨)
6. **모듈 구성** - 관련 함수를 함께 유지

---

## 관련 문서

- [시작 가이드](getting-started.md)
- [API 레퍼런스](api.md)
- [예제](examples.md)
