# expect: 2
# Guard-style early returns: classify a value through a chain of if-returns.
# (The idiomatic nl shape for multi-way branching with early exits.)
fn classify(x: Int) -> Int {
    if x < 0 { return 0 }
    if x == 0 { return 1 }
    return 2
}

fn main() -> Int {
    return classify(5)
}
