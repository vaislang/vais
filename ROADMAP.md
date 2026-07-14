# Vais Roadmap

This file tracks current work and completed gate-backed language surface.

## Working Context

설계 문서: docs/design/VAIS_90_LANGUAGE_ROADMAP.md
전역 제약:
- 모든 새 언어/API 표면은 full self-host, native direct, front, parity,
  value example, reference/prelude docs, and release gates로 검증한다.
- `compiler/self/vaisc_core.ll`은 `compiler/self/fixpoint_full.vais`에서
  direct bootstrap과 canonical full self-host 경로를 거쳐 재생성한다.
- Veriqel/VaisDB 제품화 목표에 직접 필요한 문서 처리, 구조화 텍스트,
  오류 처리, snapshot/query workflow를 우선한다.
공통 검증:
- `bash scripts/test-vaisc-front.sh`
- `bash scripts/test-vaisc-direct.sh`
- `bash scripts/test-fixpoint-full.sh`
- `bash scripts/test.sh`
- `bash scripts/test-vaisc-parity.sh`
- `bash scripts/test-fixpoint-full-self.sh`
- `git diff --check`
- `bash scripts/test-release-gates.sh`

## 현재 작업 (2026-07-14b) — @(...) self-recursion 전 위치 승격
모드: 개별선택
- [x] 1. lower_self_recursion_text ✅ 2026-07-14 — 드라이버 공유 텍스트
      lowering이 `@(`를 enclosing fn 이름으로 재작성(3개 파이프라인 동일
      삽입, 문자열/주석 안전, pub fn 인식). **코퍼스 `@` 사용 0건 실측** +
      기존 동작이 full=silent 오컴파일/direct=거부였어 순수 fix.
- [x] 2. 실증 ✅ 2026-07-14 — e343(tail/컴파운드/중첩 call-인자/리스트 인자,
      양 엔진 42, parity 362) + e341 vaisgrep grep_tree·tree_matches를
      `@`로 전환(제품 코드 실사용).
- [x] 3. 문서 ✅ 2026-07-14 — LANGUAGE Self-recursion 섹션 신설, PRELUDE/
      README/CHANGELOG.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-14) — 도그푸딩 6: vaisgrep 재귀 검색 + fs_list_dirs
모드: 개별선택
- [x] 1. fs_list_dirs 승격 ✅ 2026-07-14 — fs_list_files 완전 미러 14지점
      (S_ISDIR), e342 양 엔진 42, parity 361. full core 무변경.
- [x] 2. vaisgrep `-r` ✅ 2026-07-14 — grep_tree **이름 재귀**(verified 표면)
      + 상대경로 prefix. 갭 노출→즉시 승격: direct의 helper-call 인자 속
      builtin call 이중 재작성(3번째 사례 — parse_builtin/conversion 패스에
      user-fn 그룹 opaque skip 추가).
- [x] 3. 게이트 ✅ 2026-07-14 — workflow +2케이스(재귀 6/단일 3), self-test
      tree_matches 2단계(42 유지).
- [x] 4. 환류 + 문서 ✅ 2026-07-14 — `@(...)` self-recursion이 call-인자/
      컴파운드 위치에서 양 엔진 미승격(아래 후보 등록, 이름 재귀가 verified).
      HOST_IO/PRELUDE/LANGUAGE/README/CHANGELOG.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-13) — 도그푸딩 5: vaisgrep (두 번째 배포 도구)
모드: 개별선택
- [x] 1. e341 vaisgrep 패키지 ✅ 2026-07-13 — grep.scan 모듈(match_lines_into/
      count_matches/is_text_name) + main 디스패치, 무인자 self-test 42.
- [x] 2. 디렉토리 검색 ✅ 2026-07-13 — **갭 노출→즉시 승격: fs_is_dir(path)**.
      fs_exists는 디렉토리에도 1이라 파일 분기가 host trap — stat 기반
      fs_is_dir을 host runtime+양 엔진에 승격(9지점, fs_exists 미러).
- [x] 3. count 모드 ✅ 2026-07-13 — `-c` 플래그 first-arg 디스패치, 파일별
      count 라인, exit=총 매칭.
- [x] 4. 환류 + 문서 ✅ 2026-07-13 — parity 360(e341 main), workflow 게이트
      +7케이스, HOST_IO/PRELUDE/LANGUAGE/README/CHANGELOG. 트랩 재확인:
      문자열 리터럴 `\n` 이스케이프 없음(str_byte(10) 사용).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-12d) — List<Str> 정렬 표면 승격 (도그푸딩 4 환류)
모드: 개별선택
- [x] 1. List<Str> 원소 대입 승격 ✅ 2026-07-12 — 선행 갭. full core는
      원소-store 폴백에서 ensure_i64_op로 ptr 값 변환(.ll 재생성), direct는
      원소 대입 게이트에 Str 1줄 추가.
- [x] 2. str_cmp(a,b)->Int 빌트인 ✅ 2026-07-12 — 3-way(-1/0/1). host
      runtime + HOST_INTRINSIC_IR(full 제네릭 call 경로 그대로) + direct
      10지점 배선 + front unknown-call 화이트리스트.
- [x] 3. List<Str>.sort() ✅ 2026-07-12 — 공유 sort 데수가의 비교 라인만
      str_cmp(%V,%K)>0으로 교체(2줄). 로컬/파라미터/빈 리스트 검증.
- [x] 4. 적용+문서 ✅ 2026-07-12 — vaisdb docs 사전순 출력(self-test 순서
      검증 추가), e340(parity 359), PRELUDE/LANGUAGE/README/CHANGELOG.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-12c) — 도그푸딩 4: vaisdb 문서 관리
모드: 개별선택
- [x] 1. `docs <index>` ✅ 2026-07-12 — doc_ids_into(dedupe seen map), exit=수
- [x] 2. `remove <index> <doc-id>` ✅ 2026-07-12 — remove_doc_into가 필터된
      새 Map 재구축 후 저장(키 삭제 표면 불필요), exit 0/미존재 3
- [x] 3. `stats <index>` ✅ 2026-07-12 — docs=N terms=M, exit=N
- [x] 4. 환류 + 문서 ✅ 2026-07-12 — **컴파일러 갭 0건**(첫 시도 양 엔진 42).
      key_doc_id를 report→index로 이동(순환 없이 공유). fs_mkdirs direct
      prototype const 정리(경고 0). List<Str> 정렬 표면 부재는 아래 후보 등록
      (docs 출력은 삽입 순서 — 결정적이라 게이트 가능).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-12b) — full 미지 함수 front 진단
모드: 개별선택
- [x] 1. check_front_contract_text에 unknown-call 검사 추가 ✅ 2026-07-12
  - pass1: 호출 가능 이름 수집(fn/pub fn 선언 + let/mut 바인딩 + `name:`
    파라미터·필드 — 클로저 보유 로컬 오탐 방지, 4096 cap 초과 시 검사 비활성).
  - pass2: `.` 리시버 없는 lowercase ident+`(` 중 미등록·비빌트인 →
    "call to an unknown function" front 거부. 대문자(변환/variant)와
    `_`/`vais_` 접두(lowering 생성)는 skip.
  - 병합·lowering 후 텍스트 대상이라 모듈 간 호출 안전. trust root는 기존
    처럼 skip. 사전 코퍼스 스윕으로 화이트리스트 검증(bitand/bitor/bitnot/
    putchar/puts 추가). front 게이트 reject 케이스(unknown_call) 추가.
진행률: 1/1 (100%)

## 직전 완료 (2026-07-12) — 갭 승격: List<Struct> 인덱스 필드 in 중첩 call 인자
모드: 개별선택
- [x] 1. direct 이중 재작성 근본수정 ✅ 2026-07-12
  - 원인: `direct_rewrite_list_expr`의 builtin-skip 목록에 `Str(...)` 변환
    call이 빠져 있어 변환 인자 내부가 먼저 C 형태로 재작성되고, 이후
    `direct_rewrite_str_conversion_calls`가 인자를 다시 `direct_rewrite_expr`
    로 재귀 재작성하며 `xs.data`를 List 메서드로 오인해 거부.
  - fix: skip 목록에 `direct_is_str_conversion_builtin_name` 1항 추가(1줄).
    full core 무변경. e339(양 엔진 42, parity 358) + e337 rank_lines
    워크어라운드 제거(제품 코드가 갭 승격의 실증).
진행률: 1/1 (100%)

## 직전 완료 (2026-07-10b) — 도그푸딩 3: fs_list_files + vaisdb 제품 기능
모드: 개별선택
- [x] 1. fs_list_files host API 승격 ✅ 2026-07-12
  - changes: HOST_INTRINSIC_IR declare + write_host_runtime_c 구현(opendir/
    readdir/stat, 정렬, 누락 dir=0, full 리스트 계약 buf[4095]=len) + direct
    builtin 배선(8지점)+static 헬퍼. 부수: fs_mkdirs를 direct에도 승격
    (prototype+emission, host runtime 공유). e338 양 엔진 42. **full core
    무변경** — 제네릭 call 경로가 (i8*, i64*) shape을 이미 emit.
- [x] 2. vaisdb ingest-dir ✅ 2026-07-12
  - changes: ingest_dir_into(.txt만, doc-id=확장자 제거 — flat key의 dot 충돌
    회피) + main 디스패치 + workflow 게이트(성공 0/누락 dir 3).
- [x] 3. rank top-k ✅ 2026-07-12
  - changes: rank_lines가 RankedDoc 수집→**sort_by_desc(|d| d.score)** 제품
    실사용→k라인 렌더. rank <index> <query> <k> 서브커맨드(exit=top score,
    k<1은 1). 게이트 rank=4/bad-k=1.
- [x] 4. 환류 + 문서 ✅ 2026-07-12
  - 환류 갭: direct의 local List<Struct> 인덱스 필드 중첩 call-인자 미승격
    (원소 let-바인딩으로 우회 가능, 아래 다음 후보 등록). HOST_IO/PRELUDE/
    README/CHANGELOG 반영.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-10) — VaisDB 도그푸딩 확장 2
모드: 개별선택
- [x] 1. 다중 문서 top-k 랭킹 리포트 (Opus 직접) ✅ 2026-07-10
  - changes: examples/e332_vaisdb_topk_ranking_report.vais (점수→수동 selection
    sort→top-k 렌더→blank-query Str 에러), parity 351 등록, workflow 게이트
    expect_pair, README. probe로 Int/Str 필드 원소 스왑 양 엔진 검증(컴파일러
    갭 없음). **환류 갭 1호: built-in List sort 부재**(수동 정렬은 동작하나
    ergonomics) — 작업 5에서 등록.
- [x] 2. 스냅샷 버전 헤더 + 마이그레이션 (Opus 직접) ✅ 2026-07-10
  - changes: examples/e333_vaisdb_snapshot_version_migration.vais (version=N
    헤더, v1 bare-key→v2 doc.* 마이그레이션(key_at/value_at 순회), 미지버전/
    헤더누락 Str 에러), parity 352, workflow 게이트, README. 컴파일러 갭 없음 —
    디버깅 일화: 트랩은 str_slice 시그니처 오해((start,len), invalid range trap
    은 문서화된 동작)였고 str_starts_with(기검증)로 교체.
- [x] 3. 인덱스 영속화 + 증분 ingest (Opus 직접) ✅ 2026-07-10
  - changes: examples/e334_vaisdb_index_persistence_incremental.vais (docid.term
    평탄 키 인덱스 영속화→재로드→증분 d3 추가→재영속→fresh build와 점수 동일,
    누락 인덱스 Str 에러), parity 353, workflow 게이트, README. 첫 시도 양 엔진
    42 — 컴파일러 갭 없음.
- [x] 4. Vais-authored `vaisdb` CLI 통합 도구 (Opus 직접) ✅ 2026-07-10
  - changes: tools/vaisdb_cli.vais (ingest/query/report 서브커맨드, e334 인덱스
    레이아웃, query/report는 score를 exit code로 반환, 에러 3/2/1 구분) +
    scripts/vaisdb-cli.sh 래퍼 + workflow 게이트 10케이스(direct 엔진 report +
    래퍼 포함). 첫 시도 전 케이스 정확 — 컴파일러 갭 없음.
- [ ] 5. 갭 환류 + 문서 정리 (Opus 직접)
진행률: 4/5 (80%)

배경: Result 진단 스프린트(5/5)로 오류 표현이 완성됐다. 다음 병목은 "실제 제품
워크플로가 요구하는 조합"이다 — 정렬/랭킹, 스키마 버전, 영속 인덱스, CLI 통합.
ROADMAP 전략("dogfooding으로 언어 갭 노출")대로 각 작업은 예제/도구를 먼저
Vais로 작성하고, 막히면 컴파일러를 root-cause로 고친 뒤 진행한다. 노출됐지만
이번에 안 고치는 갭은 작업 5에서 다음 task brief로 환류한다.

### 도그푸딩 2 Task Briefs

1. **top-k 랭킹**: N개 문서를 질의 점수로 정렬해 상위 k 리포트를 만든다.
   `List<Struct>` 원소 스왑 기반 수동 정렬(검증된 indexed element assignment)로
   시작 — 동작하면 built-in sort 부재를 ergonomics 갭으로 환류, 깨지면 컴파일러
   수정. 완료: 예제 direct/full/parity/value green + workflow 게이트 편입.
2. **스냅샷 버전**: `version=N` 헤더를 가진 스냅샷을 쓰고, 구버전 load 시
   마이그레이션, 미지 버전은 `Err("unknown snapshot version ...")`. 완료: 버전
   round-trip + 오류경로 예제 green.
3. **인덱스 영속화**: term index를 디스크 저장/재로드 후 문서 1개 증분 추가가
   기존 결과와 일치. 완료: round-trip 예제 green.
4. **vaisdb CLI**: `tools/`에 ingest/query/report 서브커맨드 도구 + `scripts/`
   래퍼. 완료: workflow 게이트에서 서브커맨드 실행 검증.
5. **환류**: 노출 갭을 "다음 후보 작업"에 등록, LANGUAGE/PRELUDE/README 반영.

## 직전 완료 (2026-07-06) — VaisDB Result 진단 확장
모드: 개별선택
- [x] 1. Result 오용 P4 help 진단 신설 (impl-sonnet) ✅ 2026-07-06
  - changes: tests/fixtures/vais_check/bad.vais (+Result<Unknown,Int> reject),
    tools/vais_check_contract_check.vais·vais_check_smoke.vais (count 29→30);
    Result<Int,Str>(non-Int error)와 Result<Unknown,Int>(미선언 struct) 오용이
    checker P4 help로 거부되는 것을 bad.vais fixture count 게이트로 고정.
- [x] 2. non-Int error payload 슬라이스 승격 (Opus 직접) ✅ 2026-07-06
  - direct 슬라이스 (커밋 cdcbc00d): Result<Str,Str>가 native direct 엔진에서
    동작. lower_result_str_str_text(str_int 머신 복제) + checker
    result_str_str_type_at + direct feature shape 233 + 예제 e329.
  - full self-host 슬라이스 (커밋 fee3f697): fixpoint_full.vais에
    result_str_str_ty() 태그 + 파싱 + 프레디킷 3 + emit 2(match str/int, 양 arm
    Str 포인터) + dispatch. vaisc_core.ll canonical 재생성. native front 진단 +
    게이트 문자열 정렬. e329 parity native-supported 등록.
  - **핵심 정정**: memory에 "packed scalar i64" 표현으로 기록했으나 실측 결과
    Result<Str,Int>/Result<Str,Str> 모두 struct out-param(hidden, 3-slot: tag,
    value ptr, error ptr)로 lowering된다. Str,Str은 Ok/Err 둘 다 Str 포인터라
    str_int보다 오히려 단순(Err arm Int 분기 불필요).
  - **부수 발견 → task_8ac041ef**: Result<Str,*> local을 세미콜론 단일라인
    함수에 바인딩하면 slot 미할당(%v-1) invalid IR. str_int도 동일한 기존 버그.
    작업2와 무관. full codegen check case는 이 버그 회피 위해 제외(e329가 정상
    multi-line으로 front/direct/full/parity/value 보호).
  - 검증: 전 게이트 green(self-host fixpoint stage1==stage2 bit-identical 포함).
- [x] 3. nested Result/Option 진단 명확화 (Opus 직접) ✅ 2026-07-06
  - changes: bad.vais에 nested_result(Result<Result<Int,Int>,Int>) +
    nested_option(Option<Result<Int,Int>>) reject 추가, count 30→32 (smoke/
    contract/install 3 게이트). front_check에 result_nested_not_verified reject
    케이스 신설(option_nested는 기존). Result P4 help(vais_check_core +
    vaisc_native.c)에 "nested Option/Result payloads are not verified yet" 명시.
  - nested 진단 로직은 이미 완결(unsupported_result_generic_at가 검증 4형식 외
    전부 reject)이었고, 갭은 작업1과 동일하게 "게이트 미고정". codegen 미변경.
    front/direct/checker/native smoke green.
- [x] 4. VaisDB 인덱서에 진단 경로 적용 (Opus 직접) ✅ 2026-07-06
  - changes: examples/e330_vaisdb_ingest_error_message_flow.vais 신설 — 작업2
    Result<Str,Str>를 실제 VaisDB ingest 워크플로에서 도그푸딩. 파일 ingest +
    snapshot round trip + query scoring의 모든 실패 경로를 정수 코드 대신
    사람이 읽는 Str 에러 메시지("document not found" 등)로 표현, `?` 전파 +
    inline match로 회수. e330 parity native-supported 등록(349), workflow 게이트
    expect_pair 편입, README에 e329/e330 문서화(e329는 작업2에서 누락됐던 것).
  - 검증: e330 full/direct=42, parity 349, value corpus 349/0, workflow OK.
    codegen 미변경.
- [x] 5. 문서/게이트 정리 (Opus 직접) ✅ 2026-07-06
  - changes: docs/reference/LANGUAGE.md에 Result<Str,Str> 검증표 행 추가 +
    "Rejected Option/Result shapes" 섹션 신설(non-Int/Str error, 미선언 struct,
    nested 조합 명시 + 게이트 포인터). std/PRELUDE.md에 e329/e330 기술.
    CHANGELOG.md Unreleased에 Result<Str,Str> 승격 + 진단 강화 2항목.
  - 검증: 문서 기술한 reject 4종(Int,Str / Unknown,Int / nested Result / nested
    Option) 실측 전부 거부, accept(Result<Str,Str>) e329/e330=42 확인. diff clean.
진행률: 5/5 (100%) — VaisDB Result 진단 확장 스프린트 완료

배경: Result 값-흐름 표면은 e321에서 포화(payload Int→Str→Struct, match 필드
회수→조합→Bool 반환 완성). 반면 진단은 얇다 — 현재는 자동 wrapper 생성 위주이고
잘못 쓴 Result 코드에 대한 명시적 P4 help가 거의 없다(`vaisc_errors_check.vais`에
Result 항목 0). 이번 스프린트는 "오용이 명확한 help로 거부되는가"를 채우고,
product workflow가 실제로 노출한 non-Int error / nested payload 인접 확장을 더한다.
generic `Result<T,E>`는 여전히 열지 않는다.

## 다음 후보 작업 (이번 스프린트 이후)

- 이번 스프린트가 노출하는 concrete non-Int/nested 사례가 반복되면 generic
  `Result<T,E>` 일반화를 값-정확성 fuzzing 기반과 함께 검토한다.
