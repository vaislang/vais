# Compress API Reference

> Gzip/deflate compression and decompression built on zlib FFI

## Overview

The Compress module provides RFC-compliant compression with:
- Deflate compression/decompression (raw RFC 1951)
- Gzip compression/decompression (RFC 1952 with header + CRC32)
- Streaming compression (chunk-by-chunk processing)
- Multiple compression levels (fast/default/best)
- HTTP Content-Encoding integration helpers

## Constants

### Status Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `COMPRESS_OK` | 0 | Success |
| `COMPRESS_ERR_INIT` | -1 | Initialization failed |
| `COMPRESS_ERR_PARAM` | -2 | Invalid parameter |
| `COMPRESS_ERR_MEMORY` | -3 | Memory allocation failed |
| `COMPRESS_ERR_DATA` | -4 | Invalid or corrupted data |
| `COMPRESS_ERR_STREAM` | -5 | Stream error |
| `COMPRESS_ERR_BUFFER` | -7 | Buffer error |

### Compression Modes

| Constant | Value | Description |
|----------|-------|-------------|
| `COMPRESS_DEFLATE` | 0 | Raw deflate (RFC 1951) |
| `COMPRESS_GZIP` | 1 | Gzip format (RFC 1952) |

### Compression Levels

| Constant | Value | Description |
|----------|-------|-------------|
| `COMPRESS_LEVEL_FAST` | 1 | Fast compression |
| `COMPRESS_LEVEL_DEFAULT` | 6 | Default compression |
| `COMPRESS_LEVEL_BEST` | 9 | Best compression |

### Buffer Sizes

| Constant | Value | Description |
|----------|-------|-------------|
| `COMPRESS_CHUNK_SIZE` | 16384 | 16KB chunk size |
| `COMPRESS_MAX_OUTPUT` | 10485760 | 10MB max output |

## Structures

### CompressResult

```vais
S CompressResult {
    data_ptr: i64,      # Pointer to output data (caller must free)
    data_len: i64,      # Length of output data
    status: i64         # COMPRESS_OK or error code
}
```

Result structure returned by compression/decompression operations.

**Methods:**
- `is_ok(&self) -> i64`: Check if operation was successful
- `free(&self) -> i64`: Free the result data

### Compressor

```vais
S Compressor {
    handle: i64,        # Opaque pointer to z_stream
    mode: i64,          # COMPRESS_DEFLATE or COMPRESS_GZIP
    level: i64,         # Compression level (1-9)
    streaming: i64      # 1 if streaming mode is active
}
```

Stateful compressor for compression operations.

## Compressor Methods

### deflate / gzip

```vais
F deflate(level: i64) -> Compressor
F gzip(level: i64) -> Compressor
```

Create a new deflate or gzip compressor with the specified compression level.

**Parameters:**
- `level`: Compression level (1-9, automatically clamped)

**Returns:** New compressor instance

---

### is_valid

```vais
F is_valid(&self) -> i64
```

Check if compressor was created successfully.

**Returns:** `1` if valid, `0` otherwise

---

### compress

```vais
F compress(&self, data_ptr: i64, data_len: i64) -> CompressResult
```

Perform one-shot compression (entire input at once).

**Parameters:**
- `data_ptr`: Pointer to input data
- `data_len`: Length of input data

**Returns:** CompressResult with compressed data

---

### decompress

```vais
F decompress(&self, data_ptr: i64, data_len: i64) -> CompressResult
```

Perform one-shot decompression (entire input at once).

**Parameters:**
- `data_ptr`: Pointer to compressed data
- `data_len`: Length of compressed data

**Returns:** CompressResult with decompressed data

---

### stream_begin

```vais
F stream_begin(&self) -> i64
```

Begin streaming compression.

**Returns:** `0` on success, error code on failure

---

### stream_write

```vais
F stream_write(&self, chunk_ptr: i64, chunk_len: i64) -> CompressResult
```

Write a chunk to the compression stream.

**Parameters:**
- `chunk_ptr`: Pointer to chunk data
- `chunk_len`: Length of chunk

**Returns:** CompressResult with compressed chunk

---

### stream_finish

```vais
F stream_finish(&self) -> CompressResult
```

Finish streaming compression and get final chunk.

**Returns:** CompressResult with final compressed data

---

### free

```vais
F free(&self) -> i64
```

Free compressor resources.

**Returns:** `0`

## Convenience Functions

### gzip_compress

```vais
F gzip_compress(data_ptr: i64, data_len: i64) -> CompressResult
```

One-shot gzip compression (for HTTP Content-Encoding: gzip).

---

### gzip_decompress

```vais
F gzip_decompress(data_ptr: i64, data_len: i64) -> CompressResult
```

One-shot gzip decompression (for HTTP Content-Encoding: gzip).

---

### deflate_compress

```vais
F deflate_compress(data_ptr: i64, data_len: i64) -> CompressResult
```

One-shot deflate compression (raw deflate).

---

### deflate_decompress

```vais
F deflate_decompress(data_ptr: i64, data_len: i64) -> CompressResult
```

One-shot deflate decompression (raw deflate).

---

### compress_error_text

```vais
F compress_error_text(code: i64) -> str
```

Get human-readable error description for an error code.

## Usage Examples

### One-Shot Gzip Compression

```vais
data := "Hello, World!"
result := gzip_compress(data as i64, __strlen(data))

I result.is_ok() == 1 {
    # result.data_ptr and result.data_len contain compressed data
    # Use compressed data...

    # Free when done
    __free(result.data_ptr)
}
```

### One-Shot Decompression

```vais
result := gzip_decompress(compressed_ptr, compressed_len)

I result.is_ok() == 1 {
    # result.data_ptr and result.data_len contain decompressed data
    __free(result.data_ptr)
} E {
    error := compress_error_text(result.status)
    log_error(error)
}
```

### Streaming Compression

```vais
compressor := Compressor::deflate(COMPRESS_LEVEL_DEFAULT)

I compressor.is_valid() == 1 {
    compressor.stream_begin()

    # Write chunks
    result1 := compressor.stream_write(chunk1_ptr, chunk1_len)
    I result1.is_ok() == 1 {
        # Process result1.data_ptr, result1.data_len
        __free(result1.data_ptr)
    }

    result2 := compressor.stream_write(chunk2_ptr, chunk2_len)
    I result2.is_ok() == 1 {
        # Process result2.data_ptr, result2.data_len
        __free(result2.data_ptr)
    }

    # Finish
    final := compressor.stream_finish()
    I final.is_ok() == 1 {
        # Process final.data_ptr, final.data_len
        __free(final.data_ptr)
    }

    compressor.free()
}
```

### HTTP Content-Encoding

```vais
# Compress response body for HTTP
body := "Large JSON response data..."
result := gzip_compress(body as i64, __strlen(body))

I result.is_ok() == 1 {
    response := Response::ok()
        .with_body(result.data_ptr, result.data_len)
        .header("Content-Encoding", "gzip")

    # Send response...

    __free(result.data_ptr)
}
```

### Custom Compression Level

```vais
# Fast compression for speed
compressor := Compressor::gzip(COMPRESS_LEVEL_FAST)
result := compressor.compress(data_ptr, data_len)
compressor.free()

# Best compression for size
compressor2 := Compressor::gzip(COMPRESS_LEVEL_BEST)
result2 := compressor2.compress(data_ptr, data_len)
compressor2.free()
```

## Dependencies

The compress module requires zlib to be installed on the system.

**Link flags:** `-lz`
