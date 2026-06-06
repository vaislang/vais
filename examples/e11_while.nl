# expect: 10
fn main() -> Int {
    let mut s = 0
    let mut i = 0
    while i < 5 {
        s = s + i
        i = i + 1
    }
    return s
}
