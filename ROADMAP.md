# nl ROADMAP — source of truth (current-only)

> 이 파일은 /loop 자율 진행의 **단일 진실원**. 현재 작업만 담는다(해결된 건 WORKLOG로).
> 각 task는 **값-정확성 검증 + 커밋** 후 done. 막히면 추적(TRACKED)하고 넘어간다.

## 완료 정의 (정직)
- ✅ 목표: **L3(자체 컴파일러 프론트엔드) + 핵심 인프라**(예제코퍼스 P9, 에러 P4, std 시작, 게이트 P7b).
- ❌ 비목표: L4(생태계/시장) — 코드로 "완료" 불가. 의도적 제외.
- 백엔드: 당분간 Vais 재활용(트랜스파일). 자체 codegen은 L3 후반/후속.

## "nl은 컴파일러만이 아니다" — 만드는 것 전체
컴파일러 + std + 툴 + **예제 코퍼스(P9, nl 차별점)** + 문서 + 검증 게이트.

---

## ⚑ 상태 (2026-06-06): P0~P5 인프라 + L3 self-host 미니 컴파일러 완료

P0 게이트 / P1 코퍼스 / P2 트랜스파일러 / P3 에러인프라 / P4 std시작 / P5 레퍼런스 = **DONE**.
L3(self-host) + CX1~9 + FIXPOINT(FP1~FP12f) = **DONE**(codegen 능력 완비).

**현재(2026-06-07) 게이트 상태** (이번 세션서 크게 성장):
- 값-정확성 **71/71** (예제코퍼스 53 검증 e16~e42 + self-host codegen 모듈).
- 트랜스파일러-단위 **45/45**, nl-check-단위 **34/34** (Rust-ism 14규칙).
- self-host e2e **77 PASS** (fixpoint-full 35 + str 7 + list 5 + struct 6 + array 7 + imperative 17).
- cold-start: 신선 서브에이전트 다회 첫시도 성공 + 자기수정 1라운드 수렴 실측.

**완료 정의(L3+코퍼스+에러인프라+std) = nl측 충족**. 남은 것:
1. **실제 소스 부트스트랩**(months급, TRACKED) — 능력 완비, 순수 규모 문제.
2. **점진 인프라**(코퍼스 확장 / nl측 갭 수정 / cold-start 재측정) — scale-blocked 아님, /loop 계속.
3. **Vais 백엔드 버그 6종**(TRACKED, 근본=Vais repo) — Map/int→string/중첩Vec/리터럴인자/Vec성장.

---

## 우선순위 큐 (위에서부터)

