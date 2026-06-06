# nl 예제 코퍼스 (P9 인프라)

**검증된 nl 예제.** P9(예제 코퍼스 = 최강 레버, cold-start 1/5→5/5)의 핵심 인프라.
모든 `# expect: N` 예제는 `scripts/test.sh`로 빌드+실행+값 검증된다 (현재 61/61 PASS; 러너 전체는
self-host codegen 모듈 포함 79/79).

> 사용: `scripts/test.sh` (전체) / `scripts/test.sh e03_recursion` (하나).
> AI에게 nl을 가르칠 때 이 예제들을 컨텍스트로 제공하면 cold-start 정확도가 오른다(실측).
>
> **재확인 (2026-06-06)**: nl을 처음 보는 신선한 서브에이전트에게 이 코퍼스만 주고 코퍼스에 없는
> 새 과제(재귀 삼각수)를 시켰더니 **첫 시도에 컴파일+실행 성공**(값 28 정확). 결과를 e20으로 승격.
> 맥락 없는 진짜 AI로 P9 명제(예제가 cold-start를 가능케 함) 재입증.

## 문법 커버리지 인덱스

| 예제 | 커버 문법 | expect |
|------|-----------|--------|
| c1 | enum + match + `=> return` | 2 |
| c2 | list 리터럴 + `.sum()` | 60 |
| c3 | `bitnot()` | 5 |
| c4 | struct 기본 | 42 |
| c5 | 클로저 캡처 (in-scope) | 7 |
| d3run | Result + `?` 전파 | 6 |
| d4b | List 파라미터 + for | 9 |
| d5run | pub struct | 42 |
| fr1 | for-range `0..=n` | 15 |
| fr2 | for-collection | 60 |
| t1 | 함수 + `let` | 7 |
| e01 | 중첩 struct (3-deep) | 9 |
| e02 | enum 다중 payload | 42 |
| e03 | 재귀 (factorial) | 120 |
| e04 | 상호 재귀 | 1 |
| e05 | 다중 함수 + 중첩 호출 | 24 |
| e06 | for-range 합산 | 55 |
| e07 | else if 체인 | 2 |
| e08 | Option + arm-return | 8 |
| e09 | struct 메서드 체인 | 25 |
| e10 | bool 논리 (and/or/not) | 1 |
| e11 | while 루프 | 10 |
| e12 | exclusive range `..` | 10 |
| e13 | 중첩 for | 9 |
| e14 | print (출력) | 0 |
| e15 | List 재귀 (`&List<T>` borrow) | 10 |
| e16 | Option match + payload 바인딩 | 42 |
| e17 | struct 반환 → 필드 접근 | 12 |
| e18 | while 누적기 (가변 acc + 카운터) | 30 |
| e19 | 문자열 보간 출력 `print("{x}")` | 0 |
| e20 | 재귀 삼각수 (cold-start AI 작성) | 28 |
| e21 | Result Ok/Err match | 7 |
| e22 | enum 디스패치 (match in helper) | 12 |
| e23 | Option 흐름 (lookup→match→합산) | 15 |
| e24 | struct 필드가 enum + match | 1 |
| e25 | for + if 필터 합산 (수동 filter) | 12 |
| e26 | 함수 합성 파이프라인 (중첩 호출) | 19 |
| e27 | List 파라미터 max (loop, bound-var 전달) | 9 |
| e28 | struct 함수적 갱신 (rebuild+재대입) | 8 |
| e29 | GCD (2-인자 재귀 + modulo) | 6 |
| e30 | enum payload match (페이로드 바인딩) | 42 |
| e31 | bitwise (bitor/bitand/... 단어형) | 7 |
| e32 | 중첩 struct 필드 변이 (o.inner.v=) | 7 |
| e33 | guard 체인 (조기 return 분기) | 2 |
| e34 | 숫자 변환 Int(x) (call형, F64→Int) | 5 |
| e35 | 계산기 enum 디스패치 + bitwise (cold-start) | 2 |
| e36 | bool 반환 함수 → if 조건 | 1 |
| e37 | 다중필드 struct 계산 (area=w*h) | 50 |
| e38 | 음수 (0 - n) + 산술 | 5 |
| e39 | `?` 에러 전파 (실패 경로) | 0 |
| e40 | Option을 struct 필드로 + match | 7 |
| e41 | 재귀로 struct accumulator 전달 | 6 |
| e42 | while로 함수 반복 적용 | 8 |
| e43 | 제네릭 함수 identity<T> | 5 |
| e44 | 문자열 길이 s.len() | 5 |
| e45 | 문자열 동등성 a == b | 1 |
| e46 | 제네릭 함수 + struct 조합 | 7 |
| e47 | char 리터럴 + 비교 ('A') | 1 |
| e48 | 문자열 바이트 인덱싱 s[0] | 104 |
| e49 | 클로저를 함수 인자로 (일급함수) | 12 |
| e50 | 식 평가기 (enum Node + match 디스패치) | 14 |

## 미커버 (Vais 백엔드/트랜스파일러 한계 — ROADMAP TRACKED)
- **Vec 성장 `.push()`/`.map()`/`.filter()`** — Vais 백엔드 버그(`@Vec_push` 무음 miscompile/undefined).
  read-only(len/index/fold/sum)는 OK. 리스트 변형/구축은 **`for`-루프 누적**으로(e25/e27 참조).
- `Map<K,V>` (HashMap) — **Vais 백엔드 버그**: `HashMap.new()` 모노모픽화 누락(C002/E004 undefined
  `@HashMap_new`) + `get_opt` Option ptr/i64 불일치. Vais repo `tests/empirical/codegen_bugs/B-01,B-02`에
  repro 존재. nl 트랜스파일이 아니라 Vais codegen 문제 → Vais repo 수정 필요. PRELUDE 🔶.
- 중첩 `match` (arm 안에 `=> match {...}`) — 라인-재작성기 P001(트랜스파일러 한계). 함수분리로 우회 가능.
- **`impl Trait for Type`** (trait 기반 다형성) — Vais 미지원(P001). **`impl Type { ... }`(inherent 메서드)는
  OK**(e09/e43) → 구조체 메서드는 정상, trait 디스패치만 막힘. ROADMAP TRACKED.
- **재귀(자기참조) enum** (`enum Expr { Add(Expr, Expr) }`) — Vais 무음 miscompile. **비재귀 enum은 OK**(e50) →
  실전 AST는 struct+인덱스 인코딩으로(self-host codegen 트랙 방식). ROADMAP TRACKED.
- **중첩 리스트 `[[..]]`** (`List<List<Int>>`) — **Vais 백엔드 버그**(C003 nested Vec). 트랜스파일러는 올바른
  `Vec<Vec<i64>>` 타입 생성하나 Vais가 codegen 못 함. ROADMAP TRACKED.
- **리스트 리터럴을 함수 인자로 직접 전달** (`f([1,2,3])`) — Vais 코어션 갭(E001). **우회: 변수에 바인딩 후
  전달**(`let v = [1,2,3]; f(v)` — e27 참조). ROADMAP TRACKED.

## 규약
- 첫 줄 `# expect: N` = main이 반환할 exit code (mod 256).
- 실행형(main이 값 반환)만 expect 부착. 라이브러리 조각은 미부착(러너 skip).
