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

## 2026-06-06 (/loop iter 15: CX6 재귀 함수 — 조건식 본문)
- **CX6** cx5_compiler.nl에 `if <식> <비교> <식> then <식> else <식>`를 완전 표현식으로 추가
  (eval_value 진입점, find_kw4로 then/else 경계, 조건/분기는 eval_expr 재귀). 본문/return을
  eval_value 경유로 라우팅 → 함수 본문에 조건식 → **분기 내 재귀 호출**이 동작.
- **재귀 함수 실측**: factorial(5)=120, factorial(6)=720(208 mod256), **fibonacci(10)=55(트리 재귀)**,
  sum(1..10)=55, 비재귀 조건식 본문, 재귀+헬퍼 혼합. struct-Env+Defs 설계가 재귀를 move-safe하게 함
  (CX5의 핵심 통찰이 재귀에서 결실).
- 검증: CX5+CX6 e2e **11/11**(scripts/test-cx5.sh), 값-정확성 **30/30**(cx5_compiler.nl=재귀 factorial),
  트랜스파일러 22/22, compiler e2e green. 회귀 0.
- **nl 컴파일러가 재귀 함수(factorial/fibonacci/sum) 프로그램을 컴파일.**
  self-host 표현력: 산술→변수→조건→함수정의→호출→중첩호출→**재귀**. 인터프리터 핵심 달성.
- 다음 CX7: 다중 인자 / 지역 변수(현재 단일 param).

## 2026-06-06 (/loop iter 16: CX7 다중 인자 함수)
- **CX7** cx5_compiler.nl을 1~2 인자 함수로 확장. Defs에 param2 필드 추가(슬롯당 name+param+param2+bs+be),
  def-parser가 param-list 콤마 파싱(`(a, b)`), 호출부가 arg-list 콤마 파싱(`m(3, 4)`) 후 양 param 바인딩
  (eset 2회). 단일 인자는 param2=0/has2=0으로 하위호환.
- **실측**: m(a,b)=a+b/a*b, **2-인자 재귀 power** p(b,e)=if e<1 then 1 else b*p(b,e-1) → 3^4=81/2^8=256,
  max(a,b) (양 param 조건), 인자식 m(1+2,3*4)=15, 2-인자+재귀 혼합 a(3,4)+f(5)=127.
- 검증: CX5-7 e2e **18/18**(scripts/test-cx5.sh), 값-정확성 **30/30**, 트랜스파일러 22/22, compiler e2e green. 회귀 0.
- **nl 컴파일러가 다중 인자 함수(2-인자 재귀 포함) 프로그램을 컴파일.**
- 다음 CX8: 함수 본문 지역 변수(let) — 본문을 단일 return식에서 ;-문장열로 확장.

## 2026-06-06 (/loop iter 17: CX8 지역 변수 (let))
- **CX8** cx5_compiler.nl에 함수 본문 지역변수. `eval_body(src,bs,be,env,defs)` 신설: ;-문장열을
  순회하며 `let <v> = <e>`→env=eset(env,v,평가값), `return <e>`/bare-expr→결과값. env는 struct라
  루프 갱신+재귀호출 양쪽 move-safe.
- def-parser 수정: 본문 범위 bs를 "return 다음"(inner+6)에서 **`{` 직후(br+1)**로 — eval_body가
  return/let을 자체 파싱하므로 본문 전체가 범위. 단일 return 본문도 eval_body가 처리(회귀 0).
- run_program도 **top-level let**(tenv) 지원 → cx5_compiler가 CX1-3 compiler.nl 상위집합.
- 실측: 의존 지역변수(let b = a*2), 2-arg+local, **local→재귀호출 인자**(g(6):c=5,f(5)=120),
  bare-expr 본문(d(x)=x*3), **top-level var→fn인자**(let a=3; d(a)+a=9).
- 검증: CX5-8 e2e **25/25**(scripts/test-cx5.sh), 값-정확성 **30/30**, 트랜스파일러 22/22, compiler e2e green. 회귀 0.
- **nl 컴파일러가 지역변수+다중인자+재귀 함수 프로그램을 컴파일.** 인터프리터 표현력 거의 완비.
- 다음 CX9: Env/Defs 슬롯 확장(기계적) — 더 많은 변수/함수 동시 사용.

## 2026-06-06 (/loop iter 18: CX9 26-슬롯 Env (a-z 변수))
- **CX9** Env를 8슬롯(a-f,n,x)→**26슬롯(a-z)**로 확장. 임의 단일자 변수/param/local 지원.
- **eset 압축 핵심 발견**: 26슬롯을 rebuild-all 방식이면 26분기×26필드=676줄(비현실적). 실측으로
  **struct in-place 필드 변경(`let mut e = env; if ch==.. {{ e.X = v }}`)이 재귀에서 안전**함을 확인
  → eset 26 one-line 분기로 압축. eget도 26분기(단순). 코드 생성으로 손오류 방지.
- Vais Vec-recursion 재확인: 여전히 E022(Vec)/clang fat-ptr(&Vec) — struct 우회 계속 필요.
- 실측: 변수 t/r/s/z/w(구 8슬롯 밖), 3개 distinct fn(p/q/u), high-letter top-level var.
- 검증: CX5-9 e2e **29/29**, 값-정확성 **30/30**, 트랜스파일러 22/22, compiler e2e green. 회귀 0.
- **완료정의 충족**: P0-5 + L3 + CX1-9 done. 코퍼스37 + nl-check + PRELUDE + 게이트3종.
  이후는 인터프리터 확장 또는 진짜 fixpoint(전체문법+codegen=큰 단계, 사용자 escalate).

## 2026-06-06 (/loop iter 19: 플래그십 데모 + 멀티문자 한계 추적 + 완료정의 충족)
- **플래그십 e2e 추가**: 한 프로그램에 모든 기능 — 재귀(f=fib)+지역변수(s의 t)+2인자(a)+top-level
  var(b)+중첩호출-as-인자. f(10)=55, s(3)=9, a(55,9)=64, +b=67. **실측 67**. 모든 CX 기능 조합 동작 증명.
- **멀티문자 식별자 한계 발견·추적**: 플래그십 초안이 fib/sq/add(다중자)였는데 0 반환 — Env가 26 단일바이트
  슬롯이라 다중자 이름 미지원. 근본=name→slot 매핑 필요→문자열키 심볼테이블→Vec 재귀=Vais E022 재충돌.
  단일자가 honest scope. ROADMAP TRACKED 등록(심볼테이블 재설계=큰 단계).
- **완료정의 충족 선언**: P0-5 + L3(self-host 미니컴파일러) + CX1-9 done. 코퍼스37 + nl-check 에러인프라
  + PRELUDE std시작 + 게이트3종(값정확성30 / 트랜스파일러22 / nl-check11) + CX e2e30.
- **남은 프론티어 = 사용자 결정 필요한 큰 단계**: (a) 멀티문자 식별자(심볼테이블 재설계) (b) 진짜
  fixpoint(전체 nl 문법 파싱+실제 codegen=L3 엔드게임, 백엔드 전략 결정). /loop escalate 지점.

