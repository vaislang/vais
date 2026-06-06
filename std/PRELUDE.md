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
| `Map<K,V>` (타입) | `HashMap<K,V>` + `use std/hashmap` 자동 | 🔴 Vais HashMap codegen 버그 (B-01/B-02) |
| `v.len()` | `v.len()` | ✅ |
| `v[i]` | `v[i]` | ✅ |
| `v.sum()` | `v.fold(0, \|a,x\| a+x)` | ✅ (reduce, 새 Vec 안 만듦) |
| `v.fold(init, \|a,x\| ...)` | `v.fold(...)` | ✅ (reduce) |
| `v.push(x)` | `v.push(x)` | 🔴 Vais Vec 성장 codegen 버그 (`@Vec_push` 무음 miscompile, len 오염) |
| `v.map(\|x\| ...)` | `v.map(...)` | 🔴 Vais Vec_push 버그 (map이 새 Vec push) |
| `v.filter(\|x\| BOOL)` | `v.filter(\|x\| I (BOOL) {1} else {0})` | 🔴 Vais filter 버그 (task_7cfebeba) |

> **Vais Vec 성장(push/map/filter)은 막힘** (read-only len/index/fold/sum은 OK). 리스트를 변형/구축하려면
> `for`-루프로 누적(e25/e27 패턴). nl self-host codegen 트랙은 Vais Vec를 안 쓰고 고정버퍼(`alloca [N x i64]`
> + 수동 length)로 push를 직접 구현해 이 갭을 우회한다(fixpoint_list.nl). → ROADMAP TRACKED.

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
| nl | Vais | 상태 |
|----|------|------|
| `and`/`or`/`not` | `&&`/`\|\|`/`!` | ✅ |
| `bitnot(x)` | `(~x)` | ✅ |
| `bitand(a,b)`/`bitor`/`bitxor`/`shl`/`shr` | `(a & b)`/`\|`/`^`/`<<`/`>>` | ✅ (단순 2-인자형; 중첩 인자는 변수 바인딩 후) |

## 정직한 한계
- 현재 prelude는 **Vais std 표면 사상**일 뿐, nl 자체 구현 아님. Vais std 변경/버그에 종속.
- `filter` 등 Vais 버그가 있는 API는 nl도 막힘 (TRACKED → Vais repo에서 수정).
- 자체 런타임/메모리/std 구현은 L3 이후 — 그때 P7(단일 coercion)/P8(클로저 day-1)을 올바르게.
