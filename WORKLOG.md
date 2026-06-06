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

## 2026-06-06 (/loop iter 9: L3.3 parser/eval + Vais Vec-passing 버그)
- L3.3: compiler/self/parser.nl — nl로 쓴 산술 파서+평가 (연산자 우선순위 * > +).
  1+2*3=7, 2+3*4=14, 5*2+1=11 검증. 재귀하향 패턴(상호재귀) nl 가능 확인.
- **Vais 버그 발견+추적** (task_54658a43): Vec를 sub/재귀 fn에 전달 불가 — by-value=E022 move,
  &Vec=clang fat-ptr 불일치. 재귀하향 파서(토큰 Vec를 parse_expr/term/factor에 threading) 막힘.
  → 단일함수 인덱스 우회. 근본은 Vais Vec ABI (task_7cfebeba 동류).
- 전체 27/27 green. self-host: lexer(L3.2) + parser(L3.3) 둘 다 nl로 동작.

## 2026-06-06 (/loop iter 10: L3.4 codegen — nl이 IR 생성)
- L3.4: compiler/self/codegen.nl — nl로 쓴 codegen. 산술식 평가 → LLVM IR 텍스트 출력.
  emit_ir(7) → "define i64 @main() { ret i64 7 }" → clang 빌드+실행 → exit 7. ★self-host codegen 동작★
- **트랜스파일러 버그 수정**: nl `{{`/`}}`(리터럴 brace 이스케이프)를 Vais puts가 언이스케이프 안 함
  → map_brace_escapes 추가 (문자열 리터럴 내 {{→{, }}→}). IR의 `{` `}` 정상 출력.
- scripts/test-codegen.sh: codegen이 생성한 IR이 valid+정확한지 전용 검증 (러너는 exit만, 이건 IR값).
- 전체 27→28 green + codegen IR 검증 PASS. **lexer+parser+codegen 3단계 모두 nl로 동작.**

## 2026-06-06 (/loop iter 11: L3.5 통합 미니 컴파일러 — self-host end-to-end)
- L3.5: compiler/self/compiler.nl — nl로 쓴 통합 컴파일러. 산술 소스문자열 → lex(바이트) →
  평가(우선순위 *>+, 좌결합, 멀티자릿수, +-*/) → LLVM IR 출력. clang 빌드+실행으로 검증.
  '1+2*3'→7, '2+3*4'→14, '10-2-3'→5, '20/2/5'→2, '12+34'→46, '100'→100 전부 IR생성+실행 정확.
- scripts/test-compiler.sh: 6식 end-to-end (소스→nl컴파일러→IR→실행→값) 검증.
- 전체 29/29 green + 트랜스파일러 19/19 + nl-check 11/11 + codegen IR + compiler e2e 6/6.
- **★self-host 마일스톤: nl로 쓴 컴파일러가 소스텍스트를 실행가능 IR로 변환★** (lex+parse+codegen 통합).
- L3 큐(L3.0~L3.5) 완료. 완료정의(L3+인프라) 도달 → 사용자 escalate.

## 2026-06-06 (/loop iter 12: CX1-3 컴파일러 확장 — 변수/문장/return)
- 사용자 우선순위: nl 컴파일러 확장(산술→전체 nl). ROADMAP에 CX 큐 추가.
- compiler.nl 확장: run_program(소스) — ';'구분 문장 시퀀스 처리.
  CX1 변수: `let <단일자> = <식>` 바인딩 + 참조 (26-슬롯 심볼테이블, index=letter-'a' — string-map Vais한계 회피).
  CX2 여러 문장: ';' 분리. CX3 return 문.
  eval_arith가 숫자+변수 혼합 식 평가 (우선순위 *>+, 좌결합, 멀티자릿수).
- 검증(소스→nl컴파일러→IR→실행): 'let a=2;let b=3;return a+b*4'=14, 'let a=5;let b=a*2;return b+1'=11,
  'let x=10;return x-3'=7, 'return 6*7'=42 등 7식. test-compiler.sh 갱신.
- 전체 29/29 + 컴파일러 e2e 7/7 green. **nl 컴파일러가 변수 있는 프로그램을 컴파일.**

