# vais-sha256

FIPS 180-4 SHA-256 cryptographic hash implementation in pure Vais.

## Overview

SHA-256 (Secure Hash Algorithm 256-bit) is a cryptographic hash function that produces a 256-bit (32-byte) hash value. This implementation follows the NIST FIPS 180-4 specification exactly.

## Features

- Pure Vais implementation (no external dependencies)
- FIPS 180-4 compliant
- Incremental hashing (streaming API)
- Hexadecimal output conversion
- Comprehensive test suite with NIST test vectors

## API

### Core Functions

```vais
F sha256_init() -> i64
```
Allocate and initialize SHA-256 state. Returns pointer to state.

```vais
F sha256_update(state: i64, data: i64, len: i64) -> i64
```
Process data incrementally. Can be called multiple times to hash data in chunks.

```vais
F sha256_finalize(state: i64) -> i64
```
Finalize hash computation. Returns pointer to 32-byte hash. Frees state internally.

```vais
F sha256_hash(data: i64, len: i64) -> i64
```
Convenience function: init + update + finalize in one call. Returns 32-byte hash pointer.

```vais
F sha256_hex(hash: i64) -> i64
```
Convert 32-byte hash to 64-character hexadecimal string (null-terminated).

### Example Usage

```vais
# Simple one-shot hashing
data := str_to_ptr("hello world")
hash := sha256_hash(data, 11)
hex := sha256_hex(hash)
# hex now points to "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"

# Incremental hashing
state := sha256_init()
sha256_update(state, str_to_ptr("hello"), 5)
sha256_update(state, str_to_ptr(" world"), 6)
hash := sha256_finalize(state)
# Same result as above
```

## Implementation Details

- Uses i64 type to simulate 32-bit unsigned integers (masked with & 4294967295)
- Right rotation implemented as `rotr32(x, n) = ((x >> n) | (x << (32 - n))) & 0xFFFFFFFF`
- Follows SHA-256 specification exactly:
  - 64 rounds per block
  - 64 round constants (K[0..63])
  - 8 initial hash values (H0-H7)
  - Message schedule expansion (W[0..63])
  - Padding: append 0x80, pad to 56 bytes, append 64-bit length

## Test Vectors

All tests pass against known SHA-256 test vectors:

- Empty string: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- "abc": `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`
- "hello": `2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`
- "The quick brown fox jumps over the lazy dog": `d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592`

## Security Notes

- This is a correct implementation of SHA-256 for educational and general purposes
- Not optimized for constant-time operation (not side-channel resistant)
- For production cryptographic use, consider hardware-accelerated implementations

## License

MIT
