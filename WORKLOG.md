# nl WORKLOG

## 2026-06-08 (/loop: FP12dd — 🎯🎯🎯🎯 완전한 함수언어 컴파일러 end-to-end = 3 tier 전부 완성)
- **마일스톤(신규 능력 0)**: fixpoint3.nl 함수언어 컴파일러를 단일 통합 프로그램으로 먹임 — `run_program(src)`가 함수언어 프로그램
  토큰화(fn/return 키워드+ident+num+`+ * ( ) {{ }} ;`)→`fn` 정의 스캔으로 `List<Fn>` 함수테이블 빌드(build_fns)→top-level
  `return <call>;` 찾아 eval_call(인자를 fresh callee `List<Var>` scope 바인딩+body 평가).
- 실측 end-to-end(source string→value): **`fn double(x) {{ return x * 2 }} return double(21)`=42, `fn sq(x) {{ return x * x }} return sq(7)`=49,
  2-fn 테이블 두번째호출 `triple(14)`=42**(build_fns 다중스캔+find_fn 디스패치).
- FP12cc(List<Fn>/eval_call/fresh scope)+키워드토큰화 조합=신규 능력 0. e2e fixpoint-full **123→126**(+3 가드), 값정확성 96/96, 회귀0.
  commit 21c9483.
- **🎯🎯🎯🎯 세 self-host tier(①산술 ②산술+변수 ③함수) 전부 단일 통합 프로그램 source→value end-to-end.** FP12g~dd.
- **다음 경계(tracked)**: run_program 재귀는 함수 body에 `if cond then a else b` **식**(expression) 필요(FP12cc 재귀 probe는 native nl
  재귀였음; source-level `fac(n) = if n<=1 then 1 else n*fac(n-1)`는 if-expr 평가기=별개 갭). 다음=if-expr in eval(재귀 함수언어) or
  `-> List` dedicated or for/print or TRACKED 버그.

## 2026-06-08 (/loop: FP12cc — 함수+재귀, fixpoint3.nl 3번째 tier)
- fixpoint3.nl tier(multi-char 함수정의/호출+재귀)로 확장. fixpoint_full이 핵심 메커니즘 codegen.
- 격리 probe(컴파일러 변경 0): find_fn(`List<Fn>` 이름룩업→index 1), **eval_call**(함수 룩업+인자를 fresh per-call
  `List<Var>` scope 바인딩+body lookup 평가, double(5)=10), **재귀**(함수 body가 자기 호출, 매 호출 fresh scope:
  sum(5)=15, **factorial(5)=120**).
- **= fixpoint_full이 3개 self-host tier(①산술 ②산술+변수 ③함수+재귀) 아키텍처 전부 codegen 가능 확증.** `List<Fn>`+`List<Var>`
  테이블+fresh-scope 바인딩+재귀 전부 기존 기능 조합=신규 능력 0.
- e2e fixpoint-full **120→123**(+3 가드), 값정확성 96/96, 회귀0. commit f890a43.
- 교훈: 3 tier 모두 동일 빌딩블록(List-of-structs 테이블+name_eq lookup+재귀+fresh scope)으로 조합=능력완비. FP12g~cc.
  남은 fixpoint.nl 갭=`-> List` 직접반환/for/print/TRACKED 2건. 다음=fixpoint3 run_program 전체통합(source→fn정의+호출→값) or
  `-> List` dedicated or for/print or TRACKED 버그.

## 2026-06-07 (/loop: FP12bb — 🎯🎯🎯🎯 완전한 산술+변수 컴파일러 end-to-end)
- **마일스톤(신규 능력 0)**: fixpoint2.nl 전체를 단일 통합 프로그램으로 먹임 — `run_program(src)`가 multi-let 프로그램 토큰화
  (kw_let/kw_return 키워드인식+ident+num+`+ * = ;`)→`let` 바인딩으로 `List<Var>` 심볼테이블 빌드(변수가 이전 변수 참조 가능)
  →precedence로 평가→`return`값.
- 실측 end-to-end(source string→value): **`let x = 5; return x + 3`=8, `let a = 4; let b = 6; return a * b`=24,
  `let x = 2; let y = x + 1; return x + y * 4`=14**(y가 x 참조+precedence).
- FP12z(심볼테이블)+FP12aa(변수평가)+키워드토큰화 조합=**신규 컴파일러 능력 0**. e2e fixpoint-full **117→120**(+3 가드),
  값정확성 96/96, 회귀0. commit 185e75e.
- **🎯🎯🎯🎯 두 번째 self-host tier 완성: multi-char 변수 있는 언어의 가장 작은 완전한 self-host 컴파일러 end-to-end.**
  FP12g~bb. 남은 fixpoint.nl 갭=`-> List` 직접반환/for/print/TRACKED 2건. 다음=fixpoint3(multi-char 함수+재귀=3번째 tier) or
  `-> List` dedicated or for/print or TRACKED 버그.

## 2026-06-07 (/loop: FP12aa — 변수 평가, eval_factor가 ident를 lookup으로 해석)
- **마일스톤(신규 능력 0)**: fixpoint2.nl 변수-해석 evaluator를 fixpoint_full에 먹임 — eval_factor가 num은 t.value,
  ident면 `lookup(vars,...)`; `vars: List<Var>` 심볼테이블이 전 재귀체인(eval_expr→eval_fold→eval_term→eval_factor→lookup) 관통.
- 실측(변수 피연산자+precedence): **`x + 3`(x=5)=8, `a * b`(a=4,b=6)=24, `x + y * 4`(x=2,y=3)=14**.
- 컴파일러 변경 0 — 초기 probe 실패(got 2/5)는 **test의 eval_expr off-by-one**(`eval_term(toks,0,..)` 썼으나 실제 fixpoint2.nl은
  `eval_term(toks, i+1,..)`+eval_fold 피연산자 `i+2`)뿐. faithful 인덱싱이면 깨끗이 조합 = deep `List<Var>` param 관통+ident-token 해석 동작.
- e2e fixpoint-full **115→117**(+2 가드), 값정확성 96/96, 회귀0. commit 10f9cc4.
- 교훈: probe 실패는 컴파일러 갭 vs **test 로직 오류** 구분 필요(실제 self-host 소스의 정확한 인덱싱 대조로 규명)/이번엔 test off-by-one.
- **= fixpoint2.nl(산술+변수 tier) 변수평가 핵심 동작.** FP12g~aa(18 능력추가+2 통합마일스톤). 남은 fixpoint.nl 갭=`-> List` 직접반환/for/print/TRACKED 2건.
  다음=fixpoint2.nl run_program 전체 통합(let 바인딩+return, source string→값) or `-> List` dedicated or for/print.

## 2026-06-07 (/loop: FP12z — `!=` 연산자, name_eq + fixpoint2.nl 심볼테이블 unblock)
- 다음 큰 모듈 fixpoint2.nl(산술식+multi-char 변수, `List<Var>` 심볼테이블)로 확장 → lookup이 모든 이름에 -1(255) 반환.
- IR 격리: **`!=`가 토크나이저에 완전 누락** — `!`(33) skip되고 뒤 `=`가 assignment(kind5)로 처리 → `src[a+k] != src[b+k]`가
  `icmp ne ..., 0`로 lowering(RHS `src[b+k]` 통째 드롭). name_eq(소스바이트 동등=심볼테이블 키비교)와 lookup 무음파손.
- **fix: 토크나이저 `!`+`=`→not-equal 토큰(kind 33)**(bare `!`은 skip 유지), **gen_fold 비교 arm에 kind33 추가→`icmp ne`**.
- 실측: `!=` value/조건, name_eq(foo==foo→1, foo!=bar→0), **🎯 실제 fixpoint2.nl 심볼테이블(`List<Var>` name_eq lookup: "foo"→10, "bar"→20)**.
  e2e fixpoint-full **112→115**(+3 가드), 값정확성 96/96, 회귀0. commit a6c4882.