## 2026-06-06 (/loop iter 13: CX4 조건식 if + 트랜스파일러 문자열-보호 버그 수정)
- **CX4** compiler.nl에 `if <식> <비교> <식> then <식> else <식>` 지원 (비교 `> < ==`).
  조건/then/else 모두 변수+산술 식 가능. eval_value 신설 (skip_spaces/starts_if/find_kw4 헬퍼).
- **Vais Vec-move 우회 (task_54658a43)**: 한 함수에서 `vars`(Vec)를 straight-line 2회 함수호출에
  넘기면 E022(flow-insensitive move). 실측: 루프 반복 호출은 OK. → 4 부분식(cond-lhs/rhs/then/else)의
  [start,end) 범위를 배열에 모아 **단일 루프**로 평가, `vars`를 정확히 1곳에서만 소비.
  비조건식도 같은 경로(범위 1개)로 통일해 두 번째 straight-line 사용 제거.
- **트랜스파일러 버그 수정 (근본)**: map_if/map_words가 **문자열 리터럴 내부**의 `if`/`and`까지
  재작성 → 임베디드 프로그램 텍스트 오염(compiler.nl의 테스트 입력 `"return if..."` → `"return I..."`,
  'I'=73≠'i'=105 → starts_if 실패 → 조건식 무시 → 오값). `outside_strings(line, fn)` 헬퍼 신설
  (문자열 리터럴 분리 후 비문자열부에만 적용), map_if/map_words 둘 다 적용.
- 검증: 컴파일러 e2e **15/15** (CX4 if 8케이스: >/</== + true/false분기 + 변수조합),
  트랜스파일러 단위 **22/22** (문자열-보호 3케이스 추가), 값-정확성 **29/29**. 회귀 0.
- **nl 컴파일러가 조건분기 있는 프로그램을 컴파일.** (산술→변수→문장→return→조건)

## 2026-06-06 (/loop iter 14: CX5 함수 정의+호출 — self-host 큰 관문 통과)
- **CX5** compiler/self/cx5_compiler.nl: `fn <f>(<p>) {{ return <식> }}` 정의 파싱 + 호출
  `<f>(<인자식>)` 디스패치. 다중 fn(최대 3) + **중첩 호출**(본문이 다른 fn 호출).
- **Vais Vec-move 한계(task_54658a43) 근본 우회**: 측정으로 발견 — Vec를 재귀/연속 함수호출에
  전달하면 E022(flow-insensitive move)지만 **struct는 재귀 전달 OK**(값-복사). 또 루프 반복 호출도 OK.
  → 평가 환경 `Env`(8슬롯 a~f,n,x)와 함수 테이블 `Defs`(3슬롯×4필드)를 **고정필드 struct**로 설계.
  산술식 평가기 상호재귀(eval_factor↔eval_term↔eval_expr, 위치+값을 `Cur` struct로 반환).
  호출 시 인자 평가 → param 바인딩한 fresh Env → 본문 범위 재귀 평가.
- **트랜스파일러 버그 수정(근본)**: Vais가 **모든** 문자열 리터럴의 `{ }` 쌍을 보간으로 처리
  (`"a { b } c"`→`b` 보간 시도 E002, 코드를 문자열로 임베드 불가). Vais 이스케이프 `\{`/`\}`는
  print+passed 양쪽 literal brace(런타임 1바이트 123/125)임을 실측 → map_brace_escapes를
  nl `{{`/`}}`→Vais `\{`/`\}`로 변경(기존 collapse `{{`→`{`는 lone brace만 우연히 동작).
  lone `{`/`}`와 `{value}` 보간은 그대로 → emit_ir(compiler.nl/codegen.nl) 무영향.
- 검증: CX5 e2e **6/6**(scripts/test-cx5.sh: 단일/다중/중첩호출/산술결합), 값-정확성 **30/30**,
  트랜스파일러 단위 22/22, compiler e2e(vars+if) green. 회귀 0.
- **nl 컴파일러가 사용자 정의 함수+호출이 있는 프로그램을 컴파일.** (산술→변수→조건→함수)
- 다음 CX6: 함수 본문 조건식 → 재귀 함수(struct 기반 이미 준비됨, 본문에 if 평가 추가).
