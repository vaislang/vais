fn sum_big() -> Int {
    let list = [5, 2, 8, 1]
    return list.filter(|x| x > 3).sum()
}
fn main() -> Int {
    return sum_big()
}
