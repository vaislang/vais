// Fibonacci - recursive and iterative
#include <stdio.h>
#include <stdint.h>

int64_t fib_rec(int64_t n) {
    if (n <= 1) return n;
    return fib_rec(n - 1) + fib_rec(n - 2);
}

int64_t fib_iter(int64_t n) {
    int64_t a = 0, b = 1;
    for (int64_t i = 0; i < n; i++) {
        int64_t t = a + b;
        a = b;
        b = t;
    }
    return a;
}

int main() {
    printf("%lld\n", fib_rec(20));
    printf("%lld\n", fib_iter(50));
    return 0;
}
