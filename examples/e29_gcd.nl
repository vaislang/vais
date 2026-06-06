# expect: 6
# Euclid's GCD: two-argument recursion with modulo (gcd(48,18) = 6).
# (Deeper than single-arg recursion -- the recursive call permutes both args.)
fn gcd(a: Int, b: Int) -> Int {
    if b == 0 { return a }
    return gcd(b, a % b)
}

fn main() -> Int {
    return gcd(48, 18)
}
