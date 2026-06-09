# expect: 42
# Parse a leading unsigned decimal run from a string. This exercises tokenizer-like
# digit classification, Str byte indexing, early stop on a non-digit byte, and
# base-10 accumulation.
fn is_digit_byte(b: Int) -> Bool {
    return b >= 48 and b <= 57
}

fn parse_uint(s: Str) -> Int {
    let mut i = 0
    let mut value = 0
    while i < s.len() {
        let b = s[i]
        if is_digit_byte(b) {
            value = value * 10 + (b - 48)
        } else {
            return value
        }
        i = i + 1
    }
    return value
}

fn main() -> Int {
    let a = parse_uint("19")
    let b = parse_uint("16")
    let c = parse_uint("7x9")
    return a + b + c
}
