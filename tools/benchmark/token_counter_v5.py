#!/usr/bin/env python3
"""
Vais v5 Token Benchmark - Clear & Explicit Syntax

설계 원칙:
1. 하나의 기호는 하나의 의미만
2. 모든 연산이 명시적
3. 컨텍스트 의존성 제거
4. 100% 생성 정확도 목표

문법:
  함수: fn name(params) = body
  타입: i=int, s=str, b=bool, f=float, [T]=array, ?T=optional
  맵: @expr - 각 요소에 적용
  필터: ?expr - 조건에 맞는 것만
  리듀스: /op - 접기 연산
  파이프: >> - 체이닝
  조건: if(cond, then, else)
  람다: {x: expr} - 명시적 람다
"""

import re
from dataclasses import dataclass


def count_tokens(code: str) -> int:
    """Simple token counter."""
    tokens = re.findall(r'\w+|[^\w\s]', code)
    return len(tokens)


EXAMPLES = {
    "hello_world": {
        "desc": "Hello World",
        "python": '''def hello():
    return "Hello, World!"''',
        "v5": 'fn hello() = "Hello, World!"',
    },

    "add_numbers": {
        "desc": "두 정수 더하기",
        "python": '''def add(a: int, b: int) -> int:
    return a + b''',
        "v5": 'fn add(a:i, b:i) = a + b',
    },

    "fibonacci": {
        "desc": "피보나치",
        "python": '''def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)''',
        "v5": 'fn fib(n:i) = if(n<2, n, fib(n-1) + fib(n-2))',
    },

    "factorial": {
        "desc": "팩토리얼",
        "python": '''def factorial(n: int) -> int:
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result''',
        "v5": 'fn fact(n:i) = if(n<2, 1, n * fact(n-1))',
    },

    "filter_map": {
        "desc": "활성 사용자 이메일",
        "python": '''def get_active_emails(users: list[User]) -> list[str]:
    return [u.email.upper() for u in users if u.is_active]''',
        "v5": 'fn emails(us:[U]) = us >> ?{x: x.active} >> @{x: x.email} >> @up',
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
        "v5": 'fn cat(age:i) = req(age>=0) >> if(age>=18, "adult", if(age>=13, "teen", "child"))',
    },

    "max_number": {
        "desc": "최대값",
        "python": '''def max_num(a: int, b: int) -> int:
    if a > b:
        return a
    return b''',
        "v5": 'fn max(a:i, b:i) = if(a>b, a, b)',
    },

    "sum_list": {
        "desc": "리스트 합계",
        "python": '''def sum_list(numbers: list[int]) -> int:
    total = 0
    for n in numbers:
        total += n
    return total''',
        "v5": 'fn sum(ns:[i]) = ns >> /+',
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
        "v5": 'fn bs(a:[i], t:i) = loop(lo:0, hi:len(a)-1, if(lo>hi, nil, let(m:(lo+hi)/2, if(a[m]==t, m, if(a[m]<t, rec(lo:m+1), rec(hi:m-1))))))',
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
        "v5": 'fn qs(a:[i]) = if(len(a)<2, a, let(p:a[0], r:a[1:], qs(r >> ?{x:x<p}) + [p] + qs(r >> ?{x:x>=p})))',
    },

    "map_reduce": {
        "desc": "총 매출",
        "python": '''def total_revenue(orders: list[Order]) -> float:
    return sum(o.quantity * o.price for o in orders)''',
        "v5": 'fn rev(os:[O]) = os >> @{o: o.qty * o.price} >> /+',
    },

    "is_even": {
        "desc": "짝수 확인",
        "python": '''def is_even(n: int) -> bool:
    return n % 2 == 0''',
        "v5": 'fn even(n:i) = n % 2 == 0',
    },

    "string_length": {
        "desc": "문자열 길이 확인",
        "python": '''def is_long(s: str) -> bool:
    return len(s) >= 5''',
        "v5": 'fn long(s:s) = len(s) >= 5',
    },

    "absolute": {
        "desc": "절대값",
        "python": '''def abs_val(n: int) -> int:
    if n < 0:
        return -n
    return n''',
        "v5": 'fn abs(n:i) = if(n<0, -n, n)',
    },

    "clamp": {
        "desc": "범위 제한",
        "python": '''def clamp(val: int, min_val: int, max_val: int) -> int:
    if val < min_val:
        return min_val
    if val > max_val:
        return max_val
    return val''',
        "v5": 'fn clamp(v:i, lo:i, hi:i) = if(v<lo, lo, if(v>hi, hi, v))',
    },

    "contains": {
        "desc": "포함 여부",
        "python": '''def contains(arr: list[int], target: int) -> bool:
    for x in arr:
        if x == target:
            return True
    return False''',
        "v5": 'fn has(a:[i], t:i) = t in a',
    },

    "reverse_string": {
        "desc": "문자열 뒤집기",
        "python": '''def reverse(s: str) -> str:
    return s[::-1]''',
        "v5": 'fn rev(s:s) = s >> flip',
    },

    "count_positive": {
        "desc": "양수 개수",
        "python": '''def count_pos(nums: list[int]) -> int:
    return len([x for x in nums if x > 0])''',
        "v5": 'fn cpos(ns:[i]) = ns >> ?{x: x>0} >> len',
    },

    "double_all": {
        "desc": "모두 2배",
        "python": '''def double_all(nums: list[int]) -> list[int]:
    return [x * 2 for x in nums]''',
        "v5": 'fn dbl(ns:[i]) = ns >> @{x: x*2}',
    },

    "first_positive": {
        "desc": "첫 양수",
        "python": '''def first_pos(nums: list[int]) -> int | None:
    for x in nums:
        if x > 0:
            return x
    return None''',
        "v5": 'fn fpos(ns:[i]) = ns >> ?{x: x>0} >> first',
    },

    "average": {
        "desc": "평균값",
        "python": '''def average(nums: list[int]) -> float:
    return sum(nums) / len(nums)''',
        "v5": 'fn avg(ns:[i]) = (ns >> /+) / len(ns)',
    },

    "gcd": {
        "desc": "최대공약수",
        "python": '''def gcd(a: int, b: int) -> int:
    if b == 0:
        return a
    return gcd(b, a % b)''',
        "v5": 'fn gcd(a:i, b:i) = if(b==0, a, gcd(b, a%b))',
    },

    "palindrome": {
        "desc": "팰린드롬",
        "python": '''def is_palindrome(s: str) -> bool:
    return s == s[::-1]''',
        "v5": 'fn palin(s:s) = s == (s >> flip)',
    },

    "unique": {
        "desc": "중복 제거",
        "python": '''def unique(nums: list[int]) -> list[int]:
    return list(set(nums))''',
        "v5": 'fn uniq(ns:[i]) = ns >> set',
    },

    "zip_sum": {
        "desc": "요소별 합",
        "python": '''def zip_sum(a: list[int], b: list[int]) -> list[int]:
    return [x + y for x, y in zip(a, b)]''',
        "v5": 'fn zsum(a:[i], b:[i]) = zip(a, b) >> @{p: p.0 + p.1}',
    },

    "is_prime": {
        "desc": "소수 판별",
        "python": '''def is_prime(n: int) -> bool:
    if n < 2:
        return False
    for i in range(2, n):
        if n % i == 0:
            return False
    return True''',
        "v5": 'fn prime(n:i) = if(n<2, false, range(2,n) >> ?{i: n%i==0} >> len == 0)',
    },
}


