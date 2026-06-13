# expect: 42
# Nested match as a direct match-arm body, with an enum payload that itself
# contains an Option payload.
enum Wrap { Empty, Has(Option<Int>) }

fn pick(w: Wrap) -> Int {
    match w {
        Has(o) => match o {
            Some(v) => return v,
            None => return 0,
        },
        Empty => return 0,
    }
}

fn main() -> Int {
    return pick(Has(Some(42)))
}
