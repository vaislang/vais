# Vais 0.0.1

**AI-optimized systems programming language with token-efficient syntax.**

Vais is designed to minimize token usage while maximizing code expressiveness, making it ideal for AI-assisted development and LLM code generation.

## Key Features

- **Single-letter keywords** - `F` (function), `S` (struct), `E` (enum/else), `I` (if), `L` (loop), `M` (match)
- **Self-recursion operator** `@` - Call the current function recursively
- **Expression-oriented** - Everything is an expression
- **LLVM backend** - Native performance
- **Type inference** - Minimal type annotations

## Quick Example

```vais
# Fibonacci with self-recursion
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# Struct definition
S Point { x:f64, y:f64 }

# Sum with loop
F sum(arr:[i64])->i64 {
    s := 0
    L x:arr { s += x }
    s
}
```

## Syntax Overview

| Keyword | Meaning | Example |
|---------|---------|---------|
| `F` | Function | `F add(a:i64,b:i64)->i64=a+b` |
| `S` | Struct | `S Point{x:f64,y:f64}` |
| `E` | Enum/Else | `E Option<T>{Some(T),None}` |
| `I` | If | `I x>0{1}E{-1}` |
| `L` | Loop | `L i:0..10{print(i)}` |
| `M` | Match | `M opt{Some(v)=>v,None=>0}` |
| `@` | Self-call | `@(n-1)` (recursive call) |
| `:=` | Infer & assign | `x := 42` |

## Project Structure

```
crates/
├── vais-ast/      # Abstract Syntax Tree
├── vais-lexer/    # Tokenizer (logos-based)
├── vais-parser/   # Recursive descent parser
├── vais-types/    # Type checker
├── vais-codegen/  # LLVM IR generator
├── vais-lsp/      # Language Server Protocol
└── vaisc/         # CLI compiler & REPL

std/               # Standard library (24 modules)
vscode-vais/       # VSCode extension
docs/              # Documentation
examples/          # Example programs (40+ files)
```

## Building

```bash
cargo build --release
cargo test
```

## Usage

```bash
# Compile a Vais file
./target/release/vaisc build hello.vais -o hello

# Run directly
./target/release/vaisc run hello.vais

# Start REPL
./target/release/vaisc repl

# Format code
./target/release/vaisc fmt src/

# Check for errors
./target/release/vaisc check hello.vais
```

## Status

- [x] Lexer (logos-based tokenizer)
- [x] Parser (recursive descent)
- [x] Type checker (generics, traits, type inference)
- [x] Code generator (LLVM IR)
- [x] Standard library (24 modules: Vec, HashMap, String, File, Net, etc.)
- [x] LSP support (diagnostics, completion, hover, go-to-definition)
- [x] REPL (interactive environment)
- [x] VSCode extension (syntax highlighting, LSP integration)
- [x] Optimizer (constant folding, DCE, CSE, loop unrolling, LICM)
- [x] Formatter (`vaisc fmt`)
- [x] Debugger (DWARF metadata, lldb/gdb support)

## Documentation

- [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md) - Complete language specification
- [STDLIB.md](docs/STDLIB.md) - Standard library reference
- [TUTORIAL.md](docs/TUTORIAL.md) - Getting started tutorial
- [ROADMAP.md](ROADMAP.md) - Project roadmap and progress

## Legacy

The prototype implementation is available on the [`proto`](https://github.com/sswoo88/vais/tree/proto) branch.

## License

MIT License
