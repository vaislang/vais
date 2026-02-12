# Vais 팀 온보딩 가이드

Vais 프로그래밍 언어를 학습하고 숙달하기 위한 2주 커리큘럼입니다. 이 가이드는 기초부터 고급 주제까지 체계적으로 진행되며, 매일의 학습 목표와 실제 코드 예제를 제공합니다.

---

## 1주차: 기본 개념 이해

### Day 1-2: 기본 문법 (Basic Syntax)

#### 학습 목표
- Vais의 단일 문자 키워드 시스템 이해
- 변수 바인딩 및 기본 타입 학습
- 첫 번째 함수 작성

#### 핵심 개념

**키워드 매핑**

| 키워드 | 의미 | 사용 예 |
|--------|------|--------|
| `F` | Function (함수) | `F add(a: i64, b: i64) -> i64` |
| `I` | If (조건문) | `I x > 0 { ... }` |
| `E` | Else (선택적) | `E { ... }` |
| `L` | Loop (반복문) | `L { ... }` |
| `M` | Match (패턴 매칭) | `M value { pattern => ... }` |
| `R` | Return (반환) | `R x + y` |
| `B` | Break (루프 탈출) | `B` |
| `S` | Struct (구조체) | `S Point { x: i64, y: i64 }` |
| `E` | Enum (열거형) | `E Result { Ok, Err }` |
| `T` | Trait (특성) | `T Comparable { ... }` |
| `Y` | Await (비동기 대기) | `result.Y` |
| `~` | Mut 축약 (가변) | `~ x := 0` |

**토큰 절감 문법 (v1.0)**

| 문법 | 설명 | 예시 |
|------|------|------|
| `{expr}` | 문자열 보간 | `println("x={x}")` |
| `\|>` | 파이프 연산자 | `x \|> f \|> g` |
| `~` | mut 축약 | `~ count := 0` |
| `Y` | await 축약 | `data.Y` |
| `(a, b) :=` | 디스트럭처링 | `(x, y) := get_pair()` |

**변수 바인딩**

```vais
# 불변 변수 바인딩
x := 42
name := "Vais"

# 가변 변수 바인딩
counter := mut 0
counter = counter + 1

# ~ 축약 (mut의 단축 표기)
~ total := 0
total = total + 1
```

**기본 타입**

```vais
# 정수형
a: i64 = 100
b: i32 = 50

# 부동소수점
pi: f64 = 3.14159
f: f32 = 2.71828

# 문자열과 문자
greeting: str = "Hello, Vais!"
ch: i8 = 65  # 'A'

# 불린
is_valid: bool = true
is_error: bool = false
```

#### 실제 예제

**Hello World**

```vais
F main() -> i64 {
    println("Hello, Vais!")
    0
}
```

**기본 산술 연산**

```vais
F add(a: i64, b: i64) -> i64 = a + b

F subtract(a: i64, b: i64) -> i64 = a - b

F multiply(a: i64, b: i64) -> i64 = a * b

F divide(a: i64, b: i64) -> i64 = a / b

F main() -> i64 {
    sum := add(10, 20)
    product := multiply(sum, 2)
    product
}
```

**조건부 표현식 (Ternary)**

```vais
F max(a: i64, b: i64) -> i64 = a > b ? a : b

F is_positive(n: i64) -> bool = n > 0 ? true : false

F main() -> i64 = max(42, 10)  # 42 반환
```

#### 실습 과제

1. "Hello, [이름]!" 을 출력하는 함수 작성
2. 두 수의 최대공약수를 구하는 함수 작성
3. 온도를 섭씨에서 화씨로 변환하는 함수 작성

#### 실행 방법

```bash
# 컴파일 및 실행
vaisc examples/hello.vais

# REPL 사용 (대화형)
vaisc repl
```

---

### Day 3-4: 데이터 구조 (Data Structures)

#### 학습 목표
- 구조체(Struct) 정의 및 사용
- 열거형(Enum) 및 패턴 매칭 이해
- 기본적인 제네릭 사용

#### 구조체 (Structs)

