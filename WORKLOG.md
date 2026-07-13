# Vais Worklog

## 2026-07-14 (도그푸딩 6 — vaisgrep 재귀 검색 + fs_list_dirs + 이중 재작성 3호)

**fs_list_dirs(dir, out) 승격**: fs_list_files 완전 미러 14지점(S_ISDIR만
다름 — declare/host runtime 리스트 계약/front 화이트리스트/direct predicate·
게이트 6그룹·out-param 검증·emission·skip·추론·문장스캔·static 헬퍼).
e342 양 엔진 42(정렬/파일 제외/누락 0), parity 361. full core 무변경.

**vaisgrep -r**: grep_tree가 fs_list_dirs로 레벨별 하강, 상대경로
prefix(`sub/deeper/c.md:2: ...`) 출력. workflow +2케이스.

**갭 노출→즉시 승격(이중 재작성 3호)**: direct에서 helper-call 인자 속
builtin call(`ident(path_join(dir, xs[j]))`)이 실패 — list_expr의 user-fn
분기가 인자를 전체 파이프라인으로 재작성한 출력을, 바깥 parse_builtin이
다시 스캔하며 내부 builtin 인자(`xs.data[...]`)를 재재작성. fix:
parse_builtin/str_conversion 패스가 **선언된 helper call 그룹을 opaque로
skip**(두 패스 모두 list_expr 이후에만 호출됨을 확인 후). 이름 재귀
트리워크가 이 fix로 개통.

환류: `@(...)` self-recursion이 call-인자/컴파운드 위치에서 양 엔진 미승격
(full=ptr 타입 불일치/direct=미번역) — 이름 재귀가 verified 표면, ROADMAP
후보 등록.

## 2026-07-13 (도그푸딩 5 — vaisgrep 두 번째 배포 도구 + fs_is_dir 승격)

vaisgrep 설치형 패키지(e341, grep.scan 모듈 + main): 파일/디렉토리 대상
서브스트링 라인 검색(`N: line`/`file:N: line`), `-c` 카운트 모드, 무인자
self-test 42. 패키지 바이너리 실측 전 케이스 정확(파일 2/디렉토리 3/카운트
3/누락 3/빈 패턴 1), workflow 게이트 +7케이스, parity 360.

**갭 노출→즉시 승격: fs_is_dir(path: Str) -> Bool.** fs_exists가 디렉토리
에도 1을 반환해 경로 디스패치가 fs_read_text에서 host trap — stat/S_ISDIR
기반 host API를 fs_exists 미러로 9지점 승격(HOST_INTRINSIC_IR/host runtime/
front 화이트리스트/direct predicate·parse_builtin·skip·추론·게이트·prototype).
full core 무변경.

트랩 재확인 2건: ① 문자열 리터럴에 `\n` 이스케이프 없음 — 멀티라인은
str_byte(10) concat(기존 e337 lf 패턴). ② str_split_lines_into는 할당형
(`let n = ...`)이 검증 표면 — bare 문장은 양 엔진 미인식.

## 2026-07-12e (List<Str> 정렬 표면 승격 — 도그푸딩 4 환류 완결)

환류 갭을 즉시 승격. 선행 갭부터: **List<Str> 원소 대입**이 양 엔진 미승격
— full은 원소-store 폴백이 ptr 값을 i64 슬롯에 그대로 store(clang 타입
에러), 기존 `ensure_i64_op` 헬퍼를 kind 2/3 값에 적용하는 6줄로 root-fix
(.ll 재생성, %v-1/var-folders 0 확인). direct는 원소 대입 게이트에 Str
1줄 추가로 기존 lvalue 재작성 경로가 그대로 동작.

**str_cmp(a,b)->Int 승격**(3-way -1/0/1): host runtime 구현 +
HOST_INTRINSIC_IR declare(full은 제네릭 call 경로가 (i8*,i8*)->i64 그대로
emit, core 무변경) + direct 10지점 배선(predicate/parse_builtin 6그룹/
skip 목록/타입 추론/게이트 체인/프렐류드 static) + front unknown-call
화이트리스트. **List<Str>.sort()**: 공유 sort 데수가의 비교 라인만
`str_cmp(%V,%K) > 0`으로 교체(2줄 diff) — 로컬/파라미터/빈 리스트 42.

적용: vaisdb `docs`가 사전순 출력(self-test에 순서 검증 2단계 추가).
e340 신설(str_cmp/원소 스왑/로컬·파라미터 sort, 양 엔진 42, parity 359).
PRELUDE/LANGUAGE 표·시그니처, README, CHANGELOG 반영.

## 2026-07-12d (도그푸딩 4 — vaisdb 문서 관리, 컴파일러 갭 0건)

vaisdb 패키지에 docs(고유 doc id 목록, exit=수)/remove(문서 키 전부 제거 —
Map 키 삭제 표면 없이 필터된 새 Map 재구축 후 저장)/stats(docs=N terms=M)
서브커맨드 추가. key_doc_id를 report→index 모듈로 이동해 순환 import 없이
공유. self-test 6단계 확장(목록 3/contains/제거 키 2/제거 후 2/ghost 0),
여전히 42. workflow 게이트 +5케이스(docs 2/stats 2/remove 0/제거후 docs 1/
ghost 3).

**컴파일러 갭 0건 — 첫 시도 양 엔진 42.** dedupe(seen map)/Map 재구축/
List<Str> push·contains·인덱스 출력 표면 전부 성숙 실측(도그푸딩 2에 이어
2번째 무갭 스프린트). 부수 정리: fs_mkdirs direct prototype const(경고 0).
환류: List<Str> 정렬 표면 부재(Str 비교/sort 미승격 — docs는 삽입 순서
출력)를 다음 후보로 등록.

## 2026-07-12c (full 엔진 미지 함수 front 진단)

직전 유닛의 부수 발견 승격: 존재하지 않는 함수 호출이 full에서 bare call로
emit되어 clang 단계 타입/링크 에러로 표면화되던 진단 갭. locus는
`check_front_contract_text`(병합+lowering 후 텍스트라 모듈 간 호출 안전,
trust root skip 유지). pass1에서 호출 가능 이름 수집(fn/pub fn + let/mut +
`name:` 파라미터·필드 — 클로저 보유 로컬 오탐 방지), pass2에서 `.` 리시버
없는 lowercase ident+`(`를 검사해 "call to an unknown function" front 거부.

오탐 방지 절차: 구현 전 코퍼스 전체(examples/tools/std) 사전 스윕으로
비선언 lowercase 호출명을 수집 → 진짜 빌트인 잔여는 bitand/bitor/bitnot뿐
임을 확인, 게이트에서 putchar(IO slice)/pub fn(public_struct) 2건 추가 발견
즉시 수정. front 게이트에 unknown_call reject 케이스 추가.

## 2026-07-12b (갭 승격 — List<Struct> 인덱스 필드 in 중첩 call 인자)

도그푸딩 3에서 환류한 direct 갭을 즉시 승격. 격리 이분탐색으로 실제 트리거는
"중첩 깊이"가 아니라 **`Str(xs[j].score)` — 변환 call 인자 위치의 인덱스 필드**
로 확정(파라미터/로컬 리시버 무관). 근본 원인: `direct_rewrite_list_expr`의
builtin-skip 목록에 `Str(...)`이 빠져 변환 인자 내부가 선(先)재작성되고,
`direct_rewrite_str_conversion_calls`가 그 인자를 다시 `direct_rewrite_expr`로
재귀 재작성하는 **이중 재작성**에서 `xs.data`를 List 메서드로 오인. fix는
skip 목록 1항 추가(1줄), full core 무변경.

검증: e339 신설(변환 인자 + 4중 중첩 + 로컬/파라미터 리시버, 양 엔진 42,
parity 358) + e337 rank_lines의 `let entry` 워크어라운드 제거(제품 코드 실증).
부수 발견 기록: full 엔진은 미지 함수 호출(`int_to_str` 오타 등)을 front에서
거부하지 않고 bare call로 emit해 clang 단계에서 혼란스러운 타입 에러가 남 —
진단 갭으로 다음 후보 등록.

## 2026-07-12 (도그푸딩 3 — fs_list_files 승격 + vaisdb ingest-dir/rank)

호스트 API 승격: `fs_list_files(dir, out: List<Str>) -> Int` —
HOST_INTRINSIC_IR declare + write_host_runtime_c 구현(opendir/readdir,
정규 파일만, qsort 정렬, 누락 dir=0, full 리스트 계약 out[4095]=len) +
direct 엔진 배선(빌트인 인식 4지점 포함 총 9지점 + static C 헬퍼).
full core(.ll)는 무변경 — 제네릭 call 경로가 (i8*, i64*) shape을 이미 emit.
부수 승격: fs_mkdirs를 direct에도(호스트 런타임 공유, prototype+emission).
e338 양 엔진 42, parity 357.

제품 도그푸딩: vaisdb 패키지에 `ingest-dir <index> <dir>`(.txt만,
doc-id=확장자 제거 — flat `docid.term` 키의 dot 충돌 회피)와
`rank <index> <query> <k>`(RankedDoc 수집 → **sort_by_desc 제품 실사용**
→ top-k 라인, exit=top score) 추가. workflow 게이트 +4케이스
(ingest-dir 0/누락 dir 3/rank 4/bad-k 1).

환류 갭 1건 등록: direct의 local List<Struct> 인덱스 필드를 중첩 call
인자로 쓰는 슬라이스 미승격(`str_concat(.., xs[j].field)`) — `let entry =
xs[j]` 바인딩으로 우회 가능, ROADMAP 다음 후보에 등록. 문서:
HOST_IO/PRELUDE/README/CHANGELOG. 스프린트 4/4 종료.

## 2026-07-10 (vaisdb 설치형 패키지 — 첫 배포 가능한 Vais 도구)

"richer package layout" 후보를 실제 제품으로 도그푸딩. tools/vaisdb_cli.vais의
로직을 다중 모듈 패키지로 재구성: examples/e337_vaisdb_cli_package/
(vais.toml: binary="vaisdb", source="src"; src/vaisdb/index.vais +
src/vaisdb/report.vais(import vaisdb.index 전이 import) + src/main.vais).
main은 무인자=결정적 self-test(42, parity/value 코퍼스 진입용, e326 패턴),
인자=ingest/query/report 디스패치(도구와 동일한 exit 프로토콜).

실측: scripts/vaisc package → dist/bin/vaisdb 빌드, 배포 바이너리로
ingest(0)/query(4)/report(4)/self-test(42) 전부 정확, --archive로
vaisdb-0.1.0.tar.gz 생성·추출·실행까지. **Vais로 작성한 첫 배포 가능한
실제 도구.** workflow 게이트에 8케이스(build/self-test/서브커맨드 3종/
아카이브 존재/추출/추출본 실행) 추가, parity(e337 main) 등록. 노출 갭 0건 —
패키징+모듈 표면도 실전 조합에서 성숙 확인. ROADMAP의 richer-layout 후보는
"현 표면 도그푸딩 완료, 추가 요구 노출 시 재개"로 갱신.

## 2026-07-10 (List<Struct>.sort_by/sort_by_desc 승격 — 갭 1호 완결)

sort() 전반부와 동일 패턴으로 후반부 완결. lower_list_method_text에
lower_list_sort_by_statement_line 추가 — `xs.sort_by(|x| x.int_field)`와
`sort_by_desc`를 e332가 손으로 증명한 선택정렬 shape(인덱스 필드 비교 +
temp struct local 경유 원소 스왑)로 desugar. 기존 헬퍼 재활용:
list_method_env_type(수신자)→list_method_list_element_type_copy(원소 struct)→
list_method_struct_field_type(필드가 Int인지 검증) — Int 키일 때만 발화.
비교 연산자만 desc 플래그로 분기(</>).

첫 시도 양 엔진 42(probe: desc/asc/param receiver/Str필드 struct/중복/빈/
세미콜론 한줄). .ll 무변경(driver-only). e336 등록(parity/value 355 예정),
LANGUAGE List<Struct> 행/PRELUDE/README/CHANGELOG, ROADMAP 갭 1호 완결 표기
(Str-key sort_by는 필요 노출 시 후속). e332의 수동 정렬은 역사적 갭 문서로
유지.

## 2026-07-10 (built-in List<Int>.sort() 승격 — 환류 갭 1호 전반부)

도그푸딩-2가 등록한 갭 1호를 승격. 설계 핵심: `xs.sort()` statement를 driver의
lower_list_method_text에 **단일 text rewrite**(lower_list_sort_statement_line)로
추가 — 검증된 표면(중첩 while + 원소 read/write)으로 된 삽입정렬 블록으로
desugar하며, 이 pass가 full/embed/direct 3개 파이프라인 전부에서 돌므로
**양 엔진이 lowering을 공유**한다(C 헬퍼도 core 변경도 불필요,
split_fn_body_line과 동일 패턴). 수신자는 env 타입 조회로 List<Int>일 때만
발화.

**root-fix 1건 동반**: edge probe(param receiver)에서 full이 SIGSEGV —
격리해보니 sort와 무관한 기존 core 버그. `xs[i] = v` 원소 쓰기가 warr==4
(pointer-aliased slot: 파라미터/alias 리스트)를 분기하지 않고 slot 자체를
`[0 x i64]` 버퍼로 gep해 저장된 버퍼 포인터를 clobber(IR 실측:
`getelementptr [0 x i64], [0 x i64]* %v0`). 읽기 경로는 이미 warr==4를
처리(load 포인터 + lenidx 4095 bounds). 쓰기 경로에 동일 분기 추가
(fixpoint_full.vais 17294 근처) — bounds trap 포함. e332의 List<Struct>
param 쓰기는 별도 경로라 영향받지 않았던 것.

e335_list_int_sort.vais(local/param/중복/정렬됨/빈 리스트, param 쓰기로 core
수정도 보호), parity/value 354 예정, LANGUAGE List<Int> 행/PRELUDE/CHANGELOG.
"다음 후보"의 sort 항목은 후반부(List<Struct>.sort_by)로 갱신. .ll canonical
재생성.

## 2026-07-10 (VaisDB 도그푸딩 확장 2 스프린트 — 5/5 완료)

한 세션에 5작업 완료. (1) e332 top-k 랭킹: List<Struct> 수동 selection sort
(원소 스왑=검증 표면, Int/Str 필드 probe 선행) + Result<Str,Str> blank-query
에러. (2) e333 스냅샷 버전: version=N 헤더, v1→v2 key 마이그레이션(key_at/
value_at 순회), 미지버전 Str 에러 — 유일한 디버깅은 str_slice 시그니처 오해
((start,len), invalid range trap은 문서화된 동작)로 컴파일러 버그 아님,
str_starts_with로 교체. (3) e334 인덱스 영속화: docid.term 평탄 키, 증분
ingest가 fresh build와 점수 동일. (4) tools/vaisdb_cli.vais: ingest/query/report
서브커맨드(query/report는 score를 exit code로), scripts/vaisdb-cli.sh 래퍼,
workflow 게이트 10케이스(양 엔진+래퍼+전 에러경로). (5) 환류: built-in
List sort/sort_by 부재를 "다음 후보"에 갭 1호로 등록, PRELUDE/CHANGELOG 반영.

**핵심 신호: 이번 스프린트 컴파일러 갭 0건.** e334/e335급 워크플로가 전부
검증된 표면 위에서 첫 시도에 동작 — 문서/텍스트/인덱스 도메인의 표면이
실전 수준으로 성숙했다는 실측 증거. 남은 병목은 ergonomics(정렬)와 표현력
확장(generic Result 등)이지 정확성이 아니다.

parity/value 351→353(+e332/333/334), workflow 게이트에 예제 3 + CLI 10케이스.
전 게이트 래더로 마감 검증.

## 2026-07-10 (미해석 식별자 무음실패 → 명시적 마커)

울트라코드 판정이 권고했던 위생 강화. core의 gen_factor 식별자 tail이
find_slot=-1일 때 조용히 `%v-1`을 emit해 clang이 암호 같은 에러를 내던 것을,
`@VAIS_UNRESOLVED_IDENT_<식별자>` 마커 emit으로 교체 — 이제 half-lowered
형태가 새면 clang 실패가 **어떤 식별자가 해석 실패했는지 즉시 지목**한다
(실측: `use of undefined value '@VAIS_UNRESOLVED_IDENT_zzz_unknown'`).
core에 에러 채널이 없어(재귀 codegen이 Op 반환) 진단 plumbing 대신 attributable
컴파일타임 실패를 택했다 — 최소 변경으로 동일한 방어 효과. green 경로는 이
가드에 도달하지 않으므로(corpus %v-1=0) 무영향. slot<0 조기반환이 isarr 분기
전에 있어 tail 전체를 덮는다.

