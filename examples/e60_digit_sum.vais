# expect: 15
# Sum the decimal digits of an integer: peel the last digit with `% 10`, drop it
# with `/ 10`, until 0. digit_sum(12345) = 1+2+3+4+5 = 15.
fn digit_sum(n: Int) -> Int {
    let mut total = 0
    let mut x = n
    while x > 0 {
        total = total + x % 10
        x = x / 10
    }
    return total
}

fn main() -> Int {
    return digit_sum(12345)
}
