//! Phase 80 — MessagePack & Protobuf serialization E2E tests
//!
//! Tests for binary serialization formats:
//! - MessagePack: encode/decode nil, bool, integers, strings, arrays, maps
//! - Protobuf: varint encoding, field encoding, zigzag, message building/parsing

use super::helpers::*;

// ==================== MessagePack Tests ====================

#[test]
fn e2e_p80_msgpack_encode_nil() {
    // Encode nil -> should produce single byte 0xc0 (192)
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F main() -> i64 {
    buf := mp_buf_new()
    # nil = 0xc0 = 192
    mp_buf_write(buf, 192)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 1 { result = result + 1 }
    I load_byte(data) == 192 { result = result + 1 }
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p80_msgpack_encode_bool() {
    // false = 0xc2 (194), true = 0xc3 (195)
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F mp_encode_bool(buf: i64, b: i64) -> i64 {
    I b == 0 { mp_buf_write(buf, 194) }
    E { mp_buf_write(buf, 195) }
}
F main() -> i64 {
    buf := mp_buf_new()
    mp_encode_bool(buf, 0)
    mp_encode_bool(buf, 1)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 2 { result = result + 1 }
    I load_byte(data) == 194 { result = result + 1 }     # false
    I load_byte(data + 1) == 195 { result = result + 1 } # true
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p80_msgpack_encode_positive_fixint() {
    // Positive fixint: 0x00-0x7f for values 0-127
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F mp_encode_int(buf: i64, n: i64) -> i64 {
    I n >= 0 && n <= 127 { mp_buf_write(buf, n) }
    E I n >= 0 - 32 && n < 0 { mp_buf_write(buf, 256 + n) }
    E I n >= 0 && n <= 255 {
        mp_buf_write(buf, 204)
        mp_buf_write(buf, n)
    }
    E { 0 }
}
F main() -> i64 {
    buf := mp_buf_new()
    mp_encode_int(buf, 0)
    mp_encode_int(buf, 42)
    mp_encode_int(buf, 127)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 3 { result = result + 1 }
    I load_byte(data) == 0 { result = result + 1 }       # 0
    I load_byte(data + 1) == 42 { result = result + 1 }  # 42
    I load_byte(data + 2) == 127 { result = result + 1 } # 127
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p80_msgpack_encode_negative_fixint() {
    // Negative fixint: 0xe0-0xff for -32 to -1
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F mp_encode_int(buf: i64, n: i64) -> i64 {
    I n >= 0 && n <= 127 { mp_buf_write(buf, n) }
    E I n >= 0 - 32 && n < 0 { mp_buf_write(buf, 256 + n) }
    E { 0 }
}
F main() -> i64 {
    buf := mp_buf_new()
    mp_encode_int(buf, 0 - 1)   # -1 -> 0xff (255)
    mp_encode_int(buf, 0 - 10)  # -10 -> 0xf6 (246)
    mp_encode_int(buf, 0 - 32)  # -32 -> 0xe0 (224)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 3 { result = result + 1 }
    I load_byte(data) == 255 { result = result + 1 }      # -1
    I load_byte(data + 1) == 246 { result = result + 1 }  # -10
    I load_byte(data + 2) == 224 { result = result + 1 }  # -32
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p80_msgpack_encode_uint8() {
    // uint 8: 0xcc prefix + 1 byte for values 128-255
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F mp_encode_int(buf: i64, n: i64) -> i64 {
    I n >= 0 && n <= 127 { mp_buf_write(buf, n) }
    E I n >= 0 && n <= 255 {
        mp_buf_write(buf, 204)
        mp_buf_write(buf, n)
    }
    E { 0 }
}
F main() -> i64 {
    buf := mp_buf_new()
    mp_encode_int(buf, 200)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 2 { result = result + 1 }
    I load_byte(data) == 204 { result = result + 1 }     # 0xcc prefix
    I load_byte(data + 1) == 200 { result = result + 1 } # value
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p80_msgpack_encode_fixstr() {
    // fixstr: 0xa0 + len, then string bytes
    // "Hi" = 0xa2 'H' 'i'
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F mp_buf_write_bytes(buf: i64, src: i64, count: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    memcpy(data + len, src, count)
    store_i64(buf + 8, len + count)
    count
}
F mp_encode_str(buf: i64, s: i64, slen: i64) -> i64 {
    I slen <= 31 { mp_buf_write(buf, 160 + slen) }
    E { 0 }
    mp_buf_write_bytes(buf, s, slen)
}
F main() -> i64 {
    buf := mp_buf_new()
    s := str_to_ptr("Hi")
    mp_encode_str(buf, s, 2)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 3 { result = result + 1 }
    I load_byte(data) == 162 { result = result + 1 }    # 0xa0 + 2
    I load_byte(data + 1) == 72 { result = result + 1 } # 'H'
    I load_byte(data + 2) == 105 { result = result + 1 } # 'i'
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p80_msgpack_encode_fixarray() {
    // fixarray: 0x90 + count, then elements
    // [1, 2, 3] = 0x93, 1, 2, 3
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F main() -> i64 {
    buf := mp_buf_new()
    # fixarray with 3 elements: 0x90 + 3 = 0x93 = 147
    mp_buf_write(buf, 147)
    # Three positive fixints
    mp_buf_write(buf, 1)
    mp_buf_write(buf, 2)
    mp_buf_write(buf, 3)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 4 { result = result + 1 }
    I load_byte(data) == 147 { result = result + 1 }     # fixarray(3)
    I load_byte(data + 1) == 1 { result = result + 1 }
    I load_byte(data + 2) == 2 { result = result + 1 }
    I load_byte(data + 3) == 3 { result = result + 1 }
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p80_msgpack_decode_positive_fixint() {
    // Decode bytes [42] -> positive fixint 42
    let source = r#"
F main() -> i64 {
    # Create a buffer with a single positive fixint: 42
    data := malloc(1)
    store_byte(data, 42)

    # Decode: read byte, check range 0x00-0x7f
    b := load_byte(data)
    result := 0
    I b <= 127 {
        # It's a positive fixint, value = b
        I b == 42 { result = result + 10 }
    }
    free(data)
    result
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p80_msgpack_decode_negative_fixint() {
    // Decode byte 0xf6 (246) -> negative fixint -10
    let source = r#"
F main() -> i64 {
    data := malloc(1)
    store_byte(data, 246)  # 0xf6 = -10 in negative fixint

    b := load_byte(data)
    result := 0
    I b >= 224 {
        # Negative fixint: value = b - 256
        val := b - 256
        I val == 0 - 10 { result = 10 }
    }
    free(data)
    result
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p80_msgpack_roundtrip_int() {
    // Encode integer 42, then decode it back
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F mp_encode_int(buf: i64, n: i64) -> i64 {
    I n >= 0 && n <= 127 { mp_buf_write(buf, n) }
    E I n >= 0 - 32 && n < 0 { mp_buf_write(buf, 256 + n) }
    E I n >= 0 && n <= 255 {
        mp_buf_write(buf, 204)
        mp_buf_write(buf, n)
    }
    E { 0 }
}
F mp_decode(data: i64, pos: i64) -> i64 {
    b := load_byte(data + pos)
    I b <= 127 { b }
    E I b >= 224 { b - 256 }
    E I b == 204 { load_byte(data + pos + 1) }
    E { 0 - 1 }
}
F main() -> i64 {
    buf := mp_buf_new()
    mp_encode_int(buf, 42)
    mp_encode_int(buf, 0 - 5)
    mp_encode_int(buf, 200)

    data := load_i64(buf)

    result := 0
    v1 := mp_decode(data, 0)
    I v1 == 42 { result = result + 1 }

    v2 := mp_decode(data, 1)
    I v2 == 0 - 5 { result = result + 1 }

    v3 := mp_decode(data, 2)
    I v3 == 200 { result = result + 1 }

    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p80_msgpack_fixmap_encode() {
    // fixmap: 0x80 + count, then key-value pairs
    // {"a": 1} = 0x81, 0xa1, 'a', 1
    let source = r#"
F mp_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F mp_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F main() -> i64 {
    buf := mp_buf_new()
    # fixmap with 1 entry: 0x80 + 1 = 129
    mp_buf_write(buf, 129)
    # key: fixstr "a" = 0xa1, 97
    mp_buf_write(buf, 161)
    mp_buf_write(buf, 97)
    # value: fixint 1
    mp_buf_write(buf, 1)

    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 4 { result = result + 1 }
    I load_byte(data) == 129 { result = result + 1 }     # fixmap(1)
    I load_byte(data + 1) == 161 { result = result + 1 } # fixstr(1)
    I load_byte(data + 2) == 97 { result = result + 1 }  # 'a'
    I load_byte(data + 3) == 1 { result = result + 1 }   # value 1
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 5);
}

// ==================== Protobuf Tests ====================

#[test]
fn e2e_p80_protobuf_varint_encode_small() {
    // Varint encode: value < 128 -> single byte
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F main() -> i64 {
    buf := pb_buf_new()
    pb_encode_varint(buf, 1)
    pb_encode_varint(buf, 42)
    pb_encode_varint(buf, 127)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 3 { result = result + 1 }
    I load_byte(data) == 1 { result = result + 1 }
    I load_byte(data + 1) == 42 { result = result + 1 }
    I load_byte(data + 2) == 127 { result = result + 1 }
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p80_protobuf_varint_encode_multibyte() {
    // Varint encode: 300 = 0xAC 0x02 (two bytes)
    // 300 = 0b100101100
    // byte1: 0101100 | 1 (continuation) = 0b10101100 = 172 (0xAC)
    // byte2: 0000010 = 2 (0x02)
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F main() -> i64 {
    buf := pb_buf_new()
    pb_encode_varint(buf, 300)
    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 2 { result = result + 1 }
    I load_byte(data) == 172 { result = result + 1 }    # 0xAC
    I load_byte(data + 1) == 2 { result = result + 1 }  # 0x02
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p80_protobuf_varint_decode() {
    // Decode varint: [172, 2] -> 300
    let source = r#"
F pb_decode_varint_rec(data: i64, pos: i64, result: i64, shift: i64) -> i64 {
    byte := load_byte(data + pos)
    new_result := result | ((byte & 127) << shift)
    I (byte & 128) == 0 { new_result }
    E { pb_decode_varint_rec(data, pos + 1, new_result, shift + 7) }
}
F pb_decode_varint(data: i64, pos: i64) -> i64 {
    pb_decode_varint_rec(data, pos, 0, 0)
}
F main() -> i64 {
    data := malloc(2)
    store_byte(data, 172)  # 0xAC
    store_byte(data + 1, 2) # 0x02
    val := pb_decode_varint(data, 0)
    free(data)
    I val == 300 { 10 } E { 0 }
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p80_protobuf_varint_roundtrip() {
    // Encode and decode various values
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F pb_decode_varint_rec(data: i64, pos: i64, result: i64, shift: i64) -> i64 {
    byte := load_byte(data + pos)
    new_result := result | ((byte & 127) << shift)
    I (byte & 128) == 0 { new_result }
    E { pb_decode_varint_rec(data, pos + 1, new_result, shift + 7) }
}
F pb_varint_size(data: i64, pos: i64) -> i64 {
    byte := load_byte(data + pos)
    I (byte & 128) == 0 { 1 }
    E { 1 + pb_varint_size(data, pos + 1) }
}
F main() -> i64 {
    buf := pb_buf_new()
    pb_encode_varint(buf, 0)
    pb_encode_varint(buf, 1)
    pb_encode_varint(buf, 300)
    pb_encode_varint(buf, 65535)

    data := load_i64(buf)

    result := 0
    # Decode value 0 at pos 0
    v0 := pb_decode_varint_rec(data, 0, 0, 0)
    I v0 == 0 { result = result + 1 }
    s0 := pb_varint_size(data, 0)

    # Decode value 1 at pos s0
    v1 := pb_decode_varint_rec(data, s0, 0, 0)
    I v1 == 1 { result = result + 1 }
    s1 := pb_varint_size(data, s0)

    # Decode value 300 at pos s0+s1
    v2 := pb_decode_varint_rec(data, s0 + s1, 0, 0)
    I v2 == 300 { result = result + 1 }
    s2 := pb_varint_size(data, s0 + s1)

    # Decode value 65535 at pos s0+s1+s2
    v3 := pb_decode_varint_rec(data, s0 + s1 + s2, 0, 0)
    I v3 == 65535 { result = result + 1 }

    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p80_protobuf_zigzag() {
    // ZigZag encoding: maps signed to unsigned
    // 0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3, 2 -> 4, etc.
    let source = r#"
F pb_zigzag_encode(n: i64) -> i64 {
    I n >= 0 { n * 2 }
    E { (0 - n) * 2 - 1 }
}
F pb_zigzag_decode(n: i64) -> i64 {
    half := (n >> 1) & 4611686018427387903
    I (n & 1) == 0 { half }
    E { 0 - half - 1 }
}
F main() -> i64 {
    result := 0
    # Encode tests
    I pb_zigzag_encode(0) == 0 { result = result + 1 }
    I pb_zigzag_encode(0 - 1) == 1 { result = result + 1 }
    I pb_zigzag_encode(1) == 2 { result = result + 1 }
    I pb_zigzag_encode(0 - 2) == 3 { result = result + 1 }
    I pb_zigzag_encode(2) == 4 { result = result + 1 }

    # Decode tests (round-trip)
    I pb_zigzag_decode(pb_zigzag_encode(0)) == 0 { result = result + 1 }
    I pb_zigzag_decode(pb_zigzag_encode(0 - 1)) == 0 - 1 { result = result + 1 }
    I pb_zigzag_decode(pb_zigzag_encode(42)) == 42 { result = result + 1 }
    I pb_zigzag_decode(pb_zigzag_encode(0 - 100)) == 0 - 100 { result = result + 1 }
    I pb_zigzag_decode(pb_zigzag_encode(12345)) == 12345 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p80_protobuf_field_tag() {
    // Field tag = (field_number << 3) | wire_type
    // field 1, wire type 0 (varint) = 0x08 = 8
    // field 2, wire type 2 (length-delimited) = 0x12 = 18
    let source = r#"
F pb_make_tag(field_num: i64, wire_type: i64) -> i64 {
    (field_num << 3) | (wire_type & 7)
}
F pb_tag_field_num(tag: i64) -> i64 {
    (tag >> 3) & 536870911
}
F pb_tag_wire_type(tag: i64) -> i64 {
    tag & 7
}
F main() -> i64 {
    result := 0
    # field 1, varint
    t1 := pb_make_tag(1, 0)
    I t1 == 8 { result = result + 1 }
    I pb_tag_field_num(t1) == 1 { result = result + 1 }
    I pb_tag_wire_type(t1) == 0 { result = result + 1 }

    # field 2, length-delimited
    t2 := pb_make_tag(2, 2)
    I t2 == 18 { result = result + 1 }
    I pb_tag_field_num(t2) == 2 { result = result + 1 }
    I pb_tag_wire_type(t2) == 2 { result = result + 1 }

    # field 15, 64-bit
    t3 := pb_make_tag(15, 1)
    I t3 == 121 { result = result + 1 }
    I pb_tag_field_num(t3) == 15 { result = result + 1 }
    I pb_tag_wire_type(t3) == 1 { result = result + 1 }

    result
}
"#;
    assert_exit_code(source, 9);
}

#[test]
fn e2e_p80_protobuf_write_varint_field() {
    // Encode field 1 (varint) with value 150
    // tag = (1 << 3) | 0 = 8 (one byte varint)
    // value = 150 = 0x96 0x01
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F main() -> i64 {
    buf := pb_buf_new()
    # tag for field 1, wire type 0
    tag := (1 << 3) | 0
    pb_encode_varint(buf, tag)
    # value 150
    pb_encode_varint(buf, 150)

    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 3 { result = result + 1 }
    I load_byte(data) == 8 { result = result + 1 }       # tag = 8
    I load_byte(data + 1) == 150 { result = result + 1 } # 150 & 127 | 128 = 150 (since 150 = 10010110, & 127 = 0010110 = 22, | 128 = 150)
    I load_byte(data + 2) == 1 { result = result + 1 }   # 150 >> 7 = 1
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p80_protobuf_write_string_field() {
    // Encode field 2 (string) with value "testing"
    // tag = (2 << 3) | 2 = 18
    // length = 7
    // data = "testing"
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_buf_write_bytes(buf: i64, src: i64, count: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    memcpy(data + len, src, count)
    store_i64(buf + 8, len + count)
    count
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F main() -> i64 {
    buf := pb_buf_new()
    # tag for field 2, wire type 2 (length-delimited)
    tag := (2 << 3) | 2
    pb_encode_varint(buf, tag)
    # string "testing" has length 7
    s := str_to_ptr("testing")
    slen := 7
    pb_encode_varint(buf, slen)
    pb_buf_write_bytes(buf, s, slen)

    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 9 { result = result + 1 }     # 1 (tag) + 1 (len) + 7 (data)
    I load_byte(data) == 18 { result = result + 1 }     # tag = 18
    I load_byte(data + 1) == 7 { result = result + 1 }  # length = 7
    I load_byte(data + 2) == 116 { result = result + 1 } # 't'
    I load_byte(data + 3) == 101 { result = result + 1 } # 'e'
    I load_byte(data + 8) == 103 { result = result + 1 } # 'g'
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_p80_protobuf_fixed32_encode() {
    // Encode 32-bit fixed field: wire type 5
    // Field 3, value 0x01020304 = little-endian: 04 03 02 01
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F pb_write_fixed32(buf: i64, value: i64) -> i64 {
    pb_buf_write(buf, value & 255)
    pb_buf_write(buf, (value >> 8) & 255)
    pb_buf_write(buf, (value >> 16) & 255)
    pb_buf_write(buf, (value >> 24) & 255)
}
F main() -> i64 {
    buf := pb_buf_new()
    # tag for field 3, wire type 5
    tag := (3 << 3) | 5
    pb_encode_varint(buf, tag)
    # value 0x04030201 = 67305985
    pb_write_fixed32(buf, 67305985)

    data := load_i64(buf)
    len := load_i64(buf + 8)
    result := 0
    I len == 5 { result = result + 1 }
    I load_byte(data) == 29 { result = result + 1 }     # tag = (3<<3)|5 = 29
    I load_byte(data + 1) == 1 { result = result + 1 }  # LE byte 0
    I load_byte(data + 2) == 2 { result = result + 1 }  # LE byte 1
    I load_byte(data + 3) == 3 { result = result + 1 }  # LE byte 2
    I load_byte(data + 4) == 4 { result = result + 1 }  # LE byte 3
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_p80_protobuf_message_parse() {
    // Parse a simple message with field 1 (varint=150), field 2 (string="abc")
    // Encoded: [08, 96 01, 12, 03, 61 62 63]
    //   field1: tag=08(field1,varint), value=150(96 01)
    //   field2: tag=12(field2,ld), len=3, "abc"
    let source = r#"
F pb_decode_varint_rec(data: i64, pos_ptr: i64, result: i64, shift: i64) -> i64 {
    pos := load_i64(pos_ptr)
    byte := load_byte(data + pos)
    store_i64(pos_ptr, pos + 1)
    new_result := result | ((byte & 127) << shift)
    I (byte & 128) == 0 { new_result }
    E { pb_decode_varint_rec(data, pos_ptr, new_result, shift + 7) }
}
F main() -> i64 {
    # Build encoded message manually
    msg := malloc(8)
    store_byte(msg, 8)    # tag: field 1, varint
    store_byte(msg + 1, 150) # value 150 low byte (150 & 127 | 128 = 150)
    store_byte(msg + 2, 1)   # value 150 high byte (150 >> 7 = 1)
    store_byte(msg + 3, 18)  # tag: field 2, length-delimited
    store_byte(msg + 4, 3)   # length = 3
    store_byte(msg + 5, 97)  # 'a'
    store_byte(msg + 6, 98)  # 'b'
    store_byte(msg + 7, 99)  # 'c'

    pos_ptr := malloc(8)
    store_i64(pos_ptr, 0)

    result := 0

    # Read field 1 tag
    tag1 := pb_decode_varint_rec(msg, pos_ptr, 0, 0)
    I (tag1 & 7) == 0 { result = result + 1 }   # wire type 0 (varint)
    I ((tag1 >> 3) & 536870911) == 1 { result = result + 1 }  # field number 1

    # Read field 1 value
    val1 := pb_decode_varint_rec(msg, pos_ptr, 0, 0)
    I val1 == 150 { result = result + 1 }

    # Read field 2 tag
    tag2 := pb_decode_varint_rec(msg, pos_ptr, 0, 0)
    I (tag2 & 7) == 2 { result = result + 1 }   # wire type 2 (ld)
    I ((tag2 >> 3) & 536870911) == 2 { result = result + 1 }  # field number 2

    # Read field 2 length
    slen := pb_decode_varint_rec(msg, pos_ptr, 0, 0)
    I slen == 3 { result = result + 1 }

    # Read string bytes
    pos := load_i64(pos_ptr)
    I load_byte(msg + pos) == 97 { result = result + 1 }      # 'a'
    I load_byte(msg + pos + 1) == 98 { result = result + 1 }  # 'b'
    I load_byte(msg + pos + 2) == 99 { result = result + 1 }  # 'c'

    free(pos_ptr)
    free(msg)
    result
}
"#;
    assert_exit_code(source, 9);
}

#[test]
fn e2e_p80_protobuf_embedded_message() {
    // Encode a sub-message and embed it in a parent message
    // Sub-message: field 1 = 42
    // Parent: field 3 = sub-message (wire type 2)
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_buf_write_bytes(buf: i64, src: i64, count: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    memcpy(data + len, src, count)
    store_i64(buf + 8, len + count)
    count
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F main() -> i64 {
    # Build sub-message: field 1 = 42
    sub := pb_buf_new()
    sub_tag := (1 << 3) | 0  # field 1, varint
    pb_encode_varint(sub, sub_tag)
    pb_encode_varint(sub, 42)

    sub_data := load_i64(sub)
    sub_len := load_i64(sub + 8)

    # Build parent: field 3 = sub-message
    parent := pb_buf_new()
    parent_tag := (3 << 3) | 2  # field 3, length-delimited
    pb_encode_varint(parent, parent_tag)
    pb_encode_varint(parent, sub_len)
    pb_buf_write_bytes(parent, sub_data, sub_len)

    pdata := load_i64(parent)
    plen := load_i64(parent + 8)

    result := 0
    # Parent should be: tag(26) + len(2) + [tag(8), value(42)]
    I plen == 4 { result = result + 1 }
    I load_byte(pdata) == 26 { result = result + 1 }     # parent tag
    I load_byte(pdata + 1) == 2 { result = result + 1 }  # sub-message length
    I load_byte(pdata + 2) == 8 { result = result + 1 }  # sub field 1 tag
    I load_byte(pdata + 3) == 42 { result = result + 1 } # sub field 1 value

    free(sub_data)
    free(sub)
    free(pdata)
    free(parent)
    result
}
"#;
    assert_exit_code(source, 5);
}

// ==================== Benchmark-style comparison tests ====================

#[test]
fn e2e_p80_msgpack_vs_json_size() {
    // Compare size: {"a":1,"b":2,"c":3}
    // JSON:     ~19 bytes as text
    // MsgPack:  ~7 bytes as binary (fixmap(3) + 3*(fixstr(1)+fixint))
    let source = r#"
F main() -> i64 {
    # Manually construct msgpack for {"a":1,"b":2,"c":3}
    buf := malloc(256)
    pos := 0

    # fixmap with 3 entries
    store_byte(buf + pos, 131)  # 0x83
    pos = pos + 1

    # "a" -> 1
    store_byte(buf + pos, 161)  # fixstr(1)
    store_byte(buf + pos + 1, 97)  # 'a'
    store_byte(buf + pos + 2, 1)   # fixint 1
    pos = pos + 3

    # "b" -> 2
    store_byte(buf + pos, 161)
    store_byte(buf + pos + 1, 98)  # 'b'
    store_byte(buf + pos + 2, 2)
    pos = pos + 3

    # "c" -> 3
    store_byte(buf + pos, 161)
    store_byte(buf + pos + 1, 99)  # 'c'
    store_byte(buf + pos + 2, 3)
    pos = pos + 3

    # JSON equivalent: {"a":1,"b":2,"c":3} = 19 bytes
    # MsgPack: 10 bytes (1 header + 3*(1+1+1))
    json_size := 19
    msgpack_size := pos

    result := 0
    I msgpack_size == 10 { result = result + 1 }
    I msgpack_size < json_size { result = result + 1 }

    free(buf)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p80_protobuf_compact_encoding() {
    // Protobuf is compact: field1=42, field2=300
    // tag(8)+val(42) + tag(16)+val(300) = 1+1 + 1+2 = 5 bytes
    // vs JSON {"field1":42,"field2":300} = 25 bytes
    let source = r#"
F pb_buf_new() -> i64 {
    data := malloc(256)
    buf := malloc(24)
    store_i64(buf, data)
    store_i64(buf + 8, 0)
    store_i64(buf + 16, 256)
    buf
}
F pb_buf_write(buf: i64, b: i64) -> i64 {
    data := load_i64(buf)
    len := load_i64(buf + 8)
    store_byte(data + len, b & 255)
    store_i64(buf + 8, len + 1)
    1
}
F pb_encode_varint(buf: i64, value: i64) -> i64 {
    I value >= 0 && value < 128 {
        pb_buf_write(buf, value)
        1
    } E {
        pb_buf_write(buf, (value & 127) | 128)
        next := (value >> 7) & 576460752303423487
        pb_encode_varint(buf, next)
    }
}
F main() -> i64 {
    buf := pb_buf_new()
    # field 1 = 42
    pb_encode_varint(buf, (1 << 3) | 0)  # tag
    pb_encode_varint(buf, 42)
    # field 2 = 300
    pb_encode_varint(buf, (2 << 3) | 0)  # tag
    pb_encode_varint(buf, 300)

    pb_len := load_i64(buf + 8)
    json_len := 25  # {"field1":42,"field2":300}

    result := 0
    I pb_len == 5 { result = result + 1 }
    I pb_len < json_len { result = result + 1 }

    data := load_i64(buf)
    free(data)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 2);
}