- 교훈: 다음 모듈로 확장이 누락 연산자(`!=`) 노출/**`!=` 누락은 무음파손**(RHS 드롭→`!=0`, 컴파일은 됨)/IR 격리로 RHS-드롭 규명.
- **= fixpoint2.nl(다음 self-host tier: 산술+변수) 핵심 심볼테이블 동작.** FP12g~z(18 능력추가). 남은 fixpoint.nl 갭=`-> List` 직접반환/for/print/TRACKED 2건.
  다음=fixpoint2.nl 더 큰 조각(eval_factor가 lookup 호출=변수 평가) or `-> List` dedicated or for/print.

## 2026-06-07 (/loop: FP12y — 🎯🎯🎯🎯 완전한 self-host tokenize+eval 파이프라인 end-to-end)
- **마일스톤(신규 능력 아님)**: fixpoint.nl의 tokenize+eval 컴파일러를 **단일 통합 프로그램**으로 fixpoint_full에 먹임.
  `run(src: Str)`가 식 문자열→`List<Token>` 토큰화(out-param)→evaluator로 평가(List<Token> 재귀)→값 반환.
- FP12u(typed let)+FP12v(bool)+FP12w(tokenize)+FP12x(evaluator)가 깨끗이 조합됨 — **신규 갭 0**(능력 이미 갖춰짐).
- 실측 end-to-end(source→tokenize→eval→value): **`2+3*4`=14**(precedence), **`10 - 2 * 3`=4**(좌결합+precedence+공백),
  **`7 + 8 + 9`=24**(좌-fold). = 가장 작은 완전한 self-host 컴파일러(산술식 토큰화+평가)가 단일 프로그램으로 실행.
- e2e fixpoint-full **109→112**(+3 통합 파이프라인 가드), 값정확성 96/96, 회귀0. 신규 컴파일러 변경 0(가드만). commit c78dd2e.
- **🎯🎯🎯🎯 실제 소스 부트스트랩 arc 목표 달성: 가장 작은 완전한 self-host 컴파일러가 fixpoint_full로 컴파일된 단일 프로그램 end-to-end 실행.**
  FP12g~y(17 능력추가, FP12y는 통합 마일스톤). 주의: e2e 백그라운드 동시실행은 `/tmp/.vais-cache` 공유로 카운트 interleave→단일 실행으로 측정.
- 남은 fixpoint.nl 갭(편의/비-blocking)=`-> List` 직접반환(#4 우회)/`for`(1곳)/print interp(3곳). TRACKED 2버그(task chip)=deep else-if 빈 merge / multi-term식.
  다음=`-> List` 직접반환 dedicated(원형 복원) or for/print or TRACKED 버그 or 더 큰 모듈(fixpoint2/3 multi-char vars).

## 2026-06-07 (/loop: FP12x — 🎯🎯🎯 실제 fixpoint.nl evaluator 통째 컴파일 (let t=toks[i] LOS-원소 바인딩))
- fixpoint.nl 재귀 evaluator(eval_term/eval_expr/eval_expr_fold/skip_term, `List<Token>` param 재귀) 먹였더니 invalid IR(%v-N).
- IR 격리: **`let t = toks[i]`가 List-of-structs 원소를 local에 바인딩** 후 `t.kind`/`t.value` 필드읽기 — let-핸들러가 `toks[i]`를
  scalar RHS로 취급→`t`가 i64 1슬롯, `op.kind`가 인접(틀린)슬롯 읽음. 핵심 eval패턴 `let t=toks[i]; if t.kind==2`. 직접 `toks[i].kind`는 이미 OK.
- **fix: rhs_los_elem_sty()**(RHS가 `<listvar>[expr]`+listvar가 LOS면 원소타입)+slot 할당이 struct local([nf x i64], sty=원소타입)+
  gen_stmts가 원소복사(필드k: `t[k]=buf[idx*nf+k]`, param이면 버퍼ptr 먼저 load)+add_local_slots서 `&slots` 전달(owned라 by-value면 E022 move).
- 실측: local/param LOS-원소 바인딩+필드, **🎯 실제 fixpoint.nl evaluator(`2 + 3 * 4`=14 precedence, `10 - 2 * 3`=4 좌결합+precedence)**.
  e2e fixpoint-full **105→109**(+4 가드), 값정확성 96/96, 회귀0. commit d5c40ce.
- 교훈: 실제 eval 먹이기가 `let t=toks[i]` 바인딩 갭 노출(직접접근과 별개 경계)/owned slots는 `&` 전달 필수(E022)/IR 격리로 슬롯 오정렬 규명.
- **🎯🎯🎯 fixpoint.nl tokenize+evaluator 양쪽 완전 컴파일+실행 = 가장 작은 완전한 self-host 컴파일러의 핵심 동작.** FP12g~x(17 능력추가).
  남은 fixpoint.nl 갭=`-> List` 직접반환(#4 우회)/`for`(1곳)/print interp(3곳)/emit_ir(putchar 이미 동작). 다음=fixpoint.nl 전체(tokenize+eval+
  emit_ir+main) 통합 먹이기 or 남은 갭 or TRACKED 2버그.

## 2026-06-07 (/loop: FP12w — 🎯 실제 fixpoint.nl 토크나이저 통째 컴파일 (else-if 체인 in-loop))
- fixpoint.nl 실제 tokenize(out-param 형: 4 token kinds, is_space/is_digit 헬퍼콜, 6-way `if/else if.../else` 디스패치)
  먹였더니 오답(count 7 vs 5, garbage 값).
- IR 격리: **`if A {{}} else if B {{}} else {{}}` 체인이 while 본문서 오lowering** — if-핸들러가 `else if`를 plain `else`로
  취급→`{{` 스캔이 inner then-block에 착지→else-region이 그 블록만 덮음→마지막 `else`(`go=false`) 무조건 실행(루프 1회).
  HEAD서 3-way도 깨짐(111 기대에 65)=회귀 아닌 잠복버그(내 fix가 net 개선).
- **fix: if_stmt_end() 헬퍼**(완전한 `if [cond] {{}} [else if {{}}]* [else {{}}]` 체인 끝 인덱스, `else if` 재귀)+
  if-핸들러가 `else if`면 else-body를 nested if statement로 gen_stmts 재귀(ebody_start/end/resume 추적).
- 실측: 3-way else-if in-loop(65→3), else-if call-cond, **🎯 실제 fixpoint.nl 토크나이저**(`12 + 3 * 4`→5토큰 값12+3+4=19,
  `99*100`→199, 단일`5`→kind0 value5, `1 + 2 - 3 * 4`→7토큰). e2e fixpoint-full **101→105**(+4 가드), 값정확성 96/96, 회귀0.
  commit efb1e94.
- 교훈: **실제 self-host 함수 통째가 진짜 마일스톤**(tokenize 완전동작=fragment 아님)/IR 격리로 else-region 오착지 규명/
  pre-existing 버그도 fix가 net 개선.
- **TRACKED(pre-existing, 별개, 비-blocking)**: ①3+레벨 nested else-if 모든 branch가 return시 빈 trailing merge block(invalid IR)
  ②`.len`+`[i].field` 혼합 multi-term 식(`toks.len*100+toks[0].value+...`) 오계산. 둘 다 실제 tokenize엔 영향 없음.
- **🎯🎯🎯 실제 fixpoint.nl tokenize 함수 완전 컴파일+실행 = 부트스트랩 핵심 진전.** FP12g~w(16 능력추가). 남은 fixpoint.nl 갭=
  `-> List` 직접반환(#4 우회)/for(1곳)/print interp(3곳)/위 TRACKED 2건. 다음=eval_term/expr 먹이기 or TRACKED 버그 or 남은 갭.

## 2026-06-07 (/loop: FP12v — boolean 리터럴 true/false, 실제 digit-run 토크나이저 동작)
- 다음 갭 probe: `-> List` 직접반환 clang 스킴 검증완료(run=42, caller-allocated buffer+callee hidden out-ptr)이나
  구현이 큰 dedicated(local aliasing 등)라 우회 존재로 보류 → 대신 **fixpoint.nl 실제 tokenize 로직 통째 먹이기**로
  다음 경계 탐색.
- **실제 multi-digit tokenize**(nested `while go {{ ...; go = false }}`로 digit run을 `v=v*10+(d-48)` 한 토큰 누적)
  → %v-1. 격리: **`true`/`false`가 식별자(kind1)로 토큰화→gen_factor 변수로딩 fall-through**→`let mut go=true`가
  `load %v-1`. (`while go` bare-var는 이미 OK=초기 의심 틀림, probe가 매번 `let mut go=true` 동반해 혼동.)
- **fix: gen_factor가 `true`/`false`를 정수 1/0으로 인식**(nl bool=i64), 변수로딩 전. let+assignment 양쪽 커버.
- 실측: bool-flag while=5, true/false-in-if=42, **실제 digit-run 토크나이저(`12a34`→토큰 12+34)=46**.
  building block(digit-run i/v advance, else-if 분기) 독립확인. e2e fixpoint-full **97→101**(+4 가드),
  값정확성 96/96, 회귀0. commit 3f3fe76.
- 교훈: **실제 self-host 로직 먹이기가 진짜 갭 노출**(true/false=digit-run/scan 루프 필수)/초기 의심은 격리로 정정.
  **남은 fixpoint.nl 갭=`-> List` 직접반환(#4 우회존재, 스킴검증완료)/`for`(1곳)/print interp(3곳).** FP12g~v(15 능력추가).

## 2026-06-07 (/loop: FP12u — 실제 소스 부트스트랩 착수 + 첫 갭 해결: typed let)
- **사용자 결정**: 실제 소스 부트스트랩 본격 착수(능력 다지기 loop 대신).
- **recon**: self-host 모듈 측정 → fixpoint.nl=136줄=가장 작은 완전한 컴파일러=첫 타깃. fixpoint.nl을 fixpoint_full에
  먹이는 첫 경계 매핑: **`&List` borrow + 재귀 + `&xs` call-site는 이미 동작**(갭 아님, 격리확인=fragment 추정과 다름)/
  진짜 첫 블로커 = **typed let `let mut toks: List<Token> = []`**(`let x: Int = 42`조차 %v-1 — 타입주석이 RHS 위치 어긋나게 함).
- **fix = rhs_pos() 헬퍼**: name 뒤 `: Type` 주석(List<...>는 `>`까지) 건너뛰고 RHS 위치 반환. 3 let 핸들러
  (add_local_slots/gen_stmts/collect_top_slots) 전부 적용+다운스트림 npos+2/+3→vp. 추가: rhs_is_list가 `[]`(kind23+24)
  빈 리스트 인식/let_anno_elem_sty=`: List<Type>` 주석서 원소타입(빈 리스트 authoritative)/rhs_struct_type도 rhs_pos.
- 실측: typed scalar/mut/empty-list-of-structs/list()-of-structs/struct-lit/scalar-list/top-level + **실제 self-host
  shape(typed-list 토크나이저 `let mut toks:List<Token>=[]`+push+consume)=6**. e2e fixpoint-full **90→97**(+7 가드),
  값정확성 96/96, 회귀0. commit 569cb51.
- 교훈: **recon이 실제 갭 정밀식별**(`&List`는 이미 OK, typed-let이 진짜 첫 블로커)/typed-let은 RHS 위치 단일근(rhs_pos)으로
  3 핸들러 일괄. **남은 fixpoint.nl 갭=`-> List` 직접반환(#4, out-param 우회)/`for`(1곳)/print interp.** 다음=다음 갭 dedicated 또는
  fixpoint.nl 더 먹이기.

## 2026-06-07 (/loop: FP12t — full tokenize→eval 파이프라인, List<Token> 함수간 공유)
- **🎯🎯 완전한 self-host tokenize→parse/eval shape 컴파일**: tokenize가 `List<Token>` out-param 채움 →
  **별도 consumer 함수**가 그 `List<Token>`을 param으로 받아 `toks[i].kind`로 디스패치. 여러 consumer가 한 List 공유.
- 경계 probe: 2-함수 파이프라인(tokenize→eval) = 139(garbage) 발견.
- **근본수정 = read-only List-of-structs param 추론**: `eval(toks: List<Token>)`는 읽기만(push 없음) →
  emit_fn이 본문 push-scan(list_elem_sty)으로 원소타입 못 찾아 scalar 처리(`.len`=buf[63], stride 1)→garbage.
  수정: **emit_fn이 param 자신의 `List<Type>` 주석에서 원소타입 읽음**(param_list_elem_sty를 자기 함수+param 위치;
  push하든 읽든 authoritative), push-scan은 fallback(List<Int>).
- 실측: tokenize→eval sum=6, mini calc 디스패치=9, **2 consumer(count_nums+sum_nums) 한 List 공유=211**.
  e2e fixpoint-full **87→90**(+3 가드), 값정확성 96/96, 회귀0. commit ff72f0e.
- 교훈: read-only param은 push-scan 불가→**param 주석이 authoritative**(consumer 함수=self-host 핵심 패턴, parser가
  toks 받는 모양). **🎯 self-host tokenize→parse 파이프라인 전체(tokenize out-param→consumer가 List<Token> param 디스패치) 완전 컴파일.**
  FP12g~t(13 능력추가). 다음=더 큰 통합(parser가 toks 소비하며 struct AST 빌드) 또는 실제 소스 부트스트랩.

## 2026-06-07 (/loop: FP12s — List-of-structs PARAMETER, #5b 해결 = 실제 토크나이저 full shape)
- **🎯🎯 #5b 해결**: 실제 self-host 토크나이저 full shape 컴파일 — `fn tok(s: Str, out: List<Token>)`가
  string 스캔→Token struct를 by-pointer out-param List에 push, 호출자가 `toks[j].kind`/`toks[j].value` 소비
  (=self-host `tokenize(src) -> List<Token>`을 out-param으로 재작성, List-return 우회 패턴).
- **length-slot 재설계**(clang 검증 run=179, 40원소): struct-원소 List=`[64*nf+1 x i64]`, length를 데이터 뒤
  header `buf[64*nf]`에 저장(scalar 무변경 buf[63]). 4 버퍼 사이트 전부 `64*nf+1` 타입일치.
- **callee(is_arr=4,sty>=0)**: emit_fn이 list_elem_sty로 param 원소타입 추론, push=`ptr[len*nf+fi]`/
  index=`ptr[i*nf+fi]`/`.len`=`ptr[64*nf]`.
- **caller cross-function 추론**: 호출자 본문에 push 없어 원소타입 불가시 → call_arg_elem_sty+param_list_elem_sty가
  callee의 `List<Struct>` param 주석서 추론(arg 위치 매칭). pre-call write+sync_list_len stride-aware化.
- **🎯 트랜스파일러 근본수정**: expand_for_loops(line기반)가 while/for 본문끝 brace-count시 **`#` 주석 brace도 셈**
  → 루프 본문 주석에 unbalanced `{`("... Type {") 있으면 `}` 더 소비→stray brace→빌드깨짐. 주석서 brace 제거.
- 실측: empty-fill-len=2, caller-reads-fields=31, callee-reads-own=100, **full tokenizer=3**.
  e2e fixpoint-full **83→87**(+4 가드), 값정확성 96/96, 회귀0. commit cb81aaa.
- 교훈: clang 스킴 먼저(run=179)/length를 데이터 뒤로(stride 충돌제거)/caller 추론은 callee param 주석서(struct
  필드 추가 회피)/**트랜스파일러 brace-count가 주석 포함=루프 본문 주석 brace 금지**(진단: q.vais depth -1+main 뒤 stray }).
- **🎯🎯🎯 부트스트랩 갭 #1~#5b 전부 해결 = self-host 토크나이저 full shape(string param→List<Token> out 채움→field 소비) 컴파일.**
  FP12g~s(12 능력추가). 다음=더 큰 self-host 통합(tokenize+parse가 List<Token> 공유) 또는 실제 소스 부트스트랩.

## 2026-06-07 (/loop: FP12r — List-of-structs, 부트스트랩 갭 #5 로컬 해결)
- 경계매핑: 다함수 통합 미니컴파일러(tokenize digits→eval sum, List를 함수간 전달)는 동작(=10).
  더 어려운 모양 probe → **List-of-structs**(`toks.push(Tok{...})` 통째 struct push)서 invalid-IR 경계 발견.
- self-host grep: tokenize가 `toks.push(Token{kind,value,nstart,nlen})`로 **4-필드 Token을 List에 통째 push**
  ~30곳, parser가 `toks[i].kind`로 소비 = 진짜 `List<Token>` 모양 → **bootstrap-critical 갭 #5**.
- clang 스킴 격리검증(2건): ①연속 struct 버퍼 stride=필드수(run=21) ②dynamic-index field read(run=60).
- **구현(FP12r)**: struct-원소 List=`[64*nf x i64]` 버퍼, slot.sty=원소 struct-type.
  - `list_elem_sty()`: list() 시점 첫 `name.push(Type{...})` 스캔으로 원소타입 추론.
  - slot 할당(add_local_slots+collect_top_slots), push(각 필드 `buf[len*nf+fi]`), index+field(`buf[i*nf+fi]`).
  - **버그2 근본수정**: ①라우팅 — struct-필드-쓰기가 List-of-structs를 가로챔(asti>=0) → is_arr=0(scalar struct)
    한정, List(is_arr=2)는 push로 fall-through. ②skip_factor — `name[idx].field`(6토큰)를 1 factor로 확장
    안 해 뒤 `+` 피연산자 누락 → bracket_end 뒤 `.`면 +2 토큰.
- 검증: push/len=1, multi-term field-read=21, dynamic build→consume=6, 4-필드 Token=153, **40-원소 스케일=139**
  (별도 length alloca라 buf[63] 충돌 없음). e2e fixpoint-full **79→83**(+4 가드), 값정확성 **96/96**, 회귀0.
- commit ae107ac. **로컬 List-of-structs = parser/eval 부트스트랩 직결 핵심 능력 완비.**
- **#5b TRACKED**(다음 1순위): List-of-structs PARAMETER(`fn tok(s:Str, out:List<Token>)`)=실제 토크나이저 full
  shape. clang 스킴 검증완료(run=23, i64* 버퍼+struct stride). 원소타입은 emit_fn서 list_elem_sty 재사용(struct
  필드 추가 불필요). **남은 설계 = length-slot 충돌**(scalar param은 len@buf[63]이나 stride>1이면 원소31 슬롯과 충돌
  31*2+1=63) → len을 buf[64*nf]로 이동+sync/push/read 통일 필요. dedicated iter 대상.

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
- **자기수정 루프 실증 중 발견·정정**: rusty draft(Vec/i32/vec!/&&/.to_string())에 nl-check가 5개 다 help:로
  잡음 ✅. 그러나 fix 적용 후 빌드하니 **`.to_string()`→`Str(x)` 조언이 깨진 코드 생성**(transpiler가 `str(x)`
  emit→P001; Vais 표면 int→string 변환 부재 실측: str()/to_string()/method 전부 실패). **정정: nl-check가
  `Str(x)` 작동 약속하면 안 됨**(또 다른 "두 길" — 깨진 길). `.to_string()` 규칙을 "nl엔 .to_string() 없음 +
  표면변환은 TRACKED Vais갭"으로 변경(대체 약속 제거). ROADMAP TRACKED에 Vais int→string 부재 추가.
  **핵심 교훈: 에러인프라의 조언은 반드시 동작하는 fix여야**(자기수정 루프를 끝까지 빌드검증 — 조언이 깨진
  코드 만들면 그 조언이 P1-P3 모호성 위반). nl-check 26/26 유지(여전히 flag, help만 정직화).

## 2026-06-07 (/loop iter 50: cold-start 자기수정 루프 실측 [P4 결정적 검증] + 복합대입 규칙)
- **P4 핵심 명제 실측**: nl 에러인프라가 AI 자기수정을 가능케 하는가 — 신선 서브에이전트로 측정.
- **cold-start 첫시도 성공 2건**(능력검증): general-purpose 서브에이전트(nl 처음, 코퍼스만)에게 ① sum_evens(짝수합,
  `%` 모듈로/`<=`/중첩if 추론)=30 ② squares 리스트합(`List<Int>` annotated + `.sum()`→fold)=30 — **둘 다 nl-check
  clean + 빌드 + 실행 첫시도 성공**(자기수정 불필요). 코퍼스가 typical 과제엔 충분.
- **🎯 자기수정 루프 수렴 실측**(P4 결정적): rusty draft(`i32`×3 + 트레일링`;` + `+=`)를 의도 작성 →
  **R1 nl-check가 i32 3건 help:로 flag** → 신선 서브에이전트가 help 적용 + **코퍼스에서 `;`/`+=`도 자가추론 수정**
  → **R2 nl-check clean + 빌드 + 실행=30 = 1라운드 수렴**. nl 에러인프라+코퍼스 = **단일라운드 자기수정** 입증.
- **부수발견 → 신규 규칙**: 서브에이전트가 `+=`를 코퍼스로 자가수정(nl-check 미flag였음). `+=` 실측 P001 확인 →
  **복합대입 규칙 추가**(`+= -= *= /= %=`→`x = x + e`). **트레일링 `;`는 비-flag 결정**(실측: `;`는 nl의 정상
  문장구분자 — multiline 트레일링 `;` BUILDS). 정밀 regex `[+\-*/%]=(?!=)`로 `<=`/`>=`/`!=`/`==`/음수 비발화.
- 오탐0(49예제+17모듈 clean + `<=`/`>=`/`!=`/`==`/음수대입 clean), unit 26→34. 카탈로그 13→14종.
- 검증: nl-check 34/34, 값-정확성 49/49, 회귀0. 교훈: **P4 명제 실측=신선 서브에이전트 자기수정 라운드 카운트**
  (1라운드 수렴)/cold-start가 typical 과제엔 첫시도 성공(코퍼스 강함)/nl-check 미flag도 코퍼스가 보완(이중 안전망)/
  서브에이전트 자가수정이 새 규칙 후보 노출(`+=`)/`;`는 flag 금지(정상 구분자, 실측이 추측 정정).

## 2026-06-07 (/loop iter 51: P9 코퍼스 미커버 영역 +6 [enum/Result/Option/struct 조합])
- codegen 능력완비 후 비차단 인프라 계속 — P9 코퍼스를 **미커버 영역**으로 확장. 후보를 트랜스파일+빌드로
  먼저 검증 후 동작하는 것만 채택(코퍼스=valid-form 권위).
- **신규 검증 예제 6종**: e21 Result Ok/Err match(7) / e22 enum 디스패치 match-in-helper(12, 작은 인터프리터
  패턴) / e23 Option 흐름 lookup→match→합산(15) / e24 struct 필드가 enum + match(1, 모델링 shape) /
  e25 for+if 필터합산(12, `.filter()` Vais갭 대안 수동필터) / e26 함수합성 파이프라인 dec(dbl(inc(9)))=19.
- 이전 코퍼스는 Option(e08/e16)/enum payload(e02)는 있었으나 **Result match / enum 디스패치 / Option 다중흐름 /
  struct+enum 조합 / for-필터 / 함수합성**은 미커버였음. AI가 자주 쓰는 조합 패턴 보강.
- 검증: 값-정확성 **49→55 PASS 회귀0**, nl-check 6 신규 clean. README 인덱스 e20→e26 + 카운트 37/37(러너55/55).
  교훈: 후보는 빌드선검증(expect 오산 2건 — pipeline 19, 빌드가 정답 알려줌)/미커버는 "있는 것의 변주"가 아니라
  "조합/흐름"에 있음(Result match, struct+enum)/`.filter()` Vais갭은 for+if 수동필터가 코퍼스 권장 대안.

## 2026-06-07 (/loop iter 52: P9 +2(e27/e28) + 트랜스파일러 중첩리스트 추론 수정 + Vais갭 2종 추적)
- 코퍼스 확장 중 harder 후보로 **Vais 백엔드 갭 2종 발견·격리·추적**(실측 우선).
- **트랜스파일러 중첩리스트 타입추론 수정**: `[[1,2],[3,4]]`가 `Vec<i64>`로 잘못 추론(E001 "expected i64 found
  [i64]")되던 것을 **재귀 추론으로 `Vec<Vec<i64>>`** 생성. `_vec_elem_from_annotation`(List<List<Int>> 보존),
  `_split_top_commas`(depth-0 분할), `_infer_vec_elem`(첫원소 `[`면 재귀) 신설. 트랜스파일러 unit +2(24→26).
- **단, 런타임은 Vais 백엔드 버그로 막힘**(타입수정 후 빌드는 되나 값 garbage 139/192 → 하드코딩 Vais도 C003
  Type error 확인). **Vais 중첩 Vec codegen 버그 = TRACKED**(nl 아닌 Vais). 트랜스파일러 수정은 정당(Vais가
  고치면 즉시 동작) + 회귀0.
- **격리 발견 2: 리스트-리터럴 직접 인자 갭**: `maxof([3,9,2,7])` E001이나 `let v=[..]; maxof(v)` OK. Vais
  코어션 갭(literal→Vec param). 기존 d4b도 bound-var라 통과했던 것. TRACKED + 코퍼스는 bound-var 권장.
- **신규 검증 예제 2종**(동작 확인): e27 List param max(bound-var 전달, 9) / e28 struct 함수적 갱신(rebuild+
  재대입, 8). 값-정확성 55→57. README 인덱스 e26→e28 + 미커버에 중첩리스트/리터럴직접인자 갭 명기 + ROADMAP
  TRACKED 2종 추가.
- 검증: 값-정확성 57/57, 트랜스파일러 unit 26/26, nl-check 34/34, 회귀0. 교훈: **harder 후보가 백엔드 갭을
  노출**(중첩Vec, 리터럴인자)/실측으로 nl측(트랜스파일러 추론 수정) vs Vais측(codegen) 정확 구분/타입추론 수정은
  Vais가 못 고쳐도 정당(미래대비+회귀0)/우회 패턴(bound-var)을 코퍼스 예제로 문서화(e27).

## 2026-06-07 (/loop iter 53: P9 +2(e29/e30) + Vec 성장(push/map) Vais갭 발견·PRELUDE 정정)
- 코퍼스 확장 중 **closure+`.map()` 후보가 중대 Vais 갭 노출** → 실측·격리·PRELUDE 정정.
- **🚨 발견: Vais Vec 성장(push/map) 무음 miscompile/빌드실패**. `.map()`은 `@Vec_push` undefined 빌드실패,
  리터럴 Vec `.push(x)`는 **무음 miscompile**(빌드OK but len 오염, garbage 134). delimit: **read-only
  len/index/fold/sum은 OK**(10,20,30→len3/index40/fold60 확인), **성장(push/map)만 깨짐**. PRELUDE가 push/map을
  ✅로 잘못표기 → **🔴 정정**(B-02 `@HashMap_new`와 동일 monomorphization 클래스). `.filter()`는 기존 TRACKED.
- **핵심 통찰**: nl self-host **codegen 트랙**(fixpoint_list.nl)은 List push가 동작 — Vais Vec 안 쓰고 **고정버퍼
  (`alloca [64 x i64]`+수동 length)로 직접 구현**해 이 갭을 우회하기 때문. **transpiler 트랙**(코퍼스)만 Vais Vec
  성장에 막힘. 두 트랙의 List 구현이 다름을 명확히. 리스트 변형은 `for`-루프 누적(e25/e27)이 코퍼스 권장.
- **신규 검증 예제 2종**: e29 GCD(2-인자 재귀+modulo, gcd(48,18)=6) / e30 enum payload match(`Circle(r)=>r*3`, 42).
  값-정확성 57→59. README 인덱스 e28→e30 + 미커버에 Vec성장 갭, PRELUDE push/map 🔴, ROADMAP TRACKED 추가.
- 검증: 값-정확성 59/59, 회귀0. 교훈: **PRELUDE ✅ 표기도 실측으로 검증**(push/map 실제론 깨짐)/무음 miscompile은
  빌드OK라 더 위험(런타임 값까지 확인 필수)/**같은 기능도 트랙마다 구현 다름**(codegen=고정버퍼 OK vs transpiler=Vais
  Vec 깨짐)/read vs 성장 구분이 갭 경계/expect 오산은 빌드가 정답(closure_fold 20).

## 2026-06-07 (/loop iter 54: nl측 수정 — bitwise 2-인자 매핑 + map_bitnot 문자열버그 + P9 +3)
- **드디어 nl측 수정 가능한 갭 발견**(이전 iter들은 대부분 Vais 갭). PRELUDE가 `bitand/bitor/bitxor/shl/shr`
  약속하나 트랜스파일러는 `bitnot`만 매핑 → `bitor(4,2)` E002. **Vais `|`/`&`/`^`/`<<`/`>>` 전부 동작 실측**
  (4|2=6 등) = 순수 트랜스파일러 갭(nl측 fixable).
- **수정: `map_bitwise2` 신설** — `bitand(a,b)`→`(a & b)` 등 5종 2-인자 매핑. **+ `map_bitnot` 문자열버그
  근본수정**(발견: `map_bitnot`이 `outside_strings` 안 써서 `"call bitnot(x)"` 문자열을 `"call (~x)"`로 오염 —
  pre-existing latent 버그, 같은 code-as-data 클래스). 둘 다 `outside_strings`로 감쌈 → 문자열 verbatim.
- **신규 검증 예제 3종**: e31 bitwise(bitor 체인=7) / e32 중첩 struct 필드변이(o.inner.v=7) / e33 guard 체인(2).
  값-정확성 59→62. PRELUDE bitwise ✅ + 상태열 추가. README 인덱스 e30→e33.
- 트랜스파일러 unit +8(26→34: 5 bitwise매핑 + bitnot + 2 문자열safety guard). 값-정확성 62/62, 회귀0.
- 교훈: **nl측 vs Vais측 구분이 작업 방향 결정**(Vais `|` 동작확인 → 트랜스파일러 수정이 정답)/PRELUDE 약속을
  실측(bitor 미매핑)/**기존 map_bitnot도 문자열버그 보유**(새 기능 추가가 기존 잠복버그 노출 — outside_strings
  미적용)/모든 문자열-건드리는 매핑은 outside_strings 필수(code-as-data 불변).

## 2026-06-07 (/loop iter 55: PRELUDE ✅ 감사 — Int(x) 변환 nl측 수정 + map_types 문자열버그)
- **PRELUDE 남은 ✅ 실측 감사**(push/map/bitor가 다 틀렸으므로): print 보간/Option/Result/`?`는 **실측 OK 확인**
  (초기 P001들은 내 test-helper의 multi-fn-한줄 artifact였음 — 제대로 된 파일로 재측정시 통과). **Int(x) 변환은
  실제 깨짐 발견**: `Int(f)`가 `i64(f)`로 사상→Vais P001("found I64, expected expression"). PRELUDE는 `x as i64`
  약속했으나 트랜스파일러가 spec 미구현(map_types가 `Int`→`i64` 단어매핑만).
- **수정: `map_conversions` 신설** — `Int(x)`→`(x as i64)`, F64/UInt8.. 등 13 숫자타입 call형을 `as`형으로
  (map_types 前 실행, outside_strings). Vais `as` 변환 7종 전부 동작 실측. **Some/Ok/struct 생성자는 비변환**
  (_CONV_TYPES에 없음). **+ map_types 문자열버그 근본수정**(또 발견: `"type is Int and List"`→`"type is i64
  and Vec"` 오염, outside_strings 미적용 — map_bitnot과 동일 클래스 잠복버그). 보간 `{x}`는 Vais가 처리하므로
  outside_strings 무해(e19 `lang=nl n=3` 유지 확인).
- **신규 검증 예제**: e34 Int(x) 변환(F64 5.9→Int 5). 값-정확성 62→63. PRELUDE Int(x) 매핑 정밀화. README 인덱스 e34.
- 트랜스파일러 unit +7(34→41: 3 변환매핑 + 생성자/타입 비변환 + 2 문자열safety). 값-정확성 63/63, 회귀0.
- 교훈: **PRELUDE 감사가 spec↔구현 불일치 노출**(spec은 `x as i64` 맞았으나 구현이 `i64(x)`)/test-helper artifact
  를 실측 재확인(Option/Result는 멀쩡)/**또 다른 문자열버그**(map_types) — 새 매핑 추가 패턴이 기존 outside_strings
  누락을 연쇄 노출(map_bitnot→map_types)/숫자변환 call형은 nl P4(모호성0)의 핵심(`as` 금지하니 call형 동작 필수).

## 2026-06-07 (/loop iter 56: code-as-data 일괄 감사 — 문자열버그 3종 근본수정)
- map_bitnot/map_types 문자열버그가 **연쇄 패턴**임을 인지 → **전 map_ 함수 outside_strings 감사**(선제적,
  다음 기능이 노출하기 전에). re.sub/replace 하나 outside_strings 없는 함수 식별 → 실측으로 문자열 오염 3종 확인.
- **문자열버그 3종 근본수정**(전부 code-as-data 위반, **self-host 컴파일러 직접 영향** — `compile("...")` 임베드):
  ① **map_collection_methods**: 문자열 `"call v.sum() x"`→`"call v.fold(...)"` 오염 → outside_strings 래핑.
  ② **map_enum_qualified**: 문자열 `"Color.Red dot"`→`"Red dot"` 오염 → outside_strings 래핑.
  ③ **map_arm_return**: 문자열 `"P => return x"`→`"P => { return x" }`(구조파괴!) → string-blank 후 `=> return`이
     code에 있을 때만 변환. (outside_strings는 fn을 non-string에 적용하나 arm_return은 전체줄 재구성이라
     blank-then-check 방식 사용.)
- 실측 확인: 3 문자열 전부 verbatim 유지 + **real 변환 전부 동작**(c1 arm-wrap, c2 .sum→fold, e22 enum-strip,
  mixed arm run=5). **self-host e2e(fixpoint-full/str)도 green**(임베드 nl 문자열 무손상 확인 — 가장 중요).
- 트랜스파일러 unit +4(41→45: 3 문자열safety + 1 real-arm-wrap 회귀가드). 값-정확성 63/63, 회귀0.
- 교훈: **잠복버그가 패턴이면 일괄 감사**(map_bitnot→map_types→3종 더, 연쇄 노출 대신 선제 sweep)/code-as-data는
  self-host(compile 임베드)에 치명적 — self-host e2e가 핵심 가드/전체줄 재구성 함수(arm_return)는 outside_strings
  대신 blank-then-check/**모든 텍스트-건드리는 매핑은 문자열 안전성 검증 필수**(이제 전 map_ 함수 감사 완료).

## 2026-06-07 (/loop iter 57: cold-start 재측정(강건화 후) + P9 +4)
- 트랜스파일러 강건화(iter54-56) 후 **cold-start 재측정** — 신선 general-purpose 서브에이전트(nl 처음, 코퍼스만,
  213줄 컨텍스트)에게 **강건화된 영역 조합 과제**(enum Op 디스패치 + bitwise calc) 작성 지시.
- **🎯 cold-start 첫시도 성공**: 서브에이전트가 e22(enum 디스패치) + e31(bitwise 워드함수)에서 정확 추론 →
  `match op { Op.And => return bitand(a,b) }` → 빌드+실행=2(6&3). **iter54 bitwise 수정 + e31 예제가 복합 작동**
  (수정+예제가 함께 cold-start 가능케 함). e35로 승격.
- **신규 검증 예제 4종**: e35 계산기 enum+bitwise 디스패치(cold-start, 2) / e36 bool 반환함수→if조건(1) /
  e37 다중필드 struct 계산(area=50) / e38 음수 `0-n`(5). 값-정확성 63→67.
- **harness 교훈 재확인**: probe helper의 `printf '%s\n' "$@"` 다중-fn 한줄화 artifact(P001 오탐) → heredoc으로
  실측해야 정확(bool_fn/negatives 등 멀쩡한데 artifact로 fail 보임). 이전 iter에서도 동일 함정.
- 검증: 값-정확성 67/67, 회귀0. README 인덱스 e34→e38 + 카운트 49/49. 교훈: **수정+예제가 cold-start 복합 레버**
  (bitwise 트랜스파일러수정 없었으면 e35 cold-start 실패)/cold-start는 강건화 후 더 넓은 구문 첫시도 가능/
  probe는 heredoc(다중fn 한줄 artifact 회피)/cold-start 산출물=새 검증예제(이중 레버 지속).

## 2026-06-07 (/loop iter 58: P9 +4 깊은 조합 — `?` 실패경로 / 재귀-struct accumulator)
- nl-side 인프라 성숙(코퍼스49/트랜스파일러 강건/P4 완비) → 유사예제 양산(수확체감) 대신 **깊은 조합/에러흐름으로
  갭 탐색**(heredoc probe, 더 높은 가치). 4 후보 전부 동작(nl-side 갭 없음=트랜스파일러 견고) + 미커버 영역 보강.
- **신규 검증 예제 4종**: e39 **`?` 에러 전파 실패경로**(check(-5)=Err(99)→`?` 단락→Err arm=0; 코퍼스 첫 `?`
  실패경로) / e40 Option을 struct 필드로+match(7) / e41 **재귀로 struct accumulator 전달**(sum+count 누적,
  build(3)=6; 재귀가 struct 반환+전달) / e42 while로 함수 반복적용(1→2→4→8). 값-정확성 67→71.
- 이전 코퍼스는 `?` 성공경로(d3run)만, Option/Result 단순매치만 있었음 → **`?` 실패 단락 + 재귀+struct + Option
  in struct**는 미커버. 실제 에러처리/누적 패턴 보강.
- 검증: 값-정확성 71/71, 회귀0. README 인덱스 e38→e42 + 카운트 53/53. 교훈: **성숙단계엔 갭탐색>양산**(깊은 조합이
  미커버 영역 노출, 갭 없으면 견고성 확인+커버리지)/`?` 실패경로처럼 "같은 기능의 반대 경로"가 미커버 흔함/
  heredoc probe가 다중fn 정확측정(printf-array artifact 회피 재확인).

## 2026-06-07 (/loop iter 59: hard-edge 탐색 — 제네릭/문자열ops 동작확인 + trait dispatch Vais갭)
- hard-edge(제네릭/trait/문자열ops) heredoc probe로 갭탐색. **제네릭 함수 `identity<T>` 동작**(나 자신도 처음
  확인) + **문자열 `.len()`/`==` 동작**. 발견: **`impl Trait for Type` Vais 미지원**(P001 `for`서; 하드코딩 Vais도
  실패). **`impl Type {}`(inherent)는 OK**(e09 동작 방식, e43 신규도) → nl 구조체 메서드 정상, **trait 다형성만 막힘**.
- **신규 검증 예제 3종**: e43 제네릭함수 identity<T>(5) / e44 문자열 길이 s.len()(5) / e45 문자열 동등성 a==b(1).
  코퍼스 첫 제네릭+문자열ops. 값-정확성 71→74.
- 추적: **Vais impl Trait for Type 미지원** TRACKED(근본=Vais 파서; nl은 impl Type로 충분). README 미커버 + ROADMAP.
- 검증: 값-정확성 74/74, 회귀0. README 인덱스 e42→e45 + 카운트 56/56. 교훈: **hard-edge 탐색이 능력 경계 확인**
  (제네릭/문자열 ops는 동작=좋은 발견, trait dispatch만 Vais갭)/inherent impl vs trait impl 구분(전자 OK 후자 막힘)/
  "안 될 것 같은" 것도 실측(제네릭 동작은 예상보다 좋음)/nl측 우회 존재(impl Type)면 Vais갭이라도 기능 가용.

## 2026-06-07 (/loop iter 60: hard-edge round2 — 제네릭+struct/char/문자열인덱스/클로저인자 전부 동작)
- hard-edge round2 heredoc probe. **4종 전부 동작**(능력 경계가 예상보다 넓음): ① **제네릭 함수+struct 조합**
  (`apply<T>(Pair{})`) ② **char 리터럴+비교**(`'A'=='A'`; 이전 char_lit 실패는 harness artifact였음 — 실제 동작)
  ③ **문자열 바이트 인덱싱 `s[0]`**(='h'104, 트랜스파일 surface서) ④ **클로저를 함수 인자로**(`apply_fn(|n| n*2, 6)`
  =12, 일급함수). nl측 갭 없음=트랜스파일러+Vais 견고.
- **신규 검증 예제 4종**: e46 제네릭+struct / e47 char ops / e48 문자열인덱스 / e49 클로저인자. 코퍼스 첫 char/
  string-index/closure-as-arg/generic+struct. 값-정확성 74→78.
- 검증: 값-정확성 78/78, 회귀0. README 인덱스 e45→e49 + 카운트 60/60. 교훈: **능력 경계가 예상보다 넓음**
  (제네릭+struct/클로저인자/char/string-index 전부 동작)/harness artifact 재확인(char_lit는 멀쩡, heredoc로 측정)/
  성숙단계 hard-edge 탐색이 "동작하는 고급기능"을 코퍼스에 추가(이중 가치: 견고성 실증 + AI 학습 커버리지).

## 2026-06-07 (/loop iter 61: 종합 예제(식 평가기) + 재귀 enum Vais갭 발견)
- 성숙 plateau → **여러 구문 종합 실전형 예제**(인터프리터 코어). expr_eval/ast_eval(비재귀 enum) 둘 다 동작 →
  **e50 식 평가기**(enum Node{Lit/Add/Mul} + match 디스패치, prod=Mul(3,4)→Add(prod,2)=14) 승격. 코퍼스 첫 인터프리터.
- **🚨 재귀 enum Vais갭 발견**: 진짜 재귀 AST(`enum Expr { Add(Expr, Expr) }`=자기참조) 시도 → **무음 miscompile**
  (1-level `Mul(Lit(3),Lit(4))`도 0/139 garbage). 트랜스파일러는 올바른 Vais 생성(`enum Expr{Add(Expr,Expr)}`)
  하나 Vais가 재귀 ADT payload 추출 못 함. **비재귀 enum(2-payload)은 OK**. **이것이 self-host codegen 트랙이
  AST를 재귀enum 대신 struct+인덱스로 인코딩한 근본 이유** — 이제 명시적으로 확인·문서화. 재귀 ADT는 실전
  인터프리터/파서 핵심 → 중요 Vais 갭(8번째). TRACKED.
- **신규 검증 예제**: e50 식 평가기(14). 값-정확성 78→79. ROADMAP TRACKED + README 미커버 + PRELUDE 무관.
- 검증: 값-정확성 79/79, 회귀0. README 인덱스 e49→e50 + 카운트 61/61. 교훈: **종합 예제가 구문 interplay 검증**
  (단일기능 아닌 인터프리터 코어)/**재귀 ADT Vais갭이 self-host struct+인덱스 설계의 근본 이유 확인**(설계가 이미
  이 갭을 우회하고 있었음)/비재귀 enum은 OK라 인터프리터 코어는 flat 인코딩으로 표현가능(e50).

## 2026-06-07 (/loop iter 62: self-host 우회패턴 시연 예제 — struct+인덱스 AST / 상태머신)
- iter61의 재귀enum 갭 발견에 이어, **그 우회패턴(self-host가 실제 쓰는)을 코퍼스 예제로 시연**. AI에게 "Vais갭
  있는 구문의 올바른 idiom"을 가르침.
- **신규 검증 예제 2종**: e51 **struct+인덱스 재귀 AST**(노드를 flat 필드배열, 자식을 인덱스로 참조, eval이 인덱스로
  재귀; mul(3,4)+2=14 — 재귀enum Vais갭의 정확한 우회, self-host 방식) / e52 **상태머신 run-counter**(배열 스캔 +
  in-word 플래그로 1-run 개수=3, 토크나이저 패턴). 값-정확성 79→81.
- e51이 특히 가치 — 재귀구조를 Vais가 직접 표현 못 할 때 nl이 쓰는 실제 idiom(인덱스 간접참조)을 시연. e50(flat
  enum)+e51(struct+인덱스 재귀)로 "인터프리터를 nl로 쓰는 두 방식" 코퍼스화.
- 검증: 값-정확성 81/81, 회귀0. README 인덱스 e50→e52 + 카운트 63/63. 교훈: **Vais갭의 우회패턴을 코퍼스로 시연**
  (갭 추적만이 아니라 "그럼 어떻게 쓰나"를 AI에게 제공)/struct+인덱스 간접참조가 재귀ADT 대체(self-host 검증된 패턴)/
  성숙단계엔 "막힌 것의 올바른 우회"가 양산보다 높은 가치(AI가 갭 만났을 때 idiom 제공).

## 2026-06-07 (/loop iter 63: 복잡 cold-start 재측정 → nl측 수정(String→Str) + 단어세기 e53)
- 풍부해진 코퍼스(64, 우회패턴 포함)로 **복잡 cold-start 재측정** — 신선 서브에이전트에게 **단어 세기**(문자열
  스캔+상태머신=실제 토크나이저) 지시. **알고리즘 완벽 작성**(s[i]/s.len() from e44/e48 + in-word 플래그 from e52)
  BUT `s: String`(Rust직관) 써서 E001(String≠str literal). **cold-start가 nl측 갭 노출**.
- **nl측 수정 2건**: ① **트랜스파일러 TYPE_MAP `String`→`str`**(Rust직관 String을 동작하는 str로, 무음 E001 방지)
  ② **nl-check 규칙 `String`→`Str`**(정식 nl 타입명 안내). 둘 다: String은 이제 **동작(forgiving) + 안내(canonical Str)**.
  오탐0(식별자 myStringVar 비발화). Vais는 String/str 별타입이라 혼용시 E001이었음.
- **신규 검증 예제**: e53 단어세기(토크나이저 코어, cold-start 산출 with Str, 4). 값-정확성 81→82. **가장 어려운
  cold-start 성공**(문자열 스캔+상태머신 종합).
- 검증: 값-정확성 82/82, nl-check unit 34→37, 트랜스파일러 unit 45→46, 회귀0. PRELUDE String행 추가. 교훈:
  **cold-start가 실전과제로 nl측 갭 노출**(String — 흔한 Rust직관, 이전 probe들은 안 건드림)/forgiving map + 안내
  규칙 조합(동작시키되 canonical 가르침)/cold-start는 코퍼스 검증뿐 아니라 갭 발견 도구(이중 가치 재확인).

## 2026-06-07 (/loop iter 64: cold-start 재측정(재고도메인 성공) + match 와일드카드 + batch-probe 신뢰성 교훈)
- 실전 cold-start 계속. **batch-probe(printf-array)로 Rust-ism 일괄 테스트 시도 → 다수 false 실패**(mut-assign/
  for-range/match-wildcard/len-var "fail"로 보였으나 heredoc 재측정 시 전부 동작=6/3/2/2). **printf '%s\n'의 `;`-구분
  다중문장 artifact 재확인** — batch-probe는 다중문장 부정확, heredoc/실제 cold-start만 신뢰. 진짜 확인된 Rust-ism은
  단일토큰만(println!/usize/Vec::new — 전부 nl-check flag됨 ✓).
- **재고도메인 cold-start 첫시도 성공**: 신선 서브에이전트가 struct Item{price,qty} + subtotal 메서드 + 3인스턴스
  집계=42 깔끔히(Rust-ism 0 — String갭은 string-param 한정 task-specific이었음). e54로 승격(코퍼스 첫 다중인스턴스
  struct 집계).
- **신규 검증 예제 2종**: e54 재고합계(cold-start, 42) / e55 match 와일드카드 `_`(기본분기, 코퍼스 미커버였음, 2).
  값-정확성 82→84.
- 검증: 값-정확성 84/84, 회귀0. README 인덱스 e53→e55 + 카운트 66/66. 교훈: **batch-probe는 다중문장 부정확**
  (heredoc/cold-start가 신뢰 측정)/cold-start 갭은 task-specific(String은 string-param task서만, struct task선 무갭)/
  `_` 와일드카드 미커버였음(흔한데 빠진 것)/매 도메인 cold-start가 커버리지+신뢰 동시 제공.

## 2026-06-07 (/loop iter 65: Collatz cold-start 첫시도 성공 + 도메인 cold-start 성숙 확인)
- recursion+분기 도메인 cold-start. **Collatz 스텝 카운터 첫시도 성공**: 신선 서브에이전트가 2-arg 재귀 +
  modulo + 짝/홀 guard chain으로 collatz(6,0)=8 깔끔히(Rust-ism 0). e56로 승격(코퍼스 첫 iterated-function/짝홀 분기).
- **3연속 cold-start 첫시도 성공**(word-count[String수정후]/inventory/collatz) = 코퍼스가 "다양한 도메인 첫시도
  성공" 성숙 도달. String갭(iter63)이 주요 잔존 rough edge였던 것으로 보임 — 이후 도메인들은 무갭.
- **신규 검증 예제**: e56 Collatz(8). 값-정확성 84→85.
- 검증: 값-정확성 85/85, 회귀0. README 인덱스 e55→e56 + 카운트 67/67. 교훈: **코퍼스 "다도메인 첫시도" 성숙**
  (3연속 클린 성공)/recursion+분기는 흔한데 코퍼스 첫 등장(Collatz)/cold-start가 도메인별 커버리지 계속 추가.

## 2026-06-07 (/loop iter 66: nl측 갭 발견·수정 — break/continue 루프제어 매핑)
- Rust-idiom-heavy heredoc probe로 **nl측 갭 발견**: `break`/`continue`가 트랜스파일러서 verbatim 통과→Vais E002
  (Vais 루프제어는 `B`/`C`). chain-after-call/bool-expr-return/shadow는 PASS(견고성 확인).
- **수정: map_loop_keywords 신설** — `break`→`B`, `continue`→`C`(whole-word, outside_strings). Vais `B`/`C`가
  user 루프 본문서 동작 실측(run 3/3 확인). map_if(루프구조) 뒤 배치.
- **오탐0**: 문자열 "press break to continue" verbatim, 식별자 `breaker` 비변환(word-boundary). 전 49예제+self-host
  e2e green(self-host 무손상).
- **신규 검증 예제 2종**: e57 break(루프조기종료, 3) / e58 continue(반복건너뜀, 4). 코퍼스 첫 루프제어. 값정확성 85→87.
- 트랜스파일러 unit +4(46→50: break/continue 매핑 + 문자열safety + 식별자가드). PRELUDE 루프제어 섹션 추가.
- 검증: 값-정확성 87/87, 트랜스파일러 unit 50/50, 회귀0. README 인덱스 e56→e58 + 카운트 69/69. 교훈:
  **흔한 제어키워드(break/continue)가 미매핑이었음**(while 루프는 많았으나 user break/continue 없었어서 잠복)/
  Rust-idiom probe가 cold-start와 다른 갭 노출(continue는 cold-start 과제선 안 나왔음)/Vais B/C 동작확인 후 forgiving 매핑.

## 2026-06-07 (/loop iter 67: nl측 갭 발견·수정 — 튜플 구조분해)
- targeted idiom probe(heredoc) 계속. const/while-and/neg-literal/divmod PASS(견고성), **튜플 반환+구조분해 갭 발견**:
  `let (a, b) = pair()`가 트랜스파일러서 `let (a, b) = ` verbatim→Vais E002(Vais는 `(a, b) := `, let 없이).
  Vais 튜플 동작 실측((a,b):=pair() run=7).
- **수정: map_let에 튜플 패턴 케이스 추가** — `let (a, b) = expr`→`(a, b) := expr`(스칼라 케이스 前). 튜플 타입
  `-> (Int, Int)`는 map_types가 이미 처리(괄호 안 Int→i64).
- **신규 검증 예제**: e59 튜플 반환+구조분해(7)=코퍼스 첫 튜플. 값정확성 87→88.
- 트랜스파일러 unit +1(50→51). PRELUDE 튜플행 추가.
- 검증: 값-정확성 88/88, 트랜스파일러 unit 51/51, self-host OK, 회귀0. README 인덱스 e58→e59 + 카운트 70/70.
  교훈: **튜플 구조분해도 흔한데 미매핑이었음**(map_let이 스칼라만)/targeted probe가 또 nl측 갭 노출(break/continue
  iter66, 튜플 iter67 — idiom probe가 cold-start 못 잡는 문법 갭 연속 발견)/Vais 동작확인 후 forgiving 매핑.

## 2026-06-07 (/loop iter 68: digit-sum cold-start 첫시도 성공 — 4연속 클린)
- 방법 번갈아(이번 cold-start). **자릿수 합 cold-start 첫시도 성공**: 신선 서브에이전트가 `%10`/`/10` 추출 루프 +
  accumulator로 digit_sum(12345)=15 깔끔히(Rust-ism 0). e60로 승격(코퍼스 첫 자릿수추출 패턴).
- **4연속 cold-start 첫시도 성공**(word-count/inventory/collatz/digit-sum) — 코퍼스 다도메인 성숙 견고. cold-start로는
  최근 갭 안 나옴(실전과제갭은 String 이후 소진된 듯; 문법갭은 targeted probe가 잡음).
- 값정확성 88→89. 검증: 89/89, 회귀0. README 인덱스 e59→e60 + 카운트 71/71. 교훈: cold-start 실전과제갭 소진 단계
  (4연속 클린)→문법갭은 targeted probe가 주력/자릿수추출은 흔한데 코퍼스 첫등장.

## 2026-06-07 (/loop iter 69: targeted probe round2 — 견고성 확인 + 커버리지 +2)
- targeted idiom probe round2(heredoc). not-expr/struct-multi/range-check/unit-fn PASS, arr-in-expr는 내 expect
  오산(a[1]+a[2]=50인데 30 적음 — 빌드가 정답). **nl측 갭 없음**(트랜스파일러 견고 재확인).
- **신규 검증 예제 2종**(커버리지): e61 배열 계산인덱스 a[i]+a[i+1](50, 코퍼스 첫 computed-index 산술) / e62 struct로
  다중값 반환(minmax→{lo,hi}, 12, 튜플의 named 대안). 값정확성 89→91.
- 검증: 값-정확성 91/91, 회귀0. README 인덱스 e60→e62 + 카운트 73/73. 교훈: targeted probe 갭 없으면 견고성확인+
  커버리지(computed-index/struct-multi-return 미커버였음)/expect 오산은 빌드가 정답(arr a[1]+a[2]=50).

## 2026-06-07 (/loop iter 70: 깊은조합 probe — 제네릭struct/enum-struct payload + 클로저반환 Vais갭)
- 깊은조합 probe(heredoc). **제네릭 struct `Box<T>` 동작**(코퍼스 첫) + **enum 페이로드가 struct `Has(Pt)` 동작**
  (비재귀라 OK) + **loop+break+누적 동작**. **캡처 클로저 반환 Vais갭 발견**: `fn adder(n)->fn(Int)->Int{return |x|
  x+n}` E001(n 캡처를 bare fn-ptr로 반환 — env 없음; 하드코딩 Vais도 실패). **클로저 인자(e49)는 OK, 반환만 막힘**
  (클로저 ABI 클래스, Vais memory bare-fn-ptr 노트와 일치). 9번째 Vais갭 TRACKED.
- **신규 검증 예제 3종**: e63 제네릭struct Box<T>(7) / e64 enum-struct payload Has(Pt)(5) / e65 loop+break+누적(10).
  값정확성 91→94.
- 검증: 값-정확성 94/94, 회귀0. README 인덱스 e62→e65 + 카운트 76/76. 교훈: 제네릭struct/enum-struct 동작(능력 넓음)/
  **클로저 인자 OK vs 반환 막힘**(env 캡처 반환경계 Vais ABI 한계)/깊은조합이 잔존 Vais갭 노출(nl측은 견고).

## 2026-06-07 (/loop iter 71: 전체 회귀 스위프 검증 + 재귀 예제 +2)
- **전체 회귀 스위프**: 세션의 트랜스파일러 변경(break/continue/튜플/String/숫자변환/문자열안전 5종)이 전 테스트
  표면서 회귀0 확인. 값-정확성 94/94, 트랜스파일러 unit 51/51, nl-check unit 37/37, **self-host e2e 77 PASS 0 fail**
  (code-as-data 불변 유지 — self-host가 nl 문자열 임베드하므로 핵심). 트랜스파일러 하드닝이 완전히 회귀-클린 검증.
- **신규 검증 예제 2종**(distinct 재귀): e66 피보나치(트리 재귀, fib(7)=13; e03 factorial은 linear) / e67 거듭제곱
  (2-인자 재귀 지수, power(3,4)=81). array-sum-for는 fr2/e25 중복이라 skip. 값정확성 94→96.
- 검증: 값-정확성 96/96, 회귀0. README 인덱스 e65→e67 + 카운트 78/78. 교훈: **세션 누적 트랜스파일러 변경의 전체
  회귀 스위프가 안전판정**(self-host e2e가 code-as-data 핵심 가드)/distinct 재귀shape(트리/지수)는 linear factorial과 별개 커버리지.

## 2026-06-07 (/loop iter 72: 🎯 self-host 컴파일러 능력 추가 — 비교를 값으로(return a==b))
- 부트스트랩 경계 정밀 매핑(실제 소스 fragment를 fixpoint_full.compile()에 먹임). **fixpoint_full이 처리하는 것**:
  else-if/중첩호출 산술/배열param인덱스/struct+List+while+if 종합 fn 전부 OK. **미처리 발견**: ①`return a == b`
  (비교를 값으로 — 비교연산자를 if/while 조건서만 처리, expression position선 드롭=LHS만 반환) ②for루프 ③`(...)` 그룹화.
- **🎯 nl self-host 컴파일러 능력 추가 — 비교-as-value**: gen_fold에 `<`(18)/`>`(19)/`==`(20) arm 추가
  (icmp + zext i1→i64). 비교는 +/-보다 낮은 precedence라 RHS는 full additive(gen_expr). if/while 조건은 LHS/RHS를
  oppos서 분리해 sub-range 전달하므로 충돌 없음(조건 핸들러 무영향 확인).
- 실측: `a==b`=1/0, `a<b`=1, `n>5`=1 전부 동작. if/while 조건/재귀 회귀0(while-cond=10, fac(5)=120).
- e2e: fixpoint-full **35→39 PASS**(+4 비교-as-value), 값-정확성 aggregate 96/96, 회귀0.
- **추적(self-host 컴파일러 잔존)**: `(...)` 그룹화 미지원(gen_factor가 `(`를 call로만 — `(a<b)+(b<c)` 실패),
  for루프 codegen 미구현(while만). 둘 다 fixpoint_full 능력확장 후보(부트스트랩 향). 교훈: **실제 소스 fragment로
  부트스트랩 경계 정밀 매핑**(months 통째보다 구체적 갭 식별)/비교-as-value는 self-host 소스가 쓰는 핵심(능력 추가 정당)/
  precedence 낮은 비교는 gen_fold 최종단계/조건핸들러는 sub-range라 충돌없음.

## 2026-06-07 (/loop iter 73: 🎯 self-host 컴파일러 능력 추가 — `(...)` 그룹화)
- iter72 비교-as-value가 노출한 `(...)` 그룹화 갭 수정. **gen_factor에 `(` (kind9) 케이스 추가**: `( <expr> )`를
  paren_end로 닫고 내부를 gen_expr 재귀. **skip_factor에도 `(` 케이스**(paren_end+1로 스킵). call `name(`은 ident
  분기서 먼저 처리되므로 무영향.
- 실측: **`(a<b)+(b<c)`=2**(이전 실패), `(2+3)*4`=20(precedence override), `(n+1)*(n+2)`=20, `((1+2)*3)`=9(중첩),
  call 무영향(sq 18), bare precedence 유지(2+3*4=14). e2e fixpoint-full **39→43 PASS**(+4 그룹화), aggregate 96/96, 회귀0.
- SELF_HOST.md FP12h, ROADMAP 그룹화 해결 표시(남은 codegen 갭=for 정도). 교훈: **iter72가 노출한 갭을 iter73서
  해결**(능력추가 연쇄 — 비교-as-value→그룹화 자연스레)/`(`는 gen_factor/skip_factor 양쪽 필요/call은 ident분기 우선이라 무충돌/
  그룹화로 precedence override+중첩+비교조합 전부 가능. **self-host 컴파일러 codegen 거의 완전**(남은 주요갭=for, 그외 부트스트랩=규모).

## 2026-06-07 (/loop iter 74: 🎯 self-host 능력 추가 — `>=`/`<=` (is_digit 부트스트랩 패턴))
- self-host 소스가 쓰는 비교(is_digit `c >= 48 and c <= 57`)가 fixpoint_full서 깨짐 발견(`>=` 우연동작, `<=` 오답).
  근본: `>=`/`<=`를 단일 토큰으로 인식 안 함(`>` `=` 분리). **수정: tokenize `<=`→kind29, `>=`→kind30**(다음 char `=`
  체크) + gen_fold 비교-as-value arm + while/if 조건핸들러 양쪽에 sle/sge 추가(pred_le/pred_ge).
- 실측: `a>=b`/`a<=b` value 1/0 정확, **is_digit if-패턴** is_d(53)=1/is_d(99)=0, `while i<=n` 합산=15. 회귀0
  (`<`/`>`/`==`/재귀 유지). e2e fixpoint-full **43→48 PASS**(+5), aggregate 96/96.
- SELF_HOST.md FP12i, ROADMAP: `>=<=` 해결 + **for는 self-host 소스 미사용(while만, 0건) → 부트스트랩 비-critical**
  결론. 남은 후보: `and`/`or`를 값으로(is_digit `and` — 미검증). 교훈: **경계매핑이 bootstrap-critical 갭 식별**
  (is_digit `>=<=`=실제 self-host 소스 패턴, for는 비-critical 판명)/2-char 연산자는 tokenize+gen_fold+조건핸들러 3곳/
  self-host 소스 grep으로 critical vs nicety 구분(for 0건=비critical). **self-host 소스 codegen 구문 거의 완비.**

## 2026-06-07 (/loop iter 75: 🎯 self-host 능력 추가 — `and`/`or`를 값으로 (완전한 is_digit))
- is_digit 마지막 갭: `c >= 48 and c <= 57`서 `and`/`or` 미인식(식별자로 falthrough, 비교 한쪽만 반환). **수정**:
  tokenize `and`→kind31/`or`→kind32(kw3/kw2), gen_fold에 logical arm(`and`→`and i64`/`or`→`or i64`; 피연산자가
  0/1이라 bitwise로 정확). **precedence 핵심**: 처음 `c >= (48 and c<=57)` 오associativity 버그 → **next_logical
  헬퍼로 비교 RHS를 다음 `and`/`or`서 바운드**(`(c>=48) and (c<=57)`), comparison/and-or arm 모두 rstop서 계속 fold.
- 실측: **완전한 is_digit** is_d(53)=1/is_d(99)=0/경계48,57=1, or-chain(a<0 or a>100), and-chain ordered/unordered.
  회귀0(재귀120/그룹화20). e2e fixpoint-full **48→55 PASS**(+7), aggregate 96/96.
- SELF_HOST.md FP12j, ROADMAP: and/or-value 해결, 남은건 for+compound-condition(둘 다 우회가능=while/중첩if).
  **결론: self-host 소스 codegen 구문 거의 완비, 부트스트랩에 충분.** 교훈: **precedence는 RHS 바운딩이 핵심**
  (next_logical로 비교<and/or 보장)/0-1 피연산자엔 bitwise and/or가 logical과 동등/**4연속 bootstrap-relevant 추가**
  (비교-value/그룹화/`>=<=`/`and or`)로 is_digit류 완성 — 경계매핑이 실제 갭을 정확히 식별·해결.

## 2026-06-07 (/loop iter 76: 🎯 self-host 능력 추가 — compound 조건 (is_alpha/is_digit 완성))
- 경계매핑: self-host 소스가 **compound 조건 26곳 사용**(is_alpha `if c >= 97 and c <= 122`) — bootstrap-critical.
  기존 if/while 조건핸들러는 단일비교만 split. **수정: 조건 전체를 gen_expr로 값평가 후 `icmp ne 0` 분기**
  (gen_expr가 비교/`and or`/그룹화 처리하므로 compound 자동지원 + 핸들러 단순화). while+if 양쪽 리팩토링.
- 실측: **is_alpha if-and**(c>=97 and c<=122) 동작, if-or, **while compound**(i<n and s<100)=50. **헤비 회귀0**
  (재귀120/while-simple10/if-simple100/if-else9 — 핵심 조건로직 리팩토링했으나 전부 유지). e2e fixpoint-full **55→59**, aggregate 96/96.
- SELF_HOST.md FP12k, ROADMAP: compound조건 해결, for만 비-critical 잔존. **결론: self-host 소스 codegen 구문
  사실상 완비, 부트스트랩 충분.** 교훈: **조건 전체를 값평가(gen_expr)+`icmp ne 0`가 compound 자동지원+단순화**
  (단일비교 split보다 일반적)/리팩토링은 헤비 회귀로 안전판정/self-host grep 26곳=compound critical(for 0건과 대조).
  **5연속 bootstrap-relevant 추가**(비교-value/그룹화/`>=<=`/`and or`/compound조건)로 is_alpha/is_digit류 완성.

## 2026-06-07 (/loop iter 77: 🎯 self-host 통합 검증 — 완전한 4-함수 lexer fragment 컴파일)
- 더 큰 self-host 소스 fragment(실제 lexer shape: is_digit/is_alpha/is_space + classify 디스패처)를
  fixpoint_full.compile()에 먹임. 초기 "65 vs 321" → **8-bit exit 절단 확인**(321&0xFF=65, 버그아님; IR은 정확 321).
  값≤255로 재측정: **완전한 4-함수 lexer fragment = 171 정확**(classify(53)=1/classify(104)=2/classify(32)=3 →
  1+20+150). FP12g-k 추가가 실제 lexer 코드 컴파일 가능케 함을 통합검증.
- 격리로 모든 메커니즘 확인: direct bool-helper, bool-helper arith, if-call-bool, two-helper-dispatch,
  classify-alpha, multi-call arith(call-mul-add). **새 갭 없음** — 최근 추가가 real lexer 코드를 동작시킴.
- **신규 e2e 가드**: 4-함수 lexer-classify 통합(171) — self-host 컴파일러가 자기 lexer 코어 codegen 증명.
  e2e fixpoint-full **59→60 PASS**, aggregate 96/96.
- 교훈: **경계매핑이 갭 대신 통합 검증 산출**(최근 5 추가가 real lexer fragment 동작시킴=능력 충분 확증)/
  "65 vs 321"는 또 8-bit 절단(값≤255 유지 재확인)/격리로 메커니즘 분리 검증 후 통합이 진단순서. **self-host
  컴파일러가 자기 lexer의 완전한 4-함수 fragment를 codegen** = 부트스트랩 codegen 능력 실증 진전.

## 2026-06-07 (/loop iter 78: 🎯 #1 부트스트랩 갭 발견 — 문자열 파라미터 (`fn f(s: Str)`))
- parser/eval-shape fragment 경계매핑. token-scan-count/while-acc-struct OK, **count_digits(s) 실패**(0 vs 3).
  격리: scan-inline-cmp/call-with-s[i]/if-call-s[i]/loop-call-simple-bool 전부 0 → 공통인자 = **string-as-PARAMETER**.
  확인: 로컬문자열 `let s=...`은 OK(local-string-index=49), **param 문자열 `fn f(s)` 실패**(param-string-index=0).
  IR: `@f(i64 %a0)`(param i64 오타입), 문자열리터럴 arg가 `f(i64 0)`(0 전달!), `s[i]`가 array-GEP(오타입).
- **#1 부트스트랩 갭 확정**: fixpoint_full이 string-as-parameter 미지원. **self-host 소스 `Str` param 176곳 사용**
  (fn tokenize(src: Str)/name_eq(src: Str,..)/is_putchar — 최다 패턴) = bootstrap-critical. 로컬문자열만 동작.
  근본: compile() 입력이 무타입 param(`fn f(s)`)이라 시그니처로 string 판별 불가.
- **해결 경로(dedicated 권장, 큰 작업)**: ①embedded 문법에 param 타입주석(`fn f(s: Str)`) — tokenize `:`+타입,
  build_fns 파싱 ②Str-param=i8* slot + 호출 arg 문자열-포인터 전달 + `s[i]` byte-load. param-typing 인프라.
- 이번 iter는 **경계 정밀식별+문서화**(ROADMAP #1 갭). 큰 다부분 변경은 turn 끝에 rushed half-merge 위험 →
  dedicated 진행으로 추적(막히면 추적 원칙, 단 명확한 계획과 함께). 교훈: **경계매핑이 #1 bootstrap-critical 갭
  정확 식별**(string param 176곳, self-tokenization 직전 최대 갭)/로컬 vs param 문자열 구분이 진단핵심/
  param-typing은 시그니처 타입주석 필요(무타입선 string 추론 불가). self-host codegen은 표현식/제어흐름은 완비,
  string-param이 다음 큰 관문.

## 2026-06-07 (/loop iter 79: 🎯🎯 #1 부트스트랩 갭 대부분 해결 — 문자열 파라미터 (FP12l))
- **#1 부트스트랩 갭(문자열 파라미터, self-host 176곳) dedicated 다단계 구현**: ①tokenize `:`→kind16
  (struct 리터럴 `name: val`이 `:` 토큰 가정 안해 4 fail→struct field vstart가 `:` skip하게 수정) ②Fn struct에
  p0ty..p3ty 타입필드, build_fns가 `:` 후 타입ident 파싱(`Str`→1, src 인자 threading) ③emit_fn이 Str-param을
  `i8* %aN` 시그니처 + is_arr=3 slot + `alloca i8*`/store ④gen_factor strlit-as-value(Op kind2=i8* ptr, 전역 GEP)
  ⑤call-arg가 kind2면 `i8*` 타입 emit. 격리후통합, 단계별 빌드+e2e.
- 실측: **string param 동작** s[0]=65/s[1]=66, **name_eq shape** `s[a]==s[b]`=1, s[i]+s[i+1]=131. **잔존 sub-갭:
  `s.len()` on param**(param 컴파일타임 길이 0→runtime strlen 필요; tokenizer 주루프용, name_eq류는 명시 length라 OK).
- 회귀0(int param/재귀/struct 리터럴 전부 유지 — `:` 토큰화가 struct깬 것 즉시 수정). e2e fixpoint-full **60→64 PASS**,
  aggregate 96/96. SELF_HOST.md FP12l, ROADMAP #1 갭 대부분해결 표시.
- 교훈: **#1 갭 dedicated 다단계가 정답**(5단계 격리후통합)/새 토큰(`:`)이 기존 구문(struct 리터럴) 깰 수 있음→
  즉시 회귀확인+수정/Op kind 확장(2=i8*ptr)으로 타입드 인자/param-typing은 시그니처 타입주석 기반. **self-host
  최다 패턴(176곳 Str param) 동작=self-tokenization 큰 진전**. 다음=`s.len()` on param(runtime strlen).

## 2026-06-07 (/loop iter 80: 🎯🎯🎯 #1 부트스트랩 갭 완전 해결 — s.len() on param (runtime strlen))
- FP12l 잔존 sub-갭(`s.len()` on string param) 해결. **string `.len()` codegen에 runtime strlen 분기**: slen>0면
  컴파일타임(로컬 리터럴), slen==0(param)이면 **runtime strlen 루프 emit**(i8* base 로드 → NUL까지 walk +count,
  %slN alloca+labeled loop). 
- 실측: **param-strlen=5/2, count-digits-param(while i<s.len())=3, 🎯🎯 실제 토크나이저 ntok(s: Str) on param=3**
  (source를 string param으로 받아 스캔 — self-host 토크나이저 `fn tokenize(src: Str)` 시그니처 동작!). 로컬 string
  .len() 컴파일타임 유지(회귀0). e2e fixpoint-full **64→67 PASS**, aggregate 96/96.
- SELF_HOST.md FP12m, ROADMAP #1 갭 **완전 해결** 표시. 교훈: **string param .len()=runtime strlen**(param은
  컴파일타임 길이 모름)/로컬 vs param을 alen>0로 분기(로컬=컴파일타임, param=runtime)/**#1 부트스트랩 갭 닫힘**
  =함수가 source string param 받아 스캔 가능=self-tokenization 핵심 도달. **이번 turn FP12l+m으로 self-host
  최다 패턴(Str param 176곳) 완전 동작.** 다음=실제 self-host 토크나이저 함수 1개를 통째로 fixpoint_full에 먹여보기.

## 2026-06-07 (/loop iter 81: 🎯 #2 부트스트랩 갭 식별 — List 파라미터/반환 (`fn f(toks: &List<Token>)`))
- 실제 토크나이저 body shape 경계매핑: **string param → List<Token> 빌드 동작**(tokenize-to-list=4, struct-token-build=67,
  classify-tokenize 멀티클래스+helper dispatch=5 — FP12l/m string param이 토크나이저 body 가능케 함). 다음 경계 발견:
  **List-as-parameter/return 실패**(list-as-param=0; `fn sum_list(xs)` List param이 i64 오타입, `fn build()` `return xs`가
  List 안넘김, 호출 단일 i64 전달).
- **#2 부트스트랩 갭 확정**: List 파라미터/반환. **self-host 소스 `&List<Token>` param 177곳**(eval_term/eval_expr 등
  parser/eval 코어, string param 176과 동급 최다). 아키텍처적으로 string보다 큼(List=스택 [64 x i64] 버퍼+length라
  i64 convention 못넘김). **핵심: self-host는 `&List`(by-ref)** → 버퍼 **포인터** 전달이 자연 해법(복사 아님).
- **해결경로(dedicated)**: ①`&List<T>`/`List<T>` param 타입주석 tokenize/parse ②List-param=버퍼 포인터 slot+호출 주소전달+
  `xs[i]`/`xs.len` 포인터경유 ③`return xs` List 반환. String param(FP12l/m)과 같은 패턴이나 2-slot+by-ref.
- 이번 iter는 경계 정밀식별+#2 갭 문서화(아키텍처 변경이라 turn 끝 rushed 회피, dedicated 추적). 교훈: **#2 갭=List param
  (177곳, parser/eval 직결)**/string param 다음 최대 갭/self-host `&List` by-ref가 포인터전달 해법 시사/string param 성공이
  토크나이저 body 가능케 함(다음은 parser가 toks List 받는 부분). e2e fixpoint-full 67(회귀0), 값정확성 96/96.

## 2026-06-07 (/loop iter 82: #2 갭 Stage 1 — List param 타입파싱 + codegen 스킴 검증)
- **#2 갭(List 파라미터) dedicated 시작. Stage 1(FP12n): `List<T>` param 타입주석 파싱**(build_fns `:` 브랜치 확장:
  `List` 인식→pty=2, `List<...>` 전체를 닫는 `>`까지 skip해 element type(Token)이 param으로 오집계 안 되게; `&`는
  tokenizer가 이미 drop). parse-only, e2e 67 유지 회귀0. 커밋 a6e28fa.
