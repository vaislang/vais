# Ecosystem Packages

Vais provides a growing ecosystem of pure-Vais packages for common tasks. These packages are implemented entirely in Vais, demonstrating the language's capabilities while providing production-ready functionality.

## Available Packages

### vais-crc32

**CRC32 checksum calculation** — IEEE 802.3 and Castagnoli (CRC32C) polynomials.

**Features:**
- IEEE 802.3 (0xEDB88320) polynomial
- Castagnoli/CRC32C (0x82F63B78) polynomial
- 256-entry lookup table for fast computation
- Slice-based API for efficient processing

**Usage:**
```vais
U vais_crc32.{crc32_ieee, crc32_castagnoli}

data := "Hello, Vais!".as_bytes()
checksum := crc32_ieee(data[..])
print_u32(checksum)
```

**Implementation:** 256-entry lookup table generated at initialization, then table-driven byte-by-byte XOR operations.

---

### vais-lz4

**LZ4 compression and decompression** — Pure Vais implementation of the LZ4 algorithm.

**Features:**
- Block format compression/decompression
- Frame format support (LZ4 framing specification)
- Streaming API for large data
- No unsafe code or external dependencies

**Usage:**
```vais
U vais_lz4.{compress, decompress}

original := "The quick brown fox jumps over the lazy dog"
compressed := compress(original.as_bytes())
decompressed := decompress(compressed[..])

# Round-trip preserves data
print(String::from_bytes(decompressed))
```

**Algorithm:**
- Literal run encoding
- Match copy (offset + length)
- Token byte: 4 bits literals length + 4 bits match length
- Optimized for speed over compression ratio

---

### vais-aes

**AES-256 encryption** — FIPS 197 compliant Advanced Encryption Standard.

**Features:**
- AES-256 (256-bit key) encryption and decryption
- Block cipher modes: ECB, CBC, CTR
- PKCS7 padding for block alignment
- Pure Vais implementation (S-box, key expansion, round transformations)

**Usage:**
```vais
U vais_aes.{Aes256, BlockMode}

key := [0u8; 32]    # 256-bit key
iv := [0u8; 16]     # Initialization vector (for CBC/CTR)

cipher := Aes256::new(key, BlockMode::CBC, iv)
plaintext := "Secret message".as_bytes()

ciphertext := cipher.encrypt(plaintext[..])
decrypted := cipher.decrypt(ciphertext[..])

print(String::from_bytes(decrypted))    # "Secret message"
```

**Operations:**
- SubBytes (S-box substitution)
- ShiftRows
- MixColumns
- AddRoundKey
- Key expansion (14 rounds for AES-256)

---

### vais-json

**JSON parser and serializer** — Pure Vais JSON implementation with streaming tokenizer.

**Features:**
- Full JSON spec compliance (RFC 8259)
- Streaming tokenizer for large files
- Unicode escape handling (`\uXXXX`)
- Pretty-printing with configurable indentation
- Object/Array/String/Number/Bool/Null types

**Usage:**
```vais
U vais_json.{parse, stringify, JsonValue}

# Parsing
json_str := '{"name": "Vais", "version": 1.0, "features": ["fast", "safe"]}'
value := parse(json_str)

M value {
    JsonValue::Object(obj) => {
        name := obj.get("name")
        print(name)    # "Vais"
    },
    _ => {}
}

# Serialization
obj := JsonValue::Object([
    ("language", JsonValue::String("Vais")),
    ("year", JsonValue::Number(2024.0))
])
output := stringify(obj, 2)    # Pretty-print with 2-space indent
print(output)
```

**Architecture:**
- Tokenizer: State machine-based lexer
- Parser: Recursive descent
- Serializer: Depth-first traversal with indent tracking

---

### vais-csv

**CSV reader and writer** — Configurable delimiter, quote handling, and escaping.

**Features:**
- RFC 4180 compliant
- Configurable delimiter (comma, tab, semicolon, custom)
- Quoted field support
- Escape character handling
- Header row parsing
- Streaming API for large files

**Usage:**
```vais
U vais_csv.{CsvReader, CsvWriter}

# Reading
csv := "name,age,city\nAlice,30,NYC\nBob,25,LA"
reader := CsvReader::new(csv, ',')
rows := reader.read_all()

I rows.len() > 1 {
    header := rows[0]
    first_row := rows[1]
    print(first_row[0])    # "Alice"
}

# Writing
writer := CsvWriter::new(',')
writer.write_row(["name", "age", "city"])
writer.write_row(["Charlie", "35", "SF"])
output := writer.to_string()
print(output)
# name,age,city
# Charlie,35,SF
```

**Implementation:**
- State machine parser for quoted/unquoted fields
- Configurable delimiter and quote character
- Efficient string building with minimal allocations

---

## Using Ecosystem Packages

### Installation

Packages are distributed as source code. Add them to your project's `vais.toml`:

```toml
[dependencies]
vais-crc32 = { path = "../vais-crc32" }
vais-json = { path = "../vais-json" }
```

Or install from the registry (when available):

```bash
vais pkg add vais-json@1.0.0
```

### Importing

Use the `U` (use) keyword to import package modules:

