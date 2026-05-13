// Singly linked list with basic operations
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

typedef struct Node {
    int64_t value;
    struct Node *next;
} Node;

Node *node_new(int64_t val) {
    Node *n = (Node *)malloc(sizeof(Node));
    n->value = val;
    n->next = NULL;
    return n;
}

void list_push(Node *head, int64_t val) {
    Node *n = node_new(val);
    n->next = head->next;
    head->next = n;
}

int64_t list_len(const Node *head) {
    int64_t count = 0;
    const Node *cur = head->next;
    while (cur != NULL) {
        count++;
        cur = cur->next;
    }
    return count;
}

int64_t list_sum(const Node *head) {
    int64_t total = 0;
    const Node *cur = head->next;
    while (cur != NULL) {
        total += cur->value;
        cur = cur->next;
    }
    return total;
}

int main() {
    Node head = {0, NULL};
    for (int64_t i = 1; i <= 10; i++) {
        list_push(&head, i);
    }
    printf("%lld\n", list_len(&head));
    printf("%lld\n", list_sum(&head));
    return 0;
}
