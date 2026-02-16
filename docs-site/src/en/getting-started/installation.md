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

## Editor Setup

### VSCode Extension Installation

#### From Marketplace

1. Open VSCode
2. Press `Ctrl+Shift+X` (Windows/Linux) or `Cmd+Shift+X` (macOS)
3. Search for "vais" or "vais-language"
4. Click Install

#### From Source

```bash
cd vscode-vais
npm install
npm run build
code --install-extension vais-language-server-0.0.1.vsix
```

#### Configure Extension

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

### IntelliJ Plugin

Available for IntelliJ IDEA and other JetBrains IDEs. Provides syntax highlighting and basic LSP integration.

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

# E2E tests (655 tests)
cargo test -p vaisc --test e2e_tests

# Benchmark tests
cargo bench
```

## Troubleshooting

### LLVM Not Found

**Error:** `error: LLVM 17 not found`

**Solution:**
- Verify LLVM 17 is installed: `llvm-config-17 --version`
- Set `LLVM_DIR` correctly:
  - On macOS: `export LLVM_DIR=$(brew --prefix llvm@17)`
  - On Linux (Ubuntu): `export LLVM_DIR=/usr/lib/llvm-17`
  - On Linux (Fedora): `export LLVM_DIR=/usr/lib64/llvm-17`

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
- Build specific crates: `cargo build -p vaisc`

### Missing Standard Library

**Error:** `error: standard library not found`

**Solution:**
- Verify `std/` directory exists in project root
- Set `VAIS_STD_PATH`: `export VAIS_STD_PATH=$(pwd)/std`
- Re-clone the repository if `std/` is missing

### VSCode Extension Not Working

**Solution:**
- Ensure compiler path in settings is correct
- Restart VSCode: `Cmd+Shift+P` → "Developer: Reload Window"
- Check LSP output: View → Output → select "Vais Language Server"
- Verify LSP is running: `ps aux | grep vaisc`

### Platform-Specific Issues

**macOS Apple Silicon (M1/M2/M3):**
- Use native ARM64 builds; Rosetta translation may cause issues
- Ensure Homebrew and dependencies are installed for ARM64
- Check architecture: `uname -m` (should output `arm64`)

**Windows Path Issues:**
- Use full paths or add to system PATH environment variable
- Restart Command Prompt/PowerShell after modifying PATH
- Use forward slashes or escaped backslashes in paths

**Linux GLIBC Compatibility:**
- Error: `GLIBC_2.XX not found`
- Solution: Update glibc or compile with `RUSTFLAGS="-C target-cpu=generic"`

### Clang Not Found

**Error:** `error: clang not found in PATH`

**Solution:**
- Install clang:
  - macOS: `xcode-select --install`
  - Ubuntu/Debian: `sudo apt install clang`
  - Fedora: `sudo dnf install clang`
  - Windows: Download from https://releases.llvm.org
- Verify installation: `clang --version`

## Docker Installation

### Using Pre-built Image

```bash
# Pull and run
docker run -it vaislang/vais:latest

# Run with mounted volume
docker run -it -v $(pwd):/workspace vaislang/vais:latest
```

### Building Docker Image

```bash
# Build from Dockerfile
docker build -t vais:local .

# Run
docker run -it vais:local
```

## Next Steps

After successful installation:

1. **[Quick Start](./quick-start.md)** - Write your first program in 5 minutes
2. **[Tutorial](./tutorial.md)** - Complete guide from basics to advanced features
3. **[Language Specification](../language/language-spec.md)** - Full syntax reference
4. **[Standard Library](https://github.com/vaislang/vais/tree/main/std)** - Explore built-in modules
5. **[Example Programs](https://github.com/vaislang/vais/tree/main/examples)** - Real-world code samples

## Getting Help

- **GitHub Issues**: https://github.com/vaislang/vais/issues
- **Documentation**: https://vais.dev/docs/
- **Discussions**: https://github.com/vaislang/vais/discussions
- **Contributing**: See [CONTRIBUTING.md](https://github.com/vaislang/vais/blob/main/CONTRIBUTING.md)

## Version Information

- **Vais Version:** 1.0.0
- **LLVM Requirement:** 17.x
- **Rust MSRV:** 1.70
- **License:** MIT
