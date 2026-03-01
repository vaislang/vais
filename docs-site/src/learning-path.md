# Learning Path

Vais를 체계적으로 배울 수 있는 3단계 학습 경로입니다. 프로그래밍 경험에 따라 적절한 단계부터 시작하세요.

---

## 사전 준비

시작하기 전에 Vais를 설치하세요:

```bash
# Homebrew (추천)
brew tap vaislang/tap && brew install vais

# 또는 소스에서 빌드
git clone https://github.com/vaislang/vais.git
cd vais && cargo build --release
```

에디터 설정: [VSCode 확장](../tools/editors.md) 또는 [IntelliJ 플러그인](../tools/editors.md)을 설치하면 구문 강조와 자동 완성을 사용할 수 있습니다.

---

## Stage 1: 초급 (2시간)

> **대상**: 프로그래밍 경험이 있지만 Vais는 처음인 개발자

### 1.1 Hello World (15분)

**읽기**: [Getting Started](./guide/getting-started.md) - 설치부터 첫 프로그램까지

```vais
F main() {
    println("Hello, Vais!")
}
```

**실습**: `hello.vais` 파일을 만들고 `vaisc run hello.vais`로 실행

**확인**:
- [ ] `vaisc --version`으로 설치 확인
- [ ] Hello World 컴파일 및 실행 성공
- [ ] REPL (`vaisc repl`) 사용해보기

### 1.2 변수와 타입 (20분)