- **codegen 스킴 clang 실증(run=60)**: List param = **`i64* bufptr` + `i64 len` (2 args)**. callee `getelementptr i64,
  i64* %buf, i64 <idx>`+load 인덱스 / `xs.len`=len arg. caller 버퍼 base GEP + length 전달. → 설계 de-risk.
- **남은 Stage 2(codegen)**: emit_fn 시그니처(List=2 arg 위치) + 슬롯(ptr+len, is_arr=4) + call-site(2 arg emit) +
  gen_factor 포인터경로. **핵심 난점: List param이 2 arg 위치 소비→`%aN` 번호 재매핑**(현 1:1 깨짐)=interlocking
  변경. turn 끝 rushed half-merge 위험 → dedicated 집중 다음 회차. 막히면 추적.
- 교훈: **아키텍처 변경은 스킴 먼저 격리검증**(clang으로 i64*-param+GEP run=60 확인 후 구현=설계확신)/Stage 분리
  (파싱 먼저 커밋, codegen 다음)/2-wide param이 arg 번호 재매핑 유발=interlocking이라 dedicated 필요. e2e 67, 값정확성 96/96.

## 2026-06-07 (/loop iter 83: 🎯🎯 #2 부트스트랩 갭 Stage 2 — List 파라미터 codegen 동작)
- **List-param codegen 다단계 구현+동작**. 스킴 단순화로 interlocking 회피: **List param=1 `i64*` arg(len은 buf[63])**
  →`%aN` 1:1 유지(재매핑 불필요!), 로컬 List 표현 무변경(call-site서만 buf[63]에 len write)=회귀0. ①emit_fn 시그니처
  `i64* %aN` ②슬롯 is_arr=4(i64** alloca) ③call-site: 로컬 List arg는 len→buf[63] store + 버퍼 base GEP=Op kind3
  ④gen_factor `[`(load ptr→GEP i64→load)/`.len`(load ptr→GEP[63]→load) 포인터경로 ⑤call-arg kind3→`i64*` emit.
