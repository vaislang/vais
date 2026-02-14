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

### if 表达式

Vais 中一切都是表达式:

```vais
F main() {
    x := 10

    # if 返回一个值
    result := I x > 5 { "big" } E { "small" }
    puts(result)  # 输出: big
}
```

### 循环

```vais
F main() {
    # C 风格循环
    L i := 0; i < 10; i += 1 {
        print_i64(i)
    }

    # 带 break 的无限循环
    counter := mut 0
    L {
        counter = counter + 1
        I counter >= 5 { B }
    }
}
```

## 结构体

定义自定义数据类型:

```vais
S Point {
    x: f64,
    y: f64
}

F main() {
    p := Point { x: 3.0, y: 4.0 }
    puts("Point created")
}
```

### 方法

为结构体实现方法:

```vais
S Point { x: f64, y: f64 }

X Point {
    F distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

F main() {
    p := Point { x: 3.0, y: 4.0 }
    d := p.distance()
    print_f64(d)  # 输出: 5.0
}
```

## 枚举

定义变体类型:

```vais
E Color {
    Red,
    Green,
    Blue
}

E Option<T> {
    Some(T),
    None
}
```

## 模式匹配

使用 `M` 匹配模式:

```vais
E Color { Red, Green, Blue }

F color_name(c: Color) -> str {
    M c {
        Red => "red",
        Green => "green",
        Blue => "blue"
    }
}

F main() {
    c := Red
    puts(color_name(c))  # 输出: red
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

编写适用于任意类型的代码:

```vais
F identity<T>(x: T) -> T {
    x
}

S Box<T> {
    value: T
}

F main() {
    x := identity(42)      # T = i64
    y := identity(3.14)    # T = f64

    b := Box { value: 100 }
}
```

## Trait

定义共享行为:

```vais
W Printable {
    F print(self)
}

S Point { x: f64, y: f64 }

X Point: Printable {
    F print(self) {
        puts("Point(")
        print_f64(self.x)
        puts(", ")
        print_f64(self.y)
        puts(")")
    }
}
```

## 标准库

### 集合

```vais
U std/vec
U std/hashmap

F main() {
    # Vector
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)

    # HashMap
    m := HashMap::new()
    m.insert("name", "Alice")
    m.insert("city", "Paris")
}
```

### 文件 I/O

```vais
U std/io

F main() {
    # 读取文件
    content := read_file("data.txt")
    puts(content)

    # 写入文件
    write_file("output.txt", "Hello, file!")
}
```

## 下一步

您现在已经了解了 Vais 的基础！继续学习:

- [语言规范](../language/language-spec.md) - 完整的语法参考
- [标准库](https://github.com/vaislang/vais/tree/main/std) - 探索内置模块
- [示例](https://github.com/vaislang/vais/tree/main/examples) - 真实世界的代码示例
