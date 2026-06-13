# expect: 4
# `continue` skips the rest of the loop body for this iteration. Over i = 1..5,
# i == 3 is skipped, so s is incremented 4 times.
fn main() -> Int {
    let mut s = 0
    let mut i = 0
    while i < 5 {
        i = i + 1
        if i == 3 { continue }
        s = s + 1
    }
    return s
}