- 실측: **List-as-param 동작** sum_list(xs)=60(build+pass+sum via xs[i]/xs.len), first(xs)=7, cnt(xs)=3.
  **parser/eval 시그니처 `fn eval_expr(toks: List<Int>)` 동작.** 회귀0(List 로컬 build=100/재귀120/struct 전부 유지).
  e2e fixpoint-full **67→70 PASS**, aggregate 96/96.
- **잔존 sub-갭: List 반환(`fn build() -> List`)** — `return xs` 스택버퍼 escape 못함. 회피: out-param(호출자가 List
  만들어 &로 넘기고 callee 채움). List-param(읽기) done, List-return(쓰기) 다음. SELF_HOST.md FP12n, ROADMAP #2 읽기측 완료.
- 교훈: **스킴 단순화가 interlocking 회피 핵심**(1 i64* arg+len@buf[63]→2-wide 재매핑 불필요)/로컬 표현 무변경+call서만
  buf[63] write=회귀안전/검증된 스킴(clang run=60→30) 구현이 5단계 무회귀 통과. **#2 갭 읽기측 완료=self-host parser/eval
  코어 시그니처(177곳 &List param) 동작.** 다음=List 반환(out-param 또는 직접) 또는 더 큰 self-host fragment.

## 2026-06-07 (/loop iter 84: 🎯 #3 부트스트랩 갭 식별 — >4 파라미터 (self-host 코어 8 param))
- 양대 갭(string/List param) 후 실제 self-host 함수 먹임: name_eq(s,a,alen,b,blen)=5param, kw3(s,a,alen,w0,w1,w2)=6param
  → **invalid-IR**. 격리 확인: **5번째+ param 깨짐**(5-param IR서 %a4가 i8* 오타입=p0 타입 wrap-around, 5th alloca/store
  누락). **#3 갭 확정: >4 파라미터.** self-host 코어 `gen_stmts/gen_expr/gen_fold/gen_term` 모두 **8 param**, name_eq 5/kw3 6
  = string/List 빌딩블록 있어도 4-param 한계가 차단.
