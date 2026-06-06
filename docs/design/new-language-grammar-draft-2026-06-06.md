# 새 AI-native 언어 — 문법 명세 초안 v0.1

**작성일**: 2026-06-06
**가칭**: **Aria** (AI-Readable, 발음 쉽고 검색 가능; 확정 아님)
**근거**: `next-language-from-failures-2026-06-06.md`의 설계 원칙 P1~P9를 구체 구문으로 구현.
모든 결정에 어떤 원칙을 적용했는지 명시한다.

---

## 0. 설계 헌법 (모든 구문이 따르는 불변식)

1. **모호성 0** (P1,P2,P3): 한 토큰 = 한 의미. 한 작업 = 한 문법. 키워드 ≠ 식별자.
2. **영어스러움** (실측: AI 학습데이터 풍부): 키워드는 영어 단어. 단일문자 금지.
3. **중앙화** (P7,P8): 타입 변환·함수값 표현은 단일 규약. 산재 금지.
4. **명시성**: 암시적 변환 없음. 의도는 코드에 드러난다.
5. **AI 첫시도 정확** (P9, 실측 1/5→5/5): 문법 기능마다 검증된 예제. cold-start에서 못 틀리게.
6. **에러가 수정법을 준다** (P4): 모든 흔한 실수에 `help:` + 수정 코드.
7. **컴파일 ≠ 정답** (P7b): 값-정확성이 1급 검증.

> 비목표: **토큰 최소화** (실측으로 반증됨 — 모호성 비용 > 절약). 가독성·정확도 우선.

---

## 1. 어휘 (Lexical)

- **키워드 (예약어, 식별자로 못 씀)**: `fn struct enum trait impl if else match loop while for break continue return let mut use pub async await defer and or not true false`
  - 전부 영어 단어. 단일문자 키워드 없음 (Vais 실패 1 차단).
  - 예약어이므로 `struct A`의 `A`도 됨 — 키워드와 안 겹침 (P1).
- **연산자**: 한 기호 한 의미 (P2).
  - 산술 `+ - * / %`, 비교 `== != < > <= >=`, 논리 `and or not` (기호 `&& || !` 대신 단어 — AI가 더 정확).
  - 비트: `bitand bitor bitxor bitnot shl shr` (단어 — `~`/`&`의 문맥모호 차단, Vais 실패 2).
  - 할당 `=`, 바인딩 `let`/`let mut` (`:=`+`~mut` 모호성 제거, Vais 실패 2).
- **주석**: `#` 라인 주석.
- **리터럴**: `42`(int), `3.14`(float), `"text"`(string), `true`/`false`(bool), `'c'`(char).

---

## 2. 핵심 구문 — 예제로 (이게 "어떻게 생겼나")

### 2.1 함수 / 변수
```
fn add(a: Int, b: Int) -> Int {
    let sum = a + b      # 불변 바인딩
    let mut count = 0    # 가변: 'mut' 단어로 명시 (P2)
    count = count + 1
    return sum + count
}
```
- `let`/`let mut`로 바인딩 — 모호성 0 (Vais `:=`+`~` 실패 차단).
- 타입은 `: Type` 후위, 반환은 `-> Type`.

### 2.2 조건 / 반복 (영어 키워드, P1)
```
fn classify(n: Int) -> Int {
    if n < 0 {
        return 0
    } else if n == 0 {
        return 1
    } else {
        return 2
    }
}

fn sum_to(n: Int) -> Int {
    let mut s = 0
    for i in 0..=n {       # 범위 반복 (한 가지 문법, P3)
        s = s + i
    }
    return s
}
```

### 2.3 enum + match (variant 규칙 통일, Vais 실패 3 차단)
```
enum Color { Red, Green, Blue }

fn pick(c: Color) -> Int {
    match c {
        Color.Red => 1,     # 항상 'EnumName.Variant' — 한 가지 규칙 (P3)
        Color.Green => 2,
        Color.Blue => 3,
    }
}
```
- **결정: 항상 qualified `Color.Red`** (Vais는 unqualified만 허용해 Rust 직관과 충돌).
  qualified가 AI 학습데이터(Rust/Swift)와 가장 가까워 cold-start 정확. `::` 아닌 `.`로 통일.
