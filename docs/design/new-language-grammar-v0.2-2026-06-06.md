# 새 AI-native 언어(Aria) — 문법 명세 v0.2

**작성일**: 2026-06-06
**전신**: v0.1 (`new-language-grammar-draft-2026-06-06.md`) + 모호성 감사
(`new-language-ambiguity-audit-2026-06-06.md`).
**v0.2 변경 원칙**: 감사가 찾은 모호성·미정의를 **각각 정확히 한 가지**로 확정.
**"둘 다 허용"을 전부 제거**한다 (Vais 실패의 근본). 메타 규칙: *직관 수용 ≠ 두 길 허용.*

---

## 0. 설계 헌법 (불변, v0.1과 동일)
모호성 0 / 영어 키워드 / 중앙화 / 명시성 / AI 첫시도 정확(예제 코퍼스) / 에러가 수정법 / 컴파일≠정답.
비목표: 토큰 최소화.

---

## 1. v0.1에서 유지 (이미 모호성 0 — 감사 🟢)
- 바인딩: `let x = v` / `let mut x = v`. (단 하나)
- enum 접근: `Color.Red` — **점 하나로만** (`::` 금지, unqualified 금지).
- 타입 변환: `Int(x)` / `F32(x)` — 명시적 함수형으로만 (`as` 금지).
- 논리: `and` / `or` / `not` — 단어로만 (`&& || !` 금지).
- 비트: `bitand bitor bitxor bitnot shl shr` — 함수형으로만 (`~ & |` 금지).
- 키워드: 전부 영어 단어, 단일문자 없음, 식별자와 공간 분리.

---

## 2. v0.2 확정 — 모호성 제거 (감사 🔴🟡 해결)

### 2.1 컬렉션 생성 (A-1 해결) — **리터럴 하나로**
**결정**: 리스트/맵 생성은 **리터럴만**. `::new()` 폐기.
```
let a = [10, 20, 30]          # 비어있지 않은 리스트
let empty: List<Int> = []     # 빈 리스트 — 타입 명시 (리터럴로 추론 불가하므로)
let m = { "a": 1, "b": 2 }    # 맵
let empty_m: Map<Str, Int> = {}
```
- `List<Int>::new()` 같은 turbofish **제거** — Vais 함정 4 + v0.1 A-1 모호성 동시 차단.
- 빈 컬렉션만 타입 주석 필요 (리터럴이 비어 추론 불가). 한 가지 규칙.

### 2.2 컬렉션 연산 (A-7 해결) — **메서드 체인 하나로**
**결정**: 모든 컬렉션 연산은 **메서드**. 자유함수(`sum(v)`) **없음**.
```
let total = a.sum()
let big = a.filter(|x| x > 10).map(|x| x * 2).sum()
let n = a.len()
let first = a[0]
```
- `sum(v)`/`len(v)` 같은 자유함수 형태 제공 안 함. 한 가지 = 메서드 체인 (AI가 Rust iterator처럼 자연).

### 2.3 반복 (A-8 해결) — **용도별 한 가지**
**결정**: 한 상황 = 한 형태.
```
for i in 0..=n { }        # 정해진 범위/컬렉션 순회 — 이것만 범위에 씀
for x in list { }         # 컬렉션 순회
while cond { }            # 조건 반복
loop { ... break }        # 무한 + break
```
- 규칙: **범위/컬렉션 → for, 조건 → while, 무한 → loop.** 같은 상황에 두 형태 쓰지 않는다.
- C식 `for(init;cond;step)` **없음** (모호성·verbose).

### 2.4 match arm (A-2) — **값은 표현식, 탈출은 return** (의미 분리, 둘 다 필요)
```
let n = match c { Color.Red => 1, Color.Green => 2, Color.Blue => 3 }   # 값 생성
fn f(c: Color) -> Int {
    match c {
        Color.Red => return 0,    # 즉시 함수 탈출 (의미 다름)
        _ => {}
    }
    return 99
}
```
- 모호성 아님(의미가 다름). 단 에러/예제로 "값이냐 탈출이냐"를 강조.

---

## 3. v0.2 신규 정의 — 미정의 6종 (감사 §2 해결)
각각 **한 가지**로 확정.

### 3.1 문자열 보간 — **`"{expr}"` 하나로**
```
let name = "world"
let msg = "hello {name}, sum is {a + b}"   # 중괄호 안에 식. format(..) 함수 없음.
```
- `format(...)`/`+` 연결 대신 **보간 하나**. (Vais도 보간이라 자연.) 문자열 연결이 필요하면 보간으로.

### 3.2 에러 처리 — **`Result<T, E>` + `?` 하나로**
```
fn read(path: Str) -> Result<Int, Error> {
    let data = open(path)?      # ? : 실패면 그대로 전파
    return Ok(data.size())
}
```
- `?`로 전파, 처리는 `match`. 예외(throw/catch) **없음**. 한 가지 메커니즘.

