#!/usr/bin/env python3
"""
AOEL v4 Token Benchmark - Functional Chaining Style

Option C: 극단적 단순화 - 함수형 체이닝
"""

import re
from dataclasses import dataclass


def count_tokens(code: str) -> int:
    """Simple token counter."""
    tokens = re.findall(r'\w+|[^\w\s]', code)
    return len(tokens)


EXAMPLES = {
    "hello_world": {
        "desc": "Hello World 반환",
        "python": '''def hello():
    return "Hello, World!"''',

        # 함수형 체이닝: 상수 함수
        "v4": 'hello = "Hello, World!"',
    },

    "add_numbers": {
        "desc": "두 정수 더하기",
        "python": '''def add(a: int, b: int) -> int:
    return a + b''',

        # 체이닝: 이항 연산자 그대로
        "v4": 'add a:i b:i = a+b',
    },

    "fibonacci": {
        "desc": "피보나치",
        "python": '''def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)''',

        "v4": 'fib n:i = n<2?n:fib(n-1)+fib(n-2)',
    },

    "factorial": {
        "desc": "팩토리얼",
        "python": '''def factorial(n: int) -> int:
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result''',

        "v4": 'fact n:i = n<2?1:n*fact(n-1)',
    },

    "filter_map": {
        "desc": "활성 사용자 이메일 추출",
        "python": '''def get_active_emails(users: list[User]) -> list[str]:
    return [u.email.upper() for u in users if u.is_active]''',

        # 파이프 체이닝
        "v4": 'emails us:[U] = us|active?|email|UP',
    },

    "categorize_age": {
        "desc": "나이 분류",
        "python": '''def categorize_age(age: int) -> str:
    if age < 0:
        raise ValueError("Age cannot be negative")
    if age >= 18:
        return "adult"
    if age >= 13:
        return "teen"
    return "child"''',

        # 다중 조건
        "v4": 'cat age:i = age<0!|age>17?"adult"|age>12?"teen"|"child"',
    },

    "max_number": {
        "desc": "최대값",
        "python": '''def max_num(a: int, b: int) -> int:
    if a > b:
        return a
    return b''',

        "v4": 'max a:i b:i = a>b?a:b',
    },

    "sum_list": {
        "desc": "리스트 합계",
        "python": '''def sum_list(numbers: list[int]) -> int:
    total = 0
    for n in numbers:
        total += n
    return total''',

        # 리듀스 연산자
        "v4": 'sum ns:[i] = ns|+',
    },

    "binary_search": {
        "desc": "이진 탐색",
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

        # 복잡한 로직은 축약에 한계
        "v4": 'bs a:[i] t:i = loop lo=0 hi=#a-1|lo>hi?nil|m=(lo+hi)/2|a[m]=t?m|a[m]<t?lo=m+1:hi=m-1',
    },

    "quick_sort": {
        "desc": "퀵소트",
        "python": '''def quicksort(arr: list[int]) -> list[int]:
    if len(arr) <= 1:
        return arr
    pivot = arr[0]
    less = [x for x in arr[1:] if x < pivot]
    greater = [x for x in arr[1:] if x >= pivot]
    return quicksort(less) + [pivot] + quicksort(greater)''',

        "v4": 'qs a:[i] = #a<2?a:qs(a[1:]|<a[0])+a[0]+qs(a[1:]|>=a[0])',
    },

    "map_reduce": {
        "desc": "총 매출 계산",
        "python": '''def total_revenue(orders: list[Order]) -> float:
    return sum(o.quantity * o.price for o in orders)''',

        "v4": 'rev os:[O] = os|qty*price|+',
    },

    # 추가 예제
    "is_even": {
        "desc": "짝수 확인",
        "python": '''def is_even(n: int) -> bool:
    return n % 2 == 0''',

        "v4": 'even n:i = n%2=0',
    },

    "string_length": {
        "desc": "문자열 길이 확인",
        "python": '''def is_long(s: str) -> bool:
    return len(s) >= 5''',

        "v4": 'long s:s = #s>4',
    },

    "absolute": {
        "desc": "절대값",
        "python": '''def abs_val(n: int) -> int:
    if n < 0:
        return -n
    return n''',

        "v4": 'abs n:i = n<0?-n:n',
    },

    "clamp": {
        "desc": "범위 제한",
        "python": '''def clamp(val: int, min_val: int, max_val: int) -> int:
    if val < min_val:
        return min_val
    if val > max_val:
        return max_val
    return val''',

        "v4": 'clamp v:i lo:i hi:i = v<lo?lo|v>hi?hi|v',
    },

    "contains": {
        "desc": "리스트에 포함 여부",
        "python": '''def contains(arr: list[int], target: int) -> bool:
    for x in arr:
        if x == target:
            return True
    return False''',

        "v4": 'has a:[i] t:i = t in a',
    },

    "reverse_string": {
        "desc": "문자열 뒤집기",
        "python": '''def reverse(s: str) -> str:
    return s[::-1]''',

        "v4": 'rev s:s = s|flip',
    },

    "count_positive": {
        "desc": "양수 개수",
        "python": '''def count_pos(nums: list[int]) -> int:
    return len([x for x in nums if x > 0])''',

        "v4": 'cpos ns:[i] = ns|>0?|#',
    },

    "double_all": {
        "desc": "모든 요소 2배",
        "python": '''def double_all(nums: list[int]) -> list[int]:
    return [x * 2 for x in nums]''',

        "v4": 'dbl ns:[i] = ns|*2',
    },

    "first_positive": {
        "desc": "첫 양수 찾기",
        "python": '''def first_pos(nums: list[int]) -> int | None:
    for x in nums:
        if x > 0:
            return x
    return None''',

        "v4": 'fpos ns:[i] = ns|>0?|[0]',
    },
}


