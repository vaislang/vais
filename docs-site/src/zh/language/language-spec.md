# 语言规范

Vais 编程语言完整参考。

## 概述

Vais 是一种为以下目标设计的系统编程语言:

- **令牌效率** - 单字符关键字最小化 AI 令牌使用
- **类型安全** - 具有完整推导的强静态类型
- **原生性能** - 基于 LLVM 的原生代码编译
- **现代特性** - 泛型、trait、async/await、模式匹配

## 关键字

Vais 使用单字符关键字以实现最大效率:

| 关键字 | 含义 | 示例 |
|---------|---------|---------|
| `F` | 函数 | `F add(a: i64, b: i64) -> i64 { a + b }` |
| `S` | 结构体 | `S Point { x: f64, y: f64 }` |
| `E` | 枚举/Else | `E Color { Red, Green, Blue }` / `E { fallback }` |
| `I` | If | `I x > 0 { "positive" }` |
| `L` | 循环 | `L i := 0; i < 10; i += 1 { ... }` |
| `M` | 匹配 | `M x { 1 => "one", _ => "other" }` |
| `R` | 返回 | `R 42` |
| `B` | Break | `B` |
| `C` | Continue | `C` |
| `W` | Trait | `W Printable { F print(self) }` |
| `X` | 实现 | `X Point: Printable { ... }` |
| `U` | Use/导入 | `U std/io` |
| `P` | Public | `P F public_fn() {}` |
| `T` | 类型别名 | `T Int = i64` |
| `A` | Async | `A F fetch() -> str { ... }` |
| `Y` | Await | `result := Y fetch()` |
| `N` | Extern | `N F malloc(size: i64) -> i64` |
| `G` | Global | `G counter: i64 = 0` |
| `D` | Defer | `D cleanup()` |
| `O` | Union | `O Data { i: i64, f: f64 }` |

## 运算符

### 特殊运算符

| 运算符 | 含义 | 示例 |
|----------|---------|---------|
| `@` | 自递归 | `@(n-1) + @(n-2)` |
| `:=` | 变量绑定 | `x := 5` |
| `:= mut` | 可变绑定 | `x := mut 0` |
| `?` | Try(错误传播) | `result?` |
| `!` | Unwrap | `result!` |
| `\|>` | 管道 | `x \|> f \|> g` |
| `..` | 范围 | `1..10` |

### 算术运算符

```vais
+ - * / %        # 基本算术
+= -= *= /= %=   # 复合赋值
```

### 比较运算符

```vais
== != < > <= >=
```

### 逻辑运算符

```vais
&& ||  # 逻辑 AND、OR
!      # 逻辑 NOT
```

### 位运算符

```vais
& | ^ << >>      # AND、OR、XOR、左移、右移
```

## 类型

### 原始类型

```vais
# 整数
i8 i16 i32 i64 i128
u8 u16 u32 u64 u128

# 浮点数
f32 f64

# 布尔
bool

# 字符串
str
```

### 复合类型

```vais
# 数组
[i64; 10]         # 10 个 i64 的固定大小数组

# 切片
&[i64]            # 不可变切片
&mut [i64]        # 可变切片

# 元组
(i64, f64, str)

# 指针
*i64              # 原始指针
```

### 泛型类型

```vais
Vec<T>            # 泛型向量
HashMap<K, V>     # 泛型哈希映射
Option<T>         # 可选值
Result<T, E>      # 带错误的结果
```

## 变量声明

```vais
# 类型推导
x := 42                 # 推导为 i64
y := 3.14               # 推导为 f64

# 显式类型
count: i64 = 100

# 可变
counter := mut 0
counter = counter + 1

# 多个声明
a := 1
b := 2
c := 3
```

## 函数

### 基本函数

```vais
F add(a: i64, b: i64) -> i64 {
    a + b
}
```

### 无返回值

```vais
F greet(name: str) {
    puts("Hello, ")
    puts(name)
}
```

### 自递归

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}
```

### 泛型函数

```vais
F identity<T>(x: T) -> T {
    x
}

F max<T>(a: T, b: T) -> T {
    I a > b { a } E { b }
}
```

## 控制流

### if 表达式

```vais
# 简单 if
I x > 0 {
    puts("positive")
}

# if-else
I x > 0 {
    puts("positive")
} E {
    puts("negative or zero")
}

