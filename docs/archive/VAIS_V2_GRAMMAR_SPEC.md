# Vais v2 Grammar Specification

**Version:** 0.1.0 (Draft)
**Date:** 2026-01-12
**Status:** Phase 0 - Validation

---

## 1. Design Principles

### 1.1 Token Efficiency
- 모든 키워드/타입을 최소 문자로
- 불필요한 구분자 제거
- 반복 패턴 최소화

### 1.2 Parsing Simplicity
- S-표현식 기반 (괄호로 구조 명확)
- 컨텍스트 프리 문법
- 단일 패스 파싱 가능

### 1.3 AI Generation Accuracy
- 일관된 패턴
- 예외 없는 규칙
- 명시적 구조

---

## 2. Lexical Elements

### 2.1 Comments
```lisp
; single line comment
```

### 2.2 Identifiers
```
[a-zA-Z_][a-zA-Z0-9_-]*
```
- 케밥 케이스 허용: `get-user-by-id`
- 언더스코어 허용: `get_user`

### 2.3 Literals

**Numbers**
```lisp
42          ; integer
3.14        ; float
0xFF        ; hex
0b1010      ; binary
1_000_000   ; with separators
```

**Strings**
```lisp
"hello"           ; basic string
"line1\nline2"    ; escape sequences
```

**Booleans**
```lisp
true
false
```

**Nil**
```lisp
nil
```

### 2.4 Type Abbreviations

| Full | Abbrev | Description |
|------|--------|-------------|
| int | i | 기본 정수 |
| int32 | i32 | 32비트 정수 |
| int64 | i64 | 64비트 정수 |
| float | f | 기본 실수 |
| float32 | f32 | 32비트 실수 |
| float64 | f64 | 64비트 실수 |
| string | s | 문자열 |
| bool | b | 불리언 |
| bytes | bs | 바이트 배열 |
| any | * | 모든 타입 |
| void | v | 반환값 없음 |

### 2.5 Composite Types
```lisp
[T]       ; array of T
{K:V}     ; map from K to V
?T        ; optional T (nullable)
(T1 T2)   ; tuple
```

### 2.6 Keywords (Minimal Set)
```
fn      ; function definition
let     ; variable binding
if      ; conditional
?       ; ternary (short if)
cond    ; multi-branch conditional
match   ; pattern matching
->      ; pipeline / arrow
loop    ; loop construct
for     ; for loop
do      ; sequencing
try     ; error handling
catch   ; error handler
throw   ; raise error
require ; assertion/precondition
async   ; async marker
await   ; await async
import  ; module import
export  ; module export
struct  ; struct definition
enum    ; enum definition
```

---

## 3. Grammar

### 3.1 Program Structure
```
program     = expr*
expr        = literal | symbol | list | vector | map
list        = '(' expr* ')'
vector      = '[' expr* ']'
map         = '{' (expr expr)* '}'
```

### 3.2 Function Definition
```lisp
(fn name [params] :return-type body)

; params = [name:type ...]
; return-type = abbreviated type

; Examples
(fn add [a:i b:i] :i
  (+ a b))

(fn greet [name:s] :s
  (str "Hello, " name))

; No params
(fn get-pi [] :f 3.14159)

; Multiple expressions (implicit do)
(fn complex [x:i] :i
  (let [y (+ x 1)]
    (* y 2)))
```

### 3.3 Lambda / Anonymous Function
```lisp
(\ [params] body)

; Examples
(\ [x] (* x 2))
(\ [a b] (+ a b))

; In context
(map (\ [x] (* x 2)) numbers)
```

### 3.4 Variable Binding
```lisp
(let [name value ...] body)

; Examples
(let [x 10] (* x 2))

(let [x 10
      y 20]
  (+ x y))

; Destructuring
(let [[a b] pair] (+ a b))
(let [{:name n :age a} person] ...)
```

### 3.5 Conditionals

**if**
```lisp
(if condition then-expr else-expr)

; Example
(if (> x 0) "positive" "non-positive")
```

**? (ternary shorthand)**
```lisp
(? cond then else)

; Same as if, but shorter
(? (> x 0) x (- x))
```

**cond (multi-branch)**
```lisp
(cond
  test1 result1
  test2 result2
  :else default)

; Example
(cond
  (< age 13) "child"
  (< age 20) "teen"
  :else "adult")
```

**match (pattern matching)**
```lisp
(match expr
  pattern1 result1
  pattern2 result2
  _ default)

; Example
(match status
  :ok "success"
  :err "failed"
  _ "unknown")
```

### 3.6 Pipeline Operator
```lisp
(-> value
    (fn1 args)
    (fn2 args)
    fn3)

; Threads value as first argument
; Example
(-> users
    (filter active?)
    (map email)
    (map upper))

; Equivalent to
(map upper (map email (filter active? users)))
```

