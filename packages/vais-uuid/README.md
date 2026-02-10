# vais-uuid

UUID v4 (random) generation and manipulation library for Vais.

## Features

- UUID v4 generation using LCG (Linear Congruential Generator)
- String formatting to standard `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` format
- UUID comparison
- Nil UUID generation
- Version extraction
- Memory-safe with explicit allocation and deallocation

## Installation

Add to your `vais.toml`:

```toml
[dependencies]
vais-uuid = "1.0.0"
```

## API

### Generation

```vais
F uuid_v4() -> i64
```
Generate a random UUID v4 with default seed. Returns pointer to 16-byte buffer.

```vais
F uuid_v4_with_seed(seed: i64) -> i64
```
Generate a random UUID v4 with custom seed. Useful for deterministic generation.

```vais
F uuid_nil() -> i64
```
Generate a nil UUID (all zeros). Returns pointer to 16-byte buffer.

### Formatting

```vais
F uuid_to_string(uuid: i64) -> i64
```
Convert UUID to string format. Returns pointer to 37-byte buffer (36 chars + null terminator).

### Comparison

```vais
F uuid_compare(a: i64, b: i64) -> i64
```
Compare two UUIDs. Returns 0 if equal, non-zero otherwise.

### Utilities

```vais
F uuid_version(uuid: i64) -> i64
```
Extract version number from UUID bytes (returns 4 for v4 UUIDs).

```vais
F uuid_free(uuid: i64) -> i64
```
Free UUID buffer allocated by library functions.

## Example

```vais
# Generate UUID
uuid := uuid_v4()
str := uuid_to_string(uuid)

# UUID is now formatted as string (e.g., "a1b2c3d4-e5f6-4789-ab12-cd34ef567890")

# Verify version
version := uuid_version(uuid)  # Returns 4

# Compare UUIDs
uuid2 := uuid_v4()
I uuid_compare(uuid, uuid2) == 0 {
    # Very unlikely (collision)
}

# Cleanup
uuid_free(uuid)
uuid_free(uuid2)
free(str)
```

## Implementation Details

- Uses LCG with parameters: a=1664525, c=1013904223, m=2^31-1
- UUID v4: version bits set to `0100`, variant bits to `10xxxxxx`
- All memory allocated via `malloc`, caller responsible for freeing
- 16-byte binary representation internally
- String representation is 36 characters (32 hex + 4 dashes)

## Testing

```bash
cargo run --bin vaisc -- packages/vais-uuid/tests/test_uuid.vais
```

Test suite includes:
- UUID format validation (dashes at correct positions)
- Version bits verification (v4)
- Variant bits verification (RFC 4122)
- Nil UUID generation
- Comparison operations

## License

MIT
