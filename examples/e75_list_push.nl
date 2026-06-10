# expect: 23
# Empty List<T> plus push growth. 3 elements, second value 20 => 3 + 20 = 23.
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    return xs.len() + xs[1]
}
