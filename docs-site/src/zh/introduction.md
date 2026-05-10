# Vais 编程语言

**Vais (Vibe AI Language for Systems)** 是一种为最大化令牌效率和开发者生产力而设计的 AI 优化系统编程语言。

[![CI](https://github.com/vaislang/vais/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/vaislang/vais/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/vaislang/vais/branch/main/graph/badge.svg)](https://codecov.io/gh/vaislang/vais)

Vais 旨在最小化令牌使用量，同时最大化代码表达性，使其成为 AI 辅助开发和 LLM 代码生成的理想选择。

> **当前公开状态:** Vais 当前公开基线是 certified Core compiler 加上明确命名的 promoted runtime gates，不是 product-complete v1.0 release。公开 claim 边界见 [`PUBLIC_STATUS.md`](https://github.com/vaislang/vais/blob/main/PUBLIC_STATUS.md)。

## 主要特性

- **单字符关键字** - `F`(函数)、`S`(结构体)、`E`(枚举/else)、`I`(if)、`L`(循环)、`M`(匹配)
- **自递归运算符** `@` - 递归调用当前函数
- **表达式导向** - 一切皆为表达式
- **LLVM 后端** - 基于 LLVM 17 的 promoted native codegen path
- **类型推导** - certified Core surface 上的最少标注；更广泛的推导功能仍在 hardening
- **内存安全** - 借用检查器，支持非词法生命周期(NLL)，`--strict-borrow` 模式
- **切片类型** - `&[T]` / `&mut [T]`，基于胖指针实现
- **并行编译** - 基于 DAG 的类型检查和代码生成 workbench
- **自托管 workbench** - 50,000+ 行 Vais compiler source，用于 bootstrap 和 conformance work
- **生态系统 workbench** - std 和 downstream package 通过明确 gate 跟踪

## 快速示例

```vais
# 使用自递归的斐波那契
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# 结构体定义
S Point { x:f64, y:f64 }

# 使用循环的求和
F sum(arr:[i64])->i64 {
    s := 0
    L x:arr { s += x }
    s
}
```

## 语法概览

| 关键字 | 含义 | 示例 |
|---------|---------|---------|
| `F` | 函数 | `F add(a:i64,b:i64)->i64=a+b` |
| `S` | 结构体 | `S Point{x:f64,y:f64}` |
| `E` | 枚举/Else | `E Option<T>{Some(T),None}` |
| `I` | If | `I x>0{1}E{-1}` |
| `L` | 循环 | `L i:0..10{print(i)}` |
| `M` | 匹配 | `M opt{Some(v)=>v,None=>0}` |
| `@` | 自调用 | `@(n-1)` (递归调用) |
| `:=` | 推导并赋值 | `x := 42` |

## 项目结构

```
crates/
├── vais-ast/          # AST 定义
├── vais-lexer/        # 词法分析器 (基于 logos)
├── vais-parser/       # 递归下降解析器
├── vais-types/        # 类型检查器与推导
├── vais-codegen/      # LLVM IR 代码生成器 (inkwell/, advanced_opt/)
├── vais-codegen-js/   # JavaScript (ESM) 代码生成器
├── vais-mir/          # 中间表示 (MIR)
├── vaisc/             # 主编译器 CLI 和 REPL
├── vais-lsp/          # 语言服务器协议
├── vais-dap/          # 调试适配器协议
├── vais-jit/          # Cranelift JIT 编译器
├── vais-gc/           # 可选垃圾收集器
├── vais-gpu/          # GPU 代码生成 (CUDA/Metal/OpenCL/WebGPU)
├── vais-i18n/         # 国际化错误消息
├── vais-plugin/       # 插件系统
├── vais-macro/        # 声明式宏系统
├── vais-hotreload/    # 热重载
├── vais-dynload/      # 动态模块加载与 WASM 沙箱
├── vais-bindgen/      # FFI 绑定生成器 (C/WASM-JS)
├── vais-query/        # Salsa 风格查询数据库
├── vais-profiler/     # 编译器性能分析器
├── vais-security/     # 安全分析与审计
├── vais-supply-chain/ # SBOM 与依赖审计
├── vais-testgen/      # 基于属性的测试生成
├── vais-tutorial/     # 交互式教程
├── vais-registry-server/    # 包注册中心 (Axum/SQLite)
├── vais-playground-server/  # Web playground 后端
├── vais-python/       # Python 绑定 (PyO3)
└── vais-node/         # Node.js 绑定 (NAPI)

std/               # 标准库 (74 个模块)
selfhost/          # 自托管编译器 (51,190 行，58 个 .vais 文件)
vscode-vais/       # VSCode 扩展
intellij-vais/     # IntelliJ 插件
docs-site/         # mdBook 文档
examples/          # 示例程序 (189 个文件)
benches/           # 基准测试套件 (criterion + 语言比较)
playground/        # Web playground 前端
```

## 编译管道

```
.vais 源码 → 词法分析 → 解析 → AST → 类型检查 → 代码生成 → .ll (LLVM IR) → clang → 二进制文件
                                                    ↘ JS 代码生成 → .mjs (ESM)
                                                    ↘ WASM 代码生成 → .wasm (experimental unless gated)
```

## 为什么选择 Vais?

Vais 在最小化 AI 生成代码的令牌使用量的同时，保持了完整的系统编程能力:

- **比 Rust/C++ 少 50-70% 的令牌**
- **自托管 workbench** — 50,000+ 行 Vais compiler source，用于 bootstrap 和 conformance work
- **800K 行/秒** 的编译速度 (50K 行 → 63ms)
- **Gate-backed status** — 当前保证以 `PUBLIC_STATUS.md` 中的 certified Core 和 promoted runtime gates 为准

### 编译速度

| 阶段 | 时间 (平均) | 吞吐量 | 备注 |
|-------|------------|------------|-------|
| 词法分析 | ~0.5ms/1K LOC | ~2M 令牌/秒 | |
| 解析 | ~1.2ms/1K LOC | ~800K AST 节点/秒 | 并行 2.18 倍加速 |
| 类型检查 | ~2.5ms/1K LOC | ~400K 类型/秒 | 基于 DAG 的并行 |
| 代码生成 | ~3.0ms/1K LOC | ~300K IR 行/秒 | 并行 4.14 倍加速 |
| **完整管道** | **~1.25ms/1K LOC** | **~800K 行/秒** | **50K 行 → 63ms** |

**自托管:** repository 包含 50,000+ 行 Vais compiler source，用于 bootstrap 和 conformance work。当前正确性以 certified Core gate 和 promoted runtime fixtures 判断。

### 运行时性能

斐波那契(35) 基准测试 (Apple M 系列 ARM64, 2026-02-11):

| 语言 | 时间 | 相对值 |
|----------|------|----------|
| C (clang -O3) | 32ms | 0.94x |
| Rust (release) | 33ms | 0.97x |
| **Vais** (clang -O2) | **34ms** | **1.0x** |
| Python | 3200ms | ~94x 更慢 |

## 构建

```bash
cargo build --release
cargo test                                     # 运行 Rust test suite
cargo test -p vaisc                            # 运行 vaisc package tests
cargo clippy --workspace --exclude vais-python --exclude vais-node
```

## 使用

```bash
# 编译 Vais 文件
./target/release/vaisc build hello.vais -o hello

# 直接运行
./target/release/vaisc run hello.vais

# 启动 REPL
./target/release/vaisc repl

# 格式化代码
./target/release/vaisc fmt src/

# 检查错误
./target/release/vaisc check hello.vais
```

## 入门指南

- 在您的系统上[安装 Vais](./getting-started/installation.md)
- 遵循[快速开始](./getting-started/quick-start.md)指南
- 通过[教程](./getting-started/tutorial.md)学习
- 阅读[语言规范](./language/language-spec.md)
- 探索[标准库](./stdlib/overview.md)

## 文档

- [语言规范](./language/language-spec.md) - 完整的语言规范
- [标准库参考](./stdlib/overview.md) - 标准库参考
- [教程](./getting-started/tutorial.md) - 入门教程
- [架构](./advanced/architecture.md) - 编译器架构与设计
- [安装指南](./getting-started/installation.md) - 安装指南

## 社区与支持

- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
- **文档**: [https://vaislang.dev/docs/](https://vaislang.dev/docs/)
- **Playground**: [https://vaislang.dev/playground/](https://vaislang.dev/playground/)
- **网站**: [https://vaislang.dev/](https://vaislang.dev/)
- **Issues**: 报告 bug 和功能请求
- **讨论**: [GitHub Discussions](https://github.com/vaislang/vais/discussions)

## 链接

| 资源 | URL |
|----------|-----|
| **GitHub 组织** | https://github.com/vaislang |
| **代码仓库** | https://github.com/vaislang/vais |
| **文档** | https://vaislang.dev/docs/ |
| **Playground** | https://vaislang.dev/playground/ |
| **网站** | https://vaislang.dev/ |
| **Docker Hub** | `vaislang/vais` |
| **Homebrew Tap** | `vaislang/tap` |
| **生态系统包** | https://github.com/vaislang/vais/tree/main/packages (9 个包) |

## 许可证

Vais 是 MIT 许可的开源软件。
