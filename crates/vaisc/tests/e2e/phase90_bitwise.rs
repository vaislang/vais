//! Phase 90 -- Bitwise Operations
//!
//! Tests for bitwise AND, OR, XOR, NOT, shifts, and bit manipulation patterns.

use super::helpers::*;

// ==================== Basic Bitwise Ops ====================

#[test]
fn e2e_bit_and_basic() {
    // 0xFF & 0x0F = 0x0F = 15
    assert_exit_code("F main()->i64 = 255 & 15", 15);
}

#[test]
fn e2e_bit_or_basic() {
    // 0xF0 | 0x0F = 0xFF = 255
    assert_exit_code("F main()->i64 = 240 | 15", 255);
}

#[test]
fn e2e_bit_xor_basic() {
    // 0xFF ^ 0xAA = 0x55 = 85
    assert_exit_code("F main()->i64 = 255 ^ 170", 85);
}

#[test]
fn e2e_bit_xor_self() {
    // x ^ x = 0
    let source = r#"
F main() -> i64 {
    x := 42
    x ^ x
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_bit_and_zero() {
    // x & 0 = 0
    assert_exit_code("F main()->i64 = 12345 & 0", 0);
}

#[test]
fn e2e_bit_or_zero() {
    // x | 0 = x
    assert_exit_code("F main()->i64 = 42 | 0", 42);
}

#[test]
fn e2e_bit_xor_zero() {
    // x ^ 0 = x
    assert_exit_code("F main()->i64 = 42 ^ 0", 42);
}

// ==================== Shift Operations ====================

#[test]
fn e2e_bit_shl_basic() {
    // 1 << 5 = 32
    assert_exit_code("F main()->i64 = 1 << 5", 32);
}

#[test]
fn e2e_bit_shr_basic() {
    // 256 >> 3 = 32
    assert_exit_code("F main()->i64 = 256 >> 3", 32);
}

#[test]
fn e2e_bit_shl_by_zero() {
    // x << 0 = x
    assert_exit_code("F main()->i64 = 42 << 0", 42);
}

#[test]
fn e2e_bit_shr_by_zero() {
    // x >> 0 = x
    assert_exit_code("F main()->i64 = 42 >> 0", 42);
}

#[test]
fn e2e_bit_shl_multiply() {
    // 21 << 1 = 42 (same as 21 * 2)
    assert_exit_code("F main()->i64 = 21 << 1", 42);
}

#[test]
fn e2e_bit_shr_divide() {
    // 84 >> 1 = 42 (same as 84 / 2)
    assert_exit_code("F main()->i64 = 84 >> 1", 42);
}

// ==================== Bit Manipulation Functions ====================

#[test]
fn e2e_bit_is_even() {
    let source = r#"
F is_even(x: i64) -> i64 = 1 - (x & 1)
F main() -> i64 = is_even(42)
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_bit_is_odd() {
    let source = r#"
F is_odd(x: i64) -> i64 = x & 1
F main() -> i64 = is_odd(43)
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_bit_low_nibble() {
    let source = r#"
F low_nibble(x: i64) -> i64 = x & 15
F main() -> i64 = low_nibble(42)
"#;
    // 42 = 0x2A, low nibble = 0xA = 10
    assert_exit_code(source, 10);
}

#[test]
fn e2e_bit_high_nibble() {
    let source = r#"
F high_nibble(x: i64) -> i64 = (x >> 4) & 15
F main() -> i64 = high_nibble(42)
"#;
    // 42 = 0x2A, high nibble = 0x2 = 2
    assert_exit_code(source, 2);
}

#[test]
fn e2e_bit_swap_nibbles() {
    let source = r#"
F swap_nibbles(x: i64) -> i64 = ((x & 15) << 4) | ((x >> 4) & 15)
F main() -> i64 = swap_nibbles(0)
"#;
    assert_exit_code(source, 0);
}

// ==================== Bit Counting ====================

#[test]
fn e2e_bit_count_recursive() {
    let source = r#"
F popcount(n: i64) -> i64 {
    I n == 0 { R 0 }
    R (n & 1) + @(n >> 1)
}
F main() -> i64 = popcount(255)
"#;
    // 255 = 0xFF = 8 bits set
    assert_exit_code(source, 8);
}

#[test]
fn e2e_bit_count_power_of_two() {
    let source = r#"
F popcount(n: i64) -> i64 {
    I n == 0 { R 0 }
    R (n & 1) + @(n >> 1)
}
F main() -> i64 = popcount(64)
"#;
    // 64 = 0x40 = 1 bit set
    assert_exit_code(source, 1);
}

#[test]
fn e2e_bit_is_power_of_two() {
    let source = r#"
F is_pow2(n: i64) -> i64 {
    I n <= 0 { R 0 }
    I (n & (n - 1)) == 0 { R 1 }
    E { R 0 }
}
F main() -> i64 = is_pow2(64)
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_bit_is_not_power_of_two() {
    let source = r#"
F is_pow2(n: i64) -> i64 {
    I n <= 0 { R 0 }
    I (n & (n - 1)) == 0 { R 1 }
    E { R 0 }
}
F main() -> i64 = is_pow2(42)
"#;
    assert_exit_code(source, 0);
}

// ==================== Complex Bit Patterns ====================

#[test]
fn e2e_bit_xor_swap() {
    let source = r#"
F main() -> i64 {
    a := mut 10
    b := mut 42
    a = a ^ b
    b = b ^ a
    a = a ^ b
    a
}
"#;
    // After XOR swap, a should be 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bit_set_bit() {
    let source = r#"
F set_bit(x: i64, pos: i64) -> i64 = x | (1 << pos)
F main() -> i64 = set_bit(40, 1)
"#;
    // 40 | 2 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bit_clear_bit() {
    let source = r#"
F clear_bit(x: i64, pos: i64) -> i64 = x & (255 - (1 << pos))
F main() -> i64 = clear_bit(46, 2)
"#;
    // 46 = 0b101110, clear bit 2 → 0b101010 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bit_mask_generation() {
    let source = r#"
F mask(n: i64) -> i64 = (1 << n) - 1
F main() -> i64 = mask(6)
"#;
    // (1 << 6) - 1 = 63
    assert_exit_code(source, 63);
}
