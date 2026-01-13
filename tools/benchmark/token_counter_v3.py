#!/usr/bin/env python3
"""
Vais v3 Token Benchmark - Extreme Compression Experiment

Testing multiple compression strategies:
- v3a: Symbol-heavy (minimal keywords)
- v3b: Delimiter-based (pipe/colon)
- v3c: Postfix/Stack-based (Forth-like)
- v3d: JSON-compact
"""

import json
import re
from dataclasses import dataclass


@dataclass
class TokenResult:
    name: str
    tokens: int
    code: str


def count_tokens(code: str) -> int:
    """Simple token counter (whitespace + punctuation split)."""
    tokens = re.findall(r'\w+|[^\w\s]', code)
    return len(tokens)


def compare_all(name: str, versions: dict) -> dict:
    """Compare all versions."""
    results = {}
    for ver, code in versions.items():
        results[ver] = TokenResult(name, count_tokens(code), code)
    return results


# =============================================================================
# EXAMPLES - Multiple compression strategies
# =============================================================================

EXAMPLES = {
    "hello_world": {
        "python": '''def hello():
    return "Hello, World!"''',

        # v3a: Symbol-heavy - 괄호 최소화, 기호로 구조
        "v3a": 'F hello:s="Hello, World!"',

        # v3b: Pipe-based - 파이프로 구분
        "v3b": 'hello||s|"Hello, World!"',

        # v3c: Stack-based (Forth-like) - 후위 표기
        "v3c": '"Hello, World!"->s:hello',

        # v3d: JSON-compact - 최소 JSON
        "v3d": '{"n":"hello","o":"s","b":"Hello, World!"}',
    },

    "add_numbers": {
        "python": '''def add(a: int, b: int) -> int:
    return a + b''',

        "v3a": 'F add(a,b:i):i=a+b',

        "v3b": 'add|a:i,b:i|i|a+b',

        "v3c": 'a b +->i:add',

        "v3d": '{"n":"add","i":["a:i","b:i"],"o":"i","b":"a+b"}',
    },

    "fibonacci": {
        "python": '''def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)''',

        "v3a": 'F fib(n:i):i=n<=1?n:fib(n-1)+fib(n-2)',

        "v3b": 'fib|n:i|i|n<=1?n:fib(n-1)+fib(n-2)',

        "v3c": 'n 1<=?n,n 1-fib n 2-fib+->i:fib',

        "v3d": '{"n":"fib","i":["n:i"],"o":"i","b":"n<=1?n:fib(n-1)+fib(n-2)"}',
    },

    "factorial": {
        "python": '''def factorial(n: int) -> int:
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result''',

        # 재귀 버전으로 단순화
        "v3a": 'F fact(n:i):i=n<=1?1:n*fact(n-1)',

        "v3b": 'fact|n:i|i|n<=1?1:n*fact(n-1)',

        "v3c": 'n 1<=?1,n n 1-fact*->i:fact',

        "v3d": '{"n":"fact","i":["n:i"],"o":"i","b":"n<=1?1:n*fact(n-1)"}',
    },

    "filter_map": {
        "python": '''def get_active_emails(users: list[User]) -> list[str]:
    return [u.email.upper() for u in users if u.is_active]''',

        "v3a": 'F active_emails(users:[U]):[s]=users|.active|.email|UP',

        "v3b": 'active_emails|users:[U]|[s]|users>.active>.email>UP',

        "v3c": 'users .active? .email@ UP@->[s]:active_emails',

        "v3d": '{"n":"active_emails","i":["users:[U]"],"o":"[s]","p":["?.active",".email","UP"]}',
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

        "v3a": 'F cat(age:i):s!age>=0=age>=18?"adult":age>=13?"teen":"child"',

        "v3b": 'cat|age:i|s|!age>=0|age>=18?"adult":age>=13?"teen":"child"',

        "v3c": 'age 0>=! age 18>=?"adult",age 13>=?"teen","child"->s:cat',

        "v3d": '{"n":"cat","i":["age:i"],"o":"s","r":"age>=0","b":"age>=18?adult:age>=13?teen:child"}',
    },

    "max_number": {
        "python": '''def max_num(a: int, b: int) -> int:
    if a > b:
        return a
    return b''',

        "v3a": 'F max(a,b:i):i=a>b?a:b',

        "v3b": 'max|a:i,b:i|i|a>b?a:b',

        "v3c": 'a b>?a,b->i:max',

        "v3d": '{"n":"max","i":["a:i","b:i"],"o":"i","b":"a>b?a:b"}',
    },

    "sum_list": {
        "python": '''def sum_list(numbers: list[int]) -> int:
    total = 0
    for n in numbers:
        total += n
    return total''',

        "v3a": 'F sum(ns:[i]):i=+ns',

        "v3b": 'sum|ns:[i]|i|+ns',

        "v3c": 'ns +/->i:sum',

        "v3d": '{"n":"sum","i":["ns:[i]"],"o":"i","b":"+ns"}',
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

        "v3a": 'F bs(a:[i],t:i):?i=L(0,#a-1,lo<=hi,m=(lo+hi)/2;a[m]=t?m:a[m]<t?lo=m+1:hi=m-1)',

        "v3b": 'bs|a:[i],t:i|?i|L:lo=0,hi=#a-1;lo<=hi;m=(lo+hi)/2,a[m]=t?m:a[m]<t?(lo=m+1):(hi=m-1)',

        "v3c": '0 #a 1- W lo hi<=; lo hi+2/ m= a@m t=?m,a@m t<?m 1+>lo,m 1->hi->?i:bs',

        "v3d": '{"n":"bs","i":["a:[i]","t:i"],"o":"?i","b":"loop(0,len-1,mid,cmp)"}',
    },

    "quick_sort": {
        "python": '''def quicksort(arr: list[int]) -> list[int]:
    if len(arr) <= 1:
        return arr
    pivot = arr[0]
    less = [x for x in arr[1:] if x < pivot]
    greater = [x for x in arr[1:] if x >= pivot]
    return quicksort(less) + [pivot] + quicksort(greater)''',

        "v3a": 'F qs(a:[i]):[i]=#a<=1?a:qs(a[1:]|<a[0])+[a[0]]+qs(a[1:]|>=a[0])',

        "v3b": 'qs|a:[i]|[i]|#a<=1?a:qs(a[1:]<a[0])+[a[0]]+qs(a[1:]>=a[0])',

        "v3c": '#a 1<=?a,a 1> a@0< ? qs a@0] a 1> a@0>=? qs ++->[i]:qs',

        "v3d": '{"n":"qs","i":["a:[i]"],"o":"[i]","b":"len<=1?a:qs(less)+[p]+qs(more)"}',
    },

    # 추가 예제: 더 복잡한 케이스
    "map_reduce": {
        "python": '''def total_revenue(orders: list[Order]) -> float:
    return sum(o.quantity * o.price for o in orders)''',

        "v3a": 'F rev(os:[O]):f=os|.qty*.price|+',

        "v3b": 'rev|os:[O]|f|os>.qty*.price>+',

        "v3c": 'os .qty .price *@ +/->f:rev',

        "v3d": '{"n":"rev","i":["os:[O]"],"o":"f","p":[".qty*.price","+"]}',
    },
}