- richer reusable package layout / package diagnostics: e337(vaisdb 설치형
  패키지, 다중 모듈 src/vaisdb/* + binary + archive)이 현 표면을 실제 도구로
  도그푸딩 완료 — 노출 갭 0건. 추가 layout 요구(중첩 모듈 트리, 의존 패키지
  결합 등)가 제품에서 나오면 재개.
- ~~built-in List sort~~ (환류 갭 1호, 완결): `List<Int>.sort()`(e335) +
  `List<Struct>.sort_by/sort_by_desc(|x| x.int_field)`(e336) 모두 2026-07-10
  승격 완료 — driver 단일 desugar로 양 엔진 공유. Str-key sort_by는 필요
  노출 시 후속.

## Task Briefs

### 1. Result 오용 P4 help 진단 신설
대상 파일: tools/vaisc_errors_check.vais, tools/vaisc_front_check.vais, tools/vais_check_core.vais, tests/fixtures/vais_check/, docs/reference/LANGUAGE.md
요구사항: 잘못 쓴 concrete Result 코드가 wrapper 자동생성으로 조용히 통과하거나 모호한 codegen 에러로 실패하는 대신, 명확한 P4 help로 거부되게 한다. 최소 케이스: (a) 검증되지 않은 payload/error 타입 조합(예: `Result<Int,Str>` 미검증 error 타입), (b) 파일에 선언되지 않은 struct payload(`Result<Unknown,Int>`), (c) Ok/Err arm 누락 또는 arm 타입 불일치.
인터페이스: 기존 `vais_check_core.vais` 755/759줄의 Result 에러 메시지 스타일(P4 help)을 재사용해 reject 케이스를 늘린다.
제약사항: 현재 통과하는 e294~e321 예제는 하나도 회귀시키지 않는다. codegen 변경 없이 checker/front 진단만 확장한다.
완료조건: `vaisc_errors_check.vais`에 Result reject fixture가 추가되고, front/checker 게이트가 각 오용에 대해 P4 help 형태로 거부하며, 기존 게이트 전부 green.

### 2. non-Int error payload 슬라이스 승격
참조: tools/vais_check_core.vais(759줄 "non-Int error payloads are not verified yet")
대상 파일: compiler/self/fixpoint_full.vais, compiler/self/vaisc_core.ll, tools/vaisc_native.c, tools/vais_check_core.vais, tools/vaisc_front_check.vais, tools/vaisc_direct_feature_check.vais, tools/fixpoint_full_codegen_check.vais, tools/vaisc-parity.tsv, examples/, std/PRELUDE.md, docs/reference/LANGUAGE.md
요구사항: product workflow가 실제로 노출하는 non-Int error payload 한 슬라이스를 concrete하게 승격한다. 후보: `Result<Str,Str>`(문자열 에러 메시지) — helper return/param/forward/inline match/`?` 전파를 e321까지의 패턴과 동일 깊이로 고정.
인터페이스: 기존 concrete 3형식(`Result<Int,Int>`/`Result<Str,Int>`/`Result<Struct,Int>`)을 깨지 않고 error 타입만 확장한다.
제약사항: generic `Result<T,E>`는 열지 않는다. 한 번에 한 error 타입 슬라이스만.
완료조건: 새 예제가 direct/full/parity/value/release 게이트에서 기대값으로 실행되고 full codegen case로 보호된다.

조사 완료 (2026-07-06, 다음 세션 진입점) — 이 작업은 dedicated 세션 필요:
- **표현 스킴**: full self-host는 `Result<Str,Int>`를 packed scalar i64로 인코딩
  (fixpoint_full.vais:3400~3496 생성자). 값을 `mul 2`로 시프트해 LSB를 태그로
  사용: `Ok(v)`=`v*2`, `Err(code)`=`code*2+1`. Str payload는 heap malloc 후
  포인터를 값으로 pack(3417~3470). native.c는 별도로 struct 기반
  `VaisResultStrInt { tag, value: Str, error: Int }`(2532줄) — 두 백엔드 표현이
  다르므로 각각 확장 필요.
- **핵심 통찰**: Err payload도 heap 포인터(malloc 8-byte 정렬→LSB=0)로 처리하면
  `Result<Str,Str>`가 기존 packed 스킴에 대칭적으로 들어간다. 표현을 새로
  설계할 필요 없이 "기존 Str-payload 경로를 Err 쪽에도 적용"이 정답.
- **첫 관문**: 현재 `Result<Str,Str>`는 checker/front 타입 진단(작업 1에서 강화)이
  codegen 도달 전에 거부한다. 순서 = ①타입 인식 열기(checker
  `result_str_str_type_at`, full/native 타입 태그 `result_str_str_ty()` 신규,
  native `VaisResultStrStr` 구조체) → ②codegen이 실측으로 깨지는 지점을 packed-i64
  match unpack(9594~9704 `result_match_*`, 토큰 오프셋 하드코딩)까지 확장.
- **손댈 곳**: full self-host 123곳(result_str 관련), native.c의 `result_str_int_*`
  함수군 전체(1617~2643 등), checker 235~238줄 accept 목록, 진단 help 17273/17286줄.
- **Steps 체크포인트 권장**: (1)타입 인식+진단 (2)Ok/Err 생성자 (3)inline match
  unpack (4)`?` 전파 (5)param forward. 각 단계 후 full/direct 게이트로 회귀 확인.
- **.ll 주의**: fixpoint_full.vais 수정 후 vaisc_core.ll은 canonical 재생성만
  (임시경로 유입 금지, 2026-07-06 위생 이슈 참조).

### 3. nested Result/Option 진단 명확화
참조: tools/vais_check_core.vais(755줄 nested Option/Result 미검증)
대상 파일: tools/vais_check_core.vais, tools/vaisc_front_check.vais, tools/vaisc_errors_check.vais, docs/reference/LANGUAGE.md
요구사항: `Result<Result<...>,Int>`, `Option<Result<...>>` 같은 nested 조합이 조용히 오작동하지 않고 "not verified yet" P4 help로 명확히 거부되게 한다.
인터페이스: 작업 1의 reject 인프라를 재사용한다.
제약사항: nested를 실제로 구현하지 않는다 — 진단 명확화만. codegen 변경 없음.
완료조건: nested reject fixture가 게이트에서 P4 help로 거부되고 기존 게이트 green.

### 4. VaisDB 인덱서에 진단 경로 적용 (도그푸딩)
참조: examples/e295_vaisdb_indexer_prototype.vais, e297/e298 file ingest
대상 파일: examples/e29x_vaisdb_*.vais 또는 신규 예제, tools/vaisc-parity.tsv, scripts/test-vaisdb-workflow.sh
요구사항: VaisDB 인덱서/ingest 워크플로가 작업 1~3의 명확한 오류 경로를 실제로 사용하는 도그푸딩 예제를 만든다(예: 손상된 스냅샷/누락 필드를 non-Int error 또는 명시적 Err로 표현). 작업 중 노출된 언어 갭은 새 roadmap task로 환류.
인터페이스: example `main() -> Int` 또는 기존 워크플로 확장.
제약사항: 제품 DB 엔진이 아니라 dogfooding prototype.
완료조건: 예제가 direct/full/parity/value에서 실행되고 workflow 게이트에 편입된다.

### 5. 문서/게이트 정리
대상 파일: docs/reference/LANGUAGE.md, std/PRELUDE.md, examples/README.md, CHANGELOG.md, docs/design/VAISDB_DX_BASELINE.md, WORKLOG.md
요구사항: 이번 스프린트로 확장된 Result 진단/타입 표면을 reference/prelude/example 문서에 반영하고, 다음 contributor가 "어떤 Result 형식이 검증됐고 무엇이 왜 거부되는가"를 문서만 보고 알 수 있게 한다.
인터페이스: 문서 + scripts/test-* 게이트.
제약사항: release gate가 장시간이어도 green 유지가 우선.
완료조건: docs와 gates만 보고 검증된 Result 표면과 거부 규칙을 재현할 수 있다.

## Done

- Single-line semicolon-joined fn bodies now bind/match `Result<Str,*>` locals
  and propagate with `?` correctly: the native driver's `split_fn_body_line`
  pre-pass breaks one-line fn bodies into per-statement lines before the
  line-anchored Result lowerings run (previously emitted undefined `%v-1`
  loads). Pinned by `examples/e331_semicolon_single_line_result.vais` in
  parity/value gates. The raw self-host core path (used by the codegen check
  harness, which bypasses the driver lowerings) also handles `Result<Str,Str>`
  `?` bindings and Str-result matches now: the core's question predicate
  accepts str_str callees and the str_str match-result slot predicate is wired
  into both slot collectors. Pinned by
  `case_080m23_result_str_str_error_message` in the full codegen gate.
- Project path is `/Users/sswoo/study/projects/vais`.
- Native `vaisc` temporary intermediates are isolated under a per-run temp root,
  cleaned on normal exit, and protected by a native smoke regression check;
  `--keep-tmp` remains available for debug artifact preservation.
- Checked-in language sources use `.vais`.
- `scripts/vaisc` is the canonical compiler command.
- `scripts/vais-check` is the canonical lint/error-help command, built from
  Vais source and protected by fixture contract gates.
- The workspace now exposes only Vais source and Vais commands.
- The compiler gates cover CLI smoke, front-contract diagnostics, direct LLVM emission, parity, and the value corpus.
- The trusted self-host tier is `compiler/self/fixpoint.vais`, `fixpoint2.vais`, `fixpoint3.vais`, and `fixpoint_full.vais`.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by `scripts/vaisc`.
- The full compiler path reads `.vais` source files directly through the self-host core.
- Pure regeneration of `compiler/self/vaisc_core.ll` from `compiler/self/fixpoint_full.vais` is green.
- `str_replace(text, needle, replacement)` is verified in full/direct paths for
  all-occurrence string rewriting over literals, normalized `Map<Str,Str>`
  reads, `List<Str>` reads, and `Map<Str,Str>.get_opt` match values.
- `str_split_into(text, sep, out)` is verified in full/direct paths for
  delimiter-based tokenization into `List<Str>` out-params, including
  empty-field preservation and empty-separator whole-text behavior.
- `str_split_lines_into(text, out)` is verified in full/direct paths for
  LF/CRLF document line tokenization into `List<Str>` out-params, including
  interior blank lines, empty input, and trailing-line-break handling.
- `map_str_str_snapshot(docs)` and `map_str_str_load_snapshot(text, out)` are
  verified in full/direct paths for `Map<Str,Str>` line metadata snapshot round
  trips, including output map clearing, LF/CRLF loading, malformed-line
  skipping, empty values, and additional `=` preservation.
- Concrete `Option<Int>`/`Result<Int,Int>` value lowering is verified for
  helper return/parameter/local types, constructors, inline match, and
  local-binding `?` with `examples/e294_result_try_parse_error_flow.vais`.
  `examples/e296_result_map_param_flow.vais` extends the Result slice to
  helpers over `Map<Str,Str>` parameters with `get_opt` matches and `?`
  propagation; full self-host codegen protects the two Result surfaces through
  `case_080g6_result_encoding_parse_error_flow` and
  `case_080g7_result_map_param_flow`. Generic `Option<T>`/`Result<T,E>` remain
  intentionally closed.
- `examples/e295_vaisdb_indexer_prototype.vais` is verified as the first
  Vais-authored document indexer dogfooding prototype, combining metadata
  ingest, `Map<Str,Str>` snapshot round trip, `Map<Str,Int>` term counts, and
  weighted query scoring in direct/default/parity paths; full self-host
  codegen protects the same workflow through
  `case_080k_vaisdb_indexer_prototype`.
- `examples/e297_vaisdb_file_ingest_workflow.vais` extends the VaisDB
  dogfooding path to file-backed ingest: it reads document/query files,
  creates deterministic temp fixtures with `fs_temp_dir`, `path_join`, and
  `fs_write_text`, accepts argv-supplied paths with `proc_argc`/`proc_arg`,
  splits lines, snapshots metadata, indexes term counts, and scores a query in
  direct/default/parity paths. `scripts/test-vaisdb-workflow.sh` checks both
  generated-file and argv-file modes, and full codegen protects the standalone
  generated-IR shape through `case_080l_vaisdb_file_ingest_workflow`.
- `examples/e298_vaisdb_file_ingest_result_flow.vais` adds the first
  file-backed `Result<Int,Int>` ingest recipe: helpers guard raw
  `fs_read_text` calls with `fs_exists`, return explicit integer error codes
  for missing or malformed document/query paths, compose the helpers with
  local-binding `?`, and run in generated-file, argv-file, and missing-file
  modes through the focused VaisDB workflow gate. The native direct feature
  gate now covers `fs_exists`, and full codegen protects the standalone shape
  through `case_080m_file_exists_result_flow`.
- `examples/e301_result_str_int_file_read.vais` adds the first
  file-backed `Result<Str,Int>` payload recipe: guarded helpers return
  `Ok(text)` or `Err(code)`, compose with local-binding `?`, and recover both
  string payloads and missing-file integer error codes through inline match in
  direct/default/parity paths. `scripts/test-vaisdb-workflow.sh` includes the
  focused direct/default workflow check, the public `vais-check` contract
  accepts the concrete shape, and full codegen protects it through
  `case_080m2_result_str_int_file_read`.
- `examples/e302_result_str_int_param_flow.vais` extends that concrete
  string-payload Result slice to helper parameters and forwarding: a
  `Result<Str,Int>` local can be passed into helper functions, forwarded to
  another helper, and matched there to recover `Str` payloads or `Int` error
  values. The native source-lowering path now tracks `Result<Str,Int>`
  parameters, full self-host codegen parses `Result<Str,Int>` parameter slots,
  and full codegen protects the standalone shape through
  `case_080m3_result_str_int_param_flow`.
- `examples/e303_result_metric_int_struct_payload.vais` opens the first
  structured Result payload slice: a concrete `Result<Metric,Int>` helper can
  return `Ok(Metric)` or `Err(Int)`, pass/forward that value through helper
  parameters, and recover `Metric` fields or integer error values through
  inline matches. Full self-host codegen protects the standalone shape through
  `case_080m4_result_metric_int_struct_payload`.
- `examples/e304_result_record_int_struct_payload.vais` broadens that path
  beyond the previous `Metric`-only slice: declared Int-field struct payloads
  such as `Record` can flow as `Result<DeclaredStruct,Int>` through helper
  returns, parameters, forwarding helpers, and inline matches with three-field
  recovery. Native source lowering now derives wrappers from struct
  declarations, and full self-host codegen protects n-field structured Result
  matches through `case_080m5_result_record_int_struct_payload`.
- `examples/e305_result_multiline_struct_payload.vais` removes the one-line
  declaration limitation from that path: multiline declared Int-field struct
  payloads such as `Entry` can flow through `Result<DeclaredStruct,Int>`
  helpers and recover four fields through inline matches. Native source
  lowering now inserts derived Result wrappers after the closing struct line,
  and full self-host codegen protects the multiline source shape through
  `case_080m6_result_multiline_struct_payload`.
- `examples/e306_result_struct_str_fields.vais` expands declared structured
  Result payloads to document-like records with `Str` fields: `DocSummary`
  carries title/summary text plus an Int score through
  `Result<DeclaredStruct,Int>` helper returns and parameters, and inline
  matches recover string field lengths mixed with Int fields. Full self-host
  codegen protects this through `case_080m7_result_struct_str_fields`.
- `examples/e307_result_struct_try_payload.vais` completes the next ergonomics
  step for declared structured Result payloads: `DocSummary` can be extracted
  from `Result<DocSummary,Int>` with local-binding `?`, reused through its
  `Str` and `Int` fields, and propagated as an early integer error in
  direct/default/parity paths. Full self-host codegen protects this through
  `case_080m8_result_struct_try_payload`.
- `examples/e308_vaisdb_artifact_record_workflow.vais` promotes that surface
  into a VaisDB-style artifact/document record workflow: `DocArtifact` payloads
  are built through `Result<DocArtifact,Int>` helpers, extracted with
  local-binding `?`, stored through `List<DocArtifact>` output parameters,
  paired with `Map<Str,Str>` metadata snapshots, and checked in
  direct/default/parity paths. Full self-host codegen protects this through
  `case_080m9_vaisdb_artifact_record_workflow`.
- `examples/e309_vaisdb_artifact_store_snapshot.vais` persists that record
  surface as a small text artifact store: `List<DocArtifact>` values are
  serialized to a tab-delimited snapshot, written/read through host file
  helpers, parsed back through `Result<DocArtifact,Int>` helpers, queried for
  the best loaded record, and checked in direct/default/parity paths. Full
  self-host codegen protects this through
  `case_080m10_vaisdb_artifact_store_snapshot`.
- `examples/e310_vaisdb_artifact_query_report.vais` adds a reusable persisted
  artifact-store query/report layer: the store is loaded into
  `List<DocArtifact>`, ranked through `Map<Str,Int>` term scoring, rendered as
  a `Result<Str,Int>` report payload, persisted again with file helpers, and
  checked for missing-store and empty-query error codes. Full self-host codegen
  protects this through `case_080m11_vaisdb_artifact_query_report`.
- `examples/e311_result_call_argument_flow.vais` closes the next Result
  call-site ergonomics gap: `Result<Str,Int>` and
  `Result<DeclaredStruct,Int>` returning helpers can feed other helper calls
  directly without manual local binding. Full self-host codegen protects the
  hidden-out struct-returning call-argument path through
  `case_080m12_result_call_argument_flow`.
- `examples/e312_result_struct_local_wrapper_flow.vais` closes the next
  self-host structured-payload copy gap: explicit `VaisResult<Struct>Int`
  wrapper code can bind `flow.value` to a local struct, read all payload fields,
  and return that local in another wrapper literal without losing nested fields.
  Full self-host codegen protects this through
  `case_080m13_result_struct_local_wrapper_flow`.
- `examples/e313_result_struct_str_match_flow.vais` closes the next
  report-building gap for structured Results: `Result<DeclaredStruct,Int>`
  matches can recover `Str` fields such as `artifact.title` directly into
  string locals while `Err(Int)` arms convert codes with `Str(code)`. Full
  self-host codegen protects this through
  `case_080m14_result_struct_str_match_flow`.
- `examples/e314_result_struct_str_concat_match_flow.vais` closes the follow-on
  report-label gap for structured Results: `Result<DeclaredStruct,Int>` matches
  can compose `Str` payload fields with nested `str_concat(...)` inside `Ok`
  arms while `Err(Int)` arms convert codes with `Str(code)`. Full self-host
  codegen protects this through
  `case_080m15_result_struct_str_concat_match_flow`.
- `examples/e315_result_struct_str_transform_match_flow.vais` closes the next
  normalization gap for structured Results: `Result<DeclaredStruct,Int>` matches
  can apply `str_replace`, `str_trim`, `str_upper`, `str_lower`, and local-prefix
  `str_concat(...)` transforms to `Str` payload fields inside `Ok` arms while
  `Err(Int)` arms convert codes with `Str(code)`. Full self-host codegen protects
  this through `case_080m16_result_struct_str_transform_match_flow`.
- `examples/e316_result_struct_str_transform_len_match_flow.vais` closes the
  follow-on scoring gap for structured Results: `Result<DeclaredStruct,Int>`
  matches can compute `Int` scores from transformed `Str` payload fields with
  chained `.len()` calls while still mixing normal integer payload fields and
  preserving `Err(Int)` recovery. Full self-host codegen protects this through
  `case_080m17_result_struct_str_transform_len_match_flow`.
- `examples/e317_result_struct_payload_helper_call_score.vais` closes the
  reusable scoring-helper gap for structured Results: `Result<DeclaredStruct,Int>`
  matches can pass the `Ok` payload struct directly to an `Int` helper such as
  `score_artifact(artifact)` while preserving `Err(Int)` recovery. Full
  self-host codegen protects this through
  `case_080m18_result_struct_payload_helper_call_score`.
- `examples/e318_result_struct_payload_helper_call_arithmetic.vais` closes the
  helper-composition follow-up for structured Results: `Result<DeclaredStruct,Int>`
  matches can use an `Ok` payload helper call as one `Int` term and add normal
  payload fields such as `artifact.terms + artifact.score` while preserving
  `Err(Int)` recovery. Full self-host codegen protects this through
  `case_080m19_result_struct_payload_helper_call_arithmetic`.
- `examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` closes
  the field-helper composition follow-up for structured Results:
  `Result<DeclaredStruct,Int>` matches can pass `Ok` payload `Str` fields such
  as `artifact.title` and `artifact.body` to reusable `Int` helpers, then add
  normal payload fields while preserving `Err(Int)` recovery. Full self-host
  codegen protects this through
  `case_080m20_result_struct_payload_field_helper_call_arithmetic`.
- `examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`
  closes the numeric field-helper follow-up for structured Results:
  `Result<DeclaredStruct,Int>` matches can pass `Ok` payload `Int` fields such
  as `artifact.terms` and `artifact.score` to reusable `Int` helpers, then
  compose those helper-call terms with string-field helper terms while
  preserving `Err(Int)` recovery. Full self-host codegen protects this through
  `case_080m21_result_struct_payload_int_field_helper_call_arithmetic`.
- `examples/e321_result_struct_payload_bool_match_condition.vais` closes the
  Bool-return follow-up for structured Results: `Result<DeclaredStruct,Int>`
  matches can return conditions derived from `Ok` payload helper terms and
  `Err(Int)` code comparisons, which makes reusable validation/filter helpers
  natural to write. Full self-host codegen protects this through
  `case_080m22_result_struct_payload_bool_match_condition`.
- `examples/e322_vaisdb_module_boundary/main.vais` closes the first reusable
  VaisDB library-boundary gap: imported modules can share `DocArtifact`
  structs, `Result<DocArtifact,Int>` helpers, `List<DocArtifact>` outputs, and
  `Map<Str,Int>` scoring helpers across files, and the native direct engine now
  resolves the same static dotted local imports as the full engine before
  lowering.
- `examples/e323_cli_package` closes the first package-directory CLI gap:
  `scripts/vaisc emit-ir`, `build`, and `run` can accept a manifest-backed
  package directory, resolve `source/main.vais`, preserve imports, and forward
  argv to the compiled program in direct/default runs.
- `scripts/vaisc package <package-dir> -o <dist-dir>` closes the first
  installable package output gap: it builds `<dist-dir>/bin/<package-name>`,
  copies `<dist-dir>/vais.toml`, and is verified in direct/default package
  workflow gates.
- Packaged `examples/e323_cli_package` binaries are now verified with real CLI
  argv forwarding in native/direct/workflow gates, and `vaisc package` rejects
  unsafe manifest names before they can become output paths.
- `examples/e326_cli_binary_target` verifies optional `binary = "veriqel-demo"`
  manifest metadata so package identity and output command name can diverge
  while direct/default package workflows and file-entry parity stay aligned.
- `scripts/vaisc package examples/e326_cli_binary_target -o <dist-dir>
  --archive` verifies user-package release archive output: it writes
  `<dist-dir>/veriqel-demo-0.1.0.tar.gz`, extracts to
  `veriqel-demo-0.1.0/bin/veriqel-demo`, preserves the copied manifest, and
  rejects unsafe manifest versions before they become archive filenames.
- `time_millis() -> Int` is verified as the first elapsed-time helper for
  Vais-authored developer tools. `examples/e299_vaisdb_benchmark_report.vais`
  times document term counting/scoring, writes a benchmark report through
  `fs_write_text`, reads it back with `fs_read_text`, and runs in
  direct/default/parity paths; full codegen protects the standalone shape
  through `case_080n_time_millis_benchmark_report`.
- `examples/e300_vaisdb_benchmark_cli_report.vais` is verified as the first
  CLI-style Vais-authored benchmark/report workflow: it discovers the repo root
  with `fs_cwd`, `path_dirname`, and `path_basename`, invokes the e295 indexer
  through `proc_capture`, records direct/default elapsed milliseconds, writes a
  combined report, and runs in direct/default/parity paths. The native direct
  feature gate now covers those path helpers and full codegen protects the
  standalone shape through `case_080o_vaisdb_benchmark_cli_report`.
- `tools/vaisdb_benchmark_report.vais` is verified as the first reusable
  Vais-authored benchmark report command. It runs the e295 indexer, writes a
  raw direct/default report, parses metric lines with line splitting, prefix
  checks, slicing, and `parse_int`, computes a timing delta, writes a summary,
  and is covered by workflow/front/direct/full/parity/value gates. The shell
  wrapper is `scripts/vaisdb-benchmark-report.sh`, and full codegen protects
  the summary parsing shape through `case_080p_vaisdb_benchmark_summary_tool`.
- `docs/design/VAISDB_DX_BASELINE.md`, `scripts/test-vaisdb-workflow.sh`, and
  `scripts/bench-vaisdb-indexer.sh` now define the focused document/VaisDB
  developer workflow: e292-e324 direct/default reproducibility plus the
  reusable benchmark report tool, diagnostic commands, formatter direction, and
  a local compile+run performance baseline protocol. The focused workflow gate is included in
  `scripts/test-release-gates.sh`.
- `str_concat(left, right)` and `str_byte(value)` now lower through
  self-contained full self-host helpers, so generated standalone IR no longer
  depends on external host string-construction calls for those helpers.
- The self-host `List<Token>` retarget capacity is raised to 262144 so the
  enlarged `fixpoint_full.vais` continues to pass the full stage1/stage2
  self-host gate.
- `List<Struct>` storage now supports verified multi-field nested struct
  elements for push, whole-element copy/assignment, indexed nested reads/writes,
  parameter mutation, and non-mutating method-result nested field-chain reads in
  full/direct.
- Structs now support verified `Str` fields for document-like records in
  full/direct, including equality, string helper calls, `.len()` chains, and
  `List<Struct>` index/first/last/for-each reads plus indexed `Str` field
  reassignment on local and parameter lists and `pop`/`remove_at` method-result
  `Str` field reads.
- `proc_capture(argv: List<Str>) -> ProcessResult` is verified for the standard
  `ProcessResult { code: Int, stdout: Str, stderr: Str }` shape in full and
  direct gates, completing the first in-memory process capture slice for
  Vais-authored tools.
- Non-capturing `List<Int>.filter(|x| predicate)` now produces a reusable
  `List<Int>` result in full and direct gates, extending the previous
  filter-sum-only slice.
- Non-capturing `List<Str>.map` and annotated `List<Str>.filter` now produce
  reusable `List<Str>` results in full and direct gates for verified string
  builtin transform/predicate bodies.
- `List<Str>.filter` result type inference now uses the known receiver type for
  unannotated locals such as `let selected = words.filter(...)`.
- `List<Str>` function parameters now feed map/filter result type inference,
  including `words.map(|w| w)` followed by `filter(...)` inside helper code.
- `str_concat(left, right)` is now available in the direct string helper path
  and in verified non-capturing `List<Str>.map` closure bodies.
- `List<Str>.filter/map` closures can capture known `Str` parameters and locals
  in the verified source-prep lowering path.
- `List<Str>.filter(...).map(...)` can produce direct result lists for locals,
  helper returns, helper-call arguments including conditions, `extend(...)`
  sources, and reassignments without a user-written filtered-list temporary.
- `List<Str>.map(...).filter(...)` can produce direct result lists for locals,
  helper returns, helper-call arguments including conditions, `extend(...)`
  sources, and reassignments without a user-written mapped-list temporary.
- `List<Str>.map(...).filter(...).len/contains/index_of/count` can feed direct
  scalar contexts including locals, helper returns, helper-call arguments,
  `List<Int>` mutation arguments, reassignments, and conditions without a
  user-written mapped-list temporary.
- `List<Str>.filter(...).map(...).len/contains/index_of/count` can feed direct
  scalar contexts including locals, helper returns, helper-call arguments,
  `List<Int>` mutation arguments, reassignments, and conditions without a
  user-written filtered-list temporary.
- Multiple same-family `List<Str>.map(...).filter(...).len/contains/index_of/count`
  or `List<Str>.filter(...).map(...).len/contains/index_of/count` scalar calls
  can appear in one arithmetic or condition expression.
- `List<Str>.map(...).filter(...).len/contains/index_of/count` and
  `List<Str>.filter(...).map(...).len/contains/index_of/count` scalar calls can
  also mix inside one arithmetic or condition expression.
- Composite Bool locals built from `List<Str>` pipeline scalar conditions infer
  `Bool`, so exact pipeline scalar Bool reassignments remain verified.
- Existing `Int` locals can be updated with arithmetic-tail `List<Str>`
  pipeline scalar expressions, keeping accumulator-style code direct.
- Negated `List<Str>` pipeline scalar Bool expressions are verified for locals,
  reassignments, `if` conditions, and `while` conditions.
- Bool `if ... then ... else ...` expressions built from `List<Str>` pipeline
  scalar conditions are verified in locals, reassignments, helper-call
  arguments, and Bool returns.
- Nested helper-call arguments inside reassignment expressions can use Bool
  if-expressions built from `List<Str>` pipeline scalar conditions.
- Int `if ... then ... else ...` expressions built from `List<Str>` pipeline
  scalar conditions are verified in locals, reassignments, helper-call
  arguments, and returns.
- Scalar `if ... then ... else ...` value expressions are verified in locals,
  reassignments, helper-call arguments, and returns without requiring a
  pipeline-specific lowering trigger.
- Scalar Bool `if ... then ... else ...` value expressions are independently
  verified in locals, reassignments, helper-call arguments, and returns without
  requiring a pipeline-specific lowering trigger.
- Scalar Str `if ... then ... else ...` value expressions are independently
  verified in locals, reassignments, helper-call arguments, and Str returns
  without requiring a pipeline-specific lowering trigger.
- Scalar Char `if ... then ... else ...` value expressions are independently
  verified in locals, reassignments, helper-call arguments, and Char returns
  without requiring a pipeline-specific lowering trigger.
- `Map<Str,Str>.get_opt` string payload match expressions are verified in
  returns, reassignments, helper-call arguments, and embedded Int returns.
- `Map<Str,Str>` return-inferred locals can feed those `get_opt` string payload
  match expression contexts without requiring explicit local map annotations.
- `Map<Str,Str>.get_opt` string payload match expressions can normalize or
  compose payload strings through `str_concat`, `str_trim`, and `str_lower`.
- `Str.len()` on locals reassigned from dynamic string values now reads the
  current runtime pointer, including values from `Map<Str,Str>.get_opt`
  match-transform expressions.
- `Map<Str,Str>.get_opt` match arms can compute direct `.len()` after
  `str_trim`/`str_lower` transforms in full/direct paths.
- `Map<Str,Str>.get_opt` string payload matches lower through presence checks
  and value loads instead of pointer-tagged string payload integers, so saved
  `Str` payload locals remain stable across later embedded match/string helper
  expressions in full/direct paths; full self-host statement parsing also skips
  match-arm braces while locating `if`/`while` bodies for those embedded
  conditions.
- `Map<Str,Str>.get_opt` string payload match expressions are verified in
  `while` and `else if` condition chains, preserving per-iteration loop
  reevaluation and else-chain structure.
- `str_upper(text)` is verified in full/direct paths for ASCII lowercase to
  uppercase normalization over literals, trimmed document fields,
  `Map<Str,Str>` reads, `List<Str>` reads, and `Map<Str,Str>.get_opt` match
  payload transforms; native front keyword diagnostics now token-boundary check
  `match`/`enum`.
- `str_ends_with(text, suffix)` is verified in full/direct paths for suffix
  checks over literals, normalized strings, `Map<Str,Str>` reads,
  `List<Str>` reads, and `Map<Str,Str>.get_opt` match values.
- `List<Int>.filter/map/filter-sum` closures can capture known `Int`
  parameters and locals in the verified source-prep lowering path.
- `List<Int>.filter(...).sum()` can be assigned to typed or inferred `Int`
  locals and reused in follow-on calculations.
- `List<Int>/List<Str>.filter(...).len()` can be returned directly or assigned
  to typed/inferred `Int` locals for reusable filtered counts.
- `List<Int>.filter(...).max()` and `.min()` can be returned directly or
  assigned to typed/inferred `Int` locals for filtered ranking/selection
  without materializing an intermediate list.
- `List<Int>.map(...).sum()/max()/min()` can aggregate or rank transformed
  scalar scores directly in returns, typed/inferred `Int` locals, helper-call
  arguments, direct `List<Int>` mutation arguments, reassignments, broader
  `Int` expressions, and broader `if`/`while`/`else if` condition expressions.
- `List<Int>.filter(...).map(...).max()` and `.min()` can rank transformed
  scalar scores directly in returns and typed/inferred `Int` locals without
  materializing an intermediate list, including broader `Int` expressions used
  by locals, helper-call arguments, direct `List<Int>` mutation arguments,
  reassignments, returns, and broader `if`/`while`/`else if` condition
  expressions.
- `List<Int>.filter(...).map(...).sum()` can aggregate transformed scalar
  scores directly in returns and typed/inferred `Int` locals without
  materializing an intermediate list, including broader `Int` expressions used
  by locals, helper-call arguments, direct `List<Int>` mutation arguments,
  reassignments, returns, and broader `if`/`while`/`else if` condition
  expressions.
- `List<Struct>.filter(...).first().field` and `.last().field` can select
  document-like `Int`/`Str` record fields directly in returns and typed locals
  without materializing an intermediate record list.
- `List<Struct>.filter(...).first().str_field.len()` and
  `.last().str_field.len()` can read matched document-like string field
  lengths directly in `Int` returns and typed locals without materializing an
  intermediate record list.
- `List<Struct>.filter(...).first()` and `.last()` can select matched
  document-like records directly in same-struct returns and typed/inferred
  locals without materializing an intermediate record list, including when the
  record type is declared with multiline struct syntax.
- `List<Struct>.filter(...).first()` and `.last()` whole-record selections can
  feed same-struct `push` and `insert_at` calls directly, so matched records can
  be accumulated without a user-written temporary local.
- `List<Struct>.filter(...).first().field`/`.last().field` and string-field
  `.len()` selections can feed scalar `List<Int>`/`List<Str>` `push` and
  `insert_at` calls directly for score/title/lens accumulation patterns.
- `List<Struct>.filter(...).first().field`/`.last().field` and string-field
  `.len()` selections can infer `Int`/`Str` local types from declared record
  field metadata, so document-like field picks no longer require explicit
  local annotations in verified slices.
- `List<Struct>.filter(...).first().field`/`.last().field` and string-field
  `.len()` selections can feed `Int`/`Str` helper-call arguments directly,
  lowering each selected field into a guarded temporary before the call.
- `List<Struct>.filter(...).first()` and `.last()` whole-record selections can
  feed same-struct helper-call arguments directly, lowering each matched record
  into a guarded temporary before the call.
- `List<Struct>.filter(...)` now produces reusable declared-record result
  lists that can be returned from helpers for document-like predicates.
- `List<Struct>.map(...)` can project declared-record fields into reusable,
  directly returned, helper-call, helper-call condition, `extend(...)`, or
  reassigned `List<Int>` and `List<Str>` scalar lists for ranking/reporting;
  `Int` field projections can also aggregate directly through `sum()`,
  `max()`, and `min()` in returns, typed/inferred locals, helper-call
  arguments including simple arithmetic suffixes, standalone simple arithmetic
  suffixes, direct `List<Int>` mutation arguments, known `Int` reassignments,
  broader `Int` expressions, and broader `if`/`while`/`else if` condition
  expressions.
- `List<Struct>.filter(...).map(...)` can project filtered declared-record
  fields directly into reusable or directly returned `List<Int>` and `List<Str>`
  scalar lists, feed those scalar lists directly to helper calls, or extend
  or reassign `List<Int>`/`List<Str>` buffers directly, without a user-written
  intermediate record list; those helper calls can also start `if`, `while`,
  and `else if` condition expressions.
- `List<Struct>.filter(...).map(...).max()` and `.min()` can rank projected
  `Int` score fields directly without materializing an intermediate score list;
  filtered score `sum()`/`max()`/`min()` aggregates can also feed `Int`
  helper-call arguments directly, including helper calls that start `if`,
  `while`, or `else if` condition expressions, and can appear inside broader
  `Int` expressions used by locals, helper-call arguments, direct `List<Int>`
  mutation arguments, reassignments, and returns, plus broader `if`, `while`,
  and `else if` condition expressions.
- The native compiler and checker can be installed as standalone `vaisc` and
  `vais-check` binaries outside the checkout and packaged as a release archive.
- Source tag builds have a release archive workflow for standalone compiler and
  checker assets.
- The `v0.2.2` source tag produced a GitHub Release with Linux x64, macOS
  arm64, and macOS x64 standalone compiler archives.
- The native direct engine covers Int helper calls, locals, assignment, `if`,
  `while`, returns, simple Int-field struct locals, and struct parameter/return
  helpers through the native direct path.
- The native direct engine covers the first local `List<Int>` slice: `[]`,
  `list()`, small integer list literals, `push`, `len`/`len()`, index, and
  `sum()`.
- The native direct engine covers `List<Int>` function parameter and return
  ABI, including push-to-parameter mutation for local list arguments.
- The native direct engine covers inline `List<Int>` literal and `list()`
  values in call arguments and return expressions.
- The native direct engine hoists `List<Int>`-returning helper calls used as
  `List<Int>` call arguments in statement contexts.
- The full self-host path covers 20-field flat struct literals and field reads,
  including recursive helper evaluation over an index-encoded AST.
- The public compiler driver covers the first `Int` tuple return and local
  destructuring slice through generated struct lowering.
- The public compiler driver covers returned single-`Int` closures passed to a
  single-closure `Int` higher-order helper by expanding calls to the generated
  closure apply helper.
- The public compiler driver covers non-capturing inline closure literals
  passed directly to a single-closure `Int` higher-order helper by generating
  inline apply helpers.
- The public compiler driver covers local closures with one `Int` capture
  called inside the same function by lowering the capture to the apply-helper
  environment value.
- The public compiler driver covers the first simple `impl` struct method
  return-chain slice by lowering methods to helper functions with intermediate
  struct locals.
- The public compiler driver covers the first simple `trait` plus
  `impl Trait for Struct` method-call expression slice by treating the trait
  declaration as metadata and lowering the impl method to a struct helper.
- The public compiler driver covers `List<Int>` `map`, `filter`,
  `filter(...).sum()`, and filtered `max()`/`min()` method slices, including
  known `Int` captures, by lowering them to explicit `for` loops.
- The public compiler driver covers the first local `List<List<Int>>` literal
  double-index read slice by lowering nested rows to `List<Int>` locals.
- The public compiler driver covers the first enum `Option<Int>` payload slice
  by lowering a nested Option match arm to Int-coded branches.
- The public compiler driver covers payload enum `match` with `_` catch-all
  arms for the current Int-coded payload enum slice.
- The native direct engine hoists `List<Int>`-returning helper calls in `while`
  conditions and reevaluates them on each loop iteration.
- The native direct engine lowers `List<Int>` and `List<Struct>` returned-list
  helper calls used as list arguments in `if` and `else if` conditions.
- The native direct engine covers local `List<Struct>` values for declared
  structs: typed `[]`, `list()`, list literal initialization, `push`, `len`,
  index, and field reads.
- The native direct engine covers `List<Struct>` function parameter and return
  ABI, including inline list arguments and returned-list argument hoisting.
- The native direct engine covers context-typed assignment for `List<Int>` and
  `List<Struct>` locals and list parameters.
- The native direct engine covers `List<Int>` and `List<Struct>` element
  assignment, including assignments through list parameters.
- The native direct engine covers indexed `List<Struct>` field assignment,
  including assignments through list parameters.
- `List<T>.is_empty()` is promoted for the full self-host path and native
  direct engine, with gates for Int and declared-struct lists.
- `List<T>.last()` is promoted for non-empty lists in the full self-host path
  and native direct engine, with Int and declared-struct list gates.
- `List<T>.pop()` is promoted for non-empty lists in the full self-host path
  and native direct engine, with Int and declared-struct list gates.
- Indexed `List` reads/writes plus `last()` and `pop()` now trap at runtime on
  negative indexes, out-of-range indexes, or empty-list access.
- `Str` length, byte index, equality/inequality, `Bool` byte-classification
  helpers, user-defined integer parsing, word-count scanning, palindrome scans,
  and substring-search patterns are promoted through the full self-host
  compiler, public front, parity, and native direct gates.
- Print interpolation for simple identifiers and `putchar(Int)` output calls
  are promoted through the full self-host path, native direct engine, parity
  manifest, and value corpus.
- Additional control-flow, inclusive range, simple struct, Bool predicate,
  integer-list indexing, and state-machine examples are promoted through the
  parity manifest and value corpus.
- Additional enum, bitwise, and Option smoke files are promoted through the
  parity manifest and value corpus to keep older examples covered.
- Collection for-each over integer values is promoted through the full self-host
  path, native direct engine, parity manifest, value corpus, and regenerated
  reusable core.
- Typed non-empty local `List<Int>` literals are promoted through the full
  self-host path, including calls to `List<Int>` parameters and collection
  for-each over those values.
- Inline integer list literals can be passed directly to `List<Int>` parameters
  through the full self-host path and native direct engine.
- Additional release-corpus examples cover inline `List<Int>` parameter
  iteration, direct `Option<Int>` helper-return matching, and chained
  `Result<Int,Int>` `?` propagation.
- Borrowed `&List<Int>` helper parameters are promoted through the full
  self-host path, public front, parity manifest, and value corpus.
- Generic marker syntax on simple structs used with `Int` values is promoted
  through the full self-host path, public front, parity manifest, and value
  corpus.
- Generic identity helpers applied directly to struct literals are promoted
  through the public compiler driver, public front, parity manifest, and value
  corpus.
- Struct helper parameters, struct helper returns, and assignment from
  struct-returning calls are promoted through the full self-host path, native
  direct engine, public front, parity manifest, value corpus, and regenerated
  reusable core.
- `List<Struct>.push(make_struct(...))` for local and parameter lists is
  promoted through the full self-host path, native direct engine, public front,
  parity manifest, value corpus, and regenerated reusable core.
- `List<Struct>.insert_at(index, make_struct(...))` for local and parameter
  lists is promoted through the full self-host path, native direct engine,
  public front, parity manifest, value corpus, and regenerated reusable core.
- `List<Struct>.push(value)` for same-type struct local/parameter values is
  promoted through the full self-host path, native direct engine, public front,
  parity manifest, value corpus, and regenerated reusable core.
- `List<Struct>.push(xs[i])` and `insert_at(index, xs[i])` for same-type list
  element values are promoted through the full self-host path, native direct
  engine, public front, parity manifest, value corpus, and regenerated reusable
  core.
- `List<Struct>.push(xs.pop()/xs.remove_at(i))` and
  `insert_at(index, xs.pop()/xs.remove_at(i))` for same-type list method return
  values are promoted through the full self-host path, native direct engine,
  public front, parity manifest, value corpus, and regenerated reusable core.
- `List<Struct>.push(xs.first()/xs.last())` and
  `insert_at(index, xs.first()/xs.last())` for non-mutating same-type list
  method return values are promoted through the full self-host path, native
  direct engine, public front, parity manifest, value corpus, and regenerated
  reusable core.
- `List<Struct>.extend(make_list(...))` for same-type list-returning helper
  calls is promoted through the full self-host path, native direct engine,
  public front, parity manifest, value corpus, and regenerated reusable core.
- `List<Int>.extend(make_list(...))` and
  `List<Str>.extend(make_list(...))` for same-type list-returning helper calls
  are promoted through the full self-host path, native direct engine, public
  front, parity manifest, value corpus, and regenerated reusable core; the same
  slice also locks full-path `List<Int>.sum()` on list parameters.
- `List<Int>.extend([..])` and `List<Str>.extend([..])` for inline list
  literal source values are promoted through the full self-host path, native
  direct engine, public front, parity manifest, value corpus, and regenerated
  reusable core.
- `List<Struct>.extend([Struct { .. }])` for inline struct list literal source
  values is promoted through the full self-host path, native direct engine,
  public front, parity manifest, value corpus, and regenerated reusable core.
- `List<Struct>` typed local initialization and local/parameter assignment from
  inline struct list literal values are promoted through the full self-host
  path, native direct engine, public front, parity manifest, value corpus, and
  regenerated reusable core.
- `List<Struct>.first().field`, `.last().field`, `.pop().field`, and
  `.remove_at(index).field` are promoted for local and parameter lists through
  the full self-host path, native direct engine, public front, parity manifest,
  value corpus, and regenerated reusable core.
- Multiline typed `List<Struct>` literals with trailing commas are promoted
  through the full self-host path, native direct engine, public front, parity
  manifest, value corpus, and regenerated reusable core. This also locks
  semicolon-free full statement advancement for list methods and `let`
  initializers, plus fast no-import preflight in the import graph checker.
- Multiline inline `List<Struct>` literal call arguments with trailing commas
  are promoted through the full self-host path, native direct engine, public
  front, parity manifest, value corpus, and regenerated reusable core.
- Standalone call statements with multiline inline `List<Struct>` literal
  arguments and trailing commas are promoted through the full self-host path,
  native direct engine, public front, parity manifest, value corpus, and
  regenerated reusable core.
- `List<Struct>.push(Box { ... })` with multiline trailing-comma struct
  literals is promoted through the full self-host path, native direct engine,
  public front, parity manifest, value corpus, and regenerated reusable core.
- Multiline struct literals in `List<Struct>` indexed element assignment and
  struct-returning `return` statements are promoted through the full self-host
  path, native direct engine, public front, parity manifest, value corpus, and
  regenerated reusable core.
- Multiline struct literals in plain struct local initialization, typed local
  initialization, same-type local assignment, and struct call arguments are
  promoted through the full self-host path, native direct engine, public front,
  parity manifest, value corpus, and regenerated reusable core.
- `List<Struct>.insert_at(index, Box { ... })` and
  `List<Struct>.extend([Box { ... }])` with multiline struct literal sources
  are promoted through the full self-host path, native direct engine, public
  front, parity manifest, value corpus, and regenerated reusable core.
- Single-field nested struct literals, nested field reads, and nested field
  writes are promoted through the full self-host compiler path and native
  direct flattening for previously declared single-`Int`-field nested structs,
  with public front, parity manifest, value corpus, and regenerated reusable
  core coverage.
- Indexed `List<Struct>` element field-chain reads and writes such as
  `xs[0].inner.v` and `xs[0].inner.v = value` are promoted for elements
  containing a previously declared single-`Int`-field nested struct, including
  nested struct literals pushed into the list, through the full self-host path,
  native direct engine, public front, parity manifest, value corpus, and
  regenerated reusable core coverage.
- `List<Struct>` method-result field chains such as `xs.first().inner.v`,
  `xs.last().inner.v`, `xs.pop().inner.v`, and `xs.remove_at(i).inner.v` are
  promoted for the same single-`Int`-field nested struct shape through the full
  self-host path, native direct engine, public front, parity manifest, value
  corpus, and regenerated reusable core coverage.
- Struct-returning helper field chains such as `make_box(...).value` and
  `make_outer(...).inner.v` are promoted for top-level fields and the same
  single-`Int`-field nested struct shape through the full self-host path,
  native direct engine, public front, parity manifest, value corpus, and
  regenerated reusable core coverage.
- Direct returns of single-field nested struct literals such as
  `return Outer { inner: Inner { v: value } }` are promoted through the full
  self-host path, native direct engine, public front, parity manifest, value
  corpus, and regenerated reusable core coverage.
- Scalar multi-field nested structs such as `Outer { inner: Inner }` where
  `Inner` has multiple `Int` fields are promoted for local literals, direct
  helper returns, and field-chain reads through the full self-host path, native
  direct engine, public front, parity manifest, value corpus, and regenerated
  reusable core coverage.
- Public struct/function/field modifiers are accepted as metadata through the
  checker, public front, full self-host compiler, parity manifest, value corpus,
  and regenerated reusable core. Struct literal lowering stores `Str` fields
  through the same pointer-to-integer representation used by string-key
  collections.
- Single-byte `Char` literals, equality, explicit annotations, helper
  parameters, and helper returns are promoted through public front, full
  self-host, parity, and native direct gates as Int-compatible scalar values.
- Named integer parsing helpers `parse_uint(s)` and `parse_int(s)` are promoted
  through the full self-host compiler, native direct engine, front gate, parity
  manifest, value corpus, and regenerated reusable core.
- `Str(Int)` decimal conversion is promoted through the full self-host
  compiler, native direct engine, front gate, parity manifest, value corpus,
  and regenerated reusable core.
- The first `Map` slices are verified in the full self-host compiler and native
  direct engine for local `Map<Int,Int>` values with `{}`, assignment copy,
  `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and
  `len`, and local `Map<Int,Bool>` and `Map<Int,Char>` values with `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
  `contains`, and `len`, plus local `Map<Str,Int>` values with `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`,
  `get_opt(key)`, `contains`, and `len`, and local `Map<Str,Bool>` values
  with the same local string-key method surface, plus local `Map<Str,Char>`
  values with the same local string-key method surface.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>`
  parameters are verified by reference, so callees can mutate caller-visible
  maps. `Map<Str,Char>` parameters are also verified by reference. Concrete
  Map assignment can copy between locals, same-type Map parameters, and
  same-type Map-returning calls without aliasing.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, and `Map<Str,Char>` return values can initialize explicitly
  annotated locals through caller-owned storage.
  Parameter-target assignment copies are covered for all six verified concrete
  Map types in the release corpus and full self-host codegen gate.
- Promoted prelude APIs have value-corpus examples, including local
  `Map<Int,Int>`, local `Map<Int,Bool>`, local `Map<Int,Char>`,
  local `Map<Str,Int>`, local `Map<Str,Bool>`, local `Map<Str,Char>`,
  `Map<Int,Bool>` parameter mutation, `Map<Int,Char>` parameter mutation,
  `Map<Str,Int>` parameter mutation, `Map<Str,Bool>` parameter mutation,
  `Map<Str,Char>` parameter mutation,
  `Map<Int,Int>` return-value local initialization, `Map<Int,Bool>`
  return-value local initialization, `Map<Int,Char>` return-value local
  initialization, `Map<Str,Int>` return-value local initialization, local
  `Map<Str,Bool>` string-key operations, `Map<Str,Bool>` return-value local
  initialization, local `Map<Str,Char>` string-key operations,
  `Map<Str,Char>` return-value local initialization, concrete Map parameter
  assignment copy, concrete Map-returning call assignment copy,
  argument-bearing Map-returning call assignment copy, concrete Map key
  removal, concrete Map scalar get_opt payloads, concrete Map clear and reuse,
  `Str(Int)` decimal conversion, and
  `List<T>.is_empty()`, `last()`, and `pop()`.
- The full compiler path supports single-package local dotted imports such as
  `import math.add`, with gates for multi-file success, missing imports,
  duplicate symbols, and import cycles.
- The full compiler path supports the first `vais.toml` package manifest slice:
  required `name`, `version`, and `source` keys, source-root import resolution,
  and manifest diagnostics.
- The full compiler path supports local dependency package paths in `vais.toml`
  `[dependencies]`, with native gates for dependency imports and dependency
  manifest diagnostics.
- Local module imports, package source roots, and local dependency package
  imports have release-corpus examples in addition to front-contract gates.
- Phase 3 host file/path/process APIs are specified in `docs/design/HOST_IO.md`;
  `fs_exists`, `fs_read_text`, `fs_write_text`, `fs_mkdirs`, `fs_remove`,
  `fs_cwd`, `fs_temp_dir`, `path_join`, `path_basename`, and `path_dirname`
  are the first verified full-engine host file/path intrinsics, and
  `proc_argc() -> Int`, `proc_arg(index: Int) -> Str`,
  `proc_run(argv: List<Str>) -> Int`,
  `proc_run_env(argv: List<Str>, env: List<Str>) -> Int`,
  `proc_capture_stdout(argv: List<Str>) -> Str`,
  `proc_capture_stderr(argv: List<Str>) -> Str`, and
  `proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int`
  are the first verified process intrinsics. Program argv, child environment
  overrides, captured stdout/stderr, and status-plus-file capture are verified
  for full-engine `vaisc run` and binaries produced by `vaisc build`.
- `Str` construction helpers `str_concat`, `str_slice`, and `str_byte` are
  verified through full/direct and host gates so Vais-authored text
  transformation tools can be ported incrementally.
- Host-backed `Str` builder helpers `str_builder_new`, `str_builder_push`,
  `str_builder_append`, and `str_builder_finish` are verified through the host
  gate for large Vais-authored text transformation tools.
- Full-engine `Str` reassignment and user-defined `-> Str` helper returns are
  verified through the host gate.
- `tools/vais_check_core.vais` and `tools/vais_check_cli.vais` are the
  Vais-authored public checker sources. `scripts/vais-check` builds and runs
  them as the canonical lint/error-help command, release archives include the
  standalone `bin/vais-check` binary. `tools/vais_check_contract_check.vais`
  drives the focused checker fixture, CLI, path, help, and public wrapper
  contract gate; the shell entrypoint is only a bootstrap wrapper. The checker
  owns the invalid static import path diagnostic, and the public front contract
  keeps the same error shape gated.
- `tools/embed_self_source.vais` is the Vais-authored self-source embedding
  helper. Its focused gate is driven by `tools/embed_self_source_check.vais`,
  which writes the fixtures, runs normalized and raw embedding, builds the
  generated compilers through the trust-root path, and verifies their emitted
  IR/binary results; the shell entrypoint is only a bootstrap wrapper.
- `tools/normalize_stage_ir.vais` is the Vais-authored stage IR normalizer.
  Its focused gate checks the expected normalized IR shape directly through the
  Vais helper, and the long full-source self-host gate uses it for stage1/stage2
  compiler IR comparison. Its global-name mapping uses a 4-field struct list so
  file-sized compiler IR with more than 4,096 distinct string globals can still
  be normalized under the current fixed-list runtime.
- Internal self-host helper builds now use the native `scripts/vaisc`
  trust-root path.
- `docs/design/MAP_ABI.md` specifies the future Map parameter, return, and
  generic expansion contract without promoting broader Map behavior.

## Current Reality

- The full compiler path emits LLVM IR through the self-host compiler source in `compiler/self/fixpoint_full.vais`.
- The direct engine is intentionally narrow and currently supports Int helpers,
  Bool/Str scalar helpers, locals, assignment, calls, `if`, inline
  `if { return ... }`, `while`, range `for`, `break`, `continue`, returns, `Str` literals, `Str.len()`, `Str`
  byte index, `Str` equality/inequality, `Char` literal equality, annotations,
  helper parameters, helper returns, named `parse_uint`/`parse_int`
  helpers, simple Int-field struct local literal/read/write, struct
  parameter/return helper ABI, and local
  `List<Int>` initialization, typed non-empty local `List<Int>` literals,
  inline `List<Int>` literal call arguments, plus `push`, `len`, `is_empty`,
  `last`, `pop`, index, `sum`, and
  `List<Int>` parameter reference, return value ABI, and inline list
  literal/constructor call and return values. Statement contexts, `if`,
  `else if`, and `while` conditions also lower `List<Int>`-returning helper
  calls before passing them to `List<Int>` parameters. Range `for` supports
  exclusive `..` and inclusive `..=` bounds through both full self-host and
  native direct paths, with `break` and `continue` lowered inside `while` and
  range `for` loops. Local `List<Struct>`
  values support typed `[]`, `list()`, list literal initialization, `push`
  from struct values, list element values, list method return values, and
  struct-returning helper calls, `insert_at` including list element values,
  list method return values, and struct-returning helper calls,
  `len`, `is_empty`, `last`, `pop`, index, field reads/writes, parameter reference, return value ABI,
  inline list arguments, and returned-list argument lowering in statements plus
  `if`, `else if`, and `while` conditions. Context-typed list assignment is supported
  for `List<Int>` and `List<Struct>` locals and list parameters. Element
  assignment is supported for `List<Int>` and `List<Struct>`, including through
  list parameters. Local `Map<Int,Int>` values support `{}`, assignment copy,
  `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key) -> Option<Int>`,
  `contains`, and `len`; local `Map<Int,Bool>` values support `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
  `contains`, and `len`; local `Map<Int,Char>` values support `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
  `contains`, and `len`; local `Map<Str,Int>` values support `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`,
  `get_opt(key)`, `contains`, and `len`; local `Map<Str,Bool>` values support
  the same local string-key method surface; local `Map<Str,Char>` values
  support the same local string-key method surface;
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>` parameters support
  reference mutation in both the full self-host compiler path and native direct
  engine. `Map<Str,Char>` parameters also support reference mutation.
  Concrete Map assignment can copy between locals, same-type Map parameters,
  and same-type Map-returning calls without aliasing; the release corpus covers
  both no-argument and argument-bearing Map-returning call assignment.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, and `Map<Str,Char>` return values can initialize explicitly
  annotated locals.
  Generic key/value forms are not claimed yet.
  The future Map ABI and generic expansion contract is specified
  in `docs/design/MAP_ABI.md`.
- The release compiler command uses a native host driver for `emit-ir`,
  `build`, and `run`; internal self-host helper gates use the same native
  compiler path.
- Standalone install, uninstall, package, and install/package verification
  scripts exist for the native compiler and checker binaries.
- Internal compiler gates no longer depend on a source pass-through helper.
- Full self-host lowering for runtime `Str` equality/inequality is gate-backed,
  and Vais-authored tools can use idiomatic `a == b` / `a != b` string
  comparisons.
- The long full-source self-host gate retargets compiler sources with the
  Vais-native embed helper.
- The long full-source self-host gate normalizes stage comparison IR with the
  Vais-native normalizer.
- Public documentation now starts at `README.md` and `docs/README.md`.
- `docs/reference/LANGUAGE.md` describes only the current gate-backed language surface.
- Local official website source was refreshed and rebuilt from the canonical Vais docs.
- Official site source now lives in `website/` in this repository.
- GitHub Pages workflow was added for `website/` build and artifact deployment.
- `vaislang.dev` deploys from the `website/` GitHub Pages workflow on `main`.
- `CHANGELOG.md` records the current `v1.0.1` stable source release baseline.
- GitHub `main` now points to the current Vais-only history; old remote `main`
  is preserved at `archive/old-main-2026-06-14`.

## Next Work

1. Expand the standard library only through gate-backed APIs.
2. Specify and implement file and process primitives needed for Vais-authored
   repository tools.
3. Replace host-only internal checks incrementally with Vais-backed tools where
   the language is strong enough.
4. Broaden types, collections, and control syntax without publishing ungated
   claims.
5. Move more compiler development and verification into the self-host tier while
   keeping native host responsibilities explicit.
6. Keep GitHub Releases, GitHub Pages, self-host regeneration, direct/full parity,
   and value gates green at each milestone.

## Vais v1 Completion Roadmap

This is the durable completion plan for turning the current Vais baseline into a
language/toolchain that can reasonably be called complete for a first stable
release. "Complete" means documented, implemented, tested, packaged, and
published from this repository without compatibility notes for older names or
alternate source extensions.

### Phase 0: Release Discipline

Goal: make every future capability land behind a repeatable release process.

- [x] 0.1 Define the next release line and tag policy in `CHANGELOG.md`,
  `README.md`, and release docs.
- [x] 0.2 Add a release checklist that runs native, install/package, direct,
  front, parity, value, and self-host regeneration gates before tagging.
- [x] 0.3 Prove one source tag produces a GitHub Release with standalone
  archives and a smoke-tested packaged `vaisc`.
- [x] 0.4 Keep `vaislang.dev` synced from repository docs for every release.

Done: a clean checkout can produce and verify a tagged release archive, and the
public site describes exactly that release.

### Phase 1: Standard Library Core

Goal: grow a small, reliable prelude instead of a large speculative API list.

- [x] 1.1a Promote verified `List<T>.is_empty()` across the full self-host path
  and native direct engine.
- [x] 1.1b Promote verified `List<T>.last()` across the full self-host path and
  native direct engine.
- [x] 1.1c Promote verified `List<T>.pop()` across the full self-host path and
  native direct engine.
- [x] 1.1d Define bounds-safe diagnostics or documented trap behavior for
  indexed list operations.
- [x] 1.2a Promote `Str` operations needed by real tools: `len`, index,
  equality, byte classification helpers, and user-defined integer parsing
  patterns.
- [x] 1.2b Decide and promote a named integer parsing prelude API, if it should
  be part of the public standard library instead of a user helper pattern.
- [x] 1.3a Specify the first `Map` slice and gate unsupported `Map` use with a
  clear front diagnostic.
- [x] 1.3b Promote native direct local `Map<Int,Int>` for construction,
  insert/replace, `get(key, default)`, `get_opt(key)`, `contains`, and `len`.
- [x] 1.3c Promote full self-host local `Map<Int,Int>` for the same surface.
- [x] 1.3d Specify `Map<K,V>` generic key/value lowering and ABI behavior before
  broadening.
- [x] 1.3e Promote local `Map<Int,Int>` assignment copy through full and direct
  gates.
- [x] 1.3f Promote local `Map<Int,Bool>` through concrete full/direct/front
  gates while keeping `get_opt` behind `Option<Bool>`.
- [x] 1.3g Promote local `Map<Int,Char>` through concrete full/direct/front
  gates while keeping `get_opt` behind `Option<Char>`.
- [x] 1.3h Promote `Map<Int,Int>` function parameters by reference through
  concrete full/direct/front gates while keeping Map returns gated until a
  concrete return slice is promoted.
- [x] 1.3i Promote `Map<Int,Bool>` function parameters by reference through
  concrete full/direct/front gates while keeping non-`Map<Int,Int>` returns
  and broader Map parameters gated.
- [x] 1.3j Promote `Map<Int,Char>` function parameters by reference through
  concrete full/direct/front gates while keeping Map returns gated until the
  next concrete slice.
- [x] 1.3k Broaden `Map<K,V>` only through concrete gate-backed slices:
  promote `Map<Int,Int>` return values for local initialization while keeping
  `Map<Int,Bool>`, `Map<Int,Char>`, and generic Map returns gated.
- [x] 1.3l Broaden `Map<K,V>` only through the next concrete gate-backed slice:
  promote `Map<Int,Bool>` return values for local initialization while keeping
  `Map<Int,Char>` and generic Map returns gated.
- [x] 1.3m Broaden `Map<K,V>` only through the next concrete gate-backed slice:
  promote `Map<Int,Char>` return values for local initialization while keeping
  generic Map returns gated.
- [x] 1.3n Broaden `Map<K,V>` only through the next concrete gate-backed slice:
  promote `remove(key)` for concrete `Map<Int,Int>`, `Map<Int,Bool>`, and
  `Map<Int,Char>` values while keeping generic Map behavior gated.
- [x] 1.3o Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `get_opt(key)` for `Map<Int,Bool>` and
  `Map<Int,Char>` match payloads while keeping generic Map behavior gated.
- [x] 1.3p Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `clear()` for concrete `Map<Int,Int>`,
  `Map<Int,Bool>`, and `Map<Int,Char>` values while keeping generic Map
  behavior gated.
- [x] 1.3q Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote local `Map<Str,Int>` string-key operations before
  parameter, return, and broader generic Map behavior.
- [x] 1.3r Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Int>` function parameters by reference
  while keeping `Map<Str,Int>` returns and broader generic Map behavior gated.
- [x] 1.3s Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Int>` return values for explicitly
  annotated local initialization while keeping broader `Map<Str,V>` and
  generic Map returns gated.
- [x] 1.3t Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote local `Map<Str,Bool>` string-key operations.
- [x] 1.3u Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Bool>` function parameters by reference
  while keeping broader generic Map behavior gated.
- [x] 1.3v Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Bool>` return values for explicitly
  annotated local initialization while keeping broader generic Map behavior
  gated.
- [x] 1.3w Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote local `Map<Str,Char>` string-key operations while
  leaving follow-up ABI slices and broader generic Map behavior to later
  gate-backed tasks.
- [x] 1.3x Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Char>` function parameters by reference
  while leaving broader generic Map behavior to later gate-backed tasks.
- [x] 1.3y Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Char>` return values for explicitly
  annotated local initialization while keeping broader generic Map behavior
  gated.
- [x] 1.3z Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote concrete Map parameter-source and parameter-target
  assignment copies while keeping broader generic Map behavior gated.
- [x] 1.3aa Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote concrete Map-returning call assignment copies while
  keeping broader generic Map behavior gated.
- [x] 1.3ab Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: add argument-bearing Map-returning call assignment coverage
  while keeping broader generic Map behavior gated.
- [x] 1.3ac Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: add all-concrete Map parameter-target assignment coverage
  while keeping broader generic Map behavior gated.
- [x] 1.4 Add examples and value tests for every promoted prelude API.
- [x] 1.5 Update `std/PRELUDE.md` so "Verified" means compiler-gate protected.

Done: `std/PRELUDE.md` has no public "Verified" entry without a matching gate.

### Phase 2: Modules, Packages, And Imports

Goal: allow real projects to split code across files without inventing a large
package manager too early.

- [x] 2.1 Specify a minimal module model: file module names, import paths, symbol
  visibility, duplicate-name diagnostics, and cycle behavior.
- [x] 2.2 Implement single-package multi-file compilation for `scripts/vaisc`.
- [x] 2.3 Add `import` support for local package paths with deterministic
  ordering and stable diagnostics.
- [x] 2.4 Add package manifest support for name/version/source roots.
- [x] 2.5 Add local dependency package paths.
- [x] 2.6 Add package manifest examples, docs, gates, and source-root package
  smoke builds.
- [x] 2.7 Add local dependency examples, docs, gates, and package smoke builds.

Done: small multi-file and local dependency Vais projects build with
`scripts/vaisc build` and are covered by CI gates.

### Phase 3: File And Process Support

Goal: give Vais enough host interaction for repository tools and release
validation.

- [x] 3.1 Specify file read/write, path, temp directory, stdout/stderr, exit code,
  and process execution APIs.
- [x] 3.2 Implement the first host-backed intrinsic in the native driver without
  mixing it into pure compiler-core logic.
- [x] 3.3 Extend host-backed file intrinsics to text writes and directory
  creation.
- [x] 3.4 Add the first `Str`-returning host intrinsic for text file reads.
- [x] 3.5 Add `Str`-returning path helper intrinsics.
- [x] 3.6 Add argv-based process intrinsics.
- [x] 3.6a Add the first captured stdout process intrinsic.
- [x] 3.6b Add captured stderr process support for Vais-authored diagnostics
  harnesses.
- [x] 3.6c Add child-process environment override support for Vais-authored
  process checks.
- [x] 3.6d Add status-plus-stdout/stderr file capture for Vais-authored
  process checks without requiring a struct-returning host ABI.
- [x] 3.7 Port the simplest checker to Vais first.
- [x] 3.7a Add minimal `Str` construction helpers needed by Vais-authored
  repository tools.
- [x] 3.8 Port release archive packaging orchestration to Vais once the
  file/process APIs can read paths, capture platform commands, stage text docs,
  and run argv-based child processes.
- [x] 3.8b Port standalone install orchestration to Vais while keeping the
  initial uninstall path shell-native so removal does not require a compiler.
- [x] 3.8c Move parity, value-corpus, and host smoke gate logic into
  Vais-authored harnesses while preserving thin bootstrap wrappers.
- [x] 3.8d Move the NV-C0 public compiler smoke gate into a Vais-authored
  harness while preserving a thin bootstrap wrapper.
- [x] 3.8e Move the native driver smoke gate into a Vais-authored harness while
  preserving native C build script bootstrap.
- [x] 3.8f Move the NV-C3 diagnostics gate into a Vais-authored harness after
  adding captured stderr process support.
- [x] 3.8g Move the legacy self-host compiler smoke gate into a Vais-authored
  harness while preserving the shell bootstrap boundary.
- [x] 3.8h Move the NV-C1 front contract gate into a Vais-authored harness while
  preserving a thin bootstrap wrapper.
- [x] 3.8i Move the direct-engine no-Python PATH check into a Vais-authored
  harness after adding child environment process support.
- [x] 3.8j Move the direct-engine arithmetic/build/run smoke checks into a
  Vais-authored harness.
- [x] 3.8k Move the direct-engine import handling and List bounds trap checks
  into a Vais-authored harness using status-plus-file process capture.
- [x] 3.8l Move the direct helper/control-flow, range `for`, struct-local, and
  struct ABI success fixtures into a Vais-authored harness.
- [x] 3.8m Move direct local `List<Int>`, `Str`, `Char`,
  `parse_uint`/`parse_int`, local `Map<Int,Int>`, local `Map<Int,Bool>`, local
  `Map<Int,Char>`, and local `List<Struct>` success fixtures into the
  Vais-authored feature harness.
- [x] 3.8n Move the remaining direct List ABI, assignment, and returned-list
  hoist shell fixtures into the Vais-authored feature harness.
- [x] 3.8o Audit remaining shell wrappers and keep only bootstrap, process
  supervision, or platform-specific CI glue.
- [x] 3.8p Move the stage IR normalizer focused gate sample/expected fixture and
  shape checks into a Vais-authored check harness.
- [x] 3.8q Move the self-source embedding focused gate fixture generation,
  trust-root build/run checks, and generated compiler result assertions into a
  Vais-authored check harness.
- [x] 3.8r Move the checker focused gate output-count, diagnostic-pattern,
  path, help, and public-wrapper assertions into a Vais-authored contract
  harness.
- [x] 3.8s Move the short `fixpoint.vais` and `fixpoint2.vais` tier fixture
  lists, raw-call embedding, trust-root compiler builds, emitted-IR clang
  checks, and result assertions into a Vais-authored harness.
- [x] 3.8t Add verified `fs_remove(path)` and port standalone uninstall
  orchestration to `tools/uninstall_vaisc.vais`.
- [x] 3.8u Move standalone install/package verification assertions into
  `tools/vaisc_install_check.vais`.
- [x] 3.8v Move the NV-C2 direct-emitter gate orchestration into
  `tools/vaisc_direct_gate.vais`, leaving the shell file as only the temp-dir
  bootstrap wrapper.
- [x] 3.8w Reduce single-tool focused shell wrappers to temp-dir bootstrap
  wrappers that invoke their Vais-authored gates through `scripts/vaisc run`.
- [x] 3.8x Move the long full-source self-host compiler orchestration into
  `tools/fixpoint_full_self_check.vais`, leaving the shell file as a
  temp-directory bootstrap wrapper.
- [x] 3.8y Move the long full-codegen regression runner into
  `tools/fixpoint_full_codegen_check.vais`, leaving the shell file as a
  temp-directory bootstrap wrapper.
- [x] 3.8z Audit the remaining host boundaries and leave native C build,
  public command cache wrappers, release-gate/CI orchestration, website build,
  tar/install/clang system tools, and temp-dir bootstrap wrappers explicit.
- [x] 3.9 Keep public checker release gates on the Vais-authored checker.

Done: the public checker, release archive packager, standalone installer,
parity manifest validator, value-corpus validator, host smoke validator, NV-C0
public compiler smoke validator, front contract validator, native driver smoke
validator, NV-C3 diagnostics validator, legacy self-host compiler smoke
validator, direct no-Python environment validator, direct arithmetic/build
smoke validator, direct reject/trap validator, direct feature validator, and
direct-emitter gate runner run from Vais source. The direct feature validator
now covers the scalar,
collection, struct, helper, list ABI, assignment, and returned-list hoist
success fixture groups. The checker contract, stage IR normalizer, and
self-source embed focused gates now use Vais-authored check harnesses. The
short fixpoint tier gates also use a shared Vais-authored harness, the
full-codegen regression runner executes its 200 fixture cases plus source-file
and IR shape checks from a Vais-authored harness, and the full-source self-host
gate retargets compiler sources, builds generated compilers, validates emitted
IR, and compares normalized stage output from a Vais-authored harness.
Standalone uninstall plus install/package verification are backed by Vais
tools. The focused, full-codegen, and self-host shell entrypoints now use
`scripts/vaisc run` directly and remain only as temp-directory bootstrap
boundaries. The remaining host boundaries are audited and intentionally limited
to native C bootstrap/driver code, public command cache wrappers,
release-gate/CI orchestration, website build tooling, tar/install/clang system
tools, and temporary directory setup.

### Phase 4: Broader Language Surface

Goal: expand the language deliberately while avoiding unsupported public claims.

- [x] 4.1 Stabilize `Bool`, `Str`, and `Char` as first-class surface types across
  full and direct gates where feasible.
- [x] 4.1a Promote single-byte `Char` literal equality plus explicit `Char`
  local annotations, helper parameters, and helper returns through public
  front, native direct, full self-host, and parity gates.
- [x] 4.1b Promote explicit `Bool` local annotations, helper parameters, helper
  returns, and unary `not` through public front, native direct, full self-host,
  and parity gates.
- [x] 4.1c Promote explicit `Str` local annotations, helper parameters, helper
  returns, reassignment, length, index, and equality through public front,
  native direct, full self-host, and parity gates.
- [x] 4.1d Promote generic identity helpers applied directly to struct literals
  through public driver lowering, front, parity, and value gates.
- [x] 4.1e Promote 20-field flat struct literals and field reads through full
  self-host, parity, and value gates.
- [x] 4.1f Promote `Int` tuple returns and local destructuring through public
  driver lowering, front, parity, and value gates.
- [x] 4.1g Promote returned single-`Int` closures passed to an `Int`
  higher-order helper through public driver lowering, front, parity, and value
  gates.
- [x] 4.1h Promote simple `impl` struct method return chains through public
  driver lowering, front, parity, and value gates.
- [x] 4.1i Promote non-capturing inline closure literals passed to an `Int`
  higher-order helper through public driver lowering, front, parity, and value
  gates.
- [x] 4.1j Promote local single-capture `Int` closure calls through public
  driver lowering, front, parity, and value gates.
- [x] 4.1k Promote simple `trait` plus `impl Trait for Struct` method-call
  expressions through public driver lowering, front, parity, and value gates.
- [x] 4.1l Promote non-capturing `List<Int>` map and filter-sum method slices
  through public driver lowering, front, parity, and value gates.
- [x] 4.1m Promote local `List<List<Int>>` literal double-index reads through
  public driver lowering, front, parity, and value gates.
- [x] 4.2 Add broader enum payloads and pattern/match forms after the current
  simple return-arm shape is fully gated.
- [x] 4.2a Promote simple expression-arm `match` lowering for multi-field `Int`
  payload enum variants through public front, full self-host, parity, and value
  gates.
- [x] 4.2b Promote payload-free enum values stored in simple struct fields and
  matched through field access through public front, full self-host, parity, and
  value gates.
- [x] 4.2c Promote single-field struct payload enum lowering for constructor
  literals and payload field access through public front, parity, and value
  gates.
- [x] 4.2d Promote Int `match` literal arms with `_` catch-all lowering through
  public front, parity, and value gates.
- [x] 4.2e Promote payload-free enum `match` with `_` catch-all through public
  front, parity, and value gates.
- [x] 4.2f Promote a single enum `Option<Int>` payload with nested Option match
  arm lowering through public front, parity, and value gates.
- [x] 4.2g Promote payload enum `match` with `_` catch-all lowering through
  public front, parity, and value gates.
- [x] 4.3a Promote exclusive `..` and inclusive `..=` range `for` loops through
  public front, native direct, full self-host, and parity gates.
- [x] 4.3b Decide `break` and `continue` semantics and lower them through both
  full and direct paths where claimed.
- [x] 4.4 Expand collections with `Map`, `Option`, and `Result` only after syntax,
  ABI, and diagnostics are specified.
  - [x] Promote the first `Option<Int>` `Some`/`None` helper-return and
    statement-match slice.
  - [x] Promote the first `Result<Int,Int>` `Ok`/`Err` helper-return and
    statement-match slice.
  - [x] Promote `Option<Int>` expression-form match binding.
  - [x] Promote `Result<Int,Int>` expression-form match binding.
  - [x] Promote `Option<Int>` local-binding `?` propagation for both success
    and `None` paths.
  - [x] Promote `Result<Int,Int>` local-binding `?` propagation for both
    success and error paths.
  - [x] Promote local `Map<Int,Int>.get_opt(key) -> Option<Int>` on the full
    compiler path and native direct engine.
  - [x] Promote local `Map<Int,Int>` assignment copy on the full compiler path
    and native direct engine.
  - [x] Promote local `Map<Int,Bool>` construction, assignment copy, `insert`,
    `get(key, default)`, `contains`, and `len` on the full compiler path and
    native direct engine.
  - [x] Promote local `Map<Int,Char>` construction, assignment copy, `insert`,
    `get(key, default)`, `contains`, and `len` on the full compiler path and
    native direct engine.
  - [x] Promote `Map<Int,Int>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Int,Bool>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Int,Char>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Int,Int>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote `Map<Int,Bool>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote `Map<Int,Char>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote `remove(key)` for concrete `Map<Int,Int>`, `Map<Int,Bool>`, and
    `Map<Int,Char>` values on the full compiler path and native direct engine.
  - [x] Promote `get_opt(key)` for `Map<Int,Bool>` and `Map<Int,Char>` match
    payloads on the full compiler path and native direct engine.
  - [x] Promote local `Map<Str,Int>` construction, assignment copy,
    `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
    `contains`, and `len` on the full compiler path and native direct engine.
  - [x] Promote `Map<Str,Int>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Str,Int>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote local `Map<Str,Bool>` construction, assignment copy,
    `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
    `contains`, and `len` on the full compiler path and native direct engine.
  - [x] Promote `Map<Str,Bool>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Str,Bool>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote local `Map<Str,Char>` construction, assignment copy,
    `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
    `contains`, and `len` on the full compiler path and native direct engine.
  - [x] Promote `Map<Str,Char>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Str,Char>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote concrete Map parameter-source and parameter-target assignment
    copies on the full compiler path and native direct engine.
  - [x] Promote concrete Map-returning call assignment copies on the full
    compiler path and native direct engine.
  - [x] Add argument-bearing Map-returning call assignment coverage on the full
    compiler path and native direct engine.
  - [x] Add all-concrete Map parameter-target assignment coverage on the full
    compiler path and native direct engine.
  - [x] Gate unsupported `Option`/`Result` generic forms with front diagnostics.
- [x] 4.5 Keep unsupported syntax behind `scripts/vais-check` and front-contract
  diagnostics until promoted.
  - [x] Add checker guidance for Rust-style top-level `use` and `pub` forms.
  - [x] Add front diagnostics for unverified Map function parameters and return
    values.
  - [x] Add front and direct diagnostics for unverified `Map<Int,Int>` value
    assignment.
  - [x] Add front and direct diagnostics for unverified non-`Map<Int,Int>`
    `get_opt` until matching `Option` payload slices are verified.

Done: `docs/reference/LANGUAGE.md` describes a coherent v1 surface, and every
listed feature has examples plus compiler gates.

### Phase 5: Self-Host Expansion

Goal: make the self-host compiler own more of the actual compiler behavior over
time.

- [x] 5.1 Keep `compiler/self/fixpoint_full.vais` and `vaisc_core.ll`
  regeneration green after each language expansion.
- [x] 5.2 Move front-contract validation that belongs to the compiler into
  self-host Vais code once the language can express it cleanly.
  - [x] Move invalid static import path checking into the Vais-authored checker
    contract while keeping the public compiler front diagnostic aligned.
  - [x] Move unsupported `Option<T>` generic-surface checking into the
    Vais-authored checker contract while keeping verified `Option<Int>` clean.
  - [x] Move unsupported `Result<T,E>` generic-surface checking into the
    Vais-authored checker contract while keeping verified `Result<Int,Int>`
    clean.
  - [x] Move unsupported `Map<K,V>` generic-surface checking into the
    Vais-authored checker contract while keeping verified concrete Map shapes
    clean.
  - [x] Move missing helper return-type checking into the Vais-authored checker
    contract while keeping function-type values clean.
  - [x] Move invalid `main` entrypoint signature checking into the
    Vais-authored checker contract while keeping function-type values clean.
  - [x] Classify the remaining front-contract rejects: closure/enum/match
    subset rejects are native-front-only limits for already verified full
    language features, while manifest/import graph/source-path diagnostics stay
    in the explicit host/driver boundary tracked by 5.3.
- [x] 5.3 Move more diagnostics and source preparation out of the host driver while
  keeping OS-facing file/process work behind explicit host APIs.
  - [x] Add a Vais-authored package manifest contract checker for the current
    manifest diagnostics: missing required keys, unsafe `source`, unsupported
    keys/sections, invalid entries, unsafe dependency paths, missing dependency
    manifests, duplicate keys/aliases, missing source directories, and local
    dependency cycles.
  - [x] Add optional entry-path source-root containment checking to the
    Vais-authored package manifest checker for the native
    `package entry is outside manifest source root` diagnostic.
  - [x] Add a Vais-authored local import graph contract checker for the current
    manifest-free missing import, duplicate top-level symbol, and import cycle
    diagnostics.
  - [x] Extend the Vais-authored local import graph checker to follow the first
    package manifest local dependency alias and dependency-internal plain
    imports.
  - [x] Extend the Vais-authored local import graph checker to follow all
    declared package manifest local dependency aliases from the entry package.
  - [x] Wire `scripts/vaisc` to run cached Vais-authored package manifest and
    import graph preflight tools before native `emit-ir`, `build`, and `run`.
- [x] 5.4 Add stage comparison gates for self-host output where deterministic IR
  is practical.

Done: the compiler can rebuild its checked-in core from Vais source, and the
native host driver is limited to CLI, OS integration, and linking duties.

### Phase 6: Stable v1 Release

Goal: publish a coherent first stable Vais release.

- [x] 6.1 Freeze the v1 language reference and prelude reference.
- [x] 6.2 Cut a release candidate tag and attach verified standalone archives.
- [x] 6.3 Run all release, direct/full, install/package, website, and self-host
  gates from a clean checkout.
- [x] 6.4 Publish final docs/site copy from repository canonical docs.
- [x] 6.5 Cut the final v1 tag and verify the GitHub Release assets and
  `vaislang.dev` content.

Status: `v0.3.2` was the gate-backed release-candidate tag. The stable release
line uses `v1.0.1` because public tag `v1.0.0` already points at older commit
`33dfc6ab` and must not be moved. The `v1.0.1` release commit carries the same
verified language surface plus stable-version docs/site copy. The `v1.0.1`
GitHub Release has Linux x64, macOS arm64, and macOS x64 standalone archives,
and the live `vaislang.dev` homepage links the current stable release.

Done: users can install `vaisc`, read the v1 docs, compile the gate-backed
examples, and reproduce the release archive from source.

### Execution Rules

- Work phase order is dependency order. Do not jump to later public claims unless
  their gates and docs are also updated.
- Each milestone must update `ROADMAP.md`, `CHANGELOG.md`, canonical docs, and
  website copy when public behavior changes.
- Direct engine growth is valuable, but the full self-host path remains the
  language authority unless a direct slice is explicitly promoted.
- Host-tool reduction is not an isolated cleanup task; it depends on
  file/process support and Vais-backed replacement tools.
- Release tags are public state. Create or move tags only as a deliberate
  release milestone.

### Current First Executable Milestone

The current concrete slice moves the Vais checker from a ported rule slice to a
public command protected by its own fixture contract:

- [x] Add a release checklist document and wire it to the current gate commands.
- [x] Confirm the release archive workflow publishes archives for a chosen tag.
- [x] Decide the next release version before creating any public tag.
- [x] Promote the first small standard-library `List<T>` API slice with gates.
- [x] Promote the next `List<T>` API slice, `pop()`, with full/direct/docs
  coverage.
- [x] Define the next `List<T>` behavior slice: empty-list and out-of-range
  runtime trap behavior.
- [x] Promote the next Phase 1 slice: `Str` length/index/equality helpers and
  byte-classification utilities needed by real tools.
- [x] Decide and promote the named integer parsing prelude API.
- [x] Specify the minimal `Map<Int,Int>` design and gate unsupported `Map` use.
- [x] Promote native direct local `Map<Int,Int>` construction and local
  operations.
- [x] Promote the next Phase 1 slice: full self-host local `Map<Int,Int>`.
- [x] Promote the next concrete local Map slice: `Map<Int,Bool>`.
- [x] Promote the next concrete local Map slice: `Map<Int,Char>`.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Int>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Bool>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Char>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Int>` return values.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Bool>` return values.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Char>` return values.
- [x] Promote the next concrete Map method slice: `remove(key)` for concrete
  `Map<Int,V>` values.
- [x] Promote the next concrete Map Option slice: `get_opt(key)` for
  `Map<Int,Bool>` and `Map<Int,Char>` match payloads.
- [x] Promote the next concrete Map method slice: `clear()` for concrete
  `Map<Int,V>` values.
- [x] Promote the next concrete local Map key slice: `Map<Str,Int>`.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Int>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Int>` return values.
- [x] Promote the next concrete local Map value slice: `Map<Str,Bool>`.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Bool>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Bool>` return values.
- [x] Promote the next concrete local Map value slice: `Map<Str,Char>`.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Char>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Char>` return values.
- [x] Add release-corpus examples for the promoted prelude API surface.
- [x] Specify the next Phase 1 slice: Map ABI/generic expansion or defer to the
  Phase 2 module model.
- [x] Specify the minimal Phase 2 module/import/package model and reject
  unimplemented module syntax with public front diagnostics.
- [x] Implement single-package multi-file compilation for `scripts/vaisc`.
- [x] Add local import support with missing-import, duplicate-symbol, and
  import-cycle diagnostics.
- [x] Add the minimal package manifest slice.
- [x] Add local package dependency paths.
- [x] Specify the minimal Phase 3 file/process API needed for repository
  validation tools.
- [x] Implement the first native-driver host I/O intrinsic smoke gate.
- [x] Extend the host runtime beyond `fs_exists` to text writes and directory
  creation.
- [x] Extend host support to text reads.
- [x] Extend host support to path helpers.
- [x] Extend host support to argv-based process execution.
- [x] Port the smallest checker slice to Vais.
- [x] Expand the Vais checker slice to the current public checker fixture
  catalog.
- [x] Add line/column-aware Vais checker diagnostics.
- [x] Add a Vais-backed checker CLI path that can receive a target file path,
  return a normal issue/no-issue status, and remain gated by fixture contracts.
- [x] Promote the Vais checker CLI to the public `scripts/vais-check` command
  and package it as a standalone `bin/vais-check` binary.
- [x] Keep public-facing docs and release gates on the Vais-authored checker.
- [x] Add minimal host-backed `Str` construction helpers for future Vais tool
  ports.
- [x] Add full-engine `Str` reassignment and user-defined `-> Str` returns.
- [x] Build the parity manifest and value-corpus validators in Vais so release
  gates depend on Vais-native harnesses.

## Completed Milestone: Phase 3 Host API Specification

Mode: sequential

- [x] 1. Define the boundary between host-backed standard library intrinsics and
  pure compiler-core logic.
- [x] 2. Specify text file APIs for existence checks, whole-file reads,
  whole-file writes, and directory creation.
- [x] 3. Specify path helpers for current directory, temporary directory, joins,
  basenames, and dirnames.
- [x] 4. Specify argv-based process execution and captured process output without
  shell expansion.
- [x] 5. Mark the broad APIs as specified in canonical docs and identify the
  first checker port target.

## Completed Milestone: Local Dependency Package Paths

Mode: sequential

- [x] 1. Parse optional `vais.toml` `[dependencies]` string entries.
- [x] 2. Resolve dependency aliases to local package source roots with their own
  `vais.toml` manifests.
- [x] 3. Resolve dependency-internal plain imports under the dependency package
  source root.
- [x] 4. Reject missing dependency manifests, unsafe dependency paths, and
  dependency cycles with P4 diagnostics.
- [x] 5. Add dependency examples, canonical docs, website copy, and
  front-contract gates for native paths.

## Completed Milestone: Package Manifest Source Roots

Mode: sequential

- [x] 1. Search for nearest `vais.toml` from the entry file directory upward.
- [x] 2. Parse required `name`, `version`, and `source` string keys.
- [x] 3. Resolve static dotted imports under the manifest source root.
- [x] 4. Reject missing keys, unsafe source paths, missing source directories,
  and entries outside the source root with P4 diagnostics.
- [x] 5. Add package examples, canonical docs, website copy, and front-contract
  gates for native paths.

## Completed Milestone: Single-Package Local Imports

Mode: sequential

- [x] 1. Resolve static dotted `import` paths under the entry file directory.
- [x] 2. Merge imported modules before the entry source for full-engine builds.
- [x] 3. Resolve static dotted imports before direct-engine lowering.
- [x] 4. Reject missing imports, duplicate top-level symbols, and import cycles
  with P4 diagnostics.
- [x] 5. Add a multi-file example and front-contract gates for native paths.

## Completed Milestone: Minimal Module Model Specification

Mode: sequential

- [x] 1. Specify file-derived module names, local dotted import paths, symbol
  visibility, duplicate-name diagnostics, and cycle behavior.
- [x] 2. Keep `Map<K,V>` generic/ABI expansion deferred until its lowering and
  ABI are specified separately.
- [x] 3. Add front diagnostics for reserved `module` and `package` syntax and
  use the spec as the import implementation contract.
- [x] 4. Sync canonical docs, website copy, roadmap, worklog, and changelog.

## Completed Milestone: Prelude API Value Examples

Mode: sequential

- [x] 1. Replace stale Map example syntax with the verified local
  `Map<Int,Int>` API.
- [x] 2. Add a release-corpus List example for `is_empty()`, `last()`, and
  `pop()`.
- [x] 3. Promote both examples in `tools/vaisc-parity.tsv`.
- [x] 4. Keep the examples README and roadmap aligned with the value corpus.

## Completed Milestone: Local Map Slices

Mode: sequential

- [x] 1. Parse `Map<Int,Int>` local annotations in the direct engine.
- [x] 2. Lower `let m: Map<Int,Int> = {}` to a native local map value.
- [x] 3. Lower `m.insert(key, value)` statements with replace-on-existing-key
  behavior.
- [x] 4. Lower `m.get(key, default)`, `m.get_opt(key)`, `m.contains(key)`, and
  `m.len()` expressions.
- [x] 5. Gate direct emitted helper symbols and runtime value behavior.
- [x] 6. Lower the same local surface in the full self-host compiler and
  regenerate the reusable compiler core.
- [x] 7. Keep front diagnostics explicit about verified concrete Map slices;
  non-`Map<Int,Int>` returns and generic key/value forms stay
  rejected.
- [x] 8. Promote local `Map<Int,Int>` assignment copy while keeping Map
  returns and generic key/value forms rejected.

### Task Briefs

#### 1. Full self-host Map<Int,Int> lowering

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: match the direct local Map surface without adding generic or
  ABI claims; regenerate the reusable compiler core after the source change.
- Done: full self-host gates pass a local `Map<Int,Int>` example returning
  the same deterministic value as the direct gate.

#### 2. Map ABI and generic expansion

- Target files: `tools/vaisc_native.c`, `compiler/self/fixpoint_full.vais`,
  `docs/reference/LANGUAGE.md`, `std/PRELUDE.md`.
- Requirements: specify and gate Map parameters, return values, generic
  key/value support, and any broader `Option`/`Result` integration before
  publishing broader claims.
- Status: `docs/design/MAP_ABI.md` now specifies ownership, assignment,
  parameter, return, monomorphic helper, and expansion-order rules. Local
  `Map<Int,Int>` assignment copy and the local `Map<Int,Bool>` and
  `Map<Int,Char>` scalar-value slices are verified. `Map<Str,Int>` is
  verified for string-key local operations, parameter reference mutation, and
  return-value local initialization. Local `Map<Str,Bool>` string-key
  operations, parameter reference mutation, and return-value local
  initialization are verified. Local `Map<Str,Char>` string-key operations,
  parameter reference mutation, and return-value local initialization are
  verified. Concrete Map parameter-source, parameter-target, and Map-returning
  call assignment copies are verified for the promoted Map types, including
  no-argument and argument-bearing return calls; broader `Map<Str,V>` and
  generic Map behavior still require direct and full gates before publication.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, and `Map<Str,Int>`
  parameter reference mutation is verified.

## Completed Milestone: Map ABI and Generic Expansion Specification

Mode: sequential

- [x] 1. Keep Map expansion limited to explicitly verified concrete local
  slices.
- [x] 2. Specify Map assignment as value-copy instead of aliasing.
- [x] 3. Specify Map parameter mutation as reference-based, matching collection
  parameter behavior.
- [x] 4. Specify Map returns through caller-owned output storage or equivalent
  direct-engine lowering.
- [x] 5. Define monomorphic concrete helper families as the path for future
  `Map<K,V>` slices.
- [x] 6. Keep broader Map forms behind front/direct diagnostics until each slice
  has full gates.

## Completed Milestone: Map design and front gate contract

Mode: sequential

- [x] 1. Keep `Map<K,V>` out of the verified surface until compiler gates cover
  it.
- [x] 2. Define the first implementation target as `Map<Int,Int>` only.
- [x] 3. Choose explicit-empty construction with `let m: Map<Int,Int> = {}`.
- [x] 4. Choose `insert(key, value)` for insert/replace, `get(key, default)` for
  lookup without `Option`, `contains(key)` for presence, and `len()` for
  cardinality.
- [x] 5. Add front-gate diagnostics so unsupported public `Map` use fails
  clearly outside the verified local `Map<Int,Int>` slice.

### Task Briefs

#### 1. Concrete local Map implementation slices

- Target files: `tools/vaisc_native.c`.
- Requirements: local `Map<Int,Int>` values support `{}`, assignment copy,
  `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and
  `len`; local `Map<Int,Bool>` values support `{}`, assignment copy, `insert`,
  `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and
  `len`; local `Map<Int,Char>` values support the same surface without
  publishing broader generic Map return-value ABI claims. Local `Map<Str,Int>`
  values support the same local method surface with string keys and
  return-value local initialization. Local `Map<Str,Bool>` values support the
  same local method surface with string keys, parameter reference mutation, and
  return-value local initialization. Local `Map<Str,Char>` values support the
  same local method surface with string keys and parameter reference mutation
  and return-value local initialization while keeping broader `Map<Str,V>` and
  generic Map returns gated.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`,
  `Map<Str,Int>`, `Map<Str,Bool>`, and `Map<Str,Char>` parameters are passed by
  reference and may be mutated by callees. Same-type assignment copies are
  verified for local sources, parameter sources/targets, and Map-returning calls.
- Done: native direct gates pass local, parameter, return, assignment, and
  Map-returning call examples returning deterministic values, and full
  self-host gates pass the same Map behavior.

#### 2. Map docs and release claims

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `scripts/test-vaisc-front.sh`, `website/index.html`.
- Requirements: docs distinguish verified local concrete Map slices from
  unsupported generic and ABI Map behavior.
- Done: `scripts/test-vaisc-front.sh` accepts local `Map<Int,Int>`,
  `Map<Int,Bool>`, `Map<Int,Char>`, local `Map<Str,Int>`, local
  `Map<Str,Bool>`, local `Map<Str,Char>`,
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>`, and `Map<Str,Char>`
  parameters while rejecting unsupported generic `Map<K,V>` forms;
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>`, and `Map<Str,Char>` return values are accepted only for the
  concrete gate-backed slices; docs/site do not imply a verified generic
  `Map<K,V>`.

## Completed Milestone: Named integer parsing prelude helpers

Mode: sequential

- [x] 1. Define `parse_uint(s: Str) -> Int` as leading unsigned decimal parsing
  that stops at the first non-decimal byte and returns `0` for empty/no-digit
  input.
- [x] 2. Define `parse_int(s: Str) -> Int` as optional leading `-` plus the same
  decimal parsing behavior.
- [x] 3. Lower both helpers through the full self-host compiler and regenerate
  `compiler/self/vaisc_core.ll`.
- [x] 4. Lower both helpers through the native direct engine.
- [x] 5. Add front, direct, full self-host, parity, and value gates with
  `examples/e83_parse_helpers.vais`.
- [x] 6. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, examples index, and website copy.

### Task Briefs

#### 1. Full and direct compiler support

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `tools/vaisc_native.c`.
- Requirements: `parse_uint` and `parse_int` are named prelude helpers, not
  user-defined example helpers; the full path must emit reusable helper IR and
  the direct path must stay native-only.
- Done: full codegen emits `@__vais_parse_uint` and `@__vais_parse_int`; direct
  mode rewrites calls to native helpers and verifies `Str` arguments.

#### 2. Gates and public docs

- Target files: `scripts/test-fixpoint-full.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-direct.sh`,
  `tools/vaisc-parity.tsv`, `examples/e83_parse_helpers.vais`,
  `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`, `website/index.html`.
- Requirements: the API is public only when examples and release gates protect
  both full and direct behavior.
- Done: the named helpers are covered by full, front, direct, parity, and value
  tests.

## Completed Milestone: Str tool-helper slice

Mode: sequential

- [x] 1. Allow public front-contract scalar helper signatures with `Int`,
  `Bool`, and `Str`.
- [x] 2. Lower native direct `Str` literals, locals, parameters, return values,
  `s.len()`, `s[i]`, `a == b`, and `a != b`.
- [x] 3. Gate `Bool` byte-classification helpers and user-defined integer
  parsing over `Str`.
- [x] 4. Promote `examples/e44_string_len.vais`,
  `examples/e48_string_index.vais`, `examples/e53_word_count.vais`,
  `examples/e69_palindrome_string.vais`, `examples/e70_parse_uint.vais`,
  `examples/e71_string_index_of.vais`, and `examples/e72_identifier_scan.vais`
  in the parity manifest.
- [x] 5. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Front and direct scalar surface

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-front.sh`,
  `scripts/test-vaisc-direct.sh`.
- Requirements: keep `fn main() -> Int`, but allow helper signatures and locals
  for `Int`, `Bool`, and `Str`; direct mode must stay native-only.
- Done: front and direct gates cover `Str` params/locals, `Bool` classifier
  helpers, and native direct lowering.

#### 2. String operations and tool patterns

- Target files: `tools/vaisc_native.c`, `tools/vaisc-parity.tsv`,
  `examples/e44_string_len.vais`, `examples/e48_string_index.vais`,
  `examples/e53_word_count.vais`, `examples/e69_palindrome_string.vais`,
  `examples/e70_parse_uint.vais`, `examples/e71_string_index_of.vais`,
  `examples/e72_identifier_scan.vais`.
- Requirements: protect `s.len()`, `s[i]`, string equality/inequality,
  byte-classification helpers, parse/identifier-scan tool shapes, and
  computed byte-index word-count, substring, and palindrome searches.
- Done: direct and parity gates cover string index, string equality,
  `Str(Int)` conversion, parse_uint, word-count scans, substring search,
  palindrome scans, and identifier scanning.

#### 3. Documentation and roadmap sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: document the promoted `Str` helper surface as gate-backed while
  leaving any named integer parsing prelude API as a follow-up decision.

## Completed Milestone: List bounds trap behavior

Mode: sequential

- [x] 1. Add full self-host runtime trap lowering for invalid `List` index
  reads/writes, `last()` on an empty list, and `pop()` on an empty list.
- [x] 2. Add native direct checked-index helpers for `List<Int>` and
  `List<Struct>` reads/writes plus checked `last()` and `pop()`.
- [x] 3. Gate trap behavior with full self-host and native direct invalid-list
  access tests.
- [x] 4. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Full compiler bounds traps

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: emit `llvm.trap` before out-of-range list GEPs and before
  empty-list `last()`/`pop()` length mutation.
- Done: full gates cover invalid scalar list index, empty scalar `last()`,
  empty scalar `pop()`, and empty struct-list `last()`.

#### 2. Native direct bounds traps

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`.
- Requirements: keep direct mode native-only, avoid double-evaluating index
  expressions, and check `pop()` before length mutation.
- Done: direct gates cover invalid `List<Int>` index, empty `last()`, and empty
  `pop()`.

#### 3. Documentation and gate sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: document trap behavior as the current release-surface contract,
  not as future work.

## Completed Milestone: List pop API

Mode: sequential

- [x] 1. Add `List<T>.pop()` lowering to the full self-host compiler for
  non-empty scalar lists and struct-list local binding.
- [x] 2. Add native direct `List<Int>` and `List<Struct>` `pop()` expression
  support with type inference and deterministic prelude temporaries.
- [x] 3. Gate local and parameter `List<Int>.pop()` plus struct-list
  `let item = xs.pop()` usage, including caller-visible length mutation.
- [x] 4. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Full compiler pop API

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: compile `xs.pop()` by reading `len - 1`, returning that element,
  and storing the decremented length for local and parameter lists.
- Done: full gates cover `List<Int>.pop()` through a `List<Int>` parameter and
  `List<Tok>.pop()` through local and parameter struct-list bindings.

#### 2. Native direct pop API

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: keep direct mode native-only, infer `xs.pop()` as the list
  element type, and sequence mutation through generated temporaries.
- Done: direct gates cover `List<Int>.pop()` locals and parameters plus
  `List<Box>.pop()` binding.

#### 3. Documentation and gate sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements at that milestone: document only the non-empty-list API.
  Bounds behavior is now covered by the completed List bounds trap behavior
  milestone above.

## Completed Milestone: List last API

Mode: sequential

- [x] 1. Add `List<T>.last()` lowering to the full self-host compiler for
  non-empty scalar lists and struct-list local binding.
- [x] 2. Add native direct `List<Int>` and `List<Struct>` `last()` expression
  support with type inference.
- [x] 3. Gate local and parameter `List<Int>.last()` plus struct-list
  `let item = xs.last()` usage.
- [x] 4. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Full compiler last API

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: compile `xs.last()` by reading `len - 1` and reusing existing
  list buffer/index lowering; support struct-list values by binding the result
  to a local before field reads.
- Done: full gates cover `List<Int>.last()` through a `List<Int>` parameter and
  `List<Tok>.last()` through local and parameter struct-list bindings.

#### 2. Native direct last API

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: keep direct mode native-only, infer `xs.last()` as the list
  element type, and reject malformed calls in the rewrite path.
- Done: direct gates cover `List<Int>.last()` locals and parameters plus
  `List<Box>.last()` binding.

#### 3. Documentation and gate sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements at that milestone: document only the non-empty-list API. `pop()`
  is now covered by the completed List pop API milestone above, and bounds
  behavior is covered by the completed List bounds trap behavior milestone.

## Completed Milestone: List is_empty API

Mode: sequential

- [x] 1. Add `List<T>.is_empty()` lowering to the full self-host compiler.
- [x] 2. Regenerate `compiler/self/vaisc_core.ll` from
  `compiler/self/fixpoint_full.vais`.
- [x] 3. Add native direct `List<Int>` and `List<Struct>` `is_empty()` support.
- [x] 4. Gate the API in full, front, direct, and diagnostic test suites.
- [x] 5. Sync `std/PRELUDE.md`, the language reference, and website copy.

### Task Briefs

#### 1. Full compiler list API

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: compile `xs.is_empty()` for local and parameter lists without
  relying on a broad method fallback.
- Done: full gates cover `List<Int>.is_empty()` and declared-struct
  `List<T>.is_empty()` returning the expected boolean-as-Int values.

#### 2. Native direct list API

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: keep public direct mode native-only and reject malformed
  `is_empty` calls with explicit diagnostics.
- Done: direct gates cover local Int and struct lists, and front/error gates
  document the promoted method surface.

#### 3. Documentation and release gate

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs name only the gate-backed API and leave remaining
  list work as roadmap items.
- Done: `bash scripts/test-release-gates.sh` passed after the compiler/core
  changes.

## Completed Milestone: v0.2.2 Source Release

Mode: sequential

- [x] 1. Promote release metadata to a dated `v0.2.2` changelog entry.
- [x] 2. Run the full pre-tag release gate and produce the local standalone
  archive.
- [x] 3. Push the annotated `v0.2.2` source tag and verify the release archive
  workflow.
- [x] 4. Verify the GitHub Pages deploy and live `vaislang.dev` release copy.

### Task Briefs

#### 1. Release metadata

- Target files: `tools/vaisc_native.c`, `CHANGELOG.md`,
  `docs/release/RELEASE_CHECKLIST.md`, `website/package.json`,
  `website/package-lock.json`.
- Requirements: make the native compiler, changelog, release checklist, and
  website package agree on the `v0.2.2` source release line.
- Done: `scripts/vaisc --version` reports `0.2.2` through the native driver, and
  the changelog records `v0.2.2 - 2026-06-15`.

#### 2. Release verification

- Target files: `.github/workflows/release-archives.yml`,
  `scripts/test-release-gates.sh`, `website/`.
- Requirements: prove the tag path publishes standalone archives and the live
  website remains synced with the repository release docs.
- Done: `bash scripts/test-release-gates.sh` passed, `v0.2.2` published
  `vais-0.2.2-linux-x64.tar.gz`, `vais-0.2.2-darwin-arm64.tar.gz`, and
  `vais-0.2.2-darwin-x64.tar.gz`, and the `Deploy Website` workflow succeeded
  for commit `5dfb49e3`.

## Completed Milestone: Release Discipline Checklist

Mode: sequential

- [x] 1. Add a full pre-tag release gate script.
- [x] 2. Add a release checklist with version/tag policy and post-tag verification.
- [x] 3. Link release discipline from the first-read docs and changelog.

### Task Briefs

#### 1. Release gate command

- Target files: `scripts/test-release-gates.sh`.
- Requirements: provide one command that runs the release-level gates before a
  public source tag is created.
- Done: `bash scripts/test-release-gates.sh` runs shell syntax checks,
  front/direct/error/parity/value/native/install gates,
  self-host regeneration gates, release archive packaging, website build, and
  `git diff --check`.

#### 2. Release checklist

- Target files: `docs/release/RELEASE_CHECKLIST.md`, `README.md`,
  `docs/README.md`, `CHANGELOG.md`, `ROADMAP.md`.
- Requirements: document the next planned release line, tag policy, pre-tag
  checks, manual archive workflow trigger, and post-tag verification.
- Done: the current source release is `v1.0.1`, the next planned source
  release is `v1.0.2`, and tag creation is explicitly deferred until release
  gates are green.

## Completed Milestone: Native Direct List Else-If Condition Arguments

Mode: sequential

- [x] 1. Lower returned `List<Int>` and `List<Struct>` helper calls in `else if` conditions.
- [x] 2. Gate direct `else if score(make(...))` behavior for both integer and struct lists.
- [x] 3. Sync docs/site/changelog with the promoted condition-argument slice.

### Task Briefs

#### 1. Direct else-if returned-list argument lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts `else if` conditions such as
  `} else if score(make(20)) == 41 {` when `make` returns a list and `score`
  receives the matching `List<T>` parameter.
- Done: returned-list call arguments can lower as C compound-literal list
  temporaries in expression contexts that cannot receive a statement prelude,
  preserving `else if` evaluation order without rewriting the control-flow
  shape.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `docs/reference/LANGUAGE.md`,
  `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: prove direct `List<Int>` and `List<Struct>` returned-list
  arguments execute inside `else if` conditions and keep public docs precise
  about the promoted scope.
- Done: direct gate covers `score_int(make_int(...))` and
  `score_box(make_box(...))` inside `else if` conditions returning 42.

## Completed Milestone: Native Direct List If-Condition Hoisting

Mode: sequential

- [x] 1. Hoist returned `List<Int>` and `List<Struct>` helper calls in plain `if` conditions.
- [x] 2. Gate direct `if score(make(...))` behavior for both integer and struct lists.
- [x] 3. Sync docs/site/changelog with the promoted condition-hoisting slice.

### Task Briefs

#### 1. Direct plain-if returned-list argument hoisting

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts plain `if` conditions such as
  `if score(make(20)) == 41 {` when `make` returns a list and `score` receives
  the matching `List<T>` parameter.
- Done: direct `if` lowering now attaches the existing list-argument prelude
  before the generated C `if`, so returned-list temporaries are available to
  the condition expression.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `docs/reference/LANGUAGE.md`,
  `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: prove direct `List<Int>` and `List<Struct>` returned-list
  arguments execute inside plain `if` conditions and keep public docs precise
  about the promoted scope.
- Done: direct gate covers `score_int(make_int(...))` and
  `score_box(make_box(...))` inside plain `if` conditions returning 42.

## Completed Milestone: Native Direct List Element Assignment

Mode: sequential

- [x] 1. Parse `List` indexed element assignment targets.
- [x] 2. Infer `xs[index]` expression types from the list element type.
- [x] 3. Gate `List<Int>` and `List<Struct>` element assignment locally and through parameters.
- [x] 4. Sync docs/site/changelog with the promoted element-assignment slice.

### Task Briefs

#### 1. Indexed list element assignment

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts assignments such as `xs[0] = 42`,
  `boxes[0] = Box { value: 42 }`, and `boxes[1] = boxes[0]` when the value
  matches the list element type.
- Done: assignment target validation now recognizes `base[index]`, target type
  lookup returns the list element type, and exact list-index expressions infer
  to their element type.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove local and parameter element assignments execute through
  `scripts/vaisc --engine direct` for both `List<Int>` and `List<Struct>`, and
  keep non-list indexed assignment targets behind a P4 diagnostic.
- Done: direct gate covers `List<Box>` element literal assignment, element copy,
  parameter element replacement, and `List<Int>` element assignment returning
  42; error gate covers a non-list indexed assignment target.

## Completed Milestone: Native Direct List Struct Field Assignment

Mode: sequential

- [x] 1. Parse `List<Struct>` indexed field assignment targets.
- [x] 2. Type-check indexed struct-list field assignments as `Int` field writes.
- [x] 3. Gate local and parameter `xs[index].field = value` behavior.
- [x] 4. Sync docs/site/changelog with the promoted field-write slice.

### Task Briefs

#### 1. Indexed List<Struct> field assignment

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts assignments such as `xs[0].value = 42` and
  `xs[i].value = value` when `xs` is a `List<DeclaredStruct>` and `value` is a
  declared `Int` field.
- Done: assignment target validation now recognizes `base[index].field`, checks
  the list element struct field, and rewrites the left-hand side through the
  existing list-index expression lowering.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove local and parameter `List<Struct>` indexed field writes
  execute through `scripts/vaisc --engine direct` and keep unknown element
  fields behind a P4 diagnostic.
- Done: direct gate covers local and parameter `List<Box>` field writes
  returning 42; error gate covers an unknown indexed field target.

## Completed Milestone: Native Direct List Assignment

Mode: sequential

- [x] 1. Make direct list assignment context-typed for `List<Int>` and `List<Struct>`.
- [x] 2. Support assigning `[]`, `list()`, list literals, local lists, and returned lists to matching list locals and list parameters.
- [x] 3. Gate caller-visible replacement through list parameter assignment.
- [x] 4. Sync docs/site/changelog with the promoted assignment slice.

### Task Briefs

#### 1. Context-typed list assignment

- Target files: `tools/vaisc_native.c`.
- Requirements: direct assignment to a list target should validate list
  literals using the target element type instead of inferring bare list
  literals as `List<Int>`.
- Done: assignment lowering now treats list initializer expressions as
  context-typed when the target is `List<T>`, then rewrites the value with the
  target list type.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `docs/reference/LANGUAGE.md`,
  `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: prove direct `List<Int>` and `List<Struct>` assignment runs,
  including assignment through a `List<Struct>` parameter that replaces the
  caller's list.
- Done: direct gate covers `List<Box>` local assignment from `[]`, `list()`,
  literals, returned lists, parameter replacement, and `List<Int>` literal
  assignment returning 42.

## Completed Milestone: Native Direct List Struct ABI

Mode: sequential

- [x] 1. Accept `List<Struct>` in direct function parameter and return types.
- [x] 2. Lower `List<Struct>` parameters as native references and return values by value.
- [x] 3. Gate inline `List<Struct>` arguments and returned-list argument hoisting.
- [x] 4. Sync docs/site/changelog with the promoted struct-list ABI.

### Task Briefs

#### 1. Direct List<Struct> ABI lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can pass local `List<DeclaredStruct>` values to
  helpers by reference, return `List<DeclaredStruct>` values by value, lower
  inline struct-list literals, and hoist `List<Struct>`-returning helper calls
  before passing them to `List<Struct>` parameters.
- Done: direct lowering now uses `DirectList_<Struct> *` for list parameters,
  `DirectList_<Struct>` for returns and temporaries, and context-typed list
  literals for `List<Struct>`.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove `List<Struct>` parameters, return values, inline
  arguments, returned-list argument hoisting, and while-condition hoisting run
  through `scripts/vaisc --engine direct`.
- Done: direct gate covers `List<Box>` parameter mutation, return-by-value,
  inline arguments, returned-list arguments, and while-condition hoisting
  returning 42.

## Completed Milestone: Native Direct Local List Struct Slice

Mode: sequential

- [x] 1. Parse and validate direct-engine local `List<Struct>` types.
- [x] 2. Lower local `List<Struct>` storage, `[]`, `list()`, literals, `push`, `len`, index, and field reads.
- [x] 3. Gate the promoted slice and leave `List<Struct>` function ABI to the following milestone.
- [x] 4. Sync docs/site/changelog with the promoted local struct-list slice.

### Task Briefs

#### 1. Direct local List<Struct> lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts local `List<DeclaredStruct>` values through
  the native direct engine; `List<Struct>` function parameter/return ABI is
  handled by the following milestone.
- Done: direct lowering emits `DirectList_<Struct>` locals for typed `[]`,
  `list()`, and small struct list literals, lowers `push`, `len`/`len()`, index
  reads, and field reads such as `xs[0].value`.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove the local `List<Struct>` slice emits LLVM IR and runs
  through `scripts/vaisc --engine direct`, with function ABI left for the next
  promoted slice.
- Done: direct gate covers local `List<Box>` push, length, index, and field-read
  behavior returning 42.

## Completed Milestone: Release Automation And Native Direct Int Slice

Mode: sequential

- [x] 1. Add release archive workflow for source tags.
- [x] 2. Remove the public direct-engine non-native fallback.
- [x] 3. Expand the native direct engine through Int helper calls, locals, assignment, `if`, `while`, simple struct locals, and struct parameter/return helpers.
- [x] 4. Sync README, language docs, website copy, changelog, and gates.

### Task Briefs

#### 1. Release archive workflow

- Target files: `.github/workflows/release-archives.yml`, `scripts/package-vaisc-release.sh`.
- Requirements: tag builds package standalone compiler/checker archives and
  upload them to the matching GitHub Release.
- Done: workflow builds Linux/macOS archive jobs, smokes packaged `vaisc`, creates the release when needed, and uploads archives.

#### 2. Native direct path

- Target files: `scripts/vaisc`, `tools/vaisc_native.c`.
- Requirements: `--engine direct` must stay on the native driver.
- Done: `scripts/test-vaisc-direct.sh` proves direct mode stays native even
  when an unrelated `python3` shim is first in `PATH`.

#### 3. Direct Int control-flow and struct slice

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: direct mode accepts Int helper functions, locals, assignment, calls, `if`, `while`, returns, simple Int-field struct local literal/read/write, and struct parameter/return helper ABI; unsupported identifiers keep P4 diagnostics.
- Done: direct tests cover arithmetic, helper calls, locals, control flow, struct locals, struct parameter/return helpers, full-engine parity, and P4 errors.

#### 4. Documentation and gates

- Target files: `README.md`, `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `AGENTS.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs describe current native direct and release archive automation without publishing unsupported direct lists or self-host claims.
- Done: docs/site/changelog are synced and release gates pass.

## Completed Milestone: Native Direct Local List Slice

Mode: sequential

- [x] 1. Add native direct local `List<Int>` storage and helper lowering.
- [x] 2. Add direct tests for `[]`, small integer list literals, `push`, `len`, index, and `sum`.
- [x] 3. Sync docs/site/changelog with the promoted direct list slice.

### Task Briefs

#### 1. Direct local List<Int> lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts local `List<Int>` values through the native
  direct engine; function parameter/return list ABI stays out of this slice.
- Done: direct lowering emits `DirectListInt` locals for `[]`, `list()`, and
  small integer list literals, lowers `push`, `len`/`len()`, index reads, and
  `sum()`.

#### 2. Direct list gate

- Target files: `scripts/test-vaisc-direct.sh`.
- Requirements: prove the new list slice emits LLVM IR and runs through
  `scripts/vaisc --engine direct`.
- Done: direct gate covers local list push, length, index, literal, and sum
  behavior returning 42.

#### 3. Documentation sync

- Target files: `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`,
  `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: public docs describe the promoted direct list slice and leave
  list parameters/returns as future work.
- Done: docs and site copy are synced to the current direct/full engine split.

## Completed Milestone: Native Direct List Int Inline Values

Mode: sequential

- [x] 1. Lower inline `List<Int>` literals and `list()` as direct return values.
- [x] 2. Lower inline `List<Int>` literals and `list()` as direct call arguments.
- [x] 3. Gate inline call/return values and preserve non-local argument diagnostics.
- [x] 4. Sync docs/site/changelog with the promoted inline value slice.

### Task Briefs

#### 1. Inline list value lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can lower `return []`, `return [1, 2]`,
  `return list()`, `score([])`, `score([1, 2])`, and `score(list())` for
  `List<Int>` signatures through the native direct engine.
- Done: direct lowering emits `DirectListInt` compound literals for inline list
  return values and passes addresses of inline list compound literals to
  `List<Int>` parameters.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove inline list values execute through direct mode and keep
  list-returning helper calls used directly as list arguments behind a diagnostic.
- Done: direct gates cover inline list call/return values; non-literal
  returned-list arguments were left for the returned-argument hoisting milestone.

## Completed Milestone: Native Direct List Int Returned-Argument Hoisting

Mode: sequential

- [x] 1. Hoist `List<Int>`-returning helper calls used as `List<Int>` arguments.
- [x] 2. Gate nested returned-list arguments across return, let, list literal,
  push, and assignment statements.
- [x] 3. Keep loop-condition returned-list arguments behind a diagnostic.
- [x] 4. Sync docs/site/changelog with the promoted hoisting slice.

### Task Briefs

#### 1. Returned-list argument hoisting

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can lower statement-context calls such as
  `score(make(10))`, `score(pass(make(5)))`, list literal items containing those
  calls, `push(score(make(2)))`, and assignment from those calls.
- Done: direct lowering adds per-function temporary `DirectListInt` locals before
  the current C statement and passes their addresses to `List<Int>` parameters.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove returned-list arguments execute through direct mode and
  document that loop-condition hoisting is still outside the direct claim.
- Done: direct gates cover returned-list argument hoisting in statement contexts;
  while-condition hoisting was left for the following milestone.

## Completed Milestone: Native Direct List Int While Hoisting

Mode: sequential

- [x] 1. Hoist `List<Int>`-returning helper calls inside direct `while`
  conditions.
- [x] 2. Preserve per-iteration condition reevaluation.
- [x] 3. Gate while-condition returned-list argument hoisting.
- [x] 4. Sync docs/site/changelog with the promoted loop-hoisting slice.

### Task Briefs

#### 1. While condition hoisting

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can lower `while score(make(i)) < limit { ... }`
  without evaluating `make(i)` only once before the loop.
- Done: direct lowering emits `while (1)` when condition prelude temporaries are
  required, rebuilds the hoisted `DirectListInt` temporaries each iteration, and
  breaks when the translated condition is false.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove returned-list argument hoisting in direct `while`
  conditions executes through direct mode and keep docs synced to the new claim.
- Done: direct gates cover per-iteration while-condition hoisting returning 42.

## Completed Milestone: Native Direct List Int Out-Param Semantics

Mode: sequential

- [x] 1. Lower `List<Int>` parameters as direct native references.
- [x] 2. Preserve `List<Int>` return values as value returns.
- [x] 3. Gate callee `push` mutation of caller local lists.
- [x] 4. Keep unsupported non-local returned-list arguments covered by diagnostics.
- [x] 5. Sync docs/site/changelog with the promoted out-param slice.

### Task Briefs

#### 1. Direct list parameter references

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode passes named local `List<Int>` arguments to
  `List<Int>` parameters by reference while keeping non-list parameters on their
  existing value ABI.
- Done: direct lowering emits native pointer parameters for `List<Int>`, rewrites
  calls to pass local list addresses, and rewrites parameter `len`, index, `sum`,
  assignment, and `push` operations through the referenced list.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove callee `push` mutates the caller local list and keep
  returned list expressions out of direct list argument claims.
- Done: direct gates cover caller-visible mutation and diagnostics require
  non-literal `List<Int>` arguments to be local list names.

## Completed Milestone: Native Direct List Int ABI

Mode: sequential

- [x] 1. Parse `List<Int>` in direct function headers.
- [x] 2. Lower `List<Int>` parameters and return values through the direct ABI.
- [x] 3. Add direct/error gates for list ABI and type mismatch diagnostics.
- [x] 4. Sync docs/site/changelog with the promoted ABI slice.

### Task Briefs

#### 1. Function header parsing

- Target files: `tools/vaisc_native.c`.
- Requirements: direct function parameter and return annotations may use
  `List<Int>` in addition to `Int` and declared structs.
- Done: direct header parsing and validation accept `List<Int>`.

#### 2. List ABI lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can pass local `List<Int>` values to helpers, return
  local or helper-produced `List<Int>` values, and bind returned list values to
  locals.
- Done: direct lowering handles `List<Int>` helper parameters and return values
  and checks return, local initializer, assignment, and call-argument types
  before C/LLVM.

#### 3. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: gate the promoted ABI and keep unsupported direct list call
  expressions out of public direct claims.
- Done: direct gates cover list parameter/return ABI and diagnostics cover list
  type mismatches and non-local list call arguments.

## Completed Milestone: Standalone Install And Release Archive

Mode: sequential

- [x] 1. Add install and uninstall scripts for standalone `vaisc` and
  `vais-check`.
- [x] 2. Add release archive packaging for the native binaries and first-read
  docs.
- [x] 3. Add an install/package gate that proves installed and packaged
  binaries run.
- [x] 4. Sync docs/site/changelog and run release gates.

### Task Briefs

#### 1. Standalone install and uninstall

- Target files: `scripts/install-vaisc.sh`, `scripts/uninstall-vaisc.sh`.
- Requirements: build the native compiler from the checked-in self-host core and
  install `PREFIX/bin/vaisc` plus the Vais-built checker as
  `PREFIX/bin/vais-check`; uninstall removes those binaries.
- Done: installing into a temporary prefix creates executable `vaisc` and
  `vais-check`, and uninstall removes them.

#### 2. Release archive packaging

- Target files: `scripts/package-vaisc-release.sh`, `.gitignore`.
- Requirements: build a standalone archive containing `bin/vaisc`,
  `bin/vais-check`, and the current first-read docs; keep generated archives out
  of git.
- Done: the package script creates `dist/vais-VERSION-OS-ARCH.tar.gz`.

#### 3. Install/package gate

- Target files: `scripts/test-vaisc-install.sh`, `AGENTS.md`, `README.md`.
- Requirements: verify installed and packaged compiler binaries can report
  version, run `doctor`, and compile/run a `.vais` smoke source, and verify the
  installed and packaged checker accepts/flags fixture sources.
- Done: `bash scripts/test-vaisc-install.sh` passes without writing outside a temporary directory.

#### 4. Documentation, site, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs describe checkout use, standalone install, uninstall, package, and the gate protecting them.
- Done: docs and site are synced, website builds, stale public-claim scan is clean, and release gates pass.

## Completed Milestone: Native Public `vaisc`

Mode: sequential

- [x] 1. Native driver skeleton and build script.
- [x] 2. Release source-preparation parity with the retired prototype path.
- [x] 3. `scripts/vaisc` default switch and install/doctor UX.
- [x] 4. Documentation/site/changelog sync and release gates.

### Task Briefs

#### 1. Native driver skeleton and build script

- Target files: `tools/`, `scripts/`, `README.md`, `ROADMAP.md`.
- Requirements: compile a native `vaisc` binary from a small host driver and `compiler/self/vaisc_core.ll`; support `emit-ir`, `build`, `run`, `--help`, `--version`, and `doctor` for the full engine path.
- Done: a local native binary can compile and run `examples/c4.vais`.

#### 2. Release source-preparation parity

- Target files: native driver/source-prep implementation and existing `scripts/test-vaisc*.sh` gates.
- Requirements: keep the native release source-preparation behavior for
  enum/match, payload enum, closure-return, typed `Int`, `print`, comments, and
  semicolon normalization.
- Done: `bash scripts/test-vaisc.sh`, `bash scripts/test-vaisc-front.sh`, `bash scripts/test-vaisc-errors.sh`, `bash scripts/test-vaisc-parity.sh`, and `bash scripts/test.sh` pass through the native public command.

#### 3. Public command switch and install UX

- Target files: `scripts/vaisc`, packaging/install scripts, README docs.
- Requirements: `scripts/vaisc` uses the native driver by default for normal user `emit-ir`, `build`, and `run`; `doctor` reports missing `clang` or missing native driver clearly.
- Done: a fresh checkout can build the native driver and run `scripts/vaisc doctor`, `scripts/vaisc run examples/c4.vais`, and `scripts/vaisc build examples/c4.vais -o /tmp/c4`.

#### 4. Documentation, release, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `compiler/self/SELF_HOST.md`, `website/`, `CHANGELOG.md`, `WORKLOG.md`.
- Requirements: public docs describe the native command path only after verification.
- Done: verification baseline plus self-host gates pass, the website builds, stale public-claim scan is clean, and GitHub/site release notes are synced.

## Completed Milestone: Vais-Native Self-Host Gate Helpers

Mode: sequential

- [x] 1. Port self-source embedding to `tools/embed_self_source.vais`.
- [x] 2. Port stage IR normalization to `tools/normalize_stage_ir.vais`.
- [x] 3. Move long self-host gates onto the Vais helpers.
- [x] 4. Use Vais-only helper gates in the release baseline.

### Task Briefs

#### 1. Self-source embedding

- Target files: `tools/embed_self_source.vais`,
  `scripts/test-embed-self-source-vais.sh`, `scripts/test-fixpoint-full.sh`.
- Requirements: support normalized `.vais` source-file retargeting, raw
  compact-program embedding, and raw string-call retargeting in the fixpoint
  gates.
- Done: `scripts/test-fixpoint.sh`, `scripts/test-fixpoint2.sh`, and
  `scripts/test-fixpoint-full.sh` build the Vais helper once and use it for all
  harness inputs; `scripts/test-embed-self-source-vais.sh` exercises
  normalized source-file retargeting, raw compile embedding, and raw
  string-call retargeting through the Vais helper.

#### 2. Stage IR normalization

- Target files: `tools/normalize_stage_ir.vais`,
  `scripts/test-normalize-stage-ir-vais.sh`,
  `scripts/test-fixpoint-full-self.sh`.
- Requirements: normalize generated stage IR names through a Vais-built helper
  before comparing stage1/stage2 self-host output.
- Done: the long self-host comparison uses the Vais normalizer; its focused
  gate checks the expected normalized IR shape directly through the Vais
  helper.

#### 3. Gate integration

- Target files: `scripts/test-release-gates.sh`, `AGENTS.md`,
  `compiler/self/SELF_HOST.md`, `WORKLOG.md`.
- Requirements: release gates build and exercise the Vais-native helpers before
  self-host and archive checks.
- Done: focused helper gates, full-codegen, full self-host, archive packaging,
  and website build all run from `bash scripts/test-release-gates.sh`.

## Verification Baseline

Run before closing compiler changes:

```bash
bash -n scripts/*.sh
bash scripts/test-vais-check-vais.sh
bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test-vaisc-host.sh
bash scripts/test-embed-self-source-vais.sh
bash scripts/test-normalize-stage-ir-vais.sh
bash scripts/test-fixpoint.sh
bash scripts/test-fixpoint2.sh
bash scripts/test.sh
bash scripts/test-fixpoint-full.sh
bash scripts/test-fixpoint-full-self.sh
```

## Current Progress

- [x] `proc_capture(argv: List<Str>) -> ProcessResult` is promoted through the
  compiler, host/front/direct fixtures, parity manifest, and release corpus with
  `examples/e202_proc_capture_result.vais`.
- [x] `List<Int>.filter(|x| predicate)` result lists are promoted through
  source-prep lowering, front/direct fixtures, parity manifest, and release
  corpus with `examples/e203_list_filter_result.vais`.
- [x] `List<Str>.map` and annotated `List<Str>.filter` result lists are
  promoted through source-prep lowering, front/direct fixtures, parity
  manifest, and release corpus with `examples/e204_list_str_map.vais` and
  `examples/e205_list_str_filter.vais`.
- [x] Receiver-based `List<Str>.filter` result type inference is promoted with
  `examples/e206_list_str_filter_infer.vais`.
- [x] `List<Str>` function-parameter map/filter result type inference is
  promoted with `examples/e207_list_str_param_map_filter.vais`.
- [x] `str_concat(left, right)` is promoted through direct string helper
  lowering and `List<Str>.map` closure bodies with
  `examples/e208_list_str_map_concat.vais`.
- [x] `List<Str>.filter/map` closure captures for known `Str` parameters and
  locals are promoted with `examples/e209_list_str_closure_capture.vais`.
- [x] `List<Str>.filter(...).map(...)` result lists are promoted for direct
  local, helper-return, helper-call, condition, `extend(...)`, and reassignment
  contexts with `examples/e263_list_str_filter_map_result_contexts.vais`.
- [x] `List<Str>.map(...).filter(...)` result lists are promoted for direct
  local, helper-return, helper-call, condition, `extend(...)`, and reassignment
  contexts with `examples/e264_list_str_map_filter_result_contexts.vais`.
- [x] `List<Str>.map(...).filter(...).len/contains/index_of/count` scalar
  chains are promoted for direct local, helper-return, helper-call,
  `List<Int>` mutation, reassignment, and condition contexts with
  `examples/e265_list_str_map_filter_scalar_contexts.vais`.
- [x] `List<Str>.filter(...).map(...).len/contains/index_of/count` scalar
  chains are promoted for direct local, helper-return, helper-call,
  `List<Int>` mutation, reassignment, and condition contexts with
  `examples/e266_list_str_filter_map_scalar_contexts.vais`.
- [x] Multiple same-family `List<Str>` pipeline scalar calls are promoted
  inside one expression with
  `examples/e267_list_str_pipeline_scalar_multi_expr.vais`.
- [x] Mixed map-filter/filter-map `List<Str>` pipeline scalar calls are
  promoted inside one expression with
  `examples/e268_list_str_pipeline_scalar_mixed_expr.vais`.
- [x] Composite Bool local inference for `List<Str>` pipeline scalar conditions
  is promoted with `examples/e269_list_str_pipeline_scalar_bool_infer.vais`.
- [x] Arithmetic-tail `List<Str>` pipeline scalar reassignments are promoted
  with `examples/e270_list_str_pipeline_scalar_reassign_arithmetic_tail.vais`.
- [x] Negated `List<Str>` pipeline scalar Bool expressions are promoted with
  `examples/e271_list_str_pipeline_scalar_bool_negation.vais`.
- [x] Bool if-expressions built from `List<Str>` pipeline scalar conditions are
  promoted with `examples/e272_list_str_pipeline_scalar_bool_if_expr.vais`.
- [x] Bool if-expressions built from `List<Str>` pipeline scalar conditions in
  helper-call arguments and Bool returns are promoted with
  `examples/e273_list_str_pipeline_scalar_bool_if_expr_call_return.vais`.
- [x] Nested helper-call Bool if-expressions inside `List<Str>` pipeline scalar
  reassignments are promoted with
  `examples/e274_list_str_pipeline_scalar_bool_if_expr_nested_call_reassign.vais`.
- [x] Int if-expressions built from `List<Str>` pipeline scalar conditions in
  locals, reassignments, helper-call arguments, and returns are promoted with
  `examples/e275_list_str_pipeline_scalar_int_if_expr.vais`.
- [x] Scalar value if-expressions in locals, reassignments, helper-call
  arguments, and returns without pipeline-specific lowering are promoted with
  `examples/e276_scalar_value_if_expr_embedded_call_args.vais`.
- [x] Scalar Bool value if-expressions in locals, reassignments, helper-call
  arguments, and returns without pipeline-specific lowering are promoted with
  `examples/e277_scalar_bool_value_if_expr.vais`.
- [x] Scalar Str value if-expressions in locals, reassignments, helper-call
  arguments, and Str returns without pipeline-specific lowering are promoted
  with `examples/e278_scalar_str_value_if_expr.vais`.
- [x] Scalar Char value if-expressions in locals, reassignments, helper-call
  arguments, and Char returns without pipeline-specific lowering are promoted
  with `examples/e279_scalar_char_value_if_expr.vais`.
- [x] `Map<Str,Str>.get_opt` string payload match expressions in returns,
  reassignments, helper-call arguments, and embedded Int returns are promoted
  with `examples/e280_map_str_str_get_opt_match_contexts.vais`.
- [x] `Map<Str,Str>` return-inferred locals feed those `get_opt` string payload
  match expression contexts without explicit local map annotations, with
  `examples/e281_map_str_str_return_infer_get_opt_match_contexts.vais`.
- [x] `Map<Str,Str>.get_opt` string payload match expressions support
  `str_concat`, `str_trim`, and `str_lower` transforms in verified `Str`
  contexts, with
  `examples/e282_map_str_str_get_opt_match_str_transforms.vais`.
- [x] Reassigned `Str` locals read their current runtime string when `.len()` is
  applied after dynamic match-transform results, with
  `examples/e283_str_len_reassigned_match_transform.vais`.
- [x] `Map<Str,Str>.get_opt` match arms compute direct `.len()` after
  `str_trim`/`str_lower` match-arm transforms, with
  `examples/e284_map_str_str_get_opt_match_transform_len.vais`.
- [x] `Map<Str,Str>.get_opt` string payload matches avoid pointer-tagged string
  payload integers by lowering through presence checks and value loads, and full
  self-host statement parsing skips match-arm braces in embedded conditions, with
  `examples/e285_map_str_str_get_opt_str_payload_stability.vais`.
- [x] `Map<Str,Str>.get_opt` string payload match expressions are promoted in
  `while` and `else if` condition chains with per-iteration loop reevaluation
  and preserved else-chain structure, with
  `examples/e286_map_str_str_get_opt_condition_chains.vais`.
- [x] `str_upper(text)` is promoted in full/direct paths for ASCII lowercase to
  uppercase normalization over literals, trimmed document fields,
  `Map<Str,Str>` reads, `List<Str>` reads, and `Map<Str,Str>.get_opt` match
  payload transforms, with `examples/e287_str_upper.vais`.
- [x] `str_ends_with(text, suffix)` is promoted in full/direct paths for suffix
  checks over literals, normalized strings, `Map<Str,Str>` reads,
  `List<Str>` reads, and `Map<Str,Str>.get_opt` match values, with
  `examples/e288_str_ends_with.vais`.
- [x] `str_replace(text, needle, replacement)` is promoted in full/direct paths
  for all-occurrence string rewriting over literals, normalized `Map<Str,Str>`
  reads, `List<Str>` reads, and `Map<Str,Str>.get_opt` match values, with
  `examples/e289_str_replace.vais`.
- [x] `str_split_into(text, sep, out)` is promoted in full/direct paths for
  delimiter tokenization into `List<Str>` out-params, preserving empty fields
  and treating an empty separator as one whole-text field, with
  `examples/e290_str_split_into.vais`.
- [x] `str_join(parts, sep)` is promoted in full/direct paths for
  reconstructing `List<Str>` values with separators, including empty-list
  handling and delimiter round trips with `str_split_into`, with
  `examples/e291_str_join.vais`.
- [x] `List<Int>.filter/map/filter-sum` closure captures for known `Int`
  parameters and locals are promoted with
  `examples/e210_list_int_closure_capture.vais`.
- [x] `List<Int>.filter(...).sum()` assignment to reusable `Int` locals is
  promoted with `examples/e211_list_filter_sum_assignment.vais`.
- [x] `List<Int>/List<Str>.filter(...).len()` return and assignment count
  lowering is promoted with `examples/e212_list_filter_len_count.vais`.
- [x] `List<Struct>.filter(...)` reusable result lists are promoted with
  `examples/e213_list_struct_filter_result.vais`.
- [x] `List<Struct>.map(...)` field projection is promoted with
  `examples/e214_list_struct_map_projection.vais`.
- [x] `List<Struct>.map(...)` projected result lists are promoted for direct
  returns, helper-call arguments, `extend(...)` sources, and reassignment with
  `examples/e245_list_struct_map_projection_direct_contexts.vais`.
- [x] `List<Struct>.map(...)` projected helper-call arguments are promoted in
  `if`, `while`, and `else if` condition expressions with
  `examples/e246_list_struct_map_projection_call_arg_conditions.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct `Int` field projection
  aggregation is promoted for helper returns and typed/inferred locals with
  `examples/e247_list_struct_map_projection_aggregates.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate conditions
  are promoted for `if`, `while`, and `else if` expressions with
  `examples/e248_list_struct_map_projection_aggregate_conditions.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate helper-call
  arguments are promoted in `return`, `let`, `if`, `while`, and `else if`
  contexts with
  `examples/e249_list_struct_map_projection_aggregate_call_args.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate simple
  arithmetic suffixes are promoted for returns and typed/inferred locals with
  `examples/e250_list_struct_map_projection_aggregate_arithmetic_tail.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate helper-call
  arguments preserve simple arithmetic suffixes in `return`, `let`, `if`,
  `while`, and `else if` contexts with
  `examples/e251_list_struct_map_projection_aggregate_call_arg_arithmetic_tail.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate expressions are
  promoted as direct `List<Int>.push` and `insert_at` mutation arguments with
  simple arithmetic suffixes in
  `examples/e252_list_struct_map_projection_aggregate_mutation_args.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate expressions are
  promoted for reassignment to known `Int` locals and parameters with simple
  arithmetic suffixes in
  `examples/e253_list_struct_map_projection_aggregate_reassign.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate expressions can
  be embedded inside broader `Int` expressions for locals, helper-call
  arguments, direct `List<Int>` mutation arguments, reassignments, and returns
  with `examples/e254_list_struct_map_projection_aggregate_embedded_expr.vais`.
- [x] `List<Struct>.map(...).sum()/max()/min()` direct aggregate expressions can
  be embedded inside broader `if`/`while`/`else if` condition expressions with
  `examples/e255_list_struct_map_projection_aggregate_embedded_conditions.vais`.
- [x] `List<Struct>.filter(...).len()` return and assignment count lowering is
  promoted with `examples/e215_list_struct_filter_len_count.vais`.
- [x] `List<Struct>.filter(...).map(...).sum()` same-item score aggregation is
  promoted with `examples/e216_list_struct_filter_map_sum.vais`.
- [x] `List<Int>.max()` ranking selection is promoted for local and parameter
  lists with `examples/e217_list_int_max.vais`.
- [x] `List<Int>.min()` ranking selection is promoted for local and parameter
  lists with `examples/e218_list_int_min.vais`.
- [x] `List<Int>.filter(...).max()`/`.min()` filtered ranking selection is
  promoted for direct returns and reusable `Int` locals with
  `examples/e219_list_filter_max_min.vais`.
- [x] `List<Struct>.filter(...).map(...).max()`/`.min()` score projection
  ranking is promoted for direct returns and reusable `Int` locals with
  `examples/e220_list_struct_filter_map_max_min.vais`.
- [x] `List<Struct>.filter(...).map(...).sum()/max()/min()` score aggregates
  are promoted as direct `Int` helper-call arguments with
  `examples/e256_list_struct_filter_map_aggregate_call_args.vais`.
- [x] `List<Struct>.filter(...).map(...).sum()/max()/min()` aggregate
  helper-call arguments are promoted in `if`, `while`, and `else if` condition
  expressions with
  `examples/e257_list_struct_filter_map_aggregate_call_arg_conditions.vais`.
- [x] `List<Struct>.filter(...).map(...).sum()/max()/min()` aggregate
  expressions are promoted inside broader `Int` expressions used by locals,
  helper-call arguments, direct `List<Int>` mutation arguments, reassignments,
  and returns with
  `examples/e258_list_struct_filter_map_aggregate_embedded_expr.vais`.
- [x] `List<Struct>.filter(...).map(...).sum()/max()/min()` aggregate
  expressions are promoted inside broader `if`, `while`, and `else if`
  condition expressions with
  `examples/e259_list_struct_filter_map_aggregate_embedded_conditions.vais`.
- [x] `List<Struct>.filter(...).map(...)` projected result lists are promoted
  for reusable `List<Int>` and annotated `List<Str>` locals with
  `examples/e239_list_struct_filter_map_result_chain.vais`.
- [x] `List<Struct>.filter(...).map(...)` projected result lists are promoted
  for direct `List<Int>`/`List<Str>` helper returns with
  `examples/e240_list_struct_filter_map_return_chain.vais`.
- [x] `List<Struct>.filter(...).map(...)` projected result lists are promoted
  as direct `List<Int>`/`List<Str>` helper-call arguments with
  `examples/e241_list_struct_filter_map_call_arg.vais`.
- [x] `List<Struct>.filter(...).map(...)` projected helper-call arguments are
  promoted in `if`, `while`, and `else if` condition expressions with
  `examples/e242_list_struct_filter_map_call_arg_conditions.vais`.
- [x] `List<Struct>.filter(...).map(...)` projected result lists are promoted
  as direct `List<Int>`/`List<Str>.extend(...)` arguments with
  `examples/e243_list_struct_filter_map_extend_arg.vais`.
- [x] `List<Struct>.filter(...).map(...)` projected result lists are promoted
  for existing `List<Int>`/`List<Str>` variable reassignment with
  `examples/e244_list_struct_filter_map_reassign.vais`.
- [x] `List<Int>.filter(...).map(...).max()`/`.min()` transformed scalar
  ranking is promoted for direct returns and reusable `Int` locals with
  `examples/e221_list_filter_map_max_min.vais`.
- [x] `List<Int>.filter(...).map(...).sum()` transformed scalar aggregation is
  promoted for direct returns and reusable `Int` locals with
  `examples/e222_list_filter_map_sum.vais`.
- [x] `List<Int>.filter(...).map(...).sum()/max()/min()` transformed scalar
  aggregates are promoted inside broader `Int` expressions used by locals,
  helper-call arguments, direct `List<Int>` mutation arguments, reassignments,
  and returns with
  `examples/e260_list_filter_map_aggregate_embedded_expr.vais`.
- [x] `List<Int>.filter(...).map(...).sum()/max()/min()` transformed scalar
  aggregates are promoted inside broader `if`, `while`, and `else if`
  condition expressions with
  `examples/e261_list_filter_map_aggregate_embedded_conditions.vais`.
- [x] `List<Int>.map(...).sum()/max()/min()` transformed scalar aggregates are
  promoted inside broader `Int` expressions and broader `if`, `while`, and
  `else if` condition expressions with
  `examples/e262_list_map_aggregate_embedded_expr_conditions.vais`.
- [x] `List<Struct>.filter(...).first().field`/`.last().field` record field
  selection is promoted for `Int`/`Str` returns and typed locals with
  `examples/e223_list_struct_filter_first_last_field.vais`.
- [x] `List<Struct>.filter(...).first().str_field.len()`/`.last().str_field.len()`
  record string length selection is promoted for `Int` returns and typed locals
  with `examples/e224_list_struct_filter_first_last_field_len.vais`.
- [x] `List<Struct>.filter(...).first()`/`.last()` whole-record selection is
  promoted for same-struct returns and typed/inferred locals with
  `examples/e225_list_struct_filter_first_last_value.vais`.
- [x] The same whole-record selection is promoted for multiline struct
  declarations with
  `examples/e226_list_struct_filter_first_last_multiline_value.vais`.
- [x] Filtered whole-record first/last selections are promoted as direct
  `List<Struct>.push` and `insert_at` arguments with
  `examples/e227_list_struct_filter_first_last_push_insert.vais`.
- [x] Filtered first/last field and string-length selections are promoted as
  direct scalar list mutation arguments with
  `examples/e228_list_struct_filter_first_last_field_push_insert.vais`.
- [x] Filtered first/last field and string-length selections infer `Int`/`Str`
  local types from declared record field metadata with
  `examples/e229_list_struct_filter_first_last_field_infer.vais`.
- [x] Filtered first/last field and string-length selections are promoted as
  direct `Int`/`Str` helper-call arguments with
  `examples/e230_list_struct_filter_first_last_field_call_arg.vais`.
- [x] Filtered first/last whole-record selections are promoted as direct
  same-struct helper-call arguments with
  `examples/e231_list_struct_filter_first_last_value_call_arg.vais`.
- [x] Filtered first/last helper-call arguments can use helper signatures
  declared later in the file with
  `examples/e232_list_struct_filter_first_last_late_helper_call_arg.vais`.
- [x] Filtered first/last helper-call argument lowering preserves simple
  arithmetic expression tails with
  `examples/e233_list_struct_filter_first_last_call_arg_expr_tail.vais`.
- [x] Filtered first/last helper-call arguments can start `if` condition
  expressions with
  `examples/e234_list_struct_filter_first_last_call_arg_if_condition.vais`.
- [x] Filtered first/last helper-call arguments can start `while` condition
  expressions with per-iteration recomputation, covered by
  `examples/e235_list_struct_filter_first_last_call_arg_while_condition.vais`.
- [x] Filtered first/last helper-call arguments can start `else if` condition
  expressions while preserving the preceding `if` guard, covered by
  `examples/e236_list_struct_filter_first_last_call_arg_else_if_condition.vais`.
- [x] Filtered first/last helper-call arguments preserve chained `else if`
  flow and final `else` fallthrough, covered by
  `examples/e237_list_struct_filter_first_last_call_arg_else_if_chain.vais`.
- [x] Full codegen recognizes `else` blocks with local statements followed by
  all-return nested `if` chains as terminating, covered by
  `examples/e238_list_struct_filter_first_last_call_arg_else_if_chain_return.vais`.