- match arm은 표현식. **early return은 그냥 됨** (P6): `Color.Red => return 1` 도 허용.

### 2.4 struct (1글자 이름도 OK — P1)
```
struct Point { x: Int, y: Int }
struct V { value: Int }          # 1글자도 됨 (키워드와 안 겹침)

fn make() -> Int {
    let p = Point { x: 3, y: 4 }
    return p.x + p.y
}
```

### 2.5 컬렉션 (prelude, import 불필요 — P5; Vais 실패 5 차단)
```
fn build() -> Int {
    let mut v = List<Int>::new()    # 또는 [10, 20, 30] 리터럴
    v.push(10)
    v.push(20)
    let total = v.sum()             # 메서드 풍부 (Rust iterator처럼)
    return total + v[0]
}
```
- `List`/`Map`/`Set`은 **prelude에 항상 가용** (Vais `use std/vec` + 환경의존 실패 차단).
- **타입 인자 위치 통일**: `List<Int>::new()` 허용 (Rust turbofish 직관 수용 — Vais 실패 4 차단).
  `[10, 20, 30]` 리터럴도 제공.

### 2.6 클로저 — `{code, env}` 1급 값 (P8, Vais 실패 8 차단)
```
fn apply(f: fn(Int) -> Int, x: Int) -> Int {
    return f(x)
}

fn main() -> Int {
    let base = 10
    let add_base = |n| n + base    # 캡처 'base'를 자동으로 env에 담음
    return apply(add_base, 5)       # 경계 넘어도 캡처 유지 (15) — Vais 정렬버그 클래스 차단
}
```
- **모든 함수값은 `{code_ptr, env}`** — 캡처 유무 무관 단일 규약 (day-1 확정).
- `fn(Int)->Int` 타입은 캡처 가능한 closure를 받는다 (Vais의 bare-fn-ptr E001 실패 차단).
- **이게 vaisdb 정렬 무음오류의 근본 차단** — 캡처가 항상 동반되므로 경계에서 안 사라짐.

### 2.7 비트 연산 — 단어로 (Vais 실패 2 차단)
```
fn flip(x: Int) -> Int {
    return bitnot(x)        # '~' 모호성 없음. 함수형 단어
}
```

### 2.8 숫자 타입 — 명시적 변환 (Vais f32 실패 7의 교훈)
```
fn area(w: F32, h: F32) -> F32 {
    return w * h           # F32 리터럴은 F32로 추론 (단일 coercion 지점)
}
fn to_int(x: F32) -> Int {
    return Int(x)          # 명시적 변환 (암시적 금지)
}
```
- 타입 이름: `Int Int8..Int128 UInt8..UInt128 F32 F64 Bool Str Char`. 명확한 영어/숫자.
- **리터럴→typed 변환은 컴파일러 단일 지점**에서 (P7 — Vais f32 10-site 산재 실패 차단).

---

## 3. 타입 변환 규칙 (P7, 명시성)
- ✅ 암시적: 정수 widening (`Int8→Int16→...`), float 리터럴 문맥 추론.
- ❌ 금지(명시적 `Type(x)` 필요): narrowing, int↔float, bool↔int, str↔int.
- 모든 변환은 **단일 coercion 함수**를 거친다 (store/arg/return/field/array 전부). 산재 금지.

---

## 4. 에러 메시지 규약 (P4 — AI self-correction)
모든 에러는 다음을 포함한다:
```
error: <무엇이 틀렸나>
  --> file:line:col
help: <어떻게 고치나> + <정확한 수정 코드>
```
예 (Vais가 못 했던 것):
```
error: enum variant must be qualified
  --> x.aria:4:9
  4 |   Red => 1
  help: write the enum name: `Color.Red => 1`
```
- **흔한 실수 카탈로그**를 유지 — 각 실수에 `help:`. cold-start AI가 1라운드 수렴.

---