- **해결경로(mechanical 4→8 확장)**: Fn struct p0-p7+ty, build_fns 파람루프, emit_fn 시그니처/슬롯루프, call-arg a0-a7,
  gen_factor 호출 — 8-way로. 새 아키텍처 아님(슬롯 배수)이나 ~10 site 광범위 → dedicated(half-done 회피).
- 이번 iter 경계 정밀식별+#3 문서화. e2e 70(회귀0), 값정확성 96/96. 교훈: **양대 갭 해결 후 실제 self-host 함수가
  다음 갭(8 param) 노출**(building block 있어도 용량 한계)/#3는 mechanical(아키텍처 아님)이나 광범위=dedicated.
  **부트스트랩 경로: #1 string param✅ #2 List param✅(읽기) #3 >4 param(다음) — self-host 코어 함수 직결.**

## 2026-06-07 (/loop iter 85: 🎯🎯🎯 #3 부트스트랩 갭 해결 — 4→8 파라미터 (REAL self-host 함수 동작))
- **#3 갭(>4 param) 4→8 확장 dedicated 완료**. mechanical 확장 ~10 site: Fn struct p0-p7+ty(8), build_fns 파람루프
  8-way(decls+type-`:`branch+name-branch+push), emit_fn 시그니처+슬롯루프 8-way, call-arg a0-a7 저장+emit 8-way,
  gen_factor 호출 8 args. 격리후통합, 단계별.
