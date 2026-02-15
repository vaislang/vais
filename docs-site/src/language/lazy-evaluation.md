# 지연 평가 (Lazy Evaluation)

Vais는 `lazy`와 `force` 키워드를 통해 지연 평가(lazy evaluation)를 지원합니다. 이를 통해 필요할 때까지 계산을 미루고, 결과를 캐싱하여 성능을 최적화할 수 있습니다.

## 기본 개념

### lazy - 평가 지연

`lazy` 키워드는 표현식을 즉시 평가하지 않고 thunk로 감쌉니다:

```vais
F expensive_compute() -> i64 {
    # 무거운 연산 시뮬레이션
    42
}

F main() -> i64 {
    # 이 시점에서는 expensive_compute가 실행되지 않음
    value := lazy expensive_compute()

    # ... 다른 작업들 ...

    # 필요할 때 평가
    result := force value    # 이제 expensive_compute() 실행
    result
}
```

### force - 평가 강제

`force` 키워드는 lazy 값을 평가하고 결과를 반환합니다:

```vais
F main() -> i64 {
    val := lazy 100 + 200

    result := force val    # 300
    result
}
```

## 내부 구조

Lazy 값은 내부적으로 다음과 같은 구조체로 표현됩니다:

```vais
S Lazy<T> {
    computed: bool,        # 이미 평가되었는가?
    value: T,              # 캐시된 결과 값
    thunk: F() -> T        # 평가 함수
}
```

- **computed**: `false`로 시작, 첫 `force` 후 `true`로 변경
- **value**: 평가된 결과를 저장 (캐싱)
- **thunk**: 평가할 표현식을 담은 클로저

## 캐싱 동작

한번 평가된 lazy 값은 결과가 캐싱되어, 이후 `force` 호출에서 재계산하지 않습니다:

```vais
F side_effect_function() -> i64 {
    # 부작용이 있는 함수 (예: I/O, 로깅 등)
    println("계산 중...")
    42
}

F main() -> i64 {
    val := lazy side_effect_function()

    result1 := force val    # "계산 중..." 출력, 42 반환
    result2 := force val    # 캐시된 42 반환 (출력 없음)
    result3 := force val    # 캐시된 42 반환 (출력 없음)

    result1
}
```

## 실전 예제

### 조건부 평가

조건에 따라 무거운 계산을 건너뛸 수 있습니다:

```vais
F heavy_computation(n: i64) -> i64 {
    # 복잡한 계산
    n * n * n
}

F main() -> i64 {
    condition := false

    # lazy로 감싸면 condition이 false일 때 계산 안 함
    expensive := lazy heavy_computation(1000000)

    result := I condition {
        force expensive
    } E {
        0    # 계산 건너뜀
    }

    result
}
```

### 무한 시퀀스

재귀적 lazy 값으로 무한 시퀀스를 표현할 수 있습니다:

```vais
# 개념적 예제 (실제 구현은 더 복잡할 수 있음)
F fibonacci_lazy(a: i64, b: i64) {
    lazy {
        next := a + b
        next
    }
}

F main() -> i64 {
    fib1 := lazy 1
    fib2 := lazy 1
    fib3 := fibonacci_lazy(force fib1, force fib2)

    force fib3    # 2
}
```

### 데이터 스트림 처리

대용량 데이터를 처리할 때 필요한 부분만 평가:

```vais
F load_large_dataset() -> i64 {
    # 큰 데이터셋 로드 (메모리/시간 소모)
    1000000
}

F main() -> i64 {
    dataset := lazy load_large_dataset()

    # 실제로 데이터가 필요할 때만 로드
    use_data := true

    I use_data {
        data := force dataset
        data / 1000    # 처리
    } E {
        0    # 로드하지 않음
    }
}
```

### Memoization 패턴

복잡한 계산 결과를 캐싱하여 성능 향상:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 {
        1
    } E {
        n * factorial(n - 1)
    }
}

F main() -> i64 {
    # 복잡한 계산을 lazy로 감싸기
    fac10 := lazy factorial(10)

    # 여러 번 사용해도 한 번만 계산
    sum := force fac10 + force fac10    # factorial(10)은 1번만 실행

    sum
}
```

## 성능 최적화

### 언제 사용하나?

1. **무거운 계산**: CPU 집약적 작업을 지연
2. **조건부 로직**: 분기에 따라 실행 여부가 결정되는 코드
3. **I/O 작업**: 파일 읽기, 네트워크 요청 등을 필요시에만 수행
4. **메모리 절약**: 큰 데이터 구조를 필요할 때만 생성

### 오버헤드

- Thunk 생성 비용이 있으므로, 가벼운 계산에는 비효율적
- 작은 산술 연산(예: `2 + 3`)은 lazy로 감싸지 마세요

```vais
# 나쁜 예 - 오버헤드가 이득보다 큼
bad := lazy 2 + 3

# 좋은 예 - 무거운 계산
good := lazy expensive_algorithm(large_input)
```

## 주의사항

### 1. 부작용(Side Effects)

부작용이 있는 함수를 lazy로 감싸면 실행 시점이 예측하기 어려워집니다:

```vais
# 주의: 로그 순서가 예상과 다를 수 있음
F log_and_compute() -> i64 {
    println("로그 메시지")    # 언제 실행될까?
    42
}

F main() -> i64 {
    val := lazy log_and_compute()

    # 어느 시점에 로그가 출력될지 불분명
    # ...

    force val
}
```

### 2. 첫 force에서만 부작용 실행

캐싱으로 인해 부작용은 첫 `force`에서만 발생합니다:

```vais
F increment_counter() -> i64 {
    # 전역 카운터 증가 (부작용)
    counter := counter + 1
    counter
}

F main() -> i64 {
    val := lazy increment_counter()

    force val    # 카운터 증가
    force val    # 카�시된 값 사용, 카운터 증가 안 함
    force val    # 카�시된 값 사용, 카운터 증가 안 함

    # 카운터는 1만 증가함
    0
}
```

### 3. 타입 일관성

Lazy 값의 타입은 내부 표현식의 타입과 일치해야 합니다:

```vais
val := lazy 42           # Lazy<i64>
result := force val      # i64

# 타입 불일치 시 컴파일 에러
# bad := force 42       # 42는 lazy 값이 아님
```

## 코드젠 세부사항

Vais 컴파일러는 lazy/force를 다음과 같이 처리합니다:

1. **lazy expr**:
   - `{ i1, T, fn() -> T }` 구조체 생성
   - `computed = false`, `thunk = || expr` 초기화

2. **force lazy_val**:
   - `if !lazy_val.computed { lazy_val.value = lazy_val.thunk(); lazy_val.computed = true }`
   - `return lazy_val.value`

LLVM IR 수준에서는 분기와 함수 포인터 호출로 변환됩니다.

## 요약

- **lazy expr**: 표현식 평가를 지연하고 thunk로 감쌈
- **force expr**: Lazy 값을 평가하고 결과를 캐싱
- **캐싱**: 한 번 평가된 값은 재계산하지 않음
- **사용 사례**: 무거운 계산, 조건부 실행, I/O 최적화, 메모리 절약
- **주의**: 부작용은 첫 force에서만 발생, 가벼운 계산에는 오버헤드

지연 평가는 Vais의 강력한 성능 최적화 도구이며, 함수형 프로그래밍 패턴과 결합하여 효율적이고 우아한 코드를 작성할 수 있게 합니다.