## 5. 예제 코퍼스 인프라 (P9 — 실측 최강 레버)
- 모든 문법 기능마다 **컴파일+런타임 검증된 예제** (Vais LIVING_SPEC 강화판).
- 컴파일러가 `aria examples <feature>`로 관련 예제 제공.
- AI 생성→`aria build` 검증→정답쌍 자동 축적.
- **실측 근거**: 예제 유무로 cold-start 1/5 → 5/5.

---

## 6. 검증/게이트 (P7b)
- 모든 컴파일러 테스트는 **값-정확성**(런타임 결과)을 assert. "컴파일 성공"만으로 통과 금지.
- 정렬·검색 같은 알고리즘은 결과 순서/값을 검증 (Vais vaisdb 정렬 무음오류 클래스 차단).

---

## 7. 재활용 (백지지만 0 아님)
- Vais 백엔드 자산 재활용: LLVM codegen(단일 coercion으로 재구성), 타입체커, self-host 노하우,
  빠른 프론트엔드(Rust 2배), 게이트 패턴.
- 폐기: 단일문자 키워드, 토큰절약, 산재 coercion, bare-fn-ptr 클로저.

---

## 8. 명세 검증 — 1차 실측 완료 (2026-06-06)
이 명세는 **가설**이었고, 1차 검증을 했다.

**실험**: 신규 AI(이 대화 맥락 없음, Aria 처음 봄)에게 이 명세만 주고 5과제 cold-start 생성.
- **결과: 5/5** — qualified `Color.Green`, list 리터럴 `[10,20,30]`+`.sum()`, `bitnot(0)`,
  1글자 가능 struct, 캡처 클로저 `|x| x+bonus` 전부 자연스럽고 정확.
- **대조**: 같은 신규 AI가 **Vais** 명세로 같은 5과제 → **1/5** (`Color::Red`/`~0`/`=>return`/
  turbofish/import 함정에 빠짐).

**판정: P1~P9 설계가 cold-start AI 정확도를 1/5 → 5/5로 끌어올림 (실측).**
모호성 제거(P2: `~`→`bitnot`)·영어 키워드(P1)·Rust직관 수용(P3: `Color.Green`)·캡처 클로저(P8)가
함정 자체를 없앴다.

**한계**: 컴파일러 부재로 "컴파일 정확성"이 아닌 "명세 준수"를 측정. 표본 5. 구현 시 새 함정 가능.

### 남은 검증
1. ✅ cold-start 명세준수 정확도 (완료: 5/5).
2. 모호성 0 감사: 같은 의미를 두 가지로 쓸 수 있는 곳 점검.
3. 프로토타입: 이 표면 문법 파서 → Vais 백엔드 → 실제 컴파일 (실제 함정 노출).

---

## 9. 정직한 한계
- 이건 **종이 설계 v0.1**. 구현하면 새 함정이 나온다 (Vais도 그랬다).
- "AI가 잘 쓴다"는 달성 가능하나 **"범용 시장 대체"는 생태계 해자로 별개** (경쟁력 보고서).
- 가칭 Aria는 미확정. 핵심은 이름이 아니라 P1~P9 구현.

## 10. 모호성 감사 결과 (2026-06-06) — v0.1은 모호성 0 미달성
`new-language-ambiguity-audit-2026-06-06.md` 참조. 발견:
- 🔴 **A-1 List 생성이 실제 모호** (`List<Int>::new()` vs `[...]` 둘 다 허용 — Vais turbofish 함정 재발).
- 🟡 A-7 메서드 vs 자유함수 미정의, A-8 for/while/loop 경계 미정의.
- 🔴 미정의 6종(문자열보간/에러처리/가시성/Option·Result/제네릭제약/모듈) = 잠재 모호성.
- 🟢 let mut, enum `.`, 변환 `Int(x)`, 논리 `and/or/not`은 모호성 0 달성.
- **메타 교훈: "Rust 직관 수용 ≠ 두 길 허용". 직관에 맞는 한 가지를 골라라.** 둘 다 열면 Vais 반복.
- → **v0.2에서 위 항목을 각각 "한 가지로" 확정해야 모호성 0에 근접.** (감사 §4 지침)
