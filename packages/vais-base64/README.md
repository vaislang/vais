# vais-base64

RFC 4648 compliant Base64 encoding and decoding implementation for Vais.

## Features

- **Standard Base64** (RFC 4648 Section 4) - A-Z, a-z, 0-9, +, / with = padding
- **Base64URL** (RFC 4648 Section 5) - URL-safe variant using - and _ instead of + and /
- **Zero dependencies** - Pure Vais implementation
- **Binary safe** - Handles null bytes and arbitrary binary data
- **Memory efficient** - Direct pointer-based operations

## Installation

Add to your `vais.toml`:

```toml
[dependencies]
vais-base64 = "1.0.0"
```

## Usage

### Standard Base64 Encoding

```vais
# Encode string data
data := str_to_ptr("Hello, World!")
len := strlen("Hello, World!")
encoded := base64_encode(data, len)
# Result: "SGVsbG8sIFdvcmxkIQ=="
free(encoded)
```

### Standard Base64 Decoding

```vais
# Decode Base64 string
encoded := str_to_ptr("SGVsbG8sIFdvcmxkIQ==")
encoded_len := strlen("SGVsbG8sIFdvcmxkIQ==")

result := base64_decode(encoded, encoded_len)
data_ptr := load_i64(result)      # Decoded data pointer
data_len := load_i64(result + 8)  # Decoded data length

# Use decoded data...

free(data_ptr)
free(result)
```

### URL-Safe Base64 (Base64URL)

```vais
# Encode with URL-safe alphabet
data := str_to_ptr("subjects?_d")
len := strlen("subjects?_d")
encoded := base64url_encode(data, len)
# Uses - and _ instead of + and /
free(encoded)

# Decode URL-safe Base64
result := base64url_decode(encoded_ptr, encoded_len)
data_ptr := load_i64(result)
data_len := load_i64(result + 8)
free(data_ptr)
free(result)
```

## API Reference

### Standard Base64

- `base64_encode(data: i64, len: i64) -> i64` - Encode binary data to Base64 string (null-terminated)
- `base64_decode(encoded: i64, encoded_len: i64) -> i64` - Decode Base64 string, returns struct {data_ptr: i64, data_len: i64}

### Base64URL (URL-safe)

- `base64url_encode(data: i64, len: i64) -> i64` - Encode with URL-safe alphabet
- `base64url_decode(encoded: i64, encoded_len: i64) -> i64` - Decode URL-safe Base64

### Character Mapping Helpers

- `base64_encode_char(idx: i64) -> i64` - Map 6-bit value (0-63) to Base64 character
- `base64url_encode_char(idx: i64) -> i64` - Map 6-bit value to Base64URL character
- `base64_decode_char(c: i64) -> i64` - Map Base64 character to 6-bit value (-1 for invalid)
- `base64url_decode_char(c: i64) -> i64` - Map Base64URL character to 6-bit value

## Memory Management

**Important**: All returned pointers must be freed by the caller.

- `base64_encode()` and `base64url_encode()` return a malloc'd string - call `free()` on it
- `base64_decode()` and `base64url_decode()` return a struct pointer containing a data pointer - free both:
  ```vais
  result := base64_decode(encoded, len)
  data_ptr := load_i64(result)
  free(data_ptr)  # Free decoded data
  free(result)    # Free result struct
  ```

## Test Vectors

Standard Base64 test vectors (RFC 4648):

- `""` → `""`
- `"f"` → `"Zg=="`
- `"fo"` → `"Zm8="`
- `"foo"` → `"Zm9v"`
- `"foob"` → `"Zm9vYg=="`
- `"fooba"` → `"Zm9vYmE="`
- `"foobar"` → `"Zm9vYmFy"`

## License

MIT
