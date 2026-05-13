# vais-crc32

Fast CRC32 implementation for Vais with lookup tables for IEEE and Castagnoli polynomials.

## Features

- **CRC32 IEEE 802.3** (polynomial 0xEDB88320) - Used in ZIP, PNG, Ethernet
- **CRC32C Castagnoli** (polynomial 0x82F63B78) - Used in iSCSI, Btrfs, ext4
- **Lookup table based** - 10-100x faster than bitwise implementations
- **Incremental computation** - Update CRC with multiple data chunks
- **Zero dependencies** - Pure Vais implementation

## Installation

Add to your `vais.toml`:

```toml
[dependencies]
vais-crc32 = "1.0.0"
```

## Usage

### One-shot CRC32

```vais
# CRC32 IEEE
data := malloc(1024)
# ... fill data ...
checksum := crc32(data, 1024)
free(data)
```

### Incremental CRC32

```vais
table := crc32_make_table()
state := mut 4294967295  # Initial value

# Process data in chunks
state = crc32_update(table, state, chunk1, len1)
state = crc32_update(table, state, chunk2, len2)
state = crc32_update(table, state, chunk3, len3)

result := crc32_finalize(state)
free(table)
```

### CRC32C (Castagnoli)

```vais
# One-shot
checksum := crc32c(data, len)

# Incremental
table := crc32c_make_table()
state := mut 4294967295
state = crc32c_update(table, state, data, len)
result := crc32c_finalize(state)
free(table)
```

## API Reference

### CRC32 IEEE

- `crc32_make_table() -> i64` - Generate lookup table (call once, reuse for multiple checksums)
- `crc32_update(table: i64, state: i64, data: i64, len: i64) -> i64` - Update CRC state
- `crc32_finalize(state: i64) -> i64` - Finalize CRC (XOR with 0xFFFFFFFF)
- `crc32(data: i64, len: i64) -> i64` - One-shot CRC32 computation

### CRC32C Castagnoli

- `crc32c_make_table() -> i64` - Generate CRC32C lookup table
- `crc32c_update(table: i64, state: i64, data: i64, len: i64) -> i64` - Update CRC32C state
- `crc32c_finalize(state: i64) -> i64` - Finalize CRC32C
- `crc32c(data: i64, len: i64) -> i64` - One-shot CRC32C computation

## Test Vectors

The implementation passes standard CRC32 test vectors:

- CRC32 IEEE("123456789") = 0xCBF43926 (3421780262)
- CRC32C("123456789") = 0xE3069283 (3808858755)
- Empty string CRC32 = 0x00000000

## Performance

Lookup table implementation is 10-100x faster than bitwise:

- Bitwise: ~1-5 MB/s
- Lookup table: ~100-500 MB/s

## License

MIT

## Related

- std/crc32.vais - Bitwise CRC32 implementation (slower, no dependencies)
