# expect: 9
# Find the max of a List passed as a parameter (loop + running max).
# NOTE: pass the list via a bound variable (`let v = [...]; maxof(v)`) -- a list
# literal given DIRECTLY as an argument is a tracked Vais coercion gap.
fn maxof(xs: List<Int>) -> Int {
    let mut m = 0
    for x in xs {
        if x > m { m = x }
    }
    return m
}

fn main() -> Int {
    let v = [3, 9, 2, 7]
    return maxof(v)
}