**읽기**: [Getting Started - 변수 선언](./guide/getting-started.md#변수-선언)

Vais의 핵심 문법을 익히세요:

```vais
# 불변 변수
x := 42
name := "Vais"

# 가변 변수
counter := mut 0
counter = counter + 1

# 타입 명시 (선택)
pi: f64 = 3.14159
```

**실습**: 다양한 타입의 변수를 선언하고 출력하는 프로그램 작성

**참고 예제**: [examples/hello.vais](https://github.com/vaislang/vais/blob/main/examples/hello.vais)

### 1.3 함수 (20분)

**읽기**: [Getting Started - 함수 정의](./guide/getting-started.md#함수-정의)

```vais
# 단일 표현식 함수
F add(a: i64, b: i64) -> i64 = a + b

# 복수 줄 함수
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    R n * factorial(n - 1)
}

# 자재귀 연산자 @
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)
```

**실습**: `@` 연산자를 사용하는 재귀 함수 작성 (피보나치, 팩토리얼)

**참고 예제**: [examples/fib.vais](https://github.com/vaislang/vais/blob/main/examples/fib.vais)

### 1.4 제어 흐름 (20분)

**읽기**: [Getting Started - 조건문/반복문](./guide/getting-started.md#조건문)

```vais
# if/else (I/E)
I x > 0 { println("positive") }
E { println("non-positive") }

# 삼항 연산자
result := x > 0 ? "yes" : "no"

# 범위 루프
L i:0..10 { println("~{i}") }

# 조건 루프
L x < 100 { x = x * 2 }

# 패턴 매칭
M status {
    200 => println("OK"),
    404 => println("Not Found"),
    _ => println("Unknown")
}
```

**실습**: FizzBuzz를 Vais로 작성 (L, I, E, M 사용)

**참고 예제**: [examples/control_flow.vais](https://github.com/vaislang/vais/blob/main/examples/control_flow.vais), [examples/match_test.vais](https://github.com/vaislang/vais/blob/main/examples/match_test.vais)

### 1.5 구조체와 메서드 (25분)

**읽기**: [Getting Started - 구조체](./guide/getting-started.md#구조체)

```vais
S Point {
    x: i64
    y: i64
}

X Point {
    F sum(&self) -> i64 = self.x + self.y
}

F main() {
    p := Point { x: 10, y: 20 }
    println("~{p.sum()}")
}
```

**실습**: `Rectangle` 구조체를 정의하고 `area()`, `perimeter()` 메서드 구현

**참고 예제**: [examples/method_test.vais](https://github.com/vaislang/vais/blob/main/examples/method_test.vais)

### 1.6 Enum과 패턴 매칭 (20분)

**읽기**: [Getting Started - Enum](./guide/getting-started.md#enum-열거형)

```vais
E Shape {
    Circle(i64),
    Rectangle(i64, i64)
}

F area(s: Shape) -> i64 {
    M s {
        Shape.Circle(r) => r * r * 3,
        Shape.Rectangle(w, h) => w * h
    }
}
```

**실습**: 간단한 계산기 Enum (Add, Sub, Mul, Div) + 패턴 매칭 구현

**참고 예제**: [examples/enum_test.vais](https://github.com/vaislang/vais/blob/main/examples/enum_test.vais)

### Stage 1 체크리스트

- [ ] `:=` / `mut` 변수 선언 이해
- [ ] `F` 함수, `@` 자재귀 사용
- [ ] `I`/`E` 조건문, `L` 루프, `M` 매칭 사용
- [ ] `S` 구조체, `X` impl 블록, `E` enum 정의
- [ ] `~{expr}` 문자열 보간 사용
- [ ] 간단한 프로그램을 독립적으로 작성 가능

---

## Stage 2: 중급 (4시간)

> **대상**: Stage 1을 완료했거나 Rust/C 경험이 있는 개발자

### 2.1 제네릭 (30분)

**읽기**: [Generics](./language/generics.md)

```vais
F max<T>(a: T, b: T) -> T {
    I a > b { R a } E { R b }
}

S Container<T> {
    value: T
}

X Container<T> {
    F get(&self) -> T = self.value
}
```

**실습**: 제네릭 `Stack<T>` 구현 (push, pop, peek)

**참고 예제**: [examples/generic_test.vais](https://github.com/vaislang/vais/blob/main/examples/generic_test.vais), [examples/generic_struct_test.vais](https://github.com/vaislang/vais/blob/main/examples/generic_struct_test.vais)

### 2.2 Trait와 다형성 (40분)

**읽기**: [Advanced Types](./language/advanced-types.md)

```vais
W Printable {
    F to_string(&self) -> str
}

S Dog { name: str }

X Dog: Printable {
    F to_string(&self) -> str = self.name
}

# Trait 바운드
F print_item<T: Printable>(item: T) {
    println(item.to_string())
}
```

**실습**: `Serializable` trait 정의 + 2개 이상의 구조체에 구현

**참고 예제**: [examples/trait_test.vais](https://github.com/vaislang/vais/blob/main/examples/trait_test.vais), [examples/trait_advanced_test.vais](https://github.com/vaislang/vais/blob/main/examples/trait_advanced_test.vais)

### 2.3 에러 처리 (30분)

**읽기**: [Error Handling](./guide/error-handling.md)

```vais
# Option
F find(arr: *i64, len: i64, target: i64) -> i64 {
    L i:0..len {
        I arr[i] == target { R i }
    }
    R -1
}

# Result와 ? 연산자
F divide(a: i64, b: i64) -> i64 {
    I b == 0 { R -1 }
    R a / b
}
```

**실습**: 파일 읽기 + 파싱 + 결과 출력 함수 체인 작성

**참고 예제**: [examples/option_result_test.vais](https://github.com/vaislang/vais/blob/main/examples/option_result_test.vais), [examples/result_test.vais](https://github.com/vaislang/vais/blob/main/examples/result_test.vais)

### 2.4 클로저와 파이프 연산자 (30분)

**읽기**: [Closures & Lambda](./language/closures.md)

```vais
# 클로저
double := |x| x * 2

# 파이프 연산자
result := 5 |> double |> |x| x + 1
# result = 11
```

**실습**: 파이프 연산자로 데이터 변환 파이프라인 작성

**참고 예제**: [examples/lambda_test.vais](https://github.com/vaislang/vais/blob/main/examples/lambda_test.vais), [examples/pipe_operator.vais](https://github.com/vaislang/vais/blob/main/examples/pipe_operator.vais)

### 2.5 표준 라이브러리 활용 (40분)

**읽기**: [Standard Library Reference](./stdlib/stdlib.md)

핵심 모듈:

| 모듈 | 용도 | 예제 |
|------|------|------|
| `std/vec.vais` | 동적 배열 | [examples/simple_vec_test.vais](https://github.com/vaislang/vais/blob/main/examples/simple_vec_test.vais) |
| `std/hashmap.vais` | 해시 맵 | [examples/simple_hashmap_test.vais](https://github.com/vaislang/vais/blob/main/examples/simple_hashmap_test.vais) |
| `std/json.vais` | JSON 파싱/생성 | [examples/json_test.vais](https://github.com/vaislang/vais/blob/main/examples/json_test.vais) |
| `std/io.vais` | 파일 I/O | [examples/io_test.vais](https://github.com/vaislang/vais/blob/main/examples/io_test.vais) |
| `std/string.vais` | 문자열 처리 | [examples/string_test.vais](https://github.com/vaislang/vais/blob/main/examples/string_test.vais) |

**실습**: JSON 파일을 읽고 특정 필드를 추출하는 프로그램 작성

### 2.6 실전 프로젝트: CLI 도구 (30분)

**읽기**: [Tutorial: CLI Tool 만들기](./tutorials/cli-tool.md)

**실습**: 간단한 텍스트 처리 CLI 도구를 처음부터 작성

### 2.7 Defer와 리소스 관리 (20분)

**읽기**: [Defer Statement](./language/defer-statement.md)

```vais
F process_file(path: str) -> i64 {
    fd := open(path, 0)
    D close(fd)    # 함수 종료 시 자동 실행
    # fd를 사용한 처리...
    0
}
```

**실습**: 파일 열기/닫기에 defer 패턴 적용

**참고 예제**: [examples/defer_test.vais](https://github.com/vaislang/vais/blob/main/examples/defer_test.vais)

### Stage 2 체크리스트

- [ ] 제네릭 함수와 구조체 작성 가능
- [ ] Trait 정의, 구현, 바운드 사용
- [ ] 에러 처리 패턴 (Option, Result, ?) 적용
- [ ] 클로저와 파이프 연산자 활용
- [ ] 표준 라이브러리 주요 모듈 사용
- [ ] Defer로 리소스 관리
- [ ] 100줄 이상의 프로그램 독립 작성 가능

---

## Stage 3: 고급 (4시간)

> **대상**: Stage 2를 완료했거나 시스템 프로그래밍/타입 시스템에 익숙한 개발자

### 3.1 매크로 시스템 (30분)

**읽기**: [Macro System](./language/macros.md)

```vais
macro debug! {
    ($expr) => {
        println("DEBUG: ~{$expr}")
    }
}
```

**실습**: `assert!` 매크로 + 사용자 정의 매크로 작성

**참고 예제**: [examples/macro_test.vais](https://github.com/vaislang/vais/blob/main/examples/macro_test.vais)

### 3.2 비동기 프로그래밍 (40분)

**읽기**: [Async Programming](./language/async-tutorial.md)

```vais
A F fetch_data(url: str) -> str {
    response := Y http_get(url)
    R response
}

F main() {
    data := spawn fetch_data("http://example.com")
    # ...다른 작업...
    result := Y data
}
```

**실습**: 비동기 HTTP 클라이언트 작성

**참고 예제**: [examples/async_test.vais](https://github.com/vaislang/vais/blob/main/examples/async_test.vais)

### 3.3 FFI와 C 상호운용 (40분)

**읽기**: [FFI Guide](./advanced/ffi/guide.md)

```vais
N "C" {
    F printf(fmt: str, ...) -> i32
    F strlen(s: str) -> i64
}

F main() {
    len := strlen("hello")
    printf("Length: %lld\n", len)
}
```

**실습**: C 라이브러리 바인딩 작성 (예: SQLite 기본 연동)

**참고 예제**: [examples/ffi_test.vais](https://github.com/vaislang/vais/blob/main/examples/ffi_test.vais)

### 3.4 WASM 컴파일 (30분)

**읽기**: [WASM Getting Started](./advanced/wasm/getting-started.md)

```bash
vaisc --target wasm32-unknown-unknown calculator.vais
```

```vais
#[wasm_export("add")]
F add(a: i32, b: i32) -> i32 = a + b
```

**실습**: 간단한 계산기를 WASM으로 컴파일하고 브라우저에서 실행

**참고 예제**: [examples/wasm_calculator.vais](https://github.com/vaislang/vais/blob/main/examples/wasm_calculator.vais)

### 3.5 타입 시스템 심화 (40분)

**읽기**: [Advanced Types](./language/advanced-types.md), [Type Inference](./language/type-inference.md)

고급 타입 기능:
- Where 절: `F foo<T>(x: T) -> T where T: Printable`
- Trait 별칭: `T Numeric = Add + Sub + Mul`
- 연관 타입: `W Container { T Item; F get(&self) -> Self.Item }`

**실습**: where 절과 trait 별칭을 사용하는 제네릭 컬렉션 라이브러리 작성

### 3.6 성능 최적화 (20분)

**읽기**: [Performance Guide](./guides/performance.md)

최적화 기법:
- `comptime` 컴파일 타임 평가
- `inline` 힌트
- 배열 대신 포인터 직접 조작
- SIMD 벡터 연산

**참고 예제**: [examples/bench_fibonacci.vais](https://github.com/vaislang/vais/blob/main/examples/bench_fibonacci.vais), [examples/simd_test.vais](https://github.com/vaislang/vais/blob/main/examples/simd_test.vais)

### 3.7 실전 프로젝트: REST API 서버 (40분)

**읽기**: [Tutorial: HTTP Server 만들기](./tutorials/http-server.md)

**실습**: HTTP 서버 + JSON API + 데이터 저장 프로젝트 작성

### Stage 3 체크리스트

- [ ] 선언적 매크로 작성 가능
- [ ] async/await + spawn 사용
- [ ] C FFI 바인딩 작성
- [ ] WASM 타겟 컴파일 및 실행
- [ ] 고급 타입 시스템 (where, trait alias, associated types) 활용
- [ ] 성능 최적화 기법 적용
- [ ] 500줄 이상의 실전 프로젝트 독립 완성 가능

---

## 대상별 학습 트랙

### Systems Programmer (C/C++/Rust 경험자)

| 순서 | 주제 | 소요 시간 | 비고 |
|------|------|-----------|------|
| 1 | Stage 1.1~1.4 (문법 속성) | 1시간 | 단일 문자 키워드 매핑 중심 |
| 2 | Stage 2.1~2.3 (제네릭/에러) | 1시간 | Rust와 유사한 패턴 |
| 3 | Stage 3.3 (FFI) | 40분 | C 라이브러리 연동 |
| 4 | Stage 3.6 (성능) | 20분 | LLVM 최적화 활용 |
| 5 | [셀프호스팅 설계](./advanced/self-hosting-design.md) | 30분 | 컴파일러 내부 구조 이해 |

**추천 예제**: `bench_sorting.vais`, `ffi_test.vais`, `simd_test.vais`

### Web Developer (WASM 관심)

| 순서 | 주제 | 소요 시간 | 비고 |
|------|------|-----------|------|
| 1 | Stage 1 전체 | 2시간 | 기본 문법 |
| 2 | Stage 2.4~2.5 (클로저/Stdlib) | 1시간 | 함수형 패턴 |
| 3 | Stage 3.4 (WASM) | 30분 | 핵심 목표 |
| 4 | [WASM 컴포넌트](./advanced/wasm/component-model.md) | 30분 | 고급 WASM |
| 5 | [JS Interop](./advanced/wasm/js-interop.md) | 30분 | JavaScript 연동 |

**추천 예제**: `wasm_calculator.vais`, `wasm_todo_app.vais`, `js_target.vais`

### AI/ML Developer (GPU 활용)

| 순서 | 주제 | 소요 시간 | 비고 |
|------|------|-----------|------|
| 1 | Stage 1 전체 | 2시간 | 기본 문법 |
| 2 | Stage 2.1~2.2 (제네릭/Trait) | 1시간 | 추상화 패턴 |
| 3 | [GPU Codegen](./compiler/gpu-codegen.md) | 40분 | CUDA/Metal/OpenCL |
| 4 | Stage 3.2 (비동기) | 40분 | 병렬 데이터 처리 |
| 5 | SIMD 벡터 연산 | 20분 | 고성능 수치 계산 |

**추천 예제**: `gpu_vector_add.vais`, `simd_test.vais`, `simd_distance.vais`

---

## 추가 자료

- [언어 레퍼런스](./language/language-spec.md) - 완전한 문법 스펙
- [표준 라이브러리](./stdlib/stdlib.md) - 74개 모듈 레퍼런스
- [API 문서](./api/index.md) - 타입별 API 레퍼런스
- [트러블슈팅](./troubleshooting.md) - 자주 묻는 질문과 해결 방법
- [기여 가이드](./contributing/contributing.md) - 컴파일러 개발에 참여하기
- [Playground](https://vais.dev/playground/) - 브라우저에서 Vais 실행
- [인터랙티브 튜토리얼](./tools/vais-tutorial/README.md) - 단계별 연습 문제
