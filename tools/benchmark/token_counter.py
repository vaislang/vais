#!/usr/bin/env python3
"""
Vais v2 Token Benchmark Tool

Compares token counts between Python and Vais v2 code
using tiktoken (OpenAI's tokenizer) for accurate LLM token estimation.

Usage:
    python token_counter.py
    python token_counter.py --example hello_world
    python token_counter.py --file examples.json
"""

import json
import argparse
from dataclasses import dataclass
from typing import Optional

try:
    import tiktoken
    HAS_TIKTOKEN = True
except ImportError:
    HAS_TIKTOKEN = False
    print("Warning: tiktoken not installed. Using simple whitespace tokenizer.")
    print("Install with: pip install tiktoken")


@dataclass
class TokenResult:
    name: str
    python_code: str
    vais_code: str
    python_tokens: int
    vais_tokens: int
    savings_percent: float

    def to_dict(self):
        return {
            "name": self.name,
            "python_tokens": self.python_tokens,
            "vais_tokens": self.vais_tokens,
            "savings_percent": round(self.savings_percent, 1)
        }


class TokenCounter:
    def __init__(self, model: str = "gpt-4"):
        if HAS_TIKTOKEN:
            try:
                self.encoder = tiktoken.encoding_for_model(model)
            except KeyError:
                self.encoder = tiktoken.get_encoding("cl100k_base")
        else:
            self.encoder = None

    def count(self, code: str) -> int:
        """Count tokens in code string."""
        if self.encoder:
            return len(self.encoder.encode(code))
        else:
            # Simple fallback: split by whitespace and punctuation
            import re
            tokens = re.findall(r'\w+|[^\w\s]', code)
            return len(tokens)

    def compare(self, name: str, python_code: str, vais_code: str) -> TokenResult:
        """Compare token counts between Python and Vais."""
        python_tokens = self.count(python_code)
        vais_tokens = self.count(vais_code)

        if python_tokens > 0:
            savings = (1 - vais_tokens / python_tokens) * 100
        else:
            savings = 0

        return TokenResult(
            name=name,
            python_code=python_code,
            vais_code=vais_code,
            python_tokens=python_tokens,
            vais_tokens=vais_tokens,
            savings_percent=savings
        )


# Built-in examples for testing
EXAMPLES = {
    "hello_world": {
        "python": '''def hello():
    return "Hello, World!"''',
        "vais": '(fn hello [] :s "Hello, World!")'
    },

    "add_numbers": {
        "python": '''def add(a: int, b: int) -> int:
    return a + b''',
        "vais": '(fn add [a:i b:i] :i (+ a b))'
    },

    "fibonacci": {
        "python": '''def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)''',
        "vais": '''(fn fib [n:i] :i
  (? (<= n 1) n (+ (fib (- n 1)) (fib (- n 2)))))'''
    },

    "factorial": {
        "python": '''def factorial(n: int) -> int:
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result''',
        "vais": '''(fn fact [n:i] :i
  (loop [i n acc 1]
    (? (<= i 1) acc (recur (- i 1) (* acc i)))))'''
    },

    "filter_map": {
        "python": '''def get_active_emails(users: list[User]) -> list[str]:
    return [u.email.upper() for u in users if u.is_active]''',
        "vais": '''(fn active-emails [users:[User]] :[s]
  (-> users (filter .active) (map .email) (map upper)))'''
    },

    "categorize_age": {
        "python": '''def categorize_age(age: int) -> str:
    if age < 0:
        raise ValueError("Age cannot be negative")
    if age >= 18:
        return "adult"
    if age >= 13:
        return "teen"
    return "child"''',
        "vais": '''(fn categorize [age:i] :s
  (require (>= age 0) "Age cannot be negative")
  (cond
    (>= age 18) "adult"
    (>= age 13) "teen"
    :else "child"))'''
    },

    "max_number": {
        "python": '''def max_num(a: int, b: int) -> int:
    if a > b:
        return a
    return b''',
        "vais": '(fn max [a:i b:i] :i (? (> a b) a b))'
    },

    "sum_list": {
        "python": '''def sum_list(numbers: list[int]) -> int:
    total = 0
    for n in numbers:
        total += n
    return total''',
        "vais": '(fn sum-list [nums:[i]] :i (sum nums))'
    },

    "binary_search": {
        "python": '''def binary_search(arr: list[int], target: int) -> int | None:
    lo, hi = 0, len(arr) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        if arr[mid] == target:
            return mid
        elif arr[mid] < target:
            lo = mid + 1
        else:
            hi = mid - 1
    return None''',
        "vais": '''(fn bin-search [arr:[i] target:i] :?i
  (loop [lo 0 hi (- (len arr) 1)]
    (if (> lo hi)
      nil
      (let [mid (/ (+ lo hi) 2)
            val (nth arr mid)]
        (cond
          (= val target) mid
          (< val target) (recur (+ mid 1) hi)
          :else (recur lo (- mid 1)))))))'''
    },

    "quick_sort": {
        "python": '''def quicksort(arr: list[int]) -> list[int]:
    if len(arr) <= 1:
        return arr
    pivot = arr[0]
    less = [x for x in arr[1:] if x < pivot]
    greater = [x for x in arr[1:] if x >= pivot]
    return quicksort(less) + [pivot] + quicksort(greater)''',
        "vais": '''(fn qsort [arr:[i]] :[i]
  (if (<= (len arr) 1)
    arr
    (let [pivot (first arr)
          rest (drop 1 arr)
          less (filter (\ [x] (< x pivot)) rest)
          more (filter (\ [x] (>= x pivot)) rest)]
      (concat (qsort less) [pivot] (qsort more)))))'''
    }
}


