# expect: 28
# Recursive triangular number: tri(n) = n + tri(n-1), tri(0) = 0.
# (Written by a cold-start AI with the corpus as its ONLY reference, then
# value-verified -- a worked demonstration of P9: examples enable cold-start.)
fn tri(n: Int) -> Int {
    if n <= 0 { return 0 }
    return n + tri(n - 1)
}

fn main() -> Int {
    return tri(7)
}
