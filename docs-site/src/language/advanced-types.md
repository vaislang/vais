# 고급 타입 시스템

Vais는 현대적인 타입 시스템을 제공하며, 고급 기능들을 통해 코드 재사용성과 추상화 수준을 높일 수 있습니다.

## 목차

1. [Trait Alias (트레이트 별칭)](#trait-alias-트레이트-별칭)
2. [Existential Types (impl Trait)](#existential-types-impl-trait)
3. [Const Evaluation (상수 평가)](#const-evaluation-상수-평가)

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

## Existential Types (impl Trait)

Existential type은 구체적인 타입을 숨기고 트레이트 인터페이스만 노출하는 기능입니다. Vais에서는 `X` 키워드를 타입 위치에 사용합니다.

### 기본 문법

```vais
# 함수 리턴 타입으로 impl Trait 사용
F make_thing() -> X SomeTrait {
    ConcreteType { ... }
}
```

**핵심**: `X` 키워드는 impl 블록 선언과 existential type 모두에 사용됩니다.

### 기본 예제

```vais
W Numeric {
    F value(self) -> i64
}

S MyNum { n: i64 }
X MyNum: Numeric {
    F value(self) -> i64 { self.n }
}

# MyNum 대신 X Numeric으로 리턴
F make_num() -> X Numeric {
    MyNum { n: 42 }
}

F main() -> i64 {
    # num의 구체 타입은 숨겨지고, Numeric 메서드만 사용 가능
    num := make_num()
    R num.value()  # 42
}
```

### 다중 바운드

여러 트레이트를 동시에 구현하는 타입을 반환할 수 있습니다:

```vais
W TraitA {
    F method_a(self) -> i64
}

W TraitB {
    F method_b(self) -> i64
}

S Impl { val: i64 }
X Impl: TraitA {
    F method_a(self) -> i64 { self.val }
}
X Impl: TraitB {
    F method_b(self) -> i64 { self.val * 2 }
}

# TraitA와 TraitB 모두 구현하는 타입 반환
F make_thing() -> X TraitA + TraitB {
    Impl { val: 10 }
}

F main() -> i64 {
    thing := make_thing()
    a := thing.method_a()  # 10
    b := thing.method_b()  # 20
    R a + b  # 30
}
```

### 제네릭 함수와 함께 사용

Existential type은 제네릭 함수의 리턴 타입으로도 사용 가능합니다:

```vais
W Display {
    F show(self) -> i64
}

S MyType { n: i64 }
X MyType: Display {
    F show(self) -> i64 {
        print_i64(self.n)
        0
    }
}

# 제네릭 함수가 impl Trait 반환
F create<T: Display>(val: i64) -> X Display {
    MyType { n: val }
}

F main() -> i64 {
    obj := create(42)
    obj.show()  # 42
    0
}
```

### Where 절과 함께 사용

```vais
W Trait {
    F method(self) -> i64
}

S Thing { x: i64 }
X Thing: Trait {
    F method(self) -> i64 { self.x }
}

F produce<T>() -> X Trait
where T: Trait
{
    Thing { x: 99 }
}

F main() -> i64 {
    result := produce()
    R result.method()  # 99
}
```

### 사용 사례

Existential type은 다음과 같은 경우에 유용합니다:

1. **구현 세부사항 숨기기**: 내부 타입을 노출하지 않고 인터페이스만 제공
2. **유연한 구현 변경**: 인터페이스 유지하면서 구현 타입 교체 가능
3. **복잡한 타입 시그니처 단순화**: 긴 제네릭 타입을 트레이트로 추상화

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

## 요약

### Trait Alias
- **문법**: `T Name = TraitA + TraitB`
- **사용처**: 제네릭 바운드, where 절
- **장점**: 복잡한 트레이트 바운드 단순화, 코드 재사용성 향상

### Existential Types
- **문법**: `F func() -> X Trait`
- **사용처**: 함수 리턴 타입
- **장점**: 구현 세부사항 숨기기, 유연한 인터페이스 제공

### Const Evaluation
- **지원 연산**: 산술, 비트, 모듈로, 시프트
- **사용처**: 배열 크기, 컴파일 타임 상수
- **장점**: 컴파일 타임 최적화, 타입 안전성

---

## 다음 단계

- **제네릭 튜토리얼**: [generics.md](./generics.md)
- **트레이트 시스템**: 기본 트레이트 정의와 구현
- **타입 추론**: [type-inference.md](./type-inference.md)
- **실전 예제**: `examples/` 디렉토리

---

이러한 고급 타입 기능들을 활용하여 더 안전하고 재사용 가능한 Vais 코드를 작성할 수 있습니다!
