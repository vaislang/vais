#!/usr/bin/env python3
"""
Vais vs Python: 토큰 수 및 코드 길이 비교

동일한 로직을 Vais과 Python으로 구현했을 때의 차이를 비교합니다.
"""

import tokenize
import io

# 비교할 코드 예제들
examples = [
    {
        "name": "팩토리얼 (Factorial)",
        "vais": 'fact(n) = n < 2 ? 1 : n * $(n - 1)',
        "python": '''def fact(n):
    if n < 2:
        return 1
    else:
        return n * fact(n - 1)'''
    },
    {
        "name": "피보나치 (Fibonacci)",
        "vais": 'fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)',
        "python": '''def fib(n):
    if n < 2:
        return n
    else:
        return fib(n - 1) + fib(n - 2)'''
    },
    {
        "name": "배열 맵 (Array Map - Double)",
        "vais": 'double(arr) = arr.@(_ * 2)',
        "python": '''def double(arr):
    return [x * 2 for x in arr]'''
    },
    {
        "name": "배열 필터 (Array Filter - Evens)",
        "vais": 'evens(arr) = arr.?(_ % 2 == 0)',
        "python": '''def evens(arr):
    return [x for x in arr if x % 2 == 0]'''
    },
    {
        "name": "배열 합계 (Array Sum)",
        "vais": 'sum(arr) = arr./+',
        "python": '''def sum_arr(arr):
    return sum(arr)'''
    },
    {
        "name": "배열 곱 (Array Product)",
        "vais": 'product(arr) = arr./*',
        "python": '''from functools import reduce
def product(arr):
    return reduce(lambda x, y: x * y, arr, 1)'''
    },
    {
        "name": "체이닝 (Map + Filter + Reduce)",
        "vais": 'process(arr) = arr.@(_ * 2).?(_ > 5)./+',
        "python": '''def process(arr):
    doubled = [x * 2 for x in arr]
    filtered = [x for x in doubled if x > 5]
    return sum(filtered)'''
    },
    {
        "name": "최대값 (Max of Two)",
        "vais": 'max(a, b) = a > b ? a : b',
        "python": '''def max_of_two(a, b):
    if a > b:
        return a
    else:
        return b'''
    },
    {
        "name": "절대값 (Absolute)",
        "vais": 'abs_fn(n) = n < 0 ? -n : n',
        "python": '''def abs_fn(n):
    if n < 0:
        return -n
    else:
        return n'''
    },
    {
        "name": "Let 바인딩 (Let Binding)",
        "vais": 'calc(x) = let a = x + 1, b = x * 2 : a + b',
        "python": '''def calc(x):
    a = x + 1
    b = x * 2
    return a + b'''
    },
]

def count_python_tokens(code):
    """Python 코드의 토큰 수를 계산"""
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(code).readline))
        # ENCODING, NEWLINE, NL, ENDMARKER 제외
        meaningful = [t for t in tokens if t.type not in (
            tokenize.ENCODING, tokenize.NEWLINE, tokenize.NL,
            tokenize.ENDMARKER, tokenize.INDENT, tokenize.DEDENT,
            tokenize.COMMENT
        )]
        return len(meaningful)
    except:
        return -1

def count_vais_tokens(code):
    """Vais 코드의 토큰 수를 대략적으로 계산"""
    # 간단한 토큰화: 공백으로 분리 후 특수문자 분리
    import re
    # 연산자와 구분자를 개별 토큰으로
    tokens = re.findall(r'[a-zA-Z_][a-zA-Z0-9_]*|[0-9]+|\.\@|\.\?|\./\+|\./\*|\$|[+\-*/%<>=!&|?:,\(\)\[\]#]', code)
    return len(tokens)

def main():
    print("=" * 80)
    print("Vais vs Python: 토큰 수 및 코드 길이 비교")
    print("=" * 80)
    print()

    total_vais_chars = 0
    total_python_chars = 0
    total_vais_tokens = 0
    total_python_tokens = 0
    total_vais_lines = 0
    total_python_lines = 0

    for ex in examples:
        vais_code = ex["vais"]
        python_code = ex["python"]

        vais_chars = len(vais_code)
        python_chars = len(python_code)

        vais_lines = len(vais_code.strip().split('\n'))
        python_lines = len(python_code.strip().split('\n'))

        vais_tokens = count_vais_tokens(vais_code)
        python_tokens = count_python_tokens(python_code)

        total_vais_chars += vais_chars
        total_python_chars += python_chars
        total_vais_tokens += vais_tokens
        total_python_tokens += python_tokens
        total_vais_lines += vais_lines
        total_python_lines += python_lines

        reduction_chars = ((python_chars - vais_chars) / python_chars) * 100
        reduction_tokens = ((python_tokens - vais_tokens) / python_tokens) * 100

        print(f"📌 {ex['name']}")
        print("-" * 60)
        print(f"  Vais:   {vais_code}")
        print(f"  Python:")
        for line in python_code.split('\n'):
            print(f"          {line}")
        print()
        print(f"  {'항목':<12} {'Vais':>10} {'Python':>10} {'절감률':>10}")
        print(f"  {'-'*12} {'-'*10} {'-'*10} {'-'*10}")
        print(f"  {'문자 수':<12} {vais_chars:>10} {python_chars:>10} {reduction_chars:>9.1f}%")
        print(f"  {'토큰 수':<12} {vais_tokens:>10} {python_tokens:>10} {reduction_tokens:>9.1f}%")
        print(f"  {'라인 수':<12} {vais_lines:>10} {python_lines:>10}")
        print()

    print("=" * 80)
    print("📊 전체 통계 (Total Statistics)")
    print("=" * 80)

    total_reduction_chars = ((total_python_chars - total_vais_chars) / total_python_chars) * 100
    total_reduction_tokens = ((total_python_tokens - total_vais_tokens) / total_python_tokens) * 100

    print(f"  {'항목':<16} {'Vais':>10} {'Python':>10} {'절감률':>10}")
    print(f"  {'-'*16} {'-'*10} {'-'*10} {'-'*10}")
    print(f"  {'총 문자 수':<16} {total_vais_chars:>10} {total_python_chars:>10} {total_reduction_chars:>9.1f}%")
    print(f"  {'총 토큰 수':<16} {total_vais_tokens:>10} {total_python_tokens:>10} {total_reduction_tokens:>9.1f}%")
    print(f"  {'총 라인 수':<16} {total_vais_lines:>10} {total_python_lines:>10}")
    print()
    print(f"  💡 Vais은 Python 대비 평균 {total_reduction_chars:.1f}% 적은 문자로 동일 로직 구현")
    print(f"  💡 Vais은 Python 대비 평균 {total_reduction_tokens:.1f}% 적은 토큰으로 동일 로직 구현")
    print()

if __name__ == "__main__":
    main()
