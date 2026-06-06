# expect: 1
# A function returning Bool, used directly as an if-condition.
# (Predicates return Bool in nl; is_even(4) is true, so this returns 1.)
fn is_even(n: Int) -> Bool {
    return n % 2 == 0
}

fn main() -> Int {
    if is_even(4) { return 1 }
    return 0
}