```vais
# 구조체 정의
S Point {
    x: i64
    y: i64
}

# 메서드 정의 (impl 블록)
X impl Point {
    F new(x: i64, y: i64) -> Self {
        Self { x: x, y: y }
    }

    F distance_from_origin(self) -> f64 {
        # 거리 = sqrt(x^2 + y^2)
        # 단순화: 제곱의 합만 계산
        ((self.x * self.x) + (self.y * self.y)) as f64
    }
}
```

**구조체 사용 예제**

```vais
F main() -> i64 {
    # 구조체 인스턴스 생성
    p := Point { x: 3, y: 4 }

    # 필드 접근
    x_coord := p.x
    y_coord := p.y

    0
}
```

#### 열거형 (Enums)

```vais
# 기본 열거형
E Color {
    Red,
    Green,
    Blue
}

# 값을 가진 열거형
E Result<T, E> {
    Ok(T),
    Err(E)
}

E Option<T> {
    Some(T),
    None
}
```

**패턴 매칭 (Pattern Matching)**

```vais
F describe_color(color: Color) -> str {
    M color {
        Color::Red => "빨간색입니다",
        Color::Green => "초록색입니다",
        Color::Blue => "파란색입니다",
        _ => "알 수 없는 색상입니다"
    }
}

F handle_result(result: Result<i64, str>) -> i64 {
    M result {
        Result::Ok(value) => value,
        Result::Err(error) => {
            puts(error)
            0
        }
    }
}
```

#### 제네릭 기본

```vais
# 제네릭 구조체
S Box<T> {
    value: T
}

# 제네릭 함수
F identity<T>(x: T) -> T = x

F get_first<T>(arr: *T) -> T = arr[0]

F main() -> i64 {
    box_int := Box { value: 42 }
    box_str := Box { value: "Hello" }

    box_int.value
}
```

#### 실제 데이터 모델 예제

```vais
S User {
    id: i64
    name: str
    email: str
    age: i64
}

X impl User {
    F new(id: i64, name: str, email: str, age: i64) -> Self {
        Self { id: id, name: name, email: email, age: age }
    }

    F is_adult(self) -> bool = self.age >= 18
}

E UserResult {
    Success(User),
    NotFound,
    InvalidData
}

F main() -> i64 {
    user := User::new(1, "Alice", "alice@example.com", 25)
    I user.is_adult() {
        puts("Adult user")
    }
    0
}
```

#### 실습 과제

1. `Person` 구조체 정의: name, age, email 필드
2. `Status` 열거형: Active, Inactive, Suspended
3. 제네릭 `Container<T>` 구조체 작성
4. 패턴 매칭으로 `Status` 처리하는 함수 작성

---

### Day 5-6: 제어 흐름 (Control Flow)

#### 학습 목표
- 조건부 제어 흐름 (if/else) 작성
- 루프 및 반복 패턴 이해
- 자기 호출 연산자 `@` 활용
- 클로저(Closure) 기본 사용

#### 조건부 표현식

```vais
# If/Else 표현식 - 모든 것이 표현식
F classify(n: i64) -> str {
    I n > 0 {
        "양수"
    } E I n < 0 {
        "음수"
    } E {
        "0"
    }
}

# 중첩된 조건
F check_grade(score: i64) -> str {
    I score >= 90 {
        "A"
    } E I score >= 80 {
        "B"
    } E I score >= 70 {
        "C"
    } E {
        "F"
    }
}
```

#### 루프 (Loops)

```vais
# 기본 루프
F count_to(n: i64) -> i64 {
    L {
        I n <= 0 {
            B  # Break로 루프 탈출
        }
        n = n - 1
    }
    0
}

# 루프 내에서의 누적 (~ 축약 사용)
F sum_n(n: i64) -> i64 {
    ~ total := 0
    ~ counter := 1
    L {
        I counter > n {
            B
        }
        total = total + counter
        counter = counter + 1
    }
    total
}

# while 패턴 (조건 체크)
F factorial(n: i64) -> i64 {
    result := mut 1
    counter := mut 1
    L {
        I counter > n {
            B
        }
        result = result * counter
        counter = counter + 1
    }
    result
}
```

#### 재귀와 자기 호출 연산자 `@`