## 2026-06-06 (/loop iter 20: Vais &Vec borrow 버그 근본 수정 → nl Vec 재귀 가능 [fixpoint 기반])
- 사용자 결정: "진짜 fixpoint 도전". 경로 추적 → AST 순회에 Vec borrow 재귀 필수 → Vais `&Vec` codegen 버그 발견.
- **근본 원인**: `&v`(Vec 지역변수, 배열리터럴 초기화 → array_length 설정됨)가 **항상** 슬라이스 fat-ptr
  `{i8*,i64}`로 codegen → 호출 대상이 `&Vec<T>`(=`%Vec*`)일 때 clang "{ptr,i64} but expected ptr". Vec를
  참조로 못 넘김 → Vec 재귀(인터프리터/트리워커) 불가. (Vais Vec=move-by-value라 borrow가 정답인데 막힘.)
- **수정**(compiler 214c97cf): call-arg codegen이 expected param 타입을 expected_expr_types에 push(&expr arg
  한정), generate_ref_expr이 대상이 `&Vec<...>`이면 슬라이스 path 억제하고 주소 전달. `&Vec→&[T]` coercion은
  보존(대상이 슬라이스면 guard 미발동). ref_deref.rs + generate_expr_call.rs.
- **검증**: &Vec 2회(30)+&Vec 재귀(10)+&Vec→&[T](10) 실측, check-integrity.sh INTEGRITY OK(baseline 대비 회귀
  0), phase166/190/255 Vec codegen e2e green. 가드 phase256(text-IR harness Vec alloca 불가→ignore+build검증).
- **nl 결실**: nl이 `&List<Int>` borrow로 Vec 재귀 가능(transpiler가 &List→&Vec 매핑). e15_list_recursion
  예제 추가(sum_from 재귀 10). 코퍼스 31/31, 트랜스파일러 24/24. **fixpoint(AST 순회)의 핵심 기반 확보.**
- 다음: fixpoint 본격 — nl 토큰리스트/AST를 List로 표현하고 재귀 평가 → 전체 nl 문법 파싱 + codegen.

## 2026-06-06 (/loop iter 21: FP1 — List<Token> 토크나이저 파이프라인 [fixpoint 시작])
- 사용자 결정 "진짜 fixpoint" 본격 착수. **compiler/self/fixpoint.nl** 신설: cx5_compiler(단일-문자열-스캔)와
  달리 진짜 파이프라인 — source → tokenize → **List<Token>** → 재귀 평가(&List borrow) → LLVM IR.