def run_benchmark():
    print("=" * 80)
    print("Vais v3 Extreme Compression Benchmark")
    print("=" * 80)
    print()

    all_results = {}
    totals = {"python": 0, "v3a": 0, "v3b": 0, "v3c": 0, "v3d": 0}

    for name, versions in EXAMPLES.items():
        results = compare_all(name, versions)
        all_results[name] = results

        print(f"### {name}")
        print()

        # Python baseline
        py = results["python"]
        print(f"Python ({py.tokens} tokens):")
        print(f"  {py.code[:60]}..." if len(py.code) > 60 else f"  {py.code}")
        print()

        # All v3 versions
        for ver in ["v3a", "v3b", "v3c", "v3d"]:
            r = results[ver]
            savings = (1 - r.tokens / py.tokens) * 100
            marker = "✓" if savings > 30 else "○" if savings > 0 else "✗"
            print(f"{ver} ({r.tokens} tokens, {savings:+.0f}%) {marker}")
            print(f"  {r.code}")

        print()
        print("-" * 80)
        print()

        for ver in totals:
            totals[ver] += results[ver].tokens

    # Summary
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print()
    print(f"{'Version':<10} {'Total Tokens':>15} {'vs Python':>15} {'Status':>10}")
    print("-" * 55)

    py_total = totals["python"]
    for ver, total in totals.items():
        if ver == "python":
            print(f"{ver:<10} {total:>15} {'baseline':>15} {'---':>10}")
        else:
            savings = (1 - total / py_total) * 100
            status = "✓ PASS" if savings >= 40 else "○ CLOSE" if savings >= 30 else "✗ FAIL"
            print(f"{ver:<10} {total:>15} {savings:>+14.1f}% {status:>10}")

    print()
    print("=" * 80)
    print("DESIGN ANALYSIS")
    print("=" * 80)
    print()
    print("v3a (Symbol-heavy): F name(args):type=body")
    print("    + 읽기 쉬움, 기존 문법과 유사")
    print("    - 괄호/콜론 오버헤드")
    print()
    print("v3b (Pipe-based): name|args|type|body")
    print("    + 구분 명확, 파싱 단순")
    print("    - 파이프가 여러 의미로 사용됨")
    print()
    print("v3c (Stack-based): args ops->type:name")
    print("    + 최소 토큰")
    print("    - 읽기 어려움, AI 생성 난이도 높음")
    print()
    print("v3d (JSON-compact): {n,i,o,b}")
    print("    + 구조적, AI 친화적")
    print("    - 따옴표/중괄호 오버헤드")


if __name__ == "__main__":
    run_benchmark()
