# expect: 3
# A state machine over a coded input: count runs of 1s separated by 0s in a
# packed integer sequence (the tokenizer's in-word-flag pattern from self-host).
fn count_runs(bits: Int) -> Int {
    let arr = [1, 1, 0, 1, 0, 1, 1, 1]
    let mut i = 0
    let mut runs = 0
    let mut inrun = 0
    while i < 8 {
        if arr[i] == 0 {
            inrun = 0
        } else {
            if inrun == 0 { runs = runs + 1 }
            inrun = 1
        }
        i = i + 1
    }
    return runs
}
fn main() -> Int {
    return count_runs(0)
}
