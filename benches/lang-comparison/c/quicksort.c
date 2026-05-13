// Quicksort with array partitioning
#include <stdio.h>
#include <stdint.h>

void swap(int64_t *arr, int i, int j) {
    int64_t t = arr[i];
    arr[i] = arr[j];
    arr[j] = t;
}

int partition(int64_t *arr, int lo, int hi) {
    int64_t pivot = arr[hi];
    int i = lo;
    for (int j = lo; j < hi; j++) {
        if (arr[j] <= pivot) {
            swap(arr, i, j);
            i++;
        }
    }
    swap(arr, i, hi);
    return i;
}

void quicksort(int64_t *arr, int lo, int hi) {
    if (lo < hi) {
        int p = partition(arr, lo, hi);
        quicksort(arr, lo, p - 1);
        quicksort(arr, p + 1, hi);
    }
}

int main() {
    int64_t arr[] = {38, 27, 43, 3, 9, 82, 10, 55, 1, 77};
    quicksort(arr, 0, 9);
    for (int i = 0; i < 10; i++) {
        printf("%lld\n", arr[i]);
    }
    return 0;
}
