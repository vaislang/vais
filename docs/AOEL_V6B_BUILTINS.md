# AOEL v6b Built-in Functions

**Version:** 1.0.0
**Date:** 2026-01-12

---

## Overview

AOEL v6b 빌트인 함수는 최소한으로 유지하면서 필수 기능을 제공합니다.

### 설계 원칙

1. **필수 기능만**: 없으면 구현 불가능한 것만
2. **일관된 네이밍**: 짧고 명확
3. **체이닝 가능**: 대부분 `.func` 형태로 호출

---

## 1. Collection Operations

### 1.1 Basic

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `#` | [T] → i | 길이 | `#arr`, `#str` |
| `first` | [T] → ?T | 첫 요소 | `arr.first` |
| `last` | [T] → ?T | 마지막 요소 | `arr.last` |
| `nth(n)` | [T] → ?T | n번째 요소 | `arr.nth(2)` |
| `flip` | [T] → [T] | 뒤집기 | `arr.flip` |
| `set` | [T] → [T] | 중복 제거 | `arr.set` |
| `flatten` | [[T]] → [T] | 평탄화 | `nested.flatten` |

### 1.2 Sorting

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `sort` | [T] → [T] | 오름차순 정렬 | `arr.sort` |
| `sortd` | [T] → [T] | 내림차순 정렬 | `arr.sortd` |
| `sort(f)` | [T] → [T] | 필드로 정렬 | `us.sort(_.age)` |

### 1.3 Slicing

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `take(n)` | [T] → [T] | 앞에서 n개 | `arr.take(3)` |
| `drop(n)` | [T] → [T] | 앞에서 n개 제외 | `arr.drop(2)` |
| `slice(s,e)` | [T] → [T] | s부터 e까지 | `arr.slice(1,4)` |

---

## 2. Higher-Order Operations

### 2.1 Map (.@)

```
.@expr          각 요소에 적용
.@field         필드 추출
.@(lambda)      람다 적용
```

| 예시 | 설명 | 결과 |
|------|------|------|
| `[1,2,3].@(_*2)` | 각 요소 2배 | `[2,4,6]` |
| `users.@name` | 이름 추출 | `["John","Jane"]` |
| `users.@(_.age+1)` | 나이+1 | `[31,26]` |
| `nums.@str` | 문자열로 변환 | `["1","2"]` |

### 2.2 Filter (.?)

```
.?expr          조건 만족하는 것만
.?field         필드가 truthy인 것만
.?(lambda)      람다 조건
```

| 예시 | 설명 |
|------|------|
| `[1,-2,3].?(_>0)` | 양수만 → `[1,3]` |
| `users.?active` | active인 사용자만 |
| `users.?(_.age>=18)` | 성인만 |

### 2.3 Reduce (./)

```
./op            접기 연산
```

| 연산자 | 설명 | 예시 | 결과 |
|--------|------|------|------|
| `./+` | 합계 | `[1,2,3]./+` | `6` |
| `./*` | 곱 | `[1,2,3]./*` | `6` |
| `./min` | 최소값 | `[3,1,2]./min` | `1` |
| `./max` | 최대값 | `[3,1,2]./max` | `3` |
| `./and` | 모두 true | `[true,false]./and` | `false` |
| `./or` | 하나라도 true | `[true,false]./or` | `true` |

---

## 3. Aggregation

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `all(p)` | [T] → b | 모두 만족 | `ns.all(_>0)` |
| `any(p)` | [T] → b | 하나라도 만족 | `ns.any(_<0)` |
| `none(p)` | [T] → b | 아무것도 안 만족 | `ns.none(_==0)` |
| `count(p)` | [T] → i | 만족하는 개수 | `ns.count(_>0)` |

---

## 4. Search

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `find(p)` | [T] → ?T | 첫 번째 만족 요소 | `ns.find(_>10)` |
| `findl(p)` | [T] → ?T | 마지막 만족 요소 | `ns.findl(_>10)` |
| `idx(v)` | [T] → ?i | 값의 인덱스 | `arr.idx(5)` |
| `idxf(p)` | [T] → ?i | 조건 만족 인덱스 | `arr.idxf(_>10)` |
| `argmax` | [T] → i | 최대값 인덱스 | `ns.argmax` |
| `argmin` | [T] → i | 최소값 인덱스 | `ns.argmin` |
| `has(v)` | [T] → b | 포함 여부 | `arr.has(5)`, `5@arr` |

---

