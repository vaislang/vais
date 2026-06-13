# expect: 6
# Recursion that threads a struct accumulator (sum + count) through each call.
# build(3, {0,0}) accumulates 3+2+1 = 6.
struct Acc { sum: Int, count: Int }

fn build(n: Int, a: Acc) -> Acc {
    if n <= 0 { return a }
    return build(n - 1, Acc { sum: a.sum + n, count: a.count + 1 })
}

fn main() -> Int {
    let r = build(3, Acc { sum: 0, count: 0 })
    return r.sum
}
