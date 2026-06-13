# expect: 42
# Passing a List literal directly to a List<T> parameter.
# This locks the nl surface for the Vais expected-type Vec materialization path.
fn sum3(xs: List<Int>) -> Int {
    return xs[0] + xs[1] + xs[2]
}

fn main() -> Int {
    return sum3([10, 20, 12])
}
