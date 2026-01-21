# Vais 제네릭 및 트레이트 심화 튜토리얼

이 튜토리얼은 Vais의 제네릭 프로그래밍과 트레이트 시스템을 심층적으로 다룹니다. 타입 추론, 모노모피제이션, 그리고 실전 활용법을 배웁니다.

## 목차

1. [제네릭 개념](#제네릭-개념)
2. [제네릭 함수](#제네릭-함수)
3. [제네릭 구조체](#제네릭-구조체)
4. [제네릭 Enum](#제네릭-enum)
5. [트레이트 정의와 구현](#트레이트-정의와-구현)
6. [제네릭 바운드](#제네릭-바운드)
7. [표준 라이브러리 제네릭](#표준-라이브러리-제네릭)
8. [고급 패턴](#고급-패턴)
9. [타입 추론](#타입-추론)
10. [실전 예제](#실전-예제)

---

## 제네릭 개념

### 제네릭이란?

제네릭은 **타입을 매개변수화**하여 여러 타입에 대해 동작하는 코드를 작성할 수 있게 합니다.

**제네릭 없이:**
```vais
# i64용 함수
F identity_i64(x: i64) -> i64 = x

# f64용 함수
F identity_f64(x: f64) -> f64 = x

# str용 함수
F identity_str(x: str) -> str = x

# 각 타입마다 중복 코드!
```

**제네릭 사용:**
```vais
# 하나의 함수로 모든 타입 처리
F identity<T>(x: T) -> T = x

F main() -> i64 {
    a := identity(42)        # T = i64
    b := identity(3.14)      # T = f64
    c := identity("hello")   # T = str

    0
}
```

### Vais 제네릭의 특징

1. **Monomorphization**: 컴파일 시 각 사용된 타입에 대해 코드 생성
2. **Zero-cost Abstraction**: 런타임 오버헤드 없음
3. **타입 추론**: 대부분의 경우 타입 명시 불필요
4. **정적 디스패치**: 컴파일 타임에 모든 타입 결정

---

## 제네릭 함수

### 기본 제네릭 함수

단일 타입 매개변수:

```vais
# 가장 간단한 제네릭 함수
F identity<T>(x: T) -> T = x

# 여러 번 사용
F first<T>(a: T, b: T) -> T = a
F second<T>(a: T, b: T) -> T = b

# 블록 형태
F duplicate<T>(x: T) -> T {
    puts("Duplicating value")
    x
}

F main() -> i64 {
    # 타입 추론 - 명시 불필요
    x := identity(100)
    y := first(10, 20)
    z := duplicate(42)

    print_i64(x + y + z)  # 152
    0
}
```

### 다중 타입 매개변수

여러 타입 매개변수를 사용:

```vais
# 두 개의 타입 매개변수
F pair<A, B>(a: A, b: B) -> A = a

# 타입이 다른 값들 처리
F first_of_pair<A, B>(a: A, b: B) -> A = a
F second_of_pair<A, B>(a: A, b: B) -> B = b

# 삼중 타입
F choose_first<A, B, C>(a: A, b: B, c: C) -> A = a

F main() -> i64 {
    # A=i64, B=f64
    x := pair(10, 3.14)

    # A=i64, B=str
    y := first_of_pair(42, "hello")

    # A=i64, B=f64, C=str
    z := choose_first(100, 2.5, "world")

    print_i64(x + y + z)  # 152
    0
}
```

### 제네릭 함수의 타입 추론

Vais는 대부분의 경우 타입을 자동으로 추론합니다:

```vais
F swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
}

F main() -> i64 {
    # 타입 추론: A=i64, B=i64
    (x, y) := swap(10, 20)

    # 타입 추론: A=i64, B=str
    (num, text) := swap(42, "answer")

    print_i64(x)  # 20
    0
}
```

### 제네릭과 Self-Recursion

제네릭 함수에서 `@` 사용:

```vais
# 제네릭 재귀 함수
F repeat<T>(x: T, count: i64) -> T {
    I count <= 0 {
        x
    } E {
        @(x, count - 1)
    }
}

F main() -> i64 {
    result := repeat(42, 5)
    print_i64(result)  # 42
    0
}
```

---

## 제네릭 구조체

### 기본 제네릭 구조체

```vais
# 단일 타입 매개변수
S Box<T> {
    value: T
}

# 생성 및 사용
F main() -> i64 {
    # Box<i64>
    int_box := Box { value: 42 }

    # Box<f64>
    float_box := Box { value: 3.14 }

    print_i64(int_box.value)  # 42
    0
}
```

### 다중 타입 매개변수 구조체

```vais
# Pair - 두 개의 값을 담는 구조체
S Pair<T> {
    first: T,
    second: T
}

# Container - 서로 다른 타입
S Container<K, V> {
    key: K,
    value: V
}

# Triple - 세 개의 타입
S Triple<A, B, C> {
    first: A,
    second: B,
    third: C
}

F main() -> i64 {
    # Pair<i64>
    pair := Pair { first: 10, second: 20 }

    # Container<i64, str>
    container := Container { key: 1, value: "data" }

    # Triple<i64, f64, str>
    triple := Triple { first: 42, second: 3.14, third: "hello" }

    print_i64(pair.first)  # 10
    0
}
```

### 제네릭 구조체 메서드

`X` 키워드로 제네릭 구조체에 메서드 추가:

```vais
S Pair<T> {
    first: T,
    second: T
}

# Pair에 메서드 구현
X Pair {
    # self는 Pair<T> 타입
    F sum(&self) -> i64 {
        self.first + self.second
    }

    F swap(&self) -> Pair {
        Pair { first: self.second, second: self.first }
    }

    F first_value(&self) -> T {
        self.first
    }
}

F main() -> i64 {
    p := Pair { first: 10, second: 20 }

    # 메서드 호출
    total := p.sum()         # 30
    swapped := p.swap()      # Pair { first: 20, second: 10 }
    first := p.first_value() # 10

    print_i64(total)
    print_i64(swapped.first)
    print_i64(first)

    0
}
```

### 중첩 제네릭 구조체

```vais
S Box<T> {
    value: T
}

S Pair<T> {
    first: T,
    second: T
}

F main() -> i64 {
    # Box<i64>
    simple := Box { value: 42 }

    # Pair<i64>
    pair := Pair { first: 10, second: 20 }

    # Box<Pair<i64>> - 중첩 제네릭
    boxed_pair := Box {
        value: Pair { first: 1, second: 2 }
    }

    # 접근
    inner_pair := boxed_pair.value
    first_val := inner_pair.first

    print_i64(first_val)  # 1
    0
}
```

---

## 제네릭 Enum

### 기본 제네릭 Enum

```vais
# Option<T> - 값이 있거나 없음
E Option<T> {
    None,
    Some(T)
}

# Result<T, E> - 성공 또는 에러
E Result<T, E> {
    Ok(T),
    Err(E)
}

F main() -> i64 {
    # Option<i64>
    opt_int := Some(42)
    opt_none := None

    # Result<i64, str>
    result_ok := Ok(100)
    result_err := Err("Error message")

    0
}
```

### 제네릭 Enum 패턴 매칭

```vais
E Option<T> {
    None,
    Some(T)
}

F unwrap_or<T>(opt: Option<T>, default: T) -> T {
    M opt {
        Some(value) => value,
        None => default
    }
}

F main() -> i64 {
    opt1 := Some(42)
    opt2 := None

    v1 := unwrap_or(opt1, 0)   # 42
    v2 := unwrap_or(opt2, 99)  # 99

    print_i64(v1)
    print_i64(v2)

    0
}
```

### 다중 variant 제네릭 Enum

```vais
E Either<L, R> {
    Left(L),
    Right(R)
}

F process_either<L, R>(either: Either<L, R>) -> i64 {
    M either {
        Left(l) => {
            puts("Got Left")
            0
        },
        Right(r) => {
            puts("Got Right")
            1
        }
    }
}

F main() -> i64 {
    # Either<i64, str>
    left := Left(42)
    right := Right("hello")

    r1 := process_either(left)   # 0
    r2 := process_either(right)  # 1

    print_i64(r1 + r2)  # 1
    0
}
```

---

## 트레이트 정의와 구현

### 트레이트 정의

`W` 키워드로 트레이트(인터페이스) 정의:

```vais
# Printable 트레이트
W Printable {
    F print(&self) -> i64
}

# Comparable 트레이트
W Comparable {
    F compare(&self, other: &Self) -> i64
}

# 여러 메서드를 가진 트레이트
W Drawable {
    F draw(&self) -> i64
    F erase(&self) -> i64
    F move(&self, x: i64, y: i64) -> i64
}
```

### 트레이트 구현

`X` 키워드로 구조체에 트레이트 구현:

```vais
W Printable {
    F print(&self) -> i64
}

S Point {
    x: i64,
    y: i64
}

# Point에 Printable 구현
X Point: Printable {
    F print(&self) -> i64 {
        puts("Point(")
        print_i64(self.x)
        puts(", ")
        print_i64(self.y)
        puts(")")
        putchar(10)
        0
    }
}

F main() -> i64 {
    p := Point { x: 10, y: 20 }
    p.print()  # Point(10, 20)
    0
}
```

### 여러 트레이트 구현

```vais
W Printable {
    F print(&self) -> i64
}

W Resettable {
    F reset(&self) -> i64
}

S Counter {
    value: i64
}

# Printable 구현
X Counter: Printable {
    F print(&self) -> i64 {
        puts("Counter: ")
        print_i64(self.value)
        putchar(10)
        0
    }
}

# Resettable 구현
X Counter: Resettable {
    F reset(&self) -> i64 {
        self.value = 0
        0
    }
}

F main() -> i64 {
    c := Counter { value: 42 }
    c.print()   # Counter: 42
    c.reset()
    c.print()   # Counter: 0
    0
}
```

### 트레이트 없는 메서드 (Impl)

트레이트 없이 직접 메서드 추가:

```vais
S Rectangle {
    width: i64,
    height: i64
}

# 트레이트 없이 메서드 추가
X Rectangle {
    F area(&self) -> i64 {
        self.width * self.height
    }

    F perimeter(&self) -> i64 {
        (self.width + self.height) * 2
    }

    F is_square(&self) -> i64 {
        I self.width == self.height { 1 } E { 0 }
    }
}

F main() -> i64 {
    rect := Rectangle { width: 10, height: 20 }

    area := rect.area()           # 200
    perimeter := rect.perimeter() # 60
    is_sq := rect.is_square()     # 0

    print_i64(area)
    print_i64(perimeter)

    0
}
```

---

## 제네릭 바운드

### 트레이트 바운드 개념

제네릭 타입에 **제약 조건**을 추가:

```vais
# T는 Printable을 구현해야 함
F print_twice<T: Printable>(value: &T) -> i64 {
    value.print()
    value.print()
    0
}
```

### Where 절 (개념적)

복잡한 바운드 표현:

```vais
# 여러 바운드
F process<T>(value: T) -> i64
where
    T: Printable,
    T: Comparable
{
    value.print()
    0
}
```

### 다중 바운드 예제

```vais
W Display {
    F display(&self) -> i64
}

W Clone {
    F clone(&self) -> Self
}

S Point {
    x: i64,
    y: i64
}

X Point: Display {
    F display(&self) -> i64 {
        puts("(")
        print_i64(self.x)
        puts(",")
        print_i64(self.y)
        puts(")")
        0
    }
}

X Point: Clone {
    F clone(&self) -> Point {
        Point { x: self.x, y: self.y }
    }
}

F show_and_clone<T>(value: &T) -> T
where
    T: Display,
    T: Clone
{
    value.display()
    value.clone()
}

F main() -> i64 {
    p := Point { x: 10, y: 20 }
    p2 := show_and_clone(&p)

    p2.display()  # (10,20)
    0
}
```

---

## 표준 라이브러리 제네릭

### Vec<T> - 동적 배열

```vais
U std/vec

F main() -> i64 {
    # Vec<i64> 생성
    v := Vec.with_capacity(10)

    # 요소 추가
    v.push(10)
    v.push(20)
    v.push(30)

    # 길이 확인
    len := v.len()  # 3

    # 요소 접근
    first := v.get(0)   # 10
    second := v.get(1)  # 20

    # 요소 제거
    last := v.pop()  # 30

    print_i64(len)
    print_i64(first)
    print_i64(last)

    v.drop()  # 메모리 해제
    0
}
```

### Option<T> - 선택적 값

```vais
U std/option

F safe_divide(a: i64, b: i64) -> Option<i64> {
    I b == 0 {
        None
    } E {
        Some(a / b)
    }
}

F main() -> i64 {
    result := safe_divide(10, 2)

    # 패턴 매칭
    M result {
        Some(value) => {
            puts("Result: ")
            print_i64(value)  # 5
        },
        None => {
            puts("Division by zero!")
        }
    }

    # unwrap_or 메서드
    value := result.unwrap_or(0)
    print_i64(value)  # 5

    # is_some / is_none
    has_value := result.is_some()  # 1

    0
}
```

### Result<T, E> - 에러 처리

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}

F parse_positive(x: i64) -> Result<i64, str> {
    I x < 0 {
        Err("Negative number")
    } E I x == 0 {
        Err("Zero")
    } E {
        Ok(x)
    }
}

F main() -> i64 {
    result1 := parse_positive(42)
    result2 := parse_positive(-5)

    M result1 {
        Ok(value) => {
            puts("Success: ")
            print_i64(value)  # 42
        },
        Err(msg) => {
            puts("Error: ")
            puts(msg)
        }
    }

    M result2 {
        Ok(value) => {
            puts("Success: ")
            print_i64(value)
        },
        Err(msg) => {
            puts("Error: ")
            puts(msg)  # "Negative number"
        }
    }

    0
}
```

### HashMap<K, V> - 해시맵

```vais
U std/hashmap

F main() -> i64 {
    # HashMap<i64, i64>
    map := HashMap.with_capacity(10)

    # 삽입
    map.insert(1, 100)
    map.insert(2, 200)
    map.insert(3, 300)

    # 조회
    value := map.get(2)  # 200

    # 존재 확인
    exists := map.contains(1)  # 1

    # 크기
    size := map.len()  # 3

    print_i64(value)
    print_i64(size)

    map.drop()
    0
}
```

---

## 고급 패턴

### 제네릭 팩토리 패턴

```vais
S Box<T> {
    value: T
}

X Box<T> {
    # 정적 메서드 - 생성자
    F new(value: T) -> Box<T> {
        Box { value: value }
    }

    F get(&self) -> T {
        self.value
    }

    F set(&self, new_value: T) -> i64 {
        self.value = new_value
        0
    }
}

F main() -> i64 {
    # 팩토리 메서드 사용
    box1 := Box::new(42)
    box2 := Box::new(3.14)

    val1 := box1.get()  # 42

    box1.set(100)
    val2 := box1.get()  # 100

    print_i64(val1)
    print_i64(val2)

    0
}
```

### 제네릭 빌더 패턴

```vais
S Builder<T> {
    value: T,
    count: i64
}

X Builder<T> {
    F new(initial: T) -> Builder<T> {
        Builder { value: initial, count: 0 }
    }

    F with_count(&self, n: i64) -> Builder<T> {
        self.count = n
        self
    }

    F build(&self) -> T {
        self.value
    }
}

F main() -> i64 {
    builder := Builder::new(42)
    builder.with_count(10)
    result := builder.build()

    print_i64(result)  # 42
    0
}
```

### 제네릭 컨테이너 패턴

```vais
# Stack<T>
S Stack<T> {
    data: Vec<T>,
    top: i64
}

X Stack<T> {
    F new() -> Stack<T> {
        Stack {
            data: Vec.with_capacity(10),
            top: 0
        }
    }

    F push(&self, value: T) -> i64 {
        self.data.push(value)
        self.top = self.top + 1
        self.top
    }

    F pop(&self) -> Option<T> {
        I self.top > 0 {
            self.top = self.top - 1
            value := self.data.pop()
            Some(value)
        } E {
            None
        }
    }

    F is_empty(&self) -> i64 {
        I self.top == 0 { 1 } E { 0 }
    }
}

F main() -> i64 {
    stack := Stack::new()

    stack.push(10)
    stack.push(20)
    stack.push(30)

    M stack.pop() {
        Some(v) => print_i64(v),  # 30
        None => puts("Empty")
    }

    M stack.pop() {
        Some(v) => print_i64(v),  # 20
        None => puts("Empty")
    }

    0
}
```

---

## 타입 추론

### 타입 추론 규칙

Vais는 강력한 타입 추론을 제공합니다:

```vais
F identity<T>(x: T) -> T = x

F main() -> i64 {
    # 리터럴로부터 추론
    a := identity(42)        # T = i64
    b := identity(3.14)      # T = f64
    c := identity("text")    # T = str

    # 변수로부터 추론
    x := 100
    d := identity(x)         # T = i64 (x의 타입)

    # 반환 타입으로부터 추론
    result: i64 = identity(50)  # T = i64

    0
}
```

### 컨텍스트 기반 추론

```vais
S Pair<T> {
    first: T,
    second: T
}

F make_pair<T>(a: T, b: T) -> Pair<T> {
    Pair { first: a, second: b }
}

F main() -> i64 {
    # 반환 타입으로부터 추론
    p1: Pair<i64> = make_pair(10, 20)  # T = i64

    # 인자로부터 추론
    p2 := make_pair(1, 2)  # T = i64

    print_i64(p1.first)  # 10
    print_i64(p2.first)  # 1

    0
}
```

### 추론 실패와 명시적 타입

추론이 불가능한 경우:

```vais
# 이 함수는 T를 추론할 정보가 없음
F create_default<T>() -> T {
    # 기본값 반환
    0
}

F main() -> i64 {
    # 에러: T를 추론할 수 없음
    # x := create_default()

    # 해결: 타입 명시
    x: i64 = create_default()  # T = i64

    print_i64(x)
    0
}
```

---

## 실전 예제

### 예제 1: 제네릭 연결 리스트

```vais
E List<T> {
    Nil,
    Cons(T, i64)  # (value, next pointer)
}

X List<T> {
    F new() -> List<T> {
        Nil
    }

    F prepend(&self, value: T) -> List<T> {
        # self를 next로 하는 새 노드 생성
        Cons(value, 0)  # 간소화
    }

    F is_empty(&self) -> i64 {
        M self {
            Nil => 1,
            Cons(_, _) => 0
        }
    }

    F len(&self) -> i64 {
        M self {
            Nil => 0,
            Cons(_, next_ptr) => {
                # 재귀적으로 길이 계산
                1
            }
        }
    }
}

F main() -> i64 {
    list := List::new()

    node1 := list.prepend(10)
    node2 := node1.prepend(20)
    node3 := node2.prepend(30)

    is_empty := node3.is_empty()  # 0
    len := node3.len()             # 1

    print_i64(is_empty)
    print_i64(len)

    0
}
```

### 예제 2: 제네릭 Tree

```vais
E Tree<T> {
    Empty,
    Node(T, i64, i64)  # (value, left, right)
}

X Tree<T> {
    F empty() -> Tree<T> {
        Empty
    }

    F leaf(value: T) -> Tree<T> {
        Node(value, 0, 0)
    }

    F height(&self) -> i64 {
        M self {
            Empty => 0,
            Node(_, _, _) => 1
        }
    }

    F is_leaf(&self) -> i64 {
        M self {
            Empty => 0,
            Node(_, left, right) => {
                I left == 0 && right == 0 { 1 } E { 0 }
            }
        }
    }
}

F main() -> i64 {
    empty := Tree::empty()
    leaf := Tree::leaf(42)

    h1 := empty.height()    # 0
    h2 := leaf.height()     # 1
    is_l := leaf.is_leaf()  # 1

    print_i64(h1)
    print_i64(h2)
    print_i64(is_l)

    0
}
```

### 예제 3: 제네릭 Iterator

```vais
W Iterator<T> {
    F next(&self) -> Option<T>
    F has_next(&self) -> i64
}

S RangeIterator {
    current: i64,
    end: i64
}

X RangeIterator: Iterator<i64> {
    F next(&self) -> Option<i64> {
        I self.current < self.end {
            value := self.current
            self.current = self.current + 1
            Some(value)
        } E {
            None
        }
    }

    F has_next(&self) -> i64 {
        I self.current < self.end { 1 } E { 0 }
    }
}

F sum_iterator<T>(iter: &Iterator<T>) -> i64 {
    sum := 0

    L iter.has_next() {
        M iter.next() {
            Some(value) => {
                sum = sum + value
            },
            None => B
        }
    }

    sum
}

F main() -> i64 {
    iter := RangeIterator { current: 0, end: 10 }

    # 0+1+2+...+9 = 45
    total := sum_iterator(&iter)

    print_i64(total)  # 45
    0
}
```

### 예제 4: 제네릭 캐시

```vais
U std/hashmap
U std/option

S Cache<K, V> {
    map: HashMap<K, V>,
    capacity: i64
}

X Cache<K, V> {
    F new(capacity: i64) -> Cache<K, V> {
        Cache {
            map: HashMap.with_capacity(capacity),
            capacity: capacity
        }
    }

    F get(&self, key: K) -> Option<V> {
        has := self.map.contains(key)

        I has {
            value := self.map.get(key)
            Some(value)
        } E {
            None
        }
    }

    F put(&self, key: K, value: V) -> i64 {
        # 용량 체크
        I self.map.len() >= self.capacity {
            puts("Cache full!")
            0
        } E {
            self.map.insert(key, value)
            1
        }
    }

    F clear(&self) -> i64 {
        self.map.clear()
        0
    }
}

F main() -> i64 {
    cache := Cache::new(3)

    # 데이터 저장
    cache.put(1, 100)
    cache.put(2, 200)
    cache.put(3, 300)

    # 조회
    M cache.get(2) {
        Some(value) => {
            puts("Found: ")
            print_i64(value)  # 200
        },
        None => {
            puts("Not found")
        }
    }

    # 없는 키
    M cache.get(99) {
        Some(value) => {
            puts("Found: ")
            print_i64(value)
        },
        None => {
            puts("Not found")  # 이쪽 실행
        }
    }

    cache.clear()
    0
}
```

---

## 성능 고려사항

### Monomorphization의 장단점

**장점:**
- 런타임 오버헤드 없음 (virtual dispatch 불필요)
- 인라이닝 최적화 가능
- 타입별 최적화 가능

**단점:**
- 코드 크기 증가 (각 타입마다 별도 코드)
- 컴파일 시간 증가

### 제네릭 사용 팁

1. **자주 사용하는 타입만 제네릭으로:**
```vais
# 좋음: 재사용성 높음
F swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

# 피하기: 한 번만 사용
F process_int(x: i64) -> i64 = x * 2
# 제네릭 불필요
```

2. **타입 수 제한:**
```vais
# 많은 타입에 사용 -> 큰 바이너리
F generic<A, B, C, D, E>(...) -> ... { ... }

# 필요한 만큼만
F simple<T>(x: T) -> T { x }
```

3. **인라인 가능한 작은 함수:**
```vais
# 인라인되기 좋음
F identity<T>(x: T) -> T = x

# 큰 함수는 신중히
F complex<T>(x: T) -> T {
    # 100줄 코드...
}
```

---

## 요약

### 핵심 개념

1. **제네릭 함수**: `F name<T>(x: T) -> T`
2. **제네릭 구조체**: `S Name<T> { field: T }`
3. **제네릭 Enum**: `E Name<T> { Variant(T) }`
4. **트레이트 정의**: `W Trait { F method(&self) }`
5. **트레이트 구현**: `X Type: Trait { F method(&self) { ... } }`
6. **제네릭 바운드**: `<T: Trait>`

### 베스트 프랙티스

- ✅ 타입 추론 활용
- ✅ 재사용성 높은 코드에 제네릭 사용
- ✅ 트레이트로 공통 인터페이스 정의
- ✅ 표준 라이브러리 제네릭 타입 활용 (Vec, Option, Result)
- ❌ 과도한 제네릭화 피하기
- ❌ 불필요한 트레이트 바운드 피하기

### 다음 단계

- **고급 트레이트 패턴**
- **제네릭과 Async 결합**
- **커스텀 컬렉션 구현**
- **제네릭 라이브러리 설계**

---

## 참고 자료

- **기본 튜토리얼**: `TUTORIAL.md`
- **Async 튜토리얼**: `async_tutorial.md`
- **언어 스펙**: `LANGUAGE_SPEC.md`
- **표준 라이브러리**: `STDLIB.md`
- **예제 코드**:
  - `examples/generic_struct_test.vais`
  - `examples/generic_bounds_test.vais`
  - `examples/trait_test.vais`
  - `examples/option_test.vais`

---

Happy generic coding with Vais!