- **List<Token> 토크나이저**: Token{kind,value} struct. 멀티자릿수 숫자(digit run→value), 공백 skip,
  +/-/* 토큰. tokenize(src)→List<Token> 반환.
- **재귀 평가기**(&List<Token> borrow): eval_term(*-fold, precedence) + eval_expr/eval_expr_fold.
  처음 우결합 버그(10-2-3=11) → **left-fold로 좌결합 수정**(eval_expr_fold가 acc에 op 적용하며 재귀).
- Vais &Vec borrow 재귀 수정(214c97cf)이 List<Token> 재귀 소비를 가능케 함 — fixpoint 핵심 기반.
- **실측**: 12+3*4=24, 2+3*4-1=13(precedence), 10-2-3=5/100-50-25-10=15(좌결합), 멀티자릿수 100.
- 검증: fixpoint e2e **10/10**(test-fixpoint.sh), 값-정확성 **32/32**(fixpoint.nl 편입), 트랜스파일러 24/24. 회귀 0.
- **nl이 진짜 토크나이저→List<Token>→재귀평가 파이프라인으로 산술 컴파일.** cx5의 string-rescan 졸업.
- 다음 FP2: 변수/함수를 List 파이프라인으로 재구현 → FP4 멀티문자 식별자(토큰이 name 보유).

## 2026-06-06 (/loop iter 22: FP2+FP4 — 멀티문자 식별자 [추적 한계 해소])
- **compiler/self/fixpoint2.nl** 신설: List<Token> 파이프라인으로 **멀티문자 식별자** 지원 → 이전에
  TRACKED했던 "Env=26 단일바이트슬롯" 한계 근본 해소.
- **메커니즘**: Token이 식별자 이름을 소스범위(nstart,nlen)로 보유. 심볼테이블=List<Var>{nstart,nlen,value},
  조회는 name_eq(소스 바이트 비교, 길이 먼저 체크→접두사공유 foo/food 구분). keyword(let/return) vs ident는
  word_is로 토큰화 시 구분. eval은 &List<Token>+&List<Var> 재귀(precedence/좌결합 fixpoint.nl과 동일).
- **프리미티브 단계검증**: 멀티문자 토큰화(3토큰)/name_eq(foo==foo,foo!=bar)/List<Var>조회(30)/성장+조회(50)
  → 통합 fixpoint2.nl.
- **실측**: `let total=10; let count=total*4; return total+count`=50, width*height=20, base→doubled→result=30,
  foo(1)/food(100) 구분, n-5-3=12(좌결합). 진짜 변수명 동작.
- 검증: fixpoint2 e2e **9/9**, 값-정확성 **33/33**, 트랜스파일러 24/24, fixpoint/CX 회귀 0.
- **nl이 멀티문자 변수명을 가진 프로그램을 컴파일.** List<Token> 아키텍처가 단일자 한계 돌파(Vais &Vec 수정 기반).
- 다음 FP3: 함수 정의(멀티문자 함수명+호출)를 List 파이프라인으로.

## 2026-06-06 (/loop iter 23: FP3 — 멀티문자 함수 정의+호출)
- **compiler/self/fixpoint3.nl**: fixpoint2(멀티문자 변수) + 함수테이블. `fn name(p[, q]) {{ return e }}`
  멀티문자 함수명 + 호출.
- **메커니즘**: build_fns가 토큰 스캔해 List<Fn>{name범위, param1/2 범위, body 토큰범위[bstart,bend)} 구축.
  eval_factor서 ident 다음 '('이면 호출 → find_fn(name_eq) → 새 callee vars(param=arg push) → body 토큰범위 eval.
  평가기가 &List<Token>+&List<Fn>+&List<Var> **3중 borrow 재귀**(Vais &Vec 수정으로 가능, 사전 검증함).
  skip_factor/arg_end가 호출의 괄호 깊이 추적(중첩 호출 인자 경계).
- **실측**: square(5)=25, add(10,20)=30, 중첩 add(square(base),base)=12, 플래그십 add(sq(width),sq(height))=41,
  cross-call add2(y)=inc(inc(y))=12 / g(b)=f(b)+b=15.
- 검증: fixpoint3 e2e **7/7**, 값-정확성 **34/34**, 트랜스파일러 24/24, fixpoint/fixpoint2 회귀 0.
- **nl이 멀티문자 함수명+중첩호출 프로그램을 컴파일.** 플래그십(square/add/width/height)이 진짜 이름으로 동작.
- 다음 FP3b: 함수 본문 조건식 → 멀티문자 재귀(진짜 fib/fact).

## 2026-06-06 (/loop iter 24: FP3b — 멀티문자 재귀 함수 [조건식 본문])
- fixpoint3에 if/then/else 추가 → **멀티문자 재귀 함수** 완성. 토큰 if(15)/then(16)/else(17) +
  비교 <(18)/>(19)/==(20, `=`2개를 lookahead로 구분). eval_value 신설(find_kind로 then/else 경계,
  조건/분기는 eval_value/eval_expr 재귀). body eval + top-level return을 eval_value 경유.
- **실측**: factorial(5)=120, **fib(10)=55(멀티문자 트리재귀)**, sumto(10)=55, clamp(조건식 본문),
  재귀+변수+cross-fn add(fact(4),base)=28.
- 검증: fixpoint3 e2e **13/13**, 값-정확성 **34/34**, 트랜스파일러 24/24, fixpoint2 회귀 0.
- **nl fixpoint 컴파일러가 멀티문자 재귀 함수(factorial/fib) 프로그램을 컴파일.** cx5_compiler(단일자,
  string-rescan)를 모든 면에서 능가: List<Token> 파이프라인 + 멀티문자 변수/함수 + 재귀 + 조건 + precedence/좌결합.
- 남은 것: FP5(전체 nl 문법 파싱 + 실제 codegen 확장) = 진짜 self-compile, 큰 단계.

## 2026-06-06 (/loop iter 25: FP5 — 진짜 codegen + Vais % 버그 수정)
- **목표 전환**: 인터프리터(값 미리계산→`ret <const>`)에서 **진짜 codegen**(런타임 계산 IR emit)으로.
- **Vais % 버그 발견·수정(근본, compiler e711dac1)**: codegen 빌드 중 발견 — 문자열 보간이 printf로
  lowering되는데 **literal `%`가 `%%`로 이스케이프 안 됨** → printf가 `%tN`을 format specifier로 소비
  (`puts("  %t1 = mul {a}")` → `  = mul 2`, `%t1 ` 손실). LLVM IR(`%` 가득)을 nl로 emit 불가케 함.
  print_format.rs에서 literal `%`→`%%` push. check-integrity OK 회귀0. 가드 phase257(3 stdout assert).
  (격리 핵심: e2 `%t1 = x {a}`→`= x 2` 최소재현, &Vec<struct>+보간+% prefix 조합서 발현.)
- **compiler/self/fixpoint_codegen.nl**: gen_factor/term/expr가 Op{kind(0lit/1temp),val,next} 반환,
  emit_binop 4분기(operand literal/temp 조합)로 `%tN = op i64 a, b` emit. 좌결합 fold.
  `12+3*4` → `%t1=mul 3,4; %t2=add 12,%t1; ret %t2` → 런타임 24.
- 검증: codegen e2e **10/10**(+"mul i64 emit" 검증=상수폴딩 아닌 진짜 codegen 증명), 값-정확성 **35/35**,
  트랜스파일러 24/24. 회귀 0.
- **nl이 런타임 계산 LLVM IR을 생성.** 컴파일러의 codegen 절반 달성(인터프리터→진짜 코드생성 질적 도약).
- Vais 수정 2건 누적(214c97cf &Vec, e711dac1 %). 다음 FP6: 변수/함수 codegen(alloca/store/define).

## 2026-06-06 (/loop iter 26: FP6 — 변수 codegen)
- **compiler/self/fixpoint_codegen2.nl**: fixpoint_codegen(산술 런타임 IR) + 멀티문자 변수(fixpoint2).
  `let <name> = <expr>; ... return <expr>`를 런타임 계산 IR로 codegen.
- **SSA 변수 모델**: 변수 = 그 식이 만든 operand(literal/temp) 매핑 → 불변 바인딩은 alloca 불필요.
  심볼테이블 List<SymOp>{nstart,nlen,kind,val}, gen_factor가 ident를 name_eq로 조회해 operand 반환.
- **실측**: `let x=5; let y=x*2; return y+1` → `%t1=mul 5,2; %t2=add %t1,1; ret %t2` → 런타임 11.
  멀티문자/의존체인(base→d→r=30)/변수재사용(z*z=36).
- 검증: codegen2 e2e **8/8**, 값-정확성 **36/36**, 트랜스파일러 24/24. 회귀 0.
- **nl이 변수 있는 프로그램을 런타임 계산 IR로 codegen.** codegen이 산술→변수로 확장.
- 다음 FP7: 함수 codegen(define/call IR).

## 2026-06-06 (/loop iter 27: FP7 — 함수 codegen [define/call])
- **compiler/self/fixpoint_codegen3.nl**: 함수를 진짜 LLVM IR로 codegen. `fn name(p) {{ return e }}` →
  `define i64 @name(i64 %p) {{ ... }}`, 호출 → `call i64 @name(i64 arg)`. 멀티문자 이름을 LLVM 식별자로
  emit(emit_name=putchar 바이트별 소스출력). Op에 named(kind 2=%<param>) 추가, print_int_inline/emit_str로
  IR 수동 조립(% 출력 위해 putchar(37) 사용).
- **실측**: `fn double(x){{return x*2}}; return double(21)` →
  `define i64 @double(i64 %x){{%t1=mul %x,2; ret %t1}} define i64 @main(){{%t1=call @double(21); ret %t1}}` → 런타임 42.
  param 산술본문(x*3-1)/인자식(double(10+11))/호출결과 산술(double(20)+2).
- **트랜스파일러 한계 회피**: 인라인 let/while(`{ let x=..; ... }` 한 줄)는 라인기반 트랜스파일러가 미인식
  (E002 'let' undefined) → 멀티라인으로 분리.
- 검증: codegen3 e2e **7/7**(+"define+call emit" 증명), 값-정확성 **37/37**, 트랜스파일러 24/24. 회귀 0.
- **nl이 함수 정의+호출을 진짜 다중-함수 LLVM IR로 codegen.** 컴파일러 codegen의 핵심(함수) 달성.
- 다음 FP8: 조건/분기 codegen(br/phi) → 재귀 함수 codegen.

## 2026-06-06 (/loop iter 28: FP8 — 재귀 함수 codegen [icmp/br/phi] = codegen 캡스톤)
- **compiler/self/fixpoint_codegen4.nl**: 조건/분기를 제어흐름 IR로 codegen → **재귀 함수를 네이티브 코드로 생성**.
- **메커니즘**: 본문 `return if <식> <비교> <식> then <식> else <식>`를 icmp(slt/sgt/eq) + 조건 br +
  labeled blocks(then<N>/else<N>/merge<N>) + phi로 emit. gen_body가 if 감지(토큰 15), find_kind로 then/else/비교
  경계, 분기 각각 gen_body 재귀 codegen. **분기 내 재귀 call은 그 블록(else)에만 emit** → 재귀 정확.
  label/temp 번호 통합 threading.
- **실측**: factorial(5)=120, **fib(10)=55(멀티문자 트리재귀)**, sumto(10)=55, clamp(>), iszero(==).
  생성 IR이 icmp slt + phi i64 + `call @factorial` 포함 — 진짜 제어흐름 codegen.
- 검증: codegen4 e2e **8/8**(+제어흐름 emit 증명), 값-정확성 **38/38**, 트랜스파일러 24/24. 회귀 0.
- **🎯 CODEGEN 트랙 완성**: nl 컴파일러가 산술→변수→함수→재귀(제어흐름)를 진짜 LLVM IR로 생성.
  컴파일러 codegen의 핵심 전부 달성. fixpoint_codegen~4 (산술/변수/함수/재귀).
- 정직한 한계: 완전 self-compile=전체 nl 문법(struct/while/Vec/method/&)+codegen 재구현=수천줄. 현재는
  산술/함수 부분집합의 완전한 컴파일러(파싱+제어흐름+함수+재귀 codegen).

## 2026-06-06 (/loop iter 29: SELF_HOST.md 문서화 + 자율 아크 완료 평가)
- **compiler/self/SELF_HOST.md** 신설: 두 트랙(인터프리터 cx5_compiler~fixpoint3, codegen fixpoint_codegen~4)
  + 각 모듈이 컴파일하는 것 + 검증 스크립트 표 + 핵심 설계노트(struct-env 재귀/&List borrow/멀티문자 name_eq/
  brace 이스케이프/LLVM 식별자 emit) + 정직한 한계(산술/함수/재귀 부분집합의 완전한 컴파일러; 완전 self-compile은
  전체 문법+codegen=months급).
- **FP9 평가**: codegen 다중인자/지역변수는 전 gen_* 시그니처를 심볼테이블로 리팩터(큰 작업) 필요, 증분 가치.
  트랜스파일러 인라인 statement 한계(while/let after {)도 재확인 — 깊은 변경 필요, 실용 우회(멀티라인) 존재.
  → codegen 캡스톤(FP8 재귀) 도달로 자율 증분 아크 자연 완료점. 남은 것(FP9 대형 리팩터/완전 self-compile)은
  증분-고비용 또는 months급 → 사용자 보고/방향 확인이 적절.
- 게이트 전부 green(값정확성38 트랜스파일러24 nl-check11, 9 e2e 스위트). 회귀 0.

## 2026-06-06 (/loop iter 30: FP10a — 가변변수 codegen [완전 self-compile 도전 시작])
- 사용자 결정 "완전 self-compile 본격 도전". 목표=nl이 자기 컴파일러 구문(가변변수/while/List/method/&)을 컴파일.
  점진 확장 시작 — 명령형 codegen.
- **compiler/self/fixpoint_imperative.nl**: `let mut`/assignment/return을 **alloca/store/load** IR로 codegen.
  SSA-operand 모델(fixpoint_codegen*)로 불가능했던 **변이** 가능. collect_slots 1패스로 변수마다 alloca 슬롯 할당,
  변수 참조=load→temp, 대입=store. List<Slot>{name범위, slot번호}.
- **실측**: `let mut s=10; s=s+5; s=s*2; return s`→`alloca/store/load` IR→런타임 30. 다중변수(a,b 상호)/누적(acc).
- 검증: imperative e2e **6/6**(+alloca/store/load emit 증명), 값-정확성 **39/39**, 트랜스파일러 24/24. 회귀 0.
- **nl이 가변변수 명령형 프로그램을 alloca 기반 IR로 codegen.** 루프(FP10b)의 기반.
- 다음 FP10b: while 루프 codegen(loop/body/done 블록 + 조건 br + back-edge).

## 2026-06-06 (/loop iter 31: FP10b — while 루프 codegen [반복 명령형])
- fixpoint_imperative.nl에 **while 루프 codegen** 추가. `while <식> <비교> <식> {{ <문장들> }}`를
  loop<N>/body<N>/done<N> 블록 + 조건 icmp+조건br + back-edge(br label %loop<N>)로 emit.
- **gen_stmts 추출+재귀**: compile의 인라인 문장루프를 gen_stmts(start,end,counter)→counter로 추출, 루프 본문도
  같은 함수로 재귀 처리. match_brace로 중첩 `{}` 매칭. label은 temp 번호 기반 고유(순차 루프도 충돌 0).
  비교 < > == (slt/sgt/eq). 토큰 while=22, {=11, }=12, 비교 18/19/20 추가(==는 = 2개 lookahead).
- **실측**: sum(1..5)=15, **5!=120(루프 factorial)**, countdown(10), 0회실행(조건 false→s유지), 두 순차루프(10).
  생성 IR이 loop1: + br label %loop1(back-edge) + icmp slt 포함.
- 검증: imperative e2e **13/13**(+루프 제어흐름 emit 증명), 값-정확성 **39/39**, 트랜스파일러 24/24. 회귀 0.
- **nl이 반복 명령형 프로그램(가변변수+while)을 네이티브 IR로 codegen.** nl 컴파일러 자신이 쓰인 구문에 근접.
- 다음 FP10c: if 문(제어흐름) codegen. 이후 List/method/struct(대규모).

## 2026-06-06 (/loop iter 32: FP10c — if/else 문 codegen [명령형 3종 완성])
- fixpoint_imperative.nl에 **if/else 문 codegen**(값 아닌 제어흐름). `if <식> <비교> <식> {{ <문장> }}
  [else {{ <문장> }}]`를 icmp+조건br + ithen<N>/ielse<N>/imerge<N> 블록으로 emit. else 옵션 감지(없으면 빈
  else→merge). gen_stmts 재귀(분기도 문장열), match_brace 중첩 처리. 토큰 if=15/else=17 추가, kw2/kw4 추가.
- **실측**: true/false 분기(10/20), else 없음(99/유지5), **루프 안 if**(1..10 중 >5 카운트=5).
- **트랜스파일러 한계 재확인+우회**: 코드 주석의 lone `{`가 for/while 블록확장 brace-counting을 깨 stray `}`
  생성(P001) → 주석을 brace-free 문구로(`'{'`→`open-brace`). (라인기반 트랜스파일러의 알려진 클래스.)
- 검증: imperative e2e **17 PASS**(FP10a 가변+FP10b while+FP10c if), 값-정확성 **39/39**, 트랜스파일러 24/24. 회귀 0.
- **명령형 3종(가변변수+while+if) 완성.** nl 컴파일러가 자신이 쓰인 제어구조(가변/루프/조건)를 네이티브 IR로 codegen.
- 다음 FP10d: List/method/struct codegen(대규모 months급) — 진짜 self-compile 향.

## 2026-06-06 (/loop iter 33: FP10d — 배열 codegen [데이터구조 시작])
- 사용자 "완전 self-compile" 진행 중. 대형 함수-병합 대신 **배열 codegen**부터(자족적, List 기반).
- **compiler/self/fixpoint_array.nl**: 고정크기 정수 배열 codegen. `let a = [v0,v1,..]`→`alloca [N x i64]`+
  요소별 GEP store, `a[식]`→GEP+load(**런타임 인덱스**), `a[식] = 식`→GEP+store. Slot에 is_arr/alen,
  collect_slots가 배열 길이(콤마+1) 계산. 토큰 [=23 ]=24 ,=25 추가. expr_end가 비교/=/]/}/;에서 정지.
- **실측**: a[1]+a[2]=50, a[0]=100 대입, 런타임 a[i]=25, **루프 배열합산=100**, **루프 배열쓰기 a[i]=i*10→30**,
  5원소 합=15. 생성 IR이 alloca [N x i64] + getelementptr 포함.
- 검증: array e2e **7 PASS**(+배열 codegen 증명), 값-정확성 **40/40**, 트랜스파일러 24/24. 회귀 0.
- **nl이 배열(루프 read/write 포함) 프로그램을 네이티브 IR로 codegen.** List 자료구조 codegen의 기반.
- 다음 FP10e: struct/동적 List(push/len) codegen — months급 큰 조각. (대형 함수-병합 FP10f도 추적.)

## 2026-06-06 (/loop iter 34: FP10f — 함수+명령형본문 통합 [self-compile 직결 캡스톤])
- **compiler/self/fixpoint_full.nl**: 함수(FP7)와 명령형(FP10)을 통합 — nl 컴파일러 자신의 함수 형태
  (`fn name(p) {{ let mut...; while...; if...; return }}`)를 진짜 LLVM IR로 codegen.
- **메커니즘**: 각 함수=`define i64 @name(i64 %p_in)`, param→alloca %v0 복사(이후 균일 load/store),
  body locals alloca(슬롯 1+), 명령형 본문은 gen_stmts(let/assign/while/if/return) 재사용, 호출=`call i64 @name`.
  gen_factor/term/expr/fold/stmts 시그니처에 **&List<Fn> threading**(스크립트로 정의+호출부 일괄). build_fns
  (중첩 brace depth 추적) + per-function slot scope(emit_fn) + gen_top(top-level만, skip_fn_def로 fn 건너뜀).
  skip_factor/gen_term이 call(name(...)) 토큰을 paren_end로 스킵.
- **실측**: 루프함수 sum_to(6)=15, 루프 fact(6)=120, 함수 안 if clamp(250/7), **재귀 fac(5)=120(early-return base)**,
  **cross-call quad(5)=20**(가변 local + 두 함수). 생성 IR이 define @name(%p_in)+param-alloca+loop+call 포함.
- 검증: full e2e **7 PASS**, 값-정확성 **41/41**, 트랜스파일러 24/24. 회귀 0.
- **🎯 nl 컴파일러가 자신이 쓰인 함수 형태(함수+param+가변locals+while+if+재귀+호출)를 네이티브 IR로 codegen.**
  진짜 self-compile의 핵심 통합 달성. 남은 것=struct/동적 List(FP10e, months급).

## 2026-06-06 (/loop iter 35: FP10e — struct codegen [레코드])
- **compiler/self/fixpoint_struct.nl**: struct codegen. struct=고정 [N x i64], 필드명→인덱스(배열 GEP 재사용).
  `struct Name {{ f0,f1,.. }}` 선언(build_defs, StructDef에 ≤6 필드 위치+count) + `Name {{ f: v,.. }}` 리터럴
  (필드별 GEP store) + `p.field` read(field_index→GEP+load) + `p.field = 식` write. Slot에 ty 필드(struct-type
  인덱스 또는 -1 scalar). 토큰 struct=26 .=27 추가. 필드 `:`는 토크나이저가 drop(값이 필드명 바로 뒤).
- **실측**: p.x+p.y=7, 필드대입 p.x=100→104, **3필드 Tok{{kind,start,len}}=9**(컴파일러의 Token 형태!),
  b.w*b.h=20(area), 식 필드값 P{{a:2+3,b:4*5}}=25.
- 검증: struct e2e **6 PASS**, 값-정확성 **42/42**, 트랜스파일러 24/24. 회귀 0.
- **🎯 struct=nl 컴파일러 자신의 레코드(Token/Op/Fn/Slot) 형태를 codegen.** self-compile의 거의 마지막 조각.
- 다음 FP10g: 동적 List(push/len) codegen — 힙/가변버퍼, months급 최후 조각.

## 2026-06-06 (/loop iter 36: FP10g — 동적 List codegen [데이터구조 완비])
- **compiler/self/fixpoint_list.nl**: 동적 List codegen — self-compile 데이터구조의 최후 조각.
  List=고정용량 버퍼(alloca [64 x i64]) + 길이 카운터(alloca i64). `let lst = list()`(2슬롯, len=0 초기화),
  `lst.push(식)`(load len→GEP buf[len]→store→len++), `lst.len`(load 카운터), `lst[식]`(GEP+load),
  `lst[식]=식`(GEP+store). Slot에 kind(0 scalar/1 list). 토큰 [=23 ]=24 (=9 )=10 .=27 push.
- **실측**: push 3개+len+xs[1]=23, **push 루프(i*10) + xs.len 루프 합산=100**(컴파일러 tokenizer가 List<Token>
  빌드하고 evaluator가 소비하는 정확한 패턴!), len 추적=2, 원소 대입 xs[0]=50→56.
- 검증: list e2e **5 PASS**, 값-정확성 **43/43**, 트랜스파일러 24/24. 회귀 0.
- **🎯🎯 codegen 데이터구조 완비**: 배열+struct(레코드)+동적 List(push/len). nl 컴파일러가 자신이 쓰는 모든
  핵심 구문(산술/변수/함수/재귀/가변/while/if/함수+명령형본문/배열/struct/List)을 네이티브 IR로 생성.
  진짜 self-compile의 codegen 토대 완성. 남은 것=전체 통합 단일 컴파일러+실제 nl 소스 입력=통합/스케일(months급).

## 2026-06-06 (/loop iter 37: FP11a — 통합 시작, 함수+명령형+배열 한 컴파일러)
- 사용자 결정 "통합 계속". fixpoint_full(함수+명령형본문)에 **배열 통합** — 첫 통합 마일스톤.
- Slot에 is_arr/alen 추가(4 site 갱신), 배열 토큰 [=23 ]=24 ,=25, bracket_end/arrlen_of/count_arr_elems/
  arr_elem_end 헬퍼, gen_factor에 배열 인덱스 read(GEP+load), gen_stmts에 배열 리터럴 store + 인덱스 write,
  skip_factor에 배열 스킵, collect_top_slots/add_local_slots에 배열 길이 감지. 함수-스코프 슬롯에 통합.
- **실측**: **함수 안 배열+루프 합 sumarr(3)=60**, 배열 쓰기 build(a[i]=i*5)=15, 배열+if pick=9.
  회귀: 함수+명령형(sum_to=15)/재귀(fac=120) 그대로.
- 검증: fixpoint-full e2e **10 PASS**(통합 3 추가), 값-정확성 **43/43**, 트랜스파일러 24/24. 회귀 0.
- **함수 + 가변변수 + while + if + 배열이 한 컴파일러에서 합성.** 통합의 첫 증거. 다음 FP11b: struct+List 통합.

## 2026-06-06 (/loop iter 38: FP11b — fixpoint_full에 동적 List 통합)
- fixpoint_full(함수+명령형+배열)에 **동적 List 통합** — 더 깊은 통합 마일스톤.
- List=is_arr=2(버퍼 [64 x i64] @slot + 길이 @slot+1, **2슬롯** 소비). rhs_is_list 헬퍼로 `let lst=list()` 감지,
  슬롯수집기(add_local_slots/collect_top_slots)가 2슬롯 할당+len=0 초기화. gen_factor에 `lst.len`(load slot+1)
  +`lst[식]`(기존 배열 인덱스 재사용, alen=64), gen_stmts에 list() skip + `lst.push(식)`(load len→GEP buf[len]→
  store→len++) + `lst[식]=식`(기존 배열 쓰기). skip_factor에 `.` 3토큰 스킵. 토큰 .=27 추가.
- **실측**: **함수가 List 빌드+소비(push 루프+xs.len 합산) build(5)=100**(nl 컴파일러 tokenizer가 List<Token>
  빌드/소비하는 정확한 패턴!), List len cnt(7)=7, **함수안 배열+List 혼합 mix=105**. 회귀0(sumarr=60, fac=120).
- 검증: fixpoint-full e2e **13 PASS**(List 통합 3 추가), 값-정확성 **43/43**, 트랜스파일러 24/24. 회귀 0.
- **함수+가변변수+while+if+배열+동적List가 한 컴파일러에서 합성.** nl 컴파일러 자신의 코드 구조와 일치.
  다음 FP11c: struct 통합(마지막 데이터구조), 이후 실제 nl 소스 입력=months급.

## 2026-06-06 (/loop iter 39: FP11c struct 통합 평가 — 코어 통합 완료점)
- FP11c(struct를 fixpoint_full에 통합) 착수: Slot에 sty 추가(회귀0 확인)했으나 struct 통합의 핵심 난점
  = **`.` 모호성**(이미 lst.len/lst.push가 `.` 사용 — struct p.field와 충돌, gen_factor/skip_factor/gen_stmts
  3사이트서 slot kind로 분기 필요) + struct-type 테이블 = 가장 intricate한 merge. 절반-완성 위험 회피 위해
  sty 그라운드워크 되돌림(fixpoint_full = FP11b 검증 상태 유지, build(5)=100 회귀0).
- **코어 통합 완료 판단**: fixpoint_full이 함수+가변변수+while+if+배열+동적List를 합성 = **nl 컴파일러 자신의
  주 구조**(함수+루프로 List<Token> 빌드/소비)를 통합 컴파일. struct는 레코드(부차적, fixpoint_struct에 단독
  검증됨)이고 통합은 `.` 모호성으로 intricate → 별도 집중 세션이 적합.
- 남은 것: (a) struct를 full에 통합(intricate, `.` 분기) (b) 실제 nl 컴파일러 소스 입력=months급. 둘 다
  잘-범위화된 자율 /loop 스텝보다 큰 단일집중 작업.
- 게이트 전부 green(값정확성43 트랜스파일러24 nl-check11). 회귀 0.

## 2026-06-06 (/loop iter 40: FP11c — fixpoint_full에 struct 통합 = 전체 통합 완료)
- fixpoint_full에 **struct 통합** → 🎯 **전체 통합 완료**: 함수+가변변수+while+if+배열+동적List+struct 합성.
- StructDef 테이블(build_defs, ≤6필드 위치+count), Slot에 sty(struct-type 인덱스/-1), defs 시그니처 threading
  (gen_* 5개 + emit_fn/gen_top/collectors 일괄), 슬롯수집기에 struct-decl skip(skip_struct_def)+struct-var 감지
  (rhs_struct_type→alloca [N x i64], sty 설정), gen_factor에 struct 필드 read, gen_stmts에 struct 리터럴+필드 write.
  **핵심: `.` 모호성을 slot kind로 분기** — sty>=0이면 struct field(p.x), 아니면 List(.len/.push). 토큰 struct=26.
  체크포인트별 빌드+검증(Slot확장→threading→헬퍼→슬롯감지→codegen 순, 각 단계 회귀0).
- **실측**: **함수 안 struct Tok{{kind,start,len}}=9**(컴파일러 Token 형태!), 필드 write f=12, **struct+List 한 함수 g=15**.
  전체 회귀0(build=100, sumarr=60, fac=120).
- 검증: fixpoint-full e2e **16 PASS**(struct 3 추가), 값-정확성 **43/43**, 트랜스파일러 24/24. 회귀 0.
- **🎯 fixpoint_full = nl 컴파일러가 쓰는 전체 구문(함수+제어흐름+배열+List+struct)을 합성하는 통합 컴파일러.**
  codegen 능력+통합 완비. 남은 것=실제 nl 소스(수천 줄) 입력=규모 작업(months급).

## 2026-06-06 (/loop iter 41: FP12a — 다중/zero param 함수, 실제 소스 향)
- 사용자 "이어서". FP12(실제 nl 소스 규모) 첫 조각: 통합 컴파일러에 **다중/zero param 함수** 추가.
- Fn struct에 param 리스트(p0s/p0l~p3s/p3l + npar), build_fns가 `(`~`)` 사이 param-list 파싱(콤마구분 0~4),
  emit_fn이 `define @f(i64 %a0, i64 %a1, ...)` + 각 param을 alloca/store(%aN→%v<slot>), body locals는 npar부터.
  gen_factor 호출이 arg_comma_end로 인자 분리해 N개(0~4) 전달(중첩 호출 인자 = depth 추적). 토큰 그대로.
- **트랜스파일러 한계 재확인**: 새 코드 주석의 lone brace(`'{'` 등 6곳)가 for/while 블록확장 깨 P001 → brace-free
  문구로 수정. (라인기반 트랜스파일러 알려진 클래스, 반복 패턴.)
- **실측**: add3(1,2,3)=6, answer()=42(zero-param), add(3,4)=7, s4(...)=100(4-param), one×3=3, **중첩인자
  add(dbl(3),dbl(4))=14**. 전체 회귀0(sum_to=15, fac=120, dist=9, build/sumarr/g 등). sanity grep %p_in→%a0 갱신.
- 검증: fixpoint-full e2e **21 PASS**, 값-정확성 **43/43**, 트랜스파일러 24/24. 회귀 0.
- **통합 컴파일러가 0~4 param 함수+zero-param+중첩 호출 인자를 codegen.** 실제 nl 소스 향 한 조각.
  다음 FP12b: 문자열/putchar/s[i](IR 텍스트 emit용), 이후 부트스트랩=months급.

## 2026-06-06 (/loop iter 42: FP12b — putchar codegen, 생성 프로그램이 출력 emit)
- FP12 두번째 조각: 통합 컴파일러에 **putchar codegen** — 생성 프로그램이 출력을 emit(nl 컴파일러 자신의
  핵심 작업: IR 텍스트를 putchar로 출력).
- `putchar(<식>)` 문장 → `declare i32 @putchar(i32)`(모듈 top) + `trunc i64→i32` + `call i32 @putchar`.
  is_putchar 헬퍼(7바이트 이름 체크), gen_stmts ident 분기 최상단에 추가. 호출 인자는 식(gen_expr).
- **실측**: show()→'HI', **stars(5)→'*****'(루프 putchar = 실제 emit 패턴!)**, putchar(65+k)='C'(계산된 char).
  전체 회귀0(fac=120, add3=6 등). e2e 23 PASS(stdout 출력 assert 2개 포함, check_out 헬퍼 re.sub 이스케이프 수정).
- 검증: fixpoint-full e2e **23 PASS**, 값-정확성 **43/43**, 트랜스파일러 24/24. 회귀 0.
- **생성 프로그램이 putchar로 출력 emit 가능** = nl 컴파일러가 IR 텍스트 생성하는 방식. 실제 소스 향 핵심 조각.
  다음 FP12c: 문자열 리터럴/s[i]/s.len()(토큰화/이름비교용), 이후 부트스트랩=months급.

## 2026-06-06 (/loop iter 43: FP12c — 문자열 codegen [소스 토큰화 프리미티브])
- FP12 세번째 조각, **소스-토큰화 프리미티브**: 문자열 리터럴 + 바이트 인덱싱 + 길이를 real LLVM IR로.
  nl 컴파일러가 **자기 소스 텍스트**를 읽는 데 필요한 바로 그 능력(`while i < src.len() { c = src[i]; ... }`).
- 신규 `compiler/self/fixpoint_str.nl` (독립 모듈, 의도적 string-focused):
  ① **2-pass**: pass1 `collect`가 문자열 리터럴마다 `@.sN = private constant [len+1 x i8] c"..\00"` 전역 emit
     + 변수 슬롯 수집(kind 1=string i8*, kind 0=scalar i64). pass2 `gen_stmts`가 본문 codegen.
  ② `let s = "lit"` → 전역 + `alloca i8*` + 전역 element-0 포인터를 i8*에 store(`emit_allocas`).
  ③ `s[i]` → `load i8*` + `getelementptr i8` + `load i8` + `zext i8→i64`(gen_factor `.`/`[` 분기).
  ④ `s.len()` → 컴파일타임 길이 리터럴(slen_of 슬롯 조회). `while`+대입은 imperative서 재사용.
  ⑤ 임베드 프로그램은 따옴표 충돌 회피 위해 **backtick(96)을 문자열 구분자**로 사용(tokenize 96 분기).
- **버그 1건 근본수정**: `skip_factor`의 `.len()` 토큰 폭 off-by-one — `ident . len ( )` = 5토큰인데 `i+4`
  반환 → 다중 `.len()` 체인(`a.len()+b.len()`)의 두번째가 누락(IR에 add 1개만). `i+5`로 수정 → 통과.
- **실측**: `s[1]+s.len()`=69, ABC 바이트합 스캔=198, 2-string len 독립추적(`a.len()+b.len()`)=181,
  `"return".len()`=6, `"tokenize"`[0]+[7]=217, 2-string 스캔+인덱스=183. IR을 clang으로 빌드+런타임 검증.
  (exit code 8bit 절단 확인: 진짜값 382가 126으로 → 테스트값 ≤255 유지.)
- 검증: fixpoint-str e2e **7 PASS**, 값-정확성 aggregate **44/44**(+1, 자동발견), 6 fixpoint e2e 전부 green,
  nl-check 전 모듈 clean. 회귀 0.
- **문자열 인덱싱+길이 codegen 가능** = 컴파일러가 자기 소스를 토큰화할 수 있는 구문 임계점 통과.
  남은 갭 = 통합+규모(months급 부트스트랩), 능력 부족 아님.

## 2026-06-06 (/loop iter 44: FP12d — 문자열을 통합 컴파일러 fixpoint_full에 병합 [전 구문 단일화])
- FP12c의 독립 문자열 codegen을 **통합 컴파일러 `fixpoint_full.nl`에 병합** → 단일 프로그램이 함수/가변/
  while/if-else/배열/List/struct/**문자열**/putchar를 **모두 함께** codegen(top-level + 함수본문 양쪽).