def run_benchmark():
    print("=" * 80)
    print("Vais v5 Token Benchmark - Clear & Explicit Syntax")
    print("=" * 80)
    print()
    print("문법 규칙:")
    print("  fn name(params) = body    함수 정의")
    print("  @{x: expr}                맵 (명시적 람다)")
    print("  ?{x: expr}                필터 (명시적 람다)")
    print("  /op                       리듀스")
    print("  >>                        파이프")
    print("  if(cond, then, else)      조건")
    print()
    print("-" * 80)

    total_py = 0
    total_v5 = 0
    results = []

    for name, data in EXAMPLES.items():
        py_code = data["python"]
        v5_code = data["v5"]

        py_tokens = count_tokens(py_code)
        v5_tokens = count_tokens(v5_code)

        total_py += py_tokens
        total_v5 += v5_tokens

        savings = (1 - v5_tokens / py_tokens) * 100
        results.append((name, data["desc"], py_tokens, v5_tokens, savings, v5_code))

    # 결과 출력
    print(f"{'Name':<18} {'Desc':<15} {'Python':>8} {'v5':>8} {'Savings':>10}")
    print("-" * 65)

    for name, desc, py_t, v5_t, sav, _ in results:
        marker = "✓" if sav >= 40 else "○" if sav >= 20 else "✗"
        desc_short = desc[:14]
        print(f"{name:<18} {desc_short:<15} {py_t:>8} {v5_t:>8} {sav:>+9.1f}% {marker}")

    print("-" * 65)
    total_savings = (1 - total_v5 / total_py) * 100
    print(f"{'TOTAL':<18} {'':<15} {total_py:>8} {total_v5:>8} {total_savings:>+9.1f}%")
    print()

    if total_savings >= 40:
        print("✅ SUCCESS: 40% 이상 절감 달성!")
    elif total_savings >= 30:
        print("⚠️  CLOSE: 30-40% 절감. 추가 최적화 필요.")
    else:
        print("❌ FAIL: 30% 미만. 재설계 필요.")

    print()
    print("=" * 80)
    print("v5 문법 명세")
    print("=" * 80)
    print("""
## 기본 구조
fn name(param:type, ...) = body

## 타입
i    = int
s    = string
b    = bool
f    = float
[T]  = array of T
?T   = optional T

## 연산자 (각각 하나의 의미만)
>>   = 파이프 (데이터 흐름)
@    = 맵 (각 요소에 적용)
?    = 필터 (조건에 맞는 것만)
/    = 리듀스 (접기)

## 람다
{x: expr}           단일 인자
{x, y: expr}        다중 인자

## 제어
if(cond, then, else)    조건
let(name:val, body)     바인딩
loop(init, body)        반복
rec(updates)            재귀 호출 (loop 내)
req(cond)               assertion

## 빌트인
len, first, last, flip, set, zip, range
""")

    print()
    print("=" * 80)
    print("예제 상세")
    print("=" * 80)

    highlight = ["add_numbers", "filter_map", "sum_list", "quick_sort", "is_prime"]
    for name in highlight:
        data = EXAMPLES[name]
        py_t = count_tokens(data["python"])
        v5_t = count_tokens(data["v5"])
        sav = (1 - v5_t / py_t) * 100

        print(f"\n### {name} ({data['desc']})")
        print(f"\nPython ({py_t} tokens):")
        print(data["python"])
        print(f"\nv5 ({v5_t} tokens, {sav:+.0f}%):")
        print(data["v5"])
        print("-" * 40)


if __name__ == "__main__":
    run_benchmark()
