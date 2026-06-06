# expect: 14
# A small expression evaluator: a Node is a literal or a binary op carrying two
# already-evaluated operands, dispatched by match. Models an interpreter's core.
# NOTE: operands are Int (pre-evaluated), not nested Node -- self-referential
# recursive enums (Node containing Node) are a tracked Vais backend gap, so a
# real recursive AST uses the struct+index encoding instead (see self-host).
enum Node { Lit(Int), Add(Int, Int), Mul(Int, Int) }

fn eval(n: Node) -> Int {
    match n {
        Lit(v) => return v,
        Add(a, b) => return a + b,
        Mul(a, b) => return a * b,
    }
}

fn main() -> Int {
    let prod = eval(Mul(3, 4))
    let total = eval(Add(prod, 2))
    return total
}
