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
3 게이트 green: 값-정확성 25/25, 트랜스파일러-단위 19/19, nl-check-단위 11/11.
AI-written nl 25/25 컴파일+실행, self-correction 1라운드 수렴 실측.

**다음 = L3 (자체 컴파일러)** — 아래 'L3 진입' 참조. **사용자 결정 필요**(추측 금지):
컴파일러 작성 언어 / 백엔드 전략 / 에러 day-1. /loop는 여기서 사용자에게 escalate.

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
- Vais Vec를 sub/재귀 fn에 전달 불가 (task_54658a43): by-value=E022 move, &Vec=clang fat-ptr 불일치.
  재귀하향 파서 막힘 → 단일함수 인덱스로 우회.
- Vais `&&`/`||` 비단락평가 (task_492f7e17): `i<n && arr[i]` 가 i==n서 crash.
  nl lexer는 nested-if로 우회 중. 근본은 Vais codegen(논리연산→분기). 심각도 높음.

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
- ...최종: nl이 자기 일부 소스 컴파일 (fixpoint 근접). 현 인터프리터=값 평가; 진짜 fixpoint는
      전체 nl 문법 파싱+실제 codegen 필요(L3 엔드게임, 큰 단계 — 사용자 escalate 대상).

## 완료 정의 충족 상태 (2026-06-06)
P0~P5 + L3(self-host 미니컴파일러) + CX1~CX9 = **DONE**. ROADMAP 완료정의(L3+코퍼스37+에러인프라
nl-check+std시작 PRELUDE+게이트3종) **충족**. 이후는 (a) 인터프리터 표현력 추가 확장 또는
(b) 진짜 fixpoint(전체 문법+codegen)=사용자 결정 필요한 큰 단계.

전략: 단일파일/인덱스로 Vais 버그(Vec-재귀전달/&&단락) 회피 유지. 큰 관문(CX5+)서 막히면
자체 codegen 또는 Vais 수정 필요성 사용자 escalate.

## 진행 규칙 (/loop)
1. 큐 맨 위 미완료 task 실행 → 값-정확성/러너 green 확인 → 커밋 → 체크.
2. 막히면 TRACKED로 옮기고 다음 task.
3. P0~P5 다 끝나면 L3 결정을 사용자에게 escalate (추측으로 L3 시작 안 함).
4. WORKLOG.md에 각 iteration 기록.
