# C/C++에서 Vais로 전환하기

## 개요

Vais는 C/C++와 달리 현대적인 타입 시스템, 메모리 안전성, 그리고 AI 최적화 문법을 제공합니다. C/C++ 개발자를 위해 주요 차이점과 전환 가이드를 제공합니다.

**주요 개선사항:**
- 메모리 안전성: 자동 메모리 관리, 댕글링 포인터 방지
- 타입 안전성: 강력한 타입 추론, 컴파일 타임 체크
- 에러 처리: `errno` 대신 `Result<T, E>` 타입
- 제네릭: 템플릿보다 간결하고 명확한 제네릭
- 모던 문법: 단일 문자 키워드로 간결한 코드

## 키워드 대조표

| C/C++ | Vais | 설명 |
|-------|------|------|
| `int`, `long` | `i32`, `i64` | 정수 타입 |
| `float`, `double` | `f32`, `f64` | 실수 타입 |
| `char*`, `const char*` | `str` | 문자열 |
| `void` | `()` | Unit 타입 |
| `struct` | `S` | 구조체 |
| `enum` | `E` | 열거형 |
| `typedef` | `T` | 타입 별칭 |
| `if` | `I` | 조건문 |
| `else` | `E` | else 절 |
| `while`, `for` | `L` | 루프 |
| `switch` | `M` | 패턴 매칭 |
| `return` | `R` | 반환 |
| `break` | `B` | 루프 탈출 |
| `continue` | `C` | 루프 계속 |
| `//`, `/* */` | `#` | 주석 |

## 타입 매핑

### 기본 타입

```c
// C
char c = 'a';
short s = 100;
int i = 1000;
long l = 100000L;
unsigned int ui = 4000000000U;

float f = 3.14f;
double d = 2.718;

const char* str = "hello";
void* ptr = malloc(100);
```

```vais
# Vais
c := 'a'              # char (u8)
s := 100i16           # i16
i := 1000i32          # i32
l := 100000           # i64 (기본)
ui := 4000000000u32   # u32

f := 3.14f32          # f32
d := 2.718            # f64 (기본)

str := "hello"        # str 타입
ptr := malloc(100)    # 포인터
```

### 배열과 포인터

```c
// C
int arr[5] = {1, 2, 3, 4, 5};
int* ptr = arr;

int** matrix = (int**)malloc(10 * sizeof(int*));
for (int i = 0; i < 10; i++) {
    matrix[i] = (int*)malloc(10 * sizeof(int));
}
```

```vais
# Vais
arr := [1, 2, 3, 4, 5]     # [i32; 5] 배열
ptr := &arr[0]              # 포인터

# Vec 사용 (동적 배열)
matrix := Vec::new()
i := mut 0
L {
    I i >= 10 { B }
    row := Vec::new()
    matrix.push(row)
    i = i + 1
}
```

## 함수 정의

### 기본 함수

```c
// C
int add(int a, int b) {
    return a + b;
}

double square(double x) {
    return x * x;
}

void print_message(const char* msg) {
    printf("%s\n", msg);
}
```

```vais
# Vais
F add(a: i32, b: i32) -> i32 = a + b

F square(x: f64) -> f64 = x * x

F print_message(msg: str) {
    println(msg)
}
```

### 함수 포인터

```c
// C
typedef int (*BinaryOp)(int, int);

int apply(int a, int b, BinaryOp op) {
    return op(a, b);
}

int multiply(int a, int b) {
    return a * b;
}

int main() {
    int result = apply(3, 4, multiply);
    return 0;
}
```

```vais
# Vais
T BinaryOp = F(i32, i32) -> i32

F apply(a: i32, b: i32, op: BinaryOp) -> i32 {
    R op(a, b)
}

F multiply(a: i32, b: i32) -> i32 = a * b

F main() -> i32 {
    result := apply(3, 4, multiply)
    R result
}
```

## 구조체

### 구조체 정의

```c
// C
struct Point {
    double x;
    double y;
};

struct Rectangle {
    struct Point top_left;
    double width;
    double height;
};

typedef struct {
    int r;
    int g;
    int b;
} Color;
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

S Color {
    r: i32,
    g: i32,
    b: i32,
}
```

### 구조체 메서드

```c
// C
struct Point {
    double x;
    double y;
};

double point_distance(const struct Point* p) {
    return sqrt(p->x * p->x + p->y * p->y);
}

struct Point point_new(double x, double y) {
    struct Point p = {x, y};
    return p;
}
```

```vais
# Vais
S Point {
    x: f64,
    y: f64,
}

X Point {
    F new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    F distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

# 사용
F main() {
    p := Point::new(3.0, 4.0)
    d := p.distance()    # 5.0
}
```

## 포인터와 메모리

### 포인터 사용