```vais
# 자기 호출로 Fibonacci 계산
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)

# 팩토리얼
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)

# 카운트다운
F countdown(n: i64) -> i64 = n < 1 ? 0 : @(n - 1)

# 리스트의 합 계산 (포인터 사용)
F sum_array(arr: *i64, len: i64, idx: i64) -> i64 {
    idx >= len ? 0 : arr[idx] + @(arr, len, idx + 1)
}

F main() -> i64 = fib(10)  # 55
```

#### 파이프 연산자 (Pipe Operator)

```vais
# |> 연산자로 함수를 연쇄 호출
F double(x: i64) -> i64 = x * 2
F add_one(x: i64) -> i64 = x + 1

# 왼쪽 값을 오른쪽 함수의 첫 인자로 전달
F main() -> i64 {
    # 5 |> double |> add_one = add_one(double(5)) = 11
    result := 5 |> double |> add_one
    result
}
```

#### 문자열 보간 (String Interpolation)

```vais
F main() -> i64 {
    name := "Vais"
    x := 42

    # {expr} 으로 변수/표현식 삽입
    println("Hello, {name}!")
    println("x = {x}, x*2 = {x * 2}")

    # {{ }} 으로 중괄호 이스케이프
    println("Literal: {{braces}}")
    0
}
```

#### 클로저 (Closures)

```vais
# 기본 클로저
F test_closure() -> i64 {
    multiplier := 10

    # 변수를 캡처하는 클로저
    scale := |x: i64| x * multiplier

    result := scale(5)  # 50
    result
}

# 여러 변수 캡처
F multiple_capture() -> i64 {
    base := 20
    offset := 3

    compute := |x: i64| base + x + offset

    result := compute(7)  # 30
    result
}

# 클로저를 함수 인자로 전달
F apply_twice<T>(f: |T| -> T, x: T) -> T {
    f(f(x))
}

F main() -> i64 {
    increment := |x: i64| x + 1
    result := apply_twice(increment, 5)  # 7
    result
}
```

#### 실제 제어 흐름 예제

```vais
# 리스트 처리 (포인터 배열)
F process_list(arr: *i64, len: i64) -> i64 {
    idx := mut 0
    sum := mut 0

    L {
        I idx >= len {
            B
        }
        sum = sum + arr[idx]
        idx = idx + 1
    }

    sum
}

# 필터링 패턴
F count_positives(arr: *i64, len: i64, idx: i64) -> i64 {
    idx >= len ? 0 : (arr[idx] > 0 ? 1 : 0) + @(arr, len, idx + 1)
}

# 매핑 패턴
F double_value(x: i64) -> i64 = x * 2
```

#### 실습 과제

1. 피보나치 수열의 첫 10개 항 출력
2. 1부터 100까지의 합 계산 (루프 사용)
3. 주어진 배열에서 최댓값 찾기
4. 클로저를 사용한 숫자 필터링 함수
5. 중첩 루프로 구구단 출력 (2단~9단)

---

## 2주차: 고급 개념 및 실전

### Day 7-8: 트레이트와 제네릭 심화 (Traits and Generics)

#### 학습 목표
- 트레이트(Trait) 정의 및 구현
- 제네릭 제약조건(Bounds) 사용
- 트레이트를 통한 다형성 구현

#### 트레이트 정의

```vais
# 트레이트 정의
T Comparable {
    F compare(self, other: Self) -> i64
}

T Drawable {
    F draw(self) -> str
}

T Container<T> {
    F add(mut self, item: T)
    F remove(mut self, idx: i64) -> T
    F len(self) -> i64
}
```

#### 트레이트 구현

```vais
S Number {
    value: i64
}

# Number에 Comparable 구현
X impl Comparable for Number {
    F compare(self, other: Self) -> i64 {
        self.value - other.value
    }
}

# 사용 예제
F main() -> i64 {
    n1 := Number { value: 10 }
    n2 := Number { value: 20 }
    result := n1.compare(n2)  # -10
    result
}
```

#### 제네릭 제약조건

