# expect: 42
# Two-pointer palindrome check over string bytes. This combines Str parameters,
# s.len(), computed byte indexing from both ends, `!=`, and a shrinking while
# window.
fn is_palindrome(s: Str) -> Int {
    let mut left = 0
    let mut right = s.len() - 1
    while left < right {
        if s[left] != s[right] { return 0 }
        left = left + 1
        right = right - 1
    }
    return 1
}

fn main() -> Int {
    let odd = is_palindrome("level")
    let even = is_palindrome("abba")
    let miss = is_palindrome("robot")
    return odd * 40 + even * 2 + miss
}
