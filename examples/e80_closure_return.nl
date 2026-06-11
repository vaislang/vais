# expect: 7
# A function returns a closure that captures its argument.
# This exercises the Vais production closure object ABI ({code, env}).
fn adder(n: Int) -> fn(Int) -> Int {
    return |x| x + n
}

fn main() -> Int {
    let add3 = adder(3)
    return add3(4)
}
