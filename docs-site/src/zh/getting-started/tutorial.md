# 教程

本教程将引导您学习 Vais 编程，从基本概念到高级特性。

## Hello World

让我们从经典的第一个程序开始:

```vais
F main() {
    puts("Hello, Vais!")
}
```

**要点:**
- `F` 声明函数
- `main` 是入口点
- `puts` 将字符串打印到标准输出

## 变量和类型

### 类型推导

使用 `:=` 进行自动类型推导:

```vais
F main() {
    x := 10          # 推导为 i64
    y := 3.14        # 推导为 f64
    name := "Alice"  # 推导为 str
    flag := true     # 推导为 bool
}
```

### 显式类型

在需要时指定类型:

```vais
F main() {
    count: i64 = 100
    price: f64 = 19.99
    is_active: bool = true
}
```

### 可变变量

对可重新赋值的变量使用 `mut`:

```vais
F main() {
    x := mut 0
    x = 10  # OK: x 是可变的
    x = 20  # OK
}
```

## 函数

### 基本函数

```vais
F add(a: i64, b: i64) -> i64 {
    a + b  # 最后一个表达式是返回值
}

F greet(name: str) {
    puts("Hello, ")
    puts(name)
}
```

### 提前返回

使用 `R` 提前返回:

```vais
F abs(x: i64) -> i64 {
    I x < 0 { R -x }
    x
}
```

### 自递归

使用 `@` 调用当前函数:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}

F fibonacci(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}
```

## 控制流

### if-else 表达式

**三元形式** (单表达式):

```vais
F abs(x:i64)->i64 = x < 0 ? -x : x

F sign(x:i64)->i64 = x < 0 ? -1 : x > 0 ? 1 : 0
```

**块形式:**

```vais
F classify(x:i64)->str {
    I x < 0 {
        "negative"
    } E I x == 0 {
        "zero"
    } E {
        "positive"
    }
}
```

注意: `E` 用于 "else"。上下文决定 `E` 表示 "enum" 还是 "else"。

### 循环

**无限循环:**

```vais
F loop_forever()->i64 {
    L {
        puts("Looping...")
        # 需要 break 条件
    }
    0
}
```

**范围循环:**

```vais
F count_to_ten()->i64 {
    L i: 0..10 {
        puts("Number: ")
        print_i64(i)
        putchar(10)
    }
    0
}
```

**使用 break 和 continue:**

```vais
F find_first_even()->i64 {
    L i: 0..100 {
        I i % 2 == 0 {
            puts("Found even number:")
            print_i64(i)
            B  # Break
        }
        C  # Continue
    }
    0
}
```

### 提前返回

```vais
F validate(x: i64)->i64 {
    I x < 0 {
        puts("Error: negative value")
        R -1  # 提前返回
    }
    I x == 0 {
        puts("Error: zero value")
        R -1
    }

    # 处理有效值
    puts("Valid!")
    x * 2
}
```

## 结构体和枚举

### 定义结构体

```vais
S Point {
    x: f64,
    y: f64
}

S Person {
    name: str,
    age: i64
}

S Rectangle {
    top_left: Point,
    bottom_right: Point
}
```

### 创建结构体实例

```vais
F main()->i64 {
    # 创建一个 Point
    p := Point { x: 10.0, y: 20.0 }

    # 创建一个 Person
    person := Person { name: "Bob", age: 25 }

    # 嵌套结构体
    rect := Rectangle {
        top_left: Point { x: 0.0, y: 10.0 },
        bottom_right: Point { x: 10.0, y: 0.0 }
    }

    0
}
```

### 访问字段

```vais
F main()->i64 {
    p := Point { x: 5.0, y: 15.0 }

    x_coord := p.x
    y_coord := p.y

    puts("Point coordinates:")
    print_f64(x_coord)
    print_f64(y_coord)

    0
}
```

### 定义枚举

**简单枚举:**

```vais
E Color {
    Red,
    Green,
    Blue
}
```

**带数据的枚举:**

```vais
E Option {
    None,
    Some(i64)
}

