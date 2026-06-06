# expect: 9
fn main() -> Int {
    let mut c = 0
    for i in 0..3 {
        for j in 0..3 {
            c = c + 1
        }
    }
    return c
}