## 2026-07-10 (raw-core str_str `?`/match 갭 해결 — task_6767f45a)

driver 수정(1a7e5592) 후 남았던 raw-core 갭 해결. codegen check harness는 driver
텍스트 lowering을 우회해 stage-빌드된 self-host core로 직접 컴파일하는데, 그
경로에서 Result<Str,Str>가 두 곳 미완이었다 — 둘 다 작업 2(fee3f697)에서 core에
match emit을 추가할 때 빠뜨린 배선:

1. **`?` 바인딩 프레디킷**: rhs_result_question_result_is_str가
   call_ret_result_str_int만 확인 → str_str `?`의 바인딩 local(title)이 Str
   slot(alloca i8*, is_arr 3)을 못 받아 Int 취급 → str_lower(title)에서 i64/ptr
   불일치. 수정 = call_ret_result_str_str 추가. payload 저장(inttoptr+store
   i8*)과 Err early-return(packed 재반환)은 slot 종류 기반이라 자동 정상화.
2. **match-결과 slot 등록**: rhs_result_match_str_str_result_is_str 프레디킷을
   작업 2에서 만들어놓고 add_local_slots/collect_top_slots에 연결 안 함 →
   `let recovered = match flow { Ok(t)=>t, Err(m)=>m }`의 recovered가 스칼라
   slot으로 등록, match emit은 i8*를 store, 후속 문자열 비교는 정수 icmp
   (ptr vs i64 불일치). 수정 = 두 collector에 str_int와 평행한 분기 연결.

**고속 재현 루프 확립**(다음에 유용): 게이트 전체 대신 단일 case를 ~1분에 반복 —
embed_self_source --raw fixpoint_full.vais prog.vais c.vais →
VAISC_SELF_HOST_TRUST_ROOTS=<root> scripts/vaisc build (trust 레이아웃
<root>/compiler/self/c.vais 필수) → ./c > out.ll → clang 검증+실행.

case_080m23(str_str `?`+match, 세미콜론 형태) 복원 완료, 고속 루프 실측 42.
driver 경로 회귀 0(e329/e330/e331/e301/e294=42). .ll canonical 재생성. 전 게이트
래더로 최종 검증.

## 2026-07-10 (세미콜론 단일라인 Result 버그 근본수정 — 울트라코드)

스프린트 중 발견한 잠복 버그(task_595301c0: 세미콜론 단일라인 fn body에서
Result<Str,*> local match/`?`가 `%v-1` invalid IR)를 멀티에이전트 워크플로로
해결. 단독 조사 3회가 전부 틀린 층(core의 slot 할당/match dispatch/find_stmt_end)
을 팠던 이유가 밝혀짐 — **버그는 core가 아니라 native driver(vaisc_native.c)의
pre-core 텍스트 lowering**에 있었다.

**근본 원인**(3-렌즈 병렬 진단 만장일치 + 판정 확정): driver의 타입/생성자
rewrite는 substring 기반이라 라인 어디서든 발화하는데, 문장 rewrite
(parse_inline_match_let :1467, parse_try_let :1564)는 물리 라인이 "let "으로
시작해야만 발화. 세미콜론 한 줄 함수는 라인이 "fn f(...) {"로 시작하므로 match/`?`
가 desugar되지 않은 반쯤-lower된 하이브리드가 core로 넘어가고, core는 `match`를
식별자로 취급(find_slot=-1)해 %v-1을 emit. 개행 형태는 driver가 완전 desugar하므로
정상 — 그래서 core 쪽 수정이 전부 무효했다.

**수정**: split_fn_body_line pre-pass(+192줄, vaisc_native.c만) — 한 줄 fn body의
depth-1 세미콜론을 개행 분리해 기존 정상 경로로 정규화(문자열/주석/중첩 brace
인식). 3개 파이프라인 head에 삽입. fixpoint_full.vais 무변경, vaisc_core.ll
byte-identical(기존 코퍼스에 한줄 다중문장 fn 0개). `?` 전파 오작동도 같은
수정으로 해결. 검증: 세미콜론 probe 4종(5/10/5/12)+edge(문자열 내 ';{}', 주석),
개행 회귀 11종, e331 신설(parity native-supported 등록, full/direct=42), 전 게이트
green(value 349→350 예정, self-host fixpoint bit-identical, release gates OK).

**남은 갭(task_6767f45a)**: fixpoint_full_codegen_check의 run_exit_case harness는
driver를 우회해 raw core로 컴파일하는데, 그 경로의 str_str `?` 바인딩은 여전히
i64/ptr 불일치(str_int는 통과). case_080m23은 계속 제외(NOTE 갱신), driver 경로는
e331이 보호.

**교훈**: 3회 단독조사가 같은 계층(core)만 판 것이 실패 원인 — 병렬 다중렌즈가
계층 가정 자체를 깼다. "이 버그는 어느 컴포넌트인가"부터 의심할 것.

## 2026-07-06 (작업 5 + 스프린트 완료 — Result 진단 문서화)

작업 5(문서/게이트 정리)로 VaisDB Result 진단 확장 스프린트 5/5 완료.
docs/reference/LANGUAGE.md 검증표에 Result<Str,Str> 행 추가 + "Rejected
Option/Result shapes" 섹션 신설(non-Int/Str error payload, 미선언 struct payload,
nested Option/Result 조합을 예시와 함께 명시하고 bad.vais count 게이트 +
front-contract reject 케이스 포인터 기록). std/PRELUDE.md에 e329(Result<Str,Str>
첫 non-Int error)/e330(VaisDB ingest 도그푸딩) 기술. CHANGELOG.md Unreleased에
Result<Str,Str> 승격 + Option/Result 진단 강화 2항목. 문서 정확성 실측 검증:
기술한 reject 4종 전부 거부, accept(e329/e330)=42. diff clean.

**스프린트 5작업 요약**: (1)Result 오용 P4 help 게이트 (2)Result<Str,Str>
direct+full self-host 승격 (3)nested Result/Option reject 게이트 (4)VaisDB ingest
Str 에러 메시지 도그푸딩 (5)문서 정리. 커밋 0b585a17/cdcbc00d/fee3f697/430e9633/
1095f9df + 문서. 부수 발견 task_8ac041ef(세미콜론 단일라인 Result<Str,*> local
slot 기존버그). memory 정정: Result<Str,*>=struct out-param(packed 아님).

**남음**: 작업2가 codegen(.ll) 변경했으므로 스프린트 마감 통합 검증(전 게이트 +
release gates) 후 브랜치 push/머지.

## 2026-07-06 (작업 4 — VaisDB ingest Str 에러 메시지 도그푸딩)

작업 4(VaisDB 인덱서 진단 경로 도그푸딩). 작업2의 Result<Str,Str> non-Int error
payload를 실제 제품 워크플로에서 쓰는 예제 e330 신설. e298(Result<Int,Int>
정수 코드)을 발전시켜 파일 ingest + snapshot round trip + query scoring의 모든
실패 경로를 사람이 읽는 Str 메시지("document not found"/"query not found"/
"document body is blank" 등)로 표현. helper `-> Result<Str,Str>`, `?` 전파로
ingest 실패 조기반환, inline match로 에러 메시지 문자열 회수. 성공 경로는
"top document is VaisDB Guide"(28자) 반환. 계산: 28 + "document not found"(18) -
4 = 42.

세미콜론 잠복 버그(task_8ac041ef) 회피 위해 개행 형태로 작성(예제는 원래 개행이라
자연스러움). e330 parity native-supported 등록(348→349), test-vaisdb-workflow.sh
expect_pair 편입(full+direct 42), README에 e329(작업2 누락)+e330 문서화. codegen
미변경. 실측 e330 full/direct=42, parity 349, value corpus 349/0, workflow OK,
diff clean.

**다음: 작업 5** (문서/게이트 정리 — LANGUAGE.md/PRELUDE.md에 Result 진단·타입
표면 반영). 그 다음 스프린트 마감 시 작업2 codegen 변경 때문에 release gates 통합
실행.

## 2026-07-06 (작업 3 — nested Result/Option 진단 게이트)

작업 3(nested 진단 명확화). 조사로 확인: nested 조합 4종(Result<Result<..>,Int>,
Result<Option<..>,Int>, Option<Result<..>>, Option<Option<..>>)은 이미 checker+
full engine 양쪽에서 P4 help로 거부된다(unsupported_result_generic_at/
unsupported_option_generic_at가 검증 형식 외 전부 reject). 진단 로직은 완결.
갭은 작업1과 동일 — 게이트로 안 고정돼 회귀 시 못 잡음.

구현: bad.vais에 nested_result + nested_option reject 2줄 추가(count 30→32,
smoke/contract/install 3 게이트 동기화 — install_check는 작업1/2에서 반복해서
놓친 파일이라 이번엔 처음부터 포함). front_check에 result_nested_not_verified
reject 케이스 신설(write_result_nested + dispatch + expect_reject); nested Option은
기존 option_nested 케이스가 이미 커버. Result P4 help(vais_check_core:778 +
vaisc_native.c:18283)에 "nested Option/Result payloads are not verified yet"
문구 추가 — Option 메시지와 일관성 맞춤, front_check가 이 문구로 nested Result를
검증. codegen 미변경(진단/fixture only). front/direct/checker/native smoke green,
회귀 0.

**다음: 작업 4** (VaisDB 인덱서에 진단 경로 도그푸딩) 또는 5(문서). 작업 2의
장시간 게이트(release)는 작업 3이 codegen 미변경이라 스프린트 마감 시 통합 실행.

## 2026-07-06 (작업 2 완료 — Result<Str,Str> full self-host)

Opus 직접 구현으로 full self-host 승격 완료(커밋 fee3f697). fixpoint_full.vais에
result_str_str_ty() 타입 태그, param/return 파싱, call_ret_result_str_str,
프레디킷 3개(result_match_expr_is_result_str_str + _is_str_at + _is_int_at),
emit 함수 2개(match str_let/int_let, 양 arm 모두 Str 포인터), match dispatch 추가.
native.c front 진단 열기 + 게이트 문자열 3곳 정렬. vaisc_core.ll canonical
재생성(임시경로 0). e329 parity native-supported 등록. 실측: e329 full=42,
match probe=5, ?전파 Ok=7/Err=5.

**핵심 정정**: 이전 세션 memory/ROADMAP에 "Result<Str,Int>는 packed scalar
i64"로 기록했으나 **실측 결과 틀렸다** — Result<Str,Int>도 Result<Str,Str>도
struct out-param(hidden, 3-slot: tag/value-ptr/error-ptr)로 lowering된다.
e301도 `define void @read_text_checked(i8*, i64*)`. 따라서 str_str는 packed
확장이 아니라 struct 경로 재사용이었고, Ok/Err 둘 다 Str 포인터라 str_int보다
단순(Err arm Int 분기 불필요). 처음 packed 전제로 헤맸으나 e301 IR 실측으로 정정.

**부수 발견(task_8ac041ef)**: full codegen check case를 세미콜론 단일라인 형태
(run_exit_case 요구)로 쓰다 기존 잠복 버그 노출 — Result<Str,*> local을
세미콜론 다중문장 함수에 바인딩하면 slot 미할당으로 `%v-1` invalid IR. 개행
형태는 정상. str_int도 동일(tD.vais로 확증). 작업2와 무관한 기존 버그. codegen
check case 제외(주석 기록)하고 task로 분리. e329가 정상 multi-line으로 전
게이트 보호하므로 커버리지 손실 없음.

release gates에서 `installed vais-check bad` 실패 발견 → vaisc_install_check.vais
count 29→30(작업1에서 놓친 파일) 수정. 전 게이트 green(self-host fixpoint
stage1==stage2 bit-identical 4.45MB 포함).

**다음: 작업 3** (nested Result/Option 진단 명확화, 저위험 codegen무). 작업1
reject 인프라 재사용. 그 다음 4(도그푸딩)/5(문서).

## 2026-07-06 (작업 2 — Result<Str,Str> direct 슬라이스)

작업 1을 main에 머지(fast-forward, origin push, 브랜치 정리)한 뒤 작업 2
(Result<Str,Str> non-Int error payload 승격)에 착수. 위임을 3회 시도(이전 세션 1
+ 이번 2)했으나 모두 중간에 멈췄다 — 원인은 native.c의 얽힌 다수 검증 지점
연쇄였다. Opus가 직접 파고들어 돌파.

핵심 발견(위임이 놓친 것): direct 헤더 화이트리스트는 손댈 필요가 없었다.
소스-투-소스 desugar가 `struct VaisResultStrStr { tag, value: Str, error: Str }`를
주입하고, direct_return_type_allowed가 "선언된 struct는 무조건 허용"하므로
desugar된 프로그램이 헤더 검증을 자동 통과한다. 필요한 건 desugar 함수
`lower_result_str_str_text` 하나뿐이었다. 이전 에이전트들은 헤더 화이트리스트를
고치려다 길을 잃었다.

구현: lower_result_str_str_text + 6개 타입명 매칭 헬퍼 복제(str_int 머신 미러,
error 필드만 Str). Ok 생성자 `error: 0`→`error: ""`가 유일한 로직 차이. desugar
파이프라인 3곳에 str_int 다음 str_str 단계 삽입. checker result_str_str_type_at
+ accept 목록 + P4 help. direct feature check shape 233(VaisResultStrStr =
{ i64, ptr, ptr }, error=ptr). 예제 e329. 실측: probe(match)=5, ?전파 Ok=7/Err=5,
e329=42(문자열 payload 회수 포함). 회귀 0(front/direct/checker/value 347/0).
커밋 cdcbc00d. `.ll` 미변경(native.c는 별도 빌드)이라 위생 이슈 없음.

e329는 vaisc-parity.tsv에 등록 안 함 — default 엔진(full)이 아직 거부하므로.
parity/value 게이트는 매니페스트 기반이라 미등록 예제를 건드리지 않는다. direct
feature check(shape 233)로만 보호.

**다음 세션: 작업 2의 full self-host 부분.** fixpoint_full.vais의 packed scalar
i64 스킴에 Str error 확장(ROADMAP 작업2 브리프 + memory 참조). Err payload도 heap
포인터로 대칭 처리가 정답. 완료 후 vaisc_core.ll canonical 재생성(임시경로 금지),
e329를 parity에 native-supported 등록, native.c front 진단도 정렬.

## 2026-07-06 (Result 진단 스프린트 착수 + 작업 1)

Codex 미커밋 작업(e121~e328)을 게이트 검증 후 커밋하고 origin/main에 push
완료(324d2cae..c0e443e3). push 전 `vaisc_core.ll` 헤더에 머신-로컬 임시경로
(`/var/folders/.../vaisc-native-hGr9L4/...`)가 새어든 것을 발견 — 과거 모든
커밋과 origin은 이 헤더가 0이었다. clang 헤더 4줄만 제거(IR 본문 345 define
불변)하고 self-host fixpoint(stage1==stage2 bit-identical)·release gates
재검증 후 e121~e328 커밋에 fixup으로 합쳤다.

이후 새 스프린트 "VaisDB Result 진단 확장"(5작업)을 ROADMAP에 작성하고 작업 1
착수. 조사로 확인한 핵심: Result 값-흐름은 e321에서 포화, 진단 프레디킷
`unsupported_result_generic_at`(vais_check_core.vais:230)도 이미 강력(검증된
4형식 외 전부 reject). **진짜 갭은 그 reject가 어떤 게이트로도 검증되지 않는
것**. 작업 1은 checker의 bad.vais fixture에 Result<Unknown,Int>(미선언 struct
payload) reject를 추가하고 count 29→30으로 고정. Result<Int,Str>(non-Int
error)는 원래 fixture에 있었으나 이제 count 게이트가 두 오용을 함께 보호한다.
front/direct/checker/diff-check green, 회귀 0. codegen 미변경이라 장시간 게이트
(fixpoint-full/self/release)는 스프린트 종료 시 통합 실행 예정.

**한계**: checker 게이트가 bad.vais 총 진단 count에 의존하는 기존 설계라, 특정
Result 진단 메시지 자체를 assert하진 않는다(기존 관례 준수). count 틀어지면
회귀는 잡힌다.

**다음 세션:** 작업 2 non-Int error payload 슬라이스(Result<Str,Str>) 승격 —
이건 codegen 변경 포함이라 full/direct/parity/value 전체 게이트 필요.

## 2026-07-05 (resume + commit)

