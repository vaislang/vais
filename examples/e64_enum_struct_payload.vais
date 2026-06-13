# expect: 5
# An enum whose payload is a struct: Has(Pt) carries a Pt; match binds it.
# (Non-recursive, so it works -- a recursive enum is a tracked Vais gap.)
struct Pt { x: Int }
enum Maybe { Has(Pt), Empty }

fn main() -> Int {
    let m = Has(Pt { x: 5 })
    match m {
        Has(p) => return p.x,
        Empty => return 0,
    }
}