- 6-stage 체크포인트(각 단계 빌드+기존 e2e 회귀확인):
  ① tokenize에 backtick(96)→kind 28 strlit 분기 추가(content range nstart/nlen + 길이 value).
  ② 접근자 신설: `isarr_of`(is_arr 판별 0=scalar 1=array 2=List **3=string**), `strkey_of`(전역키).
     **Slot 필드 재활용**(구조체 불변): string은 `alen`=길이, `sty`=리터럴 nstart=전역키 `@.s<nstart>`.
  ③ `compile()`에 모듈 pre-pass `emit_str_globals` — 전 토큰 스캔, strlit마다 `@.s<nstart>` 전역 emit
     (함수 내부 문자열도 모듈톱 전역 필요 → nstart 키로 충돌없이 전역화). `emit_bytes` 헬퍼 추가.
  ④ slot 수집기 **2곳**(`collect_top_slots`=top, `add_local_slots`=함수본문)에 `rhs.kind==28` 분기 —
     `alloca i8*` + 전역 element-0 GEP + ptr store. (`collect_slots_range`는 dead/legacy 확인.)
  ⑤ `gen_factor`: `.` 분기 최상단에 string 우선체크(`isarr_of==3`→`s.len()`=길이리터럴; sty가 전역키라
     struct로 오인되므로 **순서가 핵심**), `[` 분기 최상단에 string byte-load(load i8*+GEP i8+load i8+zext).
  ⑥ `skip_factor` `.` 분기: `name.field`(3토큰, List `.len`/struct) vs `name.len()`(5토큰, 문자열) —
     `(` 뒤따르면 paren까지 skip(off-by-one 회피, FP12c 동일클래스).
