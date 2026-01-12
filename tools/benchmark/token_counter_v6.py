#!/usr/bin/env python3
"""
AOEL v6 Token Benchmark - Clear + Efficient

설계 원칙:
1. 명확성 유지 (각 기호는 하나의 의미)
2. 토큰 최소화 (불필요한 키워드 제거)
3. 100% 생성 정확도 목표

문법 변경:
- fn 키워드 제거: name(params) = body
- >> 를 . 로: 체이닝은 점으로
- 람다 축약: ?active (필드면 자동), @(_ * 2) (언더스코어 람다)
- if를 ? : 로: cond ? then : else
- 타입 생략 가능: 추론 가능하면 생략

핵심 연산자:
  .   = 체이닝/파이프 (1토큰)
  @   = 맵
  ?   = 필터 (뒤에 조건) 또는 삼항 (cond ? a : b)
  /   = 리듀스
  _   = 람다 인자 (암묵적 단일 인자)
"""

import re


def count_tokens(code: str) -> int:
    """Simple token counter."""
    tokens = re.findall(r'\w+|[^\w\s]', code)
    return len(tokens)


EXAMPLES = {
    "hello_world": {
        "desc": "Hello World",
        "python": '''def hello():
    return "Hello, World!"''',
        "v6": 'hello() = "Hello, World!"',
    },

    "add_numbers": {
        "desc": "두 정수 더하기",
        "python": '''def add(a: int, b: int) -> int:
    return a + b''',
        "v6": 'add(a:i, b:i) = a + b',
    },

    "fibonacci": {
        "desc": "피보나치",
        "python": '''def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)''',
        "v6": 'fib(n:i) = n<2 ? n : fib(n-1)+fib(n-2)',
    },

    "factorial": {
        "desc": "팩토리얼",
        "python": '''def factorial(n: int) -> int:
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result''',
        "v6": 'fact(n:i) = n<2 ? 1 : n*fact(n-1)',
    },

    "filter_map": {
        "desc": "활성 사용자 이메일",
        "python": '''def get_active_emails(users: list[User]) -> list[str]:
    return [u.email.upper() for u in users if u.is_active]''',
        # ?field = 필터(해당 필드가 truthy), @field = 맵(해당 필드 추출), @fn = 맵(함수 적용)
        "v6": 'emails(us:[U]) = us.?active.@email.@up',
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
        "v6": 'cat(age:i) = age<0 ? err : age>=18 ? "adult" : age>=13 ? "teen" : "child"',
    },

    "max_number": {
        "desc": "최대값",
        "python": '''def max_num(a: int, b: int) -> int:
    if a > b:
        return a
    return b''',
        "v6": 'max(a:i, b:i) = a>b ? a : b',
    },

    "sum_list": {
        "desc": "리스트 합계",
        "python": '''def sum_list(numbers: list[int]) -> int:
    total = 0
    for n in numbers:
        total += n
    return total''',
        "v6": 'sum(ns:[i]) = ns./+',
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
        # 복잡한 로직은 let으로 바인딩
        "v6": 'bs(a:[i], t:i, lo:i=0, hi:i=#a-1) = lo>hi ? nil : let m=(lo+hi)/2 : a[m]==t ? m : a[m]<t ? bs(a,t,m+1,hi) : bs(a,t,lo,m-1)',
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
        "v6": 'qs(a:[i]) = #a<2 ? a : let p=a[0], r=a[1:] : qs(r.?(_<p)) + [p] + qs(r.?(_>=p))',
    },

    "map_reduce": {
        "desc": "총 매출",
        "python": '''def total_revenue(orders: list[Order]) -> float:
    return sum(o.quantity * o.price for o in orders)''',
        "v6": 'rev(os:[O]) = os.@(_.qty * _.price)./+',
    },

    "is_even": {
        "desc": "짝수 확인",
        "python": '''def is_even(n: int) -> bool:
    return n % 2 == 0''',
        "v6": 'even(n:i) = n%2==0',
    },

    "string_length": {
        "desc": "문자열 길이 확인",
        "python": '''def is_long(s: str) -> bool:
    return len(s) >= 5''',
        "v6": 'long(s:s) = #s>=5',
    },

    "absolute": {
        "desc": "절대값",
        "python": '''def abs_val(n: int) -> int:
    if n < 0:
        return -n
    return n''',
        "v6": 'abs(n:i) = n<0 ? -n : n',
    },

    "clamp": {
        "desc": "범위 제한",
        "python": '''def clamp(val: int, min_val: int, max_val: int) -> int:
    if val < min_val:
        return min_val
    if val > max_val:
        return max_val
    return val''',
        "v6": 'clamp(v:i, lo:i, hi:i) = v<lo ? lo : v>hi ? hi : v',
    },

    "contains": {
        "desc": "포함 여부",
        "python": '''def contains(arr: list[int], target: int) -> bool:
    for x in arr:
        if x == target:
            return True
    return False''',
        "v6": 'has(a:[i], t:i) = t in a',
    },

    "reverse_string": {
        "desc": "문자열 뒤집기",
        "python": '''def reverse(s: str) -> str:
    return s[::-1]''',
        "v6": 'rev(s:s) = s.flip',
    },

    "count_positive": {
        "desc": "양수 개수",
        "python": '''def count_pos(nums: list[int]) -> int:
    return len([x for x in nums if x > 0])''',
        "v6": 'cpos(ns:[i]) = ns.?(_>0).#',
    },

    "double_all": {
        "desc": "모두 2배",
        "python": '''def double_all(nums: list[int]) -> list[int]:
    return [x * 2 for x in nums]''',
        "v6": 'dbl(ns:[i]) = ns.@(_*2)',
    },

    "first_positive": {
        "desc": "첫 양수",
        "python": '''def first_pos(nums: list[int]) -> int | None:
    for x in nums:
        if x > 0:
            return x
    return None''',
        "v6": 'fpos(ns:[i]) = ns.?(_>0).first',
    },

    "average": {
        "desc": "평균값",
        "python": '''def average(nums: list[int]) -> float:
    return sum(nums) / len(nums)''',
        "v6": 'avg(ns:[i]) = ns./+ / #ns',
    },

    "gcd": {
        "desc": "최대공약수",
        "python": '''def gcd(a: int, b: int) -> int:
    if b == 0:
        return a
    return gcd(b, a % b)''',
        "v6": 'gcd(a:i, b:i) = b==0 ? a : gcd(b, a%b)',
    },

    "palindrome": {
        "desc": "팰린드롬",
        "python": '''def is_palindrome(s: str) -> bool:
    return s == s[::-1]''',
        "v6": 'palin(s:s) = s == s.flip',
    },

    "unique": {
        "desc": "중복 제거",
        "python": '''def unique(nums: list[int]) -> list[int]:
    return list(set(nums))''',
        "v6": 'uniq(ns:[i]) = ns.set',
    },

    "zip_sum": {
        "desc": "요소별 합",
        "python": '''def zip_sum(a: list[int], b: list[int]) -> list[int]:
    return [x + y for x, y in zip(a, b)]''',
        "v6": 'zsum(a:[i], b:[i]) = zip(a,b).@(_.0 + _.1)',
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
        "v6": 'prime(n:i) = n<2 ? false : (2..n).?(_ | n%_==0).# == 0',
    },

    # 추가 예제
    "flatten": {
        "desc": "중첩 리스트 평탄화",
        "python": '''def flatten(lists: list[list[int]]) -> list[int]:
    result = []
    for lst in lists:
        result.extend(lst)
    return result''',
        "v6": 'flat(ls:[[i]]) = ls.flatten',
    },

    "all_positive": {
        "desc": "모두 양수인지",
        "python": '''def all_positive(nums: list[int]) -> bool:
    for n in nums:
        if n <= 0:
            return False
    return True''',
        "v6": 'allpos(ns:[i]) = ns.all(_>0)',
    },

    "any_negative": {
        "desc": "음수가 있는지",
        "python": '''def any_negative(nums: list[int]) -> bool:
    for n in nums:
        if n < 0:
            return True
    return False''',
        "v6": 'anyneg(ns:[i]) = ns.any(_<0)',
    },

    "min_value": {
        "desc": "최소값",
        "python": '''def min_val(nums: list[int]) -> int:
    return min(nums)''',
        "v6": 'minv(ns:[i]) = ns./min',
    },

    "product": {
        "desc": "모든 요소 곱",
        "python": '''def product(nums: list[int]) -> int:
    result = 1
    for n in nums:
        result *= n
    return result''',
        "v6": 'prod(ns:[i]) = ns./*',
    },
}


