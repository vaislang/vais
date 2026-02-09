# vais-aes

Real AES-256 encryption library for Vais with multiple block cipher modes.

## Features

- **Full AES-256 Implementation** (FIPS 197 compliant)
  - SubBytes transformation with S-Box
  - ShiftRows transformation
  - MixColumns with GF(2^8) arithmetic
  - AddRoundKey with 14-round key expansion
- **Block Cipher Modes**
  - ECB (Electronic Codebook)
  - CBC (Cipher Block Chaining)
  - CTR (Counter Mode)
- **PKCS7 Padding** for block alignment
- **Test Vectors** from NIST FIPS 197

## Usage

```vais
U vais_aes

F main() -> i64 {
    # Create 32-byte key
    key := malloc(32)
    i := mut 0
    L i < 32 {
        store_byte(key + i, i & 255)
        i = i + 1
    }

    # Create AES cipher
    aes := Aes256.new(key)

    # Encrypt data with CBC mode
    iv := malloc(16)
    memset(iv, 0, 16)

    plaintext := str_to_ptr("Hello, AES-256!")
    plaintext_len := strlen(plaintext)

    result := aes_cbc_encrypt(&aes, plaintext, plaintext_len, iv)

    I result.status == 0 {
        puts("Encryption successful!")
        putchar(10)
    }

    # Cleanup
    free(result.data)
    aes.cleanup()
    free(key)
    free(iv)

    R 0
}
```

## API

### Core Structure

```vais
S Aes256 {
    round_keys: i64,    # Expanded round keys (240 bytes)
    sbox: i64,          # S-Box (256 bytes)
    inv_sbox: i64       # Inverse S-Box (256 bytes)
}
```

### Methods

- `Aes256.new(key: i64) -> Aes256` - Create cipher with 32-byte key
- `encrypt_block(&self, block: i64) -> i64` - Encrypt 16-byte block in-place
- `decrypt_block(&self, block: i64) -> i64` - Decrypt 16-byte block in-place
- `cleanup(&self) -> i64` - Free all allocated memory

### Block Cipher Modes

```vais
# ECB Mode
F aes_ecb_encrypt(aes: &Aes256, data: i64, data_len: i64) -> AesResult
F aes_ecb_decrypt(aes: &Aes256, data: i64, data_len: i64) -> AesResult

# CBC Mode (requires 16-byte IV)
F aes_cbc_encrypt(aes: &Aes256, data: i64, data_len: i64, iv: i64) -> AesResult
F aes_cbc_decrypt(aes: &Aes256, data: i64, data_len: i64, iv: i64) -> AesResult

# CTR Mode (requires 8-byte nonce)
F aes_ctr_encrypt(aes: &Aes256, data: i64, data_len: i64, nonce: i64) -> AesResult
F aes_ctr_decrypt(aes: &Aes256, data: i64, data_len: i64, nonce: i64) -> AesResult
```

### Result Structure

```vais
S AesResult {
    data: i64,      # Pointer to output data
    len: i64,       # Length of output data
    status: i64     # 0 = OK, -1 = error
}
```

### Padding

```vais
F pkcs7_pad(data: i64, data_len: i64) -> AesResult
F pkcs7_unpad(data: i64, data_len: i64) -> AesResult
```

## Implementation Details

- **S-Box/Inverse S-Box**: Pre-computed lookup tables (256 bytes each)
- **Key Expansion**: AES-256 expands 32-byte key to 15 round keys (240 bytes total)
- **GF(2^8) Arithmetic**: xtime doubling method with irreducible polynomial 0x1b
- **State Layout**: Column-major (byte[row + 4*col])
- **Memory Management**: All buffers allocated with malloc, must be freed by caller

## Testing

Run the test suite:

```bash
cargo run --bin vaisc -- packages/vais-aes/tests/test_aes.vais
```

Tests include:
- S-Box verification
- Key expansion
- FIPS 197 test vectors (encryption/decryption)
- ECB/CBC/CTR mode roundtrips
- PKCS7 padding/unpadding

## Performance

- **Encryption**: ~1,369 LOC of pure Vais implementation
- **No external dependencies** (only standard memory functions)
- **Constant-time operations** where possible (S-Box lookups)

## Security Notes

⚠️ **Educational Implementation**: This is a reference implementation for learning purposes.

For production use, consider:
- Side-channel attack mitigations
- Constant-time GF(2^8) multiplication
- Secure key management
- Authenticated encryption (GCM mode)
- Hardware AES-NI acceleration

## License

MIT

## References

- [FIPS 197: AES Specification](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.197.pdf)
- [NIST Test Vectors](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program)
