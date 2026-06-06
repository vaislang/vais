# expect: 19
# Function composition pipeline: dec(dbl(inc(9))) = dec(dbl(10)) = dec(20) = 19.
# (Nested calls compose three single-argument transforms.)
fn inc(x: Int) -> Int { return x + 1 }
fn dbl(x: Int) -> Int { return x * 2 }
fn dec(x: Int) -> Int { return x - 1 }

fn main() -> Int {
    return dec(dbl(inc(9)))
}
