# expect: 7
# A generic function applied to a struct value (generics + structs compose).
struct Pair { a: Int, b: Int }
fn first_field(p: Pair) -> Int { return p.a }
fn apply<T>(x: T) -> T { return x }

fn main() -> Int {
    let p = apply(Pair { a: 7, b: 2 })
    return first_field(p)
}