```vais
# 제네릭 타입에 트레이트 바운드 추가
F max<T: Comparable>(a: T, b: T) -> T {
    a.compare(b) > 0 ? a : b
}

F print_all<T: Drawable>(items: *T, len: i64) {
    idx := mut 0
    L {
        I idx >= len {
            B
        }
        puts(items[idx].draw())
        idx = idx + 1
    }
}

# 여러 트레이트 바운드
T Serializable {
    F to_string(self) -> str
}

F serialize<T: Comparable + Drawable>(item: T) -> str {
    item.draw()
}
```

#### 연습용 트레이트 시스템

```vais
# 수학 연산 인터페이스
T MathOps {
    F add(self, other: Self) -> Self
    F multiply(self, scale: i64) -> Self
}

S Vector2D {
    x: i64
    y: i64
}

X impl MathOps for Vector2D {
    F add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }

    F multiply(self, scale: i64) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale
        }
    }
}

F process_vectors<T: MathOps>(v1: T, v2: T, scale: i64) -> T {
    result := v1.add(v2)
    result.multiply(scale)
}
```

#### 실습 과제

1. `Printable` 트레이트 정의 및 구현
2. `Comparable` 트레이트를 사용한 제네릭 정렬 함수
3. 여러 타입에 대한 `Into<T>` 트레이트 구현
4. 트레이트 바운드를 활용한 범용 필터링 함수

---

### Day 9-10: 개발 도구 (Development Tools)

#### 학습 목표
- 컴파일러 CLI 사용법
- REPL 대화형 개발
- LSP를 통한 IDE 통합
- 프로젝트 구조 및 패키지 관리

#### 컴파일러 CLI (vaisc)

```bash
# 기본 컴파일
vaisc build hello.vais -o hello

# 최적화 레벨 설정
vaisc build hello.vais -O 2

# 여러 파일 컴파일
vaisc build main.vais utils.vais -o myapp

# 직접 실행 (컴파일 + 실행)
vaisc run hello.vais

# LLVM IR 생성
vaisc emit-llvm hello.vais

# 어셈블리 출력
vaisc emit-asm hello.vais
```

#### REPL 사용

```bash
# REPL 시작
vaisc repl

# REPL 내에서:
> x := 42
> x + 8
50
> F add(a, b) = a + b
> add(10, 20)
30
> :quit
```

#### LSP 지원 (IDE 통합)

```bash
# VSCode 확장 설치
cd vscode-vais
npm install
npm run build

# VSCode에서 확장 실행
# Command Palette: "Vais: Start LSP Server"
```

**지원되는 LSP 기능:**
- 문법 강조 (Syntax Highlighting)
- 자동 완성 (Autocomplete)
- 정의로 이동 (Go to Definition)
- 호버 정보 (Hover Information)
- 오류 진단 (Diagnostics)
- 코드 포맷팅 (Code Formatting)

#### 프로젝트 구조 및 vais.toml

```toml
# vais.toml - 프로젝트 설정
[project]
name = "my_project"
version = "0.1.0"
edition = "2024"

[dependencies]
# 표준 라이브러리 자동 포함
std = "1.0"

[dev-dependencies]
# 테스트 전용 의존성

[build]
opt-level = 2
target = "native"
```

**프로젝트 디렉토리 구조:**

```
my_project/
├── vais.toml           # 프로젝트 메니페스트
├── src/
│   ├── main.vais       # 메인 엔트리 포인트
│   ├── lib.vais        # 라이브러리 코드
│   └── utils/
│       └── helpers.vais
├── tests/
│   └── integration_tests.vais
├── examples/
│   └── demo.vais
└── README.md
```

#### 실습 과제

1. 간단한 프로젝트 구조 생성 (vais.toml 포함)
2. 여러 모듈로 구성된 프로그램 작성
3. VSCode에서 LSP를 통한 자동 완성 테스트
4. REPL에서 다양한 표현식 테스트

---

### Day 11-12: 프로젝트 실습 (Project Practice)

#### 학습 목표
- 실제 프로젝트 계획 및 구현
- 테스트 작성 및 검증
- 코드 구조화 및 모듈화
- Vais 커뮤니티 패턴 학습

#### 프로젝트 1: 간단한 계산기