### 3.3 Option — **`Option<T>` (Some/None) 하나로**
```
fn find(k: Int) -> Option<Int> {
    if k > 0 { return Some(k) }
    return None
}
let v = match find(1) { Some(x) => x, None => 0 }
```
- null 없음. 부재는 항상 `Option`. (T5 과제가 우회했던 것 — 이제 정의됨.)

### 3.4 가시성 — **`pub` 접두 하나로**
```
pub fn api() -> Int { 0 }     # 모듈 밖 노출
fn helper() -> Int { 0 }      # 기본 = 모듈 내부 (private)
pub struct Config { pub host: Str, port: Int }   # 필드별 pub
```
- 기본 private, `pub`로만 노출. `private`/`internal` 같은 추가 키워드 없음. 한 축.

### 3.5 제네릭 제약 — **`<T: Trait>` 하나로**
```
fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { return a }
    return b
}
struct Box<T> { value: T }
```
- 제약은 `<T: Trait>` 한 형태. `where` 절 **없음**(v0.2; 복잡 제약은 후순위). 한 가지.

### 3.6 모듈/임포트 — **`use path.to.module` 하나로**
```
use std.list          # prelude 외 표준 모듈
use myapp.utils       # 사용자 모듈 (점 경로)
```
- 경로 구분자는 **점 `.`** (enum 접근과 동일 기호 — 일관). `/`(Vais) 아님.
- prelude(List/Map/Set/Option/Result/기본 연산)는 import 불필요 (P5).

---

## 4. 에러 메시지 — Rust 직관 위험 지점 카탈로그 (P4, A-3/A-5/A-6 보완)
AI가 다른 언어 직관으로 틀릴 흔한 실수마다 `help:` + 수정코드:
| 잘못 쓴 것 (직관) | help |
|------|------|
| `a && b` | use `and`: `a and b` |
| `~x` | use `bitnot(x)` |
| `x as Int` | use `Int(x)` |
| `Color::Red` | use `.`: `Color.Red` |
| `List<Int>::new()` | use a literal: `[]` or `[1, 2, 3]` |
| `sum(v)` | use method: `v.sum()` |
| `format("{}", x)` | use interpolation: `"{x}"` |
| `use std/list` | use dots: `use std.list` |

---

## 5. 미해결 / v0.3 후보 (정직히 명시)
- `where` 절 (복잡 제네릭 제약).
- trait 기본 메서드 / 연관 타입.
- async/await 구체 의미 (키워드만 예약).
- 패턴 매칭 고급 (가드 `if`, 구조분해 깊이).
- 연산자 오버로딩 여부 (모호성 위험 — 신중히).
- 이것들도 정의 시 **"한 가지로"** 원칙 적용.

---

## 6. 모호성 재감사 (v0.2 self-check)
| 항목 | v0.1 | v0.2 |
|------|------|------|
| List 생성 | 🔴 2가지 | 🟢 리터럴 1가지 |
| 컬렉션 연산 | 🟡 미정의 | 🟢 메서드 1가지 |
| 반복 | 🟡 경계 | 🟢 용도별 1가지 |
| 문자열보간 | 🔴 미정의 | 🟢 `"{}"` 1가지 |
| 에러처리 | 🔴 미정의 | 🟢 Result+? 1가지 |
| Option | 🔴 미정의 | 🟢 Option 1가지 |
| 가시성 | 🔴 미정의 | 🟢 pub 1가지 |
| 제네릭제약 | 🔴 미정의 | 🟢 `<T:Trait>` 1가지 |
| 모듈 | 🔴 미정의 | 🟢 `use a.b` 1가지 |

**v0.2 판정: v0.1의 모호성/미정의를 전부 "한 가지"로 확정.** 남은 모호성 위험은 v0.3 후보(§5)뿐.
→ **모호성 0에 실질적으로 근접.** (단 §5 미정의 + 구현 시 파서 모호성은 잔존 가능.)

---

## 7. 다음 검증
1. ✅ v0.1 cold-start (5/5).
2. ✅ **v0.2 cold-start 재검증 — 6/6** (2026-06-06). 신규 AI(맥락 없음)에게 v0.2의 새 구문 6과제:
   문자열보간 `"{name}"`, Option Some/None, Result+`?` 전파, 제네릭 `<T:Ord>`, `pub` 가시성,
   메서드 체인 `.filter().sum()` — **전부 정확**. 특히 자유함수 유혹(`sum(v)`)·turbofish 회피 확인
   = A-1/A-7 해결이 실효. **모호성을 한 가지로 확정할수록 cold-start가 더 견고**(1/5→5/5→6/6).
3. 모호성 재감사 (v0.2, §6은 self-check — 외부/독립 재감사 권장).
4. 프로토타입 (표면 문법 파서 → Vais 백엔드).

## 8. 정직한 한계
- 종이 v0.2. 구현 시 파서 모호성·새 함정 가능.
- "AI가 잘 쓴다" 달성 가능 ≠ "범용 시장 대체"(생태계 해자, 경쟁력 보고서).
