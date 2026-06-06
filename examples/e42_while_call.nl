# expect: 8
# A while loop repeatedly applying a function: 1 -> 2 -> 4 -> 8 over 3 iterations.
fn dbl(x: Int) -> Int { return x * 2 }

fn main() -> Int {
    let mut v = 1
    let mut i = 0
    while i < 3 {
        v = dbl(v)
        i = i + 1
    }
    return v
}
