# expect: 42
# Find the first occurrence of a needle in a haystack using nested while loops.
# This combines Str parameters, computed byte indexing (`hay[i + j]`), early
# mismatch handling, and a negative not-found sentinel.
fn index_of(hay: Str, needle: Str) -> Int {
    let hn = hay.len()
    let nn = needle.len()
    if nn == 0 { return 0 }
    if nn > hn { return 0 - 1 }

    let mut i = 0
    while i <= hn - nn {
        let mut j = 0
        let mut ok = 1
        while j < nn {
            if hay[i + j] != needle[j] {
                ok = 0
            }
            j = j + 1
        }
        if ok == 1 { return i }
        i = i + 1
    }
    return 0 - 1
}

fn main() -> Int {
    let a = index_of("selfhost", "host")
    let b = index_of("xxxt", "t")
    let miss = index_of("abc", "z")
    return a * 10 + b + miss
}
