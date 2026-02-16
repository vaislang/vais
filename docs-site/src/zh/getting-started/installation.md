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

#### 必需软件

- **Rust 1.70+** - Vais 编译器使用 Rust 编写
- **LLVM/Clang 17+** - 用于 LLVM 后端代码生成
- **Git** - 用于克隆仓库
- **CMake 3.15+** - LLVM 构建工具所需

#### 硬件要求

- 最低 2GB RAM
- 至少 5GB 磁盘空间 (用于构建产物和 LLVM)
- 推荐多核处理器

#### 支持的平台

- macOS 11+ (Intel 和 Apple Silicon)
- Linux (Ubuntu 20.04+, Fedora 35+, Debian 11+)
- Windows 10+ (MSVC 或 WSL2)

### macOS

#### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. 安装 LLVM 和依赖项

使用 Homebrew:

```bash
brew install llvm@17 clang cmake pkg-config
```

设置环境变量:

```bash
# 添加到 ~/.zshrc 或 ~/.bash_profile
export LLVM_DIR=/usr/local/opt/llvm@17
export PATH="/usr/local/opt/llvm@17/bin:$PATH"
```

#### 3. 克隆并构建

```bash
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Linux (Ubuntu/Debian)

#### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. 安装依赖项

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    llvm-17 \
    clang-17 \
    libllvm-17-ocaml-dev
```

#### 3. 设置环境变量

```bash
# 添加到 ~/.bashrc
export LLVM_DIR=/usr/lib/llvm-17
export PATH="/usr/lib/llvm-17/bin:$PATH"
```

#### 4. 克隆并构建

```bash
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Linux (Fedora/RHEL)

#### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. 安装依赖项

```bash
sudo dnf install -y \
    gcc-c++ \
    cmake \
    llvm-devel-17 \
    clang-tools-extra-17 \
    pkg-config
```

#### 3. 设置环境变量

```bash
# 添加到 ~/.bashrc
export LLVM_DIR=/usr/lib64/llvm-17
export PATH="/usr/lib64/llvm-17/bin:$PATH"
```

#### 4. 克隆并构建

```bash
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Windows

#### 1. 安装 Rust

从 https://rustup.rs/ 下载并运行安装程序

#### 2. 安装 Visual Studio Build Tools

下载地址: https://visualstudio.microsoft.com/visual-cpp-build-tools/

选择 "Desktop development with C++"

#### 3. 安装 LLVM 17

下载地址: https://releases.llvm.org/download.html (LLVM-17.x.x-win64.exe)

安装时选择将 LLVM 添加到 PATH。

#### 4. 安装 CMake

下载地址: https://cmake.org/download/

确保将 CMake 添加到 PATH。

#### 5. 克隆并构建

```bash
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

## 从源代码构建

### 开发构建

在开发期间更快的编译:

```bash
cargo build
```

编译器将位于 `./target/debug/vaisc`

### 发布构建

优化的生产二进制文件:

```bash
cargo build --release
```

编译器将位于 `./target/release/vaisc`

### 使用特定功能构建

```bash
# 启用 LSP 支持 (默认包含)
cargo build --release --all-features

# 构建时运行所有测试
cargo build --release --all-targets
```

## 环境设置

### 将 Vais 添加到 PATH

为了从任何地方轻松访问编译器:

```bash
# 对于发布构建
export PATH="$PATH:$(pwd)/target/release"

