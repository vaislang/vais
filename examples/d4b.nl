# expect: 9
fn largest(items: List<Int>) -> Int {
    let mut max = items[0]
    for x in items {
        if x > max {
            max = x
        }
    }
    return max
}
fn main() -> Int {
    let v = [3, 9, 2, 7]
    return largest(v)
}