- **버그 1건 근본수정**: `gen_stmts` let 핸들러가 string RHS를 scalar로 falthrough → `store i64 0`을 i8*
  alloca에 잘못 emit(포인터 0 오염, 증상 모든 string-arith=139 고정값). `rhs.kind==28` skip 추가(init은
  slot수집기가 이미 함=List/배열과 동일 패턴) → 해결.
- **실측**: top-level `s[1]+s.len()`=69, 스캔합=198, 2-string=181; **함수내부** `fn f(){let s=...}`=69,
  `fn count()` 스캔=5; **🎯 토큰화 코어** `fn tok(){ let s=`Hi`; let xs=list(); while i<s.len(){xs.push(s[i])...} }`
  =74(=len 2+'H'72) — fixpoint.nl 자신의 토크나이저와 **정확히 같은 shape**(문자열 바이트스캔→List push).
- 검증: fixpoint-full e2e **23→30 PASS**(+6 string +1 string-IR sanity, backtick은 bash `\`` 이스케이프),
  값-정확성 aggregate **44/44**, 6 fixpoint e2e 전부 0 fail, nl-check clean. **회귀 0.**
- **통합 완료**: 단일 컴파일러가 nl 컴파일러를 구성하는 전 코어구문을 codegen + 토크나이저 shape 포함.
  남은 갭 = **순수 규모**(실제 수천줄 컴파일러 소스를 먹이는 months급), 능력 부족 아님.

