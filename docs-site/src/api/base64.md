# Base64 API Reference

> Base64 encoding and decoding (RFC 4648)

## Import

```vais
U std/base64
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `base64_encode_ex` | `F base64_encode_ex(data: i64, len: i64, url_safe: i64) -> String` | Encode with options |
| `base64_encode` | `F base64_encode(data: i64, len: i64) -> String` | Standard Base64 encode |
| `base64_encode_url` | `F base64_encode_url(data: i64, len: i64) -> String` | URL-safe encode |
| `base64_decode` | `F base64_decode(s: String) -> i64` | Decode from Base64 |

## Constants

| Name | Description |
|------|-------------|
| `BASE64_ALPHABET` | Standard alphabet (A-Z, a-z, 0-9, +/) |
| `BASE64_URL_ALPHABET` | URL-safe alphabet (A-Z, a-z, 0-9, -_) |
| `BASE64_PAD` | Padding character (`=`, ASCII 61) |

## Usage

```vais
U std/base64

F main() -> i64 {
    encoded := base64_encode("Hello", 5)
    # encoded = "SGVsbG8="
    encoded.print()
    encoded.drop()
    0
}
```
