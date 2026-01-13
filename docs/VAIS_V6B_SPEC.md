# Vais v6b Language Specification

**Version:** 1.0.0
**Date:** 2026-01-12
**Status:** Phase 1 - Implementation Ready

---

## 1. Overview

Vais v6b는 AI가 효율적으로 생성하고 직접 실행할 수 있는 프로그래밍 언어입니다.

### 설계 원칙

1. **토큰 효율성**: Python 대비 40%+ 토큰 절감
2. **명확성**: 각 기호는 하나의 의미만 가짐
3. **생성 정확도**: AI가 100% 정확하게 생성 가능
4. **실행 가능**: 자체 VM 또는 네이티브 컴파일

### 검증 결과 (Phase 0)

| 항목 | 결과 |
|------|------|
| 토큰 절감률 | 43.9% |
| 생성 정확도 | 93% → 100% (빌트인 추가 후) |

---

## 2. Lexical Structure

### 2.1 Character Set

- UTF-8 인코딩
- 식별자: ASCII 알파벳, 숫자, 언더스코어
- 문자열: 유니코드 허용

### 2.2 Whitespace

- 공백, 탭, 개행은 토큰 구분자로만 사용
- 연산자 주변 공백은 선택적

### 2.3 Comments

```
// 한 줄 주석
/* 여러 줄 주석 */
```

### 2.4 Identifiers

```ebnf
identifier = letter (letter | digit | "_")*
letter     = "a".."z" | "A".."Z"
digit      = "0".."9"
```

예시: `add`, `myFunc`, `get_user`, `x1`

### 2.5 Keywords

```
let, if, else, nil, true, false, err
```

**예약어 최소화**: 대부분의 제어는 연산자로 처리

### 2.6 Literals

#### Numbers
```
42          // 정수
3.14        // 실수
-17         // 음수
1_000_000   // 구분자 허용
0xFF        // 16진수
0b1010      // 2진수
```

#### Strings
```
"hello"           // 기본 문자열
"line1\nline2"    // 이스케이프
"say \"hi\""      // 따옴표 이스케이프
```

#### Boolean
```
true
false
```

#### Nil
```
nil
```

#### Arrays
```
[1, 2, 3]
["a", "b"]
[]              // 빈 배열
```

#### Objects (선택적)
```
{name: "John", age: 30}
```

---

## 3. Type System

### 3.1 Primitive Types

| 타입 | 축약 | 설명 |
|------|------|------|
| int | i | 정수 (64-bit) |
| float | f | 실수 (64-bit) |
| string | s | 문자열 |
| bool | b | 불리언 |

### 3.2 Composite Types

| 타입 | 문법 | 예시 |
|------|------|------|
| array | [T] | [i], [s], [[i]] |
| optional | ?T | ?i, ?s |
| tuple | (T1,T2) | (i,s) |
| object | {field:T} | {name:s, age:i} |

### 3.3 Type Inference

타입 선언은 **선택적**. 추론 가능하면 생략.

```
// 타입 명시
add(a:i, b:i)=a+b

// 타입 생략 (추론)
add(a,b)=a+b
```

---

## 4. Operators

### 4.1 Arithmetic

| 연산자 | 의미 | 예시 |
|--------|------|------|
| + | 덧셈/연결 | a+b, "a"+"b" |
| - | 뺄셈 | a-b |
| * | 곱셈 | a*b |
| / | 나눗셈 | a/b |
| % | 나머지 | a%b |
| ** | 거듭제곱 | a**2 |

### 4.2 Comparison

| 연산자 | 의미 |
|--------|------|
| == | 같음 |
| != | 다름 |
| < | 작음 |
| > | 큼 |
| <= | 작거나 같음 |
| >= | 크거나 같음 |

### 4.3 Logical

| 연산자 | 의미 |
|--------|------|
| & | AND |
| \| | OR |
| ! | NOT |

### 4.4 Special Operators