def run_benchmark():
    print("=" * 80)
    print("AOEL v4 Token Benchmark - Functional Chaining")
    print("=" * 80)
    print()
    print("문법: name params = body")
    print("체이닝: |op 는 파이프, ? 는 필터/조건, | 는 리듀스")
    print()
    print("-" * 80)

    total_py = 0
    total_v4 = 0
    results = []

    for name, data in EXAMPLES.items():
        py_code = data["python"]
        v4_code = data["v4"]

        py_tokens = count_tokens(py_code)
        v4_tokens = count_tokens(v4_code)

        total_py += py_tokens
        total_v4 += v4_tokens

        savings = (1 - v4_tokens / py_tokens) * 100
        results.append((name, data["desc"], py_tokens, v4_tokens, savings))

    # 결과 출력
    print(f"{'Name':<18} {'Desc':<20} {'Python':>8} {'v4':>8} {'Savings':>10}")
    print("-" * 70)

    for name, desc, py_t, v4_t, sav in results:
        marker = "✓" if sav >= 40 else "○" if sav >= 20 else "✗"
        print(f"{name:<18} {desc:<20} {py_t:>8} {v4_t:>8} {sav:>+9.1f}% {marker}")

    print("-" * 70)
    total_savings = (1 - total_v4 / total_py) * 100
    print(f"{'TOTAL':<18} {'':<20} {total_py:>8} {total_v4:>8} {total_savings:>+9.1f}%")
    print()

    if total_savings >= 40:
        print("✅ SUCCESS: 40% 이상 절감 달성!")
    elif total_savings >= 30:
        print("⚠️  CLOSE: 30-40% 절감. 추가 최적화 필요.")
    else:
        print("❌ FAIL: 30% 미만. 재설계 필요.")

    print()
    print("=" * 80)
    print("EXAMPLES")
    print("=" * 80)

    # 몇 가지 예제 상세 출력
    highlight = ["add_numbers", "filter_map", "fibonacci", "sum_list", "quick_sort"]
    for name in highlight:
        data = EXAMPLES[name]
        py_t = count_tokens(data["python"])
        v4_t = count_tokens(data["v4"])
        sav = (1 - v4_t / py_t) * 100

        print(f"\n### {name} ({data['desc']})")
        print(f"\nPython ({py_t} tokens):")
        print(data["python"])
        print(f"\nv4 ({v4_t} tokens, {sav:+.0f}%):")
        print(data["v4"])
        print("-" * 40)


if __name__ == "__main__":
    run_benchmark()
