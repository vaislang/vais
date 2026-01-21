# vais-python

Python bindings for the Vais compiler, providing a native Python interface to compile, check, parse, and tokenize Vais source code.

## Features

- **compile(source, opt_level=0, module_name=None, target=None)** - Compile Vais source to LLVM IR
- **check(source)** - Type check Vais source code
- **parse(source)** - Parse Vais source into an AST
- **tokenize(source)** - Tokenize Vais source code

## Installation

```bash
# Build the Python extension
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release -p vais-python

# The compiled module will be in target/release/
```

## Usage

```python
import vais

# Tokenize source code
source = "F add(a:i64,b:i64)->i64=a+b"
tokens = vais.tokenize(source)
for token in tokens:
    print(f"{token.token_type} at {token.span}: {token.text}")

# Check for type errors
errors = vais.check(source)
if not errors:
    print("No type errors!")
else:
    for err in errors:
        print(f"{err.error_type}: {err.message}")

# Parse to AST
ast = vais.parse(source)
print(f"Module with {ast['items_count']} items")

# Compile to LLVM IR
ir = vais.compile(source, opt_level=2, module_name="my_module")
print(ir)

# Compile with target specification
ir = vais.compile(source, target="wasm32-unknown-unknown")
```

## API Reference

### Functions

#### compile(source, opt_level=0, module_name=None, target=None)

Compiles Vais source code to LLVM IR.

**Parameters:**
- `source` (str): The Vais source code
- `opt_level` (int, optional): Optimization level 0-3 (default: 0)
- `module_name` (str, optional): Name of the module (default: "main")
- `target` (str, optional): Target triple (default: native)

**Returns:** str - The compiled LLVM IR

**Raises:** RuntimeError if compilation fails

#### check(source)

Type checks Vais source code.

**Parameters:**
- `source` (str): The Vais source code

**Returns:** List[Error] - List of errors (empty if no errors)

#### parse(source)

Parses Vais source code into an AST representation.

**Parameters:**
- `source` (str): The Vais source code

**Returns:** dict - Dictionary representing the AST

**Raises:** ValueError if parsing fails

#### tokenize(source)

Tokenizes Vais source code.

**Parameters:**
- `source` (str): The Vais source code

**Returns:** List[TokenInfo] - List of tokens

**Raises:** ValueError if tokenization fails

### Classes

#### Error

Represents a compilation or type error.

**Attributes:**
- `message` (str): Error message
- `span` (tuple[int, int] | None): Source location (start, end)
- `error_type` (str): Type of error (e.g., "TypeError", "ParseError")

#### TokenInfo

Represents a token from the lexer.

**Attributes:**
- `token_type` (str): Type of token (e.g., "Function", "Ident", "Int")
- `span` (tuple[int, int]): Source location (start, end)
- `text` (str | None): Token text if applicable

## Building

```bash
# Development build
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build -p vais-python

# Release build
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release -p vais-python

# Install in development mode (requires maturin)
maturin develop
```

## Requirements

- Rust 1.70+
- Python 3.8+
- pyo3 0.22+

## License

MIT
