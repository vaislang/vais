# expect: 42
pub struct User {
    pub name: Str,
    id: Int,
}
pub fn make_user(name: Str, id: Int) -> User {
    return User { name: name, id: id }
}
fn main() -> Int {
    let u = make_user("x", 42)
    return u.id
}