```vais
# calc.vais - 간단한 계산기
E Operation {
    Add,
    Subtract,
    Multiply,
    Divide
}

S Calculator {
    last_result: i64
}

X impl Calculator {
    F new() -> Self {
        Self { last_result: 0 }
    }

    F execute(mut self, op: Operation, a: i64, b: i64) -> i64 {
        result := M op {
            Operation::Add => a + b,
            Operation::Subtract => a - b,
            Operation::Multiply => a * b,
            Operation::Divide => a / b,
            _ => 0
        }
        self.last_result = result
        result
    }

    F get_last(self) -> i64 = self.last_result
}

F main() -> i64 {
    calc := mut Calculator::new()

    result1 := calc.execute(Operation::Add, 10, 20)
    puts("10 + 20 = 50")

    result2 := calc.execute(Operation::Multiply, result1, 2)
    puts("50 * 2 = 100")

    0
}
```

#### 프로젝트 2: 투두 리스트

```vais
E TaskStatus {
    Pending,
    InProgress,
    Completed
}

S Task {
    id: i64
    title: str
    status: TaskStatus
}

S TodoList {
    tasks: *Task
    count: i64
}

X impl TodoList {
    F new(capacity: i64) -> Self {
        Self {
            tasks: malloc(capacity),
            count: 0
        }
    }

    F add_task(mut self, id: i64, title: str) {
        I self.count < 10 {  # 최대 10개
            self.tasks[self.count] = Task {
                id: id,
                title: title,
                status: TaskStatus::Pending
            }
            self.count = self.count + 1
        }
    }

    F complete_task(mut self, id: i64) {
        idx := mut 0
        L {
            I idx >= self.count {
                B
            }
            I self.tasks[idx].id == id {
                self.tasks[idx].status = TaskStatus::Completed
                B
            }
            idx = idx + 1
        }
    }

    F list_tasks(self) {
        idx := mut 0
        L {
            I idx >= self.count {
                B
            }
            task := self.tasks[idx]
            puts(task.title)
            idx = idx + 1
        }
    }
}

F main() -> i64 {
    todo := mut TodoList::new(10)

    todo.add_task(1, "Learn Vais")
    todo.add_task(2, "Build a project")
    todo.add_task(3, "Deploy to production")

    todo.list_tasks()

    todo.complete_task(1)

    0
}
```

#### 테스트 작성

```vais
# tests/calculator_tests.vais

# 테스트 헬퍼 함수
F assert_equal(expected: i64, actual: i64, msg: str) -> bool {
    I expected == actual {
        puts("✓ PASS: " + msg)
        true
    } E {
        puts("✗ FAIL: " + msg)
        false
    }
}

F test_addition() -> bool {
    result := 10 + 20
    assert_equal(30, result, "10 + 20 = 30")
}

F test_fibonacci() -> bool {
    F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
    result := fib(10)
    assert_equal(55, result, "fib(10) = 55")
}

F main() -> i64 {
    passed := mut 0
    total := mut 0

    I test_addition() { passed = passed + 1 }
    total = total + 1

    I test_fibonacci() { passed = passed + 1 }
    total = total + 1

    puts("Tests passed: " + (passed as str))
    puts("Total: " + (total as str))

    0
}
```

#### Vais 코딩 패턴

```vais
# 패턴 1: 옵션 타입 처리
E Maybe<T> {
    Just(T),
    Nothing
}

F map<T, U>(maybe: Maybe<T>, f: |T| -> U) -> Maybe<U> {
    M maybe {
        Maybe::Just(value) => Maybe::Just(f(value)),
        Maybe::Nothing => Maybe::Nothing
    }
}

# 패턴 2: 에러 처리
E Outcome<T, E> {
    Success(T),
    Failure(E)
}

# 패턴 3: 빌더 패턴
S ConfigBuilder {
    host: str
    port: i64
    timeout: i64
}

X impl ConfigBuilder {
    F new() -> Self {
        Self {
            host: "localhost",
            port: 8080,
            timeout: 30
        }
    }

    F with_host(mut self, host: str) -> Self {
        self.host = host
        self
    }

    F with_port(mut self, port: i64) -> Self {
        self.port = port
        self
    }

    F build(self) -> str {
        self.host  # 실제로는 Config 구조체 반환
    }
}

# 사용
F main() -> i64 {
    config := ConfigBuilder::new()
        .with_host("example.com")
        .with_port(9000)
        .build()

    0
}
```

