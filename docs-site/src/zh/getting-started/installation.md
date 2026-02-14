# 安装

本指南将帮助您在系统上安装 Vais 编译器。

## 快速安装(推荐)

开始使用 Vais 的最快方法。

### Homebrew (macOS / Linux)

```bash
brew tap vaislang/tap
brew install vais
```

### 预编译二进制文件

从 [GitHub Releases](https://github.com/vaislang/vais/releases/tag/v1.0.0) 下载:

| 平台 | 下载 |
|----------|----------|
| macOS ARM (Apple Silicon) | [vais-v1.0.0-aarch64-apple-darwin.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz) |
| macOS Intel | [vais-v1.0.0-x86_64-apple-darwin.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-apple-darwin.tar.gz) |
| Linux x86_64 | [vais-v1.0.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-unknown-linux-gnu.tar.gz) |
| Windows x86_64 | [vais-v1.0.0-x86_64-pc-windows-msvc.zip](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-pc-windows-msvc.zip) |

```bash
# 示例: macOS ARM
curl -LO https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz
tar -xzf vais-v1.0.0-aarch64-apple-darwin.tar.gz
sudo cp vais/vaisc /usr/local/bin/
```

### 前置条件: clang

Vais 使用 `clang` 将生成的 LLVM IR 编译为原生二进制文件:

- **macOS**: `xcode-select --install`
- **Linux**: `sudo apt install clang` 或 `sudo dnf install clang`
- **Windows**: 从 https://releases.llvm.org 安装

## 从源代码构建

如果您想从源代码构建或为开发做贡献。

### 系统要求

- **Rust 1.70+** - Vais 编译器使用 Rust 编写
- **LLVM/Clang 17+** - 用于 LLVM 后端代码生成
- **Git** - 用于克隆仓库
- **CMake 3.15+** - LLVM 构建工具所需

### macOS

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 通过 Homebrew 安装依赖
brew install llvm@17 clang cmake pkg-config

# 设置环境
export LLVM_DIR=/usr/local/opt/llvm@17
export PATH="/usr/local/opt/llvm@17/bin:$PATH"

# 克隆并构建
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Linux (Ubuntu/Debian)

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装依赖
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config \
    llvm-17 clang-17 libllvm-17-ocaml-dev

# 设置环境
export LLVM_DIR=/usr/lib/llvm-17
export PATH="/usr/lib/llvm-17/bin:$PATH"

# 克隆并构建
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Windows

```bash
# 从 https://rustup.rs/ 安装 Rust
# 安装 Visual Studio Build Tools
# 从 https://releases.llvm.org 安装 LLVM 17
# 从 https://cmake.org/download/ 安装 CMake

# 克隆并构建
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

## 验证

测试您的安装:

```bash
# 检查编译器版本
vaisc --version

# 编译示例
vaisc build examples/hello.vais -o hello
./hello
```

预期输出:

```
Hello, World!
```

## 故障排除

### 找不到 LLVM

**错误:** `error: LLVM 17 not found`

**解决方案:**
- 验证 LLVM 17 已安装: `llvm-config-17 --version`
- 正确设置 `LLVM_DIR`
- 在 macOS 上: `export LLVM_DIR=$(brew --prefix llvm@17)`

### 找不到标准库

**错误:** `error: standard library not found`

**解决方案:**
- 验证项目根目录存在 `std/` 目录
- 设置 `VAIS_STD_PATH`: `export VAIS_STD_PATH=$(pwd)/std`

## 下一步

- 遵循[快速开始](./quick-start.md)指南
- 阅读[教程](./tutorial.md)
- 探索[示例程序](https://github.com/vaislang/vais/tree/main/examples)
