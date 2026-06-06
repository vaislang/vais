# expect: 2
fn grade(n: Int) -> Int {
    if n >= 90 {
        return 1
    } else if n >= 70 {
        return 2
    } else {
        return 3
    }
}
fn main() -> Int {
    return grade(75)
}
