# expect: 43
# nl self-host compiler — L3.1 lexer (written in nl).
#
# Bootstrap path: this .nl source is transpiled to Vais by the seed transpiler,
# compiled by vaisc, producing gen1. Eventually gen1 lexes nl directly.
#
# This first version scans a source string and classifies bytes into token
# categories, returning a small encoded count so the value-correctness gate can
# verify the scan logic (exit code). It uses only the transpiler-supported subset
# (while / if / and / s[i] / comparisons) — no Vec yet (added in a later step to
# avoid the Vais filter/Vec-specialization issues while bootstrapping).
#
# Token categories (byte-class based, prototype):
#   digit   0-9        (48..57)
#   alpha   a-z A-Z _  (65..90, 97..122, 95)
#   space   ' ' \t \n  (32, 9, 10)
#   punct   everything else printable

fn is_digit(c: Int) -> Bool {
    return c >= 48 and c <= 57
}

fn is_alpha(c: Int) -> Bool {
    if c >= 97 and c <= 122 { return true }
    if c >= 65 and c <= 90 { return true }
    if c == 95 { return true }
    return false
}

fn is_space(c: Int) -> Bool {
    if c == 32 { return true }
    if c == 9 { return true }
    if c == 10 { return true }
    return false
}

# Count how many maximal "word" tokens (runs of alpha) are in the source.
# This is the core lexer loop: scan, skip non-words, count word starts.
fn count_words(src: Str) -> Int {
    let mut count = 0
    let mut i = 0
    let mut in_word = 0
    while i < src.len() {
        let c = src[i]
        if is_alpha(c) {
            if in_word == 0 {
                count = count + 1
            }
            in_word = 1
        } else {
            in_word = 0
        }
        i = i + 1
    }
    return count
}

fn count_numbers(src: Str) -> Int {
    let mut count = 0
    let mut i = 0
    let mut in_num = 0
    while i < src.len() {
        let c = src[i]
        if is_digit(c) {
            if in_num == 0 {
                count = count + 1
            }
            in_num = 1
        } else {
            in_num = 0
        }
        i = i + 1
    }
    return count
}

fn main() -> Int {
    # Source to lex: "fn add a b" has 4 words; "x1 y22 z333" has 3 numbers.
    let words = count_words("fn add a b")
    let nums = count_numbers("x1 y22 z333")
    # Encode both into one exit code to verify both: words*10 + nums = 43.
    return words * 10 + nums
}
