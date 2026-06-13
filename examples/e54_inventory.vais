# expect: 42
# Inventory total: a struct method (subtotal = price * qty) summed across three
# item instances. 10*2 + 5*3 + 7*1 = 42. (Written by a cold-start AI from the
# corpus, first-try; the corpus's first multi-instance struct aggregation.)
struct Item { price: Int, qty: Int }

fn subtotal(it: Item) -> Int {
    return it.price * it.qty
}

fn main() -> Int {
    let a = Item { price: 10, qty: 2 }
    let b = Item { price: 5, qty: 3 }
    let c = Item { price: 7, qty: 1 }
    return subtotal(a) + subtotal(b) + subtotal(c)
}
