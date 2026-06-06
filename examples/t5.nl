fn find(k: Int) -> Option<Int> {
    if k > 0 {
        return Some(k)
    }
    return None
}
fn main() -> Int {
    match find(5) {
        Some(x) => x,
        None => 0,
    }
}
