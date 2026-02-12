# Getting Started with Vais

Vais는 AI 최적화된 시스템 프로그래밍 언어입니다. 단일 문자 키워드, 완전한 타입 추론, LLVM 백엔드를 제공합니다. 이 가이드를 따라 Vais를 설치하고 첫 번째 프로그램을 작성해보세요.

## 설치

### 시스템 요구사항

- Rust 1.70 이상
- LLVM 17
- Git

### 소스에서 빌드

```bash
# Vais 저장소 클론
git clone https://github.com/vaislang/vais.git
cd vais

# 컴파일러 빌드
cargo build --release

# 설치 (선택 사항)
cargo install --path crates/vaisc
```

### 바이너리 설치

최신 릴리스에서 직접 다운로드할 수 있습니다:

```bash
# macOS / Linux
curl -L https://github.com/vaislang/vais/releases/latest/download/vaisc-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m) -o vaisc
chmod +x vaisc
sudo mv vaisc /usr/local/bin/
```

## Hello World

### 첫 번째 프로그램 작성

`hello.vais` 파일을 생성합니다:

```vais
F main() {
    println("Hello, Vais!")
}
```

> **참고**: `main()` 함수는 반환 타입을 생략할 수 있습니다. 생략 시 암시적으로 `i64` 반환 타입이 적용되며, 명시적 `R`(return) 없이 종료하면 `0`을 반환합니다. 명시적으로 `F main() -> i64 { ... }` 형태도 여전히 지원됩니다.

### 컴파일 및 실행

```bash
# 방법 1: 컴파일 후 실행
vaisc build hello.vais -o hello
./hello

# 방법 2: 직접 실행 (vaisc run 사용)
vaisc run hello.vais

# 방법 3: REPL에서 실행
vaisc repl
> puts("Hello, Vais!")
```

## 첫 번째 프로젝트

### 프로젝트 구조 만들기

```bash
# 새로운 프로젝트 초기화
mkdir my-vais-app
cd my-vais-app

# 기본 폴더 구조
mkdir src
mkdir bin
touch Vais.toml
```

### Vais.toml 작성

프로젝트 메타데이터 파일을 작성합니다:

```toml
[package]
name = "my-vais-app"
version = "0.1.0"
edition = "2024"

[dependencies]

[dev-dependencies]
```

### 소스 코드 구조

```
my-vais-app/
├── Vais.toml
├── src/
│   ├── lib.vais       # 라이브러리 코드
│   └── main.vais      # 메인 엔트리 포인트
└── bin/
    └── cli.vais       # 추가 바이너리 (선택 사항)
```

### 프로젝트 컴파일 및 실행

```bash
# 프로젝트 빌드
vaisc build

# 최적화 빌드
vaisc build --release

# 프로젝트 실행
vaisc run

# 테스트 실행
vaisc test
```

## 기본 문법 투어

### 변수 선언

```vais
# 불변 변수
x := 42
name := "Vais"

# 가변 변수 (mut 키워드)
counter := mut 0
counter = counter + 1

# 축약형 (~ = mut)
# ⚠️ ~ 축약형은 레거시 — := mut 권장
~ total := 0
total = total + 5

# 타입 명시 (선택 사항)
age: i64 = 25
pi: f64 = 3.14159
```

### 함수 정의

```vais
# 간단한 함수
F add(a: i64, b: i64) -> i64 = a + b

# 함수 본문 포함
F greet(name: str) -> str {
    message := "Hello, ~{name}!"
    message
}

# 반환값 명시
F factorial(n: i64) -> i64 {
    I n <= 1 {
        R 1
    }
    R n * factorial(n - 1)
}

# 기본값 없는 반환 (0 반환)
F print_info(msg: str) {
    puts(msg)
}
```

### 조건문

```vais
x := 42

# if/else 기본
I x > 0 {
    puts("positive")
} E {
    puts("not positive")
}

# if/else if/else
I x > 100 {
    puts("greater than 100")
} I x > 50 {
    puts("greater than 50")
} E {
    puts("50 or less")
}

# 삼항 연산자
result := x > 0 ? "positive" : "non-positive"
```

### 반복문

```vais
# for 루프 (범위)
F print_range() {
    L i:0..5 {
        puts("~{i}")
    }
}

# 무한 루프
F infinite_loop_example() {
    count := mut 0
    L {
        I count >= 10 {
            B
        }
        puts("~{count}")
        count = count + 1
    }
}

# while 루프 (조건 기반)
F while_example() {
    x := mut 0
    L x < 10 {
        puts("~{x}")
        x = x + 2
    }
}
```

### 구조체

```vais
# 구조체 정의
S Point {
    x: i64
    y: i64
}

S Person {
    name: str
    age: i64
    email: str
}

# 구조체 인스턴스 생성
p := Point { x: 10, y: 20 }

# 필드 접근
puts("~{p.x}, ~{p.y}")

# 구조체 메서드 (impl 블록)
X Point {
    F distance_from_origin(&self) -> f64 {
        a := self.x as f64
        b := self.y as f64
        sqrt(a * a + b * b)
    }
}

# 메서드 호출
dist := p.distance_from_origin()
```