E Result {
    Ok(i64),
    Err(str)
}

E Message {
    Quit,
    Move(i64, i64),
    Write(str)
}
```

### 使用枚举

```vais
F main()->i64 {
    color := Red
    opt := Some(42)
    result := Ok(100)
    msg := Move(10, 20)

    puts("Enums created!")
    0
}
```

## 模式匹配

使用 `M` (match) 进行模式匹配对于处理枚举和值非常强大。

### 基本匹配

```vais
F describe_number(n: i64)->str {
    M n {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => "other"  # 通配符: 匹配其他所有情况
    }
}
```

### 带绑定的匹配

从匹配的模式中提取值:

```vais
E Option {
    None,
    Some(i64)
}

F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(x) => x,        # 将值绑定到 'x'
        None => default
    }
}

F main()->i64 {
    opt1 := Some(42)
    opt2 := None

    v1 := unwrap_or(opt1, 0)  # 返回 42
    v2 := unwrap_or(opt2, 99) # 返回 99

    print_i64(v1)
    print_i64(v2)
    0
}
```

### 匹配 Result 类型

```vais
E Result {
    Ok(i64),
    Err(str)
}

F handle_result(res: Result) -> i64 {
    M res {
        Ok(value) => value,
        Err(msg) => {
            puts("Error: ")
            puts(msg)
            0
        }
    }
}
```

### 完整示例

```vais
E Color {
    Red,
    Green,
    Blue
}

F color_to_code(c: Color) -> i64 {
    M c {
        Red => 0xFF0000,
        Green => 0x00FF00,
        Blue => 0x0000FF
    }
}

F main()->i64 {
    red_code := color_to_code(Red)
    green_code := color_to_code(Green)

    puts("Color codes calculated!")
    0
}
```

## 错误处理

### Result 和 Option 类型

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("Division by zero") }
    Ok(a / b)
}
```

### Try 运算符

使用 `?` 传播错误:

```vais
F compute() -> Result<i64, str> {
    x := divide(10, 2)?  # 如果是 Err 则传播
    y := divide(x, 0)?   # 这里将返回 Err
    Ok(y)
}
```

### Unwrap 运算符

使用 `!` 解包或 panic:

```vais
F main() {
    result := divide(10, 2)
    value := result!  # 解包 Ok 值，Err 时 panic
    print_i64(value)
}
```

## 泛型

泛型允许您编写适用于多种类型的代码。

### 泛型函数

```vais
F identity<T>(x: T) -> T = x

F first<T>(a: T, b: T) -> T = a

F swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
}
```

### 泛型结构体

```vais
S Pair<T> {
    first: T,
    second: T
}

S Box<T> {
    value: T
}

S Container<K, V> {
    key: K,
    value: V
}
```

### 使用泛型结构体

```vais
F main()->i64 {
    # 整数的 Pair
    int_pair := Pair { first: 10, second: 20 }

    # 浮点数的 Pair
    float_pair := Pair { first: 1.5, second: 2.5 }

    # 不同类型的 Container
    container := Container { key: 1, value: "hello" }

    0
}
```

### 泛型类型的方法

```vais
S Pair<T> {
    first: T,
    second: T
}

X Pair {
    F sum(&self) -> i64 {
        self.first + self.second
    }

    F swap(&self) -> Pair {
        Pair { first: self.second, second: self.first }
    }
}

F main()->i64 {
    p := Pair { first: 10, second: 20 }
    total := p.sum()
    swapped := p.swap()

    print_i64(total)  # 30
    0
}
```

### 泛型枚举

```vais
E Option<T> {
    None,
    Some(T)
}

E Result<T, E> {
    Ok(T),
    Err(E)
}

F main()->i64 {
    # i64 的 Option
    opt_int := Some(42)

    # str 的 Option
    opt_str := Some("hello")

    # 带 i64 值和 str 错误的 Result
    result := Ok(100)

    0
}
```

