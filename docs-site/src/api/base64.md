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
| `base64_encode_str` | `F base64_encode_str(str: i64) -> String` | Encode C string |
| `base64_encode_vec` | `F base64_encode_vec(vec: Vec) -> String` | Encode Vec to Base64 |
| `base64_decode` | `F base64_decode(s: String) -> Vec` | Decode from Base64 |
| `base64_decode_str` | `F base64_decode_str(s: String) -> i64` | Decode to C string |
| `base64_decode_cstr` | `F base64_decode_cstr(cstr: i64) -> Vec` | Decode C string |
| `is_base64_char` | `F is_base64_char(c: i64) -> i64` | Check if char is valid base64 |
| `is_base64` | `F is_base64(s: String) -> i64` | Check if string is valid base64 |
| `base64_decoded_len` | `F base64_decoded_len(encoded_len: i64) -> i64` | Calculate decoded length |
| `base64_encoded_len` | `F base64_encoded_len(data_len: i64) -> i64` | Calculate encoded length |

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
