# expect: 42
# Scan identifier spans and classify a couple of keywords. This mirrors the
# lexer pattern used by the self-host compiler: start-byte check, continuation
# scan, source-range equality against keyword literals, and no short-circuit
# indexing at end-of-string.
fn is_alpha(c: Int) -> Bool {
    if c >= 97 and c <= 122 { return true }
    if c >= 65 and c <= 90 { return true }
    return false
}

fn is_digit(c: Int) -> Bool {
    return c >= 48 and c <= 57
}

fn is_ident_start(c: Int) -> Bool {
    return is_alpha(c) or c == 95
}

fn is_ident_continue(c: Int) -> Bool {
    return is_ident_start(c) or is_digit(c)
}

fn ident_len(src: Str, start: Int) -> Int {
    if start >= src.len() { return 0 }
    if is_ident_start(src[start]) == false { return 0 }

    let mut i = start
    let mut go = 1
    while go == 1 {
        if i >= src.len() {
            go = 0
        } else if is_ident_continue(src[i]) {
            i = i + 1
        } else {
            go = 0
        }
    }
    return i - start
}

fn span_eq(src: Str, start: Int, len: Int, word: Str) -> Int {
    if len != word.len() { return 0 }
    let mut i = 0
    while i < len {
        if src[start + i] != word[i] { return 0 }
        i = i + 1
    }
    return 1
}

fn classify_ident(src: Str, start: Int) -> Int {
    let len = ident_len(src, start)
    if len == 0 { return 0 }
    if span_eq(src, start, len, "let") == 1 { return 7 }
    if span_eq(src, start, len, "return") == 1 { return 8 }
    return 1
}

fn main() -> Int {
    let src = "let return value42 _tmp"
    let kw_let = classify_ident(src, 0)
    let kw_return = classify_ident(src, 4)
    let value_len = ident_len(src, 11)
    let plain = classify_ident(src, 19)
    return kw_let * 4 + kw_return + value_len - plain
}
