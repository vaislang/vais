# 전역 변수 (G)

`G` 키워드는 정적 저장 기간(static storage duration)을 가지는 전역 변수를 선언합니다.

## 기본 문법

```vais
G name: Type = value
```

전역 변수는 프로그램 전체에서 접근 가능하며, 프로그램 시작부터 종료까지 존재합니다.

## 예제

### 단순 전역 변수

```vais
G counter: i64 = 0
G max_size: i64 = 1024
G pi: f64 = 3.14159265358979

F main() -> i64 {
    puts("max_size:")
    print_i64(max_size)
    0
}
```

### 가변 전역 변수

전역 변수는 기본적으로 가변(mutable)입니다:

```vais
G total: i64 = 0

F add_to_total(n: i64) -> i64 {
    total = total + n
    total
}

F main() -> i64 {
    add_to_total(10)
    add_to_total(20)
    add_to_total(30)
    total   # 60
}
```

### 공개 전역 변수

`P` 키워드와 함께 사용하면 다른 모듈에서 접근 가능합니다:

```vais
P G VERSION: i64 = 1
P G DEBUG_MODE: bool = false
```

## 타입 지정

전역 변수는 반드시 타입을 명시해야 합니다:

```vais
G count: i64 = 0       # i64 타입
G ratio: f64 = 0.5     # f64 타입
G flag: bool = true     # bool 타입
```

## 스코프 규칙

- 전역 변수는 선언 이후 모든 함수에서 접근 가능합니다.
- 같은 이름의 지역 변수가 있으면 지역 변수가 우선합니다 (shadowing).
- 모듈 간 공유는 `P G` (public global)로 선언해야 합니다.

## 주의사항

- 전역 변수의 초기값은 컴파일 타임에 평가 가능한 상수여야 합니다.
- 전역 변수의 과도한 사용은 코드의 추론을 어렵게 만듭니다. 가능하면 함수 파라미터를 통한 명시적 전달을 권장합니다.
- 멀티스레드 환경에서 전역 변수 접근은 동기화가 필요합니다.
