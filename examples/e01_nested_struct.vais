# expect: 9
struct Inner { v: Int }
struct Outer { inner: Inner }
fn main() -> Int {
    let o = Outer { inner: Inner { v: 9 } }
    return o.inner.v
}