### P0 — 검증 게이트 먼저 (P7b: 컴파일≠정답)
- [x] **G1. 값-정확성 테스트 러너** (`scripts/test.sh`): examples/*.nl 중 expect 주석 있는 것 빌드+실행+exit 비교.
      각 .nl 첫 줄 `# expect: N` 규약. green 유지가 이후 모든 작업의 안전망.
- [x] **G2. 예제에 expect 주석 부착** + 러너로 현재 baseline 측정 (12/13 예상, filter는 Vais버그).

### P1 — 예제 코퍼스 확장 (P9: 최강 레버, nl 핵심 인프라)
- [x] **C1. 코퍼스 2배 확장** (현재 ~13 동작 → 25+): 더 다양한 과제(중첩 struct, enum payload,
      재귀, 다중 함수, 문자열 처리). 각 expect 주석 + 러너 green.
- [x] **C2. 코퍼스 README** (`examples/README.md`): 어떤 문법을 어떤 예제가 커버하는지 인덱스.

### P2 — 트랜스파일러 견고화 (L2 마무리)
- [x] **T1. while 루프** 지원 (현재 미지원 유일 구문).
- [x] **T2. exclusive range** `..` (현재 `..=`만).
- [x] **T3. 트랜스파일러 자체 단위테스트** (입력 nl → 기대 Vais 출력 비교, 회귀 방지).
- [x] **T4. nested for / 복합 표현** 견고성 (현재 라인-재작성기 한계 구간).

### P3 — 에러 인프라 (P4: AI self-correction)
- [x] **E1. nl 에러 래퍼** (`tools/nl-check`): nl→Vais 트랜스파일 실패/Vais 에러를 nl 좌표+`help:`로 변환.
      흔한 실수 카탈로그(Rust직관: `&&`→`and`, `::`→`.`, `as`→`Int(x)`, turbofish 등)에 수정코드.
- [x] **E2. cold-start self-correction 측정**: nl 에러로 신규 AI가 N라운드에 수렴하는가 (Rust 대비).

### P4 — std 시작 (Vais 재활용 위에 nl 표면)
- [x] **S1. nl prelude 명세** (List/Map/Option/Result/print) — Vais std로 매핑되는 표면 API 문서.
- [x] **S2. print/IO** 최소 (nl `print("{x}")` → Vais) — 실행 결과 출력.

### P5 — 문서 (레퍼런스)
- [x] **D1. 언어 레퍼런스** (`docs/reference/`): v0.2 문법을 사용자용 튜토리얼로 (설계문서와 별개).

---

## TRACKED (막혀서 넘어간 것 — 근본은 Vais repo)
- Vais filter 버그 (task_7cfebeba): `.filter()`가 specialized body서 `%Vec*` 오타입 → nl d6 막힘.
  Vais repo 작업. nl 트랜스파일은 정상.
- 트랜스파일 천장(원천적): P7 단일coercion / P8 클로저 day-1은 Vais 위에선 실현 불가.
  → L3 자체 컴파일러에서만 해결. (현재 큐는 L3 프론트엔드 진입 전 인프라 다지기.)

---

## TRACKED 추가 (Vais 버그)
- **Vais 캡처 클로저 반환 미지원**: `fn adder(n) -> fn(Int)->Int { return |x| x + n }`(n 캡처 클로저를 bare
  fn-ptr로 반환) E001. **클로저를 인자로 받는 건 OK**(e49) — 반환 경계서 env 캡처가 막힘(bare fn-ptr엔 env 없음,
  {code,env} 표현 미지원). Vais 클로저 ABI 작업 필요. 하드코딩 Vais도 실패. 2026-06-07 실측.
- **Vais 재귀(자기참조) enum codegen 버그**: `enum Expr { Lit(Int), Add(Expr, Expr), ... }`(enum이 자신을
  payload로) 무음 miscompile — 1-level `Mul(Lit(3),Lit(4))`도 0/garbage 139 반환. 트랜스파일러는 올바른
  Vais(`enum Expr { Add(Expr,Expr) }`) 생성하나 Vais가 재귀 ADT payload 추출 못 함. **비재귀 enum(2-payload Int)은
  OK**(e50). **이것이 nl self-host codegen 트랙이 AST를 재귀enum 대신 struct+인덱스로 인코딩하는 근본 이유.**
  재귀 ADT는 실전 인터프리터/파서의 핵심 → 중요 갭. Vais 코어 작업 필요. 2026-06-07 실측.
- **Vais `impl Trait for Type` 미지원**: `impl Area for Sq { ... }`가 P001(`for`서). **`impl Type { ... }`
  (inherent 메서드)는 OK**(e09/e43 동작) → nl 구조체 메서드는 정상, trait 기반 다형성만 막힘. Vais 파서 확장
  필요. nl측은 `impl Type` 형태로 충분(트랜스파일 정상). 2026-06-07 실측.
- **Vais Vec 성장(push/map/filter) codegen 버그**: 리터럴 Vec에 `.push(x)`는 무음 miscompile(`@Vec_push`
  undefined, len 오염 → garbage 134), `.map()`은 `@Vec_push` undefined로 빌드실패, `.filter()`는 E001
  (task_7cfebeba). **read-only(len/index/fold/sum)는 OK**. 리스트 변형/구축은 `for`-루프 누적(e25/e27) 권장.
  nl self-host codegen 트랙은 Vais Vec 안 쓰고 고정버퍼로 우회. PRELUDE push/map 🔴 정정. 2026-06-07 실측.
- **Vais 중첩 Vec codegen 버그**: `Vec<Vec<i64>>` 리터럴/인덱싱이 C003 Type error(하드코딩 Vais도 실패).
  nl 트랜스파일러는 올바른 `Vec<Vec<i64>>` 타입 생성하나(nested 추론 수정 685ba63 다음 커밋) Vais가 codegen
  못 함. nl 중첩 리스트 `[[..]]` 막힘. Vais repo 작업 필요. 2026-06-07 실측.
- **Vais 리스트-리터럴 직접 인자 코어션 갭**: `f([1,2,3])`(리터럴을 Vec 파라미터에 직접) E001 Type mismatch.
  바인딩 후 전달(`let v=[..]; f(v)`)은 OK. nl 코퍼스는 bound-var 패턴 권장(e27). Vais 코어션 수정 필요. 2026-06-07 실측.
- **Vais 표면 int→string 변환 부재**: `str(42)` P001(str=타입키워드, 호출불가), `to_string(42)` E002,
  `(42).to_string()` C002. 내부 `__i64_to_str`(FFI impl)만 존재. nl `Str(x)` 변환콜이 `str(x)`로 사상돼 실패.
  → nl-check가 `.to_string()`을 flag하되 대체 약속 안 함(정직). 표면 변환은 Vais 백엔드 작업 필요. 2026-06-06 실측.
- **Vais HashMap codegen 버그** (Map<K,V> 막힘): `HashMap.new()` 모노모픽화 누락(C002/E004 undefined
  `@HashMap_new`) + `get_opt` Option ptr/i64 불일치. Vais repo `tests/empirical/codegen_bugs/B-01,B-02`에
  repro. nl Map 예제/PRELUDE ✅ 승격 막힘. `.filter()`와 동일 클래스(Vais repo 수정 필요). 2026-06-06 실측 확인.
- ✅ **Vais &Vec borrow 재귀 — 해결**(2026-06-06, compiler 214c97cf): `&Vec<T>`가 슬라이스 fat-ptr로
  잘못 codegen되던 버그 근본 수정 → 이제 주소 전달. **nl이 `&List<T>` borrow로 Vec 재귀 가능**
  (e15_list_recursion 실측 10). fixpoint(AST 순회)의 핵심 기반 확보. by-value=E022 move는 여전(설계상
  move 시맨틱이 정상 — borrow가 정답). task_54658a43의 &Vec 측면 closed; by-value move는 의도된 동작.
- Vais `&&`/`||` 비단락평가 (task_492f7e17): `i<n && arr[i]` 가 i==n서 crash.
  nl lexer는 nested-if로 우회 중. 근본은 Vais codegen(논리연산→분기). 심각도 높음.
- Vais 전역 Vec 리터럴 codegen: `G v: Vec<i64> = [..]` → clang "integer constant must have integer type".
- Vais `&&`/`||` 비단락평가 (task_492f7e17): `i<n && arr[i]` 가 i==n서 crash.
  nl lexer는 nested-if로 우회 중. 근본은 Vais codegen(논리연산→분기). 심각도 높음.
- Vais 전역 Vec 리터럴 codegen: `G v: Vec<i64> = [..]` → clang "integer constant must have integer type".
  CX5 재귀 fn-테이블 시도 시 발견 → struct Defs로 대체.

## TRACKED (nl 인터프리터 한계 — 근본은 위 Vais Vec-recursion)
- **멀티문자 식별자 미지원**: 현 Env=26 단일바이트 슬롯(변수명 1글자=슬롯). `fib`/`sq` 같은 다중자
  이름은 name(문자열)→slot 매핑 필요 → 문자열-키 심볼테이블 = Vec/map 재귀전달 = Vais E022 재충돌.
  CX1-9 전부 단일자 이름이 honest scope. 멀티문자는 심볼테이블 재설계(이름 인터닝 등) = 큰 단계.
- **진짜 self-compile fixpoint**: 현 cx5_compiler는 산술/함수/재귀 프로그램을 *값으로 평가*하는
  인터프리터. nl이 자기 컴파일러 소스를 컴파일(fixpoint)하려면 전체 nl 문법 파싱 + 실제 codegen
  필요 = L3 엔드게임. 백엔드 전략(자체 codegen vs Vais 수정) **사용자 결정** 필요.

## L3 진입 (인프라 다진 후 — 별도 큰 단계)
자체 컴파일러: lexer → parser → typecheck → (Vais IR lowering 또는 자체 codegen).
**시작 시 사용자 결정 필요**: 컴파일러 작성 언어(Rust/self-host/기타), 백엔드 전략, 에러 day-1.
→ 이건 추측 금지. 인프라(P0~P5) 완료 후 사용자에게 escalate.

## L3 진입 — self-host (사용자 결정: nl 자체로, 2026-06-06)

부트스트랩 경로: nl 컴파일러 소스(.nl) → [트랜스파일러=시드] → Vais → vaisc → gen1.
nl 컴파일러 코드는 트랜스파일러 지원 부분집합으로만 작성 (검증: while/if/and/s[i]/struct/enum/Vec).

### L3 큐
- [x] **L3.0** 부트스트랩 가능성 검증 (nl 렉서 조각 트랜스파일+실행 OK).
- [x] **L3.1** lexer 시작 (compiler/self/lexer.nl): 문자분류(is_digit/alpha/space) + 단어/숫자 스캔.
      값-정확성 게이트 편입 (test.sh가 compiler/self/*.nl도 검증). 26/26 green.
- [x] **L3.2** lexer 확장: 실제 토큰 emit (종류+위치). 키워드/식별자/숫자/기호/문자열 인식.
- [x] **L3.3** parser/eval (compiler/self/parser.nl): 토큰 → AST (작은 부분집합 fn/let/return/표현식).
- [x] **L3.4** codegen (compiler/self/codegen.nl): AST → Vais 텍스트 lowering (재활용) 또는 직접 IR.
- [x] **L3.5** 통합 미니 컴파일러 (compiler/self/compiler.nl): 소스→lex→eval→IR: gen1이 자기 nl 소스 처리.

각 단계 값-정확성 게이트 green + 커밋. Vec/문자열 연산이 트랜스파일러/Vais 한계에 부딪히면 TRACKED.

## 컴파일러 확장 큐 (CX) — 산술 → 전체 nl 향해 (2026-06-06~, /loop)

compiler.nl을 점진 확장. 각 단계 값-정확성(생성 IR 실행) 검증 + 커밋. Vais 버그는 우회+추적.
- [x] **CX1** 변수: `let x = <식>` 바인딩 + 참조. 다중 let + 최종식.
- [x] **CX2** 여러 문장 (;-구분, run_program 구현됨) 시퀀스(줄/세미콜론) → 최종값.
- [x] **CX3** return 문 (구현됨).
- [x] **CX4** 간단 if (조건식). `if <식> <비교> <식> then <식> else <식>`, 비교 `> < ==`,
      조건/분기 모두 변수+산술 지원. **Vec-move 우회**: 4 부분식을 단일 루프로 평가
      (straight-line 2회 호출=E022, 루프 반복=OK 실측). **트랜스파일러 버그 수정**:
      문자열 리터럴 안의 `if`/`and` 등 키워드가 코드처럼 재작성돼 임베디드 프로그램
      텍스트 오염(`return if`→`return I`) → `outside_strings` 헬퍼로 map_if/map_words
      문자열 보호. e2e 15/15, 트랜스파일러 단위 22/22.
- [x] **CX5** 함수 정의 다중 fn (self-host 큰 관문). `fn <f>(<p>) {{ return <식> }}` 정의 +
      호출 `<f>(<인자식>)`, 다중 fn + **중첩 호출**(한 본문이 다른 fn 호출). compiler/self/cx5_compiler.nl.
      **핵심 설계(Vais Vec-move 우회)**: 평가 환경 `Env`(8슬롯)와 함수 테이블 `Defs`(3슬롯)를
      **고정필드 struct**로 → 재귀 평가에서 E022 없이 전달(struct는 재귀-복사 안전, Vec는 move 실패 실측).
      소스는 불변 Str. 산술식 평가기는 상호재귀(eval_factor↔term↔expr, Cur struct 반환).
      **트랜스파일러 버그 수정**: Vais가 모든 문자열 리터럴의 `{ }`를 보간으로 처리 → 코드-as-데이터
      불가. nl `{{`/`}}` → Vais `\{`/`\}`(보간 회피 literal brace)로 변경(map_brace_escapes).
      e2e 6/6(중첩호출 포함). 값-정확성 30/30.
- [x] **CX6** 함수 본문 조건식 → **재귀 함수**. `if <식> <비교> <식> then <식> else <식>`를 완전
      표현식으로(eval_value, 분기는 eval_expr 재귀 → 분기 내 재귀호출 동작). factorial(5)=120,
      **fibonacci(10)=55**(트리 재귀), sum(1..10)=55 실측. struct-Env가 재귀를 move-safe하게.
      e2e 11/11(CX5 6 + CX6 5). 값-정확성 30/30.
- [x] **CX7** 다중 인자 함수 (1~2 param). `fn m(a, b) {{ return a + b }}` 정의/호출, 콤마 파싱
      (정의 param-list + 호출 arg-list), Defs에 param2 추가, 호출 시 양 param 바인딩. 2-인자 재귀
      (power p(b,e)=3^4=81), max(a,b), 인자식 m(1+2,3*4)=15 실측. e2e 18/18. 값-정확성 30/30.
- [x] **CX8** 지역 변수 (let). 함수 본문을 `let <v> = <e>; ... return <e>` ;-문장열로(eval_body 신설:
      let→env eset, return/bare-expr→값). def-parser bs를 `{` 직후로(eval_body가 return 파싱). run_program도
      top-level let 지원(tenv) → cx5_compiler가 CX1-3 compiler 상위집합. 실측: 의존 지역변수, 2-arg+local,
      local→재귀호출인자, bare-expr 본문, top-level var→fn인자. e2e 25/25. 값-정확성 30/30.
- [x] **CX9** Env 슬롯 a-z 26개로 확장. **eset 압축 핵심**: rebuild-all(676줄) 대신 `let mut e = env;
      if ch==.. {{ e.X = v }}` 26 one-line(struct in-place mutation + 재귀 안전 실측). 변수 t/r/s/z/w,
      3개 distinct fn, high-letter top-level var 실측. e2e 29/29. 값-정확성 30/30.
- ...최종: nl이 자기 일부 소스 컴파일 (fixpoint 근접).

## FIXPOINT 큐 (사용자 결정: 진짜 fixpoint 도전, 2026-06-06)
List 기반 파이프라인: source → tokenize → **List<Token>** → 재귀 평가/codegen → IR.
cx5_compiler(단일-문자열-스캔)와 달리 진짜 토크나이저+AST 단계. Vais &Vec borrow 재귀
수정(compiler 214c97cf)으로 List 재귀 가능해져 시작.
- [x] **FP1** List<Token> 토크나이저 + 재귀 평가 (compiler/self/fixpoint.nl). 멀티자릿수/공백/
      precedence(*>+,-)/**좌결합**(left-fold eval_expr_fold). source→List<Token>→&List 재귀 eval→IR.
      e2e 10/10(test-fixpoint.sh). 값-정확성 32/32.
- [x] **FP2+FP4** 변수(let) + **멀티문자 식별자** List 파이프라인 (compiler/self/fixpoint2.nl).
      Token이 이름을 소스범위(nstart,nlen) 보유, 심볼테이블 List<Var>를 name_eq(소스바이트비교)로 조회.
      `let total = 10; let count = total * 4; return total + count` → 50. **이전 단일자 한계 해소.**
      접두사공유(foo/food) 구분, 의존체인, precedence/좌결합 전부. e2e 9/9. 값-정확성 33/33.
      keyword(let/return) vs ident 토큰 구분(word_is). 핵심: Vais &Vec 수정으로 &List<Var>/&List<Token>
      재귀 가능. (FP2 변수 + FP4 멀티문자 동시 달성.)
- [x] **FP3** 멀티문자 함수 정의+호출 (compiler/self/fixpoint3.nl). `fn name(p[, q]) {{ return e }}` →
      List<Fn>{name범위, param범위들, body 토큰범위}, find_fn(name_eq)로 디스패치. 호출 시 새 vars 스코프
      (param=arg push) + body 토큰범위 eval. **중첩/cross-call**(add(square(x),x) / g(b)=f(b)+b). 평가기가
      &List<Token>+&List<Fn>+&List<Var> 3중 재귀(Vais &Vec 수정 기반). 실측 square(5)=25, add(sq(w),sq(h))=41.
      e2e 7/7. 값-정확성 34/34. (body 내 `{{`/`}}`로 brace 이스케이프.)
- [x] **FP3b** 함수 본문 조건식 → **멀티문자 재귀 함수**. fixpoint3에 if/then/else 토큰(15/16/17) +
      비교 `< > ==`(18/19/20, `==`는 `=` 2개 구분) + eval_value(find_kind로 then/else 경계, 분기는
      eval_value 재귀). body/return을 eval_value 경유. 실측 **factorial(5)=120, fib(10)=55(멀티문자 트리재귀)**,
      sumto(10)=55, clamp, 재귀+변수+cross-fn add(fact(4),base)=28. e2e 13/13. 값-정확성 34/34.
- [x] **FP5** **진짜 codegen** (compiler/self/fixpoint_codegen.nl). 값 미리계산(인터프리터) 대신
      **런타임 계산 IR emit** — mul/add/sub 명령 + SSA temp. gen_factor/term/expr가 Op{kind,val,next}
      반환(kind 0=literal/1=temp), emit_binop 4분기(operand literal/temp). 좌결합 fold. `12+3*4` →
      `%t1=mul 3,4; %t2=add 12,%t1; ret %t2` → 런타임 24. e2e 10/10(+"mul emit" 검증=진짜 codegen 증명).
      값-정확성 35/35. **Vais 2버그 수정 기반**: &Vec 재귀(214c97cf)+`%` 이스케이프(e711dac1, `%tN`이 출력서
      printf specifier로 소비되던 것 수정).
- [x] **FP6** 변수 codegen (compiler/self/fixpoint_codegen2.nl). `let`/return을 런타임 계산 IR로.
      SSA 모델: 변수=그 식이 만든 operand(literal/temp) 매핑(불변이라 alloca 불필요), List<SymOp> 심볼테이블,
      참조는 name_eq로 operand 해소. `let x=5; let y=x*2; return y+1` → `%t1=mul 5,2; %t2=add %t1,1` → 11.
      멀티문자/의존체인/변수재사용(z*z). e2e 8/8. 값-정확성 36/36.
- [x] **FP7** 함수 codegen (compiler/self/fixpoint_codegen3.nl). `fn name(p) {{ return e }}` →
      `define i64 @name(i64 %p) {{ ... }}`, 호출 → `call i64 @name(i64 arg)`. 멀티문자 이름을 LLVM
      식별자로 emit(emit_name putchar 바이트별). Op에 named(%<param>) 종류 추가. print_int_inline/emit_str로
      IR 조립. `fn double(x){{return x*2}}; return double(21)` → `define @double`+`call @double` → 런타임 42.
      param 산술본문/인자식/호출결과 산술. e2e 7/7. 값-정확성 37/37.
- [x] **FP8** 조건/분기 codegen → **재귀 함수 codegen** (compiler/self/fixpoint_codegen4.nl).
      `return if <식> <비교> <식> then <식> else <식>`를 icmp+조건br+labeled blocks(then/else/merge)+phi로 emit.
      비교 < > == (slt/sgt/eq), label은 temp 번호 기반 고유. gen_body가 분기 각각 codegen(분기 내 재귀 call은
      그 블록에만). **factorial(5)=120, fib(10)=55(멀티문자 트리재귀), sumto(10)=55** 모두 네이티브 IR 생성+실행.
      e2e 8/8(+icmp/phi/recursive-call emit 증명). 값-정확성 38/38.

**🎯 CODEGEN 트랙 완성도**: nl 컴파일러가 산술→변수→함수→**재귀 함수**(제어흐름 포함)를 진짜 LLVM IR로
생성. 컴파일러 codegen의 핵심 전부 달성. (정직한 한계: 완전 self-compile=전체 nl 문법[struct/while/Vec/
method/&]+그 codegen 재구현=수천줄. 현재는 산술/함수 부분집합의 완전한 컴파일러.)
- [x] **FP9** (선택) — **subsumed**: 다중 인자는 FP12a(0-4 param), 지역변수(let in body)는 CX8 + fixpoint_full
      슬롯수집기로 이미 통합 달성. 별도 작업 불필요.

## 완전 self-compile 도전 (사용자 결정 2026-06-06): 명령형 codegen
목표=nl이 자기 컴파일러가 쓰인 구문(가변변수/while/List/method/&)을 컴파일. 점진 확장.
- [x] **FP10a** 가변변수 codegen (compiler/self/fixpoint_imperative.nl). `let mut`/assignment/return을
      **alloca/store/load**로 → SSA-operand 모델로 불가능했던 **변이** 가능. 변수마다 alloca 슬롯(collect_slots
      1패스), 참조=load, 대입=store. `let mut s=10; s=s+5; s=s*2; return s`→30. 다중변수/누적. e2e 6/6. 값-정확성 39/39.
- [x] **FP10b** while 루프 codegen. `while <식> <비교> <식> {{ <문장들> }}`를 loop/body/done 블록 +
      조건 icmp+br + back-edge로 emit. gen_stmts 재귀(본문도 문장열), match_brace로 중첩 `{}` 처리,
      label은 temp 번호 기반 고유(순차 루프도 안전). 비교 < > ==. **sum(1..5)=15, 5!=120(루프), countdown,
      0회실행, 두 순차루프** 모두 네이티브. e2e 13/13. 값-정확성 39/39. **nl이 반복 명령형 프로그램을 codegen.**
- [x] **FP10c** if/else 문 codegen (제어흐름). `if <식> <비교> <식> {{ <문장> }} [else {{ <문장> }}]`를
      icmp+조건br + ithen/ielse/imerge 블록으로 emit(else 없으면 빈 else→merge). gen_stmts 재귀(분기도 문장열),
      else 옵션 감지, match_brace 중첩. **루프 안 if도 동작**(1..10 중 >5 카운트=5). e2e 17 PASS. 값-정확성 39/39.
      **명령형 3종(가변변수+while+if) 완성** = nl 컴파일러 자신이 쓰인 제어구조.
- [x] **FP10d** 배열 codegen (compiler/self/fixpoint_array.nl) — 데이터구조 codegen 시작.
      `let a = [v0, v1, ...]`→`alloca [N x i64]`+GEP store, `a[식]`→GEP+load(런타임 인덱스), `a[식] = 식`→
      GEP+store. Slot에 is_arr/alen 추가, collect_slots가 배열 길이(콤마+1) 계산. **루프로 배열 합산**
      (sum array=100), **루프로 배열 쓰기**(a[i]=i*10), 런타임 인덱스 a[i]. e2e 7 PASS. 값-정확성 40/40.
      List(컴파일러가 쓰는 핵심 자료구조)의 기반.
- [x] **FP10e** struct codegen (compiler/self/fixpoint_struct.nl). struct=고정 [N x i64], 필드명→인덱스.
      `struct Name {{ f0, f1, .. }}` 선언(build_defs, ≤6필드) + `Name {{ f: v, .. }}` 리터럴(필드별 GEP store) +
      `p.field` read(GEP+load) + `p.field = 식` write. Slot에 ty(struct-type 인덱스 또는 -1 scalar). 실측 p.x+p.y=7,
      필드대입 104, **3필드 Tok{{kind,start,len}}=9**(컴파일러의 Token 형태), b.w*b.h=20, 식 필드값 25. e2e 6 PASS.
      값-정확성 42/42. **struct=nl 컴파일러 자신의 레코드(Token/Op/Fn/Slot) 형태.**
- [x] **FP10g** 동적 List(push/len/index) codegen (compiler/self/fixpoint_list.nl) — **데이터구조 codegen 완비**.
      List=고정용량 버퍼(alloca [64 x i64]) + 길이 카운터(alloca i64). `let lst = list()`(2슬롯+len=0),
      `lst.push(식)`(buf[len] store + len++), `lst.len`(load 카운터), `lst[식]`(GEP+load), `lst[식]=식`(store).
      Slot에 kind(0 scalar/1 list). **push 루프 + xs.len 루프 합산=100**(컴파일러 tokenizer 빌드+소비 패턴!),
      len 추적, 원소 대입. e2e 5 PASS. 값-정확성 43/43. **List<T>=nl 컴파일러 핵심 자료구조(List<Token>/List<Fn>).**

**🎯🎯 codegen 데이터구조 완비**: 배열 + struct(레코드) + 동적 List(push/len). nl 컴파일러가 자신이 쓰는
모든 핵심 구문(산술/변수/함수/재귀/가변변수/while/if/함수+명령형본문/배열/struct/List)을 네이티브 LLVM IR로
생성. 진짜 self-compile의 codegen 토대 완성. (남은 것=전체 통합 단일 컴파일러로 묶고 실제 nl 컴파일러
소스를 입력으로 = 통합/스케일 작업, months급. 현재는 각 구문의 검증된 codegen 보유.)
- [x] **FP10f** 함수+명령형본문 통합 (compiler/self/fixpoint_full.nl) — **self-compile 직결 캡스톤**.
      `fn name(p) {{ let mut ...; while ...; if ...; return }}` + 호출을 진짜 IR로. 각 함수=`define i64
      @name(i64 %p_in)`, param→alloca %v0 복사, body locals alloca, 명령형 본문(gen_stmts), 호출=`call`.
      gen_* 시그니처에 &List<Fn> threading. build_fns(중첩 brace depth) + per-function slot scope + gen_top
      (top-level만, fn def skip). **루프함수 sum_to(6)=15, 루프 fact(6)=120, 함수 안 if clamp, 재귀 fac(5)=120,
      cross-call quad=20**. e2e 7 PASS. 값-정확성 41/41. **nl 컴파일러 자신의 함수 형태(함수+가변locals+루프+조건+재귀)를 codegen.**

**FIXPOINT 진척 종합**: fixpoint3.nl이 멀티문자 변수/함수/재귀+조건+precedence/좌결합을 List<Token>
파이프라인으로 컴파일. cx5_compiler(단일자, string-rescan)를 모든 면에서 능가. 진짜 self-compile
(nl이 자기 컴파일러 컴파일)에는 FP5(전체 nl 문법+실제 codegen)가 남음 — 큰 단계.

## 완료 정의 충족 상태 (2026-06-06)
P0~P5 + L3(self-host 미니컴파일러) + CX1~CX9 = **DONE**. ROADMAP 완료정의(L3+코퍼스37+에러인프라
nl-check+std시작 PRELUDE+게이트3종) **충족**. FIXPOINT 큐는 그 너머 "진짜 self-compile"로 진행 중.

전략: 단일파일/인덱스로 Vais 버그(Vec-재귀전달/&&단락) 회피 유지. 큰 관문(CX5+)서 막히면
자체 codegen 또는 Vais 수정 필요성 사용자 escalate.

## 진행 규칙 (/loop)
1. 큐 맨 위 미완료 task 실행 → 값-정확성/러너 green 확인 → 커밋 → 체크.
2. 막히면 TRACKED로 옮기고 다음 task.
3. P0~P5 다 끝나면 L3 결정을 사용자에게 escalate (추측으로 L3 시작 안 함).
4. WORKLOG.md에 각 iteration 기록.

## 통합 (사용자 결정: 구문별 codegen을 하나로, 2026-06-06)
- [x] **FP11a** fixpoint_full에 배열 통합 — **함수+명령형본문+배열**이 한 컴파일러에서 합성.
      Slot에 is_arr/alen 추가, 배열 토큰/codegen(리터럴 store, 인덱스 read/write)을 함수-스코프 슬롯에
      통합, collect/add_local_slots/gen_factor/gen_stmts/skip_factor 일괄. **함수 안 배열+루프 sumarr(3)=60,
      배열 쓰기 build=15, 배열+if pick=9**, 기존 재귀/명령형 회귀0. e2e 10 PASS. 값-정확성 43/43.
- [x] **FP11b** fixpoint_full에 **동적 List 통합** — 함수+가변변수+while+if+배열+**List**가 한 컴파일러 합성.
      List=is_arr=2(버퍼 [64 x i64] @slot + 길이 @slot+1, 2슬롯). `let lst=list()`/`lst.push(식)`/`lst.len`/
      `lst[식]`/`lst[식]=식` codegen, 슬롯수집기에 list() 감지(rhs_is_list, 2슬롯 할당). 토큰 . 추가.
      **함수가 List 빌드+소비(push 루프+xs.len 합산) build(5)=100**(컴파일러 tokenizer 패턴!), 함수안 배열+List
      혼합 mix=105. 회귀0(sumarr=60, fac=120). e2e 13 PASS. 값-정확성 43/43.
- [x] **FP11c** fixpoint_full에 **struct 통합** — 🎯 **전체 통합 완료**: 함수+가변변수+while+if+배열+동적List+
      **struct**가 한 컴파일러 합성. StructDef 테이블(build_defs ≤6필드), Slot에 sty, defs 시그니처 threading,
      슬롯수집기 struct-decl skip+struct-var 감지(alloca [N x i64]), gen_factor/gen_stmts에 struct 리터럴/필드
      read/write를 **`.` 모호성을 slot kind로 분기**(sty>=0=struct field, List=`.len`/`.push`)로 추가. **함수 안
      struct Tok{{kind,start,len}}=9**(컴파일러 Token 형태!), 필드 write f=12, **struct+List 한 함수 g=15**.
      전체 회귀0(build=100, sumarr=60, fac=120). e2e 16 PASS. 값-정확성 43/43. **fixpoint_full = nl 컴파일러가
      쓰는 전체 구문(함수+제어흐름+배열+List+struct)을 합성하는 통합 컴파일러.**
- [x] **FP12a** 다중/zero param 함수 (fixpoint_full). Fn에 param 리스트(p0~p3, npar), build_fns가 param-list
      파싱, emit_fn이 `define @f(i64 %a0, i64 %a1, ...)` + 각 param alloca/store, 호출이 N개 인자(arg_comma_end로
      분리) 전달. **add3(1,2,3)=6, answer()=42(zero-param), s4(...)=100(4-param), 중첩인자 add(dbl(3),dbl(4))=14**.
      전체 회귀0(1-param/재귀/struct/List). e2e 21 PASS. 값-정확성 43/43.
- [x] **FP12b** putchar codegen — 생성 프로그램이 **출력 emit**(nl 컴파일러 자신의 핵심 작업). `putchar(<식>)`
      문장을 `declare i32 @putchar(i32)` + `trunc i64→i32` + `call @putchar`로. 루프 putchar = 실제 IR-emit 패턴.
      **show()→'HI', stars(5)→'*****'(루프), putchar(65+k)='C'**. 전체 회귀0. e2e 23 PASS(출력 assert 포함). 값-정확성 43/43.
- [x] **FP12c** 문자열 리터럴/`s[i]`/`s.len()` codegen (독립 `fixpoint_str.nl`). 2-pass(strlit→`@.sN`
      전역 + 슬롯). `s[i]`=GEP i8+load i8+zext, `s.len()`=컴파일타임 길이. backtick(96) 구분자. 버그:
      skip_factor `.len()` off-by-one(i+4→i+5). e2e 7 PASS. 값-정확성 44/44. (실제 소스 토큰화 프리미티브.)
- [x] **FP12d** 문자열을 통합컴파일러 fixpoint_full에 **병합** — 함수/가변/while/if/배열/List/struct/putchar/
      **문자열**을 단일프로그램 codegen(top+함수본문). 6-stage 체크포인트(tokenize backtick→kind28 / isarr_of
      0/1/2/3=string + Slot필드재활용 / compile() pre-pass emit_str_globals 모듈톱 전역 nstart키 / 슬롯수집기
      2곳 rhs.kind==28 / gen_factor `.`/`[` string우선 / skip_factor paren skip). 버그: gen_stmts string-RHS
      scalar-fallthrough가 i8* alloca를 `store i64 0` 오염(고정값139)→skip. **🎯 토큰화코어 `fn tok(){let s=...;
      let xs=list(); while i<s.len(){xs.push(s[i])...}}`=74**(fixpoint.nl 토크나이저와 같은 shape). e2e 23→30 PASS.
- [x] **FP12e** **동작하는 토크나이저 실증** — string+if/while/state-var 조합이 완전한 렉서. `digits()` 중첩if
      문자클래스, **🎯 `ntok()` "ab cd ef" 토큰런=3**(if/else+중첩if+in-word 상태머신 over 문자열스캔=실제렉서).
      recon만으로(추가 codegen 0). e2e 30→32 PASS. 값-정확성 44/44.
- [x] **FP12f** **파서 코어 실증** — name_eq(두 바이트범위 비교) + 키워드인식. `eq()` "let"=="let"=1,
      **🎯 `kw()` 키워드인식**: `s.len()==3` + 바이트비교 중첩if → "let"=7(실제 렉서 kw3/kw5 패턴). recon만으로.
      e2e 32→35 PASS. **통합컴파일러가 토크나이저+파서 키워드인식 둘 다 codegen = 가장 어려운 문자열-의존
      두 부분 완비. 남은 갭=순수 규모(코드 양), 능력 아님.**

**🎯🎯🎯 codegen 능력 완성 (2026-06-06)**: 통합 컴파일러 fixpoint_full.nl이 nl 컴파일러를 구성하는 **모든 핵심
구문**(함수 0-4param/재귀/중첩 + 가변변수 + while + if/else + 배열 + 동적List + struct + putchar + 문자열 s[i]/
s.len())을 단일 프로그램에서 네이티브 LLVM IR로 생성. **동작하는 토크나이저(ntok)와 파서 키워드인식(kw)을 실증.**
self-host 핵심 능력 전부 달성. 남은 갭 = **순수 규모**(실제 수천줄 컴파일러 소스를 먹이는 months급 부트스트랩),
능력/통합 부족 아님. → 다음 = 실제소스 부트스트랩(months급, TRACKED) 또는 핵심 인프라 추가 다지기.

## TRACKED (능력 완비, 순수 규모 블로커)
- **실제 nl 컴파일러 소스 부트스트랩**: fixpoint_full이 전 코어구문 + 토크나이저/파서 shape를 codegen하나,
  실제 컴파일러 소스는 수천 줄(깊은 중첩 + 대량 함수). 통합본에 그대로 먹여 emit IR이 재현하려면 months급
  엔지니어링. **능력/통합은 완비**(FP12f까지 실증) — 순수 코드량 문제. /loop는 여기서 인프라 다지기로 전환.