```c
// C
void swap(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
    int x = 10, y = 20;
    swap(&x, &y);
    printf("x=%d, y=%d\n", x, y);
    return 0;
}
```

```vais
# Vais
F swap(a: &mut i32, b: &mut i32) {
    temp := *a
    *a = *b
    *b = temp
}

F main() {
    x := mut 10
    y := mut 20
    swap(&mut x, &mut y)
    println("x=" + x.to_string() + ", y=" + y.to_string())
}
```

### 동적 메모리 할당

```c
// C
int* allocate_array(size_t n) {
    int* arr = (int*)malloc(n * sizeof(int));
    if (arr == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < n; i++) {
        arr[i] = i;
    }
    return arr;
}

void use_array() {
    int* arr = allocate_array(10);
    if (arr != NULL) {
        // use arr...
        free(arr);
    }
}
```

```vais
# Vais
F allocate_array(n: i64) -> Vec<i32> {
    arr := Vec::new()
    i := mut 0
    L {
        I i >= n { B }
        arr.push(i as i32)
        i = i + 1
    }
    R arr
}

F use_array() {
    arr := allocate_array(10)
    # use arr...
    # 자동으로 메모리 해제됨
}
```

## 에러 처리

### errno vs Result

```c
// C
#include <stdio.h>
#include <errno.h>
#include <string.h>

FILE* open_file(const char* path) {
    FILE* f = fopen(path, "r");
    if (f == NULL) {
        fprintf(stderr, "Error: %s\n", strerror(errno));
        return NULL;
    }
    return f;
}

int read_number(const char* path) {
    FILE* f = open_file(path);
    if (f == NULL) {
        return -1;  // error indicator
    }

    int value;
    if (fscanf(f, "%d", &value) != 1) {
        fclose(f);
        return -1;
    }

    fclose(f);
    return value;
}
```

```vais
# Vais
U std/io

F open_file(path: str) -> Result<File, str> {
    M File::open(path) {
        Ok(f) => Ok(f),
        Err(e) => Err("Failed to open file: " + e),
    }
}

F read_number(path: str) -> Result<i32, str> {
    f := open_file(path)?
    line := f.read_line()?
    number := parse_i32(line)?
    R Ok(number)
}
```

## 제네릭

### C++ 템플릿 vs Vais 제네릭

```cpp
// C++
template<typename T>
T max(T a, T b) {
    return (a > b) ? a : b;
}

template<typename T>
class Vec {
private:
    T* data;
    size_t size;
    size_t capacity;

public:
    Vec() : data(nullptr), size(0), capacity(0) {}

    void push(T value) {
        if (size >= capacity) {
            resize();
        }
        data[size++] = value;
    }

    T get(size_t index) const {
        return data[index];
    }
};
```

```vais
# Vais
F max<T: Ord>(a: T, b: T) -> T {
    I a > b { a } E { b }
}

S Vec<T> {
    data: *mut T,
    size: i64,
    capacity: i64,
}

X Vec<T> {
    F new() -> Vec<T> {
        Vec { data: null(), size: 0, capacity: 0 }
    }

    F push(&mut self, value: T) {
        I self.size >= self.capacity {
            self.resize()
        }
        # store value...
        self.size = self.size + 1
    }

    F get(&self, index: i64) -> &T {
        # return reference...
    }
}
```

## 열거형

### C enum vs Vais enum

```c
// C
enum Color {
    RED,
    GREEN,
    BLUE
};

enum Status {
    OK = 0,
    ERROR = -1,
    PENDING = 1
};

void process_color(enum Color c) {
    switch (c) {
        case RED:
            printf("Red\n");
            break;
        case GREEN:
            printf("Green\n");
            break;
        case BLUE:
            printf("Blue\n");
            break;
    }
}
```

```vais
# Vais
E Color {
    Red,
    Green,
    Blue,
}

E Status {
    Ok,
    Error,
    Pending,
}

F process_color(c: Color) {
    M c {
        Red => println("Red"),
        Green => println("Green"),
        Blue => println("Blue"),
    }
}
```

### 값을 가진 열거형 (Tagged Union)

```c
// C
enum ShapeType {
    SHAPE_CIRCLE,
    SHAPE_RECTANGLE
};

struct Shape {
    enum ShapeType type;
    union {
        struct { double radius; } circle;
        struct { double width; double height; } rectangle;
    } data;
};

double shape_area(const struct Shape* s) {
    switch (s->type) {
        case SHAPE_CIRCLE:
            return 3.14159 * s->data.circle.radius * s->data.circle.radius;
        case SHAPE_RECTANGLE:
            return s->data.rectangle.width * s->data.rectangle.height;
        default:
            return 0.0;
    }
}
```