| 연산자 | 의미 | 예시 |
|--------|------|------|
| # | 길이 | #arr, #str |
| $ | 자기 재귀 | $(n-1) |
| @ | in 연산자 | x@arr |
| .. | 범위 | 1..10 |
| ? : | 삼항 조건 | a?b:c |

### 4.5 Chaining Operators

| 연산자 | 의미 | 예시 |
|--------|------|------|
| .@ | 맵 | ns.@(_*2) |
| .? | 필터 | ns.?(_>0) |
| ./ | 리듀스 | ns./+ |

---

## 5. Expressions

### 5.1 Function Definition

```ebnf
function = identifier "(" [params] ")" "=" expression
params   = param ("," param)*
param    = identifier [":" type]
```

예시:
```
add(a,b)=a+b
max(a,b)=a>b?a:b
fib(n)=n<2?n:$(n-1)+$(n-2)
```

### 5.2 Ternary Conditional

```ebnf
ternary = expression "?" expression ":" expression
```

예시:
```
a>b?a:b
n<0?err:n==0?1:n*$(n-1)
```

**중첩 조건**:
```
age>=18?"adult":age>=13?"teen":"child"
```

### 5.3 Let Binding

```ebnf
let_expr = "let" binding ("," binding)* ":" expression
binding  = identifier "=" expression
```

예시:
```
let x=10:x*2
let a=1,b=2:a+b
let p=arr[0],r=arr[1:]:$(r)+p
```

### 5.4 Lambda (Implicit Parameter)

언더스코어 `_`는 암묵적 람다 파라미터:

```
_           // 현재 요소
_.field     // 필드 접근
_*2         // 변환
_>0         // 조건
_.0, _.1    // 튜플 접근
```

예시:
```
ns.@(_*2)           // [1,2,3] -> [2,4,6]
ns.?(_>0)           // 양수만 필터
us.@(_.name)        // 이름 추출
pairs.@(_.0+_.1)    // 튜플 합
```

### 5.5 Map (.@)

```ebnf
map_expr = expression ".@" (field | "(" lambda ")")
```

예시:
```
ns.@(_*2)           // 각 요소 2배
us.@name            // 필드 추출 (축약)
us.@(_.email)       // 필드 추출 (명시)
ns.@str             // 함수 적용
```

### 5.6 Filter (.?)

```ebnf
filter_expr = expression ".?" (field | "(" lambda ")")
```

예시:
```
ns.?(_>0)           // 양수만
us.?active          // active가 true인 것
us.?(_.age>=18)     // 성인만
```

### 5.7 Reduce (./)

```ebnf
reduce_expr = expression "./" operator
operator    = "+" | "*" | "min" | "max" | "and" | "or"
```

예시:
```
ns./+               // 합계
ns./*               // 곱
ns./min             // 최소값
ns./max             // 최대값
bs./and             // 모두 true
bs./or              // 하나라도 true
```

### 5.8 Self Recursion ($)

`$`는 현재 함수 자신을 참조:

```
fib(n)=n<2?n:$(n-1)+$(n-2)
fact(n)=n<2?1:n*$(n-1)
gcd(a,b)=b==0?a:$(b,a%b)
```

### 5.9 Range (..)

```ebnf
range = expression ".." expression
```

예시:
```
1..10               // [1,2,3,4,5,6,7,8,9]
0..#arr             // 인덱스 범위
2..n                // 2부터 n-1까지
```

### 5.10 In Operator (@)

```ebnf
in_expr = expression "@" expression
```

예시:
```
x@arr               // x가 arr에 포함?
"a"@str             // 문자열 포함?
```

### 5.11 Index Access

```ebnf
index = expression "[" expression "]"
slice = expression "[" [start] ":" [end] "]"
```

예시:
```
arr[0]              // 첫 번째
arr[-1]             // 마지막
arr[1:3]            // 슬라이스
arr[1:]             // 첫 번째 이후
arr[:-1]            // 마지막 제외
```

### 5.12 Field Access

```ebnf
field_access = expression "." identifier
```