# 或创建符号链接
ln -s $(pwd)/target/release/vaisc /usr/local/bin/vaisc
```

### 配置标准库路径

设置 `VAIS_STD_PATH` 环境变量 (可选，默认自动检测):

```bash
export VAIS_STD_PATH=$(pwd)/std
```

### LSP 配置

Vais 语言服务器内置于编译器中:

```bash
vaisc lsp
```

这将在 stdio 上启动语言服务器。

## VSCode 扩展安装

### 从市场安装

1. 打开 VSCode
2. 按 `Ctrl+Shift+X` (Windows/Linux) 或 `Cmd+Shift+X` (macOS)
3. 搜索 "vais" 或 "vais-language"
4. 点击安装

### 从源代码安装

```bash
cd vscode-vais
npm install
npm run build
code --install-extension vais-language-server-0.0.1.vsix
```

### 配置扩展

添加到 `.vscode/settings.json`:

```json
{
  "vais.compilerPath": "/path/to/vaisc",
  "vais.stdPath": "/path/to/vais/std",
  "vais.lsp.enable": true,
  "[vais]": {
    "editor.defaultFormatter": "vais.vais-language",
    "editor.formatOnSave": true
  }
}
```

## 验证

### 测试安装

```bash
# 检查编译器版本
vaisc --version

# 运行测试套件
cargo test --release

# 编译示例
vaisc build examples/hello.vais -o hello
./hello
```

### 预期输出

```bash
$ ./hello
Hello, World!
```

### 运行综合测试

```bash
# 单元测试
cargo test --lib

# 集成测试
cargo test --test '*'

# 基准测试
cargo bench
```

## 故障排除

### 找不到 LLVM

**错误:** `error: LLVM 17 not found`

**解决方案:**
- 验证 LLVM 17 已安装: `llvm-config-17 --version`
- 正确设置 `LLVM_DIR`
- 在 macOS 上: `export LLVM_DIR=$(brew --prefix llvm@17)`

### Rust 编译错误

**错误:** `error: could not compile vaisc`

**解决方案:**
- 更新 Rust: `rustup update`
- 清理构建产物: `cargo clean && cargo build --release`
- 检查 Rust 版本: `rustc --version` (应为 1.70+)

### 构建期间内存不足

**解决方案:**
- 增加交换空间或 RAM
- 不使用优化构建: `cargo build` (而不是 `--release`)
- 关闭其他应用程序

### 找不到标准库

**错误:** `error: standard library not found`

**解决方案:**
- 验证项目根目录存在 `std/` 目录
- 设置 `VAIS_STD_PATH`: `export VAIS_STD_PATH=$(pwd)/std`

### VSCode 扩展不工作

**解决方案:**
- 确保设置中的编译器路径正确
- 重启 VSCode: `Cmd+Shift+P` → "Developer: Reload Window"
- 检查 LSP 输出: View → Output → 选择 "Vais Language Server"

### 平台特定问题

**macOS Apple Silicon (M1/M2):**
- 使用原生 ARM64 构建; Rosetta 转换可能导致问题
- 确保为 ARM64 安装 Homebrew 和依赖项

**Windows 路径问题:**
- 使用完整路径或添加到系统 PATH 环境变量
- 修改 PATH 后重启命令提示符/PowerShell

**Linux GLIBC 兼容性:**
- 错误: `GLIBC_2.XX not found`
- 解决方案: 更新 glibc 或使用 `RUSTFLAGS="-C target-cpu=generic"` 编译

## Docker 安装

### 使用预构建镜像

```bash
# 拉取并运行
docker run -it vaislang/vais:latest

# 使用挂载卷运行
docker run -it -v $(pwd):/workspace vaislang/vais:latest
```

### 构建 Docker 镜像

```bash
# 从 Dockerfile 构建
docker build -t vais:local .

# 运行
docker run -it vais:local
```

## 下一步

安装成功后:

1. **阅读教程:** [教程](./tutorial.md)
2. **语言规范:** [语言规范](../language/language-spec.md)
3. **标准库:** [标准库概览](../stdlib/overview.md)
4. **示例程序:** [示例目录](https://github.com/vaislang/vais/tree/main/examples)

## 获取帮助

- GitHub Issues: https://github.com/vaislang/vais/issues
- 文档: https://github.com/vaislang/vais/tree/main/docs
- 贡献: 参见 `CONTRIBUTING.md`

## 版本信息

- **Vais 版本:** 1.0.0
- **LLVM 要求:** 17.x
- **Rust MSRV:** 1.70
- **许可证:** MIT