def run_benchmark(examples: dict = None, verbose: bool = True) -> list[TokenResult]:
    """Run benchmark on all examples."""
    if examples is None:
        examples = EXAMPLES

    counter = TokenCounter()
    results = []

    if verbose:
        print("=" * 70)
        print("Vais v2 Token Benchmark")
        print("=" * 70)
        print()

    for name, code in examples.items():
        result = counter.compare(name, code["python"], code["vais"])
        results.append(result)

        if verbose:
            print(f"### {name}")
            print()
            print("Python:")
            print("```python")
            print(result.python_code)
            print("```")
            print(f"Tokens: {result.python_tokens}")
            print()
            print("Vais v2:")
            print("```lisp")
            print(result.vais_code)
            print("```")
            print(f"Tokens: {result.vais_tokens}")
            print()
            if result.savings_percent > 0:
                print(f"**Savings: {result.savings_percent:.1f}%**")
            else:
                print(f"**Overhead: {-result.savings_percent:.1f}%**")
            print()
            print("-" * 70)
            print()

    # Summary
    if verbose:
        print("=" * 70)
        print("SUMMARY")
        print("=" * 70)
        print()
        print(f"{'Example':<20} {'Python':>10} {'Vais v2':>10} {'Savings':>10}")
        print("-" * 50)

        total_python = 0
        total_vais = 0

        for r in results:
            total_python += r.python_tokens
            total_vais += r.vais_tokens
            savings_str = f"{r.savings_percent:+.1f}%"
            print(f"{r.name:<20} {r.python_tokens:>10} {r.vais_tokens:>10} {savings_str:>10}")

        print("-" * 50)
        total_savings = (1 - total_vais / total_python) * 100 if total_python > 0 else 0
        print(f"{'TOTAL':<20} {total_python:>10} {total_vais:>10} {total_savings:+.1f}%")
        print()

        if total_savings >= 40:
            print("✅ SUCCESS: Target 40% savings achieved!")
        elif total_savings >= 30:
            print("⚠️  CLOSE: 30-40% savings. Consider optimizations.")
        else:
            print("❌ BELOW TARGET: Less than 30% savings. Redesign needed.")

    return results


def main():
    parser = argparse.ArgumentParser(description="Vais v2 Token Benchmark Tool")
    parser.add_argument("--example", type=str, help="Run specific example")
    parser.add_argument("--file", type=str, help="Load examples from JSON file")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    parser.add_argument("--quiet", action="store_true", help="Minimal output")

    args = parser.parse_args()

    examples = EXAMPLES

    if args.file:
        with open(args.file, 'r') as f:
            examples = json.load(f)

    if args.example:
        if args.example in examples:
            examples = {args.example: examples[args.example]}
        else:
            print(f"Unknown example: {args.example}")
            print(f"Available: {', '.join(examples.keys())}")
            return

    results = run_benchmark(examples, verbose=not args.quiet and not args.json)

    if args.json:
        output = {
            "results": [r.to_dict() for r in results],
            "summary": {
                "total_python": sum(r.python_tokens for r in results),
                "total_vais": sum(r.vais_tokens for r in results),
                "average_savings": sum(r.savings_percent for r in results) / len(results) if results else 0
            }
        }
        print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
