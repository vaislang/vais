# expect: 42
# Binary search over a sorted borrowed List. Lists move by value, so reusable
# collection helpers should take `&List<T>`. This combines while loops, computed
# indexing, early returns, and a negative not-found sentinel.
fn binary_search(xs: &List<Int>, n: Int, target: Int) -> Int {
    let mut lo = 0
    let mut hi = n
    while lo < hi {
        let mid = (lo + hi) / 2
        let v = xs[mid]
        if v == target { return mid }
        if v < target {
            lo = mid + 1
        } else {
            hi = mid
        }
    }
    return 0 - 1
}

fn main() -> Int {
    let values: List<Int> = [3, 6, 9, 12, 15, 18, 21]
    let found = binary_search(&values, 7, 15)
    let missing = binary_search(&values, 7, 10)
    return found * 10 + missing + 3
}