## 5. String Operations

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `#` | s → i | 길이 | `#str` |
| `up` | s → s | 대문자 | `s.up` |
| `low` | s → s | 소문자 | `s.low` |
| `trim` | s → s | 공백 제거 | `s.trim` |
| `triml` | s → s | 왼쪽 공백 제거 | `s.triml` |
| `trimr` | s → s | 오른쪽 공백 제거 | `s.trimr` |
| `split(d)` | s → [s] | 분리 | `s.split(",")` |
| `join(d)` | [s] → s | 합치기 | `arr.join("-")` |
| `rep(n)` | s → s | 반복 | `"ab".rep(3)` → `"ababab"` |
| `sub(s,e)` | s → s | 부분 문자열 | `s.sub(0,3)` |
| `repl(a,b)` | s → s | 치환 | `s.repl("a","b")` |
| `starts(p)` | s → b | 시작 문자열 | `s.starts("http")` |
| `ends(p)` | s → b | 끝 문자열 | `s.ends(".txt")` |
| `has(p)` | s → b | 포함 여부 | `s.has("abc")` |

---

## 6. Type Conversion

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `str` | T → s | 문자열로 | `42.str`, `true.str` |
| `int` | s → ?i | 정수로 | `"42".int` |
| `float` | s → ?f | 실수로 | `"3.14".float` |
| `bool` | T → b | 불리언으로 | `0.bool` → `false` |
| `arr` | T → [T] | 배열로 | `"abc".arr` → `["a","b","c"]` |

---

## 7. Math

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `abs` | i/f → i/f | 절대값 | `(-5).abs` → `5` |
| `neg` | i/f → i/f | 부호 반전 | `5.neg` → `-5` |
| `min(a,b)` | (T,T) → T | 최소값 | `min(3,7)` → `3` |
| `max(a,b)` | (T,T) → T | 최대값 | `max(3,7)` → `7` |
| `floor` | f → i | 내림 | `3.7.floor` → `3` |
| `ceil` | f → i | 올림 | `3.2.ceil` → `4` |
| `round` | f → i | 반올림 | `3.5.round` → `4` |
| `pow(n)` | i/f → i/f | 거듭제곱 | `2.pow(3)` → `8` |
| `sqrt` | f → f | 제곱근 | `16.sqrt` → `4.0` |

---

## 8. Utility

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `zip(a,b)` | ([A],[B]) → [(A,B)] | 병합 | `zip([1,2],[3,4])` |
| `range(s,e)` | (i,i) → [i] | 범위 | `range(1,5)` → `[1,2,3,4]` |
| `enum` | [T] → [(i,T)] | 인덱스 부착 | `arr.enum` |
| `keys` | {K:V} → [K] | 키 목록 | `obj.keys` |
| `vals` | {K:V} → [V] | 값 목록 | `obj.vals` |

---

## 9. Error Handling

| 함수 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `err` | → ! | 에러 발생 | `age<0?err:age` |
| `err(msg)` | s → ! | 메시지와 함께 | `err("invalid age")` |
| `try(e,d)` | (T,T) → T | 에러 시 기본값 | `try(s.int,0)` |
| `nil?` | ?T → b | nil인지 | `val.nil?` |
| `ok?` | ?T → b | nil 아닌지 | `val.ok?` |

---

## 10. Special

| 기호 | 타입 | 설명 | 예시 |
|------|------|------|------|
| `$` | → fn | 자기 자신 재귀 | `$(n-1)` |
| `_` | → T | 람다 인자 | `_*2`, `_.name` |
| `@` | (T,[T]) → b | in 연산자 | `5@arr` |
| `..` | (i,i) → [i] | 범위 | `1..10` |
| `#` | [T]/s → i | 길이 | `#arr` |

---

## Summary by Category

### Collection (17)
`#`, `first`, `last`, `nth`, `flip`, `set`, `flatten`, `sort`, `sortd`, `take`, `drop`, `slice`, `has`, `idx`, `idxf`, `argmax`, `argmin`

### Higher-Order (3)
`.@`, `.?`, `./`

### Aggregation (4)
`all`, `any`, `none`, `count`

### Search (6)
`find`, `findl`, `idx`, `idxf`, `argmax`, `argmin`

### String (14)
`up`, `low`, `trim`, `triml`, `trimr`, `split`, `join`, `rep`, `sub`, `repl`, `starts`, `ends`, `has`, `#`

### Type Conversion (5)
`str`, `int`, `float`, `bool`, `arr`

### Math (10)
`abs`, `neg`, `min`, `max`, `floor`, `ceil`, `round`, `pow`, `sqrt`, `+`, `-`, `*`, `/`, `%`

### Utility (5)
`zip`, `range`, `enum`, `keys`, `vals`

### Error (5)
`err`, `try`, `nil?`, `ok?`

### Special (5)
`$`, `_`, `@`, `..`, `#`

---

## Total: ~60 Built-ins

(연산자 포함)