## Trait 和方法

### 定义 Trait

Trait 定义类型可以实现的接口:

```vais
W Printable {
    F print(&self) -> i64
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}
```

### 实现 Trait

```vais
S Counter {
    value: i64
}

# 为 Counter 实现 Printable trait
X Counter: Printable {
    F print(&self) -> i64 {
        puts("Counter value: ")
        print_i64(self.value)
        putchar(10)
        0
    }
}
```

### 添加方法

使用 `X` 添加方法，无需 trait:

```vais
X Counter {
    F increment(&self) -> i64 {
        self.value + 1
    }

    F double(&self) -> i64 {
        self.value * 2
    }

    F reset() -> Counter {
        Counter { value: 0 }
    }
}
```

### 使用方法

```vais
F main()->i64 {
    c := Counter { value: 10 }

    # 调用 trait 方法
    c.print()

    # 调用 impl 方法
    inc := c.increment()
    dbl := c.double()

    puts("Incremented: ")
    print_i64(inc)
    puts("Doubled: ")
    print_i64(dbl)

    0
}
```

### 完整示例

```vais
W Shape {
    F area(&self) -> f64
}

S Circle {
    radius: f64
}

S Rectangle {
    width: f64,
    height: f64
}

X Circle: Shape {
    F area(&self) -> f64 {
        pi := 3.14159
        pi * self.radius * self.radius
    }
}

X Rectangle: Shape {
    F area(&self) -> f64 {
        self.width * self.height
    }
}

F main()->i64 {
    circle := Circle { radius: 5.0 }
    rect := Rectangle { width: 4.0, height: 6.0 }

    circle_area := circle.area()
    rect_area := rect.area()

    puts("Circle area: ")
    print_f64(circle_area)

    puts("Rectangle area: ")
    print_f64(rect_area)

    0
}
```

## 标准库基础

### 使用数学库

```vais
U std/math

F main()->i64 {
    # 常量
    pi := PI
    e := E

    # 基本数学
    x := abs(-10.0)          # 绝对值
    min_val := min(5.0, 10.0)
    max_val := max(5.0, 10.0)

    # 高级数学
    root := sqrt(16.0)       # 平方根: 4.0
    power := pow(2.0, 8.0)   # 2^8 = 256.0

    # 三角函数
    sine := sin(PI / 2.0)    # sin(90°) = 1.0
    cosine := cos(0.0)       # cos(0°) = 1.0

    # 对数
    natural_log := log(E)    # ln(e) = 1.0
    log_base_10 := log10(100.0)  # 2.0

    print_f64(root)
    0
}
```

### 使用 I/O 库

```vais
U std/io

F main()->i64 {
    # 打印到标准输出
    puts("Hello, World!")
    println("With newline!")

    # 读取输入
    line := read_line()
    puts("You entered: ")
    puts(line)

    0
}
```

### 集合

```vais
U std/vec
U std/hashmap

F main()->i64 {
    # Vector
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)

    # 访问元素
    first := v.get(0)
    puts("First element:")
    print_i64(first)

    # HashMap
    m := HashMap::new()
    m.insert("name", "Alice")
    m.insert("city", "Paris")

    # 查找值
    name := m.get("name")
    puts(name)

    0
}
```

### 文件 I/O

```vais
U std/fs

F main()->i64 {
    # 读取文件
    content := read_file("data.txt")
    puts(content)

    # 写入文件
    write_file("output.txt", "Hello, file!")

    # 追加到文件
    append_file("log.txt", "New log entry\n")

    0
}
```

## 下一步

您现在已经了解了 Vais 的基础！继续学习:

- [语言规范](../language/language-spec.md) - 完整的语法参考
- [标准库](https://github.com/vaislang/vais/tree/main/std) - 探索内置模块
- [示例](https://github.com/vaislang/vais/tree/main/examples) - 真实世界的代码示例
