# expect: 5
# A generic function: identity<T> works for any type (here Int).
# (nl supports type-parameter generics on functions.)
fn identity<T>(x: T) -> T {
    return x
}

fn main() -> Int {
    return identity(5)
}
