# Vais 编程语言

**Vais** 是一种为最大化令牌效率和开发者生产力而设计的 AI 优化系统编程语言。

## 主要特性

- **单字符关键字**: 函数用 `F`，结构体用 `S`，条件用 `I`，模式匹配用 `M`
- **表达式导向**: 一切皆返回值
- **自递归运算符**: 用 `@` 编写简洁的递归函数
- **高级类型系统**: 完整的类型推导、泛型、trait
- **LLVM 后端**: 通过优化的代码生成实现原生性能
- **现代特性**: async/await、模式匹配、用 `?` 和 `!` 进行错误处理

## 为什么选择 Vais?

Vais 在最小化 AI 生成代码的令牌使用量的同时，保持了完整的系统编程能力:

- **比 Rust/C++ 少 50-70% 的令牌**
- **自托管编译器** 超过 50,000 行 Vais 代码
- **774K 行/秒** 的编译速度
- **2,500+ 个测试** 覆盖所有组件

## 示例

```vais
# 使用自递归的斐波那契
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}

F main() {
    puts("fib(10) = ")
    print_i64(fib(10))
}
```

## 入门指南

- 在您的系统上[安装 Vais](./getting-started/installation.md)
- 遵循[快速开始](./getting-started/quick-start.md)指南
- 通过[教程](./getting-started/tutorial.md)学习
- 阅读[语言规范](./language/language-spec.md)

## 社区与支持

- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
- **Issues**: 报告 bug 和功能请求
- **文档**: 浏览完整文档

## 许可证

Vais 是 MIT 许可的开源软件。
