# expect: 2
# A match with a catch-all `_` arm (the default case).
fn classify(x: Int) -> Int {
    match x {
        1 => return 1,
        2 => return 2,
        _ => return 0,
    }
}

fn main() -> Int {
    return classify(2)
}
