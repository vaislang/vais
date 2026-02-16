# Vais 语言规范

版本: 1.0.0

Vais 编程语言完整参考。

## 目录

1. [概述](#概述)
2. [词法结构](#词法结构)
3. [关键字](#关键字)
4. [类型](#类型)
5. [运算符](#运算符)
6. [表达式](#表达式)
7. [语句](#语句)
8. [函数](#函数)
9. [结构体](#结构体)
10. [枚举](#枚举)
11. [错误处理](#错误处理)
12. [Trait 和实现](#trait-和实现)
13. [模式匹配](#模式匹配)
14. [泛型](#泛型)
15. [模块系统](#模块系统)
16. [Async/Await](#asyncawait)
17. [内存管理](#内存管理)
18. [内置函数](#内置函数)

---

## 概述

Vais 是一种令牌高效、AI 优化的系统编程语言，旨在最小化 AI 代码生成中的令牌使用，同时保持完整的系统编程能力。其特点:

- **单字符关键字** 以实现最大令牌效率
- **表达式导向语法**，一切都返回值
- **自递归运算符 `@`** 用于简洁的递归函数
- **基于 LLVM 的编译** 以实现原生性能
- **类型推导**，最少的标注
- **高级特性**: 泛型、Trait、Async/Await、模式匹配

---

## 词法结构

### 注释

注释以 `#` 开头，延续到行尾:

```vais
# 这是一个注释
F add(a:i64, b:i64)->i64 = a + b  # 行内注释
```

### 空白字符

空白字符 (空格、制表符、换行符) 被忽略，除非用于分隔令牌。

### 标识符

标识符以字母或下划线开头，后跟字母、数字或下划线:

```
[a-zA-Z_][a-zA-Z0-9_]*
```

示例: `x`, `my_var`, `Counter`, `_private`

### 字面量

**整数字面量:**
```vais
42
1_000_000    # 下划线提高可读性
-42          # 负数 (使用一元减运算符)
```

**浮点字面量:**
```vais
3.14
1.0e10
2.5e-3
1_000.5_00
```

**字符串字面量:**
```vais
"Hello, World!"
"Line with \"quotes\""
```

**字符串插值:**
```vais
name := "Vais"
println("Hello, ~{name}!")           # 变量插值
println("Result: ~{2 + 3}")          # 表达式插值
println("Escaped: {{not interp}}")  # 转义大括号
```

**布尔字面量:**
```vais
true
false
```

## 关键字

Vais 使用单字符关键字以实现最大令牌效率:

| 关键字 | 含义 | 用法 |
|---------|---------|-------|
| `F` | 函数 | 定义函数 |
| `S` | 结构体 | 定义结构体类型 |
| `E` | 枚举 (或 Else) | 定义枚举类型，或 if 中的 else 分支 |
| `I` | If | 条件表达式 |
| `L` | 循环 | 循环构造 |
| `M` | 匹配 | 模式匹配 |
| `W` | Trait (Where) | 定义 trait (接口) |
| `X` | 实现 (eXtend) | 实现方法或 trait |
| `T` | 类型 | 类型别名定义 |
| `U` | Use | 导入/使用模块 |
| `P` | Public | 公共可见性 |
| `A` | Async | 异步函数标记 |
| `R` | 返回 | 从函数提前返回 |
| `B` | Break | 跳出循环 |
| `C` | Continue/Const | 继续下一次循环迭代，或常量 |
| `D` | Defer | 延迟执行 |
| `N` | Extern | 外部函数声明 |
| `G` | Global | 全局变量声明 |
| `O` | Union | C 风格无标签联合 |
| `Y` | Yield/Await | 产出值 (await 的简写) |

注意: `C` 关键字具有双重含义 - 循环中的 continue 使用 `C`，常量也使用 `C`。上下文决定用法。

### 多字符关键字

- `mut` - 可变变量/引用
- `self` - 实例引用
- `Self` - impl 中的类型引用
- `true`, `false` - 布尔字面量
- `spawn` - 生成异步任务
- `await` - 等待异步结果 (也可用 `Y` 简写)
- `weak` - 弱引用
- `clone` - 克隆操作
- `yield` - 在迭代器/协程中产出值

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

**整数类型:**
- `i8`, `i16`, `i32`, `i64`, `i128` - 有符号整数
- `u8`, `u16`, `u32`, `u64`, `u128` - 无符号整数

**浮点类型:**
- `f32` - 32 位浮点数
- `f64` - 64 位浮点数

**其他类型:**
- `bool` - 布尔类型 (`true` 或 `false`)
- `str` - 字符串类型

### 指针类型

```vais
*i64        # 指向 i64 的指针
*T          # 指向类型 T 的指针
```

### 数组类型

```vais
[i64]       # i64 数组
[T]         # 类型 T 的数组
```

### 切片类型

切片是对数组子序列的引用:

```vais
&[T]        # 不可变切片
&mut [T]    # 可变切片

# 示例
arr := [1, 2, 3, 4, 5]
slice := &arr[1..4]      # [2, 3, 4]
mut_slice := &mut arr[..]  # 整个数组的可变切片
```

### 元组类型

```vais
(i64, f64)       # 两个元素的元组
(str, i64, bool) # 三个元素的元组
```

### 泛型类型

```vais
Option<T>   # 泛型 Option 类型
Vec<T>      # 泛型向量类型
Pair<A, B>  # 多个类型参数
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

## 表达式

### 二元表达式

```vais
a + b          # 加法
a - b          # 减法
a * b          # 乘法
a / b          # 除法
a % b          # 取模
a == b         # 相等
a != b         # 不等
a < b          # 小于
a > b          # 大于
a <= b         # 小于等于
a >= b         # 大于等于
a && b         # 逻辑与
a || b         # 逻辑或
a & b          # 位与
a | b          # 位或
a ^ b          # 位异或
a << b         # 左移
a >> b         # 右移
```

### 一元表达式

```vais
-x             # 负数
!x             # 逻辑非
~x             # 位非
```

### 函数调用

```vais
function_name(arg1, arg2)
obj.method(arg)
```

### 字段访问

```vais
struct_instance.field_name
```

### 索引

```vais
array[index]
array[start..end]    # 切片
```

### 三元表达式

```vais
condition ? value_if_true : value_if_false
```

### 元组表达式

```vais
(1, 2, 3)
(x, y) := get_pair()
(a, b, c) := (1, 2, 3)
```

### 块表达式

块是返回其最后一个表达式值的表达式:

```vais
{
    x := 10
    y := 20
    x + y    # 返回 30
}
```

### 自动返回

Vais 中的函数自动返回其最后一个表达式的值。除非需要提前返回，否则不需要显式的 `R` (return):

```vais
F add(a: i64, b: i64) -> i64 {
    a + b    # 自动返回
}

F max(a: i64, b: i64) -> i64 {
    I a > b {
        a    # 每个分支返回其最后一个表达式
    } E {
        b
    }
}

# 只有在需要提前返回时才需要显式 R
F safe_divide(a: i64, b: i64) -> i64 {
    I b == 0 {
        R 0    # 提前返回
    }
    a / b      # 自动返回
}
```

这适用于所有块表达式，包括 `I`/`E`、`M` 和 `L`。

---

## 语句

### 变量声明

```vais
# 类型推导 (不可变)
x := 10

# 显式类型
y: i64 = 20

# 可变
z := mut 30
```

### If-Else 表达式

```vais
# 单行三元
result := x > 0 ? 1 : -1

# 块形式
I x < 0 {
    -1
} E {
    0
}

# Else-if 链
I x < 0 {
    -1
} E I x == 0 {
    0
} E {
    1
}
```

注意: 在 if 表达式中 `E` 用于 "else"。

### 循环表达式

```vais
# 无限循环
L {
    # ... 主体
    B  # Break
}

# 范围循环
L i: 0..10 {
    puts("Iteration")
}

# 数组迭代 (概念性)
L item: array {
    # ... 处理 item
}
```

### Match 表达式

```vais
M value {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"    # 通配符/默认
}

# 带变量绑定
M option {
    Some(x) => x,
    None => 0
}
```

### Break 和 Continue

```vais
L i: 0..100 {
    I i == 50 { B }      # Break
    I i % 2 == 0 { C }   # Continue
    process(i)
}
```

### Return 语句

```vais
F compute(x: i64) -> i64 {
    I x < 0 {
        R 0    # 提前返回
    }
    x * 2
}
```

---

## 函数

### 函数定义

**表达式形式 (单表达式):**
```vais
F add(a:i64, b:i64)->i64 = a + b
```

**块形式:**
```vais
F factorial(n:i64)->i64 {
    I n < 2 {
        1
    } E {
        n * @(n-1)
    }
}
```

### 参数

```vais
F example(x: i64, y: f64, name: str) -> i64 { ... }
```

### 参数类型推导

当可以从调用点推导时，参数类型可以省略:

```vais
# 从使用中推导类型
F add(a, b) = a + b

# 混合: 一些显式，一些推导
F scale(x, factor: f64) -> f64 = x * factor

# 编译器从函数调用方式推导类型
F main() -> i64 {
    add(1, 2)       # 推导为 a: i64, b: i64
    scale(3.0, 2.0)  # 推导为 x: f64
    0
}
```

### 返回类型

```vais
F returns_int() -> i64 { 42 }
F returns_nothing() -> i64 { 0 }  # 约定: 0 表示 void
```

### 泛型函数

```vais
F identity<T>(x: T) -> T = x

F swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
}
```

### 自递归

```vais
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
F countdown(n:i64)->i64 = n<1 ? 0 : @(n-1)
```

### 外部函数

使用 `N F` 声明 C 函数:

```vais
N F puts(s: i64) -> i64
N F malloc(size: i64) -> i64
N F sqrt(x: f64) -> f64
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

## Trait 和实现

### 定义 Trait

Trait 定义类型必须实现的方法集合:

```vais
W Printable {
    F print(&self) -> i64
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}

W Shape {
    F area(&self) -> f64
    F perimeter(&self) -> f64
}
```

### 实现 Trait

```vais
S Circle {
    radius: f64
}

X Circle: Shape {
    F area(&self) -> f64 {
        pi := 3.14159
        pi * self.radius * self.radius
    }

    F perimeter(&self) -> f64 {
        pi := 3.14159
        2.0 * pi * self.radius
    }
}

F main()->i64 {
    c := Circle { radius: 5.0 }
    a := c.area()
    p := c.perimeter()

    puts("Circle area: ")
    print_f64(a)
    puts("Circle perimeter: ")
    print_f64(p)

    0
}
```

### 泛型 Trait

```vais
W Container<T> {
    F get(&self) -> T
    F set(&self, value: T) -> i64
}

S Box<T> {
    value: T
}

X Box<T>: Container<T> {
    F get(&self) -> T {
        self.value
    }

    F set(&self, value: T) -> i64 {
        self.value = value
        0
    }
}
```

### 实现块 (无 Trait)

也可以在不实现 trait 的情况下向类型添加方法:

```vais
S Point {
    x: f64,
    y: f64
}

X Point {
    F distance(&self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }

    F translate(&self, dx: f64, dy: f64) -> Point {
        Point { x: self.x + dx, y: self.y + dy }
    }

    F origin() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}

F main()->i64 {
    p := Point { x: 3.0, y: 4.0 }
    d := p.distance()
    p2 := p.translate(1.0, 1.0)

    origin := Point::origin()

    print_f64(d)
    0
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

Vais 使用 `Result` 和 `Option` 类型进行显式错误处理。

### Option 类型

表示可能不存在的值:

```vais
E Option<T> {
    None,
    Some(T)
}

F find(arr: [i64], target: i64) -> Option<i64> {
    L i: 0..arr.len() {
        I arr[i] == target {
            R Some(i)
        }
    }
    None
}

F main()->i64 {
    arr := [1, 2, 3, 4, 5]
    result := find(arr, 3)

    M result {
        Some(index) => {
            puts("Found at index: ")
            print_i64(index)
        },
        None => puts("Not found")
    }
    0
}
```

### Result 类型

表示可能失败的操作:

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 {
        R Err("division by zero")
    }
    Ok(a / b)
}

F main()->i64 {
    result := divide(10, 2)

    M result {
        Ok(value) => {
            puts("Result: ")
            print_i64(value)
        },
        Err(msg) => {
            puts("Error: ")
            puts(msg)
        }
    }
    0
}
```

### Try 运算符 `?`

`?` 运算符传播错误:

```vais
F compute() -> Result<i64, str> {
    a := divide(10, 2)?   # 如果是 Err 则提前返回
    b := divide(a, 3)?    # 继续链式调用
    Ok(b)
}

# 等价于:
F compute_verbose() -> Result<i64, str> {
    a_result := divide(10, 2)
    a := M a_result {
        Ok(val) => val,
        Err(e) => { R Err(e) }
    }

    b_result := divide(a, 3)
    b := M b_result {
        Ok(val) => val,
        Err(e) => { R Err(e) }
    }

    Ok(b)
}
```

### Unwrap 运算符 `!`

`!` 运算符在 Err/None 时 panic:

```vais
F main()->i64 {
    result := divide(10, 2)
    value := result!  # 如果是 Err 则 panic

    opt := Some(42)
    val := opt!       # 如果是 None 则 panic

    print_i64(value)
    0
}
```

**注意:** 仅在确定值必定存在时使用 `!`。在生产代码中，优先使用模式匹配或 `?`。

## 模块系统

### 导入模块

使用 `U` (Use) 关键字导入模块:

```vais
U std/io        # 导入 I/O 模块
U std/vec       # 导入 Vec 模块
U std/hashmap   # 导入 HashMap 模块
```

### 使用导入的项

```vais
U std/vec

F main()->i64 {
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)

    first := v.get(0)
    print_i64(first)

    0
}
```

### 模块路径

模块使用 `/` 分隔的路径:

```vais
U std/collections/vec
U std/io/file
U std/net/tcp
```

### 项目结构

```
project/
├── main.vais           # 主文件
├── utils.vais          # 实用工具模块
└── math/
    ├── vector.vais     # 向量模块
    └── matrix.vais     # 矩阵模块
```

在 `main.vais` 中:

```vais
U utils
U math/vector

F main()->i64 {
    # 使用导入的模块
    0
}
```

### 可见性

使用 `P` (Public) 关键字导出项:

```vais
# 在 utils.vais 中
P F public_function()->i64 {
    42
}

F private_function()->i64 {
    0
}
```

只有 `public_function` 可以从其他模块访问。

## Async/Await

Vais 支持使用 `A` (Async) 和 `Y` (Await/Yield) 关键字的异步编程。

### 异步函数

```vais
A F fetch_data() -> str {
    # 异步操作
    "data"
}

A F process() -> i64 {
    data := Y fetch_data()  # 等待结果
    puts(data)
    0
}
```

### Spawn 任务

```vais
A F background_task() -> i64 {
    puts("Running in background")
    0
}

F main()->i64 {
    # 生成异步任务
    task := spawn background_task()

    # 等待完成
    result := Y task

    0
}
```

### Future 组合器

```vais
U std/async

A F fetch_user(id: i64) -> User {
    # 获取用户数据
}

A F fetch_posts(user_id: i64) -> [Post] {
    # 获取帖子
}

A F get_user_with_posts(id: i64) -> (User, [Post]) {
    user := Y fetch_user(id)
    posts := Y fetch_posts(user.id)
    (user, posts)
}
```

## 闭包和 Lambda

### Lambda 表达式

```vais
# 基本 lambda
add := |a, b| a + b

# 带块体
compute := |x| {
    result := x * 2
    result + 1
}

# 使用
sum := add(1, 2)
value := compute(5)
```

### 捕获变量

```vais
F make_adder(n: i64) -> |i64| -> i64 {
    |x| x + n  # 捕获 n
}

F main()->i64 {
    add_5 := make_adder(5)
    result := add_5(10)  # 返回 15

    print_i64(result)
    0
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
