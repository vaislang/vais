# Quicksort with array partitioning
def partition(arr: list[int], lo: int, hi: int) -> int:
    pivot = arr[hi]
    i = lo
    for j in range(lo, hi):
        if arr[j] <= pivot:
            arr[i], arr[j] = arr[j], arr[i]
            i += 1
    arr[i], arr[hi] = arr[hi], arr[i]
    return i

def quicksort(arr: list[int], lo: int, hi: int) -> None:
    if lo < hi:
        p = partition(arr, lo, hi)
        quicksort(arr, lo, p - 1)
        quicksort(arr, p + 1, hi)

def main():
    arr = [38, 27, 43, 3, 9, 82, 10, 55, 1, 77]
    quicksort(arr, 0, len(arr) - 1)
    for v in arr:
        print(v)

if __name__ == "__main__":
    main()
