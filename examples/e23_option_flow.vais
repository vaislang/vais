# expect: 15
# Option returned from a function, matched, and the values combined.
# (Two lookups, each defaulted via a None arm, then summed.)
fn lookup(k: Int) -> Option<Int> {
    if k > 0 { return Some(k * 3) }
    return None
}

fn main() -> Int {
    let a = match lookup(2) { Some(v) => v, None => 0 }
    let b = match lookup(3) { Some(v) => v, None => 0 }
    return a + b
}
