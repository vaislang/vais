# Rust에서 Vais로 전환하기

## 개요

Vais는 Rust와 유사한 타입 시스템과 소유권 모델을 가지고 있지만, AI 최적화를 위해 단일 문자 키워드를 사용합니다. Rust 개발자라면 Vais의 핵심 개념이 익숙할 것입니다.

**주요 차이점:**
- 단일 문자 키워드 (`F`, `S`, `E`, `I`, `L`, `M`, `R` 등)
- `@` 연산자로 자기 재귀 표현
- `:=` 로 변수 바인딩
- 간결한 문법 (세미콜론 선택적)
- LLVM 기반 컴파일러
- 완전한 타입 추론

## 키워드 대조표

| Rust | Vais | 설명 |
|------|------|------|
| `fn` | `F` | 함수 정의 |
| `struct` | `S` | 구조체 |
| `enum` | `E` | 열거형 |
| `if` | `I` | 조건문 |
| `else` | `E` | else 절 |
| `loop` | `L` | 무한 루프 |
| `while` | `W` + `L` | while 루프 (W는 trait) |
| `for` | 없음 | L + iterator 사용 |
| `match` | `M` | 패턴 매칭 |
| `trait` | `W` | 트레이트 (Witness) |
| `impl` | `X` | impl 블록 |
| `type` | `T` | 타입 별칭 |
| `use` | `U` | 모듈 import |
| `return` | `R` | 반환 |
| `break` | `B` | 루프 탈출 |
| `continue` | `C` | 루프 계속 |
| `let` | `:=` | 변수 바인딩 |
| `mut` | `mut` | 가변성 |
| `//` | `#` | 주석 |
| `async` | `A` | 비동기 함수 |
| `await` | `Y` | await 연산자 |
| - | `@` | 자기 재귀 |

## 타입 매핑

### 기본 타입

```rust
// Rust
let x: i8 = 42;
let y: i16 = 100;
let z: i32 = 1000;
let w: i64 = 10000;
let a: u8 = 255;
let b: u32 = 4294967295;
let f: f32 = 3.14;
let d: f64 = 2.718;
let s: &str = "hello";
let b: bool = true;
```

```vais
# Vais
x := 42i8
y := 100i16
z := 1000i32
w := 10000          # i64가 기본
a := 255u8
b := 4294967295u32
f := 3.14f32
d := 2.718          # f64가 기본
s := "hello"        # str 타입
b := true           # bool 타입
```

### 컬렉션 타입

```rust
// Rust
let arr: [i32; 3] = [1, 2, 3];
let vec: Vec<i32> = vec![1, 2, 3];
let opt: Option<i32> = Some(42);
let res: Result<i32, String> = Ok(42);
```

```vais
# Vais
arr := [1, 2, 3]              # [i32; 3]
vec := Vec::new()             # Vec<i32>
opt := Some(42)               # Option<i32>
res := Ok(42)                 # Result<i32, str>
```

## 함수 정의

### 기본 함수

```rust
// Rust
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn greet(name: &str) -> String {
    format!("Hello, {}", name)
}
```

```vais
# Vais
F add(a: i64, b: i64) -> i64 = a + b

F greet(name: str) -> str {
    "Hello, " + name
}
```

### 제네릭 함수

```rust
// Rust
fn identity<T>(x: T) -> T {
    x
}

fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

```vais
# Vais
F identity<T>(x: T) -> T = x

F max<T: Ord>(a: T, b: T) -> T {
    I a > b { a } E { b }
}
```

### 메서드

```rust
// Rust
impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

```vais
# Vais
X Point {
    F new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    F distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

## 구조체

```rust
// Rust
struct Point {
    x: f64,
    y: f64,
}

struct Rectangle {
    top_left: Point,
    width: f64,
    height: f64,
}

// 튜플 구조체
struct Color(u8, u8, u8);
```

```vais
# Vais
S Point {
    x: f64,
    y: f64,
}

S Rectangle {
    top_left: Point,
    width: f64,
    height: f64,
}

# 튜플 구조체
S Color(u8, u8, u8)
```

## 열거형

### 기본 열거형

```rust
// Rust
enum Color {
    Red,
    Green,
    Blue,
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}
```

```vais
# Vais
E Color {
    Red,
    Green,
    Blue,
}

E Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(str),
    ChangeColor(u8, u8, u8),
}
```

### Option과 Result

```rust
// Rust
fn divide(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

fn parse_number(s: &str) -> Result<i64, String> {
    s.parse().map_err(|_| "Invalid number".to_string())
}
```

```vais
# Vais
F divide(a: i64, b: i64) -> Option<i64> {
    I b == 0 {
        None
    } E {
        Some(a / b)
    }
}

F parse_number(s: str) -> Result<i64, str> {
    # parse 함수 사용
    parse_i64(s)
}
```

## 에러 처리

### ? 연산자

```rust
// Rust
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    Ok(content.to_uppercase())
}

fn process() -> Result<i64, String> {
    let x = parse_number("42")?;
    let y = parse_number("10")?;
    Ok(x + y)
}
```

```vais
# Vais
U std/io

F read_file(path: str) -> Result<str, str> {
    content := read_to_string(path)?
    R Ok(content.to_uppercase())
}

F process() -> Result<i64, str> {
    x := parse_number("42")?
    y := parse_number("10")?
    R Ok(x + y)
}
```

### unwrap/expect

```rust
// Rust
let x = Some(42).unwrap();
let y = parse_number("100").expect("Failed to parse");
```

```vais
# Vais
x := Some(42)!          # ! = unwrap
y := parse_number("100")!
```

## 패턴 매칭

### 기본 매칭

```rust
// Rust
fn describe(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1 => "one",
        2..=10 => "small",
        _ => "large",
    }
}
```

```vais
# Vais
F describe(n: i32) -> str {
    M n {
        0 => "zero",
        1 => "one",
        2..=10 => "small",
        _ => "large",
    }
}
```

### 열거형 매칭

```rust
// Rust
fn area(shape: Shape) -> f64 {
    match shape {
        Shape::Circle(r) => 3.14159 * r * r,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle(b, h) => 0.5 * b * h,
    }
}
```

```vais
# Vais
F area(shape: Shape) -> f64 {
    M shape {
        Circle(r) => 3.14159 * r * r,
        Rectangle(w, h) => w * h,
        Triangle(b, h) => 0.5 * b * h,
    }
}
```

### Option/Result 매칭

```rust
// Rust
fn process_option(opt: Option<i32>) -> i32 {
    match opt {
        Some(x) => x * 2,
        None => 0,
    }
}

fn process_result(res: Result<i32, String>) -> i32 {
    match res {
        Ok(x) => x,
        Err(_) => -1,
    }
}
```

```vais
# Vais
F process_option(opt: Option<i32>) -> i32 {
    M opt {
        Some(x) => x * 2,
        None => 0,
    }
}

F process_result(res: Result<i32, str>) -> i32 {
    M res {
        Ok(x) => x,
        Err(_) => -1,
    }
}
```

## 트레이트와 Impl

### 트레이트 정의

```rust
// Rust
trait Display {
    fn display(&self) -> String;
}

trait Add<T> {
    type Output;
    fn add(self, rhs: T) -> Self::Output;
}
```

```vais
# Vais
W Display {
    F display(&self) -> str;
}

