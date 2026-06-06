# expect: 7
# Mutate a nested struct field through two levels (o.inner.v = ...).
# (Builds on e01's nested struct read with a nested field write.)
struct Inner { v: Int }
struct Outer { inner: Inner }

fn main() -> Int {
    let mut o = Outer { inner: Inner { v: 3 } }
    o.inner.v = 7
    return o.inner.v
}
