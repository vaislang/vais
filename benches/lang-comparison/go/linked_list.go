// Singly linked list with basic operations
package main

import "fmt"

type Node struct {
	Value int64
	Next  *Node
}

func newNode(val int64) *Node {
	return &Node{Value: val, Next: nil}
}

func listPush(head *Node, val int64) {
	n := newNode(val)
	n.Next = head.Next
	head.Next = n
}

func listLen(head *Node) int64 {
	var count int64
	cur := head.Next
	for cur != nil {
		count++
		cur = cur.Next
	}
	return count
}

func listSum(head *Node) int64 {
	var total int64
	cur := head.Next
	for cur != nil {
		total += cur.Value
		cur = cur.Next
	}
	return total
}

func main() {
	head := &Node{Value: 0, Next: nil}
	for i := int64(1); i <= 10; i++ {
		listPush(head, i)
	}
	fmt.Println(listLen(head))
	fmt.Println(listSum(head))
}