def run_benchmark():
    print("=" * 80)
    print("AOEL v6 Token Benchmark - Clear + Efficient")
    print("=" * 80)
    print()
    print("핵심 문법:")
    print("  name(params) = body     함수 정의 (fn 키워드 없음)")
    print("  .                       체이닝 (1토큰)")
    print("  @expr 또는 @(_.expr)    맵")
    print("  ?cond 또는 ?(_>0)       필터")
    print("  /op                     리듀스 (/+, /*, /min, /max)")
    print("  a ? b : c               삼항 조건")
    print("  _                       람다의 암묵적 인자")
    print("  #                       길이 (len)")
    print()
    print("-" * 80)

    total_py = 0
    total_v6 = 0
    results = []

    for name, data in EXAMPLES.items():
        py_code = data["python"]
        v6_code = data["v6"]

        py_tokens = count_tokens(py_code)
        v6_tokens = count_tokens(v6_code)

        total_py += py_tokens
        total_v6 += v6_tokens

        savings = (1 - v6_tokens / py_tokens) * 100
        results.append((name, data["desc"], py_tokens, v6_tokens, savings, v6_code))

    # 결과 출력
    print(f"{'Name':<18} {'Desc':<15} {'Python':>8} {'v6':>8} {'Savings':>10}")
    print("-" * 65)

    for name, desc, py_t, v6_t, sav, _ in results:
        marker = "✓" if sav >= 40 else "○" if sav >= 20 else "✗"
        desc_short = desc[:14]
        print(f"{name:<18} {desc_short:<15} {py_t:>8} {v6_t:>8} {sav:>+9.1f}% {marker}")

    print("-" * 65)
    total_savings = (1 - total_v6 / total_py) * 100
    print(f"{'TOTAL':<18} {'':<15} {total_py:>8} {total_v6:>8} {total_savings:>+9.1f}%")
    print()

    if total_savings >= 40:
        print("✅ SUCCESS: 40% 이상 절감 달성!")
    elif total_savings >= 30:
        print("⚠️  CLOSE: 30-40% 절감. 추가 최적화 가능.")
    else:
        print("❌ FAIL: 30% 미만. 재설계 필요.")

    print()
    print("=" * 80)
    print("v6 문법 명세")
    print("=" * 80)
    print("""
## 함수 정의
name(param:type, ...) = body

## 타입 (축약)
i     = int
s     = string
b     = bool
f     = float
[T]   = array of T
?T    = optional T
#     = len (길이)

## 연산자
.     체이닝/파이프
@     맵: .@field, .@fn, .@(_.expr)
?     필터: .?field, .?(cond)
/     리듀스: ./+, ./*, ./min, ./max
? :   삼항 조건: cond ? then : else

## 람다
_           현재 요소 (암묵적)
_.field     현재 요소의 필드
(_>0)       조건식
(_*2)       변환식

## 바인딩
let x=val : expr
let x=v1, y=v2 : expr

## 범위
2..n        range(2, n)
0..#a       range(0, len(a))

## 빌트인
#, first, last, flip, set, flatten, all, any, zip
""")

    print()
    print("=" * 80)
    print("예제 비교")
    print("=" * 80)

    examples_to_show = [
        "add_numbers", "filter_map", "sum_list", "fibonacci",
        "quick_sort", "double_all", "count_positive", "average"
    ]

    for name in examples_to_show:
        if name not in EXAMPLES:
            continue
        data = EXAMPLES[name]
        py_t = count_tokens(data["python"])
        v6_t = count_tokens(data["v6"])
        sav = (1 - v6_t / py_t) * 100

        print(f"\n### {name} ({data['desc']})")
        print(f"\nPython ({py_t} tokens):")
        print(data["python"])
        print(f"\nv6 ({v6_t} tokens, {sav:+.0f}%):")
        print(data["v6"])
        print("-" * 40)


if __name__ == "__main__":
    run_benchmark()
