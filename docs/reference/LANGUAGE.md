# New Vais 언어 레퍼런스

> New Vais 작성용 실용 레퍼런스. 전환기 파일 확장자는 `.nl`이다.
> **모든 구문은 `examples/`로 검증됨**(값-정확성 게이트 통과).
> 설계 근거(왜 이렇게)는 `docs/design/`; 이 문서는 "어떻게 쓰나".
> 핵심 규칙: **한 작업 = 한 가지 문법** (모호성 0). Rust 직관으로 쓰면 `nl-check`가 수정 안내.

---

## 1. 함수
```
fn add(a: Int, b: Int) -> Int {
    return a + b
}
```
- `fn 이름(파라미터: 타입, ...) -> 반환타입 { ... }`
- 반환은 `return`. 진입점은 `fn main() -> Int`.

## 2. 변수
```
let x = 5          # 불변
let mut count = 0  # 가변
count = count + 1  # 재대입은 가변만
```
- 바인딩은 `let` / `let mut` **한 가지**. (`:=`/`~` 같은 다른 형태 없음.)
- 타입은 보통 추론. 명시하려면 `let x: Int = 5`.

## 3. 타입
| 종류 | 이름 |
|------|------|
| 정수 | `Int`, `Int8`/`Int16`/`Int32`/`Int64`/`Int128`, `UInt8`..`UInt128` |
| 실수 | `F32`, `F64` |
| 기타 | `Bool`, `Str`, `Char` |
| 컬렉션 | `List<T>`, `Map<K,V>` |
- 변환은 **명시적 함수형**: `Int(x)`, `F32(y)`. (`x as Int` 안 됨 — nl-check가 잡음.)

## 4. 조건
```
if n < 0 {
    return 0
} else if n == 0 {
    return 1
} else {
    return 2
}
```
- 논리는 **단어**: `and`, `or`, `not`. (`&&`/`||`/`!` 안 됨.)
  ```
  if a and not b { ... }
  ```

## 5. 반복
```
for i in 0..=10 { ... }   # 포함 범위 (10 포함)
for i in 0..10 { ... }    # 제외 범위 (10 미포함)
for x in items { ... }    # 컬렉션 순회
while cond { ... }        # 조건 반복
```
- 규칙: 범위/컬렉션 → `for`, 조건 → `while`. C식 `for(;;)` 없음.

## 6. enum + match
```
enum Color { Red, Green, Blue }

fn pick(c: Color) -> Int {
    match c {
        Color.Red => 1,       # variant는 항상 점: EnumName.Variant
        Color.Green => 2,
        Color.Blue => 3,
    }
}
```
- variant 접근은 **점 `.`** 한 가지. (`::` 안 됨, unqualified 안 됨.)
- payload: `enum Shape { Circle(Int), Rect(Int, Int) }` → `match s { Circle(r) => r, Rect(w, h) => w*h }`.
- arm에서 즉시 반환: `Color.Red => return 1` (그냥 됨).

## 7. struct
```
struct Point { x: Int, y: Int }

fn main() -> Int {
    let p = Point { x: 3, y: 4 }
    return p.x + p.y
}
```
- 1글자 이름도 됨 (`struct V { ... }`) — 키워드와 안 겹침.
- 메서드 (impl):
  ```
  impl Counter {
      fn bump(self, n: Int) -> Counter { return Counter { value: self.value + n } }
      fn get(self) -> Int { return self.value }
  }
  ```
- 가시성: `pub struct`, 필드 `pub name: ...` (기본 private).

## 8. 컬렉션
```
let a = [10, 20, 30]              # 리스트 리터럴
let empty: List<Int> = []        # 빈 것은 타입 명시
let total = a.sum()              # 메서드 체인
let big = a.filter(|x| x > 10).map(|x| x * 2).sum()
let n = a.len()
let first = a[0]
```
- 컬렉션 연산은 **메서드** 한 가지. 자유함수(`sum(v)`) 없음.

## 9. 클로저
```
let base = 10
let add = |n| n + base       # 캡처 자동
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x) }
```
- `|파라미터| 식` 또는 `|파라미터| { ... }`.
- 캡처는 자동. 반환된 캡처 클로저와 고차함수 재전달도 코퍼스에서 검증됨(e80~e81).

## 10. Option / Result / 에러
```
fn find(k: Int) -> Option<Int> {
    if k > 0 { return Some(k) }
    return None
}
let v = match find(5) { Some(x) => x, None => 0 }

fn read(a: Int, b: Int) -> Result<Int, Str> {
    if b == 0 { return Err("zero") }
    return Ok(a / b)
}
fn use_it() -> Result<Int, Str> {
    let r = read(10, 2)?   # ? : 실패면 전파
    return Ok(r + 1)
}
```
- 부재는 항상 `Option` (null 없음). 실패는 `Result` + `?`.

## 11. 출력
```
let x = 42
print("the answer is {x}")     # 보간 "{식}"
```
- `print(EXPR)` — 한 줄 출력, `"{식}"` 보간 지원. (`format(...)`/`+` 연결 없음.)

## 12. 연산자
- 산술: `+ - * / %`
- 비교: `== != < > <= >=`
- 논리(단어): `and or not`
- 비트(함수): `bitnot(x) bitand(a,b) bitor(a,b) bitxor(a,b) shl(x,n) shr(x,n)`

---

## 흔한 실수 → 올바른 nl (nl-check가 잡아줌)
| Rust 직관 (틀림) | nl (맞음) |
|------|------|
| `a && b` | `a and b` |
| `a \|\| b` | `a or b` |
| `!x` | `not x` |
| `x as Int` | `Int(x)` |
| `Color::Red` | `Color.Red` |
| `List<Int>::new()` | `[]` 또는 `[1,2,3]` |
| `sum(v)` | `v.sum()` |
| `format("{}", x)` | `"{x}"` |

---

## 빌드/검증
```
scripts/build.sh prog.nl -o out && ./out   # 빌드+실행
scripts/test.sh                            # 값-정확성 (examples 전체)
scripts/vaisc emit-ir program.vais -o program.ll  # New Vais compiler IR 출력
scripts/vaisc build program.vais -o program       # New Vais compiler + clang
scripts/test-vaisc-front.sh                # native day-1 front 계약 검증
python3 tools/nl-check.py prog.nl          # 문법 lint (help: 수정안내)
```

## 현재 한계 (전환기)
- 현재 실행 경로는 Legacy Vais 재활용(트랜스파일)이다. 새 Vais 자체 컴파일러가 mainline으로 전환 중이다.
- `scripts/vaisc` native day-1 front는 Int 함수/let/return/if/while/plain call만 받는다.
  넓은 언어 표면은 아직 Legacy bootstrap 경로나 후속 native slice에서 다룬다.
- 진짜 차별점(P7 단일coercion, P8 클로저 day-1, P4 help 에러)은 자체 컴파일러에서 최종 소유한다.
- 이 레퍼런스의 모든 예제는 검증됨(`examples/`); 미검증 구문은 안 적었다.