- 실측: **5/6/8-param 동작**(s5=15/s6=21/s8=36), **🎯🎯 REAL self-host `name_eq`(5param, string param+byte loop)=1/0,
  `kw3`(6param, keyword 인식)=1** — 실제 파서 identifier-matcher/keyword recognizer 컴파일+실행. 회귀0(0-4param/
  string param/List param 전부 유지). e2e fixpoint-full **70→74 PASS**, aggregate 96/96.
- SELF_HOST.md FP12o(0-8 params), ROADMAP #3 해결. 교훈: **mechanical 확장도 단계별+헤비회귀**(~10 site 8-way 무회귀)/
  **#1/#2/#3 부트스트랩 3대 갭 전부 해결**(string param 176곳/List param 177곳/8param). **self-host 코어 함수 시그니처
  (tokenize/name_eq/kw3/gen_stmts/gen_expr) 전부 fixpoint_full로 컴파일.** 다음=실제 self-host 함수 여러개 조합(tokenize+
  name_eq+kw3 통합 미니렉서) 먹여 다음 경계, 또는 List 반환(out-param).

## 2026-06-07 (/loop iter 86: 다함수 미니렉서 동작 + #4 갭 식별 — List 반환)
- 3대 갭 해결 후 **다함수 실제 self-host 미니렉서 동작 확인**: is_d/is_a/kw_let 헬퍼 + `lex(s: Str)` 스캔이
  string param 위에서 keyword 카운트=2. **tokenize→consume 파이프라인 동작**: build List<token> 로컬(string 스캔)
  +SEPARATE consumer에 List-param 전달=4. self-host tokenize→parse shape 컴파일.
