# Memory API Reference

> Low-level memory operations (memset, memcpy, memmove, memcmp)

## Import

```vais
U std/memory
```

## Functions

### Fill Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_set` | `F mem_set(dest: i64, value: i64, n: i64) -> i64` | Fill with byte value |
| `mem_zero` | `F mem_zero(dest: i64, n: i64) -> i64` | Fill with zeros |
| `mem_fill_i64` | `F mem_fill_i64(dest: i64, value: i64, count: i64) -> i64` | Fill with i64 pattern |

### Copy Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_copy` | `F mem_copy(dest: i64, src: i64, n: i64) -> i64` | Copy bytes (non-overlapping) |
| `mem_move` | `F mem_move(dest: i64, src: i64, n: i64) -> i64` | Copy bytes (overlapping safe) |
| `mem_copy_i64` | `F mem_copy_i64(dest: i64, src: i64, count: i64) -> i64` | Copy i64 values |

### Comparison Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_cmp` | `F mem_cmp(s1: i64, s2: i64, n: i64) -> i64` | Compare memory regions |
| `mem_eq` | `F mem_eq(s1: i64, s2: i64, n: i64) -> Bool` | Check equality |
| `mem_chr` | `F mem_chr(ptr: i64, byte: i64, n: i64) -> i64` | Find first byte occurrence |
| `mem_rchr` | `F mem_rchr(ptr: i64, byte: i64, n: i64) -> i64` | Find last byte occurrence |

### Search Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_find` | `F mem_find(haystack: i64, haystack_len: i64, needle: i64, needle_len: i64) -> i64` | Find pattern in memory |

### Allocation Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_alloc_zeroed` | `F mem_alloc_zeroed(size: i64) -> i64` | Allocate and zero-initialize |
| `mem_realloc` | `F mem_realloc(old_ptr: i64, old_size: i64, new_size: i64) -> i64` | Reallocate memory |
| `mem_dup` | `F mem_dup(src: i64, size: i64) -> i64` | Duplicate memory block |

### Pointer Arithmetic

| Function | Signature | Description |
|----------|-----------|-------------|
| `ptr_align_up` | `F ptr_align_up(ptr: i64, alignment: i64) -> i64` | Align pointer up |
| `ptr_align_down` | `F ptr_align_down(ptr: i64, alignment: i64) -> i64` | Align pointer down |
| `ptr_is_aligned` | `F ptr_is_aligned(ptr: i64, alignment: i64) -> Bool` | Check if aligned |
| `ptr_diff` | `F ptr_diff(p1: i64, p2: i64) -> i64` | Calculate pointer distance |
| `ptr_offset` | `F ptr_offset(ptr: i64, offset: i64) -> i64` | Offset pointer by bytes |
| `ptr_offset_i64` | `F ptr_offset_i64(ptr: i64, count: i64) -> i64` | Offset by i64 elements |

### Byte Swap (Endianness)

| Function | Signature | Description |
|----------|-----------|-------------|
| `bswap16` | `F bswap16(x: i64) -> i64` | Swap bytes of 16-bit value |
| `bswap32` | `F bswap32(x: i64) -> i64` | Swap bytes of 32-bit value |
| `bswap64` | `F bswap64(x: i64) -> i64` | Swap bytes of 64-bit value |

### Bit Manipulation

| Function | Signature | Description |
|----------|-----------|-------------|
| `clz64` | `F clz64(x: i64) -> i64` | Count leading zeros |
| `ctz64` | `F ctz64(x: i64) -> i64` | Count trailing zeros |
| `popcount64` | `F popcount64(x: i64) -> i64` | Count set bits |
| `is_power_of_2` | `F is_power_of_2(x: i64) -> Bool` | Check if power of 2 |
| `next_power_of_2` | `F next_power_of_2(x: i64) -> i64` | Round up to power of 2 |

## Usage

```vais
U std/memory

F main() -> i64 {
    # Allocation
    buf := mem_alloc_zeroed(256)

    # Fill and copy
    mem_set(buf, 65, 10)  # Fill first 10 bytes with 'A'
    buf2 := mem_dup(buf, 256)

    # Search
    pos := mem_chr(buf, 65, 256)  # Find 'A'

    # Comparison
    I mem_eq(buf, buf2, 256) {
        puts("Equal")
    }

    # Endianness
    le_val := bswap32(0x12345678)  # Convert endianness

    # Bit operations
    zeros := clz64(0xFF)
    I is_power_of_2(256) {
        puts("Power of 2")
    }

    free(buf)
    free(buf2)
    0
}
```
