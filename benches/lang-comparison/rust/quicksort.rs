// Quicksort with array partitioning
fn partition(arr: &mut [i64], lo: usize, hi: usize) -> usize {
    let pivot = arr[hi];
    let mut i = lo;
    for j in lo..hi {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, hi);
    i
}

fn quicksort(arr: &mut [i64], lo: isize, hi: isize) {
    if lo < hi {
        let p = partition(arr, lo as usize, hi as usize);
        quicksort(arr, lo, p as isize - 1);
        quicksort(arr, p as isize + 1, hi);
    }
}

fn main() {
    let mut arr = [38i64, 27, 43, 3, 9, 82, 10, 55, 1, 77];
    let len = arr.len() as isize;
    quicksort(&mut arr, 0, len - 1);
    for val in &arr {
        println!("{}", val);
    }
}