**->> (thread last)**
```lisp
(->> value (fn1 args) (fn2 args))
; Threads value as last argument
```

### 3.7 Collection Operations

**Creating**
```lisp
[1 2 3]           ; vector/array
{:a 1 :b 2}       ; map
#{1 2 3}          ; set
```

**Accessing**
```lisp
(get coll key)
(. obj field)     ; field access
(.field obj)      ; alternative
(nth coll idx)
(first coll)
(last coll)
```

**Transforming**
```lisp
(map fn coll)
(filter pred coll)
(reduce fn init coll)
(sort coll)
(sort-by fn coll)
(reverse coll)
(take n coll)
(drop n coll)
(flatten coll)
```

**Aggregating**
```lisp
(count coll)
(sum coll)
(avg coll)
(min coll)
(max coll)
(all? pred coll)
(any? pred coll)
```

### 3.8 Arithmetic & Logic

**Arithmetic**
```lisp
(+ a b ...)    ; add (variadic)
(- a b)        ; subtract
(* a b ...)    ; multiply (variadic)
(/ a b)        ; divide
(% a b)        ; modulo
(** a b)       ; power
(abs x)        ; absolute value
(neg x)        ; negate
```

**Comparison**
```lisp
(= a b)        ; equal
(!= a b)       ; not equal
(< a b)        ; less than
(> a b)        ; greater than
(<= a b)       ; less or equal
(>= a b)       ; greater or equal
```

**Logic**
```lisp
(and a b ...)  ; logical and (variadic)
(or a b ...)   ; logical or (variadic)
(not a)        ; logical not
```

### 3.9 String Operations
```lisp
(str a b ...)     ; concatenate
(len s)           ; length
(upper s)         ; uppercase
(lower s)         ; lowercase
(trim s)          ; trim whitespace
(split s delim)   ; split string
(join coll delim) ; join with delimiter
(substr s start end)
(contains? s sub)
(starts? s prefix)
(ends? s suffix)
(replace s old new)
```

### 3.10 Error Handling
```lisp
; Precondition
(require condition "error message")

; Try-catch
(try
  expr
  (catch e
    handler))

; Throw error
(throw "error message")
(throw {:type :validation :msg "invalid"})
```

### 3.11 Loops & Iteration

**loop/recur (tail recursion)**
```lisp
(loop [bindings]
  body
  (recur new-values))

; Example: factorial
(fn fact [n:i] :i
  (loop [i n acc 1]
    (if (<= i 1)
      acc
      (recur (- i 1) (* acc i)))))
```

**for (comprehension)**
```lisp
(for [x coll] body)
(for [x coll :when pred] body)

; Example
(for [x (range 10) :when (even? x)]
  (* x x))
```

### 3.12 Struct Definition
```lisp
(struct Name
  [field1:type1
   field2:type2])

; Example
(struct User
  [id:i
   name:s
   email:s
   active:b])

; Creating instance
(User 1 "John" "john@example.com" true)

; Or with named fields
{:User id:1 name:"John" email:"john@example.com" active:true}
```

### 3.13 Enum Definition
```lisp
(enum Name
  Variant1
  Variant2
  (Variant3 type))

; Example
(enum Status
  Pending
  Active
  (Error s))

; Usage
(match status
  Status/Pending "waiting"
  Status/Active "running"
  (Status/Error msg) msg)
```

### 3.14 Module System
```lisp
; Import
(import "path/to/module")
(import "module" :as m)
(import "module" :only [fn1 fn2])

; Export
(export fn-name)
(export [fn1 fn2 fn3])
```

### 3.15 Async/Await
```lisp
; Async function
(fn fetch-user [id:s] :?User :async
  (await (http/get (str "/users/" id))))

; Await
(await async-expr)

; Parallel await
(await-all [expr1 expr2 expr3])
```

### 3.16 Special Forms Summary

| Form | Syntax | Description |
|------|--------|-------------|
| fn | `(fn name [args] :type body)` | 함수 정의 |
| \ | `(\ [args] body)` | 람다 |
| let | `(let [bindings] body)` | 바인딩 |
| if | `(if cond then else)` | 조건 |
| ? | `(? cond then else)` | 삼항 |
| cond | `(cond tests...)` | 다중 분기 |
| match | `(match expr patterns...)` | 패턴 매칭 |
| -> | `(-> val fns...)` | 파이프라인 |
| loop | `(loop [bindings] body)` | 루프 |
| try | `(try expr (catch e h))` | 에러 처리 |
| do | `(do expr1 expr2 ...)` | 순차 실행 |

---

## 4. Built-in Functions