예시:
```
user.name
order.items.#
point.x
```

---

## 6. Built-in Functions

### 6.1 Collection

| 함수 | 설명 | 예시 |
|------|------|------|
| # | 길이 | #arr, #str |
| first | 첫 요소 | arr.first |
| last | 마지막 요소 | arr.last |
| flip | 뒤집기 | arr.flip, str.flip |
| set | 중복 제거 | arr.set |
| flatten | 평탄화 | [[1],[2]].flatten |
| sort | 정렬 | arr.sort |
| sort(f) | 필드로 정렬 | us.sort(_.age) |

### 6.2 Aggregation

| 함수 | 설명 | 예시 |
|------|------|------|
| ./+ | 합계 | ns./+ |
| ./* | 곱 | ns./* |
| ./min | 최소 | ns./min |
| ./max | 최대 | ns./max |
| all(p) | 모두 만족 | ns.all(_>0) |
| any(p) | 하나라도 | ns.any(_<0) |

### 6.3 Search

| 함수 | 설명 | 예시 |
|------|------|------|
| find(p) | 조건 만족 첫 요소 | ns.find(_>10) |
| idx(v) | 값의 인덱스 | arr.idx(5) |
| argmax | 최대값 인덱스 | ns.argmax |
| argmin | 최소값 인덱스 | ns.argmin |

### 6.4 String

| 함수 | 설명 | 예시 |
|------|------|------|
| up | 대문자 | s.up |
| low | 소문자 | s.low |
| trim | 공백 제거 | s.trim |
| split(d) | 분리 | s.split(",") |
| join(d) | 합치기 | arr.join("-") |

### 6.5 Conversion

| 함수 | 설명 | 예시 |
|------|------|------|
| str | 문자열로 | n.str, 42.str |
| int | 정수로 | s.int, "42".int |
| float | 실수로 | s.float |

### 6.6 Utility

| 함수 | 설명 | 예시 |
|------|------|------|
| zip(a,b) | 병합 | zip(a,b) |
| range(s,e) | 범위 | range(1,10) |
| abs | 절대값 | n.abs, (-5).abs |
| min(a,b) | 최소 | min(a,b) |
| max(a,b) | 최대 | max(a,b) |

### 6.7 Error

| 함수 | 설명 | 예시 |
|------|------|------|
| err | 에러 발생 | age<0?err:age |
| err(msg) | 메시지와 함께 | err("invalid") |

---

## 7. Grammar (EBNF)

```ebnf
program     = function*

function    = identifier "(" [params] ")" "=" expr
params      = param ("," param)*
param       = identifier [":" type]

type        = "i" | "s" | "b" | "f"
            | "[" type "]"
            | "?" type
            | "{" field_types "}"

expr        = ternary
ternary     = logic ("?" expr ":" expr)?
logic       = compare (("&" | "|") compare)*
compare     = range_e (("==" | "!=" | "<" | ">" | "<=" | ">=") range_e)*
range_e     = add (".." add)?
add         = mult (("+" | "-") mult)*
mult        = unary (("*" | "/" | "%") unary)*
unary       = ("!" | "-" | "#")? postfix
postfix     = primary (index | call | field | chain)*

index       = "[" expr "]"
            | "[" [expr] ":" [expr] "]"
call        = "(" [args] ")"
field       = "." identifier
chain       = ".@" map_arg
            | ".?" filter_arg
            | "./" reduce_op

map_arg     = identifier | "(" lambda ")"
filter_arg  = identifier | "(" lambda ")"
reduce_op   = "+" | "*" | "min" | "max" | "and" | "or"

lambda      = "_" (field | index | binop)*
binop       = ("+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=") expr

primary     = identifier
            | number
            | string
            | "true" | "false" | "nil"
            | "$" "(" [args] ")"
            | "let" bindings ":" expr
            | "(" expr ")"
            | "[" [exprs] "]"
            | "{" [fields] "}"

bindings    = binding ("," binding)*
binding     = identifier "=" expr

args        = expr ("," expr)*
exprs       = expr ("," expr)*
fields      = field_def ("," field_def)*
field_def   = identifier ":" expr
```

---

## 8. Operator Precedence

높음 → 낮음:

| 우선순위 | 연산자 |
|----------|--------|
| 1 | . (field), [] (index), () (call) |
| 2 | .@ .? ./ (chaining) |
| 3 | # ! - (unary) |
| 4 | ** |
| 5 | * / % |
| 6 | + - |
| 7 | .. |
| 8 | @ (in) |
| 9 | == != < > <= >= |
| 10 | & |
| 11 | \| |
| 12 | ? : |
| 13 | = (assignment in let) |

---

## 9. Examples

### 9.1 Basic

```
// Hello World
hello()="Hello, World!"

// 덧셈
add(a,b)=a+b

// 최대값
max(a,b)=a>b?a:b

// 절대값
abs(n)=n<0?-n:n

// 짝수 확인
even(n)=n%2==0
```

### 9.2 Recursion

```
// 피보나치
fib(n)=n<2?n:$(n-1)+$(n-2)

// 팩토리얼
fact(n)=n<2?1:n*$(n-1)

// 최대공약수
gcd(a,b)=b==0?a:$(b,a%b)
```

### 9.3 Collections

```
// 합계
sum(ns)=ns./+

// 평균
avg(ns)=ns./+/#ns

// 중복 제거
uniq(ns)=ns.set

// 뒤집기
rev(arr)=arr.flip

// 평탄화
flat(ls)=ls.flatten
```

### 9.4 Higher-Order

```
// 맵: 모두 2배
dbl(ns)=ns.@(_*2)

// 필터: 양수만
pos(ns)=ns.?(_>0)

// 맵+필터+리듀스
sumpos(ns)=ns.?(_>0)./+

// 체이닝
emails(us)=us.?active.@email.@up
```

### 9.5 Complex

```
// 퀵소트
qs(a)=#a<2?a:let p=a[0],r=a[1:]:$(r.?(_<p))+[p]+$(r.?(_>=p))

// 이진 탐색
bs(a,t,lo=0,hi=#a-1)=lo>hi?nil:let m=(lo+hi)/2:a[m]==t?m:a[m]<t?$(a,t,m+1,hi):$(a,t,lo,m-1)

// 소수 판별
prime(n)=n<2?false:(2..n).all(n%_!=0)

// 나이 분류
cat(age)=age<0?err:age>=18?"adult":age>=13?"teen":"child"
```

---

## 10. Comparison with Python

| 기능 | Python | Vais v6b | 토큰 절감 |
|------|--------|----------|----------|
| 합계 | `sum(nums)` | `ns./+` | 50%+ |
| 필터+맵 | `[u.email for u in users if u.active]` | `us.?active.@email` | 50%+ |
| 최대값 | `max(a, b)` | `a>b?a:b` | 30%+ |
| 팩토리얼 | 5줄 함수 | `fact(n)=n<2?1:n*$(n-1)` | 40%+ |

---

## 11. Implementation Notes

### 11.1 Lexer

주요 토큰:
- IDENT, NUMBER, STRING
- DOT_AT (.@), DOT_Q (.?), DOT_SLASH (./)
- DOLLAR ($), HASH (#), AT (@)
- DOTDOT (..)
- QUESTION, COLON
- 기타 연산자

### 11.2 Parser

- Pratt Parser (연산자 우선순위 기반) 권장
- 재귀 하향도 가능

### 11.3 IR

기존 Vais IR 재사용 가능:
- Value, OpCode, Module, Function
- 체이닝 연산 → 일련의 IR 명령으로 변환

### 11.4 VM

기존 Vais VM 재사용 가능:
- 스택 기반 실행
- 빌트인 함수 추가 필요

---

## Changelog

### v1.0.0 (2026-01-12)
- Initial specification
- Phase 0 검증 완료
- 43.9% 토큰 절감 달성
