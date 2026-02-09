# Vais Large-Scale Project Generator

This directory contains the large-scale project generator for benchmarking the Vais compiler at scale (10K, 50K, 100K lines).

## Overview

The generator creates synthetic Vais code with diverse constructs to test compiler performance:

- **Functions**: Arithmetic, recursive, conditional, loops, match expressions, ternary operators, struct operations
- **Structs**: Simple data structures with field access
- **Enums**: Pattern matching targets
- **Generic Types**: Generic containers
- **Multi-module Projects**: Cross-module dependencies

## Files

### `gen_large_example.rs`
Example program demonstrating how to use the generator functions. Run with:

```bash
cargo run --example gen_large_example -p vais-benches
```

This generates:
- `generated_10k.vais` - 10,000 line single-file project
- `generated_50k.vais` - 50,000 line single-file project
- `generated_100k.vais` - 100,000 line single-file project
- `multi_module_*.vais` - Multi-module project (10 modules × 1K lines each)
- `distributed_*.vais` - Distributed project (20K lines / 5 modules)

### Generator Functions (in `benches/lib.rs`)

#### `generate_large_project(target_lines: usize) -> String`
Generates a single-file Vais project targeting a specific line count.

**Features:**
- Automatic module distribution based on target size
- 8 different function patterns (arithmetic, recursive, conditional, loop, match, ternary, struct, compute)
- Structs with 3 fields (x, y, z)
- Enums with 3 variants (Ok, Err, Pending)
- Generic container types
- Entry point function (main)

**Example:**
```rust
use vais_benches::utils::generate_large_project;

let code = generate_large_project(10_000);
// Generates ~10K lines of valid Vais code
```

#### `generate_multi_module_project(num_modules: usize, lines_per_module: usize) -> Vec<(String, String)>`
Generates a multi-module project with cross-module dependencies.

**Features:**
- Each module has public structs, enums, and functions
- Modules import previous modules (linear dependency chain)
- Main module imports all others
- Cross-module function calls

**Example:**
```rust
use vais_benches::utils::generate_multi_module_project;

let modules = generate_multi_module_project(5, 1_000);
// Returns Vec<(filename, source_code)>
// Generates: module0.vais, module1.vais, ..., module4.vais, main.vais
```

#### `generate_distributed_project(target_total_lines: usize, num_modules: usize) -> Vec<(String, String)>`
Convenience function to distribute a target line count across multiple modules.

**Example:**
```rust
use vais_benches::utils::generate_distributed_project;

let modules = generate_distributed_project(20_000, 4);
// Generates 4 modules + main, totaling ~20K lines
```

## Generated Code Structure

### Single-File Projects

```vais
# Module 0 — Large-scale benchmark

S Point0_0_2 {
    x: i64,
    y: i64,
    z: i64
}

E Result0_0_32 {
    Ok(i64),
    Err(i64),
    Pending
}

S Container0_0_50<T> {
    value: T,
    count: i64
}

F mod0_arithmetic_0(x: i64, y: i64) -> i64 {
    a := x * 1 + y
    b := a - 1 * x
    c := b + 2 * y
    R a + b + c
}

F mod0_recursive_1(n: i64) -> i64 {
    I n <= 1 {
        R 1
    }
    R n * @(n - 1)
}

F mod0_match_4(x: i64) -> i64 {
    M x {
        4 => x * 2,
        5 => x * 3,
        6 => x * 4,
        _ => x
    }
}

F main() -> i64 {
    result := mod0_arithmetic_0(42, 13)
    R result
}
```

### Multi-Module Projects

**module0.vais:**
```vais
# Module 0 — Part of multi-module benchmark

P S Data0_0 {
    value: i64,
    status: bool
}

P E Status0_0 {
    Active,
    Inactive,
    Error(i64)
}

P F process_0(x: i64) -> i64 {
    result := x * 1 + 0
    I result > 100 {
        R result / 2
    }
    R result
}
```

**module1.vais:**
```vais
# Module 1 — Part of multi-module benchmark

U module0

P F call_previous() -> i64 {
    R module0::process_0(42)
}
```

**main.vais:**
```vais
# Main module — Entry point for multi-module benchmark

U module0
U module1

F main() -> i64 {
    sum := mut 0
    sum = sum + module0::process_0(0)
    sum = sum + module1::process_0(10)
    R sum
}
```

## Function Pattern Distribution

The generator creates 8 different function patterns in round-robin:

1. **Arithmetic** (f % 8 == 0): Multi-step arithmetic operations
2. **Recursive** (f % 8 == 1): Factorial-like recursion using `@` operator
3. **Conditional** (f % 8 == 2): If-else-if chains
4. **Loop** (f % 8 == 3): Mutable accumulator loops with break
5. **Match** (f % 8 == 4): Pattern matching on integers
6. **Ternary** (f % 8 == 5): Chained ternary operators
7. **Struct** (f % 8 == 6): Struct construction and field access
8. **Compute** (f % 8 == 7): Multi-variable computations

## Testing

Run the generator tests:

```bash
cargo test -p vais-benches --test generator_tests
```

Test coverage:
- Basic code generation
- 10K, 50K, 100K line projects parse correctly
- Multi-module projects with cross-references
- Distributed projects
- Syntax validity checks
- Performance benchmarks (generation speed)

## Usage in Benchmarks

The generator is used in `largescale_bench.rs` for compiler performance testing:

```rust
use vais_benches::utils::generate_large_project;
use vais_parser::parse;
use vais_types::TypeChecker;
use vais_codegen::CodeGenerator;

// Benchmark full pipeline
let source = generate_large_project(10_000);
let _tokens = tokenize(&source)?;
let ast = parse(&source)?;
let mut checker = TypeChecker::new();
let _ = checker.check_module(&ast);
let mut codegen = CodeGenerator::new("bench");
codegen.generate_module(&ast)?;
```

## Performance Characteristics

- **Generation speed**: ~10ms for 10K lines, ~50ms for 100K lines
- **Memory**: ~50 bytes per line average
- **Parse time**: Linear scaling (O(n) in line count)
- **Type check time**: Near-linear for generated code (low complexity)

## Vais Syntax Features Used

- Single-character keywords: `F` (function), `S` (struct), `E` (enum), `I` (if), `L` (loop), `M` (match), `R` (return), `B` (break), `P` (pub), `U` (use)
- Variable binding: `:=` and `:= mut`
- Self-recursion: `@(...)`
- Ternary operator: `? :`
- Pattern matching: `M expr { pattern => value, ... }`
- Struct field access: `point.x`
- Module paths: `module0::function()`

## Future Enhancements

Potential additions:
- Trait definitions and implementations
- More complex generic constraints
- Async/await functions
- Macro usage
- FFI declarations
- Attribute annotations
- Doc comments
