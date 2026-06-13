# expect: 30
# while-loop accumulator: sum of squares 1..=4 = 1 + 4 + 9 + 16 = 30.
# (The canonical imperative idiom — mutable accumulator + counter + condition.)
fn main() -> Int {
    let mut sum = 0
    let mut i = 1
    while i <= 4 {
        sum = sum + i * i
        i = i + 1
    }
    return sum
}
