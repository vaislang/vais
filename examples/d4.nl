fn largest(items: List<Int>) -> Int {
    let mut max = items[0]
    for x in items {
        if x > max {
            max = x
        }
    }
    return max
}
