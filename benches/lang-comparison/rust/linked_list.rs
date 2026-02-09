// Singly linked list with basic operations
struct Node {
    value: i64,
    next: Option<Box<Node>>,
}

impl Node {
    fn new(val: i64) -> Self {
        Node { value: val, next: None }
    }
}

fn list_push(head: &mut Node, val: i64) {
    let mut new_node = Box::new(Node::new(val));
    new_node.next = head.next.take();
    head.next = Some(new_node);
}

fn list_len(head: &Node) -> i64 {
    let mut count = 0i64;
    let mut cur = &head.next;
    while let Some(node) = cur {
        count += 1;
        cur = &node.next;
    }
    count
}

fn list_sum(head: &Node) -> i64 {
    let mut total = 0i64;
    let mut cur = &head.next;
    while let Some(node) = cur {
        total += node.value;
        cur = &node.next;
    }
    total
}

fn main() {
    let mut head = Node::new(0);
    for i in 1..=10 {
        list_push(&mut head, i);
    }
    println!("{}", list_len(&head));
    println!("{}", list_sum(&head));
}
