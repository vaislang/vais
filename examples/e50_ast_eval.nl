# expect: 14
# A small recursive expression evaluator: a Node is a literal or a binary op
# carrying nested Node operands, dispatched by match.
enum Node { Lit(Int), Add(Node, Node), Mul(Node, Node) }

fn eval(n: Node) -> Int {
    match n {
        Lit(v) => return v,
        Add(a, b) => return eval(a) + eval(b),
        Mul(a, b) => return eval(a) * eval(b),
    }
}

fn main() -> Int {
    return eval(Add(Mul(Lit(3), Lit(4)), Lit(2)))
}
