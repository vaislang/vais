fn safe_div(a: Int, b: Int) -> Result<Int, Str> {
    if b == 0 {
        return Err("divide by zero")
    }
    return Ok(a / b)
}
fn div_plus_one(a: Int, b: Int) -> Result<Int, Str> {
    let r = safe_div(a, b)?
    return Ok(r + 1)
}