- **#4 갭 식별: List 반환**(`fn tokenize(src) -> List<Token>`). list-return invalid-IR(스택버퍼 escape). **self-host
  28 함수가 List 반환**(tokenize/lex/build_fns/build_defs=토크나이저/파서 주출력). **검증된 스킴(clang run=43):
  return-as-out-param**(hidden i64* out arg, callee가 out에 원소+len@[63] write, 호출자 버퍼할당+읽음). **회피 NOW:
  tokenize-pipeline 동작하므로 out-param 재작성으로 우회 가능**(직접 List-return은 편의).
- 경계 정밀식별+#4 문서화(검증된 스킴). 해결경로=List-param과 같은 패턴(build_fns ret타입 감지+emit_fn out-arg+
  return-copy+call 버퍼할당). dedicated. e2e 74 회귀0, 값정확성 96/96. 교훈: **3대 갭 해결 후 다함수 미니렉서/
  파이프라인 동작**(능력 통합 실증)/#4 List-return은 우회(out-param) 존재=비-blocking, 직접반환은 편의/검증된 스킴
  먼저. **부트스트랩: #1/#2/#3 해결, #4 List-return 식별(우회존재).**

## 2026-06-07 (/loop iter 87: 🎯🎯 out-param 패턴 완전 동작 — push-to-List-param + bare call stmt)
- #4 List-return을 out-param으로 우회. **2 갭 발견·해결**: ①**push-to-List-param**(List param에 `out.push(x)`가 안 됐음=
  읽기전용이었음; is_arr=4 write-through 추가: ptr 로드+len@[63]+ptr[len] store+len+1@[63]) ②**bare call statement**
  (`fill(xs, 3);` 버려지는 결과 호출이 gen_stmts서 drop됐음; `nx.kind == 9` 케이스 추가=gen_expr로 emit+discard).
