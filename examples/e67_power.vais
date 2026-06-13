# expect: 81
# Recursive exponentiation: power(base, exp) = base * power(base, exp-1), base case
# exp==0 -> 1. power(3, 4) = 81. (Two-argument recursion decrementing one arg.)
fn power(base: Int, exp: Int) -> Int {
    if exp == 0 { return 1 }
    return base * power(base, exp - 1)
}

fn main() -> Int {
    return power(3, 4)
}
