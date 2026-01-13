# Vais 예제

Vais의 기능을 보여주는 다양한 실용 예제 모음입니다.

## 목차

- [기본 예제](#기본-예제)
- [재귀](#재귀)
- [컬렉션 연산](#컬렉션-연산)
- [데이터 처리](#데이터-처리)
- [알고리즘](#알고리즘)
- [실용 응용](#실용-응용)
- [예제 실행하기](#예제-실행하기)

---

## 기본 예제

### Hello World

```vais
print("Hello, World!")
```

### 변수와 표현식

```vais
// 변수
name = "Vais"
version = 1.0
is_ready = true

// 산술 연산
x = 10
y = 3
print("덧셈:", x + y)      // 13
print("뺄셈:", x - y)      // 7
print("곱셈:", x * y)      // 30
print("나눗셈:", x / y)    // 3
print("나머지:", x % y)    // 1

// 문자열 연결
greeting = "안녕, " ++ name ++ "!"
print(greeting)           // "안녕, Vais!"
```

### 함수

```vais
// 간단한 함수
add(a, b) = a + b

// 여러 표현식이 있는 함수
greet(name) = "안녕, " ++ name ++ "!"

// 고차 함수(Higher-order Function)
apply_twice(f, x) = f(f(x))

double(x) = x * 2
print(apply_twice(double, 5))   // 20
```

---

## 재귀

### 팩토리얼(Factorial)

```vais
// 자기 재귀($) 사용
factorial(n) = n < 2 ? 1 : n * $(n - 1)

print(factorial(5))   // 120
print(factorial(10))  // 3628800

// 1-10의 팩토리얼 계산
[1..11].@(factorial(_))
// [1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800]
```

### 피보나치(Fibonacci)

```vais
// 재귀적 피보나치
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)

// 처음 15개의 피보나치 수
[0..15].@(fib(_))
// [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377]

// 처음 10개의 피보나치 수의 합
[0..10].@(fib(_))./+(0, _ + _)   // 88
```

### 최대공약수(GCD)

```vais
gcd(a, b) = b == 0 ? a : $(b, a % b)

print(gcd(48, 18))    // 6
print(gcd(100, 35))   // 5
```

### 거듭제곱 함수

```vais
// 재귀적 거듭제곱
power(base, exp) = exp == 0 ? 1 : base * $(base, exp - 1)

print(power(2, 10))   // 1024
print(power(3, 4))    // 81
```

---

## 컬렉션 연산

### Map 예제

```vais
numbers = [1, 2, 3, 4, 5]

// 각 요소를 2배
doubled = numbers.@(_ * 2)
print(doubled)   // [2, 4, 6, 8, 10]

// 각 요소의 제곱
squared = numbers.@(_ * _)
print(squared)   // [1, 4, 9, 16, 25]

// 문자열로 변환
strings = numbers.@(str(_))
print(strings)   // ["1", "2", "3", "4", "5"]

// 객체에서 필드 추출
users = [{name: "Alice", age: 30}, {name: "Bob", age: 25}]
names = users.@(_.name)
print(names)     // ["Alice", "Bob"]
```

### Filter 예제

```vais
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// 짝수 유지
evens = numbers.?(_ % 2 == 0)
print(evens)     // [2, 4, 6, 8, 10]

// 홀수 유지
odds = numbers.?(_ % 2 != 0)
print(odds)      // [1, 3, 5, 7, 9]

// 5보다 큰 수 유지
large = numbers.?(_ > 5)
print(large)     // [6, 7, 8, 9, 10]

// 객체 필터링
users = [{name: "Alice", age: 30}, {name: "Bob", age: 17}]
adults = users.?(_.age >= 18)
print(adults)    // [{name: "Alice", age: 30}]
```

### Reduce 예제

```vais
numbers = [1, 2, 3, 4, 5]

// 합계
sum = numbers./+(0, _ + _)
print(sum)       // 15

// 곱
product = numbers./*(1, _ * _)
print(product)   // 120

// 최대값 찾기
max_val = numbers./(numbers[0], _ > _ ? _1 : _2)
print(max_val)   // 5

// 최소값 찾기
min_val = numbers./(numbers[0], _ < _ ? _1 : _2)
print(min_val)   // 1

// 요소 개수 세기
count = numbers./(0, _1 + 1)
print(count)     // 5

// 문자열 결합
words = ["안녕", "세상", "Vais"]
joined = words./("", _1 ++ " " ++ _2)
print(trim(joined))   // "안녕 세상 Vais"
```

### 연산 체이닝

```vais
// 파이프라인: 짝수 필터 -> 제곱 -> 합계
result = [1..11]
    .?(_ % 2 == 0)      // [2, 4, 6, 8, 10]
    .@(_ * _)           // [4, 16, 36, 64, 100]
    ./+(0, _ + _)       // 220

print(result)

// 성인 사용자의 이름을 대문자로
users = [
    {name: "alice", age: 30},
    {name: "bob", age: 17},
    {name: "charlie", age: 25}
]

adult_names = users
    .?(_.age >= 18)
    .@(_.name)
    .@(upper(_))

print(adult_names)   // ["ALICE", "CHARLIE"]
```

---

## 데이터 처리

### 통계

```vais
data = [23, 45, 67, 12, 89, 34, 56, 78, 90, 11]

// 합계
total = data./+(0, _ + _)
print("합계:", total)         // 505

// 평균
avg = total / len(data)
print("평균:", avg)           // 50.5

// 최소값과 최대값
min_val = data./(data[0], _ < _ ? _1 : _2)
max_val = data./(data[0], _ > _ ? _1 : _2)
print("최소값:", min_val)     // 11
print("최대값:", max_val)     // 90

// 평균 초과 개수
above_avg = data.?(_ > avg)
print("평균 초과:", len(above_avg))   // 5
```

### 문자열 처리

```vais
text = "  Hello, World! Welcome to Vais.  "

// 공백 제거
trimmed = trim(text)
print(trimmed)

// 단어로 분할
words = split(trimmed, " ")
print(words)

// 단어 개수
word_count = len(words.?(_ != ""))
print("단어 개수:", word_count)

// 대문자로 변환
upper_text = upper(text)
print(upper_text)

// 대체
replaced = replace(text, "Vais", "Rust")
print(replaced)
```

### JSON 처리

```vais
// JSON 파싱
json_str = '{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}'
data = json_parse(json_str)

// 사용자 추출
users = json_get(data, "users")
print(users)

// 모든 이름 가져오기
names = users.@(_.name)
print("이름:", names)

// 필터링 및 변환
adults = users
    .?(_.age >= 18)
    .@({name: upper(_.name), adult: true})

print(json_stringify_pretty(adults))
```

---

## 알고리즘

### 소수(Prime Numbers)

```vais
// 소수 확인
is_prime(n) = n < 2 ? false :
    n == 2 ? true :
    n % 2 == 0 ? false :
    check_divisors(n, 3)

check_divisors(n, i) =
    i * i > n ? true :
    n % i == 0 ? false :
    $(n, i + 2)

// n까지의 모든 소수 찾기
primes_up_to(n) = [2..n+1].?(is_prime(_))

print(primes_up_to(50))
// [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
```

### 퀵정렬(Quicksort)

```vais
quicksort(arr) =
    len(arr) <= 1 ? arr :
    let pivot = arr[0] in
    let less = tail(arr).?(_ < pivot) in
    let greater = tail(arr).?(_ >= pivot) in
    $(less) ++ [pivot] ++ $(greater)

unsorted = [64, 34, 25, 12, 22, 11, 90]
sorted = quicksort(unsorted)
print(sorted)   // [11, 12, 22, 25, 34, 64, 90]
```

### 이진 탐색(Binary Search)

```vais
binary_search(arr, target) = search(arr, target, 0, len(arr) - 1)

search(arr, target, low, high) =
    low > high ? -1 :
    let mid = (low + high) / 2 in
    arr[mid] == target ? mid :
    arr[mid] < target ? $(arr, target, mid + 1, high) :
    $(arr, target, low, mid - 1)

sorted = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19]
print(binary_search(sorted, 7))    // 3
print(binary_search(sorted, 6))    // -1
```

### 병합 정렬(Merge Sort)

```vais
merge_sort(arr) =
    len(arr) <= 1 ? arr :
    let mid = len(arr) / 2 in
    let left = $(take(arr, mid)) in
    let right = $(drop(arr, mid)) in
    merge(left, right)

merge(left, right) =
    len(left) == 0 ? right :
    len(right) == 0 ? left :
    head(left) <= head(right) ?
        [head(left)] ++ $(tail(left), right) :
        [head(right)] ++ $(left, tail(right))

arr = [38, 27, 43, 3, 9, 82, 10]
print(merge_sort(arr))   // [3, 9, 10, 27, 38, 43, 82]
```

---

## 실용 응용

### 온도 변환기

```vais
celsius_to_fahrenheit(c) = c * 9 / 5 + 32
fahrenheit_to_celsius(f) = (f - 32) * 5 / 9

// 온도 변환
temps_c = [0, 10, 20, 30, 100]
temps_f = temps_c.@(celsius_to_fahrenheit(_))
print("섭씨:", temps_c)
print("화씨:", temps_f)
```

### 성적 계산기

```vais
grade(score) =
    score >= 90 ? "A" :
    score >= 80 ? "B" :
    score >= 70 ? "C" :
    score >= 60 ? "D" : "F"

scores = [95, 82, 76, 65, 58, 91, 73]
grades = scores.@(grade(_))
print(zip(scores, grades))

// 각 성적 개수
count_grade(grades, g) = len(grades.?(_ == g))
print("A:", count_grade(grades, "A"))
print("B:", count_grade(grades, "B"))
print("C:", count_grade(grades, "C"))
```

### 장바구니

```vais
cart = [
    {name: "사과", price: 1500, qty: 4},
    {name: "빵", price: 2500, qty: 2},
    {name: "우유", price: 3000, qty: 1}
]

// 항목별 금액 계산
line_totals = cart.@(_.price * _.qty)
print("항목별 금액:", line_totals)

// 총액
total = line_totals./+(0, _ + _)
print("총액:", total)

// 10% 할인 적용
discount = total * 0.10
final = total - discount
print("할인액:", discount)
print("최종 금액:", final)
```

### 단어 빈도 카운터

```vais
text = "the quick brown fox jumps over the lazy dog the fox"

// 단어로 분할
words = split(lower(text), " ")

// 고유 단어 가져오기
unique_words = unique(words)

// 각 단어 개수 세기
count_word(word) = len(words.?(_ == word))

// 빈도 맵 생성
frequencies = unique_words.@({word: _, count: count_word(_)})

// 개수별 정렬 (내림차순)
sorted_freq = sort(frequencies.@(_.count))
print(reverse(sorted_freq))
```

### FizzBuzz

```vais
fizzbuzz(n) =
    n % 15 == 0 ? "FizzBuzz" :
    n % 3 == 0 ? "Fizz" :
    n % 5 == 0 ? "Buzz" :
    str(n)

// FizzBuzz 1-20
result = [1..21].@(fizzbuzz(_))
print(result)
```

### 회문 검사기(Palindrome Checker)

```vais
is_palindrome(s) =
    let cleaned = lower(replace(s, " ", "")) in
    cleaned == reverse(cleaned)

print(is_palindrome("racecar"))        // true
print(is_palindrome("A man a plan"))   // false (공백이 중요)
print(is_palindrome("hello"))          // false
```

---

## 예제 실행하기

예제를 `.vais` 파일로 저장하고 실행하세요:

```bash
# 인터프리터로 실행
vais run example.vais

# JIT로 실행 (더 빠름)
vais run example.vais --jit

# 대화형 REPL
vais repl
```

---

## 관련 문서

- [시작 가이드](getting-started.md)
- [문법 가이드](syntax.md)
- [API 레퍼런스](api.md)
