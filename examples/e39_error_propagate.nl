# expect: 0
# The ? operator propagating an error: check(-5) returns Err(99), so the ? in
# process short-circuits and the Err arm runs (returns 0). The FAILURE path of ?.
fn check(x: Int) -> Result<Int, Int> {
    if x < 0 { return Err(99) }
    return Ok(x)
}

fn process(x: Int) -> Result<Int, Int> {
    let v = check(x)?
    return Ok(v + 1)
}

fn main() -> Int {
    match process(0 - 5) {
        Ok(v) => return v,
        Err(e) => return 0,
    }
}
