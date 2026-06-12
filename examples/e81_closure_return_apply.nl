# expect: 21
# A captured closure escapes from one function, then crosses another
# higher-order function boundary before being called twice.
fn make_step(delta: Int) -> fn(Int) -> Int {
    return |x| x + delta
}

fn apply_twice(f: fn(Int) -> Int, start: Int) -> Int {
    return f(f(start))
}

fn main() -> Int {
    let add10 = make_step(10)
    return apply_twice(add10, 1)
}
