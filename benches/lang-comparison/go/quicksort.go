// Quicksort with array partitioning
package main

import "fmt"

func partition(arr []int64, lo, hi int) int {
	pivot := arr[hi]
	i := lo
	for j := lo; j < hi; j++ {
		if arr[j] <= pivot {
			arr[i], arr[j] = arr[j], arr[i]
			i++
		}
	}
	arr[i], arr[hi] = arr[hi], arr[i]
	return i
}

func quicksort(arr []int64, lo, hi int) {
	if lo < hi {
		p := partition(arr, lo, hi)
		quicksort(arr, lo, p-1)
		quicksort(arr, p+1, hi)
	}
}

func main() {
	arr := []int64{38, 27, 43, 3, 9, 82, 10, 55, 1, 77}
	quicksort(arr, 0, len(arr)-1)
	for _, v := range arr {
		fmt.Println(v)
	}
}
