# expect: 10
# Recursion over a List by BORROW (`&List<Int>`). Summing a list recursively is
# the canonical "walk a collection" shape — needed for interpreters/tree walkers.
#
# nl Lists are move-by-value (passing one by value moves it), so recurse by
# reference: `&List<Int>`. This compiles to Vais `&Vec<i64>` and now works
# end-to-end (the Vais `&Vec` borrow codegen was fixed so a borrowed Vec passes
# its address, not a slice fat pointer).

fn sum_from(v: &List<Int>, i: Int, n: Int) -> Int {
    if i >= n { return 0 }
    let rest = sum_from(v, i + 1, n)
    return v[i] + rest
}

fn main() -> Int {
    let nums: List<Int> = [1, 2, 3, 4]
    return sum_from(&nums, 0, 4)
}
