# 고급 타입 시스템

Vais는 현대적인 타입 시스템을 제공하며, 고급 기능들을 통해 코드 재사용성과 추상화 수준을 높일 수 있습니다.

## 목차

1. [Trait Alias (트레이트 별칭)](#trait-alias-트레이트-별칭)
2. [Const Evaluation (상수 평가)](#const-evaluation-상수-평가)
3. [Union 타입 (O)](#union-타입-o)
4. [SIMD 벡터 타입](#simd-벡터-타입)

---

## Trait Alias (트레이트 별칭)

Trait alias는 여러 트레이트 바운드를 하나의 이름으로 묶어서 재사용할 수 있게 합니다.

### 기본 문법

```vais
# 단일 트레이트 별칭
T Display = Printable

# 다중 트레이트 별칭
T StringLike = Display + Clone + Hash
```

**핵심**: `T` 키워드는 타입 별칭과 트레이트 별칭 모두에 사용됩니다.

### 기본 예제

```vais
W Numeric {
    F value(self) -> i64
}

# Trait alias 정의
T Number = Numeric

S MyInt { n: i64 }
X MyInt: Numeric {
    F value(self) -> i64 { self.n }
}

# 별칭을 제네릭 바운드로 사용
F show<T: Number>(x: T) -> i64 {
    x.value()
}

F main() -> i64 {
    num := MyInt { n: 42 }
    R show(num)  # 42
}
```

### 다중 바운드 예제

여러 트레이트를 하나로 묶어서 코드를 간결하게 만들 수 있습니다:

```vais
W Display {
    F show(self) -> i64
}

W Clone {
    F clone(self) -> Self
}

W Hash {
    F hash(self) -> i64
}

# 세 개의 트레이트를 하나로 묶기
T StringLike = Display + Clone + Hash

S MyString { data: str }
X MyString: Display {
    F show(self) -> i64 {
        puts(self.data)
        0
    }
}
X MyString: Clone {
    F clone(self) -> MyString {
        MyString { data: self.data }
    }
}
X MyString: Hash {
    F hash(self) -> i64 {
        42  # 간단한 해시 구현
    }
}

# StringLike 바운드 사용 (Display + Clone + Hash 대신)
F process<T: StringLike>(val: T) -> i64 {
    val.show()
    cloned := val.clone()
    hash := val.hash()
    hash
}

F main() -> i64 {
    s := MyString { data: "hello" }
    R process(s)
}
```

### Where 절과 함께 사용

Trait alias는 where 절에서도 사용 가능합니다:

```vais
W Printable {
    F to_num(self) -> i64
}

T Display = Printable

S Point { x: i64, y: i64 }
X Point: Printable {
    F to_num(self) -> i64 { self.x + self.y }
}

F show<T>(val: T) -> i64
where T: Display
{
    val.to_num()
}

F main() -> i64 {
    p := Point { x: 10, y: 32 }
    R show(p)  # 42
}
```

### 중첩 별칭

Trait alias는 다른 trait alias를 참조할 수 있습니다:

```vais
W Base {
    F base_fn(self) -> i64
}

# 첫 번째 별칭
T Level1 = Base

# 별칭을 참조하는 별칭
T Level2 = Level1

S Thing { n: i64 }
X Thing: Base {
    F base_fn(self) -> i64 { self.n }
}

F use_nested<T: Level2>(x: T) -> i64 {
    x.base_fn()
}

F main() -> i64 {
    t := Thing { n: 100 }
    R use_nested(t)
}
```

---

## Const Evaluation (상수 평가)

Vais는 컴파일 타임에 상수 표현식을 평가할 수 있습니다. 특히 배열 크기 등에서 복잡한 산술 연산을 사용할 수 있습니다.

### 지원되는 연산자

Phase 37에서 확장된 const evaluation은 다음 연산자를 지원합니다:

- **산술 연산**: `+`, `-`, `*`, `/`
- **나머지**: `%`
- **비트 연산**: `&`, `|`, `^`, `<<`, `>>`
- **부호**: `-` (단항 negation)

### 기본 예제

```vais
F main() -> i64 {
    # 배열 크기에 산술 연산
    arr1 := [0; 2 + 3]      # 크기 5
    arr2 := [0; 10 - 2]     # 크기 8
    arr3 := [0; 4 * 3]      # 크기 12
    arr4 := [0; 20 / 4]     # 크기 5

    0
}
```

### 모듈로 연산

```vais
F main() -> i64 {
    # 10 % 3 = 1
    arr := [0; 10 % 3]  # 크기 1

    0
}
```

### 비트 연산

```vais
F main() -> i64 {
    # 비트 AND: 3 & 7 = 3
    arr1 := [0; 3 & 7]   # 크기 3

    # 비트 OR: 2 | 4 = 6
    arr2 := [0; 2 | 4]   # 크기 6

    # 비트 XOR: 5 ^ 3 = 6
    arr3 := [0; 5 ^ 3]   # 크기 6

    0
}
```

### 시프트 연산

```vais
F main() -> i64 {
    # 왼쪽 시프트: 1 << 3 = 8
    arr1 := [0; 1 << 3]   # 크기 8

    # 오른쪽 시프트: 16 >> 2 = 4
    arr2 := [0; 16 >> 2]  # 크기 4

    0
}
```

### 복합 표현식

여러 연산자를 조합할 수 있습니다:

```vais
F main() -> i64 {
    # (2 + 3) * 2 = 10
    arr1 := [0; (2 + 3) * 2]  # 크기 10

    # ((1 << 4) - 2) / 2 = (16 - 2) / 2 = 7
    arr2 := [0; ((1 << 4) - 2) / 2]  # 크기 7

    # 복잡한 비트 연산
    arr3 := [0; (8 | 4) & 15]  # (12) & 15 = 12

    0
}
```

### 연산자 우선순위

Const evaluation은 표준 연산자 우선순위를 따릅니다:

1. `<<`, `>>` (시프트)
2. `&` (비트 AND)
3. `^` (비트 XOR)
4. `|` (비트 OR)
5. `*`, `/`, `%` (곱셈, 나눗셈, 나머지)
6. `+`, `-` (덧셈, 뺄셈)

괄호 `()`를 사용하여 우선순위를 명시할 수 있습니다.

### 사용 사례

Const evaluation은 다음과 같은 경우에 유용합니다:

1. **컴파일 타임 계산**: 배열 크기를 동적 계산 대신 컴파일 타임에 결정
2. **비트마스크**: 비트 연산을 통한 플래그 크기 계산
3. **정렬 계산**: 메모리 정렬 요구사항을 컴파일 타임에 계산

---

## Union 타입 (O)

Union 타입은 C-style 비태그 union입니다. 여러 타입의 값을 같은 메모리 공간에 저장할 수 있으며, `O` 키워드로 정의합니다.

### 기본 문법

```vais
O IntOrFloat {
    i: i64,
    f: f64
}

F main() -> i64 {
    u := IntOrFloat { i: 42 }

    # union 필드 접근 (unsafe — 올바른 필드를 읽어야 함)
    val := u.i    # 42

    0
}
```

### 사용 사례

Union은 다음과 같은 경우에 유용합니다:

1. **FFI 호환성**: C 라이브러리와의 인터페이스에서 C union 매핑
2. **메모리 최적화**: 여러 타입 중 하나만 사용할 때 메모리 절약
3. **비트 레벨 조작**: 같은 메모리를 다른 타입으로 해석

### 주의사항

- Union은 타입 안전하지 않습니다. 잘못된 필드를 읽으면 정의되지 않은 동작이 발생합니다.
- 태그가 없으므로 현재 어떤 필드가 활성인지 추적해야 합니다.
- 대부분의 경우 `E` (enum)을 사용하는 것이 안전합니다.

---

## SIMD 벡터 타입

Vais는 SIMD(Single Instruction, Multiple Data) 벡터 타입을 내장 지원합니다.

### 지원 타입

| 타입 | 설명 | 크기 |
|------|------|------|
| `Vec4f32` | 4×f32 벡터 | 128-bit (SSE) |
| `Vec2f64` | 2×f64 벡터 | 128-bit (SSE2) |
| `Vec8f32` | 8×f32 벡터 | 256-bit (AVX) |
| `Vec4f64` | 4×f64 벡터 | 256-bit (AVX) |

### 기본 사용

```vais
F main() -> i64 {
    # SIMD 벡터 연산은 C 런타임 함수를 통해 수행됩니다
    # SSE2/NEON/스칼라 폴백이 자동 선택됩니다
    0
}
```

### 플랫폼 지원

- **x86_64**: SSE2 기본, AVX/AVX2 가능
- **aarch64**: NEON 기본
- **기타**: 스칼라 폴백 (자동)

---

## 요약

### Trait Alias
- **문법**: `T Name = TraitA + TraitB`
- **사용처**: 제네릭 바운드, where 절
- **장점**: 복잡한 트레이트 바운드 단순화, 코드 재사용성 향상

### Const Evaluation
- **지원 연산**: 산술, 비트, 모듈로, 시프트
- **사용처**: 배열 크기, 컴파일 타임 상수
- **장점**: 컴파일 타임 최적화, 타입 안전성

### Union 타입
- **문법**: `O Name { field: Type, ... }`
- **사용처**: FFI 호환성, 메모리 최적화, 비트 레벨 조작
- **주의**: 타입 안전하지 않음, 태그 없음 (enum 사용 권장)

### SIMD 벡터 타입
- **지원 타입**: `Vec4f32`, `Vec2f64`, `Vec8f32`, `Vec4f64`
- **플랫폼**: SSE2, AVX, NEON, 스칼라 폴백
- **사용처**: 고성능 벡터 연산

> **참고**: `X Trait` existential return types (impl Trait)와 Higher-Kinded Types (`F<_>`)는 ROADMAP #18에서 제거되었습니다. 동등한 기능은 명시적 제네릭(`F foo<T: Trait>() -> T`)으로 표현하세요.

---

## 다음 단계

- **제네릭 튜토리얼**: [generics.md](./generics.md)
- **트레이트 시스템**: 기본 트레이트 정의와 구현
- **타입 추론**: [type-inference.md](./type-inference.md)
- **실전 예제**: `examples/` 디렉토리

---

이러한 고급 타입 기능들을 활용하여 더 안전하고 재사용 가능한 Vais 코드를 작성할 수 있습니다!
