# expect: 25
struct Counter { value: Int }
impl Counter {
    fn bump(self, n: Int) -> Counter {
        return Counter { value: self.value + n }
    }
    fn get(self) -> Int {
        return self.value
    }
}
fn main() -> Int {
    let c = Counter { value: 0 }
    return c.bump(10).bump(15).get()
}
