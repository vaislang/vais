# expect: 7
# An Option stored as a struct field, then matched after access.
struct Box { val: Option<Int> }

fn main() -> Int {
    let b = Box { val: Some(7) }
    match b.val {
        Some(v) => return v,
        None => return 0,
    }
}
