# expect: 42
enum Shape { Circle(Int), Rect(Int, Int) }
fn area(s: Shape) -> Int {
    match s {
        Circle(r) => r,
        Rect(w, h) => w * h,
    }
}
fn main() -> Int {
    return area(Rect(6, 7))
}
