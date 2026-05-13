# Fibonacci - recursive and iterative
def fib_rec(n: int) -> int:
    if n <= 1:
        return n
    return fib_rec(n - 1) + fib_rec(n - 2)

def fib_iter(n: int) -> int:
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

def main():
    print(fib_rec(20))
    print(fib_iter(50))

if __name__ == "__main__":
    main()
