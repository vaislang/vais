# expect: 12
# Return multiple values via a named struct (the typed alternative to a tuple).
# minmax(8, 4) returns {lo: 4, hi: 8}; lo + hi = 12.
struct Result2 { lo: Int, hi: Int }

fn minmax(a: Int, b: Int) -> Result2 {
    if a < b { return Result2 { lo: a, hi: b } }
    return Result2 { lo: b, hi: a }
}

fn main() -> Int {
    let r = minmax(8, 4)
    return r.lo + r.hi
}
