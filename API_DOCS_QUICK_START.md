# API Documentation Quick Start

## Overview

Phase 34 Stage 3 adds comprehensive API documentation for Vais standard library modules. This guide shows you how to use and regenerate the documentation.

## Viewing the Documentation

### Option 1: mdBook (Recommended)

```bash
cd docs-site
mdbook serve
```

Then open http://localhost:3000 and navigate to **API Reference** in the sidebar.

### Option 2: Direct File Access

View markdown files directly in your editor or GitHub:
- `/docs-site/src/api/vec.md` - Dynamic arrays
- `/docs-site/src/api/json.md` - JSON parsing/serialization
- `/docs-site/src/api/http.md` - HTTP client/server
- `/docs-site/src/api/log.md` - Structured logging
- `/docs-site/src/api/compress.md` - Gzip/deflate compression

## Generating Documentation

### For a Single File

```bash
cargo run --bin vaisc -- doc std/vec.vais -o docs-site/src/api -f markdown
```

### For All Standard Library Files

```bash
# Generate for all .vais files in std/
cargo run --bin vaisc -- doc std/ -o docs-site/src/api -f markdown
```

### HTML Format

```bash
cargo run --bin vaisc -- doc std/ -o docs/api -f html
```

## What's Documented

Each API reference includes:

1. **Constants** - Named constants with values and descriptions
2. **Structs** - Data structures with field descriptions
3. **Methods** - Functions with parameters, return values, and examples
4. **External Functions** - FFI declarations
5. **Usage Examples** - Working code examples for common use cases

## Example: Using Vec API

```vais
U std/vec

# Create a vector with initial capacity
v := Vec::with_capacity(10)

# Add elements
v.push(1)
v.push(2)
v.push(3)

# Access elements safely with Option
M v.get_opt(0) {
    Some(val) => { print(val) }
    None => { print("Index out of bounds") }
}

# Check length
I v.len() > 0 {
    # Process elements
}

# Clean up
v.drop()
```

## Documentation Structure

```
docs-site/src/
├── SUMMARY.md          # Navigation (includes API Reference section)
└── api/
    ├── vec.md          # Dynamic arrays (Vec<T>)
    ├── json.md         # JSON parsing and serialization
    ├── http.md         # HTTP client and server
    ├── log.md          # Structured logging with spans
    └── compress.md     # Gzip/deflate compression
```

## Adding Documentation to Your Code

The doc generator extracts documentation from comments above declarations:

```vais
# Calculate the factorial of n
# Returns the factorial or 0 for negative inputs
F factorial(n: i64) -> i64 {
    I n < 0 { R 0 }
    I n == 0 { R 1 }
    n * @(n - 1)
}
```

Use `#` comments (not `///`) for Vais source files. The doc generator will:
- Extract comments above declarations
- Skip separator lines (===, ---)
- Generate markdown with proper formatting

## Key Features

- **Auto-extraction**: Parses Vais AST to extract APIs
- **Multiple Formats**: Markdown and HTML output
- **Comprehensive**: Includes all public APIs
- **Examples**: Usage examples for each module
- **Integration**: Part of main mdBook documentation

## Statistics

- 5 core modules documented
- 1,596 total lines of documentation
- 25+ complete code examples
- 12 reference tables
- Average 319 lines per module

## Troubleshooting

### "Cannot find module" error

Make sure you're in the project root directory:
```bash
cd /Users/sswoo/study/projects/vais
```

### Documentation not showing in mdBook

Rebuild the mdBook:
```bash
cd docs-site
mdbook clean
mdbook build
mdbook serve
```

### Want to add more modules?

1. Create the .vais file in `std/`
2. Add doc comments using `#`
3. Generate docs: `cargo run --bin vaisc -- doc std/yourmodule.vais -o docs-site/src/api -f markdown`
4. Add entry to `docs-site/src/SUMMARY.md`

## Next Steps

1. Browse the API documentation at http://localhost:3000
2. Use the examples in your Vais programs
3. Generate docs for additional modules as needed
4. Refer to PHASE34_STAGE3_API_DOCS_SUMMARY.md for implementation details

## Support

For questions or issues:
- Check the main documentation: `docs-site/`
- Review example programs: `examples/`
- See implementation: `crates/vaisc/src/doc_gen.rs`