W Add<T> {
    T Output;
    F add(self, rhs: T) -> Output;
}
```

### 트레이트 구현

```rust
// Rust
impl Display for Point {
    fn display(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
```

```vais
# Vais
X Display for Point {
    F display(&self) -> str {
        "(" + self.x.to_string() + ", " + self.y.to_string() + ")"
    }
}

X Add<Point> for Point {
    T Output = Point;

    F add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
```

### 제네릭 제약

```rust
// Rust
fn print_if_display<T: Display>(value: T) {
    println!("{}", value.display());
}

fn compare<T: Ord + Clone>(a: T, b: T) -> T {
    if a > b { a.clone() } else { b.clone() }
}
```

```vais
# Vais
F print_if_display<T: Display>(value: T) {
    println(value.display())
}

F compare<T: Ord + Clone>(a: T, b: T) -> T {
    I a > b { a.clone() } E { b.clone() }
}
```

## Async/Await

```rust
// Rust
async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

async fn process_data() -> Result<(), Box<dyn std::error::Error>> {
    let data1 = fetch_url("https://api.example.com/1").await?;
    let data2 = fetch_url("https://api.example.com/2").await?;

    // Process data...

    Ok(())
}
```

```vais
# Vais
A F fetch_url(url: str) -> Result<str, str> {
    response := http_get(url).Y
    body := response.text().Y
    R Ok(body)
}

A F process_data() -> Result<(), str> {
    data1 := fetch_url("https://api.example.com/1").Y
    data2 := fetch_url("https://api.example.com/2").Y

    # Process data...

    R Ok(())
}
```

## 소유권과 차용

### 소유권 이동

```rust
// Rust
fn take_ownership(s: String) {
    println!("{}", s);
}

fn main() {
    let s = String::from("hello");
    take_ownership(s);
    // s는 더 이상 사용 불가
}
```

```vais
# Vais
F take_ownership(s: str) {
    println(s)
}

F main() {
    s := "hello"
    take_ownership(s)
    # s는 더 이상 사용 불가
}
```

### 참조와 차용

```rust
// Rust
fn calculate_length(s: &String) -> usize {
    s.len()
}

fn append(s: &mut String, suffix: &str) {
    s.push_str(suffix);
}

fn main() {
    let s = String::from("hello");
    let len = calculate_length(&s);

    let mut s2 = String::from("hello");
    append(&mut s2, " world");
}
```

```vais
# Vais
F calculate_length(s: &str) -> i64 {
    s.len()
}

F append(s: &mut str, suffix: str) {
    s.push_str(suffix)
}

F main() {
    s := "hello"
    len := calculate_length(&s)

    s2 := mut "hello"
    append(&mut s2, " world")
}
```

## 모듈 시스템

### 모듈 정의와 사용

```rust
// Rust
// math.rs
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn multiply(a: i64, b: i64) -> i64 {
    a * b
}

// main.rs
mod math;
use math::{add, multiply};

fn main() {
    let sum = add(1, 2);
    let product = multiply(3, 4);
}
```

```vais
# Vais
# math.vais
F add(a: i64, b: i64) -> i64 = a + b
F multiply(a: i64, b: i64) -> i64 = a * b

# main.vais
U math

F main() {
    sum := add(1, 2)
    product := multiply(3, 4)
}
```

### 표준 라이브러리 사용

```rust
// Rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut file = File::open("data.txt")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut map = HashMap::new();
    map.insert("key", "value");

    Ok(())
}
```

```vais
# Vais
U std/collections
U std/io

F main() -> Result<(), str> {
    file := File::open("data.txt")?
    contents := file.read_to_string()?

    map := HashMap::new()
    map.insert("key", "value")

    R Ok(())
}
```

## 자기 재귀 (`@` 연산자)

Vais의 독특한 기능 중 하나는 `@` 연산자를 사용한 자기 재귀입니다.

```rust
// Rust
fn factorial(n: i64) -> i64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```

```vais
# Vais
F factorial(n: i64) -> i64 {
    I n <= 1 { 1 } E { n * @(n - 1) }
}

F fibonacci(n: i64) -> i64 {
    I n <= 1 { n } E { @(n - 1) + @(n - 2) }
}
```

## 루프

### 무한 루프

```rust
// Rust
loop {
    if should_break {
        break;
    }
    // work...
}
```

```vais
# Vais
L {
    I should_break { B }
    # work...
}
```

### while 루프 스타일

```rust
// Rust
let mut i = 0;
while i < 10 {
    println!("{}", i);
    i += 1;
}
```

```vais
# Vais
i := mut 0
L {
    I i >= 10 { B }
    println(i)
    i = i + 1
}
```

### for 루프 스타일 (iterator)

```rust
// Rust
for i in 0..10 {
    println!("{}", i);
}

for item in vec.iter() {
    println!("{}", item);
}
```

```vais
# Vais
# Range iterator 사용
range := 0..10
L {
    opt := range.next()
    M opt {
        Some(i) => println(i),
        None => B,
    }
}

# Vec iterator 사용
iter := vec.iter()
L {
    opt := iter.next()
    M opt {
        Some(item) => println(item),
        None => B,
    }
}
```

## 마무리

Vais는 Rust의 강력한 타입 시스템과 소유권 모델을 유지하면서도 더 간결한 문법을 제공합니다. 주요 차이점은:

1. **단일 문자 키워드**: 코드 길이 단축 및 AI 최적화
2. **`@` 연산자**: 재귀 호출을 명시적으로 표현
3. **`:=` 바인딩**: `let` 대신 간결한 표기
4. **표현식 중심**: 대부분의 구문이 값을 반환
5. **간소화된 모듈 시스템**: `U` 키워드로 통합

Rust 개발자라면 Vais의 학습 곡선이 매우 낮을 것입니다. 기존 Rust 코드를 Vais로 포팅할 때는 주로 키워드만 변경하면 됩니다.