#### 실습 과제

1. 계산기 프로젝트 완성
2. 투두 리스트에 검색 기능 추가
3. 테스트 스위트 작성
4. 에러 처리 메커니즘 추가
5. 빌더 패턴을 사용한 설정 시스템

---

### Day 13-14: 고급 기능 (Advanced Features)

#### 학습 목표
- 비동기 프로그래밍 기초
- 모듈 시스템 이해
- FFI(외부 함수 인터페이스) 사용
- 매크로 및 고급 메타프로그래밍

#### 비동기 프로그래밍 (Async/Await)

```vais
# 비동기 함수 정의
F async fetch_data(url: str) -> str {
    # 실제 HTTP 요청 수행
    "response data"
}

# await를 사용한 비동기 대기 (Y 축약 사용 가능)
F async process_data() -> i64 {
    data := fetch_data("https://api.example.com").Y
    println("Data: {data}")
    0
}

# 여러 비동기 작업 동시 실행
F async concurrent_tasks() {
    task1 := fetch_data("url1")
    task2 := fetch_data("url2")

    result1 := await task1
    result2 := await task2

    puts(result1 + result2)
}
```

#### 모듈 시스템

**모듈 파일 구조:**

```
src/
├── main.vais
├── lib.vais
├── math/
│   ├── lib.vais     # mod math
│   ├── algebra.vais # pub mod algebra
│   └── geometry.vais
└── utils/
    └── helpers.vais
```

**모듈 선언과 사용:**

```vais
# math/lib.vais
pub mod algebra
pub mod geometry

# math/algebra.vais
pub F solve_quadratic(a: i64, b: i64, c: i64) -> (i64, i64) {
    # 구현
    (0, 0)
}

# main.vais
use math::algebra
use math::geometry

F main() -> i64 {
    result := algebra::solve_quadratic(1, 2, 1)
    0
}
```

#### FFI (Foreign Function Interface)

```vais
# C 함수 바인딩
extern {
    F printf(format: *i8, ...) -> i32
    F malloc(size: i64) -> *void
    F free(ptr: *void)
}

# Vais에서 C 라이브러리 호출
F print_c(msg: str) -> i32 {
    printf("Message: %s\n", msg)
}

F main() -> i64 {
    ptr := malloc(1024)
    print_c("Hello from FFI")
    free(ptr)
    0
}
```

#### 매크로 기초

```vais
# 간단한 매크로
macro debug_print(expr) {
    puts("Debug: " + expr)
}

macro assert(condition, message) {
    I !condition {
        puts("Assertion failed: " + message)
        panic()
    }
}

# 사용
F test_macro() {
    x := 42
    debug_print("x = " + (x as str))
    assert(x > 0, "x must be positive")
}

# 패턴 매크로
macro unless(condition, body) {
    I !condition {
        body
    }
}

F main() -> i64 {
    x := 10
    unless(x > 20, {
        puts("x is not greater than 20")
    })
    0
}
```

#### 고급 타입 시스템

```vais
# 제네릭 타입 별칭
type List<T> = *T
type Dict<K, V> = S { keys: *K, values: *V }

# 고급 트레이트
T Iterator<T> {
    F next(mut self) -> Maybe<T>
    F count(mut self) -> i64
}

# 조건부 컴파일
#[cfg(target = "x86_64")]
F cpu_optimized() -> i64 = 42

#[cfg(target = "wasm")]
F wasm_version() -> i64 = 24
```

#### 프로덕션 코드 예제

