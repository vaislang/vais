// Fibonacci - recursive and iterative
package main

import "fmt"

func fibRec(n int64) int64 {
	if n <= 1 {
		return n
	}
	return fibRec(n-1) + fibRec(n-2)
}

func fibIter(n int64) int64 {
	var a, b int64 = 0, 1
	for i := int64(0); i < n; i++ {
		a, b = b, a+b
	}
	return a
}

func main() {
	fmt.Println(fibRec(20))
	fmt.Println(fibIter(50))
}
