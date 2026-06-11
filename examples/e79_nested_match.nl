# expect: 42
# Nested match as a direct match-arm body. The transpiler wraps the inner match
# arm body for Vais, while nl keeps the compact `=> match ...` surface.
enum Wrap { Empty, Has(Int) }

fn lookup(x: Int) -> Option<Int> {
    if x > 0 { return Some(x + 1) }
    return None
}

fn pick(w: Wrap) -> Int {
    match w {
        Has(x) => match lookup(x) {
            Some(v) => return v,
            None => return 0,
        },
        Empty => return 0,
    }
}

fn main() -> Int {
    return pick(Has(41))
}
