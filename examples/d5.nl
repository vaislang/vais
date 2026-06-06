pub struct User {
    pub name: Str,
    id: Int,
}
pub fn make_user(name: Str, id: Int) -> User {
    return User { name: name, id: id }
}