### 4.1 Math
```lisp
(+ - * / %)
(abs neg)
(min max)
(floor ceil round)
(sqrt pow)
(sin cos tan)
(log log10 exp)
(rand rand-int)
```

### 4.2 String
```lisp
(str len)
(upper lower trim)
(split join)
(substr replace)
(contains? starts? ends?)
(parse-int parse-float)
```

### 4.3 Collection
```lisp
(map filter reduce)
(first last nth)
(take drop)
(sort sort-by reverse)
(flatten concat)
(count sum avg min max)
(all? any? none?)
(empty? contains?)
(keys vals)
(assoc dissoc update)
(group-by partition)
```

### 4.4 I/O (Future)
```lisp
(print println)
(read-file write-file)
(http/get http/post)
(json/parse json/stringify)
```

---

## 5. Examples

### 5.1 Hello World
```lisp
(fn hello [] :s "Hello, World!")
```

### 5.2 Add Two Numbers
```lisp
(fn add [a:i b:i] :i (+ a b))
```

### 5.3 Fibonacci
```lisp
(fn fib [n:i] :i
  (? (<= n 1) n (+ (fib (- n 1)) (fib (- n 2)))))
```

### 5.4 Factorial (loop)
```lisp
(fn fact [n:i] :i
  (loop [i n acc 1]
    (? (<= i 1) acc (recur (- i 1) (* acc i)))))
```

### 5.5 Filter Active Users
```lisp
(fn active-emails [users:[User]] :[s]
  (-> users
      (filter .active)
      (map .email)
      (map upper)))
```

### 5.6 Categorize Age
```lisp
(fn categorize [age:i] :s
  (require (>= age 0) "Age must be non-negative")
  (cond
    (>= age 18) "adult"
    (>= age 13) "teen"
    :else "child"))
```

### 5.7 Fetch User Orders Total
```lisp
(fn user-total [uid:s] :f :async
  (-> uid
      (http/get (str "/profile/" _))
      (. id)
      (http/get (str "/orders/" _))
      await
      (map .amount)
      sum))
```

### 5.8 Binary Search
```lisp
(fn bin-search [arr:[i] target:i] :?i
  (loop [lo 0 hi (- (len arr) 1)]
    (if (> lo hi)
      nil
      (let [mid (/ (+ lo hi) 2)
            val (nth arr mid)]
        (cond
          (= val target) mid
          (< val target) (recur (+ mid 1) hi)
          :else (recur lo (- mid 1)))))))
```

### 5.9 Quick Sort
```lisp
(fn qsort [arr:[i]] :[i]
  (if (<= (len arr) 1)
    arr
    (let [pivot (first arr)
          rest (drop 1 arr)
          less (filter (\ [x] (< x pivot)) rest)
          more (filter (\ [x] (>= x pivot)) rest)]
      (concat (qsort less) [pivot] (qsort more)))))
```

### 5.10 Group By Category
```lisp
(fn by-category [items:[Item]] :{s:[Item]}
  (group-by .category items))
```

---

## 6. Token Count Comparison

| Example | Python | Vais v2 | Savings |
|---------|--------|---------|---------|
| Hello World | 15 | 8 | 47% |
| Add Numbers | 20 | 12 | 40% |
| Fibonacci | 30 | 22 | 27% |
| Filter+Map | 25 | 18 | 28% |
| Categorize | 45 | 28 | 38% |
| API Call | 40 | 25 | 38% |
| Binary Search | 60 | 40 | 33% |
| Quick Sort | 55 | 38 | 31% |
| **Average** | - | - | **35%** |

---

## 7. EBNF Grammar (Formal)

```ebnf
program     = { expr } ;
expr        = atom | list | vector | map | quote ;
atom        = number | string | symbol | keyword | boolean | nil ;
list        = "(" { expr } ")" ;
vector      = "[" { expr } "]" ;
map         = "{" { expr expr } "}" ;
quote       = "'" expr ;

number      = integer | float ;
integer     = ["-"] digit { digit | "_" } ;
float       = ["-"] digit { digit } "." digit { digit } ;
string      = '"' { char } '"' ;
symbol      = ident ;
keyword     = ":" ident ;
boolean     = "true" | "false" ;
nil         = "nil" ;

ident       = (letter | "_") { letter | digit | "_" | "-" } ;
letter      = "a"-"z" | "A"-"Z" ;
digit       = "0"-"9" ;
```

---

## 8. Next Steps

1. [ ] 토큰 벤치마크 도구 작성
2. [ ] 10개 예제 Python/Vais v2 비교
3. [ ] LLM 생성 테스트
4. [ ] 결과 분석 및 Go/No-Go 결정

---

## Changelog

### v0.1.0 (2026-01-12)
- Initial draft
- S-expression based syntax
- Type abbreviations
- Core forms defined
- 10 examples
