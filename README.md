# Vais 2.0

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
└── vais-codegen/  # LLVM IR generator
```

## Building

```bash
cargo build
cargo test
```

## Status

- [x] Lexer
- [x] Parser
- [x] Type checker (basic)
- [ ] Code generator (in progress)
- [ ] Standard library
- [ ] LSP support

## Design Philosophy

See [DESIGN.md](DESIGN.md) for the complete language specification.

## Legacy

The prototype implementation is available on the [`proto`](https://github.com/sswoo88/vais/tree/proto) branch.

## License

MIT License
