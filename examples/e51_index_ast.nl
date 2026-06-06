# expect: 14
# A recursive AST encoded as struct+index (the self-host pattern that sidesteps
# the recursive-enum Vais gap): each node has a kind (0=lit,1=add,2=mul), a value
# (for lits), and two child INDICES into a flat node array. eval recurses by index.
# Tree: mul(3,4) + 2 -> nodes: 0=lit3, 1=lit4, 2=mul(0,1), 3=lit2, 4=add(2,3).
struct NodeArr { k0: Int, v0: Int, l0: Int, r0: Int, k1: Int, v1: Int, l1: Int, r1: Int, k2: Int, v2: Int, l2: Int, r2: Int, k3: Int, v3: Int, l3: Int, r3: Int, k4: Int, v4: Int, l4: Int, r4: Int }
fn kind_of(a: NodeArr, i: Int) -> Int {
    if i == 0 { return a.k0 }
    if i == 1 { return a.k1 }
    if i == 2 { return a.k2 }
    if i == 3 { return a.k3 }
    return a.k4
}
fn val_of(a: NodeArr, i: Int) -> Int {
    if i == 0 { return a.v0 }
    if i == 1 { return a.v1 }
    if i == 2 { return a.v2 }
    if i == 3 { return a.v3 }
    return a.v4
}
fn lhs_of(a: NodeArr, i: Int) -> Int {
    if i == 2 { return a.l2 }
    if i == 4 { return a.l4 }
    return 0
}
fn rhs_of(a: NodeArr, i: Int) -> Int {
    if i == 2 { return a.r2 }
    if i == 4 { return a.r4 }
    return 0
}
fn eval(a: NodeArr, i: Int) -> Int {
    let k = kind_of(a, i)
    if k == 0 { return val_of(a, i) }
    let lv = eval(a, lhs_of(a, i))
    let rv = eval(a, rhs_of(a, i))
    if k == 1 { return lv + rv }
    return lv * rv
}
fn main() -> Int {
    let a = NodeArr { k0: 0, v0: 3, l0: 0, r0: 0, k1: 0, v1: 4, l1: 0, r1: 0, k2: 2, v2: 0, l2: 0, r2: 1, k3: 0, v3: 2, l3: 0, r3: 0, k4: 1, v4: 0, l4: 2, r4: 3 }
    return eval(a, 4)
}
