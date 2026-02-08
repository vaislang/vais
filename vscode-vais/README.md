# Vais Language Support for Visual Studio Code

Syntax highlighting and language support for the **Vais** programming language - an AI-optimized language with single-character keywords.

## Features

- **Syntax Highlighting**: Full syntax highlighting for Vais code including:
  - Single-character keywords (F, S, E, I, L, M, W, T, X, V, C, R, B, N, A)
  - Type annotations (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, str)
  - Self-recursion operator `@`
  - Async/await syntax
  - Comments, strings, and literals

- **Language Server Protocol (LSP) Support**:
  - Real-time diagnostics and error reporting
  - Code completion (IntelliSense)
  - Hover information for symbols
  - Go to definition
  - Find all references
  - Document symbols
  - Workspace symbols

- **Debugging Support (DAP)**:
  - Launch and debug Vais programs
  - Attach to running processes
  - Breakpoints and stepping
  - Variable inspection
  - Call stack navigation
  - Auto-compilation before debugging

- **Code Snippets**:
  - 50+ built-in snippets for common patterns
  - Function definitions, structs, enums, traits
  - Control flow constructs
  - Async/await patterns
  - Self-recursion templates

- **Build Tasks**:
  - Integrated task definitions for Vais compiler
  - Build, test, run, and benchmark commands
  - Problem matcher for compiler errors
  - Configurable optimization levels

- **Language Features**:
  - Bracket matching and auto-closing pairs
  - Comment toggling with `#`
  - Smart indentation
  - Code folding

## Vais Language Overview

Vais is an AI-optimized programming language with Rust-like syntax but shorter keywords:

### Keywords

- `F` - Function definition
- `S` - Struct definition
- `E` - Enum definition
- `I` - If statement
- `L` - Loop
- `M` - Match expression
- `W` - While loop / Trait (interface)
- `T` - Trait definition
- `X` - Implementation block
- `V` - Variable declaration (let)
- `C` - Constant declaration
- `R` - Return statement
- `B` - Break statement
- `N` - Continue statement
- `A` - Async function modifier

### Special Operators

- `@` - Self-recursion operator (calls the current function recursively)
- `:=` - Variable declaration with type inference
- `->` - Return type annotation
- `=>` - Match arm separator

### Example Code

```vais
# Fibonacci with self-recursion
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}

# Struct definition
S Point {
    x: i64,
    y: i64
}

# Enum definition
E Option<T> {
    Some(T),
    None
}

# Trait definition
W Printable {
    F print(&self) -> i64
}

# Implementation
X Point: Printable {
    F print(&self) -> i64 {
        print_i64(self.x)
        print_i64(self.y)
        0
    }
}

# Async function
A F compute(x: i64) -> i64 {
    x * 2
}

# Main function
F main() -> i64 {
    V result = fib(10)
    print_i64(result)

    V value := compute(21).await
    print_i64(value)

    0
}
```

## Installation

### Prerequisites

For full language support, you need to build the Vais compiler and tools:

**Requirements:**
- LLVM 17 (required for compilation)
- Rust toolchain (for building from source)

```bash
# In the vais project root directory
cargo build --release

# The binaries will be at target/release/
# - vaisc: Vais compiler
# - vais-lsp: Language server
# - vais-dap: Debug adapter

# Optionally, add them to your PATH:
export PATH="$PATH:/path/to/vais/target/release"
```

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/vais
   cd vais/vscode-vais
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Compile TypeScript:
   ```bash
   npm run compile
   ```

4. Package the extension:
   ```bash
   npm install -g vsce
   vsce package
   ```

5. Install the generated `.vsix` file in VSCode:
   - Open VSCode
   - Go to Extensions (Ctrl+Shift+X / Cmd+Shift+X)
   - Click the "..." menu at the top
   - Select "Install from VSIX..."
   - Choose the generated `.vsix` file

### From Marketplace

Coming soon!

## Language Features

### Comments

Single-line comments start with `#`:

