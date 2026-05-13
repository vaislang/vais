# vais-lz4

Pure Vais implementation of LZ4 compression and decompression.

## Overview

This package provides a complete implementation of the LZ4 block and frame formats in pure Vais, with no external dependencies. LZ4 is a lossless compression algorithm focused on fast compression and decompression speeds.

## Features

- **LZ4 Block Format**: Core compression/decompression
- **LZ4 Frame Format**: Framed format with magic number and checksums
- **Pure Vais**: No C dependencies, fully implemented in Vais
- **Hash-based matching**: Fast pattern detection using multiplicative hashing
- **Comprehensive tests**: 5 test cases covering edge cases and compression scenarios

## API

### Types

```vais
S Lz4Result {
    data: i64,      # Pointer to output data (caller must free)
    len: i64,       # Length of output data
    status: i64     # 0 = OK, negative = error
}
```

### Block Compression

```vais
F lz4_compress(src: i64, src_len: i64) -> Lz4Result
```

Compresses data using LZ4 block format. Returns compressed data in `Lz4Result.data` (caller must free).

### Block Decompression

```vais
F lz4_decompress(src: i64, src_len: i64, original_len: i64) -> Lz4Result
```

Decompresses LZ4 block format data. Requires knowing the original uncompressed size.

### Frame Format

```vais
F lz4_frame_compress(src: i64, src_len: i64) -> Lz4Result
F lz4_frame_decompress(src: i64, src_len: i64) -> Lz4Result
```

LZ4 frame format with magic number (0x184D2204) and frame descriptor.

### Convenience Functions

```vais
F lz4_compress_str(s: str) -> Lz4Result
```

Compresses a string directly.

## Usage Example

```vais
U vais-lz4

F main() -> i64 {
    # Compress data
    input := str_to_ptr("Hello, World! This is some data to compress.")
    input_len := strlen(input)

    compressed := lz4_compress(input, input_len)
    I compressed.status == 0 {
        # Compression succeeded
        puts("Compressed size: ")
        # ... print compressed.len

        # Decompress
        decompressed := lz4_decompress(compressed.data, compressed.len, input_len)
        I decompressed.status == 0 {
            # Decompression succeeded
            # ... use decompressed.data
            free(decompressed.data)
        }

        free(compressed.data)
    }

    R 0
}
```

## Algorithm Details

### Compression

1. **Hash Table**: Uses 65,536-entry hash table for fast pattern matching
2. **Matching**: Minimum match length is 4 bytes, using Knuth's multiplicative hash
3. **Encoding**: Sequences of (literals, match) pairs with token bytes
4. **Token Format**: High 4 bits = literal length, low 4 bits = match length - 4
5. **Extended Lengths**: For lengths ≥ 15, additional bytes encode remaining length

### Decompression

1. **Token Parsing**: Extract literal and match lengths from token byte
2. **Literal Copy**: Copy literal bytes directly to output
3. **Match Copy**: Copy from earlier position using 2-byte offset
4. **Overlap Handling**: Byte-by-byte copy handles overlapping matches correctly

### Constants

- `LZ4_HASH_SIZE`: 65,536 (2^16) hash table entries
- `LZ4_MIN_MATCH`: 4 bytes minimum match length
- `LZ4_MF_LIMIT`: 12 bytes (minimum match + safety margin)
- `LZ4_MAGIC`: 0x184D2204 (407710212) for frame format

## Testing

Run the test suite:

```bash
cargo run --bin vaisc -- packages/vais-lz4/tests/test_lz4.vais
./test_lz4  # Run the compiled test binary
```

Tests include:
- Empty input handling
- Simple roundtrip compression/decompression
- Highly compressible repeated data
- Poorly compressible random-like data
- Frame format magic number verification

## Performance Characteristics

- **Compression Speed**: Fast hash-based matching (single pass)
- **Decompression Speed**: Very fast (simple token parsing and copying)
- **Compression Ratio**: Good for repeated patterns, poor for random data
- **Memory Usage**: Hash table (512 KB) + output buffer (worst case: input_size * 1.004)

## Implementation Notes

- All integers in Vais are `i64` (64-bit signed)
- Uses bitmasking to keep values in valid byte ranges
- Hash function: `(value * 2654435761) >> 16 & 0xFFFF`
- Handles match copy overlaps correctly for RLE-style patterns
- No hex literals - all constants in decimal

## License

MIT License - see package manifest for details.

## References

- [LZ4 Format Description](https://github.com/lz4/lz4/blob/dev/doc/lz4_Block_format.md)
- [LZ4 Frame Format](https://github.com/lz4/lz4/blob/dev/doc/lz4_Frame_format.md)