## 2026-06-06 (/loop iter 45: FP12e — 동작하는 토크나이저 실증 [렉서 inner loop])
- iter 44에서 통합된 string + 기존 if/while/state-var가 **함께** 동작함을 실증 = **완전한 토크나이저**.
  (FP12c 독립모듈서 실패했던 "yellow count"는 그 모듈에 if codegen이 없어서였음 — fixpoint_full엔 있음.)
- 실증 4종(전부 PASS): ① `cnt()` "yellow"의 'l' 개수=2(문자열인덱싱+비교+조건 in 스캔루프)
  ② `digits()` "a1b2c3" 숫자개수=3(**중첩 if**로 문자클래스 `ch>47 && ch<58` 판정 = 렉서 숫자토큰 인식)
  ③ `spaces()` 공백개수=2(토큰경계/whitespace skip shape)
  ④ **🎯 `ntok()` "ab cd ef" 토큰런 개수=3** — if/else + 중첩if + in-word 플래그 상태머신 over 문자열스캔
     = **실제 렉서의 동작 그 자체**. 단일 컴파일러가 서브셋 내에서 완전한 토크나이저를 codegen.
- e2e: fixpoint-full **30→32 PASS**(+cnt +ntok 가드), 값-정확성 44/44, 회귀 0.
- **"자기 토크나이저 shape를 codegen할 수 있나?"의 답 = YES**, 동작하는 토큰카운터로 실증. 남은 갭=순수 규모.

