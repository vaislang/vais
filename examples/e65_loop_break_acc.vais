# expect: 10
# A while loop that accumulates and breaks once a threshold is reached.
# 1+2+3+4 = 10 >= 10, so it breaks with s = 10.
fn main() -> Int {
    let mut s = 0
    let mut i = 1
    while i < 100 {
        s = s + i
        if s >= 10 { break }
        i = i + 1
    }
    return s
}