### Enum (열거형)

```vais
# 간단한 Enum
E Color {
    Red,
    Green,
    Blue
}

# 데이터를 포함한 Enum
E Result<T> {
    Ok(T),
    Err(str)
}

# Enum 사용
color := Color.Red

M color {
    Color.Red => puts("Red color"),
    Color.Green => puts("Green color"),
    Color.Blue => puts("Blue color")
}
```

### 패턴 매칭

```vais
status := 200

# 기본 match
M status {
    200 => puts("OK"),
    404 => puts("Not Found"),
    500 => puts("Server Error"),
    _ => puts("Unknown status")
}

# Enum과 함께 사용
E Response {
    Success(str),
    Failure(str)
}

response := Response.Success("Done")

M response {
    Response.Success(msg) => puts("Success: ~{msg}"),
    Response.Failure(err) => puts("Error: ~{err}")
}
```

### 제네릭

```vais
# 제네릭 함수
F max<T>(a: T, b: T) -> T {
    I a > b { R a } E { R b }
}

# 제네릭 구조체
S Container<T> {
    value: T
}

X Container<T> {
    F get_value(&self) -> T = self.value
}

# 제네릭 사용
container := Container<i64> { value: 42 }
val := container.get_value()
```

### Trait (특성)

```vais
# Trait 정의
W Drawable {
    F draw(&self) -> i64
}

# 구조체 정의
S Circle {
    radius: i64
}

# Trait 구현
X Circle: Drawable {
    F draw(&self) -> i64 {
        puts("Drawing circle with radius ~{self.radius}")
        0
    }
}

# Trait 메서드 호출
circle := Circle { radius: 5 }
circle.draw()
```

## 실제 예제

### 간단한 계산기

```vais
S Calculator {
    result: i64
}

X Calculator {
    F add(&self, n: i64) -> i64 = self.result + n
    F subtract(&self, n: i64) -> i64 = self.result - n
    F multiply(&self, n: i64) -> i64 = self.result * n
}

F main() -> i64 {
    calc := mut Calculator { result: 0 }
    calc.result = calc.add(10)
    calc.result = calc.multiply(2)
    calc.result = calc.subtract(5)
    puts("Result: ~{calc.result}")
    0
}
```

### 문자열 처리

```vais
F count_chars(s: str) -> i64 {
    # 문자열 길이 계산
    len := mut 0
    # (실제로는 s.len() 메서드 사용)
    len
}

F main() -> i64 {
    greeting := "Hello, Vais!"
    puts(greeting)

    # 문자열 보간
    name := "World"
    message := "Hello, ~{name}!"
    puts(message)

    0
}
```

### 배열 처리

```vais
F sum_array(arr: [i64; 5]) -> i64 {
    result := mut 0
    L i:0..5 {
        result = result + arr[i]
    }
    result
}

F main() {
    numbers := [1, 2, 3, 4, 5]
    total := sum_array(numbers)
    println("Sum: ~{total}")
}
```

### 빌트인 함수

Vais 컴파일러가 제공하는 내장 함수입니다:

| 함수 | 설명 | 예시 |
|------|------|------|
| `swap(ptr, i, j)` | 배열 요소 교환 | `swap(arr, 0, 2)` |
| `sizeof(expr)` | 표현식 크기 (바이트) | `sizeof(x)` |
| `type_size<T>()` | 타입 T의 크기 | `type_size<i64>()` |
| `store_byte(ptr, offset, val)` | 바이트 저장 | `store_byte(buf, 0, 65)` |
| `load_byte(ptr, offset)` | 바이트 로드 | `load_byte(buf, 0)` |
| `puts(msg)` | 문자열 출력 + 줄바꿈 | `puts("hello")` |
| `putchar(c)` | 문자 출력 | `putchar(65)` |
| `println(msg)` | 문자열 출력 + 줄바꿈 | `println("hello")` |

#### swap — 배열 요소 교환

`swap(ptr, idx1, idx2)` 빌트인으로 배열 요소를 교환할 수 있습니다:

```vais
F main() {
    arr: *i64 = [10, 20, 30]
    swap(arr, 0, 2)       # arr[0]과 arr[2] 교환
    # arr = [30, 20, 10]
    println("~{load_i64(arr + 0 * 8)}")  # 30
}
```

## 다음 단계

- [언어 레퍼런스](../language/language-spec.md): 완전한 언어 사양
- [표준 라이브러리](../stdlib/stdlib.md): 내장 함수와 타입
- [에러 처리 가이드](./error-handling.md): 에러 처리 패턴
- [성능 튜닝 가이드](./performance.md): 최적화 기법
- [코딩 스타일 가이드](./style-guide.md): 커뮤니티 스타일 가이드
