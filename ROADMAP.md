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

## ⚑ 상태 (2026-06-09): 🎯🎯🎯🎯🎯 실제 소스 부트스트랩 — full-source + stage 비교 oracle 통과

P0 게이트 / P1 코퍼스 / P2 트랜스파일러 / P3 에러인프라 / P4 std시작 / P5 레퍼런스 = **DONE**.
L3(self-host) + CX1~9 + FIXPOINT(FP1~FP12f) = **DONE**.

**🎯 실제 소스 부트스트랩 arc 정점(2026-06-08~09, FP12g~qq)**: fixpoint_full(통합 nl-self-host 컴파일러)이
**세 self-host 언어 tier를 전부 source string→value로 end-to-end 컴파일**:
- **①산술식**(fixpoint.nl): tokenize+eval, `2 + 3 * 4`=14 (FP12y)
- **②산술+변수**(fixpoint2.nl): 심볼테이블+변수평가, `let x = 2; let y = x + 1; return x + y * 4`=14 (FP12bb)
- **③재귀 함수언어**(fixpoint3.nl): 함수테이블+fresh-scope+if-expr+재귀, `fn fac(n) {{ return if n <= 1 then 1 else n * fac(n - 1) }} return fac(5)`=120 (FP12ee)
부트스트랩 갭 #1~#5b(string/List param, List-of-structs 로컬+param, typed let, bool, `!=`, let-bind-LOS, else-if-in-loop 등 20 능력추가) 전부 해결.

**현재 게이트 상태**:
- self-host e2e `scripts/test-fixpoint-full.sh` **OK** (struct ABI/List alias/metadata/division +
  direct double-string/List<Struct> call authority 회귀 포함).
