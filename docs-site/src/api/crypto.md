# Crypto API Reference

> Cryptographic primitives: SHA-256, HMAC-SHA256, AES-256

**Warning:** This is an educational implementation. Do not use in production without formal security review.

## Import

```vais
U std/crypto
```

## Structs

### Sha256

SHA-256 hash context for incremental hashing.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Sha256` | Create hasher with initial state |
| `update` | `F update(&self, data: i64, data_len: i64) -> i64` | Feed data |
| `finalize` | `F finalize(&self) -> i64` | Get digest pointer (32 bytes) |
| `digest_i64` | `F digest_i64(&self) -> i64` | Get first 8 bytes as i64 |
| `cleanup` | `F cleanup(&self) -> i64` | Free resources |

### Hmac

HMAC-SHA256 message authentication code.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(key: i64, key_len: i64) -> Hmac` | Create with key |
| `compute` | `F compute(&self, data: i64, data_len: i64) -> i64` | Compute MAC |

### Aes256

AES-256 block cipher (simplified/educational).

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(key: i64) -> Aes256` | Create with 32-byte key |
| `encrypt_block` | `F encrypt_block(&self, block: i64) -> i64` | Encrypt 16-byte block |
| `decrypt_block` | `F decrypt_block(&self, block: i64) -> i64` | Decrypt 16-byte block |
| `cleanup` | `F cleanup(&self) -> i64` | Free and zero keys |

## Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `sha256` | `F sha256(data: i64, len: i64) -> i64` | One-shot SHA-256 hash |
| `hmac_sha256` | `F hmac_sha256(key: i64, key_len: i64, data: i64, data_len: i64) -> i64` | One-shot HMAC |

## Usage

```vais
U std/crypto

F main() -> i64 {
    hash := sha256("hello", 5)
    0
}
```
