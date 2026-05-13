# Singly linked list with basic operations
from dataclasses import dataclass
from typing import Optional

@dataclass
class Node:
    value: int
    next: Optional["Node"] = None

def list_push(head: Node, val: int) -> None:
    new_node = Node(value=val, next=head.next)
    head.next = new_node

def list_len(head: Node) -> int:
    count = 0
    cur = head.next
    while cur is not None:
        count += 1
        cur = cur.next
    return count

def list_sum(head: Node) -> int:
    total = 0
    cur = head.next
    while cur is not None:
        total += cur.value
        cur = cur.next
    return total

def main():
    head = Node(value=0)
    for i in range(1, 11):
        list_push(head, i)
    print(list_len(head))
    print(list_sum(head))

if __name__ == "__main__":
    main()
