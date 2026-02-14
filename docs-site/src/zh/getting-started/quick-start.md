# 快速开始

几分钟内开始使用 Vais。

## 安装

```bash
# macOS / Linux (Homebrew)
brew tap vaislang/tap && brew install vais

# 或通过 Cargo
cargo install vaisc
```

> 要从源代码构建，请参阅[安装指南](./installation.md)。

## 第一个程序

创建名为 `hello.vais` 的文件:

```vais
F main() {
    puts("Hello, Vais!")
}
```

## 编译和运行

```bash
# 编译
vaisc build hello.vais -o hello
./hello

# 或直接运行
vaisc run hello.vais
```

**输出:**
```
Hello, Vais!
```

## 基本语法

### 变量

```vais
F main() {
    x := 42              # 推导为 i64
    y := 3.14            # 推导为 f64
    name := "Alice"      # 推导为 str
    flag := true         # 推导为 bool

    puts("Variables declared!")
}
```

### 函数

```vais
F add(a: i64, b: i64) -> i64 {
    a + b  # 最后一个表达式是返回值
}

F main() {
    result := add(10, 20)
    print_i64(result)  # 输出: 30
}
```

### 控制流

```vais
F main() {
    x := 10

    # if 表达式
    msg := I x > 5 { "big" } E { "small" }
    puts(msg)

    # 循环
    L i := 0; i < 5; i += 1 {
        print_i64(i)
    }
}
```

### 自递归

使用 `@` 调用当前函数:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}

F main() {
    print_i64(factorial(5))  # 输出: 120
}
```

## 下一步

- [教程](./tutorial.md) - 深入学习 Vais
- [语言规范](../language/language-spec.md) - 完整的语法参考
- [示例程序](https://github.com/vaislang/vais/tree/main/examples) - 浏览代码示例