Resumed a Codex session that had stopped mid-work with ~10 days of uncommitted
progress: the whole e121~e328 example corpus (205 new examples) plus the
compiler changes behind it (`fixpoint_full.vais`, `vaisc_core.ll`,
`vaisc_native.c`, tools/*check*, parity manifest) — 221 added + 35 modified,
all in the working tree, last commit was 2026-06-26.

Ran the full validation ladder before committing: front, direct,
fixpoint-full, value corpus (pass=347/0/0), parity (native=347),
full self-host fixpoint (stage1==stage2 IR bit-identical, 4.4MB),
`git diff --check`, and `test-release-gates.sh` — all green.

Committed the lot on branch `work/vaisdb-result-package-surface-2026-07-05`
(0479d5f8) rather than directly on `main` (main tracks origin/main, shared
public repo). Not pushed — waiting on user decision to merge/push.

**다음 세션:** push/merge 결정 대기. ROADMAP 현재 스프린트는 5/5 완료 상태이므로,
"다음 후보 작업"(richer package layout / Result 구조화 diagnostics 확장) 중 선택 필요.

## 2026-07-05

- Added e328 package static assets: package manifests can set optional
  `assets = "assets"` so `scripts/vaisc package` copies regular
  files/directories to `<dist-dir>/assets` and includes the same payload under
  `<binary-or-name>-<version>/assets/` when `--archive` is used.
  `examples/e328_cli_package_assets` verifies the argv-capable packaged binary,
  copied guide file, archive extraction, and manifest payload in native,
  direct, manifest-contract, and VaisDB workflow checks. Unsafe asset paths and
  missing asset directories are now manifest diagnostics.

## 2026-07-04

- Added e327 package release archives: `scripts/vaisc package
  examples/e326_cli_binary_target -o <dist-dir> --archive` now creates
  `<dist-dir>/veriqel-demo-0.1.0.tar.gz` with an extractable
  `veriqel-demo-0.1.0/bin/veriqel-demo` payload and copied `vais.toml`.
  Native, direct, and VaisDB workflow gates extract the archive and run the
  packaged command with argv; native diagnostics reject unsafe manifest
  `version` values when `--archive` would use them in filenames.
- Added `examples/e326_cli_binary_target`: package manifests can now set
  optional `binary = "veriqel-demo"` metadata so `scripts/vaisc package`
  writes `<dist-dir>/bin/veriqel-demo` instead of reusing the package name,
  while package-directory `run/build/emit-ir` still resolve `source/main.vais`.
  Native/direct/front/workflow/parity gates now cover the binary target path
  and unsafe `binary` diagnostics.
- Hardened e324 package output into an argv-capable package gate: native,
  direct, and VaisDB workflow checks now run packaged `e323_cli_package`
  binaries with `vaisdb cache` arguments, and native smoke verifies
  `vaisc package` rejects unsafe manifest names such as `bad/name` before they
  can become output paths.
- Added installable package output: `scripts/vaisc package
  examples/e323_cli_package -o <dist-dir>` now builds
  `<dist-dir>/bin/e323_cli_package`, copies the package manifest to
  `<dist-dir>/vais.toml`, and supports both default/full and `--engine direct`
  compilation. Native smoke, direct feature, and VaisDB workflow gates now run
  the packaged binary and verify the manifest is present.
- Added `examples/e323_cli_package`: `scripts/vaisc emit-ir`, `build`, and
  `run` can now take a package directory directly. Manifest-backed directories
  resolve to `source/main.vais`, so `scripts/vaisc run examples/e323_cli_package`
  works in direct and default/full paths, and argv forwarding continues to
  reach the compiled program. Native smoke, direct/front checks, workflow, and
  parity now protect the package-directory CLI surface.
- Added `examples/e322_vaisdb_module_boundary/main.vais` with transitive
  imports through `vaisdb.artifact` and `vaisdb.scoring`: reusable modules can
  now share a `DocArtifact` struct, `Result<DocArtifact,Int>` construction and
  validation helpers, `List<DocArtifact>` output mutation, and
  `Map<Str,Int>` term scoring helpers across files. The native direct path now
  resolves the module graph before direct lowering, so local/package/dependency
  imports run in direct as well as default/full. The direct/front/workflow and
  parity gates now include the imported VaisDB module-boundary workflow.
- Added `examples/e321_result_struct_payload_bool_match_condition.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now return `Bool`
  conditions directly from `Ok` payload helper terms such as
  `trimmed_len(artifact.title) + weight_terms(artifact.terms) +
  artifact.score >= 35`, while `Err` arms compare integer error codes. Native
  lowering rewrites `return match` to wrapper-tag `if` returns, and full
  self-host codegen now emits `resStructBool` branches with `icmp`/`zext`
  Bool results. The focused gates now include
  `case_080m22_result_struct_payload_bool_match_condition`.
- Added `examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now pass `Ok` payload
  `Int` fields such as `artifact.terms` and `artifact.score` to reusable
  `Int` helpers, then compose those helper-call terms with string-field helper
  terms in the same score arm. Full self-host codegen now emits
  `helper(artifact.int_field)` as `call i64 @helper(i64 field_value)` while
  preserving the e319 `Str` field helper path. The focused gates now include
  `case_080m21_result_struct_payload_int_field_helper_call_arithmetic`.
- Added `examples/e319_result_struct_payload_field_helper_call_arithmetic.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now pass `Ok` payload
  `Str` fields such as `artifact.title` and `artifact.body` to reusable
  `Int` helpers, then compose those helper-call terms with normal payload
  fields. Full self-host codegen now recognizes verified
  `helper(artifact.str_field)` terms inside structured Result arithmetic arms
  and lowers them as `call i64 @helper(i8* field_ptr)`. The focused gates now
  include
  `case_080m20_result_struct_payload_field_helper_call_arithmetic`.
- Added `examples/e318_result_struct_payload_helper_call_arithmetic.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now compose a reusable
  `text_score(artifact: DocArtifact) -> Int` helper-call term with normal
  payload fields such as `artifact.terms + artifact.score`, while `Err` arms
  recover integer codes. Full self-host codegen now recognizes verified helper
  calls as `Int` terms inside structured Result arithmetic arms without
  widening Op-returning compiler function signatures beyond the 10-parameter
  self-host limit. The focused gates now include
  `case_080m19_result_struct_payload_helper_call_arithmetic`.
- Added `examples/e317_result_struct_payload_helper_call_score.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now pass the `Ok`
  payload directly to a reusable `score_artifact(artifact: DocArtifact) -> Int`
  helper, while `Err` arms recover integer codes. Full self-host codegen now
  recognizes single-argument `Int` helper-call terms whose argument is the
  Result `Ok` binder and lowers them as `call i64 @helper(i64* payload_ptr)`.
  The focused gates now include
  `case_080m18_result_struct_payload_helper_call_score`.
- Added `examples/e316_result_struct_str_transform_len_match_flow.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now compute numeric
  document scores from transformed string payload fields such as
  `str_trim(artifact.title).len()`,
  `str_concat(prefix, str_lower(str_trim(artifact.id))).len()`, and
  `str_replace(artifact.body, "cache", "store").len()`, while `Err` arms
  recover integer codes. Native direct rewriting/type inference now recognizes
  trailing `.len()` on string-returning helper calls as `Int`, and full
  self-host codegen now lowers transformed string-length terms inside
  `Result<Struct,Int>` match arms. The focused gates now include
  `case_080m17_result_struct_str_transform_len_match_flow`.
- Added `examples/e315_result_struct_str_transform_match_flow.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now normalize document
  ID/title `Str` payload fields with `str_replace`, `str_trim`, `str_upper`,
  `str_lower`, and a local-prefix `str_concat(...)` expression, while `Err`
  arms stringify integer codes. The focused gates now include
  `case_080m16_result_struct_str_transform_match_flow`.
- Added `examples/e314_result_struct_str_concat_match_flow.vais`:
  declared-struct `Result<DocArtifact,Int>` matches can now compose `Str`
  payload fields with nested `str_concat(...)` expressions such as
  `artifact.id + ":" + artifact.title`, while `Err` arms stringify integer
  codes. Full self-host lowering now recognizes recursive struct Result string
  arms and emits `__vais_str_concat` calls from payload field pointers. The
  focused gates now include
  `case_080m15_result_struct_str_concat_match_flow`.
- Added `examples/e313_result_struct_str_match_flow.vais`: declared-struct
  `Result<DocArtifact,Int>` matches can now recover `Str` payload fields such
  as `artifact.title`/`artifact.id` directly into string locals while `Err`
  arms convert integer codes with `Str(code)`. Native Result-struct match
  lowering now binds Ok payloads via the verified `let artifact = flow.value`
  copy path, and full self-host lowering now allocates/emits string slots for
  struct Result match fields. The focused gates now include
  `case_080m14_result_struct_str_match_flow`.
- Added `examples/e312_result_struct_local_wrapper_flow.vais`: explicit
  `VaisResult<Struct>Int` wrapper code can now copy nested struct payloads
  through local variables (`let artifact = flow.value`) and return those locals
  in wrapper literals without dropping fields. Full self-host lowering now
  allocates field-chain struct lets as flattened struct locals and copies
  nested struct locals field-by-field into local/out wrapper fields. The
  focused gates now include
  `case_080m13_result_struct_local_wrapper_flow`.
- Added `examples/e311_result_call_argument_flow.vais`: `Result<Str,Int>` and
  declared-struct `Result<DocArtifact,Int>` helpers can now be passed directly
  as call arguments, e.g. `result_text(read_checked(path))` and
  `artifact_score(build_doc_artifact(...))`, instead of requiring explicit
  temporary locals. Native source lowering now hoists Result-returning call
  arguments and clears per-function Result local-name caches; the full
  self-host fixture protects the hidden-out struct-return call-argument path
  with `case_080m12_result_call_argument_flow`.
- Hardened native `vaisc` temp cleanup after the disk-capacity investigation:
  the native driver now allocates intermediates inside one per-run temp root,
  deletes it on normal exit, keeps debug artifacts only with `--keep-tmp`, and
  the native smoke gate now fails if `vaisc-native-*` directories leak.
- Added `examples/e310_vaisdb_artifact_query_report.vais`: Vais can now express
  a reusable persisted artifact-store query/report workflow. The example loads
  `List<DocArtifact>` records from a store path, ranks them with
  `Map<Str,Int>` term scoring, returns a report as `Result<Str,Int>`, writes
  and rereads that report, and keeps missing-store/empty-query failures as
  integer error codes. Native/full lowering now covers `Result<Int,Int>?` and
  `Result<DocArtifact,Int>?` early-return propagation inside
  `Result<Str,Int>` helpers. The focused gates now include
  `case_080m11_vaisdb_artifact_query_report`.
- Added `examples/e309_vaisdb_artifact_store_snapshot.vais`: Vais can now
  express a persistable artifact store workflow over `List<DocArtifact>`.
  The example serializes document records to a tab-delimited text snapshot,
  writes/reads it with host file helpers, rebuilds `DocArtifact` values through
  `Result<DocArtifact,Int>` parsing helpers, queries the best loaded record,
  and reports malformed or missing store paths as `Result<Int,Int>` errors.
  The focused gates now include `case_080m10_vaisdb_artifact_store_snapshot`.
- Added `examples/e308_vaisdb_artifact_record_workflow.vais`: Vais can now
  express a small VaisDB artifact/document record workflow where
  `Result<DocArtifact,Int>` helpers build document payloads, local-binding `?`
  extracts them, a `List<DocArtifact>` output parameter is mutated in the
  callee, and `Map<Str,Str>` metadata snapshots round trip. Full self-host
  codegen now parses `List<Struct>` parameter element types by comma-delimited
  signature segments so preceding `Map<Str,Str>` generic commas no longer make
  the callee write scalar-list length slots. The focused gates now include
  `case_080m9_vaisdb_artifact_record_workflow`.
- Added `examples/e307_result_struct_try_payload.vais`: declared
  `Result<Struct,Int>` payloads can now be extracted with local-binding `?` in
  a Result-returning helper. Native direct source lowering rewrites the
  struct-payload try into an early `Err` return plus a payload struct copy,
  while full self-host codegen allocates the payload local as a struct and
  copies fields from the encoded Result payload pointer. The focused gates now
  include `case_080m8_result_struct_try_payload`.
- Added `examples/e306_result_struct_str_fields.vais`: declared
  `Result<Struct,Int>` payload structs can now include `Str` fields in the
  gate-backed `DocSummary` workflow. Native direct source lowering preserves
  payload field types and emits empty-string defaults for `Err` payload
  placeholders, while full self-host codegen recognizes Ok-arm
  `doc.str_field.len()` terms mixed with Int fields.
- Added `examples/e305_result_multiline_struct_payload.vais`: native direct
  source lowering now collects multiline declared Int-field struct payloads and
  emits the `VaisResult<Struct>Int` wrapper after the closing struct line.
  `case_080m6_result_multiline_struct_payload` protects the full self-host path
  with a real multiline `Entry` declaration and 4-field Result match recovery.
- Added `examples/e304_result_record_int_struct_payload.vais`: the structured
  Result payload path now works beyond the previous `Metric`-only slice. Native
  source lowering derives `VaisResult<Struct>Int` wrappers and field recovery
  from declared Int-field structs, the public checker/front contract accepts
  `Result<DeclaredStruct,Int>`, and full self-host codegen now emits n-field
  struct-payload Result match sums. `case_080m5_result_record_int_struct_payload`
  protects the raw full path.

## 2026-07-03

- Added `examples/e303_result_metric_int_struct_payload.vais`: a concrete
  `Result<Metric,Int>` value can now carry a small struct payload through
  helper returns, helper parameters, and forwarding helpers, then recover
  payload fields through inline `match`. Full self-host codegen keeps the
  struct payload heap-backed across helper boundaries, and
  `case_080m4_result_metric_int_struct_payload` protects the raw full path.
- Added `examples/e302_result_str_int_param_flow.vais`: `Result<Str,Int>`
  values can now be passed through helper parameters, forwarded to another
  helper, and matched inside helpers to recover `Str` payloads or `Int` error
  values. Native source lowering now tracks `Result<Str,Int>` parameters, full
  self-host codegen parses them as packed scalar result slots, and
  `case_080m3_result_str_int_param_flow` protects the raw full path.
- Added `examples/e301_result_str_int_file_read.vais`: Vais now has a
  gate-backed `Result<Str,Int>` file-read slice where guarded helpers return
  `Ok(text)` or `Err(code)`, another helper binds the string payload with `?`,
  and inline matches recover normalized string payloads or integer error
  codes. The public `vais-check` contract now accepts the concrete
  `Result<Str,Int>` shape.
- Added `tools/vaisdb_benchmark_report.vais` and
  `scripts/vaisdb-benchmark-report.sh`: the reusable Vais-authored developer
  command now runs the e295 indexer, writes a raw direct/default benchmark
  report, parses metric lines, computes a timing delta, and writes a summary.
  Workflow/front/direct/full/parity/value gates cover the tool path.
- Added `examples/e300_vaisdb_benchmark_cli_report.vais`: Vais code now
  discovers the repo root with `fs_cwd`/`path_dirname`/`path_basename`, invokes
  the e295 indexer through `proc_capture`, records direct/default elapsed
  milliseconds with `time_millis`, writes a combined report, and validates it
  through front/direct/full/VaisDB/parity gates.
- Added `time_millis() -> Int` and
  `examples/e299_vaisdb_benchmark_report.vais`: host/full runtime and native
  direct lowering now expose a millisecond timing helper, the example writes
  and validates a VaisDB benchmark report with document term counts and
  weighted scoring, and front/direct/full/host/VaisDB/parity gates cover the
  new slice.
- Added `examples/e298_vaisdb_file_ingest_result_flow.vais` as the
  file-backed Result follow-up: direct/native now recognizes `fs_exists`, the
  example guards `fs_read_text` with existence checks, returns explicit
  `Result<Int,Int>` error codes for missing or malformed paths, verifies
  generated-file, argv-file, and missing-document modes in
  `scripts/test-vaisdb-workflow.sh`, and full codegen protects the standalone
  shape with `case_080m_file_exists_result_flow`.
- Added `examples/e297_vaisdb_file_ingest_workflow.vais` and extended the
  direct/native host builtin lowering so Vais code can read document/query
  files, create temp fixtures, accept argv paths, split lines, snapshot
  metadata, index term counts, and score a query. The focused VaisDB workflow
  gate now checks both generated-file and argv-file modes, and the full
  codegen gate links `tools/fixpoint_host_runtime.c` so standalone generated IR
  can exercise the same file workflow.
- Closed the e295 follow-up compiler gap with
  `examples/e296_result_map_param_flow.vais`: public enum lowering now
  prioritizes concrete `Result<Int,Int>` when Result and Option markers appear
  together, the direct feature gate covers the new shape, full self-host
  `fixpoint_full.vais` lowers surface `Ok`/`Err`, local-binding `?`, and
  payload-only Result matches, `compiler/self/vaisc_core.ll` was regenerated,
  and front/direct/full/self/parity gates are green for the slice.
- Added `docs/design/VAISDB_DX_BASELINE.md`,
  `scripts/test-vaisdb-workflow.sh`, and
  `scripts/bench-vaisdb-indexer.sh` for the Task 5 developer-experience slice:
  contributors now have a focused direct/default e292-e297 document workflow
  gate, formatter direction, diagnostic commands, and a repeatable local
  compile+run timing protocol for the Vais-authored indexer prototype.
- Added `examples/e295_vaisdb_indexer_prototype.vais` as the first
  Vais-authored document indexer dogfooding prototype, combining document
  ingest, metadata snapshot round trip, `Map<Str,Int>` term counts, and
  weighted query scoring. The prototype keeps Map-mutating helpers returning
  `Int` status values, while e296 now separately verifies
  `Result<Int,Int>` helpers that take `Map<Str,Str>` parameters.
- Promoted concrete `Option<Int>`/`Result<Int,Int>` value lowering through the
  native direct path for helper return/parameter/local types, constructors,
  inline expression-match bindings, and local-binding `?`; locked the
  document-style `Result<Int,Int>` parse/error flow with
  `examples/e294_result_try_parse_error_flow.vais`. The full self-host codegen
  gate now also protects the surface Result+Map helper flow through
  `case_080g7_result_map_param_flow`.
- Promoted `map_str_str_snapshot(docs)` and
  `map_str_str_load_snapshot(text, out)` through the full self-host and native
  direct paths, regenerated `compiler/self/vaisc_core.ll`, and locked
  `Map<Str,Str>` line snapshot round trips for small VaisDB metadata with
  `examples/e293_map_str_str_snapshot_builtin.vais`.
- Promoted `str_split_lines_into(text, out)` through the full self-host and
  native direct paths, regenerated `compiler/self/vaisc_core.ll`, and locked
  LF/CRLF line tokenization into `List<Str>` out-params with interior blank
  lines, empty input, and trailing-line-break handling in
  `examples/e292_str_split_lines_into.vais`.
- Closed full self-host `str_concat(left, right)` and `str_byte(value)` lowering
  over internal runtime helpers, then updated the host IR shape gate to assert
  those self-contained helper calls instead of external host calls.
- Promoted `str_join(parts, sep)` through the full self-host and native direct
  paths, regenerated `compiler/self/vaisc_core.ll`, and locked `List<Str>`
  string reconstruction with separators, empty-list handling, and
  `str_split_into` round trips with `examples/e291_str_join.vais`.

## 2026-07-02

- Promoted `str_split_into(text, sep, out)` through the full self-host and
  native direct paths, regenerated `compiler/self/vaisc_core.ll`, and locked
  delimiter-based tokenization into `List<Str>` out-params while preserving
  empty fields and treating an empty separator as one whole-text field with
  `examples/e290_str_split_into.vais`.
- Promoted `str_replace(text, needle, replacement)` through the full self-host
  and native direct paths, regenerated `compiler/self/vaisc_core.ll`, and locked
  all-occurrence string rewriting over literals, normalized `Map<Str,Str>`
  reads, `List<Str>` reads, and `Map<Str,Str>.get_opt` match values with
  `examples/e289_str_replace.vais`; raised the self-host `List<Token>` retarget
  capacity to 262144 so the enlarged `fixpoint_full.vais` can consume itself
  through the full stage1/stage2 gate.
- Promoted `str_ends_with(text, suffix)` through the full self-host and native
  direct paths, regenerated `compiler/self/vaisc_core.ll`, and locked suffix
  checks over literals, normalized strings, `Map<Str,Str>` reads, `List<Str>`
  reads, and `Map<Str,Str>.get_opt` match values with
  `examples/e288_str_ends_with.vais`.
- Promoted `str_upper(text)` through the full self-host and native direct
  paths, regenerated `compiler/self/vaisc_core.ll`, and locked literals,
  trimmed document fields, `Map<Str,Str>` reads, `List<Str>` reads, and
  `Map<Str,Str>.get_opt` match payload transforms with
  `examples/e287_str_upper.vais`; native front keyword diagnostics now check
  token boundaries for `match`/`enum`, so identifiers such as `from_match` do
  not trigger unsupported-syntax errors.
- Promoted `Map<Str,Str>.get_opt` string payload match expressions in `while`
  and `else if` condition chains by lowering loop conditions with per-iteration
  temporaries and rewriting embedded `else if` matches without breaking the
  surrounding chain, with
  `examples/e286_map_str_str_get_opt_condition_chains.vais`.
- Lowered `Map<Str,Str>.get_opt` string payload matches through map presence
  checks and value loads instead of pointer-tagged string payload integers, so
  saved `Str` payload locals remain stable across later embedded match/string
  helper expressions; full self-host statement parsing now skips match-arm
  braces while finding `if`/`while` bodies for those embedded conditions, with
  `examples/e285_map_str_str_get_opt_str_payload_stability.vais`.
- Added `Map<Str,Str>.get_opt` match-arm transform length support so
  `str_lower(str_trim(v)).len()` and transformed fallback lengths lower in
  full/direct paths, with
  `examples/e284_map_str_str_get_opt_match_transform_len.vais`.
- Fixed full self-host `Str.len()` for locals reassigned from dynamic string
  values so it reads the current runtime pointer instead of an initial literal
  length, regenerated `compiler/self/vaisc_core.ll`, and locked the behavior
  with `examples/e283_str_len_reassigned_match_transform.vais`.
- Added `str_concat`, `str_trim`, and `str_lower` result detection for
  `Map<Str,Str>.get_opt` string payload match expressions in verified `Str`
  contexts, with
  `examples/e282_map_str_str_get_opt_match_str_transforms.vais`.
- Promoted `Map<Str,Str>` return-inferred locals into the `get_opt` string
  payload match-expression lowering path for returns, reassignments,
  helper-call arguments, and embedded Int returns, with
  `examples/e281_map_str_str_return_infer_get_opt_match_contexts.vais`.
- Promoted `Map<Str,Str>.get_opt` string payload match expressions beyond local
  bindings into returns, reassignments, helper-call arguments, and embedded Int
  returns, with `examples/e280_map_str_str_get_opt_match_contexts.vais`.
- Added independent scalar Char `if ... then ... else ...` value-expression
  coverage for locals, reassignments, helper-call arguments, and Char returns
  without a pipeline-specific trigger, with
  `examples/e279_scalar_char_value_if_expr.vais`.
- Added independent scalar Str `if ... then ... else ...` value-expression
  coverage for locals, reassignments, helper-call arguments, and Str returns
  without a pipeline-specific trigger, with
  `examples/e278_scalar_str_value_if_expr.vais`.
- Added independent scalar Bool `if ... then ... else ...` value-expression
  coverage for locals, reassignments, helper-call arguments, and returns without
  a pipeline-specific trigger, with
  `examples/e277_scalar_bool_value_if_expr.vais`.
- Promoted scalar `if ... then ... else ...` value expressions in locals,
  reassignments, helper-call arguments, and returns without requiring a
  pipeline-specific lowering trigger, including embedded helper-call argument
  temps for both `Int` and `Bool` branches, with
  `examples/e276_scalar_value_if_expr_embedded_call_args.vais`.
- Promoted Int `if ... then ... else ...` expressions built from `List<Str>`
  pipeline scalar conditions in locals, reassignments, helper-call arguments,
  and returns, and lowered full-source value if-expressions through source-prep
  temps/statements so scoring-style branches return the same value in full and
  direct engines, with
  `examples/e275_list_str_pipeline_scalar_int_if_expr.vais`.
- Promoted nested helper-call Bool `if ... then ... else ...` expressions
  inside `List<Str>` pipeline scalar reassignments, and fixed `let mut`
  helper-call argument lowering so the actual variable name remains tracked,
  with
  `examples/e274_list_str_pipeline_scalar_bool_if_expr_nested_call_reassign.vais`.
- Promoted Bool `if ... then ... else ...` expressions built from `List<Str>`
  pipeline scalar conditions across helper-call arguments and Bool returns with
  `examples/e273_list_str_pipeline_scalar_bool_if_expr_call_return.vais`.
- Promoted Bool `if ... then ... else ...` expressions built from `List<Str>`
  pipeline scalar conditions, including Bool local inference and direct
  reassignment support, with
  `examples/e272_list_str_pipeline_scalar_bool_if_expr.vais`.
- Promoted negated `List<Str>` pipeline scalar Bool expressions in locals,
  reassignments, `if` conditions, and `while` conditions with
  `examples/e271_list_str_pipeline_scalar_bool_negation.vais`.
- Promoted arithmetic-tail reassignments using
  `List<Str>.map(...).filter(...).len/index_of/count` and
  `List<Str>.filter(...).map(...).len/index_of/count` pipeline scalar
  expressions with
  `examples/e270_list_str_pipeline_scalar_reassign_arithmetic_tail.vais`.
- Promoted composite Bool local inference for
  `List<Str>.map(...).filter(...).contains(...)` and
  `List<Str>.filter(...).map(...).contains(...)` scalar conditions, preserving
  exact Bool pipeline scalar reassignments with
  `examples/e269_list_str_pipeline_scalar_bool_infer.vais`.
- Promoted mixed map-filter/filter-map
  `List<Str>.len/contains/index_of/count` pipeline scalar calls inside one
  expression with `examples/e268_list_str_pipeline_scalar_mixed_expr.vais`.
- Promoted multiple same-family
  `List<Str>.map(...).filter(...).len/contains/index_of/count` or
  `List<Str>.filter(...).map(...).len/contains/index_of/count` scalar calls
  inside one expression with
  `examples/e267_list_str_pipeline_scalar_multi_expr.vais`.
- Promoted `List<Str>.filter(...).map(...).len/contains/index_of/count`
  scalar chains for direct locals, helper returns, helper-call arguments,
  `List<Int>` mutation arguments, reassignments, and conditions with
  `examples/e266_list_str_filter_map_scalar_contexts.vais`.
- Promoted `List<Str>.map(...).filter(...).len/contains/index_of/count`
  scalar chains for direct locals, helper returns, helper-call arguments,
  `List<Int>` mutation arguments, reassignments, and conditions with
  `examples/e265_list_str_map_filter_scalar_contexts.vais`.
- Promoted `List<Str>.map(...).filter(...)` result lists for direct locals,
  helper returns, helper-call arguments including conditions, `extend(...)`
  sources, and reassignments with
  `examples/e264_list_str_map_filter_result_contexts.vais`.
- Promoted `List<Str>.filter(...).map(...)` result lists for direct locals,
  helper returns, helper-call arguments including conditions, `extend(...)`
  sources, and reassignments with
  `examples/e263_list_str_filter_map_result_contexts.vais`.
- Promoted `List<Int>.map(...).sum()/max()/min()` transformed scalar aggregate
  expressions inside broader `Int` expressions and broader `if`/`while`/
  `else if` condition expressions with
  `examples/e262_list_map_aggregate_embedded_expr_conditions.vais`.
- Promoted `List<Int>.filter(...).map(...).sum()/max()/min()` transformed
  scalar aggregate expressions inside broader `if`/`while`/`else if` condition
  expressions with
  `examples/e261_list_filter_map_aggregate_embedded_conditions.vais`.
- Promoted `List<Int>.filter(...).map(...).sum()/max()/min()` transformed
  scalar aggregate expressions inside broader `Int` expressions used by
  locals, helper-call arguments, direct `List<Int>` mutation arguments,
  reassignments, and returns with
  `examples/e260_list_filter_map_aggregate_embedded_expr.vais`.
- Promoted `List<Struct>.filter(...).map(...).sum()/max()/min()` aggregate
  expressions inside broader `if`/`while`/`else if` condition expressions with
  `examples/e259_list_struct_filter_map_aggregate_embedded_conditions.vais`.
- Promoted `List<Struct>.filter(...).map(...).sum()/max()/min()` aggregate
  expressions inside broader `Int` expressions used by locals, helper-call
  arguments, direct `List<Int>` mutation arguments, reassignments, and returns
  with `examples/e258_list_struct_filter_map_aggregate_embedded_expr.vais`.
- Promoted `List<Struct>.filter(...).map(...).sum()/max()/min()` aggregate
  helper-call arguments inside `if`/`while`/`else if` condition expressions with
  `examples/e257_list_struct_filter_map_aggregate_call_arg_conditions.vais`.
- Promoted `List<Struct>.filter(...).map(...).sum()/max()/min()` direct score
  aggregates as `Int` helper-call arguments with
  `examples/e256_list_struct_filter_map_aggregate_call_args.vais`.
- Promoted `List<Struct>.map(...).sum()/max()/min()` direct `Int` projection
  aggregates inside broader `if`/`while`/`else if` condition expressions with
  `examples/e255_list_struct_map_projection_aggregate_embedded_conditions.vais`.
- Promoted `List<Struct>.map(...).sum()/max()/min()` direct `Int` projection
  aggregates inside broader `Int` expressions, covering local assignment,
  helper-call arguments, direct `List<Int>.push`/`insert_at` mutation
  arguments, known `Int` reassignments, and return expressions with
  `examples/e254_list_struct_map_projection_aggregate_embedded_expr.vais`.

## 2026-07-01

- Promoted `List<Struct>.map(...).sum()/max()/min()` direct `Int` projection
  aggregates for reassignment to known `Int` variables and parameters,
  preserving simple arithmetic suffixes with
  `examples/e253_list_struct_map_projection_aggregate_reassign.vais`.
- Promoted `List<Struct>.map(...).sum()/max()/min()` direct `Int` projection
  aggregates as direct `List<Int>.push` and `insert_at` mutation arguments,
  preserving simple arithmetic suffixes and covering local/parameter list
  targets with
  `examples/e252_list_struct_map_projection_aggregate_mutation_args.vais`.
- Promoted simple arithmetic suffixes on `List<Struct>.map(...).sum()/max()/min()`
  direct `Int` projection aggregates when they are used as helper-call arguments
  in `return`, `let`, `if`, `while`, and `else if` contexts, covered by
  `examples/e251_list_struct_map_projection_aggregate_call_arg_arithmetic_tail.vais`.
- Promoted simple arithmetic suffixes on `List<Struct>.map(...).sum()/max()/min()`
  direct `Int` projection aggregates in return expressions and typed/inferred
  locals, covered by
  `examples/e250_list_struct_map_projection_aggregate_arithmetic_tail.vais`.
- Promoted `List<Struct>.map(...).sum()/max()/min()` direct `Int` projection
  aggregates as helper-call arguments in `return`, `let`, `if`, `while`, and
  `else if` contexts, so aggregate/ranking expressions can compose with
  domain helpers without user-written temporary totals, covered by
  `examples/e249_list_struct_map_projection_aggregate_call_args.vais`.
- Promoted `List<Struct>.map(...).sum()/max()/min()` direct `Int` projection
  aggregates in `if`, `while`, and `else if` condition expressions, preserving
  per-iteration recomputation for loops, covered by
  `examples/e248_list_struct_map_projection_aggregate_conditions.vais`.
- Promoted `List<Struct>.map(...).sum()/max()/min()` for direct `Int` field
  projections in helper returns and typed/inferred `Int` locals, so record
  scores can be aggregated or ranked without first materializing a scalar list,
  covered by `examples/e247_list_struct_map_projection_aggregates.vais`.
- Promoted `List<Struct>.map(...)` projected helper-call arguments in `if`,
  `while`, and `else if` condition expressions, so whole-record score/title
  projections can drive branch and loop decisions without user-written
  temporary lists, covered by
  `examples/e246_list_struct_map_projection_call_arg_conditions.vais`.
- Promoted `List<Struct>.map(...)` projected result lists for direct
  `List<Int>`/`List<Str>` helper returns, helper-call arguments,
  `extend(...)` sources, and existing-list reassignment, covered by
  `examples/e245_list_struct_map_projection_direct_contexts.vais`.
- Promoted `List<Struct>.filter(...).map(...)` projected result lists for
  existing `List<Int>`/`List<Str>` variable reassignment, lowering filtered
  score/title projections into temporary scalar lists before assignment and
  covered by `examples/e244_list_struct_filter_map_reassign.vais`.
- Promoted `List<Struct>.filter(...).map(...)` projected result lists as direct
  `List<Int>`/`List<Str>.extend(...)` arguments, so score/title buffers can be
  extended from filtered record projections without user-written temporary
  lists, covered by `examples/e243_list_struct_filter_map_extend_arg.vais`.
- Promoted `List<Struct>.filter(...).map(...)` projected helper-call arguments
  in `if`, `while`, and `else if` condition expressions, so filtered
  `List<Int>`/`List<Str>` score/title projections can drive branch and loop
  decisions without user-written temporaries, covered by
  `examples/e242_list_struct_filter_map_call_arg_conditions.vais`.
- Promoted `List<Struct>.filter(...).map(...)` projected result lists as direct
  helper-call arguments for `List<Int>` and `List<Str>` parameters, lowering
  filtered score/title projections into temporary scalar lists before the call
  and covered by `examples/e241_list_struct_filter_map_call_arg.vais`.
- Promoted direct-return `List<Struct>.filter(...).map(...)` projected result
  lists, so helpers can return filtered `List<Int>` and `List<Str>` field
  projections without a temporary local, covered by
  `examples/e240_list_struct_filter_map_return_chain.vais`.
- Promoted direct `List<Struct>.filter(...).map(...)` projected result lists,
  lowering filtered record field projections into reusable `List<Int>` and
  annotated `List<Str>` lists without a user-written intermediate record list
  and covered by
  `examples/e239_list_struct_filter_map_result_chain.vais`.
- Fixed full codegen termination analysis for blocks whose last statement is an
  all-return nested `if` chain after earlier local statements, covered by
  `examples/e238_list_struct_filter_first_last_call_arg_else_if_chain_return.vais`.
- Added a chained `else if` coverage slice for filtered first/last
  helper-call argument lowering, including a false first branch, true second
  branch, and final `else`, covered by
  `examples/e237_list_struct_filter_first_last_call_arg_else_if_chain.vais`.
- Extended filtered first/last helper-call argument lowering to `else if`
  condition expressions by rewriting them into a guarded nested `else` block,
  covered by
  `examples/e236_list_struct_filter_first_last_call_arg_else_if_condition.vais`.
- Extended filtered first/last helper-call argument lowering to `while`
  condition expressions, preserving per-iteration recomputation by lowering the
  condition to an explicit loop guard, covered by
  `examples/e235_list_struct_filter_first_last_call_arg_while_condition.vais`.
- Extended filtered first/last helper-call argument lowering to `if` condition
  expressions whose condition starts with the helper call, covered by
  `examples/e234_list_struct_filter_first_last_call_arg_if_condition.vais`.
- Preserved simple arithmetic expression tails on helper calls whose arguments
  include filtered first/last field or whole-record selections, so calls can be
  used inside larger `Int` expressions, covered by
  `examples/e233_list_struct_filter_first_last_call_arg_expr_tail.vais`.
- Removed the source-order limitation for filtered first/last helper-call
  argument lowering by pre-scanning function signatures before list-method
  lowering, covered by
  `examples/e232_list_struct_filter_first_last_late_helper_call_arg.vais`.
- Promoted filtered first/last whole-record selections as direct same-struct
  helper-call arguments, reusing helper parameter type metadata and guarded
  record-selection temporaries before the call, covered by
  `examples/e231_list_struct_filter_first_last_value_call_arg.vais`.
- Promoted filtered first/last field and string-length selections as direct
  `Int`/`Str` helper-call arguments, tracking helper parameter types in the
  source-prep environment and lowering each selected field into a guarded
  temporary before the call, covered by
  `examples/e230_list_struct_filter_first_last_field_call_arg.vais`.
- Promoted unannotated local inference for
  `List<Struct>.filter(...).first().field`/`.last().field` and string-field
  `.len()` selections, carrying declared struct field type metadata through
  source-prep so matched document fields can be stored as inferred `Int`/`Str`
  locals and covered by
  `examples/e229_list_struct_filter_first_last_field_infer.vais`.
- Promoted `List<Struct>.filter(...).first().field`/`.last().field` and
  string-field `.len()` selections as direct scalar `push` and `insert_at`
  arguments for `List<Int>`/`List<Str>`, reusing guarded field-selection
  temporaries and covered by
  `examples/e228_list_struct_filter_first_last_field_push_insert.vais`.
- Promoted `List<Struct>.filter(...).first()`/`.last()` whole-record selections
  as direct same-struct `push` and `insert_at` arguments, lowering the argument
  into a guarded temporary record before reusing existing struct-list mutation
  support and covered by
  `examples/e227_list_struct_filter_first_last_push_insert.vais`.
- Extended the whole-record `List<Struct>.filter(...).first()`/`.last()`
  selection lowering to multiline struct declarations by tracking struct field
  blocks in source-prep, covered by
  `examples/e226_list_struct_filter_first_last_multiline_value.vais`.
- Promoted `List<Struct>.filter(...).first()`/`.last()` whole-record
  selection for document-like records in same-struct returns and typed or
  inferred locals, lowering through a guarded single-pass record-selection loop
  without an intermediate record list and covered by
  `examples/e225_list_struct_filter_first_last_value.vais`.
- Promoted `List<Struct>.filter(...).first().str_field.len()`/`.last().str_field.len()`
  for direct document-like string field length selection in `Int` returns and
  typed locals, reusing the guarded single-pass field-selection loop and
  covered by `examples/e224_list_struct_filter_first_last_field_len.vais`.
- Promoted `List<Struct>.filter(...).first().field`/`.last().field` for
  document-like record selection in `Int`/`Str` returns and typed locals,
  lowering to a guarded single-pass field-selection loop without an
  intermediate record list and covered by
  `examples/e223_list_struct_filter_first_last_field.vais`.
- Promoted `List<Int>.filter(...).map(...).sum()` for transformed scalar
  aggregation in returns and reusable `Int` locals, reusing the single-pass
  accumulator lowering and covered by `examples/e222_list_filter_map_sum.vais`.
- Promoted `List<Int>.filter(...).map(...).max()`/`.min()` for transformed
  scalar ranking in returns and reusable `Int` locals, reusing the guarded
  single-pass selection lowering and covered by
  `examples/e221_list_filter_map_max_min.vais`.
- Promoted `List<Struct>.filter(...).map(...).max()`/`.min()` for direct
  record score ranking in returns and reusable `Int` locals, lowering projected
  fields through the same guarded single-pass selection loop and covered by
  `examples/e220_list_struct_filter_map_max_min.vais`.
- Promoted `List<Int>.filter(...).max()`/`.min()` for filtered ranking
  selection in direct returns and reusable `Int` locals, lowering to a guarded
  single-pass selection loop without an intermediate list and covered by
  `examples/e219_list_filter_max_min.vais`.
- Promoted `List<Int>.min()` for local and parameter lists, sharing the
  full-path selection lowering with `List<Int>.max()` and covered by
  `examples/e218_list_int_min.vais`.
- Promoted `List<Int>.max()` for local and parameter lists, including negative
  values and empty-list runtime traps, covered by
  `examples/e217_list_int_max.vais`.

## 2026-06-30

- Promoted `List<Struct>.filter(...).map(...).sum()` score aggregation for
  declared record lists, lowering same-item field projections directly into an
  accumulator without an intermediate score list, covered by
  `examples/e216_list_struct_filter_map_sum.vais`.
- Promoted `List<Struct>.filter(...).len()` count lowering for declared record
  lists, so document-like `List<Doc>` values can count field predicates through
  direct returns plus typed/inferred `Int` locals, covered by
  `examples/e215_list_struct_filter_len_count.vais`.
- Promoted `List<Struct>.map(...)` field projection for declared record lists,
  so filtered `List<Doc>` values can project `Int` scores and annotated `Str`
  titles into reusable scalar lists, covered by
  `examples/e214_list_struct_map_projection.vais`.
- Promoted `List<Struct>.filter(...)` result-list lowering for declared record
  lists, so document-like `List<Doc>` values can be filtered by field
  predicates, reused locally, and returned from helpers, covered by
  `examples/e213_list_struct_filter_result.vais`.
- Promoted `List<Int>/List<Str>.filter(...).len()` count lowering, so filtered
  counts can be returned directly or stored in typed/inferred `Int` locals with
  known predicate captures, covered by
  `examples/e212_list_filter_len_count.vais`.
- Promoted non-capturing `List<Str>.map` and annotated `List<Str>.filter`
  result lists through the same source-prep lowering path, allowing verified
  string builtin bodies such as `str_lower(str_trim(w))` and predicates such as
  `str_contains(w, "ai") == 1` in full/direct gates with
  `examples/e204_list_str_map.vais` and `examples/e205_list_str_filter.vais`.
- Added receiver-based `List<Str>.filter` result type inference, so
  `let selected = words.filter(...)` lowers to `List<Str>` when `words` is a
  known `List<Str>`, covered by `examples/e206_list_str_filter_infer.vais`.
- Added `List<Str>` function-parameter tracking to list method lowering, so
  helper code can infer `words.map(|w| w)` and follow-on `filter(...)` results
  from `fn score(words: List<Str>)`, covered by
  `examples/e207_list_str_param_map_filter.vais`.
- Promoted `str_concat(left, right)` through the direct string helper path and
  allowed it in non-capturing `List<Str>.map` closure bodies, covered by
  `examples/e208_list_str_map_concat.vais`.
- Promoted `List<Str>.filter/map` closure captures for known `Str` parameters
  and locals, so helper code can use patterns such as
  `words.filter(|w| str_contains(w, needle) == 1)` and
  `selected.map(|w| str_concat(w, suffix))`, covered by
  `examples/e209_list_str_closure_capture.vais`.
- Promoted `List<Int>.filter/map/filter-sum` closure captures for known `Int`
  parameters and locals, so helper code can use patterns such as
  `nums.filter(|n| n > min)`, `selected.map(|n| n + offset)`, and
  `nums.filter(|n| n > min).sum()`, covered by
  `examples/e210_list_int_closure_capture.vais`.
- Promoted `List<Int>.filter(...).sum()` assignment lowering, so filtered sums
  can be stored in typed or inferred `Int` locals and reused in follow-on
  calculations, covered by `examples/e211_list_filter_sum_assignment.vais`.
- Promoted non-capturing `List<Int>.filter(|x| predicate)` result lists, so
  `let ys = xs.filter(|x| x > 3)` lowers to a reusable `List<Int>` in full and
  direct paths, with `examples/e203_list_filter_result.vais`.
- Promoted `proc_capture(argv: List<Str>) -> ProcessResult` for the standard
  `ProcessResult { code: Int, stdout: Str, stderr: Str }` shape, so Vais tools
  can capture child exit status, stdout, and stderr in memory through full and
  direct gates, with `examples/e202_proc_capture_result.vais`.
- Promoted `List<Struct>` mutating method-result `Str` field reads for
  document-like records, so `docs.pop().title` and
  `docs.remove_at(i).body.len()` are gate-backed in full/direct with
  `examples/e201_list_struct_str_method_fields.vais`.
- Promoted indexed `List<Struct>` `Str` field writes for document-like records,
  so `docs[i].title = title` and `docs[i].body = body` work on local and
  parameter lists in full/direct, with front/direct/full parity coverage and
  `examples/e200_list_struct_str_field_write.vais`.
- Promoted `Str` fields in structs for document-like records, so
  `Doc { title: Str, body: Str, score: Int }` works in full/direct with string
  equality, `str_contains`, `.len()` chains, and `List<Doc>` index/first/last
  field reads, with front/direct/full parity coverage and
  `examples/e198_struct_str_fields.vais` plus
  `examples/e199_list_struct_str_fields.vais`.
- Promoted `List<Struct>` elements containing multi-field nested structs, so
  list storage now uses flat struct widths for push, indexed nested reads/writes,
  whole-element copy/assignment, parameter mutation, and non-mutating
  method-result nested field-chain reads in full/direct, with
  front/direct/full parity coverage and
  `examples/e197_list_multi_field_nested_struct.vais`.
- Promoted scalar multi-field nested structs, so `Outer { inner: Inner }`
  where `Inner` has multiple `Int` fields can be initialized locally, returned
  directly from helpers, and read through chains like `o.inner.a` and
  `make(...).inner.b` in full/direct, with front/direct/full parity coverage
  and `examples/e196_multi_field_nested_struct.vais`.
- Promoted direct returns of single-field nested struct literals, so helpers can
  `return Outer { inner: Inner { v: value } }` and callers can immediately read
  `make(...).inner.v` through full/direct flattening, with front/direct/full
  parity coverage and `examples/e195_nested_struct_literal_return.vais`.
- Promoted struct-returning helper field-chain reads, so
  `make_box(...).value` and the verified single-field nested shape
  `make_outer(...).inner.v` lower through full/direct call-result materialization,
  with front/direct/full parity coverage and
  `examples/e194_struct_return_field_chain.vais`.
- Promoted `List<Struct>` method-result nested field-chain reads for elements
  containing a previously declared single-`Int`-field nested struct, so
  `xs.first().inner.v`, `xs.last().inner.v`, `xs.pop().inner.v`, and
  `xs.remove_at(i).inner.v` lower through the same flattened field slot in
  full/direct, with front/direct/full parity coverage and
  `examples/e193_list_nested_struct_method_field_chain.vais`.
- Promoted indexed field-chain writes on `List<Struct>` elements that contain a
  previously declared single-`Int`-field nested struct, so
  `xs[0].inner.v = ...` mutates the flattened list element slot for local and
  parameter lists, with front/direct/full parity coverage and
  `examples/e192_list_nested_struct_field_chain_write.vais`.
- Promoted indexed field-chain reads on `List<Struct>` elements that contain a
  previously declared single-`Int`-field nested struct, so
  `xs[0].inner.v` works through full and direct flattening after nested struct
  literals are pushed into a list, with front/direct/full parity coverage and
  `examples/e191_list_nested_struct_field_chain.vais`.
- Promoted native direct flattening for previously declared single-`Int`-field
  nested structs, so multiline `Outer { inner: Inner { ... } }` literals,
  `o.inner.v` reads, and `o.inner.v = ...` writes work in full/direct with
  front/direct/full parity coverage and
  `examples/e190_direct_nested_struct_multiline.vais`.
- Promoted `List<Struct>.insert_at(index, Box { ... })` and
  `List<Struct>.extend([Box { ... }])` with multiline struct literal sources
  in the full self-host path and native direct engine, with front/direct/full
  parity coverage and
  `examples/e189_list_struct_multiline_insert_extend.vais`.
- Promoted multiline struct literals for plain struct local initialization,
  typed local initialization, same-type local assignment, and struct call
  arguments in the full self-host path and native direct engine, replacing a
  full-path semicolon-only statement advance and scalar assignment fallthrough
  with brace-aware struct field stores, with front/direct/full parity coverage
  and `examples/e188_struct_multiline_local_assignment_call.vais`.
- Promoted multiline struct literals in `List<Struct>` indexed element
  assignment and struct-returning `return` statements in the native direct
  engine, teaching direct statement joining to track declared-struct literal
  braces without swallowing function/control blocks, with front/direct/full
  parity coverage and
  `examples/e187_list_struct_multiline_assignment_return.vais`.
- Promoted `List<Struct>.push(Box { ... })` with multiline trailing-comma
  struct literals in the native direct engine, broadening direct statement
  joining from square-only to paren/square-aware calls and teaching the direct
  struct literal rewriter to skip trailing empty field parts, with
  front/direct/full parity coverage and
  `examples/e186_list_struct_push_multiline_literal.vais`.
- Promoted standalone call statements with multiline inline `List<Struct>`
  literal arguments and trailing commas in the full self-host path and native
  direct engine, fixing semicolon-free full statement-call advancement and
  adding generic direct function-call statement lowering that reuses expression
  argument rewriting, with front/direct/full parity coverage and
  `examples/e185_list_struct_multiline_inline_arg_statement.vais`.
- Promoted multiline inline `List<Struct>` literal call arguments with trailing
  commas in the full self-host path, reusing the typed list literal
  materializer for temporary call-argument buffers and keeping the native direct
  engine gated for the same syntax, with front/direct/full parity coverage and
  `examples/e184_list_struct_multiline_inline_arg.vais`.
- Promoted multiline typed `List<Struct>` literals with trailing commas in the
  full self-host path and native direct engine, preventing full-source
  semicolon normalization and statement advancement from splitting list
  literals, and teaching direct lowering to join bracketed statements and treat
  newlines as whitespace. The same slice also tightened semicolon-free full
  statement advancement for list methods and `let` initializers, and added an
  import-graph fast path for no-import tool files, with front/direct/full
  parity coverage and `examples/e183_list_struct_multiline_literal.vais`.
- Promoted direct `List<Struct>` method-result field chains
  (`xs.first().field`, `xs.last().field`, `xs.pop().field`, and
  `xs.remove_at(index).field`) in the full self-host path, sharing the existing
  list-method struct materializer and fixing slot inference so chained field
  reads bind as scalar locals, with front/direct/full parity coverage and
  `examples/e182_list_struct_method_field_chain.vais`.
- Promoted `List<Struct>` typed local initialization and local/parameter
  assignment from inline struct list literals in the full self-host path,
  sharing one literal materializer for scalar, string, and struct list data,
  with front/direct/full parity coverage and
  `examples/e181_list_struct_literal_assignment.vais`.
- Promoted `List<Struct>.extend([Struct { .. }])` from inline struct list
  literal sources in the full self-host path and native direct engine for local
  and parameter target lists, materializing struct literal fields into temporary
  list buffers before the existing extend copy loop, with front/direct/full
  parity coverage and
  `examples/e180_list_struct_extend_inline_literal_source.vais`.

## 2026-06-29

- Promoted `List<Int>.extend([..])` and `List<Str>.extend([..])` from inline
  list literal sources in the full self-host path and native direct engine for
  local and parameter target lists, materializing literal sources into temporary
  list buffers before the existing extend copy loop, with front/direct/full
  parity coverage and `examples/e179_list_extend_inline_literal_source.vais`.
- Promoted `List<Int>.extend(make_list(...))` and
  `List<Str>.extend(make_list(...))` from same-type list-returning helper calls
  in the full self-host path and native direct engine for local and parameter
  target lists, and fixed the full path so `List<Int>.sum()` on list
  parameters emits an actual accumulation loop instead of falling through to
  length reads, with front/direct/full/parity coverage and
  `examples/e178_list_scalar_str_extend_return_call.vais`.
- Promoted `List<Struct>.extend(make_list(...))` from same-type
  list-returning helper calls in the full self-host path and native direct
  engine for local and parameter target lists, materializing the returned list
  into a temporary source buffer before reusing the existing extend copy loop,
  with front/direct/full/parity coverage and
  `examples/e177_list_struct_extend_return_call.vais`.
- Added dedicated front/direct/full/parity coverage for
  `List<Struct>.push(xs.first()/xs.last())` and
  `List<Struct>.insert_at(index, xs.first()/xs.last())` from non-mutating
  same-type list method return values, including same-list insertion
  materialization, with
  `examples/e176_list_struct_push_insert_first_last_value.vais`.
- Promoted `List<Struct>.push(xs.pop()/xs.remove_at(i))` and
  `List<Struct>.insert_at(index, xs.pop()/xs.remove_at(i))` from same-type list
  method return values in the full self-host path for local and parameter
  lists, materializing returned struct elements before target writes while
  preserving source `pop`/`remove_at` mutation, with front/direct/full
  regression cases and
  `examples/e175_list_struct_push_insert_method_value.vais`.
- Promoted `List<Struct>.push(xs[i])` and
  `List<Struct>.insert_at(index, xs[i])` from same-type list element values in
  the full self-host path for local and parameter lists, materializing
  `insert_at` element values before shifting to preserve same-list semantics,
  with front/direct/full regression cases and
  `examples/e174_list_struct_push_insert_element_value.vais`.
- Reworked the Vais-authored stage IR normalizer to store global-name mappings
  as 4-field integer struct entries instead of paired `List<Str>` values, so
  full self-host stage comparison handles compiler IR with more than 4,096
  distinct string globals.
- Raised the native direct list backing cap to match the full path's 4,096
  scalar slots and added capacity traps before direct list literal writes and
  `push` writes, keeping `insert_at`/`extend` checks on the same cap.
- Promoted `List<Struct>.push(value)` from same-type struct local/parameter
  values in the full self-host path for local and parameter lists, copying the
  source struct fields into the next list slot, with front/direct/full
  regression cases and `examples/e173_list_struct_push_struct_value.vais`.
- Promoted `List<Struct>.insert_at(index, make_struct(...))` from
  struct-returning helper calls in the full self-host path for local and
  parameter lists, reusing the right-shift logic and storing the returned
  struct field-by-field into the insertion slot, with front/direct/full
  regression cases and `examples/e172_list_struct_insert_return_call.vais`.
- Promoted `List<Struct>.push(make_struct(...))` from struct-returning helper
  calls in the full self-host path for local and parameter lists, lowering the
  returned struct through a temporary out buffer and field-by-field list
  storage, with front/direct/full regression cases and
  `examples/e171_list_struct_push_return_call.vais`.
- Promoted `List<Struct>` indexed whole-element assignment from
  struct-returning calls in the full self-host path, lowering `xs[i] =
  make_box(...)` through a temporary struct out-param and field-by-field list
  storage for local and parameter lists, with front/direct/full regression
  cases and `examples/e170_list_struct_element_return_call.vais`.
- Promoted `List<Struct>` indexed whole-element assignment for local and
  parameter lists in the full self-host path, copying struct literals, same-type
  struct locals, and same-type list elements field-by-field with list bounds
  checks, and adding front/full regression cases,
  `examples/e169_list_struct_element_assignment.vais`, parity, docs, and core
  regen coverage.
- Hardened semicolon-free indexed element assignment for local `List<Int>` in
  the full path by teaching the native source normalizer to terminate
  `xs[index] = value` statements and making the self-host indexed-assignment
  lowering use source-position statement boundaries, with front/full regression
  cases, `examples/e168_list_index_assignment.vais`, parity, docs, and core
  regen coverage.
- Promoted `List<Struct>` indexed field assignment for local and parameter
  lists in the full self-host path, storing through `xs[index].field = value`
  with bounds checks and struct-field stride math, and adding front/full
  regression cases, `examples/e167_list_struct_field_write.vais`, parity,
  docs, and core regen coverage.
- Promoted `List<Struct>` for-each over local and parameter lists, copying each
  declared-struct element field-by-field into the loop variable, adding full
  self-host lowering, direct native acceptance/lowering, front/direct/full
  regression cases, `examples/e166_list_struct_for_each.vais`, parity, docs,
  and core regen coverage.
- Promoted `List<Struct>.extend(other)` for local and parameter lists, copying
  declared-struct elements field-by-field from same-type named source lists,
  preserving combined-length capacity traps, and adding full self-host lowering,
  direct native lowering, front/direct/full regression cases,
  `examples/e165_list_struct_extend.vais`, parity, docs, and core regen
  coverage.
- Promoted `List<Struct>.insert_at(index, value)` for local and parameter lists,
  shifting following struct elements right field-by-field, accepting
  declared-struct literal/local/parameter values, preserving insert bounds and
  capacity trap behavior, and adding full self-host lowering, direct native
  lowering, front/direct/full regression cases,
  `examples/e164_list_struct_insert_at.vais`, parity, docs, and core regen
  coverage.
- Promoted `List<Struct>.remove_at(index)` for local and parameter lists,
  returning the removed struct in a local binding, shifting following struct
  elements left field-by-field, preserving bounds trap behavior, and adding
  full self-host lowering, direct native lowering, front/direct/full regression
  cases, `examples/e163_list_struct_remove_at.vais`, parity, docs, and core
  regen coverage.

## 2026-06-28

- Promoted `List<Struct>.first()` for local and parameter lists, copying the
  head element into struct locals without mutation, preserving empty-list trap
  semantics, and adding full self-host lowering, front/direct/full regression
  cases, `examples/e162_list_struct_first.vais`, parity, docs, and core regen
  coverage.
- Promoted `List<Int>.first()` and `List<Str>.first()` for local and parameter
  lists, returning the head element without mutation, trapping on empty lists,
  allowing direct `.len()` on returned `List<Str>` elements, and adding full
  self-host lowering, direct native lowering, front/direct/full regression
  cases, `examples/e161_list_first.vais`, regenerated core, and parity/docs
  updates.
- Promoted `List<Int>.extend(other)` and `List<Str>.extend(other)` for local
  and parameter lists with same-type named list sources, appending source
  elements after a capacity check, supporting self-extension through source
  length snapshotting, and adding full self-host lowering, direct native runtime
  helpers, front/direct/full regression cases, `examples/e160_list_extend.vais`,
  and parity/docs updates.
- Promoted `List<Int>.insert_at(index, value)` and
  `List<Str>.insert_at(index, value)` for local and parameter lists, inserting
  before an index or at `len`, shifting following elements right, trapping on
  invalid indexes or full buffers, and adding full self-host lowering, direct
  native runtime helpers, front/direct/full regression cases,
  `examples/e159_list_insert_at.vais`, and parity/docs updates.
- Promoted `List<Int>.remove_at(index)` and `List<Str>.remove_at(index)` for
  local and parameter lists, returning the removed element, shifting following
  elements left, and adding full self-host lowering, direct native runtime
  helpers, front/direct/full regression cases, `examples/e158_list_remove_at.vais`,
  and parity/docs updates.
- Promoted `List<Int>.index_of(value)` and `List<Int>.count(value)` for local
  and parameter integer lists, aligning the scalar list API with `List<Str>` and
  adding full self-host lowering, direct native lowering, front/direct/full
  regression cases, `examples/e157_list_int_index_count.vais`, and parity/docs
  updates.
- Promoted `List<Str>.count(value)` for local and parameter string lists,
  returning the matching string count or `0` when missing, including full
  self-host lowering, direct native lowering, front/direct/full regression
  cases, `examples/e156_list_str_count.vais`, and parity/docs updates.
- Promoted `List<Str>.index_of(value)` for local and parameter string lists,
  returning the first matching index or `-1`, including full self-host lowering,
  direct native lowering, front/direct/full regression cases,
  `examples/e155_list_str_index_of.vais`, and parity/docs updates.
- Promoted `List<Str>.contains(value)` for local and parameter string lists,
  including full self-host lowering through `__vais_str_eq`, direct native
  lowering, front/direct/full regression cases,
  `examples/e154_list_str_contains.vais`, and parity/docs updates.
- Promoted `List<Int>.contains(value)` for local and parameter integer lists,
  including full self-host lowering, direct native lowering, front/direct/full
  regression cases, `examples/e153_list_contains.vais`, and parity/docs updates.
- Promoted `List.clear()` for local and parameter `List<Int>`/`List<Str>`/
  `List<Struct>` reuse, including full self-host lowering, direct native
  lowering, front/direct/full regression cases, `examples/e152_list_clear.vais`,
  and parity/docs updates.
- Added `str_starts_with(text, prefix)` as a verified full/direct prefix
  primitive returning `1` for matching or empty prefixes and `0` otherwise,
  including `examples/e150_str_starts_with_builtin.vais`, direct native
  lowering, full self-host LLVM helper generation, and regenerated core.
- Added full-engine `List.push` capacity traps for local and parameter
  `List<Int>`/`List<Struct>` push paths, preventing fixed backing buffers from
  silently overwriting adjacent compiler state when full.
- Hardened self-host tokenization so `#` line comments are skipped through the
  end of the line while `#` bytes inside string literals remain intact, including
  `examples/e151_line_comment_skip.vais` and a full-codegen regression case.
- Raised the full-engine 4-field struct list cap to 131,072 slots and applied
  the macOS 64MB stack link option to native compiler builds, keeping the
  growing self-host compiler source stable under embedded self-source probes.
- Added `str_index_of(text, needle)` as a verified full/direct string search
  primitive returning the first byte index, `-1` when absent, and `0` for an
  empty needle, including `examples/e149_str_index_of_builtin.vais`, direct
  native lowering, full self-host LLVM helper generation, and regenerated core.
- Added `doc_term_weighted_score(query, doc)` as a verified repeated-term
  ranking primitive over `Map<Str,Int>` term-frequency maps, including direct
  native lowering, full self-host LLVM helper generation,
  `examples/e148_doc_term_weighted_score.vais`, and front/direct/full
  regression cases.
- Added `doc_term_overlap_score(query, doc)` as a verified first ranking
  primitive over `Map<Str,Int>` term-frequency maps, including direct native
  lowering, full self-host LLVM helper generation,
  `examples/e147_doc_term_overlap_score.vais`, and front/direct/full
  regression cases.

## 2026-06-27

- Added `str_split_ws_into(text, out)` as a verified whitespace-tokenization
  builtin for `List<Str>` out-params, including direct native lowering, full
  self-host LLVM helper generation, `examples/e145_str_split_ws_into.vais`, and
  front/direct/full regression cases.
- Added `doc_term_counts_into(text, out)` as a verified document
  term-frequency builtin for `Map<Str,Int>` out-params, including direct native
  lowering, full self-host LLVM helper generation, `examples/e146_doc_term_counts_into.vais`,
  and front/direct/full regression cases.
- Promoted local `List<Str>` element bindings: `let s = words[index]`,
  `let s = words.last()`, and `let s = words.pop()` now allocate `Str` slots and
  lower stored pointer values back to `i8*`; regenerated
  `compiler/self/vaisc_core.ll` and added `examples/e121_list_str_methods.vais`.
- Promoted `List<Str>` helper parameters through the public preflight and
  self-host full-engine lowering; parameter index reads now lower stored string
  pointers back to `i8*`, with `examples/e122_list_str_param.vais` in the
  parity corpus.
- Promoted `List<Str>` helper returns by separating the `call_retty` no-list
  sentinel from the `List<Str>` element tag, allowing returned string lists to
  initialize locals and keep `index`/`last`/`pop` as `Str` values.
- Promoted typed local `List<Str>` literals by recording `List<Str>` annotations
  as scalar list slots and lowering literal elements through `ptrtoint` before
  storing them in the native list buffer.
- Fixed `embed_self_source` normalization performance for already-embedded
  one-line compiler sources by caching scanned string lengths in hot loops; this
  keeps the full self-host gate practical after large source retargeting.
- Promoted local `List<Str>` assignment copy and literal assignment, including
  string-pointer materialization for assigned literals and value-copy semantics
  that preserve the source list after the target is mutated.
- Promoted `List<Str>` return-call assignment and parameter-target assignment
  forms, so helpers can replace caller-provided string-list storage from another
  list, a literal, or a `List<Str>`-returning call.
- Raised the full-engine 4-field struct list cap from 65,536 to 81,920 slots so
  embedded self-host probes can tokenize the expanded compiler source without
  overwriting the fixed `List<Token>` stack buffer.
- Promoted `List<Str>` inline literal call arguments by materializing string
  element pointers into temporary scalar list buffers before helper calls; the
  e131/front contract covers direct `words[i] != "literal"` comparisons.
- Promoted `Map<Str,Str>` local values, assignment copy, function parameters,
  return-value local initialization, and string-valued `insert`, `get`,
  `get_opt` match binding, `remove`, `clear`, `contains`, and `len`; string
  option matching lowers through `contains` plus `get(key, "")` while the
  broader explicit `Option<Str>` type remains unpromoted.
- Added concrete Map `key_at(index)` / `value_at(index)` entry reads for
  serialization/debugging loops across the verified Map surface, including
  `Map<Str,Str>` for document metadata and VaisDB-style storage probes.
- Added a full-engine `Map<Str,Str>` snapshot example that serializes entries
  with `str_builder`, writes them through `fs_write_text`, and reads them back
  with `fs_read_text`; this is the first end-to-end VaisDB-style persistence
  smoke on top of the promoted Map entry API.
- Extended the persistence smoke with a loader that parses a text snapshot back
  into `Map<Str,Str>` using dynamic strings from `str_builder_finish`, proving a
  minimal save/read/rebuild cycle for document metadata.
- Promoted local Map type inference for Map-returning calls, so helpers such as
  `make_docs() -> Map<Str,Str>` can be consumed as `let docs = make_docs()`
  while still lowering through caller-owned Map storage.
- Promoted `.len()` chains on Str-returning Map reads, so `docs.get("title", "").len()`
  and `docs.value_at(1).len()` compile through both the full self-host compiler
  and the direct native emitter.
- Promoted `.len()` chains on `List<Str>` index, `last()`, and `pop()` results,
  including returned-list locals such as `let words = make_words()`, across the
  full self-host compiler and direct native emitter.
- Promoted direct string equality on `List<Str>` index, `last()`, and `pop()`
  results into the examples and front/direct/full codegen gates, so string-list
  element checks no longer need temporary `Str` bindings.
- Promoted `str_contains(text, needle)` as a verified string builtin across the
  full self-host compiler and direct native emitter, including document-field
  probes over `Map<Str,Str>` values for the VaisDB path.
- Extended direct `str_contains` checks to accept `List<Str>` index, `last()`,
  and `pop()` results directly, aligning document-field and token-list string
  probes for Veriqel-style retrieval code.
- Promoted `str_trim(text)` as a verified full/direct string cleanup builtin,
  with coverage over literals, `Map<Str,Str>` document fields, and `List<Str>`
  token-list values.
- Promoted `str_lower(text)` as a verified full/direct ASCII case-normalization
  builtin, including `str_lower(str_trim(...))` document-field cleanup and
  `List<Str>` token normalization coverage.
- Added native direct `str_slice(text, start, len)` support and a document
  tokenization example that returns `List<Str>` from `str_slice -> str_trim ->
  str_lower`, giving Veriqel/VaisDB a gate-backed minimum preprocessing shape.
- Extended full/direct `for word in words` support to local and parameter
  `List<Str>` values, with a document-token scoring example over normalized
  tokens.

## 2026-06-26

- Prepared the final stable `v1.0.1` release line while preserving the archived
  public `v1.0.0` tag: bumped compiler/site version metadata, updated
  changelog, release checklist, README/docs, site copy, and ROADMAP from the
  verified `v0.3.2` release-candidate surface.
- Cut and verified the final stable `v1.0.1` tag: release archive workflow
  passed for Linux x64, macOS arm64, and macOS x64; the GitHub Release is
  published; and the live `vaislang.dev` homepage links `v1.0.1`.
- Prepared and published the `v0.3.2` release-candidate tag from clean mainline:
  updated compiler/site version metadata, cut the annotated tag, verified the
  GitHub Release archives for Linux x64, macOS arm64, and macOS x64, verified
  the GitHub Pages deploy, and reran `bash scripts/test-release-gates.sh` from
  the clean tagged checkout.
- Published the `v0.3.2` docs/site copy from canonical repository docs and
  verified the live `vaislang.dev` homepage exposes the current release and
  release archive link.

## 2026-06-25

- Extended the Vais-authored local import graph checker to follow the first
  package manifest local dependency alias and dependency-internal plain imports.
- Extended the Vais-authored local import graph checker to follow all declared
  entry-package local dependency aliases.
- Wired `scripts/vaisc` to run cached Vais-authored package manifest and import
  graph preflight tools before native `emit-ir`, `build`, and `run`.
- Closed the Phase 5 self-host expansion checklist after release gates confirmed
  regenerated core, preflight, import graph, and self-host paths remain green.
- Froze the v1-candidate language and prelude reference docs around the current
  gate-backed surface.
- Added a Vais-authored local import graph contract checker and release gate for
  manifest-free missing import, duplicate top-level symbol, and import cycle
  diagnostics.
- Reconciled the Phase 4.2 parent roadmap checkbox now that the listed enum
  payload and pattern/match slices are all gate-backed.
- Added optional entry-path source-root containment checking to the
  Vais-authored package manifest checker contract.
- Added local dependency cycle detection to the Vais-authored package manifest
  checker contract using normalized local manifest paths.
- Moved the package manifest missing-source-directory diagnostic into the
  Vais-authored manifest checker contract, matching the native driver error
  while preserving the product driver's OS-facing package discovery boundary.
- Added `tools/vais_manifest_check.vais` and
  `tools/vais_manifest_contract_check.vais` as a Vais-authored package manifest
  contract gate, then wired it into the release gate through
  `scripts/test-vais-manifest-check-vais.sh`.
- Closed Phase 5.2 for the current compiler-owned static source diagnostics:
  remaining native-front-only closure/enum/match rejects are not public checker
  rejects because the full language already verifies those features, and
  manifest/import graph/source-path diagnostics remain explicit 5.3 host-boundary
  work.
- Added invalid `main` entrypoint signature detection to the Vais-authored
  checker contract while preserving function-type and closure examples.
- Added missing helper return-type detection to the Vais-authored checker
  contract and kept function-type values out of that diagnostic path.
- Added unsupported generic `Map<K,V>` detection to the Vais-authored checker
  contract, kept verified concrete Map shapes clean, and updated the standalone
  checker issue count.
- Added unsupported generic `Result<T,E>` detection to the Vais-authored checker
  contract, kept the standalone checker issue count aligned, and preserved
  verified `Result<Int,Int>` as the only claimed Result shape.
- Added unsupported generic `Option<T>` detection to the Vais-authored checker
  contract, kept verified `Option<Int>` examples clean, and updated the
  standalone install/package checker issue count.
- Moved invalid static import path checking into the Vais-authored checker
  contract, added the matching public front reject fixture, and updated the
  checker fixture count to keep `scripts/vais-check` and `scripts/vaisc`
  diagnostic shapes aligned.
- Reconciled Phase 5 roadmap status for the existing stage comparison gate:
  `tools/normalize_stage_ir.vais` is already covered by a focused gate and by
  the full-source self-host stage1/stage2 comparison.
- Promoted `examples/e120_enum_payload_wildcard.vais` into the release corpus
  as the first payload enum `match` with `_` catch-all slice, with matching
  public front fixture, parity entry, docs, site count, changelog, and roadmap
  updates.
- Added `examples/e119_map_param_target_assignment.vais` to cover
  parameter-target assignment copies for every verified concrete Map type,
  updated front and full self-host codegen coverage, docs, site count,
  changelog, and roadmap while keeping generic Map behavior gated.
- Reconciled Phase 4 parent roadmap checkboxes for the completed
  Map/Option/Result and unsupported-syntax diagnostic slices.
- Promoted `examples/e25_for_filter_sum.vais`, `examples/e27_list_max.vais`,
  and `examples/fr2.vais` into the release corpus as gate-backed collection
  for-each examples, covering full self-host array iteration, scalar
  `List<Int>` local/parameter iteration, typed non-empty local `List<Int>`
  literals, inline `List<Int>` literal call arguments, native direct
  `List<Int>` iteration, parity, value, docs, and site count updates.
- Promoted `examples/e82_list_literal_direct_arg.vais` into the release corpus
  as the direct public smoke for inline `List<Int>` literal call arguments.
- Promoted `examples/e63_generic_struct_def.vais` into the release corpus as a
  front/parity/value-gated generic marker struct example used with `Int` values.
- Promoted six struct helper examples into the release corpus, covering
  struct-returning helpers, struct parameter helpers, assignment from
  struct-returning calls, recursive struct accumulators, and multi-value struct
  returns through full self-host and native direct gates.
- Promoted the local module, package source-root, and local dependency package
  examples into the release corpus as value-gated import/package smokes.
- Fixed and promoted single-field nested struct literal/read/write lowering in
  the full self-host compiler, including regenerated reusable core coverage.
- Promoted additional already-correct Result propagation, inline `List<Int>`
  parameter iteration, and direct `Option<Int>` match examples into the release
  corpus.
- Promoted borrowed `&List<Int>` helper parameters through the public front and
  release corpus with recursive traversal and binary-search examples.
- Promoted public struct/function modifiers and `Str` fields in struct literals
  through the checker, public front, full self-host compiler, regenerated core,
  and release corpus with `examples/d5run.vais`.
- Promoted already-supported `examples/t2.vais`, `examples/t3.vais`, and
  `examples/t5.vais` into the release corpus as enum, bitwise, and Option smoke
  coverage.
- Promoted `examples/d2.vais` into the release corpus by lowering multiline
  `Option<Int>` expression-match bindings through the public compiler driver
  before the self-host core receives the source.
- Promoted `examples/e73_int_to_string.vais` into the release corpus by adding
  full self-host and native direct lowering for `Str(Int)` decimal conversion,
  with regenerated reusable compiler core and front/direct/full gate fixtures.
- Promoted `examples/e46_generic_struct.vais` into the release corpus by
  lowering generic identity helpers applied directly to struct literals before
  the self-host core receives the source, with front/parity/value gate coverage.
- Promoted `examples/e51_index_ast.vais` into the release corpus by extending
  self-host `StructDef` field metadata to 20 stored fields and regenerating the
  reusable compiler core.
- Promoted `examples/e59_tuple.vais` into the release corpus by lowering `Int`
  tuple function returns and local destructuring to generated struct storage in
  the public compiler driver.
- Promoted `examples/e81_closure_return_apply.vais` into the release corpus by
  lowering a returned single-`Int` closure passed to an `Int` higher-order
  helper into the existing environment/apply helper representation.
- Promoted `examples/e09_struct_method.vais` into the release corpus by
  lowering simple `impl` struct methods and return-expression method chains to
  ordinary helper calls with intermediate struct locals.
- Promoted `examples/e49_closure_arg.vais` into the release corpus by lowering
  non-capturing inline closure literals passed to a single-closure `Int`
  higher-order helper into generated apply helpers.
- Promoted `examples/c5.vais` into the release corpus by lowering a local
  closure with one `Int` capture to an apply helper and captured environment
  value.
- Promoted `examples/e78_trait_impl_for.vais` into the release corpus by
  treating a simple `trait` declaration as metadata and lowering
  `impl Trait for Struct` methods to ordinary struct helper calls.
- Promoted `examples/e76_list_map.vais` and `examples/d6run.vais` into the
  release corpus by lowering non-capturing `List<Int>` map and filter-sum
  method slices to explicit `for` loops.
- Promoted `examples/e77_nested_list.vais` into the release corpus by lowering
  a local `List<List<Int>>` literal to row `List<Int>` locals and rewriting the
  verified double-index read.
- Promoted `examples/e79_nested_match.vais` into the release corpus by allowing
  a single enum `Option<Int>` payload and lowering its nested Option match arm
  to Int-coded branches.

## 2026-06-24

- Promoted `examples/t4.vais` and `examples/t6.vais` into the release corpus as
  simple struct literal/field-read smoke examples, raising the release corpus
  to 100 native-supported examples.
- Promoted `examples/fr1.vais` into the release corpus as an inclusive range
  for-loop summation smoke, raising the release corpus to 98 native-supported
  examples.
- Promoted `examples/e19_interpolation_print.vais` into the release corpus,
  adding native direct lowering for `print("...{name}...")` interpolation and
  `putchar(Int)` output calls.
- Promoted `examples/e71_string_index_of.vais` into the release corpus as a
  `Str` substring-search pattern with computed byte indexes, covering public
  front, native direct, full self-host codegen, parity, value, docs, and site
  count updates.
- Promoted `examples/e69_palindrome_string.vais` into the release corpus as a
  two-pointer `Str` scan with computed byte indexes from both ends, covering the
  same front, direct, full self-host, parity, value, docs, and site gates.
- Promoted 12 smaller control-flow, Bool predicate, integer-list, and `Str`
  scanner examples into the parity manifest and value corpus, raising the
  release corpus to 96 native-supported examples.

## 2026-06-20

- Prepared the `v0.3.0` source release metadata across the native compiler
  version, website package metadata, changelog, release checklist, and roadmap.
- Fixed self-host `print`/`puts` lowering for string-expression arguments,
  regenerated `compiler/self/vaisc_core.ll`, and promoted the fix as the
  `v0.3.1` patch release line because `v0.3.0` release assets had already been
  published before the darwin-arm64 archive failure was diagnosed.

## 2026-06-19

- Added `tools/vais_check_core.vais` as the first Vais-authored checker slice,
  covering the public non-Vais spelling fixture catalog while reading fixture
  files through verified host file APIs.
- Added `tools/vais_check_smoke.vais`, checker fixtures, and
  `scripts/test-vais-check-vais.sh` as the Vais checker contract gate.
- Expanded the Vais checker slice to cover the main spelling catalog, added a
  fixture count check, and made `.vais` source files visible to git by removing
  the stale ignore rule.
- Added line/column/help output to the Vais-authored checker slice and extended
  the checker contract gate to require error, coordinate, and help counts.
- Added `proc_argc()` and `proc_arg(index)` for `vaisc run -- ...` programs,
  then added `tools/vais_check_cli.vais` as an argv-backed checker entrypoint
  with bad/clean fixture gates.
- Extended `proc_argc()` and `proc_arg(index)` to `vaisc build` binaries by
  linking generated programs through a host runtime `main(argc, argv)` wrapper.
- Added `proc_capture_stdout(argv: List<Str>) -> Str` as the first captured
  process-output slice for Vais-authored repository tools.
- Added `proc_capture_stderr(argv: List<Str>) -> Str` as the captured
  diagnostics stream slice and regenerated `compiler/self/vaisc_core.ll`.
- Promoted the Vais checker to `scripts/vais-check`, then installed and
  packaged it as standalone `bin/vais-check` with install/package gate coverage.
- Expanded the clean checker fixture to cover the former unit-test
  false-positive catalog and removed that separate unit test from the release
  gate.
- Removed checker oracle use from the checker release gate; the public
  `scripts/vais-check` command is now checked by Vais fixture contracts.
- Added verified host-backed `Str` construction helpers `str_concat`,
  `str_slice`, and `str_byte`, regenerated `compiler/self/vaisc_core.ll`, and
  extended the host smoke gate to cover native build/run runtime support.
- Added full-engine lowering for `Str` reassignment and user-defined
  `-> Str` returns, then covered both through the host smoke gate.
- Added full self-host runtime lowering for `Str` equality/inequality,
  regenerated `compiler/self/vaisc_core.ll`, and restored the Vais checker CLI
  to idiomatic `path == "--help" or path == "-h"` syntax.
- Added `tools/package_vaisc_release.vais` as the Vais-authored release archive
  packager and reduced `scripts/package-vaisc-release.sh` to a thin wrapper
  that passes repo root, environment defaults, and CLI options into Vais.
- Added `tools/install_vaisc.vais` as the Vais-authored standalone installer
  and reduced `scripts/install-vaisc.sh` to a wrapper that passes repo root,
  environment defaults, and CLI options into Vais.
- Added verified `fs_remove(path)` and `tools/uninstall_vaisc.vais`, reducing
  `scripts/uninstall-vaisc.sh` to the same Vais-tool bootstrap wrapper shape.
- Added `tools/vaisc_install_check.vais` and reduced
  `scripts/test-vaisc-install.sh` to a bootstrap wrapper; installed binary
  smoke checks, checker fixture checks, archive extraction, packaged binary
  checks, and uninstall assertions now run in Vais.
- Added verified host-backed `Str` builder helpers for large text tools and
  regenerated `compiler/self/vaisc_core.ll`.
- Added `tools/embed_self_source.vais` as the Vais-authored self-source
  embedding helper, with byte-for-byte parity against the previous helper for
  checker fixtures and all self-host compiler tiers.
- Switched `scripts/test-fixpoint-full-self.sh` to build and use the Vais
  embed helper directly, and wired `scripts/test-embed-self-source-vais.sh`
  into the release gate.
- Added `tools/vaisc_errors_check.vais` as the Vais-authored NV-C3 diagnostics
  validator behind `scripts/test-vaisc-errors.sh`, using captured stderr to
  check coordinate, caret, help, and fix output.
- Added `tools/vaisc_front_check.vais` as the Vais-authored NV-C1 front
  contract validator behind `scripts/test-vaisc-front.sh`, including accepted
  source fixtures, rejected diagnostics, and package/import temp trees.
- Added `proc_run_env(argv, env)` for child-process environment overrides,
  extended the host smoke gate, and moved the direct-engine no-Python PATH
  check into `tools/vaisc_direct_env_check.vais`.
- Added `tools/vaisc_direct_smoke_check.vais` and moved the NV-C2 direct
  arithmetic/build/run smoke checks out of `scripts/test-vaisc-direct.sh`.
- Added `proc_capture_to(argv, stdout_path, stderr_path)` for status-plus-file
  process capture, extended the host smoke gate, and documented it as the
  pragmatic step before in-memory `ProcessResult`.
- Added `tools/vaisc_direct_error_check.vais` and moved direct import handling
  plus List bounds trap checks out of `scripts/test-vaisc-direct.sh`.
- Added `tools/vaisc_direct_feature_check.vais` and moved the direct
  helper/control-flow, range `for`, struct-local, and struct ABI success
  fixtures out of `scripts/test-vaisc-direct.sh`.
- Expanded `tools/vaisc_direct_feature_check.vais` to cover direct local
  `List<Int>`, `Str`, `Char`, `parse_uint`/`parse_int`, local `Map<Int,Int>`,
  and local `List<Struct>` success fixtures, and removed those cases from the
  direct shell wrapper.
- Moved the remaining direct List ABI, assignment, and returned-list hoist
  fixtures into `tools/vaisc_direct_feature_check.vais`, reducing
  `scripts/test-vaisc-direct.sh` to a bootstrap wrapper around Vais-authored
  direct validators.
- Added `tools/vaisc_direct_gate.vais` and reduced
  `scripts/test-vaisc-direct.sh` again so the NV-C2 direct-emitter gate
  orchestration itself runs from Vais; shell now only provides the temp-dir
  bootstrap boundary.
- Reduced the single-tool focused shell wrappers for checker contract, NV-C0,
  NV-C1, NV-C3, host, native smoke, legacy compiler smoke, fixpoint tiers,
  parity, value corpus, embed, and normalizer checks to invoke their
  Vais-authored gates with `scripts/vaisc run`; the wrappers now only provide
  temp directories and environment-specific bootstrap arguments.
- Added `tools/normalize_stage_ir_check.vais` and reduced
  `scripts/test-normalize-stage-ir-vais.sh` to a bootstrap wrapper; sample IR,
  expected-output comparison, and replacement-shape assertions now run in Vais.
- Added `tools/embed_self_source_check.vais` and reduced
  `scripts/test-embed-self-source-vais.sh` to a bootstrap wrapper; fixture
  writing, helper invocation, trust-root generated-compiler builds, clang IR
  validation, and binary result assertions now run in Vais.
- Added `tools/vais_check_contract_check.vais` and reduced
  `scripts/test-vais-check-vais.sh` to a bootstrap wrapper; checker output
  counts, diagnostic pattern assertions, real-path checks, help checks, and
  public `scripts/vais-check` wrapper checks now run in Vais.
- Added `tools/fixpoint_tier_check.vais` and reduced
  `scripts/test-fixpoint.sh` plus `scripts/test-fixpoint2.sh` to bootstrap
  wrappers; their compact-program fixtures, raw-call embedding, trust-root
  compiler builds, emitted-IR clang validation, and result assertions now run
  in Vais.
- Added `tools/fixpoint_full_self_check.vais` and reduced
  `scripts/test-fixpoint-full-self.sh` to a bootstrap wrapper; full-source
  self-host retargeting, generated compiler builds/runs, emitted IR checks,
  final binary assertions, and normalized stage comparison now run in Vais.
- Added `tools/fixpoint_full_codegen_check.vais` and reduced
  `scripts/test-fixpoint-full.sh` to a bootstrap wrapper; the long full-codegen
  fixture catalog, stdout/trap cases, source-file checks, and IR shape
  assertions now run in Vais.
- Audited the remaining host boundaries after the full-codegen port; the
  remaining shell is limited to native C bootstrap, public command cache
  wrappers, release/CI orchestration, website build tooling, system tools, and
  temp-dir bootstrap wrappers.
- Fixed native front-contract probes to ignore unsupported-syntax markers inside
  string, raw-string, character literal, and comment text.
- Added `tools/compiler_smoke_check.vais` as the Vais-authored legacy
  self-host compiler smoke validator behind `scripts/test-compiler.sh`,
  replacing shell `sed` retargeting with Vais string rewriting and wiring the
  smoke into `scripts/test-release-gates.sh`.
- Added full-engine local `List<Str>` index reads, regenerated
  `compiler/self/vaisc_core.ll`, and covered the path through a Vais-authored
  stage IR normalizer.
- Added `tools/normalize_stage_ir.vais`, parity-gated it against the previous
  helper, and switched `scripts/test-fixpoint-full-self.sh` to use the Vais
  normalizer for stage1/stage2 IR comparison.
- Switched the focused self-source embedding and stage IR normalizer gates from
  external parity oracles to Vais-only behavioral and expected-output checks,
  and removed those helper checks from the release gate.
- Added native self-host trust-root handling to `scripts/vaisc`.
- Fixed native source-prep parity for one-line struct fields and multi-field
  struct lines, removed the internal self-host compiler escape hatch, removed
  the fallback branch from `scripts/vaisc`, and verified the embed, normalizer,
  fixpoint, full-codegen, and full self-host gates through the native path.
- Promoted single-byte `Char` literal equality plus explicit `Char` locals,
  helper parameters, and helper returns through the native direct engine and
  front contract as Int-compatible scalar values, and added
  `examples/e85_char_type.vais` to the release corpus.
- Promoted exclusive `..` and inclusive `..=` range `for` loops through the
  native direct engine and front contract, and added
  `examples/e86_for_loop.vais` to the release corpus.
- Promoted `break` and `continue` inside `while` and range `for` loops through
  the native direct engine, full self-host compiler, front contract, and parity
  gates, and added `examples/e87_break_continue.vais` to the release corpus.
- Promoted explicit `Bool` locals, helper parameters, helper returns, and unary
  `not` through the native direct engine, full self-host compiler, front
  contract, and parity gates, and added `examples/e88_bool_type.vais` to the
  release corpus.
- Promoted explicit `Str` locals, helper parameters, helper returns,
  reassignment, length, index, and equality through the native direct engine,
  full self-host compiler, front contract, and parity gates, and added
  `examples/e89_str_type.vais` to the release corpus.
- Promoted simple expression-arm `match` lowering for multi-field `Int` payload
  enum variants through the public front contract, full self-host compiler, and
  parity gates, and added `examples/e02_enum_payload.vais` to the release
  corpus.
- Fixed enum type-token rewriting so struct literal values such as
  `c: Color.Green` are not mistaken for type annotations, then promoted
  payload-free enum struct-field matching with `examples/e24_struct_enum_field.vais`.
- Promoted single-field struct payload enum lowering through the public front
  contract and parity gates, with `examples/e64_enum_struct_payload.vais`
  covering constructor literal extraction and payload field access.
- Promoted Int `match` literal arms with `_` catch-all lowering through the
  public front contract and parity gates, with
  `examples/e55_match_wildcard.vais` added to the release corpus.
- Promoted payload-free enum `match` with `_` catch-all through the public front
  contract and parity gates, with `examples/e90_enum_wildcard.vais` added to
  the release corpus.
- Promoted payload enum `match` with `_` catch-all through the public front
  contract and parity gates, with `examples/e120_enum_payload_wildcard.vais`
  added to the release corpus.
- Added `tools/vais_parity_check.vais` as the Vais-authored NV-C4 parity
  manifest harness and reduced `scripts/test-vaisc-parity.sh` to a bootstrap
  wrapper.
- Added `tools/vais_value_check.vais` as the Vais-authored value-corpus
  build/run/exit-code harness and reduced `scripts/test.sh` to a bootstrap
  wrapper.
- Added `tools/vais_host_check.vais` as the Vais-authored host
  file/path/string/process smoke harness and reduced `scripts/test-vaisc-host.sh`
  to a bootstrap wrapper.
- Added `tools/vaisc_smoke_check.vais` as the Vais-authored NV-C0 public
  compiler smoke harness and reduced `scripts/test-vaisc.sh` to a bootstrap
  wrapper.
- Added `tools/vaisc_native_check.vais` as the Vais-authored native-driver
  smoke harness and reduced `scripts/test-vaisc-native.sh` to a bootstrap
  wrapper.
- Strengthened the Vais checker contract gate to assert real file paths in
  diagnostics and clean output, then fixed the checker CLI path output to use
  explicit `Str` concatenation.

## 2026-06-18

- Added the first `vais.toml` package manifest slice for `name`, `version`, and
  `source` source-root resolution.
- Added local dependency package paths under `vais.toml` `[dependencies]`, with
  dependency imports resolving through native public driver paths.
- Added native gates for package manifest success,
  dependency imports, and manifest diagnostics.
- Specified the Phase 3 host file/path/process API boundary in
  `docs/design/HOST_IO.md` and listed the APIs as specified in
  `std/PRELUDE.md`.
- Implemented `fs_exists(path: Str) -> Bool`, `fs_write_text(path: Str, text:
  Str) -> Int`, and `fs_mkdirs(path: Str) -> Int` as the first host-backed file
  intrinsics for full-engine builds, with the native public driver injecting
  the LLVM declarations and linking a small host runtime.
- Added `scripts/test-vaisc-host.sh` for native
  temp-directory existence, directory creation, and text write checks, and wired
  it into the release gate.
- Added `fs_read_text(path: Str) -> Str` as the first `Str`-returning
  host-backed intrinsic, regenerated `compiler/self/vaisc_core.ll`, and
  extended `scripts/test-vaisc-host.sh` to verify text reads through full-engine
  runs.
- Added verified path helpers `fs_cwd()`, `fs_temp_dir()`, `path_join(...)`,
  `path_basename(...)`, and `path_dirname(...)` as `Str`-returning host-backed
  intrinsics, regenerated `compiler/self/vaisc_core.ll`, and extended the host
  smoke gate to validate native path behavior.
- Added verified `proc_run(argv: List<Str>) -> Int` as the first process
  intrinsic, including full-engine `List<Str>` local `push` support for argv
  construction, native-driver host runtime support, and host smoke coverage for
  `emit-ir`, `build`, and `run`.

## 2026-06-17

- Replaced the stale Map example with the verified local `Map<Int,Int>` API:
  `{}`, `insert`, `get(key, default)`, `contains`, and `len`.
- Added a release-corpus List method example for `is_empty()`, `last()`, and
  `pop()`.
- Promoted both examples in `tools/vaisc-parity.tsv` and synced the roadmap,
  examples README, and changelog with the value corpus.
- Specified the Phase 2 module/package/import model in `docs/design/MODULES.md`.
- Added front and `vais-check` diagnostics for reserved `module` and `package`
  declarations.
- Implemented the first full-engine local import slice in the native public
  driver.
- Added gates for multi-file import success, missing imports, duplicate
  symbols, and import cycles.

## 2026-06-16

- Added `List<T>.is_empty()` to the full self-host compiler for local and
  parameter lists.
- Regenerated `compiler/self/vaisc_core.ll` from
  `compiler/self/fixpoint_full.vais`.
- Added native direct `List<Int>` and `List<Struct>` `is_empty()` lowering and
  diagnostics.
- Added full, front, direct, and error gate coverage for the promoted API.
- Synced `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`, roadmap, changelog,
  and website copy with the current gate-backed list surface.
- Ran `bash scripts/test-release-gates.sh`; it passed and produced
  `dist/vais-0.2.2-darwin-arm64.tar.gz`.
- Added `List<T>.last()` for non-empty lists to the full self-host compiler and
  native direct engine.
- Added struct-list `last()` binding coverage with `let item = xs.last()`.
- Updated front, direct, full, and diagnostic gates plus public docs for the
  promoted `last()` API.
- Added `List<T>.pop()` for non-empty lists to the full self-host compiler and
  native direct engine.
- Added scalar and struct-list `pop()` gate coverage, including parameter-list
  length mutation.
- Updated front, direct, full, and diagnostic gates plus public docs for the
  promoted `pop()` API.
- Added runtime trap behavior for invalid `List` access in the full self-host
  compiler and native direct engine.
- Added full and direct gate coverage for out-of-range index, empty `last()`,
  and empty `pop()` behavior.
- Updated `std/PRELUDE.md`, language reference, roadmap, changelog, and website
  copy for the list bounds trap contract.
- Promoted the first `Str` tool-helper slice through the public front contract,
  native direct engine, and parity manifest.
- Added direct lowering for `Str` literals, locals, parameters, returns,
  `s.len()`, `s[i]`, and `Str` equality/inequality, plus `Bool` helper
  signatures.
- Promoted string index, parse_uint, and identifier-scan examples in
  `tools/vaisc-parity.tsv`.
- Promoted named `parse_uint(s)` and `parse_int(s)` prelude helpers through the
  full self-host compiler, native direct engine, front gate, parity manifest,
  and value corpus.
- Regenerated `compiler/self/vaisc_core.ll` with the named parsing helpers.
- Added native direct local `Map<Int,Int>` lowering for `{}`, `insert`,
  `get(key, default)`, `contains`, and `len`, with direct gate coverage.
- Added full self-host local `Map<Int,Int>` lowering for the same surface and
  regenerated `compiler/self/vaisc_core.ll`.
- Updated front diagnostics so local `Map<Int,Int>` values are accepted while
  Map parameters, returns, assignment, and generic key/value forms stay gated.

## 2026-06-15

- Expanded native direct mode with the first local `List<Int>` slice:
  `[]`, `list()`, small integer list literals, `push`, `len`/`len()`, index,
  and `sum()`.
- Added direct-engine gate coverage for local `List<Int>` emit/run behavior.
- Expanded native direct mode with `List<Int>` function parameter and return
  ABI.
- Switched direct `List<Int>` parameters to native references for local list
  arguments and gated caller-visible callee `push` mutation.
- Added direct-engine lowering and gate coverage for inline `List<Int>` literal
  and `list()` call/return values.
- Added direct-engine temporary hoisting for `List<Int>`-returning helper calls
  passed directly to `List<Int>` parameters in statement contexts.
- Added per-iteration direct-engine hoisting for returned-list arguments inside
  `while` conditions.
- Added direct-engine local `List<Struct>` lowering for declared structs,
  including typed `[]`, `list()`, list literals, `push`, `len`, index, and
  field reads.
- Added direct-engine gate coverage for local `List<Box>` emit/run behavior.
- Expanded direct-engine `List<Struct>` support through function parameter and
  return ABI, inline call arguments, returned-list argument hoisting, and
  while-condition hoisting.
- Added direct-engine gate coverage for `List<Box>` ABI behavior returning 42.
- Added context-typed direct list assignment for `List<Int>` and `List<Struct>`,
  including list-parameter replacement and gate coverage returning 42.
- Added direct-engine indexed field assignment for `List<Struct>` locals and
  parameters, plus P4 diagnostics for unknown indexed fields.
- Added direct-engine element assignment for `List<Int>` and `List<Struct>`
  locals and parameters, including list-index element type inference.
- Added direct-engine returned-list argument lowering for `if` and `else if`
  conditions with both `List<Int>` and `List<Struct>`.
- Synced language reference, roadmap, changelog, design notes, and site copy
  with the promoted direct list slices.
- Added `scripts/test-release-gates.sh` as the pre-tag release gate covering
  native, install/package, front, direct, errors, parity, value, self-host,
  archive, website, and diff checks.
- Added `docs/release/RELEASE_CHECKLIST.md` with the `v0.2.2` next-release
  line, tag policy, manual archive workflow command, and post-tag checks.
- Prepared the `v0.2.2` source release metadata across the native compiler
  version, changelog, release checklist, roadmap, and website package.
- Ran `bash scripts/test-release-gates.sh` for `v0.2.2`; it passed and produced
  `dist/vais-0.2.2-darwin-arm64.tar.gz`.
- Pushed the annotated `v0.2.2` source tag and verified the GitHub Release
  assets for Linux x64, macOS arm64, and macOS x64.
- Verified the `Deploy Website` workflow for commit `5dfb49e3` and confirmed
  live `vaislang.dev` still exposes `scripts/vaisc` and
  `bash scripts/test-release-gates.sh`.

## 2026-06-14

- Vais-only source surface enforced.
- Public compiler input is `.vais`.
- Removed wrapper tools and non-Vais gates.
- Updated README, ROADMAP, AGENTS, language reference, examples README, prelude notes, and self-host notes to current Vais status.
- Renamed temporary test sources to `.vais`.
- Added `.vais` suffix validation to compiler and self-host helper paths.
- Added `tools/embed_self_source.vais` raw compact-program/call embedding and
  moved `scripts/test-fixpoint.sh`, `scripts/test-fixpoint2.sh`, and
  `scripts/test-fixpoint-full.sh` input generation onto the Vais helper.
- `scripts/vaisc --engine` now exposes `full` and `direct`.
- `scripts/vaisc` full mode now uses `compiler/self/vaisc_core.ll` and reads `.vais` inputs directly.
- Pure core regeneration from `compiler/self/fixpoint_full.vais` into `compiler/self/vaisc_core.ll` is green.
- Documentation is being consolidated around `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, and `compiler/self/SELF_HOST.md`.
- Official website source at `/Users/sswoo/study/projects/vais-public-claim-guard-compiler/website` was reduced to the current `.vais` language, `scripts/vaisc`, self-host status, and verification gates.
- Website `dist/` was rebuilt and checked for stale public syntax, install, and ecosystem claims.
- Official website source was copied into this repository at `website/` so future docs and site changes share one Vais baseline.
- Added `.github/workflows/deploy-website.yml` for GitHub Pages deployment from `website/dist`.
- Pushed `codex/website-docs-deploy` to `vaislang/vais`.
- Deployed the built site to `gh-pages` and switched `vaislang.dev` Pages settings to `gh-pages` with HTTPS enforced.
- Preserved the old remote `main` at `archive/old-main-2026-06-14`.
- Force-updated remote `main` to the current Vais-only history.
- Switched `vaislang.dev` from the temporary `gh-pages` deployment to the `main`
  GitHub Pages workflow.
- Added `CHANGELOG.md` for the current source release baseline.
- Added a native `vaisc` host driver and switched `scripts/vaisc` normal
  `emit-ir`, `build`, and `run` to the native public path.
- Added standalone native install, uninstall, package, and install/package gates.
- Added release archive workflow for source tags.
- Moved `--engine direct` onto the native driver and expanded it through Int
  helper calls, locals, assignment, `if`, and `while`.
- Expanded native direct mode with simple Int-field struct local literal,
  field read, and field write support.
- Expanded native direct mode with struct parameter and return helper ABI.