```vais
# 웹 서버의 핵심 로직
S HttpServer {
    host: str
    port: i64
    handlers: *str  # 경로별 핸들러
}

X impl HttpServer {
    F new(host: str, port: i64) -> Self {
        Self {
            host: host,
            port: port,
            handlers: malloc(100)
        }
    }

    F register_route(mut self, path: str, handler: str) {
        # 라우트 등록
    }

    F start(self) -> i64 {
        puts("Server starting on " + self.host + ":" + (self.port as str))
        # 서버 루프
        0
    }
}

F main() -> i64 {
    server := mut HttpServer::new("0.0.0.0", 8080)
    server.register_route("/", "index_handler")
    server.register_route("/api", "api_handler")
    server.start()
}
```

#### 실습 과제

1. 비동기 데이터 처리 파이프라인 구현
2. 재사용 가능한 모듈 라이브러리 작성
3. C 라이브러리와의 FFI 통합
4. 유용한 매크로 작성 (프로젝트별 반복 코드 제거)
5. 실제 프로덕션 규모의 작은 프로젝트 완성

---

## 참고 자료 (References)

### 공식 문서
- [Vais 공식 사이트](https://vaislang.dev)
- [언어 사양](./language/language-spec.md)
- [표준 라이브러리](./stdlib/stdlib.md)

### 학습 자료
- [온라인 튜토리얼](./getting-started/tutorial.md)
- [인터랙티브 플레이그라운드](./tools/playground/README.md)
- [예제 코드 모음](https://github.com/vaislang/vais/tree/main/examples)

### 커뮤니티
- [GitHub 이슈](https://github.com/vaislang/vais/issues)
- [디스커션 포럼](https://github.com/vaislang/vais/discussions)
- [기여 가이드](./contributing/contributing.md)

### 추가 학습
- [비동기 프로그래밍 가이드](./language/async-tutorial.md)
- [제네릭 심화](./language/generics.md)
- [FFI 가이드](./advanced/ffi/README.md)
- [컴파일러 아키텍처](./compiler/architecture.md)

---

## 체크리스트: 온보딩 완료 확인

다음을 확인하면 온보딩이 완료되었습니다:

### 1주차
- [ ] 모든 기본 키워드 이해 및 사용
- [ ] 변수 바인딩 및 기본 타입 숙달
- [ ] 간단한 함수 작성 가능
- [ ] 구조체 및 열거형 정의 및 사용
- [ ] 패턴 매칭 기본 이해
- [ ] If/Else 및 루프 제어 흐름 작성
- [ ] 재귀 및 자기 호출 연산자 사용
- [ ] 클로저 작성 및 캡처 이해

### 2주차
- [ ] 트레이트 정의 및 구현
- [ ] 제네릭 바운드 활용
- [ ] CLI 컴파일러 능숙한 사용
- [ ] REPL에서 대화형 개발
- [ ] VSCode LSP 통합
- [ ] 실제 프로젝트 구현 (계산기, 투두 등)
- [ ] 테스트 코드 작성
- [ ] 비동기 및 모듈 시스템 기본 이해

### 최종 목표
- [ ] Vais로 간단한 CLI 도구 개발 가능
- [ ] 표준 라이브러리 활용 능력
- [ ] 팀 프로젝트에 즉시 참여 가능
- [ ] 커뮤니티 가이드라인 숙지

---

## 추가 팁과 모범 사례

### 디버깅
```bash
# 상세 컴파일 메시지
vaisc build --verbose hello.vais

# LLVM IR 검사
vaisc emit-llvm hello.vais > output.ll

# 최적화 확인
vaisc build -O 3 hello.vais
```

### 성능 최적화
- 재귀 대신 루프 사용 (스택 오버플로우 방지)
- 불필요한 클론 피하기
- 메모리 할당 최소화
- 컴파일타임에 계산할 수 있는 것은 컴파일 시간에 처리

### 코드 스타일
```vais
# 좋은 예
F calculate_average(values: *i64, len: i64) -> i64 =
    sum_array(values, len) / len

# 나쁜 예
F calculate_average(values:*i64,len:i64)->i64=sum_array(values,len)/len
```

### 커뮤니티 참여
- GitHub Discussions에서 질문하기
- 예제 코드 공유
- 버그 리포트 및 기능 제안
- 문서 개선 사항 제출

---

**이제 Vais 개발자로서의 여정을 시작할 준비가 되었습니다. 행운을 빕니다!**