- 실측: bare call 발화(beep()→stdout 'A'), push-param-callee-len=3, **caller-sees-elements=6**(호출자가 callee 푸시한
  원소 읽음), **🎯🎯 FULL out-param 토크나이저 `fn tokenize(src: Str, out: List<Int>)`**=197(src 스캔+out 채움+호출자
  토큰 읽음). **List-return 우회 완전 동작.** 회귀0, e2e fixpoint-full **74→77 PASS**, aggregate 96/96.
- SELF_HOST.md FP12p(bare call stmt+List push/write-through), ROADMAP #4 우회동작. 교훈: **out-param이 List-return
  우회**(push-to-param+bare-call-stmt 2갭이 진짜 막던 것)/List param은 읽기만이 아니라 push도 필요/bare call stmt가
  drop되던 잠복버그(let/return 위치만 emit). **🎯🎯🎯 부트스트랩 양대 패턴(읽기 List param + 쓰기 out-param) 완전 동작
  = self-host tokenize 완전한 shape(src param→out List 채움) 컴파일.** 다음=실제 self-host tokenize 함수 통째 또는 eval.

## 2026-06-07 (/loop iter 88: 🎯🎯 out-param 파이프라인 완성 — List 길이전파 + tokenize→consume)
- 통합 미니컴파일러 경계: tokenize(out)→classify(List-param) 파이프라인이 0(실패). 근본: **List 길이전파 누락**
  (로컬 List len=`%v<slot+1>` vs param len=buf[63] 불일치; out-param 호출 후 호출자 `.len`이 stale, 다음 함수 전달 시
  stale len을 buf[63]에 덮어씀→0). **수정: sync_list_len 헬퍼**(call 후 호출자 `%v<slot+1>`=load buf[63]; List-arg
  슬롯 ls0-ls3 추적+call 후 동기화).
- 실측: **caller-len-after-fill=3**(out-param 후 호출자 len 반영), **🎯🎯 tokenize→consume 파이프라인 2-함수=3**
  (tokenize List 채움→count_digits 스캔). 회귀0(list-param-read=2, out-param-elements=6). e2e fixpoint-full **77→79**,
  aggregate 96/96. SELF_HOST.md FP12q, ROADMAP out-param 파이프라인 완성.
- 교훈: **List 길이 2-convention(로컬 slot vs param buf[63]) 동기화 필요**(out-param 후 sync)/통합 파이프라인이
  길이전파 갭 노출(단일 함수선 안 보임). **🎯🎯🎯 부트스트랩 List 의미론 전부 동작**(읽기 List param/쓰기 out-param/
  길이전파)=self-host tokenize→parse 파이프라인(함수간 List 전달) 완전 컴파일. 다음=실제 self-host 다함수 통째 또는 eval.
