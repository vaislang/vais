# Vais Compiler Installation Guide

Welcome to the Vais compiler installation guide. This document will walk you through setting up Vais on your system, from prerequisites to verification.

## System Requirements

### Required Software

- **Rust 1.70+** - The Vais compiler is written in Rust
- **LLVM/Clang 17+** - For LLVM backend code generation
- **Git** - For cloning the repository
- **CMake 3.15+** - Required by LLVM build tools

### Hardware Requirements

- Minimum 2GB RAM
- At least 5GB disk space (for build artifacts and LLVM)
- Multi-core processor recommended

### Supported Platforms

- macOS 11+ (Intel and Apple Silicon)
- Linux (Ubuntu 20.04+, Fedora 35+, Debian 11+)
- Windows 10+ (with MSVC or WSL2)

## Installation by Platform

### macOS

#### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. Install LLVM and Dependencies

Using Homebrew:

```bash
brew install llvm@17 clang cmake pkg-config
```

Set up environment variables:

```bash
# Add to ~/.zshrc or ~/.bash_profile
export LLVM_DIR=/usr/local/opt/llvm@17
export PATH="/usr/local/opt/llvm@17/bin:$PATH"
```

#### 3. Clone and Build

```bash
git clone https://github.com/sswoo88/vais.git
cd vais
cargo build --release
```

### Linux (Ubuntu/Debian)

#### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. Install Dependencies

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

#### 3. Set Environment Variables

```bash
# Add to ~/.bashrc
export LLVM_DIR=/usr/lib/llvm-17
export PATH="/usr/lib/llvm-17/bin:$PATH"
```

#### 4. Clone and Build

```bash
git clone https://github.com/sswoo88/vais.git
cd vais
cargo build --release
```

### Linux (Fedora/RHEL)

#### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. Install Dependencies

```bash
sudo dnf install -y \
    gcc-c++ \
    cmake \
    llvm-devel-17 \
    clang-tools-extra-17 \
    pkg-config
```

#### 3. Set Environment Variables

```bash
# Add to ~/.bashrc
export LLVM_DIR=/usr/lib64/llvm-17
export PATH="/usr/lib64/llvm-17/bin:$PATH"
```

#### 4. Clone and Build

```bash
git clone https://github.com/sswoo88/vais.git
cd vais
cargo build --release
```

### Windows

#### 1. Install Rust

Download and run the installer from https://rustup.rs/

#### 2. Install Visual Studio Build Tools

Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

Select "Desktop development with C++"

#### 3. Install LLVM 17

Download from: https://releases.llvm.org/download.html (LLVM-17.x.x-win64.exe)

During installation, add LLVM to PATH when prompted.

#### 4. Install CMake

Download from: https://cmake.org/download/

Ensure CMake is added to PATH.

#### 5. Clone and Build

```bash
git clone https://github.com/sswoo88/vais.git
cd vais
cargo build --release
```

## Building from Source

### Development Build

For faster compilation during development:

```bash
cargo build
```

The compiler will be available at `./target/debug/vaisc`

### Release Build

For optimized production binary:

```bash
cargo build --release
```

The compiler will be available at `./target/release/vaisc`

### Build with Specific Features

```bash
# With LSP support (included by default)
cargo build --release --all-features

# Run all tests during build
cargo build --release --all-targets
```

## Environment Setup

### Add Vais to PATH

For easy access to the compiler from anywhere:

```bash
# For release build
export PATH="$PATH:$(pwd)/target/release"

# Or create a symlink
ln -s $(pwd)/target/release/vaisc /usr/local/bin/vaisc
```

### Configure Standard Library Path

Set the `VAIS_STD_PATH` environment variable (optional, auto-detected by default):

```bash
export VAIS_STD_PATH=$(pwd)/std
```

### LSP Configuration

The Vais Language Server is built into the compiler:

```bash
vaisc lsp
```

This starts the language server on stdio.

## VSCode Extension Installation

### From Marketplace

1. Open VSCode
2. Press `Ctrl+Shift+X` (Windows/Linux) or `Cmd+Shift+X` (macOS)
3. Search for "vais" or "vais-language"
4. Click Install

### From Source

```bash
cd vscode-vais
npm install
npm run build
code --install-extension vais-language-server-0.0.1.vsix
```

### Configure Extension

Add to `.vscode/settings.json`:

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

## Verification

### Test the Installation

```bash
# Check compiler version
vaisc --version

# Run the test suite
cargo test --release

# Compile an example
vaisc build examples/hello.vais -o hello
./hello
```

### Expected Output

```bash
$ ./hello
Hello, World!
```

### Run Comprehensive Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Benchmark tests
cargo bench
```

## Troubleshooting

### LLVM Not Found

**Error:** `error: LLVM 17 not found`

**Solution:**
- Verify LLVM 17 is installed: `llvm-config-17 --version`
- Set `LLVM_DIR` correctly
- On macOS: `export LLVM_DIR=$(brew --prefix llvm@17)`

### Rust Compilation Errors

**Error:** `error: could not compile vaisc`

**Solution:**
- Update Rust: `rustup update`
- Clean build artifacts: `cargo clean && cargo build --release`
- Check Rust version: `rustc --version` (should be 1.70+)

### Out of Memory During Build

**Solution:**
- Increase swap space or RAM
- Build without optimizations: `cargo build` (instead of `--release`)
- Close other applications

### Missing Standard Library

**Error:** `error: standard library not found`

**Solution:**
- Verify `std/` directory exists in project root
- Set `VAIS_STD_PATH`: `export VAIS_STD_PATH=$(pwd)/std`

### VSCode Extension Not Working

**Solution:**
- Ensure compiler path in settings is correct
- Restart VSCode: `Cmd+Shift+P` → "Developer: Reload Window"
- Check LSP output: View → Output → select "Vais Language Server"

### Platform-Specific Issues

**macOS Apple Silicon (M1/M2):**
- Use native ARM64 builds; Rosetta translation may cause issues
- Ensure Homebrew and dependencies are installed for ARM64

**Windows Path Issues:**
- Use full paths or add to system PATH environment variable
- Restart Command Prompt/PowerShell after modifying PATH

**Linux GLIBC Compatibility:**
- Error: `GLIBC_2.XX not found`
- Solution: Update glibc or compile with `RUSTFLAGS="-C target-cpu=generic"`

## Next Steps

After successful installation:

1. **Read the Tutorial:** `docs/TUTORIAL.md`
2. **Language Specification:** `docs/LANGUAGE_SPEC.md`
3. **Standard Library:** `docs/STDLIB.md`
4. **Example Programs:** `examples/` directory

## Getting Help

- GitHub Issues: https://github.com/sswoo88/vais/issues
- Documentation: https://github.com/sswoo88/vais/tree/main/docs
- Contributing: See `CONTRIBUTING.md`

## Version Information

- **Vais Version:** 0.0.1
- **LLVM Requirement:** 17.x
- **Rust MSRV:** 1.70
- **License:** MIT
