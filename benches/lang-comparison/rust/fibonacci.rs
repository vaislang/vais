// Fibonacci - recursive and iterative
fn fib_rec(n: i64) -> i64 {
    if n <= 1 { n } else { fib_rec(n - 1) + fib_rec(n - 2) }
}

fn fib_iter(n: i64) -> i64 {
    let mut a: i64 = 0;
    let mut b: i64 = 1;
    for _ in 0..n {
        let t = a + b;
        a = b;
        b = t;
    }
    a
}

fn main() {
    println!("{}", fib_rec(20));
    println!("{}", fib_iter(50));
}