- long self-host gate `scripts/test-fixpoint-full-self.sh` **OK**:
  - 실제 `compiler/self/fixpoint_full.nl` 전체 소스 → generated compiler IR `990159` bytes, `@main` 1개,
    negative GEP 0개 → clang/run → emitted binary exit **42**.
  - 그 1세대 컴파일러를 실제 `compiler/self/fixpoint.nl`로 retarget → generated compiler IR `994779` bytes,
    `@main` 1개, negative GEP 0개 → emitted compiler 실행 → final IR `ret i64 24` → clang/run exit **24**.
  - 같은 1세대 retarget을 `compiler/self/fixpoint2.nl`/`fixpoint3.nl`까지 확대:
    generated compiler IR `1000891`/`1012621` bytes → final IR `ret i64 50`/`ret i64 120` → clang/run exit **50**/**120**.
  - 1세대 retarget을 `compiler/self/fixpoint_full.nl` 자체까지 확대:
    generated compiler IR `1103434` bytes → 2세대 compiler 실행 → final IR `ret i64 42` → clang/run exit **42**.
  - `tools/normalize_stage_ir.py`로 source-position 기반 `@.sNNN`/`@.fmtNNN` global 이름만 정규화한 뒤,
    stage1 compiler IR과 stage2 compiler IR을 byte-compare → normalized `989685` bytes 일치.
- 값-정확성 aggregate **111/111** (예제코퍼스 93/93 + self-host codegen 모듈).
- 트랜스파일러-단위/nl-check-단위 유지.

**완료 정의(L3+코퍼스+에러인프라+std) = nl측 충족 + 실제 소스 부트스트랩 핵심 tier 전부 end-to-end.** 남은 것:
1. ~~**fixpoint.nl 편의갭 `-> List` 직접반환**~~ **✅ 근본해결**(FP12hh, 2026-06-08, commit 858defe+1cd0bef): `fn build() -> List<T> {{ ...; return xs }}`가 hidden out-param(caller가 버퍼 할당, callee가 `return xs`서 버퍼 복사)으로 컴파일=**fixpoint.nl 원형 `fn tokenize(src) -> List<Token>` 복원**(gap #4). 실측 scalar/LOS/arg/tokenize-shape 전부 동작.
1b. ~~**general-nl `for` 구문 codegen**~~ **✅ 해결**(FP12jj, 2026-06-08): `for v in lo..hi { body }`(exclusive)/`..=`(inclusive)를 induction-variable while-loop로 desugar(토크나이저 for=34/in=35/..=36/..==37, gen_stmts kind==34 핸들러, 두 slot collector에 루프변수 예약). 실측 exclusive/inclusive/중첩/if-in-body/List-push/표현식상한/외부var의존 전부 동작. = **fixpoint_full이 이제 모든 일반 nl 구문을 codegen**.
1c. ~~**`print(...)` 보간 codegen (fixpoint.nl codegen 단계)**~~ **✅ 해결**(FP12kk~ll, 2026-06-08): 재평가로 비-critical→진짜 갭 격상(fixpoint.nl/fixpoint_codegen의 codegen 단계가 `print("ret i64 {value}")`/`print("%t{counter} = {op_s}...")`로 IR을 emit=핵심 emission). 트랜스파일러가 print→puts rewrite; brace-bearing 리터럴은 `@.fmt<nstart>` 포맷글로벌+`printf`로 라우팅(`{ident}`→`%d` 또는 Str `%s`, `%`→`%%`, trailing `\n`). **lone-`{` vs `{ident}` 모호성**(transpiler가 둘 다 단일 brace로 전달) 해결=Vais lexer 규칙(`{`+식별자+`}`만 보간, lone `{`=리터럴; interp_end 단일구현 3곳 공유). FP12ll이 포맷 전역 pre-pass를 함수 metadata 뒤로 옮겨 Str 파라미터/local string let을 `%s` + `i8*` vararg로 emit, self-host `{op_s}` gap을 닫음. 실측 stdout 9종+capstone(fixpoint.nl 원형 tokenize→eval→emit_ir 통째 컴파일→`2+3*4`→`define i64 @main() {{ ret i64 14 }}` IR emit) + `%s/i8*` IR sanity. = **fixpoint_full이 self-host codegen 단계까지 컴파일=front+codegen 전체 arc.**
1d. ~~**실제 `fixpoint.nl` source-file bootstrap smoke**~~ **✅ 첫 파일 게이트 해결**(FP12mm, 2026-06-08): `tools/embed_self_source.py`가 실제 `compiler/self/fixpoint.nl` 파일을 현재 compact self-host subset으로 정규화(comments 제거, double-string→backtick, struct field type 제거, semicolon 보강, outer brace escape)해 `fixpoint_full`의 `compile("...")` 입력으로 주입. `fn main()`이 있는 실제 파일에서 duplicate `@main`이 나던 갭을 `has_top_stmts`로 해결(top-level 실행문이 없으면 synthetic wrapper 생략). 실측: 정규화된 실제 `fixpoint.nl` → fixpoint_full compile → generated compiler IR `@main` 1개 → clang/run → `ret i64 24` LLVM IR emit → emitted IR clang/run exit 24. = **snippet 통합에서 실제 파일 입력 자동 게이트로 한 단계 상승.**
1e. ~~**실제 `fixpoint2.nl` source-file bootstrap smoke + 10-param arity**~~ **✅ 두 번째 파일 게이트 해결**(FP12nn, 2026-06-08): 실제 `compiler/self/fixpoint2.nl` 원본의 `word_is(src, a, alen, w0, w1, w2, w3, w4, w5, wlen)`가 10개 파라미터를 쓰면서 기존 8-param 슬롯 한계를 노출. `Fn` metadata/타입판정/signature/call arg capture/param alloca/post-call List length sync를 p8/p9까지 확장하고, `s10`, late `List` out-param, 실제 `word_is("return", ...)` shape를 회귀 가드로 추가. 실측: 정규화된 실제 `fixpoint2.nl` → fixpoint_full compile → generated compiler IR `@main` 1개 → clang/run → `ret i64 50` LLVM IR emit → emitted IR clang/run exit 50. = **실제 파일 입력 자동 게이트가 산술 tier에서 산술+변수 tier로 확장.**
1f. ~~**실제 `fixpoint3.nl` source-file bootstrap smoke**~~ **✅ 세 번째 파일 게이트 해결**(FP12oo, 2026-06-08): 실제 `compiler/self/fixpoint3.nl` 원본의 multi-line `Fn` struct/`fns.push(Fn { ... })`, nested string brace escape, 8-field `Fn` metadata, Str param retlist call, `List[index].field` scalar assignment, `-> List` void signature 갭을 해결. 실측: 정규화된 실제 `fixpoint3.nl` → fixpoint_full compile → generated compiler IR `@main` 1개 → clang/run → `ret i64 120` LLVM IR emit → emitted IR clang/run exit 120. = **실제 파일 입력 자동 게이트가 재귀 함수언어 tier까지 확장.**
1g. ~~**`fixpoint_full.nl` full-source probe — struct ABI blocker**~~ **✅ full-source 1세대 컴파일러 probe 통과**(FP12pp~qq, 2026-06-09): 실제 `compiler/self/fixpoint_full.nl` 전체를 source-file harness로 주입. embed helper의 regex replacement backslash 손실과 LLVM C-string `\`/`"` escaping 갭을 먼저 해결한 뒤, `emit_op(o: Op)`/`emit_binop(..., l: Op, r: Op)`/`gen_factor -> Op` 계열이 요구한 **struct-valued function params/returns**를 hidden out-param ABI로 지원. 추가로 List param alias, 10-arg `-> List` call path, internal `Fn`/`StructDef` metadata field lookup, `/` operator를 닫음. 실측: normalize/embed → `nl2vais` → `vaisc build` → generated compiler IR `954648` bytes, `@main` 1개, negative GEP 0개 → clang/run → emitted IR clang/run exit 42. = **현재 `fixpoint_full.nl` 전체 소스가 `fixpoint_full`로 컴파일되어 실행 가능한 컴파일러를 만들고, 그 컴파일러가 다시 nl 프로그램을 LLVM IR로 emit/run한다.**
2. ~~**TRACKED 컴파일러 버그 2건**~~ **✅ 둘 다 근본수정**(2026-06-08): (ⓐ) all-return if/else-if 빈 merge block → block_returns()+`unreachable`(FP12gg, 580ca3a); (ⓑ) `.len` on List-of-structs가 struct-field GEP로 오인 → struct-field branch에 `karr != 2` 가드(FP12ff, 0d64afb). 둘 다 task chip dismiss.
3. ~~**반복 self-host/fixpoint 긴 게이트 자동화**~~ **✅ 해결**(2026-06-09): `scripts/test-fixpoint-full-self.sh`
   추가. 최종 `fixpoint_full.nl` 3.9k줄 full-source 1세대 compiler probe뿐 아니라, generated compiler가 실제
   `fixpoint.nl`/`fixpoint2.nl`/`fixpoint3.nl`/`fixpoint_full.nl` 파일을 다시 입력받아 final IR `ret i64 24`/`50`/`120`/`42`를
   emit/run하는 반복 경로까지 자동화. 이 과정에서 4-field `Token` List cap을 65536으로 확장하고,
   함수 호출 인자 emit은 callee param type(`Int`/`Str`/`List`/`Struct`) 기준으로만 struct pointer를 전달하게 보강.
4. ~~**stage output 비교용 stable oracle 정의**~~ **✅ 해결**(2026-06-09): `tools/normalize_stage_ir.py`가
   source-position 기반 LLVM string global 이름(`@.sNNN`, `@.fmtNNN`)만 occurrence-order 이름으로 정규화하고,
   `scripts/test-fixpoint-full-self.sh`가 최초 full-source stage1 compiler IR과 `fixpoint_full.nl` retarget으로 생성된
   stage2 compiler IR을 byte-compare한다. 실측 normalized IR `989685` bytes 완전 일치. 이 과정에서 double-quoted
   string literal decode를 fixpoint compiler에 반영하고, 호출 인자 emit은 callee의 `List<Struct>`/struct param 타입을
   caller slot lookup보다 우선해 stage drift를 제거. `scripts/test-fixpoint-full.sh`에도 두 원인을 직접 찌르는
   짧은 회귀 fixture를 추가.
5. **점진 인프라**(코퍼스 확장 / nl측 갭 수정 / cold-start 재측정) — scale-blocked 아님.
6. **Vais 백엔드/파서 갭**(TRACKED, 근본=Vais repo) — 리스트 리터럴 직접 인자는 해결, 잔여 Map/int→string/중첩Vec/Vec성장 등.

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
- [x] **T5. 중첩 `match` arm**: `Pattern => match ... { ... }`를 block arm으로 구조 pre-pass.

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
- ✅ Vais filter 버그(task_7cfebeba) — 해결 확인(2026-06-11): `.filter()` + `.sum()`이 nl `d6run`
  값 검증으로 통과. nl transpiler는 Bool predicate를 Vais stdlib의 i64 predicate ABI로 감싼다.
- 트랜스파일 천장(원천적): P8 캡처 클로저 반환은 Vais production ABI로 해결됐지만, P7 단일coercion /
  P4 에러 / 전체 day-1 일관성은 여전히 L3 자체 컴파일러에서만 근본 해결.
  (현재 큐는 L3 프론트엔드 진입 전 인프라 다지기.)

---

## TRACKED 추가 (Vais 버그)
- ✅ **Vais 캡처 클로저 반환/전달 ABI — 해결 확인**(2026-06-11, compiler): production `vaisc build`
  경로가 캡처 클로저를 tagged `{code, env}` 객체로 넘기고, bare fn-ptr와 공존. `fn adder(n) -> fn(Int)->Int { return |x| x + n }`
  및 self-capturing callback 통과. 회귀: Vais `phase253_closure_capture_boundary`, nl `e80_closure_return`,
  `e81_closure_return_apply`.
- ✅ **Vais 재귀(자기참조) enum codegen 버그 — 해결 확인**(2026-06-11, compiler): `enum Expr { Lit(Int), Add(Expr, Expr), ... }`
  payload layout 계산 전에 enum backing type을 선등록해 recursive payload를 i64 fallback 대신 enum struct로 낮춘다.
  회귀: Vais `phase266_recursive_enum`, nl `e50_ast_eval`.
- ✅ **Vais enum payload 안의 enum payload ICE — 해결 확인**(2026-06-11, compiler): `enum Wrap { Has(Option<Int>) }`
  처럼 enum variant payload가 또 다른 enum일 때 payload를 꺼내 `match`하던 경로를 user enum single-field
  payload resolved-type tracking으로 수정. 회귀: Vais `phase265_enum_nested_payload`, nl `e79_nested_match`.
- ✅ **Vais `impl Trait for Type` 문법 — 해결 확인**(2026-06-11, compiler `78abb89c`):
  `impl Area for Sq { ... }`를 기존 내부형 `impl Sq: Area { ... }`와 같은 AST로 파싱. 회귀:
  Vais `phase264_impl_trait_for_type`, nl `e78_trait_impl_for`.
- ✅ **Vais Vec 성장(push/map/filter) — 해결 확인**(2026-06-11, compiler `83c7b3a6`): 현재 compiler는 `Vec.new()+push`
  성장, `Vec.map(...).fold(...)`, `Vec.filter(...).fold(...)` full build/run 통과. nl은 빈 `List<T>`를
  `Vec::new()`으로 만들고 `.sum()`을 `.fold(...)`로 낮춘다. 회귀: Vais `phase261_vec_collection_methods`,
  nl `d6run`, `e75_list_push`, `e76_list_map`. PRELUDE push/map/filter 승격.
- ✅ **Vais 중첩 Vec codegen 버그 — 해결 확인**(2026-06-11, compiler `8e0719f2`): `Vec<Vec<i64>>`
  리터럴이 outer backing buffer에 inner data pointer만 저장해 `rows[1]`에서 40바이트 복사가 garbage/segfault로
  이어지던 문제를, expected `Vec<T>` 기반 typed literal materialization으로 수정. 회귀: Vais
  `phase262_nested_vec_literals`, nl `e77_nested_list`.
- ✅ **Vais 리스트-리터럴 직접 인자 코어션 갭 — 해결**(2026-06-10, compiler): `f([1,2,3])`
  (리터럴을 `Vec<T>` 파라미터에 직접 전달)이 기대 타입 기반 typecheck + inline Vec materialization으로
  build/run 통과. 회귀: `e2e_phase256_vec_literal_direct_arg_full_build` exit 37.
  전체 Vais `scripts/check-integrity.sh`도 `INTEGRITY OK`.
- ✅ **Vais 표면 int→string 변환 — 해결**(2026-06-10, compiler `0824cfdf`): `to_string(i64) -> str`와
  `n.to_string()` lowering을 `snprintf` 기반 str fat pointer 생성으로 추가. `str(42)`는 여전히
  타입키워드 호출이라 canonical 표면은 `to_string(42)`이며, nl은 `Str(x)`를 `to_string(x)`로
  트랜스파일한다. 회귀: Vais `phase259_int_to_string`, nl `e73_int_to_string`; nl-check는
  Rust식 `.to_string()`을 `Str(expr)`로 안내.
- ✅ **Vais HashMap/Map 기본 경로 — 해결 확인**(2026-06-11, compiler `835c9672`): `HashMap<u64,u64>.new()`
  모노모픽화 누락(B-02)은 build/run `exit=200`으로 통과하고, `get_opt(&key)`의 `&K -> K` silent path(B-01)는
  type checker가 `expected u64, found &u64`로 거부한다. nl `Map<K,V>`는 `HashMap<K,V>`로 트랜스파일하며
  `e74_map_basic`으로 PRELUDE 기본 insert/get_opt 승격. 회귀: Vais `phase260_hashmap_regressions`,
  nl `e74_map_basic`; nl-check는 Rust식 `HashMap`을 `Map`으로 안내.
- ✅ **Vais &Vec borrow 재귀 — 해결**(2026-06-06, compiler 214c97cf): `&Vec<T>`가 슬라이스 fat-ptr로
  잘못 codegen되던 버그 근본 수정 → 이제 주소 전달. **nl이 `&List<T>` borrow로 Vec 재귀 가능**
  (e15_list_recursion 실측 10). fixpoint(AST 순회)의 핵심 기반 확보. by-value=E022 move는 여전(설계상
  move 시맨틱이 정상 — borrow가 정답). task_54658a43의 &Vec 측면 closed; by-value move는 의도된 동작.
- ✅ **Vais `&&`/`||` 단락 평가 — 해결**(2026-06-10, compiler 4fb16591):
  `false && rhs` / `true || rhs`에서 RHS side effect가 실행되던 eager codegen을 전용
  branch+phi lowering으로 교체. `logic.left`/`logic.rhs.done` predecessor 블록을 둬 nested
  control-flow expression 뒤에서도 PHI predecessor가 안정적으로 맞음. phase258 값-정확성 가드 2개 +
  `scripts/check-integrity.sh` `INTEGRITY OK`.
- ✅ **Vais 전역 Vec 리터럴 codegen — 해결 확인**(2026-06-11, compiler `efe806fe`):
  `G v: Vec<i64> = [..]`를 정적 backing array + Vec struct initializer로 emit. 회귀:
  Vais `phase263_global_vec_literals`.

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
      두 부분 완비. 당시에는 남은 갭을 규모로 봤고, 이후 FP12pp~qq full-source probe에서 struct-valued function
      ABI까지 해소되어 1세대 compiler probe가 통과.**

**🎯🎯🎯 codegen 능력 완성 (2026-06-06)**: 통합 컴파일러 fixpoint_full.nl이 nl 컴파일러를 구성하는 **모든 핵심
구문**(함수 0-4param/재귀/중첩 + 가변변수 + while + if/else + 배열 + 동적List + struct + putchar + 문자열 s[i]/
s.len())을 단일 프로그램에서 네이티브 LLVM IR로 생성. **동작하는 토크나이저(ntok)와 파서 키워드인식(kw)을 실증.**
self-host 핵심 능력 전부 달성. 이후 FP12pp full-source probe에서 실제 `fixpoint_full.nl`은 generated IR까지 전진했고,
FP12qq에서 **struct-valued function params/returns(`Op`) ABI**까지 해결되어 full-source 1세대 compiler probe가 통과.

## TRACKED (full-source bootstrap next gate)
- ✅ **stage comparison oracle 해결**(2026-06-09): long gate가 stage1/stage2 compiler IR을 비교한다.
  normalization 범위는 source-position 기반 `@.sNNN`/`@.fmtNNN` global 이름뿐이며, 그 외 IR은 byte-for-byte 일치해야 한다.
- **fixpoint_full 잔존 codegen 능력 갭 기록**(2026-06-07 경계매핑 이력; 대부분 후속 FP12에서 해결, 최신 full-source 상태는 상단 1g):
  ① 비교-as-value **해결**(FP12g). ② `(...)` 그룹화 **해결**(FP12h). ③ `>=`/`<=` 2-char 비교 **해결**(FP12i,
  단일토큰 kind29/30, sge/sle, value+조건 양쪽; is_digit `c>=48 and c<=57` shape). ④ `for` 루프 **해결**(FP12jj).
  ⑤ `and`/`or`를 값으로 **해결**(FP12j, kind31/32, precedence comparison보다 낮게=next_logical로 RHS 바운드,
  `and`→`and i64`/`or`→`or i64`; **완전한 is_digit `c>=48 and c<=57` 동작**). ⑥ if/while 조건의 compound
  `and`/`or`(`if a and b`) **해결**(FP12k, 조건 전체를 gen_expr로 값평가 후 `icmp ne 0` 분기; self-host 26곳=is_alpha/is_digit).
  표현식/제어흐름 codegen은 사실상 완비(함수/재귀/가변/while/if/else/배열/List/struct/문자열-local/비교/`>=<=`/
  `and or`-value/compound 조건/그룹화). **단, 2026-06-07 경계매핑서 #1 부트스트랩 갭 발견(아래).**
- **🎯 #1 부트스트랩 갭: 문자열 파라미터(`fn f(s: Str)`) — 대부분 해결**(FP12l). fixpoint_full이 이제 **string-as-parameter 지원**: param 타입주석(`:` kind16 tokenize + build_fns p0ty..p3ty), Str-param=i8* slot(is_arr=3), 문자열리터럴 arg를 i8* 포인터로 전달(Op kind2), `s[i]` byte-load. 실측 s[0]=65, name_eq shape(`s[a]==s[b]`)=1, s[i]+s[i+1]. **완전 해결**(FP12m): `s.len()` on param도 runtime strlen(NUL까지 walk)로 동작 → 함수가 source를 `s: Str`로 받아 `while i < s.len()`로 스캔 가능 = **self-host 토크나이저 시그니처(`fn tokenize(src: Str)`) 동작**. 실측: param-strlen=5, count-digits-param=3, **실제 토크나이저 ntok(s:Str) on param=3**. #1 부트스트랩 갭 닫힘.

- **🎯 #2 부트스트랩 갭: List 파라미터/반환(`fn f(toks: &List<Token>)`)** — fixpoint_full이 **List-as-parameter/return 미지원**. 현재 List=스택 `[64 x i64]` 버퍼+별도 length라 i64 calling convention으로 못 넘김(param이 i64 오타입, `return xs`가 List 안 넘김, 호출이 단일 i64 전달). **self-host 소스 `&List<Token>` param 177곳 사용**(`fn eval_term(toks: &List<Token>,..)`, `fn eval_expr(toks: &List<Token>,..)` 등 = parser/evaluator 코어, 문자열 param 176과 동급 최다). **핵심 통찰: self-host는 `&List`(by-reference)** = List 버퍼 **포인터** 전달이 자연스러운 해법(복사 아님). **해결경로(dedicated, 문자열보다 큼=아키텍처)**: ①embedded 문법 `&List<T>`/`List<T>` param 타입주석 ②List-param=버퍼 포인터(i64* 또는 전용) slot, 호출 시 버퍼 주소 전달, `xs[i]`/`xs.len`이 포인터 경유 ③`return xs`=List 반환(포인터/구조). String param(FP12l/m)과 같은 패턴이나 2-slot+by-ref라 더 복잡. **#1 다음 최대 갭, parser/eval 부트스트랩 직결.**
  - **Stage 1 완료(FP12n, a6e28fa)**: `List<T>` param 타입주석 파싱(build_fns, pty=2, `List<...>` 전체 skip).
  - **검증된 codegen 스킴(clang 실증, run=60)**: List param = **`i64* bufptr` + `i64 len` (2 args)**. callee는
    `getelementptr i64, i64* %buf, i64 <idx>` + load로 인덱스, `xs.len`=len arg. caller는 버퍼 base GEP
    (`getelementptr [N x i64],.., i64 0, i64 0`) + length 전달. is_arr=4(List-by-ptr) slot.
  - **남은 Stage 2(codegen, interlocking)**: emit_fn 시그니처(List param=`i64* %aN, i64 %a<N+1>`=2 arg position)
    + 슬롯(ptr+len) + call-site(버퍼ptr+len 2 arg emit) + gen_factor `[`/`.len` 포인터경로. **핵심 난점: List param이
    2 arg 위치 소비→`%aN` 번호 재매핑**(현 1:1 param↔arg 깨짐). dedicated 집중 필요(아키텍처, half-merge 위험).
    fixpoint_full List push는 read-only `&List`엔 불필요(self-host는 &List=읽기).
  - **Stage 2 완료(FP12n)**: List-param codegen 동작. emit_fn 시그니처(List=`i64* %aN`, 1 arg=`%aN` 재매핑 회피!),
    슬롯(is_arr=4, i64** alloca), call-site(로컬 List arg는 len을 buf[63]에 store + 버퍼 base GEP를 i64*=Op kind3),
    gen_factor `[`(GEP i64+load)/`.len`(buf[63] load) 포인터경로. **핵심 단순화: List param=1 i64* arg(len은 buf[63])**
    →`%aN` 1:1 유지(재매핑 불필요), 로컬 List 표현 무변경(call서만 buf[63] write)=회귀0. 실측: sum_list(xs)=60,
    first(xs)=7, cnt(xs)=3. **parser/eval 시그니처 `fn eval_expr(toks: List<Int>)` 동작.** e2e 67→70.
  - **잔존 sub-갭: List 반환(`fn build() -> List`)** — `return xs`가 List 안 넘김(스택버퍼 escape). self-host는
    tokenize가 List 반환(`fn tokenize(src) -> List<Token>`)하나, 회피: 호출자가 List 만들어 &로 넘기고 callee가 채움
    (out-param 패턴). List-param(읽기)은 done, List-return(쓰기)은 다음. **#2 갭 읽기측 완료.**
- **🎯 #3 부트스트랩 갭: >4 파라미터** — fixpoint_full이 **최대 4 param(p0-p3)**만 지원, 5번째+ 깨짐(타입 wrap-around,
  alloca/store 누락). **self-host 코어 함수는 최대 8 param**(`fn gen_stmts(toks,slots,fns,defs,src,i0,end,counter0)`,
  gen_expr/gen_fold/gen_term 모두 8; name_eq 5, kw3 6). string/List param 빌딩블록은 있으나 4-param 한계가 차단.
  **해결경로(mechanical, 4→8 확장)**: Fn struct p0-p7+ty(8), build_fns 파람루프 8-way, emit_fn 시그니처/슬롯루프 8-way,
  call-arg a0-a7 저장+emit 8-way, gen_factor 호출 8 args. 새 아키텍처 아님(슬롯 배수)=mechanical이나 ~10 site 광범위.
  dedicated 집중(half-done 상태 회피). **양대 갭(string/List param) 다음 차단요소, self-host 코어 직결.**
  - **해결됨(FP12o)**: 4→8 param 확장 완료(Fn struct p0-p7+ty, build_fns 8-way, emit_fn 시그니처+슬롯 8-way,
    call-arg a0-a7, gen_factor 8 args). mechanical 확장 ~10 site 무회귀. 실측: s8(8param)=36, **REAL self-host
    `name_eq`(5param)=1/0, `kw3`(6param)=1** 동작. **#1/#2/#3 = 부트스트랩 3대 갭 전부 해결**(string param/List param/8param).
    self-host 코어 함수 시그니처(tokenize/name_eq/kw3/gen_stmts/gen_expr) 전부 fixpoint_full로 컴파일.
- **🎯 #4 부트스트랩 갭: List 반환(`fn tokenize(src) -> List<Token>`)** — fixpoint_full이 **List-return 미지원**
  (`return xs` 스택버퍼 escape 못함). **self-host 28 함수가 List 반환**(tokenize/lex/build_fns/build_defs 등=토크나이저/
  파서 주출력). **검증된 스킴(clang run=43)**: return-as-out-param — `-> List` 함수에 hidden `i64* out` 첫 arg 추가,
  callee가 out 버퍼에 원소+len@[63] write, 호출자가 `[64 x i64]` 버퍼 할당+base 전달+이후 자기 버퍼서 읽음.
  **회피 가능(NOW)**: tokenize-pipeline(build List 로컬+consumer에 List-param 전달)은 이미 동작 → 자기 함수를
  `fn tokenize(out: List<Int>)`(out-param)로 재작성하면 우회. 직접 List-return은 편의(능력은 out-param으로 존재).
  **해결경로(dedicated, List-param과 같은 패턴)**: build_fns가 `-> List` ret타입 감지, emit_fn hidden out-arg,
  return-xs를 out-copy로, call-site 버퍼할당+바인딩. mechanical+아키텍처 혼합. **부트스트랩 4번째 갭(28곳, 우회존재).**
  - **우회 동작(FP12p)**: out-param 패턴 완전 동작. 2 갭 추가 해결: ①**push-to-List-param**(is_arr=4 write-through:
    ptr 로드+len@[63] 로드+ptr[len] store+len+1@[63]) ②**bare call statement**(버려지는 결과 호출도 emit;
    gen_stmts에 `name(args);` 케이스). 실측: **FULL out-param 토크나이저 `fn tokenize(src: Str, out: List<Int>)`**가
    src 스캔+out 채움, 호출자가 토큰 읽음=197. **List-return 우회 완전 동작** → self-host tokenize/build_fns 등을
    out-param으로 재작성하면 직접 List-return 불필요. 직접 List-return은 여전히 편의(미구현)이나 **부트스트랩 비-blocking**.
    e2e 74→77.
  - **out-param 파이프라인 완성(FP12q)**: List length-sync 추가. out-param 호출 후 호출자의 `xs.len`이 buf[63]서
    동기화(callee 푸시 반영) → 채운 List를 다음 함수에 다시 넘길 수 있음. **🎯🎯 FULL tokenize→consume 파이프라인
    2-함수 동작**: `tokenize(src: Str, out: List<Int>)` List 채움 → `count_digits(toks: List<Int>)` 스캔=3.
    sync_list_len 헬퍼(call 후 %v<slot+1>=buf[63]). e2e 77→79. **부트스트랩 List 의미론 전부 동작**(읽기/쓰기/길이전파).
- **🎯 #5 부트스트랩 갭: List-of-structs(`List<Token>` 원소가 통째 struct) — 로컬 해결**(FP12r, 2026-06-07,
  commit ae107ac). self-host 소스가 `toks.push(Token { kind, value, nstart, nlen })`로 **4-필드 Token struct를
  List에 통째 push**(tokenize 내 ~30곳, parser가 `toks[i].kind`로 소비) = 진짜 `List<Token>` 모양. 이전엔 List 원소가
  i64 1개라 struct push가 invalid-IR. **해결**: struct-원소 List는 연속 `[64*nfields x i64]` 버퍼(원소 stride=필드수),
  slot의 `sty`에 원소 struct-type 저장. ①`list_elem_sty()` = `list()` 시점에 첫 `name.push(Type{...})` 스캔으로 원소
  타입 추론(원소 타입은 나중에 나타남) ②slot 할당(add_local_slots+collect_top_slots): struct-원소 List=`[64*nf x i64]`
  ③push(gen_stmts): 각 필드를 `buf[len*nf + field_index]` 저장 후 len++ ④index+field(gen_factor): `toks[i].field`=
  `buf[i*nf + field_index]` load ⑤라우팅: struct-필드-쓰기(`p.field=e`)는 scalar struct(is_arr=0)만, List-of-structs
  (is_arr=2,sty>=0)는 push로 fall-through ⑥skip_factor: `name[idx].field`=1 factor(6토큰)로 확장(뒤 binop `+` 파싱).
  clang 스킴(연속 stride + dynamic-index field) 격리검증 후 구현. 실측: push/len, multi-term field-read 식, dynamic
  build→consume 루프(tokenize→eval 모양), 실제 4-필드 Token struct, **40-원소 스케일**(별도 length alloca라 buf[63]
  충돌 없음). e2e 79→83, 값정확성 96/96, 회귀0. **로컬 List-of-structs=parser/eval 부트스트랩 직결 핵심 능력.**
- **🎯🎯 #5b 부트스트랩 갭: List-of-structs PARAMETER — 해결**(FP12s, 2026-06-07, commit cb81aaa).
  **실제 self-host 토크나이저 full shape 컴파일**: `fn tok(s: Str, out: List<Token>)`가 string 스캔→Token struct를
  by-pointer out-param List에 push, 호출자가 `toks[j].kind`/`toks[j].value`로 소비(=self-host `tokenize(src) -> List<Token>`을
  out-param으로 재작성한 모양, List-return 우회). **length-slot 재설계**(buf[63]↔struct-stride 충돌 제거): struct-원소
  List 버퍼=`[64*nf+1 x i64]`, length를 데이터 영역 뒤 header slot `buf[64*nf]`에 저장(scalar List는 무변경=`[64 x i64]`,
  buf[63]); 4 버퍼 사이트(로컬 push GEP/로컬 index GEP/양 slot 할당자) 전부 `64*nf+1`로 타입 일치. clang 검증 run=179
  (40원소 param). **callee(is_arr=4,sty>=0)**: emit_fn이 list_elem_sty로 param 원소타입 추론(본문 `out.push(Type{})`),
  push=`ptr[len*nf+fi]`/index=`ptr[i*nf+fi]`/`.len`=`ptr[64*nf]`(stride-aware). **caller cross-function 추론**: 호출자
  본문에 push 없어 원소타입 불가시 → **call_arg_elem_sty+param_list_elem_sty**가 호출되는 callee의 `List<Struct>` param
  주석에서 추론(arg 위치를 callee param에 매칭), 양 slot 할당자서 fallback. pre-call write+sync_list_len도 stride-aware
  (length index+buffer size를 List-arg별 thread: scalar 63/64, struct 64*nf/64*nf+1). **트랜스파일러 근본수정**:
  expand_for_loops(line기반)가 while/for 본문 끝 찾을 때 brace를 세는데 **`#` 주석 안 brace도 셈** → 루프 본문 내
  주석에 unbalanced `{`(예 "... Type {")가 있으면 `}` 하나 더 소비→stray brace emit. 해당 주석들서 unbalanced brace 제거.
  실측: empty-fill-len=2, caller-reads-fields=31, callee-reads-own=100, **full tokenizer(out-param List<Token>)=3**.
  e2e 83→87, 값정확성 96/96, 회귀0. 교훈: clang 스킴 검증 먼저(run=179)/length를 데이터 뒤로(충돌제거)/caller cross-function
  추론은 callee param 주석서(struct 필드 추가 회피)/**트랜스파일러 brace-count가 주석 brace 포함=루프 본문 주석 brace 금지**.
- **🎯🎯 full tokenize→eval 파이프라인 — List<Token> 함수간 공유**(FP12t, 2026-06-07, commit ff72f0e).
  **완전한 self-host tokenize→parse/eval shape 컴파일**: tokenize가 `List<Token>` out-param 채움 → **별도 consumer 함수**가
  그 `List<Token>`을 param으로 받아 `toks[i].kind`로 디스패치. 여러 consumer가 한 List 공유(multi-pass). **근본수정=read-only
  List-of-structs param 추론**: `eval(toks: List<Token>)`는 읽기만(push 없음) → emit_fn이 본문 push-scan(list_elem_sty)으로
  원소타입 못 찾아 scalar로 처리(`.len`=buf[63], stride 1)→garbage. **수정: emit_fn이 param 자신의 `List<Type>` 주석에서
  원소타입 읽음**(param_list_elem_sty를 자기 함수+param 위치에; push하든 읽든 authoritative), push-scan은 fallback(List<Int> 등).
  실측: tokenize→eval sum=6, mini calc 디스패치=9, **2 consumer(count_nums+sum_nums) 한 List 공유=211**. e2e 87→90, 값정확성
  96/96, 회귀0. 교훈: **read-only param은 push-scan 불가→param 주석이 authoritative**(consumer 함수가 self-host 핵심 패턴).
- **🎯🎯 실제 소스 부트스트랩 착수 + 첫 갭 해결: typed let**(FP12u, 2026-06-07, commit 569cb51). 사용자 결정=실제 소스
  부트스트랩 본격 착수. **recon**: self-host 모듈 측정(fixpoint.nl=136줄=가장 작은 완전한 컴파일러=첫 타깃). fixpoint.nl을
  fixpoint_full에 먹이는 첫 경계 매핑 → **`&List` borrow+재귀+`&xs` call-site는 이미 동작**(갭 아님, 격리확인)/진짜 첫 블로커=
  **typed let `let mut toks: List<Token> = []`**(타입 주석이 RHS 위치 어긋나게 함→`let x: Int = 42`조차 %v-1). **fix=rhs_pos()
  헬퍼**(name 뒤 `: Type` 주석 건너뛰고 RHS 위치 반환, `List<...>`는 `>`까지) 3 let 핸들러(add_local_slots/gen_stmts/
  collect_top_slots) 전부 적용+다운스트림 npos+2/+3→vp. 추가: rhs_is_list가 `[]` 빈 리스트 리터럴(kind23+24) 인식/
  let_anno_elem_sty=`: List<Type>` 주석서 원소타입(빈 리스트 authoritative, push-scan→call-site fallback)/rhs_struct_type도
  rhs_pos 사용. 실측: typed scalar/mut/empty-list-of-structs/list()-of-structs/struct-lit/scalar-list/top-level + **실제
  self-host shape(typed-list 토크나이저 `let mut toks:List<Token>=[]`+push+consume)=6**. e2e 90→97, 값정확성 96/96, 회귀0.
  교훈: **recon이 실제 갭 정밀식별**(`&List`는 이미 OK=fragment 추정과 다름, typed-let이 진짜 첫 블로커)/typed-let은 RHS
  위치 단일근(rhs_pos)으로 3 핸들러 일괄. **남은 fixpoint.nl 갭=`-> List` 직접반환(#4, out-param 우회존재)/`for`(1곳)/print interp.**
- **🎯🎯 boolean 리터럴 true/false 해결 — 실제 digit-run 토크나이저 동작**(FP12v, 2026-06-07, commit 3f3fe76).
  fixpoint.nl의 실제 multi-digit tokenize 로직(nested `while go {{ ...; go = false }}`로 digit run을 한 토큰에 `v=v*10+(d-48)`
  누적)을 fixpoint_full에 먹였더니 %v-1. 격리: **`true`/`false`가 식별자(kind1)로 토큰화→gen_factor가 변수로딩으로 fall-through**
  →`let mut go = true`가 `load %v-1`(슬롯없음), `go = false` 동일. (`while go` bare-var 조건 자체는 이미 OK였음=초기 의심 틀림,
  digit-run probe가 매번 `while go`+`let mut go = true` 동반해 혼동.) **fix: gen_factor가 `true`/`false`를 정수 상수 1/0으로 인식**
  (nl bool=i64), 변수로딩 전. `let mut go=true`+`go=false` 양쪽 커버(assignment RHS도 gen_expr→gen_factor). 실측: bool-flag
  while, true/false-in-if, **실제 self-host digit-run 토크나이저(nested `while go` multi-digit 누적, `12a34`→토큰 12+34=46)**.
  building block(digit-run i/v advance, else-if +/digit 분기) 독립확인. e2e 97→101, 값정확성 96/96, 회귀0. 교훈: **실제 self-host
  로직 먹이기가 진짜 갭 노출**(true/false 리터럴=digit-run/scan 루프 필수)/초기 의심(while bare-var)은 격리로 정정(probe 동반변수 혼동).
  **남은 fixpoint.nl 갭=`-> List` 직접반환(#4 우회존재)/`for`(1곳)/print interp(3곳).**
- **🎯🎯🎯 실제 fixpoint.nl 토크나이저 통째 컴파일 — else-if 체인 in-loop 해결**(FP12w, 2026-06-07, commit efb1e94).
  fixpoint.nl 실제 tokenize(out-param 형: 4 token kinds, is_space/is_digit 헬퍼콜, 6-way `if/else if.../else` 디스패치)를
  먹였더니 오답. IR 격리: **`if A {{}} else if B {{}} else {{}}` 체인이 while 본문서 오lowering** — if-핸들러가 `else if`를
  plain `else {{...}}`로 취급→다음 `{{` 스캔이 inner then-block 브레이스에 착지→else-region이 그 블록만 덮음→마지막 `else`가
  무조건 실행(루프 1회). HEAD서 3-way도 깨짐(111 기대에 65)=회귀 아닌 잠복버그. **fix: if_stmt_end() 헬퍼**(완전한
  `if [cond] {{}} [else if {{}}]* [else {{}}]` 체인 끝 인덱스, `else if` 재귀)+if-핸들러가 `else if`면 else-body를 nested if
  statement로 gen_stmts 재귀. 실측: 3-way else-if in-loop(65→3), else-if call-cond, **실제 fixpoint.nl 토크나이저(`12+3*4`→5토큰
  값12+3+4=19, `99*100`→199, `5`→kind0 value5, `1+2-3*4`→7토큰)**. e2e 101→105, 값정확성 96/96, 회귀0. 교훈: **실제 self-host
  함수 통째가 진짜 마일스톤**(tokenize 완전동작=fragment 아님)/IR 격리로 else-region 오착지 규명/pre-existing 버그도 fix가 net개선.
  **🎯 실제 fixpoint.nl tokenize 완전 컴파일+실행.** TRACKED(pre-existing, 별개, 비-blocking): ①3+레벨 nested else-if 모든 branch가
  return시 빈 trailing merge block(invalid IR) ②`.len`+`[i].field` 혼합 multi-term 식(`toks.len*100+toks[0].value+...`) 오계산.
  남은 fixpoint.nl 갭=`-> List` 직접반환(#4 우회존재)/`for`(1곳)/print interp(3곳).
- **🎯🎯🎯 실제 fixpoint.nl evaluator 통째 컴파일+실행 — `let t = toks[i]` LOS-원소 바인딩 해결**(FP12x, 2026-06-07, commit d5c40ce).
  fixpoint.nl 재귀 evaluator(eval_term/eval_expr/eval_expr_fold/skip_term, `List<Token>` param 재귀) 먹였더니 invalid IR(%v-N).
  IR 격리: **`let t = toks[i]`가 List-of-structs 원소를 local에 바인딩** 후 `t.kind`/`t.value` 필드읽기 — let-핸들러가 `toks[i]`를
  scalar RHS로 취급→`t`가 i64 1슬롯, 필드접근이 인접(틀린)슬롯 읽음. 핵심 eval패턴 `let t=toks[i]; if t.kind==2`. 직접 `toks[i].kind`는
  이미 OK. **fix: rhs_los_elem_sty()**(RHS가 `<listvar>[expr]`이고 listvar가 LOS면 원소타입 반환)+slot 할당이 struct local([nf x i64],
  sty=원소타입)+gen_stmts가 원소복사(필드k: `t[k]=buf[idx*nf+k]`, param이면 버퍼ptr 먼저 load)+add_local_slots서 `&slots` 전달(owned라
  by-value면 E022 move). 실측: local/param LOS-원소 바인딩+필드, **실제 fixpoint.nl evaluator(`2 + 3 * 4`=14 precedence, `10 - 2 * 3`=4
  좌결합+precedence)**. e2e 105→109, 값정확성 96/96, 회귀0. **🎯🎯🎯 fixpoint.nl tokenize+evaluator 양쪽 완전 컴파일+실행=가장 작은
  완전한 self-host 컴파일러의 핵심 동작.** 교훈: 실제 eval 먹이기가 `let t=toks[i]` 바인딩 갭 노출(직접접근과 별개)/owned slots는 `&` 전달
  필수(E022). 남은 fixpoint.nl 갭=`-> List` 직접반환(#4 우회)/`for`(1곳)/print interp(3곳)+emit_ir(putchar 이미 동작). collect_top_slots
  미적용(top-level `let t=toks[i]` self-host 미사용, defer).
- **🎯🎯🎯🎯 완전한 self-host tokenize+eval 파이프라인 end-to-end 실행**(FP12y, 2026-06-07, commit c78dd2e). **마일스톤(신규 능력 아님)**:
  fixpoint.nl의 tokenize+eval 컴파일러를 **단일 통합 프로그램**으로 fixpoint_full에 먹임 — `run(src: Str)`가 식 문자열을 `List<Token>`으로
  토큰화(out-param)→evaluator로 평가(List<Token> 재귀)→값 반환. FP12u(typed let)+FP12v(bool)+FP12w(tokenize)+FP12x(evaluator)가
  깨끗이 조합됨(신규 갭 0=능력이 이미 갖춰짐). 실측 end-to-end(source→tokenize→eval→value): `2+3*4`=14(precedence), `10 - 2 * 3`=4
  (좌결합+precedence+공백), `7 + 8 + 9`=24(좌-fold). **= 가장 작은 완전한 self-host 컴파일러(산술식 언어 토큰화+평가)가 fixpoint_full로
  컴파일된 단일 프로그램으로 실행 = 실제 소스 부트스트랩 arc의 목표 달성.** e2e 109→112(+3 통합 파이프라인 가드), 값정확성 96/96, 회귀0.
  남은 fixpoint.nl 갭(편의/비-blocking)=`-> List` 직접반환(#4 우회존재)/`for`(1곳)/print interp(3곳). TRACKED 2버그(task chip): deep
  nested else-if 빈 merge / multi-term `.len`+`[i].field` 식.
- **🎯🎯 `!=` 연산자 — name_eq + fixpoint2.nl 심볼테이블 unblock**(FP12z, 2026-06-07, commit a6c4882). 다음 큰 모듈 fixpoint2.nl
  (산술식+multi-char 변수, `List<Var>` 심볼테이블)로 확장 → lookup이 모든 이름에 -1 반환. IR 격리: **`!=`가 토크나이저에 완전 누락**
  (`!`(33) skip, 뒤 `=`가 assignment) → `src[a+k] != src[b+k]`가 `icmp ne ..., 0`로 lowering(RHS `src[b+k]` 드롭). name_eq(소스바이트
  동등=심볼테이블 키)와 그것이 쓰는 lookup 무음파손. **fix: 토크나이저 `!`+`=`→not-equal 토큰(kind33), bare `!`은 skip 유지; gen_fold
  비교 arm에 kind33 추가→`icmp ne`.** 실측: `!=` value/조건, name_eq(foo==foo→1, foo!=bar→0), **실제 fixpoint2.nl 심볼테이블
  (`List<Var>` name_eq lookup: "foo"→10, "bar"→20)**. e2e 112→115(+3 FP12z 가드), 값정확성 96/96, 회귀0. 교훈: 다음 모듈로 확장이 누락
  연산자(`!=`) 노출/`!=` 누락은 무음파손(RHS 드롭→`!=0`). **= fixpoint2.nl(다음 self-host tier: 산술+변수) 핵심 심볼테이블 동작.**
- **🎯🎯 변수 평가 — eval_factor가 ident를 lookup으로 해석**(FP12aa, 2026-06-07, commit 10f9cc4). **마일스톤(신규 능력 0)**:
  fixpoint2.nl 변수-해석 evaluator를 fixpoint_full에 먹임 — eval_factor가 num은 t.value, ident면 `lookup(vars,...)` 호출;
  `vars: List<Var>` 심볼테이블이 전 재귀체인(eval_expr→eval_fold→eval_term→eval_factor→lookup) 관통. **= 산술+변수 tier**(FP12z 위에).
  실측(변수 피연산자+precedence): `x + 3`(x=5)=8, `a * b`(a=4,b=6)=24, `x + y * 4`(x=2,y=3)=14. 컴파일러 변경 0 — 초기 probe 실패는
  test의 eval_expr off-by-one(실제 fixpoint2.nl은 eval_term i+1, eval_fold 피연산자 i+2)뿐, faithful 인덱싱이면 깨끗이 조합. deep
  `List<Var>` param 관통+ident-token 해석 동작 확인. e2e 115→117(+2 가드), 값정확성 96/96, 회귀0. **= fixpoint2.nl 산술+변수 핵심 동작.**
- **🎯🎯🎯🎯 완전한 산술+변수 컴파일러 end-to-end**(FP12bb, 2026-06-07, commit 185e75e). **마일스톤(신규 능력 0)**:
  fixpoint2.nl 전체를 **단일 통합 프로그램**으로 먹임 — `run_program(src)`가 multi-let 프로그램 토큰화(kw_let/kw_return 키워드인식
  +ident+num+`+ * = ;`)→`let` 바인딩으로 `List<Var>` 심볼테이블 빌드(변수가 이전 변수 참조 가능)→precedence로 평가→`return`값.
  실측 end-to-end(source→value): `let x = 5; return x + 3` =8, `let a = 4; let b = 6; return a * b` =24, **`let x = 2; let y = x + 1;
  return x + y * 4` =14**(y가 x 참조+precedence). **= 두 번째 self-host tier 완성: multi-char 변수 있는 언어의 가장 작은 완전한 self-host
  컴파일러 end-to-end.** FP12z(심볼테이블)+FP12aa(변수평가)+키워드토큰화 조합=신규 능력 0. e2e 117→120(+3 가드), 값정확성 96/96, 회귀0.
- **🎯🎯🎯 함수+재귀 (fixpoint3.nl, 3번째 tier)**(FP12cc, 2026-06-08, commit f890a43). fixpoint3.nl tier(multi-char 함수정의/호출+재귀)
  확장. fixpoint_full이 핵심 메커니즘 codegen: `List<Fn>` 함수테이블 find_fn 룩업; **eval_call이 인자를 fresh per-call `List<Var>`
  scope에 바인딩**하고 body를 lookup으로 평가; **재귀**(함수 body가 자기 호출, 매 호출 fresh scope). 실측: find_fn(이름→index 1),
  eval_call(fresh scope, double(5)=10), **재귀 fresh-scope eval(factorial(5)=120, sum(5)=15)**. **= fixpoint_full이 3개 self-host tier
  (산술/산술+변수/함수+재귀) 아키텍처 전부 codegen 가능 확증.** `List<Fn>`+`List<Var>` 테이블+fresh-scope 바인딩+재귀 전부 기존 기능 조합
  =신규 능력 0. e2e 120→123(+3 가드), 값정확성 96/96, 회귀0.
- **🎯🎯🎯🎯 완전한 함수언어 컴파일러 end-to-end (3번째 tier) = 3 tier 전부 end-to-end**(FP12dd, 2026-06-08, commit 21c9483).
  **마일스톤(신규 능력 0)**: fixpoint3.nl 함수언어 컴파일러를 **단일 통합 프로그램**으로 먹임 — `run_program(src)`가 함수언어 프로그램
  토큰화(fn/return 키워드+ident+num+`+ * ( ) {{ }} ;`)→`fn` 정의 스캔으로 `List<Fn>` 함수테이블 빌드(build_fns)→top-level
  `return <call>;` 찾아 eval_call(인자를 fresh callee `List<Var>` scope 바인딩+body 평가). 실측 end-to-end(source→value):
  `fn double(x) {{ return x * 2 }} return double(21)`=42, `fn sq(x) {{ return x * x }} return sq(7)`=49, **2-fn 테이블 두번째 호출
  `triple(14)`=42**(build_fns 다중 스캔+find_fn 디스패치). **= 🎯🎯🎯🎯 세 self-host tier(①산술 ②산술+변수 ③함수) 전부 단일 통합
  프로그램 source→value end-to-end.** FP12cc(List<Fn>/eval_call/fresh scope)+키워드토큰화 조합=신규 능력 0. e2e 123→126(+3 가드),
  값정확성 96/96, 회귀0. **다음 경계(tracked)**: run_program 재귀는 함수 body에 `if cond then a else b` **식**(expression) 필요
  (FP12cc 재귀 probe는 native nl 재귀 사용; source-level `fac(n) = if n<=1 then 1 else n*fac(n-1)`는 if-expr 평가기 필요=별개 갭).
- **🎯🎯🎯🎯🎯 재귀 함수언어 end-to-end — if-expression 평가기 + 재귀**(FP12ee, 2026-06-08, commit c6b9288). **가장 깊은 self-host
  통합**: source-level **재귀** 함수언어를 fixpoint_full이 단일 프로그램으로 컴파일. 함수 body=`return if <cond> then <a> else <b>`;
  eval_value가 **if-expression 평가**(then/else 찾기→조건을 비교연산자로 split→양변 평가→branch 선택); eval_call이 fresh per-call
  `List<Var>` scope로 body 재귀(재귀 호출이 else 분기에 위치). 실측 end-to-end(source→value): **`fn fac(n) {{ return if n <= 1 then 1
  else n * fac(n - 1) }} return fac(5)`=120**, `fac(4)`=24, `fn sum(n) {{ return if n <= 0 then 0 else n + sum(n - 1) }} return sum(10)`=55.
  **= 3번째 tier WITH 재귀 완성: 조건+재귀 호출 있는 함수언어=모든 self-host evaluator의 핵심 모양.** FP12dd(함수테이블+eval_call+fresh
  scope)+if-expr 평가기(if/then/else 토큰화+branch 선택)+call-arg/factor skipping(call 피연산자 처리) 조합. e2e 126→129(+3 가드),
  값정확성 96/96, 회귀0. 교훈: if-expr 평가기가 source-level 재귀 가능케(재귀=else 분기); 모든 빌딩블록(List<Fn>/List<Var>/fresh scope/
  재귀/if-expr/산술)이 조합돼 재귀 함수언어 완성. **세 self-host tier(산술/산술+변수/함수언어+재귀) 전부 source→value end-to-end.**
- (구) #1 갭 원문:
  현재 compile() 입력은 무타입 param(`fn f(s)`)이라 s가 문자열인지 시그니처로 모름 → param을 i64 slot으로 처리,
  문자열 리터럴 arg를 `0`으로 전달, `s[i]`가 array-GEP(오타입). **self-host 소스는 `Str` param을 176곳 사용**
  (`fn tokenize(src: Str)`, `fn name_eq(src: Str,..)` 등) = 가장 많이 쓰는 패턴, bootstrap-critical. 로컬 문자열
  (`let s = "..."`)은 동작(FP12c), param만 막힘. **해결 경로**: ①embedded 문법에 param 타입주석(`fn f(s: Str)`)
  추가(tokenize `:`+타입, build_fns 파싱) ②Str-param을 i8* slot+호출 arg 문자열-포인터 전달+`s[i]` byte-load.
  큰 작업(param-typing 인프라) → dedicated 진행 권장. **이게 self-tokenization 직전 최대 갭.**
- `for` 루프: self-host 미사용(0건) → 비-critical, general-nl용 TRACKED.
