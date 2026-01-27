# vais-bindgen

Rust bindgen-style FFI binding generator for the Vais programming language.

## Features

- Parse C header files
- Generate Vais FFI bindings automatically
- Support for:
  - Functions (including variadic)
  - Structs (regular and opaque)
  - Enums
  - Typedefs
  - Pointers (mutable and const)
  - Custom type mappings
  - Library naming

## Usage

### Basic Example

```rust
use vais_bindgen::Bindgen;

let header = r#"
    int add(int a, int b);
    void print_hello(void);
"#;

let mut bindgen = Bindgen::new();
bindgen.parse_header(header).unwrap();
let output = bindgen.generate().unwrap();
```

### With Configuration

```rust
use vais_bindgen::{Bindgen, BindgenConfig};

let mut config = BindgenConfig::default();
config.set_library_name("mylib");
config.add_type_mapping("size_t", "u64");

let mut bindgen = Bindgen::with_config(config);
bindgen.header("mylib.h").unwrap();
bindgen.generate_to_file("bindings.vais").unwrap();
```

### Custom Type Mappings

```rust
bindgen.configure(|config| {
    config.add_type_mapping("my_custom_t", "MyCustomType");
    config.set_library_name("custom");
});
```

## Supported C Types

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
| T* | *mut T |
| const T* | *const T |

## Examples

See the `examples/` directory for complete examples:

- `simple.rs` - Basic math library bindings
- `graphics.rs` - Graphics library with opaque handles
- `advanced.rs` - Complex example with callbacks and variadic functions

## Testing

```bash
cargo test
```

## License

Same as the Vais project.
