# vais-bindgen Design Document

## Overview

`vais-bindgen` is a Rust bindgen-style FFI binding generator for the Vais programming language. It automatically generates Vais FFI bindings from C header files.

## Architecture

### Core Components

1. **Parser** (`src/parser.rs`)
   - Parses C header files
   - Extracts function declarations, structs, enums, and typedefs
   - Handles basic C syntax including pointers, const qualifiers, and variadic functions
   - Uses regex-based parsing for simplicity (not a full C parser)

2. **Generator** (`src/generator.rs`)
   - Converts parsed C declarations to Vais FFI syntax
   - Maps C types to Vais types
   - Generates extern blocks with proper syntax

3. **Config** (`src/config.rs`)
   - Configuration options for binding generation
   - Custom type mappings
   - Allow/blocklists for selective generation
   - Library naming

4. **Bindgen** (`src/lib.rs`)
   - Main API for users
   - Coordinates parsing and generation
   - Provides builder-style API

## Type Mappings

### Primitive Types

| C Type | Vais Type |
|--------|-----------|
| void | () |
| char | i8 |
| short | i16 |
| int | i32 |
| long | i64 |
| unsigned char | u8 |
| unsigned short | u16 |
| unsigned int | u32 |
| unsigned long | u64 |
| float | f32 |
| double | f64 |
| bool | bool |
| size_t | usize |

### Pointer Types

- `T*` → `*mut T`
- `const T*` → `*const T`
- `void*` → `*mut ()`
- `const void*` → `*const ()`

### Complex Types

- **Opaque Structs**: Forward-declared structs become `type Name = *mut ()`
- **Regular Structs**: Fully translated with fields
- **Enums**: C-style enums with explicit values
- **Typedefs**: Type aliases

## Features

### Implemented

- ✅ Function declarations (including variadic)
- ✅ Struct definitions (regular and opaque)
- ✅ Enum definitions with values
- ✅ Typedef declarations
- ✅ Pointer types (mutable and const)
- ✅ Custom type mappings
- ✅ Library naming for extern blocks
- ✅ Preprocessor directive filtering
- ✅ Comment stripping
- ✅ CLI tool

### Not Implemented (Future Work)

- ❌ Full C preprocessor support
- ❌ Macro expansion
- ❌ Function pointers as types
- ❌ Unions
- ❌ Bitfields
- ❌ Array types with explicit sizes
- ❌ Nested struct definitions
- ❌ C++ support
- ❌ Automatic wrapper generation
- ❌ Documentation comment preservation

## Usage Patterns

### Library Usage

```rust
use vais_bindgen::{Bindgen, BindgenConfig};

let mut bindgen = Bindgen::new();
bindgen
    .header("mylib.h")
    .unwrap()
    .configure(|config| {
        config.set_library_name("mylib");
        config.add_type_mapping("size_t", "u64");
    })
    .generate_to_file("bindings.vais")
    .unwrap();
```

### CLI Usage

```bash
vais-bindgen mylib.h -o bindings.vais -l mylib
vais-bindgen -t size_t=u64 mylib.h
```

## Design Decisions

### Why Regex-Based Parsing?

- **Simplicity**: Easier to implement and maintain
- **Good Enough**: Works for most common C headers
- **Fast**: No need for a full parser for simple declarations
- **Trade-off**: Doesn't handle all C syntax edge cases

### Why Not Full C Parser?

- Complex C syntax with preprocessor makes full parsing difficult
- Most FFI bindings need simple declarations
- Users can always preprocess headers first if needed

### Opaque Struct Handling

Opaque structs (forward declarations) are mapped to `*mut ()` to:
- Maintain type safety
- Allow pointer manipulation
- Prevent accidental dereferencing

## Testing Strategy

### Unit Tests

- Individual function testing for parser, generator, and config
- Type mapping verification
- Edge case handling

### Integration Tests

- Complete binding generation from various C headers
- Real-world examples (graphics, database, etc.)
- Error handling scenarios

### Examples

- Simple usage demonstrations
- Complex scenarios (graphics library, advanced types)
- CLI tool examples

## Limitations

1. **Not a Full C Parser**: Cannot handle complex C syntax
2. **No Preprocessor**: Macros and conditionals are ignored
3. **Simple Type System**: Some C types may not map perfectly
4. **No Validation**: Generated code is not validated
5. **Manual Cleanup**: Generated code may need manual adjustment

## Future Enhancements

1. **Better Parser**: Use a proper C parser library (e.g., `lang-c`)
2. **Wrapper Generation**: Generate safe Rust-style wrappers
3. **Documentation**: Preserve C comments as doc comments
4. **Validation**: Validate generated Vais code
5. **Configuration Files**: Support for config files
6. **Templates**: Customizable output templates
7. **Plugin System**: Allow custom type converters

## Performance Considerations

- Regex compilation is cached
- Single-pass parsing where possible
- Minimal allocations during generation
- Streaming output for large headers

## Error Handling

- Parse errors provide context
- Type mapping errors are clear
- I/O errors are propagated properly
- CLI tool provides helpful error messages
