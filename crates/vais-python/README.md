# vais-python

Python bindings for the Vais compiler, providing a native Python interface to compile, check, parse, and tokenize Vais source code.

## Features

### Function API

- **compile(source, opt_level=0, module_name=None, target=None)** - Compile Vais source to LLVM IR (raises on error)
- **compile_to_result(source, opt_level=0, module_name=None, target=None)** - Compile and return CompileResult
- **compile_and_run(source, opt_level=0)** - Compile and execute (returns RunResult)
- **check(source)** - Type check Vais source code
- **parse(source)** - Parse Vais source into an AST
- **tokenize(source)** - Tokenize Vais source code

### Object-Oriented API

- **VaisCompiler** - Stateful compiler instance with configurable settings

## Installation

```bash
# Build the Python extension
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release -p vais-python

# The compiled module will be in target/release/
```

## Usage

### Function API

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

# Compile to LLVM IR (raises on error)
ir = vais.compile(source, opt_level=2, module_name="my_module")
print(ir)

# Compile with result object (no exceptions)
result = vais.compile_to_result(source, opt_level=2)
if result.success:
    print(f"Compilation successful!\n{result.ir}")
else:
    for err in result.errors:
        print(f"Error: {err.message}")

# Compile with target specification
ir = vais.compile(source, target="wasm32-unknown-unknown")

# Compile and run (note: JIT execution not yet fully implemented)
run_result = vais.compile_and_run(source)
if run_result.success:
    print(f"Exit code: {run_result.exit_code}")
    print(f"Output: {run_result.stdout}")
```

### Object-Oriented API

```python
import vais

# Create a compiler instance
compiler = vais.VaisCompiler(opt_level=2, module_name="mymodule")

# Compile code
source = "F add(a:i64,b:i64)->i64=a+b"
result = compiler.compile(source)

if result.success:
    print(f"IR:\n{result.ir}")
else:
    print("Compilation failed:")
    for err in result.errors:
        print(f"  {err.error_type}: {err.message}")

# Use other methods
tokens = compiler.tokenize(source)
ast = compiler.parse(source)
errors = compiler.check(source)

# Change settings
compiler.set_opt_level(3)
compiler.set_target("wasm32-unknown-unknown")

# Get current settings
print(f"Optimization level: {compiler.get_opt_level()}")
print(f"Module name: {compiler.get_module_name()}")
print(f"Target: {compiler.get_target()}")
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

#### compile_to_result(source, opt_level=0, module_name=None, target=None)

Compiles Vais source code and returns a CompileResult (no exceptions).

**Parameters:**
- `source` (str): The Vais source code
- `opt_level` (int, optional): Optimization level 0-3 (default: 0)
- `module_name` (str, optional): Name of the module (default: "main")
- `target` (str, optional): Target triple (default: native)

**Returns:** CompileResult - Compilation result with IR and errors

#### compile_and_run(source, opt_level=0)

Compiles and executes Vais source code.

**Parameters:**
- `source` (str): The Vais source code
- `opt_level` (int, optional): Optimization level 0-3 (default: 0)

**Returns:** RunResult - Execution result

**Note:** JIT execution is not yet fully implemented. Returns NotImplemented error.

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

#### VaisCompiler

Stateful compiler instance with configurable settings.

**Constructor:**
```python
VaisCompiler(opt_level=0, module_name=None, target=None)
```

**Methods:**
- `compile(source)` - Compile to CompileResult
- `compile_ir(source)` - Compile to IR string (raises on error)
- `run(source)` - Compile and execute
- `tokenize(source)` - Tokenize source
- `parse(source)` - Parse source to AST
- `check(source)` - Type check source
- `set_opt_level(level)` - Set optimization level
- `set_module_name(name)` - Set module name
- `set_target(target)` - Set target triple

**Properties:**
- `opt_level` (int, read-only): Current optimization level
- `module_name` (str, read-only): Current module name
- `target` (str | None, read-only): Current target triple

#### CompileResult

Result of a compilation operation.

**Attributes:**
- `success` (bool): Whether compilation succeeded
- `ir` (str | None): The compiled LLVM IR (if successful)
- `errors` (List[Error]): List of errors
- `warnings` (List[str]): List of warnings

**Methods:**
- `get_ir()` - Get IR or raise if compilation failed

#### RunResult

Result of running compiled code.

**Attributes:**
- `success` (bool): Whether execution succeeded
- `exit_code` (int | None): Exit code
- `stdout` (str): Standard output
- `stderr` (str): Standard error
- `errors` (List[Error]): List of errors

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
