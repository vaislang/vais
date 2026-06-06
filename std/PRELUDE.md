# nl 표준 prelude (S1 명세)

nl의 **prelude** — import 없이 항상 가용한 것 (설계 P5: 핵심 std는 prelude+global).
현재 백엔드는 Vais 재활용이므로, 각 nl API가 어떤 Vais로 매핑되는지 명시한다.
(자체 codegen/런타임은 L3 이후 — 그때 이 표면 API를 직접 구현.)

> 상태: 프로토타입. ✅=동작 검증, 🔶=부분/제약, 🔴=Vais 백엔드 버그로 막힘.

## 출력
| nl | Vais 매핑 | 상태 |
|----|-----------|------|
| `print(EXPR)` | `puts(EXPR)` (한 줄 출력, 보간 `"{x}"` 지원) | ✅ |

## 컬렉션
| nl | Vais 매핑 | 상태 |
|----|-----------|------|
| `[1, 2, 3]` (리스트 리터럴) | `Vec<i64> = [1,2,3]` + `use std/vec` 자동 | ✅ |
| `List<T>` (타입) | `Vec<T>` | ✅ |
| `Map<K,V>` (타입) | `HashMap<K,V>` + `use std/hashmap` 자동 | 🔶 (미검증) |
| `v.len()` | `v.len()` | ✅ |
| `v[i]` | `v[i]` | ✅ |
| `v.push(x)` | `v.push(x)` | ✅ |
| `v.sum()` | `v.fold(0, \|a,x\| a+x)` | ✅ |
| `v.map(\|x\| ...)` | `v.map(...)` | ✅ |
| `v.fold(init, \|a,x\| ...)` | `v.fold(...)` | ✅ |
| `v.filter(\|x\| BOOL)` | `v.filter(\|x\| I (BOOL) {1} else {0})` | 🔴 Vais filter 버그 (task_7cfebeba) |

## 타입
| nl | Vais |
|----|------|
| `Int`/`Int8..Int128` | `i64`/`i8..i128` |
| `UInt8..UInt128` | `u8..u128` |
| `F32`/`F64` | `f32`/`f64` |
| `Bool`/`Str`/`Char` | `bool`/`str`/`char` |
| `Int(x)` (변환) | `x as i64` | ✅ |

## 옵셔널/에러 (prelude 타입)
| nl | Vais | 상태 |
|----|------|------|
| `Option<T>` (Some/None) | `Option<T>` | ✅ |
| `Result<T, E>` (Ok/Err) | `Result<T, E>` | ✅ |
| `?` (전파) | `?` | ✅ |

## 연산자 (단어형 — 모호성 0)
| nl | Vais |
|----|------|
| `and`/`or`/`not` | `&&`/`\|\|`/`!` |
| `bitnot(x)`/`bitand`/`bitor`/`bitxor`/`shl`/`shr` | `~`/`&`/`\|`/`^`/`<<`/`>>` |

## 정직한 한계
- 현재 prelude는 **Vais std 표면 사상**일 뿐, nl 자체 구현 아님. Vais std 변경/버그에 종속.
- `filter` 등 Vais 버그가 있는 API는 nl도 막힘 (TRACKED → Vais repo에서 수정).
- 자체 런타임/메모리/std 구현은 L3 이후 — 그때 P7(단일 coercion)/P8(클로저 day-1)을 올바르게.
