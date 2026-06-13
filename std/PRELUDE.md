# New Vais 표준 prelude (S1 명세)

New Vais의 **prelude** — import 없이 항상 가용한 것 (설계 P5: 핵심 std는 prelude+global).
현재 bootstrap/oracle 백엔드는 Legacy Vais 재활용이므로, 각 New Vais API가 어떤 Legacy Vais 표면으로
매핑되는지 명시한다.
(자체 codegen/런타임은 L3 이후 — 그때 이 표면 API를 직접 구현.)

> 상태: 프로토타입. ✅=동작 검증, 🔶=부분/제약, 🔴=Legacy Vais 백엔드 버그로 막힘.

## 출력
| New Vais | Legacy Vais 매핑 | 상태 |
|----|-----------|------|
| `print(EXPR)` | `puts(EXPR)` (한 줄 출력, 보간 `"{x}"` 지원) | ✅ |

## 컬렉션
| New Vais | Legacy Vais 매핑 | 상태 |
|----|-----------|------|
| `[1, 2, 3]` (리스트 리터럴) | `Vec<i64> = [1,2,3]` + `use std/vec` 자동 | ✅ |
| `List<T>` (타입) | `Vec<T>` | ✅ |
| `Map<K,V>` (타입) | `HashMap<K,V>` + `use std/hashmap` 자동 | ✅ |
| `v.len()` | `v.len()` | ✅ |
| `v[i]` | `v[i]` | ✅ |
| `v.sum()` | `v.fold(0, \|a,x\| a+x)` | ✅ (reduce, 새 Vec 안 만듦) |
| `v.fold(init, \|a,x\| ...)` | `v.fold(...)` | ✅ (reduce) |
| `v.push(x)` | `v.push(x)` | ✅ |
| `v.map(\|x\| ...)` | `v.map(...)` | ✅ |
| `v.filter(\|x\| BOOL)` | `v.filter(\|x\| I (BOOL) {1} else {0})` | ✅ |

> Legacy Vais Vec 성장(push/map/filter)과 HashMap 기본 경로는 2026-06-11에 해결 확인됐다.
> New Vais self-host codegen 트랙은 여전히 Legacy Vec에 의존하지 않고 고정버퍼(`alloca [N x i64]`
> + 수동 length)로 push를 직접 구현한다(`fixpoint_list.vais`).

## 타입
| New Vais | Legacy Vais |
|----|------|
| `Int`/`Int8..Int128` | `i64`/`i8..i128` |
| `UInt8..UInt128` | `u8..u128` |
| `F32`/`F64` | `f32`/`f64` |
| `Bool`/`Str`/`Char` | `bool`/`str`/`char` |
| `String` (Rust직관) | `str` (정식 New Vais 타입명은 `Str` — vais-check가 `String`→`Str` 안내, 트랜스파일은 동작) |
| `(A, B)` 튜플 + `let (a, b) = ...` 구조분해 | `(A, B)` + `(a, b) := ...` | ✅ |
| `Int(x)`/`F64(x)`/`UInt8(x)`/… (숫자 변환) | `(x as i64)`/`(x as f64)`/… | ✅ (단순 인자형; `x as Int`은 vais-check가 금지) |

## 옵셔널/에러 (prelude 타입)
| New Vais | Legacy Vais | 상태 |
|----|------|------|
| `Option<T>` (Some/None) | `Option<T>` | ✅ |
| `Result<T, E>` (Ok/Err) | `Result<T, E>` | ✅ |
| `?` (전파) | `?` | ✅ |

## 연산자 (단어형 — 모호성 0)
| New Vais | Legacy Vais | 상태 |
|----|------|------|
| `and`/`or`/`not` | `&&`/`\|\|`/`!` | ✅ |
| `bitnot(x)` | `(~x)` | ✅ |
| `bitand(a,b)`/`bitor`/`bitxor`/`shl`/`shr` | `(a & b)`/`\|`/`^`/`<<`/`>>` | ✅ (단순 2-인자형; 중첩 인자는 변수 바인딩 후) |

## 루프 제어
| New Vais | Legacy Vais | 상태 |
|----|------|------|
| `break` (루프 종료) | `B` | ✅ |
| `continue` (반복 건너뜀) | `C` | ✅ |

## 정직한 한계
- 현재 prelude는 **Legacy Vais std 표면 사상**일 뿐, New Vais 자체 std 구현 아님. Legacy Vais std 변경/버그에 종속.
- Legacy Vais 백엔드 버그를 만나면 ROADMAP TRACKED에 묶고 Legacy repo에서 근본 수정한다.
- 자체 런타임/메모리/std 구현은 L3 이후 — 그때 P7(단일 coercion)/P8(클로저 day-1)을 올바르게.
