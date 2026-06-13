# expect: 7
# Result Ok/Err match — bind the success value or the error in each arm.
# (Complements e08/e16 Option matching with the two-variant Result form.)
fn div(a: Int, b: Int) -> Result<Int, Int> {
    if b == 0 { return Err(0) }
    return Ok(a / b)
}

fn main() -> Int {
    match div(21, 3) {
        Ok(v) => return v,
        Err(e) => return e,
    }
}