# if 作为表达式
sign := I x > 0 { 1 } E I x < 0 { -1 } E { 0 }
```

### match 表达式

```vais
M x {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "many"
}
```

### 循环

```vais
# C 风格循环
L i := 0; i < 10; i += 1 {
    print_i64(i)
}

# 无限循环
L {
    I should_break { B }
}

# 带 break 和 continue 的循环
L i := 0; i < 20; i += 1 {
    I i % 2 == 0 { C }  # 跳过偶数
    I i > 15 { B }      # 在 15 处中断
    print_i64(i)
}
```

## 结构体

```vais
# 定义结构体
S Point {
    x: f64,
    y: f64
}

# 创建实例
p := Point { x: 3.0, y: 4.0 }

# 访问字段
x_coord := p.x
```

### 方法

```vais
X Point {
    F distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }

    F translate(self, dx: f64, dy: f64) -> Point {
        Point { x: self.x + dx, y: self.y + dy }
    }
}
```

## 枚举

```vais
# 简单枚举
E Color {
    Red,
    Green,
    Blue
}

# 带数据的枚举
E Option<T> {
    Some(T),
    None
}

E Result<T, E> {
    Ok(T),
    Err(E)
}
```

## 模式匹配

```vais
E Option<T> { Some(T), None }

F unwrap_or<T>(opt: Option<T>, default: T) -> T {
    M opt {
        Some(v) => v,
        None => default
    }
}
```

## Trait

```vais
# 定义 trait
W Printable {
    F print(self)
}

# 实现 trait
S Person { name: str, age: i64 }

X Person: Printable {
    F print(self) {
        puts(self.name)
    }
}
```

## 泛型

```vais
# 泛型结构体
S Box<T> {
    value: T
}

# 泛型函数
F swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

# 泛型 trait
W Container<T> {
    F get(self) -> T
}
```

## 错误处理

### Option 类型

```vais
E Option<T> { Some(T), None }

F find(arr: &[i64], target: i64) -> Option<i64> {
    # ... 搜索逻辑
    Some(index)  # 或 None
}
```

### Result 类型

```vais
E Result<T, E> { Ok(T), Err(E) }

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("division by zero") }
    Ok(a / b)
}
```

### Try 运算符 `?`

```vais
F compute() -> Result<i64, str> {
    a := divide(10, 2)?   # 传播错误
    b := divide(a, 3)?
    Ok(b)
}
```

### Unwrap 运算符 `!`

```vais
result := divide(10, 2)
value := result!  # Err 时 panic
```

## 模块系统

```vais
# 导入模块
U std/io
U std/vec

# 从模块使用项
v := Vec::new()
content := read_file("data.txt")
```

## 注释

```vais
# 单行注释

F main() {
    x := 42  # 行内注释
}
```

## 字符串插值

```vais
name := "Alice"
age := 30

# 变量插值(不支持 - 使用 puts)
puts("Name: ")
puts(name)

# 连接
msg := "Hello, " + name
```

## 内置函数

```vais
# I/O
puts(s: str)              # 打印字符串
print_i64(x: i64)         # 打印整数
print_f64(x: f64)         # 打印浮点数

# 内存
malloc(size: i64) -> i64  # 分配内存
free(ptr: i64)            # 释放内存

# 类型操作
sizeof(T) -> i64          # 类型大小
```

## 最佳实践

1. **使用类型推导** 当类型显而易见时
2. **使用显式类型** 用于函数参数和返回值
3. **优先使用表达式** 而非语句(使用 `I` 而非 if 语句)
4. **使用 `@` 进行递归** 而非函数名
5. **处理错误** 使用 `Result` 和 `?` 运算符
6. **使用模式匹配** 使用 `M` 处理复杂条件
7. **保持函数简洁** 专注于单一职责

## 示例

### 斐波那契

```vais
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}
```

### 链表

```vais
S Node<T> {
    value: T,
    next: Option<Box<Node<T>>>
}

X Node<T> {
    F new(value: T) -> Node<T> {
        Node { value: value, next: None }
    }
}
```

### 错误处理

```vais
F parse_number(s: str) -> Result<i64, str> {
    # 解析逻辑
    I is_valid {
        Ok(number)
    } E {
        Err("Invalid number")
    }
}
```

## 了解更多

- [教程](../getting-started/tutorial.md) - 逐步指南
- [标准库](https://github.com/vaislang/vais/tree/main/std) - 内置模块
- [示例](https://github.com/vaislang/vais/tree/main/examples) - 代码示例