```vais
# This is a comment
F main() -> i64 = 0  # Inline comment
```

### Type System

Built-in types:
- **Integers**: `i8`, `i16`, `i32`, `i64` (signed), `u8`, `u16`, `u32`, `u64` (unsigned)
- **Floats**: `f32`, `f64`
- **Boolean**: `bool` (`true`, `false`)
- **String**: `str`
- **Self type**: `Self`

### Self-Recursion

Use `@` to recursively call the current function:

```vais
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)
```

### Pattern Matching

```vais
M value {
    0 => result_for_zero,
    1 => result_for_one,
    _ => default_result
}
```

## Development

To contribute to this extension:

1. Clone the repository
2. Open the `vscode-vais` folder in VSCode
3. Press F5 to launch Extension Development Host
4. Test your changes
5. Submit a pull request

## Configuration

The extension can be configured through VSCode settings:

### Language Server Path

By default, the extension looks for `vais-lsp` in your PATH. You can specify a custom path:

```json
{
  "vais.languageServer.path": "/path/to/vais-lsp"
}
```

If you're working in a Vais workspace, the extension will automatically try to use `target/release/vais-lsp` from your workspace root.

### Debug Adapter Path

Configure the debug adapter location:

```json
{
  "vais.debugAdapter.path": "/path/to/vais-dap"
}
```

### Trace Server Communication

For debugging LSP issues, you can enable communication tracing:

```json
{
  "vais.trace.server": "verbose"
}
```

Options: `off`, `messages`, `verbose`

## Keyboard Shortcuts

The extension provides quick access to common operations:

- **Compile Current File**: No default shortcut (can be configured via keybindings)
- **Run Tests**: No default shortcut (can be configured via keybindings)
- **Build Project**: Use VSCode tasks (`Ctrl+Shift+B` / `Cmd+Shift+B`)
- **Start Debugging**: `F5`
- **Toggle Breakpoint**: `F9`

To configure custom shortcuts, open Keyboard Shortcuts (`Ctrl+K Ctrl+S` / `Cmd+K Cmd+S`) and search for "vais".

## Build Tasks

The extension supports Vais build tasks that can be configured in `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "type": "vais",
      "command": "build",
      "file": "${file}",
      "optimizationLevel": 2,
      "problemMatcher": ["$vais"],
      "label": "vais: Build",
      "group": {
        "kind": "build",
        "isDefault": true
      }
    },
    {
      "type": "vais",
      "command": "test",
      "problemMatcher": ["$vais"],
      "label": "vais: Test"
    },
    {
      "type": "vais",
      "command": "bench",
      "label": "vais: Benchmark"
    }
  ]
}
```

The `$vais` problem matcher automatically parses compiler errors and warnings in the format:
```
error[E001]: Type mismatch
  → file.vais:10:5
```

## Requirements

- Visual Studio Code 1.75.0 or higher
- LLVM 17 (required for Vais compilation)
- Vais compiler and tools:
  - `vaisc`: Vais compiler
  - `vais-lsp`: Language server (for LSP features)
  - `vais-dap`: Debug adapter (for debugging support)

## Known Issues

- Vais compiler and tools must be manually built from source
- Some advanced debugging features are still in development

## Release Notes

### 0.2.0

- Added Language Server Protocol (LSP) support
- Real-time diagnostics and error checking
- Code completion (IntelliSense)
- Hover information
- Go to definition
- Find all references
- Document and workspace symbols
- Configurable LSP server path
- Status bar indicator for LSP server state

### 0.1.0

- Initial release
- Basic syntax highlighting
- Bracket matching and auto-closing
- Comment support
- Support for all Vais keywords and operators

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

MIT License - see LICENSE file for details

## Resources

- [Vais GitHub Repository](https://github.com/vaislang/vais)
- [Report Issues](https://github.com/vaislang/vais/issues)
- [Documentation](https://github.com/vaislang/vais/tree/main/docs-site)

---

**Enjoy coding in Vais!**