```vais
# Vais
E Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

F shape_area(s: Shape) -> f64 {
    M s {
        Circle(r) => 3.14159 * r * r,
        Rectangle(w, h) => w * h,
    }
}

# 사용
F main() {
    circle := Circle(5.0)
    rect := Rectangle(3.0, 4.0)

    a1 := shape_area(circle)     # 78.54
    a2 := shape_area(rect)        # 12.0
}
```

## 매크로

### C 전처리기 vs Vais

```c
// C
#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define SQUARE(x) ((x) * (x))
#define PI 3.14159

#ifdef DEBUG
    #define LOG(msg) printf("DEBUG: %s\n", msg)
#else
    #define LOG(msg)
#endif

int main() {
    int x = MAX(10, 20);
    double area = PI * SQUARE(5.0);
    LOG("Starting program");
    return 0;
}
```

```vais
# Vais
# 매크로 대신 인라인 함수 또는 상수 사용
F max(a: i32, b: i32) -> i32 = I a > b { a } E { b }
F square(x: f64) -> f64 = x * x

PI := 3.14159

#[cfg(debug)]
F log(msg: str) {
    println("DEBUG: " + msg)
}

#[cfg(not(debug))]
F log(msg: str) {
    # no-op
}

F main() -> i32 {
    x := max(10, 20)
    area := PI * square(5.0)
    log("Starting program")
    R 0
}
```

## 문자열 처리

### C 문자열 vs Vais 문자열

```c
// C
#include <string.h>
#include <stdlib.h>

char* concat(const char* a, const char* b) {
    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    char* result = malloc(len_a + len_b + 1);

    strcpy(result, a);
    strcat(result, b);

    return result;
}

void process_string() {
    const char* s1 = "Hello";
    const char* s2 = " World";
    char* s3 = concat(s1, s2);

    printf("%s\n", s3);

    free(s3);
}
```

```vais
# Vais
F concat(a: str, b: str) -> str {
    R a + b
}

F process_string() {
    s1 := "Hello"
    s2 := " World"
    s3 := concat(s1, s2)

    println(s3)

    # 자동 메모리 해제
}
```

## 입출력

### C stdio vs Vais I/O

```c
// C
#include <stdio.h>

void write_file(const char* path, const char* content) {
    FILE* f = fopen(path, "w");
    if (f == NULL) {
        fprintf(stderr, "Failed to open file\n");
        return;
    }

    fprintf(f, "%s", content);
    fclose(f);
}

void read_file(const char* path) {
    FILE* f = fopen(path, "r");
    if (f == NULL) {
        fprintf(stderr, "Failed to open file\n");
        return;
    }

    char buffer[256];
    while (fgets(buffer, sizeof(buffer), f) != NULL) {
        printf("%s", buffer);
    }

    fclose(f);
}
```

```vais
# Vais
U std/io

F write_file(path: str, content: str) -> Result<(), str> {
    f := File::create(path)?
    f.write_all(content)?
    R Ok(())
}

F read_file(path: str) -> Result<str, str> {
    f := File::open(path)?
    content := f.read_to_string()?
    R Ok(content)
}

F main() {
    M write_file("test.txt", "Hello, World!") {
        Ok(_) => println("Written successfully"),
        Err(e) => println("Error: " + e),
    }

    M read_file("test.txt") {
        Ok(content) => println(content),
        Err(e) => println("Error: " + e),
    }
}
```

## 주요 차이점 요약

### 메모리 관리

| C/C++ | Vais |
|-------|------|
| `malloc`/`free` | 자동 메모리 관리 |
| `new`/`delete` | 소유권 시스템 |
| 댕글링 포인터 위험 | 컴파일 타임 체크 |
| 메모리 누수 가능 | 자동 해제 |

### 에러 처리

| C/C++ | Vais |
|-------|------|
| `errno`, `-1` 반환 | `Result<T, E>` |
| `NULL` 체크 | `Option<T>` |
| 예외 (C++) | `?` 연산자 |
| `try-catch` (C++) | 패턴 매칭 |

### 타입 안전성

| C/C++ | Vais |
|-------|------|
| 암시적 변환 | 명시적 변환 필요 |
| `void*` 무타입 포인터 | 제네릭 타입 |
| 템플릿 에러 메시지 복잡 | 명확한 에러 메시지 |
| UB(Undefined Behavior) | 정의된 동작 |

## 마무리

Vais는 C/C++의 성능을 유지하면서 다음을 제공합니다:

1. **메모리 안전성**: 댕글링 포인터, 이중 해제 방지
2. **타입 안전성**: 강력한 타입 시스템, 컴파일 타임 체크
3. **모던 문법**: 간결하고 읽기 쉬운 코드
4. **에러 처리**: `Result`/`Option`으로 명시적 에러 처리
5. **제로 비용 추상화**: 런타임 오버헤드 없음

C/C++ 개발자가 Vais로 전환하면 생산성과 안전성이 크게 향상됩니다.