## 2026-06-06 (/loop iter 46: FP12f — 파서 코어 실증 [name_eq + 키워드인식])
- 토크나이저(iter 45)에 이어 **파서의 코어** = 두 바이트범위 비교(name_eq) + 키워드인식이 통합컴파일러서
  동작함을 실증. recon만으로 통과(추가 codegen 0 — 기존 string+if+while 조합으로 충분).
- 실증 3종(전부 PASS, e2e 가드 추가): ① `eq()` "let"=="let" 바이트비교=1, `ne()` "let"!="mut"=0
  (**name_eq shape** — 두 문자열을 인덱스루프로 바이트비교) ② **🎯 `kw()` 키워드인식**: `s.len()==3` 길이체크 +
  `s[0]==108 && s[1]==101 && s[2]==116`(중첩if) → "let" 인식=7. **실제 렉서의 kw3/kw5 패턴 그 자체**.
- e2e: fixpoint-full **32→35 PASS**(+eq +ne +kw), 값-정확성 44/44, 회귀 0.
- **결론: 통합 컴파일러가 토크나이저(iter45) + 파서 키워드인식(iter46) 둘 다 codegen** = 실제 컴파일러의
  가장 어려운 문자열-의존 두 부분 완비. 남은 갭은 **순수 규모(코드 양)**, 능력 아님. 능력 측면 self-host 핵심
  구문 전부 실증 완료. 다음 방향: 실제 소스 부트스트랩(months급) 또는 핵심 인프라(예제코퍼스/에러/std).

