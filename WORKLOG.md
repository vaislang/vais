# nl WORKLOG

## 2026-06-06 (/loop iter 1: P0 — 값-정확성 게이트)
- 폴더 구조를 언어 전체로 보강 (std/ tools/ docs/reference/ tests/).
- ROADMAP.md 신설 (단일 진실원, P0~P5 우선순위 + L3 진입 + TRACKED).
- **G1 값-정확성 러너** scripts/test.sh: examples/*.nl `# expect: N` → 트랜스파일+빌드+실행+exit 비교.
- **G2** 실행형 예제 11개에 expect 주석 부착.
- baseline: **11/11 PASS** (filter d6 등 Vais버그/no-main은 expect 없어 제외).
- 의미: 이제 모든 후속 작업이 러너 green으로 보호됨 (P7b 컴파일≠정답 인프라).

## 2026-06-06 (/loop iter 2: P1 — 예제 코퍼스 2배 확장)
- C1: 10개 신규 예제 (e01~e10): 중첩struct/enum-payload/재귀/상호재귀/다중함수/for/else-if/Option/
  메서드체인/bool논리. 전부 # expect + 러너 PASS.
- C2: examples/README.md — 문법 커버리지 인덱스 (21개 예제, 미커버 항목 명시).
- 코퍼스 11→21 (2배). **전체 21/21 PASS, 회귀 0.**
- P9 인프라 강화: AI cold-start 학습용 검증 예제 집합.

## 2026-06-06 (/loop iter 3: P2 — 트랜스파일러 견고화)
- T1: while 루프 → Vais `L { I !(cond) { B } ... }`. e11_while PASS.
- T2: exclusive range `..` → `I i >= hi { B }` (inclusive `..=`와 분리). e12 PASS.
- T3: tests/transpiler_test.py — 입력 nl→기대 Vais 출력 단위테스트 19/19 (트랜스파일러 회귀 방지).
- T4: nested for 견고성 — e13_nested_for PASS (중첩 idx 카운터 충돌 없음).
- 값-정확성 23/23 + 트랜스파일러 단위 19/19 둘 다 green. 트랜스파일러 미지원 구문 0.

## 2026-06-06 (/loop iter 4: P3 — 에러 인프라 nl-check)
- E1: tools/nl-check.py — nl 소스에서 Rust직관 실수를 `help:`+수정코드로 잡음 (P4 핵심).
  잡는 것: `&&`→and, `||`→or, `!`→not, `as Type`→Type(x), `::`→`.`, turbofish→리터럴.
  안 잡는 것: 정상 nl(and/Color.Red/Int()/=> return) + 문자열·주석 내 패턴.
- 핵심: 트랜스파일러가 Rust식(`&&`,`as`)을 통과시켜 '두 길'을 열던 것을 nl-check가 차단 → 모호성0 강제.
- false positive 0 (예제 36 전부 clean). 단위테스트 tests/nl_check_test.py 11/11.
- 3개 게이트 green: 값-정확성 24/24 + 트랜스파일러 19/19 + nl-check 11/11.
- E2(cold-start self-correction 측정)는 신선한 에이전트 필요 → 다음 iter.

## 2026-06-06 (/loop iter 5: P3-E2 self-correction 측정 + P4 std 시작)
- E2: 버그 nl(`&&`/`Color::`/`as Int`) + nl-check help: 에러를 신선한 AI에게 → **1라운드에 3/3 수정**
  (and/Color.Small/Int(n)), 수정본 nl-check clean. **P4(자동수정 에러)=AI 1라운드 수렴 입증.**
- S1: std/PRELUDE.md — nl 표면 API → Vais 매핑 명세 (print/컬렉션/타입/Option·Result/연산자, 상태표).
- S2: nl `print(EXPR)` → Vais `puts(EXPR)` (보간 지원). e14_print: stdout "the answer is 42" + exit 0 PASS.
- 코퍼스 24→25. 3 게이트 green (값-정확성 25/25, 트랜스파일러 19/19, nl-check 11/11).

## 2026-06-06 (/loop iter 6: P5 — 레퍼런스 문서 → P0~P5 인프라 완료)
- D1: docs/reference/LANGUAGE.md — 사용자/AI용 nl 레퍼런스 (함수/변수/타입/조건/반복/enum/struct/
  컬렉션/클로저/Option·Result/출력/연산자 + 흔한실수표 + 빌드/검증). 모든 예제 검증됨.
- ROADMAP: P0~P5 전부 [x], 완료 배너 + L3 escalation 명시.
- **P0~P5 핵심 인프라 완료.** 3 게이트 green (값 25/25, 트랜스파일러 19/19, nl-check 11/11).
- **다음 = L3(자체 컴파일러)** — 컴파일러 작성언어/백엔드/에러전략은 사용자 결정 필요 → /loop escalate.

## 2026-06-06 (/loop iter 7: L3 self-host 착수 — lexer 첫 모듈)
- 사용자 결정: L3 컴파일러를 **nl 자체(self-host)**로. 부트스트랩=현재 트랜스파일러가 시드.
- L3.0: 부트스트랩 가능성 검증 — nl 렉서 조각(s[i]/while/and/비교)이 트랜스파일+실행 OK (digit 3 카운트).
- L3.1: compiler/self/lexer.nl — nl로 쓴 lexer 첫 모듈 (is_digit/alpha/space + 단어/숫자 스캔).
  검증: "fn add a b"=4단어, "x1 y22 z333"=3숫자 → 43. **nl로 컴파일러 코드 작성·실행 입증.**
- test.sh 확장: compiler/self/*.nl도 값-정확성 게이트에 편입. 전체 26/26 green.
- ROADMAP: L3 self-host 큐 (L3.0~L3.5) 추가.

## 2026-06-06 (/loop iter 8: L3.2 lexer 실제 토큰 emit + 버그 2건)
- **트랜스파일러 버그 수정**: `let mut x: List<T> = []`가 타입 유실 → E004. _list_binding 재작성:
  빈 []→`Vec::new()`(리터럴 []는 Vec<struct>서 crash), 비어있지않음→리터럴, 타입주석 보존. Vec<Token> 동작.
- **Vais 버그 발견+추적**: `&&`/`||` **비단락평가** (task_492f7e17) — `i<n && arr[i]`가 i==n서 crash(exit134).
  side-effect RHS가 LHS-false에도 실행됨 확정. 심각(가드접근 전부 깨짐). nl lexer는 nested-if로 우회.
- **L3.2**: compiler/self/lexer.nl — Token{kind,start,length} 구조체 + List<Token> emit하는 진짜 lexer.
  lex("x = 1 + 2")=5토큰(IDENT/PUNCT/NUM/PUNCT/NUM), lex("a b c d e f g")=7 검증. 전체 26/26 green.
