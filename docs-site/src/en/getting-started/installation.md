# Installation

This guide will help you install the Vais compiler on your system.

## Quick Install (Recommended)

The fastest way to get started with Vais.

### Homebrew (macOS / Linux)

```bash
brew tap vaislang/tap
brew install vais
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/vaislang/vais/releases/tag/v1.0.0):

| Platform | Download |
|----------|----------|
| macOS ARM (Apple Silicon) | [vais-v1.0.0-aarch64-apple-darwin.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz) |
| macOS Intel | [vais-v1.0.0-x86_64-apple-darwin.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-apple-darwin.tar.gz) |
| Linux x86_64 | [vais-v1.0.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-unknown-linux-gnu.tar.gz) |
| Windows x86_64 | [vais-v1.0.0-x86_64-pc-windows-msvc.zip](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-pc-windows-msvc.zip) |

```bash
# Example: macOS ARM
curl -LO https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz
tar -xzf vais-v1.0.0-aarch64-apple-darwin.tar.gz
sudo cp vais/vaisc /usr/local/bin/
```

### Prerequisite: clang

Vais uses `clang` to compile generated LLVM IR to native binaries:

- **macOS**: `xcode-select --install`
- **Linux**: `sudo apt install clang` or `sudo dnf install clang`
- **Windows**: Install from https://releases.llvm.org

## Build from Source

If you want to build from source or contribute to development.

### System Requirements

- **Rust 1.70+** - The Vais compiler is written in Rust
- **LLVM/Clang 17+** - For LLVM backend code generation
- **Git** - For cloning the repository
- **CMake 3.15+** - Required by LLVM build tools

### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies via Homebrew
brew install llvm@17 clang cmake pkg-config

# Set up environment
export LLVM_DIR=/usr/local/opt/llvm@17
export PATH="/usr/local/opt/llvm@17/bin:$PATH"

# Clone and build
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Linux (Ubuntu/Debian)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config \
    llvm-17 clang-17 libllvm-17-ocaml-dev

# Set up environment
export LLVM_DIR=/usr/lib/llvm-17
export PATH="/usr/lib/llvm-17/bin:$PATH"

# Clone and build
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Windows

```bash
# Install Rust from https://rustup.rs/
# Install Visual Studio Build Tools
# Install LLVM 17 from https://releases.llvm.org
# Install CMake from https://cmake.org/download/

# Clone and build
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

## Verification

Test your installation:

```bash
# Check compiler version
vaisc --version

# Compile an example
vaisc build examples/hello.vais -o hello
./hello
```

Expected output:

```
Hello, World!
```

## Troubleshooting

### LLVM Not Found

**Error:** `error: LLVM 17 not found`

**Solution:**
- Verify LLVM 17 is installed: `llvm-config-17 --version`
- Set `LLVM_DIR` correctly
- On macOS: `export LLVM_DIR=$(brew --prefix llvm@17)`

### Missing Standard Library

**Error:** `error: standard library not found`

**Solution:**
- Verify `std/` directory exists in project root
- Set `VAIS_STD_PATH`: `export VAIS_STD_PATH=$(pwd)/std`

## Next Steps

- Follow the [Quick Start](./quick-start.md) guide
- Read the [Tutorial](./tutorial.md)
- Explore [example programs](https://github.com/vaislang/vais/tree/main/examples)