## 2026-06-06 (/loop iter 47: P9 코퍼스 확장 + README 갱신 [능력완비 후 인프라 전환])
- codegen 능력 완비(FP12f) + 실제소스 부트스트랩=months급 TRACKED → **scale-blocked 아닌 핵심 인프라**로 전환.
  std는 PRELUDE.md가 Vais 사상 명세(자체구현은 L3後)=적정수준 완료 판단 → **P9 예제코퍼스**(nl 최강레버)에 집중.
- **신규 검증 예제 4종**(AI가 자주 틀리는 패턴, 전부 트랜스파일+빌드+값검증): e16 Option match+payload 바인딩
  (`Some(v)=>return v`)=42, e17 struct 반환→필드접근(`make(4).x+.y`)=12, e18 while 누적기(가변 acc+카운터,
  sum of squares 1..=4)=30, e19 문자열 보간 출력(`print("{name} {n}")`)=0. 검증 코퍼스 26→30, 러너 44→48.
- **corpus README 갱신**(stale 수정): "21/21"→"30/30(러너 48/48)", 인덱스 e10→e19 추가, 미커버 정정
  (while/print는 이제 커버됨 → 제거; Map/중첩match가 진짜 갭).
- **추적(정정)**: Map<K,V> 실측 → 처음 transpiler 갭으로 추정했으나 **Vais 백엔드 버그로 판명**. Vais측
  직접 probe(`HashMap.new()` 정식문법 포함)도 C002/E004 `@HashMap_new` 모노모픽화 누락 + get_opt Option
  ptr/i64 불일치. Vais repo `tests/empirical/codegen_bugs/B-01,B-02`에 repro 존재 = **알려진 Vais codegen 버그**.
  `.filter()`와 동일 클래스(nl 아닌 Vais repo 수정). README/PRELUDE 🔶→🔴 정정. 중첩 match=P001 트랜스파일러 한계.
- 검증: 값-정확성 **48/48 회귀0**, nl-check 4 신규 clean. 교훈: std는 백엔드=Vais인 동안 사상명세가 정답
  (nl-native std는 L3後)/**추정("transpiler 갭")을 실측으로 검증 — 실제는 Vais 백엔드 버그**(B-01/B-02 repro
  존재)/P9 예제는 scale-blocked 아닌 즉시 레버.

## 2026-06-06 (/loop iter 48: P9 cold-start 재입증 [신선한 서브에이전트 실측] + e20 승격)
- P9 명제("예제 코퍼스가 cold-start를 가능케 함, 1/5→5/5")를 **맥락 없는 진짜 AI로 재입증**(설계원칙:
  검증=신선한 서브에이전트 cold-start). 방법: nl을 처음 보는 general-purpose 서브에이전트에게 **코퍼스
  README + 검증예제 9개만**(/tmp/nl_coldstart_context.txt, 172줄, Vais지식/프로젝트맥락 0) 주고 **코퍼스에
  없는 새 과제**(재귀 삼각수 tri(n)=n+tri(n-1)) 작성 지시.
- **결과: 첫 시도 성공**. 서브에이전트가 e03(factorial) 패턴에서 시그니처/`if`가드/재귀를 정확 추론 →
  `fn tri(n:Int)->Int { if n<=0 {return 0}; return n+tri(n-1) }`. nl-check clean, 트랜스파일+빌드+실행=**28 정확**.
- 결과를 **e20_triangular로 승격**(검증 코퍼스 30→31, 러너 48→49). e03(factorial, `<2` 가드)과 다른
  recurrence + `<=0` 가드 스타일 보강. README에 cold-start 재확인 노트 추가.
- 검증: 값-정확성 **49/49 회귀0**. 교훈: **cold-start 주장은 신선 서브에이전트로 주기적 재측정**(이전
  측정은 과거 코퍼스)/맥락격리가 핵심(컨텍스트파일만, 프로젝트지식 차단)/cold-start 산출물이 곧 새 검증예제
  (이중 레버: 명제입증 + 코퍼스+1).

## 2026-06-06 (/loop iter 49: P4 nl-check 카탈로그 강화 [Rust-ism 6→13종, 실측 기반])
- P4 에러인프라(nl-check)를 **실측으로 강화**: 어떤 Rust직관 실수가 nl-check를 빠져나가는지 17종 probe →
  7종 MISS 발견 → 각각 트랜스파일+빌드로 **진짜 nl서 실패함(E002/P001)** 확인 후 규칙 추가(오탐 방지: 코퍼스가
  보여주는 valid form 기준). 카탈로그 6→13종.
- 신규 7규칙: `vec![..]`→`[..]` / `Vec<T>`→`List<T>` / **i8..i128·u8..u128·f32·f64·usize·isize**→Int/UInt/F
  (capitalized) / `.to_string()`→`Str(x)` / `.unwrap()`·`.expect()`→match or `?` / `if let`→match /
  `elsif`·`elif`→`else if`. 각 `help:`+수정코드.
- **오탐 0 검증**(P4 핵심): 전 49 예제 + 17 self-host 모듈 nl-check clean 유지. 특히 식별자 부분문자열
  가드(`pi32`/`vector`는 i32/Vec 규칙 비발화 — `(?<![A-Za-z0-9_])..(?![A-Za-z0-9_])` 경계). `else if`도 비발화.
- 단위테스트 +15(7 true-positive + 8 clean-guard incl. 식별자substr/else if). nl-check unit **11→26 PASS**.
- 검증: nl-check 26/26, 값-정확성 49/49, 회귀 0. docstring에 카탈로그 명시. 교훈: **에러규칙은 추측말고
  실측으로**(17 probe→7 진짜 MISS, 각 빌드로 실패확인)/오탐 방지는 식별자 경계 가드 필수(i32 substr 함정)/
  코퍼스가 "valid form" 권위(규칙이 코퍼스 깨면 그 규칙이 틀림).
