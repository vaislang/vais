# expect: 50
# Array indexing with a computed index inside an expression: a[i] + a[i+1].
fn main() -> Int {
    let a = [10, 20, 30]
    let i = 1
    return a[i] + a[i + 1]
}
