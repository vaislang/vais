#!/usr/bin/env python3
"""
AOEL v6b Token Benchmark - Maximum Optimization

v6에서 추가 최적화:
1. 타입 선언 생략 (추론 가능하면)
2. 공백 제거 (a+b, 콤마 뒤 공백 없음)
3. 괄호 최소화
4. 단축 문법 추가

문법:
  name(params) = body     타입 생략 가능
  .@ .? ./                체이닝 연산
  ?:                      삼항 (공백 없이)
  _                       람다 인자
  #                       len
  $                       self 재귀 호출
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
        "v6b": 'hello()="Hello, World!"',
    },

    "add_numbers": {
        "desc": "두 정수 더하기",
        "python": '''def add(a: int, b: int) -> int:
    return a + b''',
        "v6b": 'add(a,b)=a+b',
    },

    "fibonacci": {
        "desc": "피보나치",
        "python": '''def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)''',
        # $는 자기 자신 재귀 호출
        "v6b": 'fib(n)=n<2?n:$(n-1)+$(n-2)',
    },

    "factorial": {
        "desc": "팩토리얼",
        "python": '''def factorial(n: int) -> int:
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result''',
        "v6b": 'fact(n)=n<2?1:n*$(n-1)',
    },

    "filter_map": {
        "desc": "활성 사용자 이메일",
        "python": '''def get_active_emails(users: list[User]) -> list[str]:
    return [u.email.upper() for u in users if u.is_active]''',
        "v6b": 'emails(us)=us.?active.@email.@up',
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
        "v6b": 'cat(age)=age<0?err:age>=18?"adult":age>=13?"teen":"child"',
    },

    "max_number": {
        "desc": "최대값",
        "python": '''def max_num(a: int, b: int) -> int:
    if a > b:
        return a
    return b''',
        "v6b": 'max(a,b)=a>b?a:b',
    },

    "sum_list": {
        "desc": "리스트 합계",
        "python": '''def sum_list(numbers: list[int]) -> int:
    total = 0
    for n in numbers:
        total += n
    return total''',
        "v6b": 'sum(ns)=ns./+',
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
        # 기본값 인자 + $ 재귀
        "v6b": 'bs(a,t,lo=0,hi=#a-1)=lo>hi?nil:let m=(lo+hi)/2:a[m]==t?m:a[m]<t?$(a,t,m+1,hi):$(a,t,lo,m-1)',
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
        "v6b": 'qs(a)=#a<2?a:let p=a[0],r=a[1:]:$(r.?(_<p))+[p]+$(r.?(_>=p))',
    },

    "map_reduce": {
        "desc": "총 매출",
        "python": '''def total_revenue(orders: list[Order]) -> float:
    return sum(o.quantity * o.price for o in orders)''',
        "v6b": 'rev(os)=os.@(_.qty*_.price)./+',
    },

    "is_even": {
        "desc": "짝수 확인",
        "python": '''def is_even(n: int) -> bool:
    return n % 2 == 0''',
        "v6b": 'even(n)=n%2==0',
    },

    "string_length": {
        "desc": "문자열 길이 확인",
        "python": '''def is_long(s: str) -> bool:
    return len(s) >= 5''',
        "v6b": 'long(s)=#s>=5',
    },

    "absolute": {
        "desc": "절대값",
        "python": '''def abs_val(n: int) -> int:
    if n < 0:
        return -n
    return n''',
        "v6b": 'abs(n)=n<0?-n:n',
    },

    "clamp": {
        "desc": "범위 제한",
        "python": '''def clamp(val: int, min_val: int, max_val: int) -> int:
    if val < min_val:
        return min_val
    if val > max_val:
        return max_val
    return val''',
        "v6b": 'clamp(v,lo,hi)=v<lo?lo:v>hi?hi:v',
    },

    "contains": {
        "desc": "포함 여부",
        "python": '''def contains(arr: list[int], target: int) -> bool:
    for x in arr:
        if x == target:
            return True
    return False''',
        "v6b": 'has(a,t)=t@a',
    },

    "reverse_string": {
        "desc": "문자열 뒤집기",
        "python": '''def reverse(s: str) -> str:
    return s[::-1]''',
        "v6b": 'rev(s)=s.flip',
    },

    "count_positive": {
        "desc": "양수 개수",
        "python": '''def count_pos(nums: list[int]) -> int:
    return len([x for x in nums if x > 0])''',
        "v6b": 'cpos(ns)=ns.?(_>0).#',
    },

    "double_all": {
        "desc": "모두 2배",
        "python": '''def double_all(nums: list[int]) -> list[int]:
    return [x * 2 for x in nums]''',
        "v6b": 'dbl(ns)=ns.@(_*2)',
    },

    "first_positive": {
        "desc": "첫 양수",
        "python": '''def first_pos(nums: list[int]) -> int | None:
    for x in nums:
        if x > 0:
            return x
    return None''',
        "v6b": 'fpos(ns)=ns.?(_>0).first',
    },

    "average": {
        "desc": "평균값",
        "python": '''def average(nums: list[int]) -> float:
    return sum(nums) / len(nums)''',
        "v6b": 'avg(ns)=ns./+/#ns',
    },

    "gcd": {
        "desc": "최대공약수",
        "python": '''def gcd(a: int, b: int) -> int:
    if b == 0:
        return a
    return gcd(b, a % b)''',
        "v6b": 'gcd(a,b)=b==0?a:$(b,a%b)',
    },

    "palindrome": {
        "desc": "팰린드롬",
        "python": '''def is_palindrome(s: str) -> bool:
    return s == s[::-1]''',
        "v6b": 'palin(s)=s==s.flip',
    },

    "unique": {
        "desc": "중복 제거",
        "python": '''def unique(nums: list[int]) -> list[int]:
    return list(set(nums))''',
        "v6b": 'uniq(ns)=ns.set',
    },

    "zip_sum": {
        "desc": "요소별 합",
        "python": '''def zip_sum(a: list[int], b: list[int]) -> list[int]:
    return [x + y for x, y in zip(a, b)]''',
        "v6b": 'zsum(a,b)=zip(a,b).@(_.0+_.1)',
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
        "v6b": 'prime(n)=n<2?0:(2..n).all(n%_!=0)',
    },

    "flatten": {
        "desc": "평탄화",
        "python": '''def flatten(lists: list[list[int]]) -> list[int]:
    result = []
    for lst in lists:
        result.extend(lst)
    return result''',
        "v6b": 'flat(ls)=ls.flatten',
    },

    "all_positive": {
        "desc": "모두 양수",
        "python": '''def all_positive(nums: list[int]) -> bool:
    for n in nums:
        if n <= 0:
            return False
    return True''',
        "v6b": 'allpos(ns)=ns.all(_>0)',
    },

    "any_negative": {
        "desc": "음수 존재",
        "python": '''def any_negative(nums: list[int]) -> bool:
    for n in nums:
        if n < 0:
            return True
    return False''',
        "v6b": 'anyneg(ns)=ns.any(_<0)',
    },

    "min_value": {
        "desc": "최소값",
        "python": '''def min_val(nums: list[int]) -> int:
    return min(nums)''',
        "v6b": 'minv(ns)=ns./min',
    },

    "product": {
        "desc": "모든 곱",
        "python": '''def product(nums: list[int]) -> int:
    result = 1
    for n in nums:
        result *= n
    return result''',
        "v6b": 'prod(ns)=ns./*',
    },

    "square_all": {
        "desc": "모두 제곱",
        "python": '''def square_all(nums: list[int]) -> list[int]:
    return [x * x for x in nums]''',
        "v6b": 'sq(ns)=ns.@(_*_)',
    },

    "concat_strings": {
        "desc": "문자열 합치기",
        "python": '''def concat(a: str, b: str) -> str:
    return a + b''',
        "v6b": 'concat(a,b)=a+b',
    },

    "last_element": {
        "desc": "마지막 요소",
        "python": '''def last(arr: list[int]) -> int:
    return arr[-1]''',
        "v6b": 'last(a)=a[-1]',
    },

    "count_items": {
        "desc": "요소 개수",
        "python": '''def count(arr: list[int]) -> int:
    return len(arr)''',
        "v6b": 'count(a)=#a',
    },

    "negate": {
        "desc": "부호 반전",
        "python": '''def negate(n: int) -> int:
    return -n''',
        "v6b": 'neg(n)=-n',
    },
}


def run_benchmark():
    print("=" * 80)
    print("AOEL v6b Token Benchmark - Maximum Optimization")
    print("=" * 80)
    print()
    print("최적화 적용:")
    print("  - 타입 선언 생략")
    print("  - 공백 제거 (a+b, 콤마 뒤)")
    print("  - $ = 자기 자신 재귀 호출")
    print("  - @ = in 연산자 (t@a = t in a)")
    print()
    print("-" * 80)

    total_py = 0
    total_v6b = 0
    results = []

    for name, data in EXAMPLES.items():
        py_code = data["python"]
        v6b_code = data["v6b"]

        py_tokens = count_tokens(py_code)
        v6b_tokens = count_tokens(v6b_code)

        total_py += py_tokens
        total_v6b += v6b_tokens

        savings = (1 - v6b_tokens / py_tokens) * 100
        results.append((name, data["desc"], py_tokens, v6b_tokens, savings, v6b_code))

    # Sort by savings
    results_sorted = sorted(results, key=lambda x: -x[4])

    # 결과 출력
    print(f"{'Name':<18} {'Desc':<12} {'Python':>8} {'v6b':>8} {'Savings':>10}")
    print("-" * 62)

    for name, desc, py_t, v6b_t, sav, _ in results_sorted:
        marker = "✓" if sav >= 40 else "○" if sav >= 20 else "✗"
        desc_short = desc[:11]
        print(f"{name:<18} {desc_short:<12} {py_t:>8} {v6b_t:>8} {sav:>+9.1f}% {marker}")

    print("-" * 62)
    total_savings = (1 - total_v6b / total_py) * 100
    print(f"{'TOTAL':<18} {'':<12} {total_py:>8} {total_v6b:>8} {total_savings:>+9.1f}%")
    print()

    # 통계
    over_40 = sum(1 for r in results if r[4] >= 40)
    over_20 = sum(1 for r in results if 20 <= r[4] < 40)
    under_20 = sum(1 for r in results if r[4] < 20)

    print(f"40% 이상: {over_40}개")
    print(f"20-40%: {over_20}개")
    print(f"20% 미만: {under_20}개")
    print()

    if total_savings >= 40:
        print("✅ SUCCESS: 40% 이상 절감 달성!")
    elif total_savings >= 35:
        print("⚠️  VERY CLOSE: 35-40% 절감. 거의 도달!")
    elif total_savings >= 30:
        print("⚠️  CLOSE: 30-35% 절감.")
    else:
        print("❌ FAIL: 30% 미만.")

    print()
    print("=" * 80)
    print("v6b 문법 명세 (최종)")
    print("=" * 80)
    print("""
## 함수 정의
name(params)=body           타입 생략, 공백 없음

## 연산자
.@expr      맵: ns.@(_*2), us.@email
.?expr      필터: ns.?(_>0), us.?active
./op        리듀스: ns./+, ns./*, ns./min
a?b:c       삼항 조건 (공백 없음)
#           길이: #arr, #str
$           자기 자신 재귀: $(n-1), $(a,b)
@           in 연산자: t@a (t in a)
..          범위: 2..n

## 람다
_           현재 요소
_.field     필드 접근
_*2         변환
_>0         조건

## 바인딩
let x=v:expr
let x=v1,y=v2:expr

## 빌트인
#, first, last, flip, set, flatten, all, any, zip, err
""")

    print()
    print("=" * 80)
    print("Best/Worst 예제")
    print("=" * 80)

    print("\n### BEST (40% 이상)")
    for name, desc, py_t, v6b_t, sav, code in results_sorted[:8]:
        print(f"\n{name} ({sav:+.0f}%): {code}")

    print("\n### WORST (20% 미만)")
    for name, desc, py_t, v6b_t, sav, code in results_sorted[-5:]:
        print(f"\n{name} ({sav:+.0f}%): {code}")


if __name__ == "__main__":
    run_benchmark()
