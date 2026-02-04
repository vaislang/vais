# Crypto API Reference

> Cryptographic primitives: SHA-256, HMAC-SHA256, AES-256

**Warning:** This is an educational implementation. Do not use in production without formal security review.

## Import

```vais
U std/crypto
```

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `SHA256_BLOCK_SIZE` | 64 | SHA-256 block size (512 bits) |
| `SHA256_DIGEST_SIZE` | 32 | SHA-256 digest size (256 bits) |
| `AES_BLOCK_SIZE` | 16 | AES block size (128 bits) |
| `AES_KEY_SIZE` | 32 | AES-256 key size (256 bits) |
| `AES_ROUNDS` | 14 | Number of AES-256 rounds |

## Structs

### Sha256

```vais
S Sha256 {
    state: i64,      # Pointer to 8 x i64 state array (H0-H7)
    buffer: i64,     # Pointer to 64-byte block buffer
    buf_len: i64,    # Current buffer fill level
    total_len: i64   # Total bytes processed
}
```

SHA-256 hash context for incremental hashing.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Sha256` | Create hasher with initial state (H0-H7) |
| `update` | `F update(&self, data: i64, data_len: i64) -> i64` | Feed data into hasher |
| `process_block` | `F process_block(&self) -> i64` | Process a single 512-bit block (internal) |
| `finalize` | `F finalize(&self) -> i64` | Get digest pointer (32 bytes) |
| `digest_i64` | `F digest_i64(&self) -> i64` | Get first 8 bytes as i64 |
| `cleanup` | `F cleanup(&self) -> i64` | Free allocated resources |

### Hmac

```vais
S Hmac {
    key: i64,           # Pointer to key data
    key_len: i64,
    inner_hasher: i64,  # Inner SHA-256 state pointer
    outer_hasher: i64   # Outer SHA-256 state pointer
}
```

HMAC-SHA256 message authentication code.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(key: i64, key_len: i64) -> Hmac` | Create HMAC with key |
| `compute` | `F compute(&self, data: i64, data_len: i64) -> i64` | Compute MAC (returns pointer to 32-byte MAC) |

### Aes256

```vais
S Aes256 {
    key: i64,           # Pointer to 32-byte key
    round_keys: i64     # Pointer to expanded round keys
}
```

AES-256 block cipher (simplified/educational - uses XOR-based placeholder).

**Note:** This is a simplified implementation using XOR. A real AES-256 would require SubBytes, ShiftRows, MixColumns, and AddRoundKey transformations.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(key: i64) -> Aes256` | Create cipher with 32-byte key and expand round keys |
| `encrypt_block` | `F encrypt_block(&self, block: i64) -> i64` | Encrypt 16-byte block in-place |
| `decrypt_block` | `F decrypt_block(&self, block: i64) -> i64` | Decrypt 16-byte block in-place |
| `cleanup` | `F cleanup(&self) -> i64` | Free and zero out round keys for security |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `sha256` | `F sha256(data: i64, len: i64) -> i64` | One-shot SHA-256 hash (returns first 8 bytes as i64) |
| `hmac_sha256` | `F hmac_sha256(key: i64, key_len: i64, data: i64, data_len: i64) -> i64` | One-shot HMAC-SHA256 (returns first 8 bytes as i64) |

## Usage

### SHA-256 Incremental Hashing

```vais
U std/crypto

F main() -> i64 {
    # Create hasher
    hasher := Sha256::new()

    # Feed data incrementally
    hasher.update("Hello, ", 7)
    hasher.update("world!", 6)

    # Get hash as i64
    hash := hasher.digest_i64()

    # Cleanup
    hasher.cleanup()
    0
}
```

### One-shot Hash

```vais
U std/crypto

F main() -> i64 {
    hash := sha256("hello", 5)
    0
}
```

### HMAC-SHA256

```vais
U std/crypto

F main() -> i64 {
    key := "secret"
    message := "data to authenticate"

    mac := hmac_sha256(key, 6, message, 20)
    0
}
```

### AES-256 Encryption

```vais
U std/crypto

F main() -> i64 {
    # 32-byte key
    key := malloc(32)
    # ... initialize key ...

    cipher := Aes256::new(key)

    # Encrypt 16-byte block
    block := malloc(16)
    # ... initialize block ...

    cipher.encrypt_block(block)

    # Decrypt
    cipher.decrypt_block(block)

    # Cleanup
    cipher.cleanup()
    free(block)
    free(key)
    0
}
```
