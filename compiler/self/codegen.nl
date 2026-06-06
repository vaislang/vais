# expect: 0
# 미니 codegen: 산술식을 평가하고, 그 값을 반환하는 LLVM IR을 출력한다.
# (self-host codegen의 씨앗: nl이 IR 텍스트를 생성)
fn eval_expr(toks: List<Int>) -> Int {
    let n = toks.len()
    if n == 0 { return 0 }
    let mut total = 0
    let mut term = toks[0]
    let mut i = 1
    while i < n {
        let op = toks[i]
        let rhs = toks[i + 1]
        if op == 0 - 2 {
            term = term * rhs
        } else {
            total = total + term
            term = rhs
        }
        i = i + 2
    }
    total = total + term
    return total
}
fn emit_ir(value: Int) -> Int {
    print("define i64 @main() {{")
    print("  ret i64 {value}")
    print("}}")
    return 0
}
fn main() -> Int {
    let toks: List<Int> = [1, 0 - 1, 2, 0 - 2, 3]
    let value = eval_expr(toks)
    return emit_ir(value)
}