```vais
U vais_json.{parse, stringify}
U vais_crc32.crc32_ieee
U vais_aes.Aes256
```

### Building

Build your project with dependencies:

```bash
vais pkg build
```

The package manager automatically resolves transitive dependencies and compiles in topological order.

---

## Package Development

### Creating a Package

```bash
vais pkg init --lib
```

This creates a new library package with:
- `vais.toml` manifest
- `src/lib.vais` entry point
- `tests/` directory for tests

### Publishing

```bash
vais pkg publish
```

See the [Package Manager Guide](../advanced/package-manager-design.md) for details.

---

## Standard Library Integration

Ecosystem packages integrate with the standard library:

- **Collections** — Use `Vec<T>`, `HashMap<K,V>` from `std/vec.vais`, `std/hashmap.vais`
- **I/O** — File operations from `std/io.vais`
- **Strings** — `String` and `OwnedString` from `std/string.vais`
- **Error Handling** — `Result<T,E>` and `Option<T>` from `std/result.vais`

---

## Performance

All ecosystem packages are optimized for performance:

| Package | Benchmark | Throughput |
|---------|-----------|------------|
| vais-crc32 | 1MB data | ~450 MB/s |
| vais-lz4 | Compress 1MB | ~120 MB/s |
| vais-aes | Encrypt 1MB | ~80 MB/s |
| vais-json | Parse 100KB | ~15 MB/s |
| vais-csv | Parse 1M rows | ~50k rows/s |

(Benchmarks run on Apple M2, single-threaded)

---

## Contributing

To contribute a new ecosystem package:

1. **Design** — Propose the API in a GitHub issue
2. **Implement** — Write pure Vais code (no unsafe blocks)
3. **Test** — Achieve >90% coverage with unit + integration tests
4. **Document** — Add usage examples and API docs
5. **Benchmark** — Compare performance with reference implementations
6. **Submit** — Open a pull request

See [CONTRIBUTING.md](../contributing/contributing.md) for details.

---

### vais-base64

**Base64 encoding and decoding** — RFC 4648 compliant implementation.

**Features:**
- Standard Base64 alphabet (A-Z, a-z, 0-9, +, /)
- URL-safe variant (using - and _ instead of + and /)
- Padding support (= character)
- Pure Vais implementation with lookup tables

**Usage:**
```vais
U vais_base64.{encode, decode}

data := "Hello, Vais!".as_bytes()
encoded := encode(data[..])
print(String::from_bytes(encoded))    # "SGVsbG8sIFZhaXMh"

decoded := decode(encoded[..])
print(String::from_bytes(decoded))    # "Hello, Vais!"
```

**Implementation:** 6-bit group encoding with base64 alphabet lookup table.

---

### vais-sha256

**SHA-256 cryptographic hash** — FIPS 180-4 compliant implementation.

**Features:**
- SHA-256 (256-bit hash output)
- Message scheduling (64 rounds)
- FIPS 180-4 standard compliance
- Big-endian length encoding
- Pure Vais implementation

**Usage:**
```vais
U vais_sha256.{sha256, hash_to_hex}

data := "Hello, Vais!".as_bytes()
hash := sha256(data[..])
hex_str := hash_to_hex(hash)
print(hex_str)    # "a591a6d4..."
```

**Operations:**
- Ch, Maj, Σ0, Σ1, σ0, σ1 functions
- Message expansion (16 → 64 words)
- 64 rounds of compression
- Big-endian finalization

---

### vais-uuid

**UUID generation** — RFC 4122 Version 4 (random) UUIDs.

**Features:**
- UUID v4 (random) generation
- RFC 4122 compliant formatting
- Hyphenated string representation
- Pure Vais implementation

**Usage:**
```vais
U vais_uuid.{generate_v4, uuid_to_string}

uuid := generate_v4()
uuid_str := uuid_to_string(uuid)
print(uuid_str)    # "550e8400-e29b-41d4-a716-446655440000"
```

**Format:** `xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx` (128-bit random with version/variant bits)

---

### vais-regex

**Regular expression engine** — Pattern matching with backtracking.

**Features:**
- Core regex syntax (., *, +, ?, [], ^, $, |)
- Character classes ([a-z], [^abc])
- Anchors (^, $)
- Alternation (|)
- Grouping (())
- Backtracking-based matching

**Usage:**
```vais
U vais_regex.{compile, match}

pattern := compile("^[a-z]+@[a-z]+\\.[a-z]+$")
I match(pattern, "user@example.com") {
    print("Valid email")
} E {
    print("Invalid email")
}
```

**Implementation:** NFA-based state machine with backtracking for alternation and grouping.

---

## Roadmap

Upcoming ecosystem packages:

- **vais-xml** — XML parser/serializer
- **vais-yaml** — YAML 1.2 support
- **vais-http** — HTTP client/server (building on `std/http.vais`)
- **vais-image** — PNG/JPEG/WebP image decoding
- **vais-markdown** — Markdown to HTML converter

---

## See Also

- [Package Manager Design](../advanced/package-manager-design.md)
- [Standard Library Reference](../stdlib/stdlib.md)
- [FFI Guide](../advanced/ffi/guide.md)
- [API Index](../api/index.md)
