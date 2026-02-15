# 클로저 & 람다

Vais는 일급 함수(first-class functions)를 지원하며, 간결한 람다 표현식과 강력한 클로저 기능을 제공합니다.

## 기본 문법

### 람다 표현식

람다는 `|params| expr` 형태로 작성합니다:

```vais
# 단일 표현식
double := |x| x * 2
result := double(5)    # 10

# 여러 파라미터
add := |x, y| x + y
sum := add(3, 7)       # 10

# 블록 형태
compute := |x, y| {
    temp := x * 2
    temp + y
}
```

### 타입 추론

대부분의 경우 타입을 명시할 필요가 없습니다:

```vais
# 자동 타입 추론
numbers := vec![1, 2, 3, 4, 5]
doubled := numbers.map(|x| x * 2)

# 명시적 타입 지정 (선택사항)
parse := |s: str| -> i64 {
    # 문자열을 정수로 파싱
    42
}
```

## 캡처 모드

Vais는 4가지 캡처 모드를 제공합니다:

### 1. ByValue (기본)

값을 복사하여 캡처합니다. 원본 변수는 영향받지 않습니다:

```vais
F main() -> i64 {
    x := 10

    # x를 값으로 캡처 (복사)
    closure := |y| x + y

    result := closure(5)   # 15
    # x는 여전히 10

    result
}
```

### 2. Move 캡처

`move` 키워드를 사용하여 소유권을 클로저로 이전합니다:

```vais
F process_data() -> i64 {
    data := 100

    # data의 소유권을 클로저로 이전
    consumer := move |multiplier| data * multiplier

    result := consumer(2)   # 200
    # 이후 data 사용 불가 (소유권 이전됨)

    result
}
```

**주의**: Move 캡처 후 원본 변수를 사용하면 컴파일 에러가 발생합니다.

### 3. ByRef 캡처

불변 참조로 캡처합니다. 캡처된 변수를 읽을 수 있지만 수정할 수 없습니다:

```vais
F main() -> i64 {
    counter := 42

    # 불변 참조로 캡처
    reader := |&counter| counter + 1

    value := reader()      # 43
    # counter는 여전히 42 (수정 불가)

    counter
}
```

**제약사항**: 현재 ByRef는 완전히 구현되지 않았습니다. Lambda ABI 재설계가 필요하며, 사용 시 `Unsupported` 에러가 발생할 수 있습니다.

### 4. ByMutRef 캡처

가변 참조로 캡처합니다. 클로저 내부에서 캡처된 변수를 수정할 수 있습니다:

```vais
F main() -> i64 {
    counter := mut 0

    # 가변 참조로 캡처
    increment := |&mut counter| {
        counter = counter + 1
        counter
    }

    increment()  # counter = 1
    increment()  # counter = 2

    counter      # 2
}
```

**제약사항**: 현재 ByMutRef는 완전히 구현되지 않았습니다. Lambda ABI 재설계가 필요하며, 사용 시 `Unsupported` 에러가 발생할 수 있습니다.

## 고차 함수

클로저를 함수 파라미터로 전달할 수 있습니다:

```vais
# 고차 함수: 함수를 파라미터로 받음
F apply_twice(f, x: i64) -> i64 {
    temp := f(x)
    f(temp)
}

F main() -> i64 {
    double := |n| n * 2

    result := apply_twice(double, 3)  # ((3 * 2) * 2) = 12
    result
}
```

### 컬렉션과 함께 사용

```vais
F main() -> i64 {
    numbers := vec![1, 2, 3, 4, 5]

    # map: 각 요소에 함수 적용
    doubled := numbers.map(|x| x * 2)
    # [2, 4, 6, 8, 10]

    # filter: 조건을 만족하는 요소만 선택
    evens := numbers.filter(|x| x % 2 == 0)
    # [2, 4]

    # reduce: 누적 연산
    sum := numbers.fold(0, |acc, x| acc + x)
    # 15

    sum
}
```

## 클로저 체이닝

여러 고차 함수를 연쇄적으로 사용할 수 있습니다:

```vais
F main() -> i64 {
    result := vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        .filter(|x| x % 2 == 0)           # 짝수만
        .map(|x| x * x)                    # 제곱
        .fold(0, |acc, x| acc + x)         # 합계

    # (2² + 4² + 6² + 8² + 10²) = 220
    result
}
```

## 실전 예제

### 커링(Currying)

```vais
F make_adder(n: i64) {
    |x| n + x
}

F main() -> i64 {
    add_10 := make_adder(10)

    result1 := add_10(5)   # 15
    result2 := add_10(20)  # 30

    result1 + result2      # 45
}
```

### 콜백 패턴

```vais
F process_async(data: i64, callback) -> i64 {
    # 비동기 처리 시뮬레이션
    processed := data * 2
    callback(processed)
}

F main() -> i64 {
    result := process_async(21, |x| {
        # 결과 처리
        x + 10
    })

    result  # 52
}
```

### 지연 실행

```vais
F defer_execution(action) -> i64 {
    # 일부 초기화 작업
    init := 100

    # 나중에 action 실행
    action(init)
}

F main() -> i64 {
    defer_execution(|value| {
        # 지연된 작업
        value * 3
    })  # 300
}
```

## 성능 고려사항

- **인라이닝**: 작은 람다는 컴파일러가 자동으로 인라인 최적화합니다.
- **클로저 크기**: 캡처하는 변수가 많을수록 클로저 객체가 커집니다.
- **Move vs Copy**: 큰 데이터 구조는 move 캡처를 사용하여 불필요한 복사를 피하세요.

## 제약사항

1. **ByRef/ByMutRef 미완성**: 현재 참조 캡처는 Lambda ABI 재설계가 필요하며, 사용 시 `CodegenError::Unsupported` 에러가 발생합니다.

2. **재귀 람다**: 람다는 자기 자신을 직접 호출할 수 없습니다. 재귀가 필요한 경우 일반 함수를 사용하세요.

3. **타입 추론 한계**: 복잡한 경우 타입을 명시해야 할 수 있습니다.

## 요약

- **기본 문법**: `|params| expr` 또는 `|params| { body }`
- **4가지 캡처 모드**: ByValue(기본), Move, ByRef, ByMutRef
- **타입 추론**: 대부분 자동, 필요시 명시 가능
- **고차 함수**: 함수를 파라미터로 전달하거나 반환 가능
- **실용적 패턴**: 커링, 콜백, 지연 실행 등

클로저는 Vais의 함수형 프로그래밍 기능의 핵심이며, 간결하고 표현력 있는 코드를 작성하는 데 필수적인 도구입니다.
