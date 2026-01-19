# Vais Language Support for Visual Studio Code

Syntax highlighting and language support for the **Vais** programming language - an AI-optimized language with single-character keywords.

## Features

- **Syntax Highlighting**: Full syntax highlighting for Vais code including:
  - Single-character keywords (F, S, E, I, L, M, W, T, X, V, C, R, B, N, A)
  - Type annotations (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, str)
  - Self-recursion operator `@`
  - Async/await syntax
  - Comments, strings, and literals

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

3. Package the extension:
   ```bash
   npm install -g vsce
   vsce package
   ```

4. Install the generated `.vsix` file in VSCode:
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

## Requirements

- Visual Studio Code 1.75.0 or higher

## Known Issues

- No LSP (Language Server Protocol) support yet
- No debugging support yet
- Limited IntelliSense features

## Release Notes

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

- [Vais GitHub Repository](https://github.com/yourusername/vais)
- [Report Issues](https://github.com/yourusername/vais/issues)

---

**Enjoy coding in Vais!**
