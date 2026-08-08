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

## 현재 작업 (2026-08-08d) — v1.2.0 릴리스 컷 + 도그푸딩 재개
모드: 개별선택 (사용자 지시: 릴리스 컷 → 도그푸딩)
- [x] 1. v1.2.0 컷 ✅ 2026-08-08 — VAIS_VERSION 1.1.0→1.2.0(참조는
      --version 3사이트뿐), CHANGELOG Unreleased→v1.2.0 회전(v1.1.0 이후
      38커밋: 읽기 경로 선형화 search x77·similar x336·top 합산 ~64배 /
      크래시 일관성 fs_rename·temp-then-rename·dedup / 진단 2종 front
      fn-스코프·unknown-variable / 셸소트 데수가 / vaisbox 13종).
      release 게이트 GREEN 후 태그.
- [x] 2. 도그푸딩 재개 ✅ — flat 갭 라이브 확증(docs/ 13개 중 2개만) →
      `-r` 구현(collect_text_files: vaisgrep 워크 셰이프, 확장자-제거
      상대경로 doc id `design/LANGUAGE`, flat 기본값 계약 불변) →
      워크플로 게이트 +11(서브디렉토리 skip/update/삭제싱크/id 형식/
      플래그 거부) → scripts/vaisdb-repo.sh 일상화 래퍼(항상 증분
      reindex 후 서브커맨드). **실사용 발견 3건**: ① 4096 맵 창이 실수요
      2건을 차단(top on repo 코퍼스 "vocabulary too large" + 워킹 노트
      3종 per-doc 어휘 초과 skip) — 다음 스케일 아크 근거 ② exact
      화이트스페이스-토큰 검색이 하이픈/언더스코어 결합어(`test-
      fixpoint-full`) 미검색 — substring/부분어 수요 ③ similar/why/증분
      정상 체감.
진행률: 2/2 (100%)

### 다음 후보 (2026-08-08 도그푸딩 환류)
- 4096 맵 창 상향 or 문서 청킹: 실수요 2건 실증(top on repo 코퍼스 +
  워킹 노트 ingest). 컴파일러 계약 아크(trap·게이트 정합, 양 엔진) or
  제품측 청킹 — 설계 판단 필요.
- 부분어/substring 검색: exact-token 한계로 결합어 미검색. 토큰화 확장
  (하이픈/언더스코어 분리) or substring 폴백 — 랭킹 의미론 재검토 동반.

## 직전 완료 (2026-08-08c) — 잔여 후보 소탕: unknown-variable front + top 상수
모드: 개별선택 (2026-08-08 두 사이클의 경유 발견 2건 종결)
- [x] 1. top 상수 근본 진단 ✅ 2026-08-08 — 격리 프로브(스캔+field_copy만
      28ms/전샤드)로 빌더 가설 기각: **~31µs/라인의 정체 = totals Map
      선형 탐색**(~3000 엔트리 × 라인당 contains/get/insert). 교훈 재적중:
      곡선 관찰만으로 알고리즘 단정 금지, 프리미티브 격리 프로브 먼저.
- [x] 2. term_totals_into per-shard 로컬 집계 ✅ — term-해시 샤딩이 샤드당
      distinct ~1/64을 보장하므로 라인당 조회를 로컬 맵(~47)으로, 샤드
      종료 시 전역 병합. -2 계약 양 경로 보존(로컬 4096 도달 = 전역도
      초과). **top@1000: 2,945→154ms(19배)**, 출력 200/1000 완전 동일.
      트랩: 루프 내 재-let 맵은 반복마다 초기화되지 않음 — 루프 밖 선언
      + 반복 선두 clear()가 정석(초회 시도는 누적 오염+감속으로 실측
      기각).
- [x] 3. unknown-variable front 검사 ✅ — fn-스코프 defined-names 테이블
      (파라미터/let·let (a,b)/for·for (a,b)/Some·Ok·Err 바인더 — 바인더는
      같은 줄 사용 대비 검사 전 등록, let/for는 검사 후 등록으로 RHS
      자기참조 차단). 검사는 **소문자-callee 호출의 bare 단일-식별자
      인자만**(표현식 인자·대문자 생성자 인자 불관여). 삭제된 `let tab`
      부류가 clang IR 타입 오류 대신 front에서 LOUD 거부. 미정의 재현
      거부 + 동일-줄 바인더/tuple-for/파라미터 수용 프로브 양 엔진 42 +
      fixpoint_full 23k줄 스윕 clean + unknown_variable reject 픽스처.
- [x] 4. 코퍼스 스윕 게이트 + 환류 ✅ — front 334/direct/fixpoint-full/
      test.sh 410/parity native=410/self/workflow/scale/fmt/release 전
      체인 GREEN(오탐 0 입증). PERF-BASELINE top 절 갱신, CHANGELOG/
      WORKLOG 반영.
진행률: 4/4 (100%) — **양 사이클 경유 발견 소탕 완료, 등록 후보 0건**

## 직전 완료 (2026-08-08b) — P3 ingest 원자성: fs_rename 승격 + 부분 실패 불변
모드: 개별선택
- [x] 1. fs_rename(old, new) -> Int 승격 ✅ 2026-08-08 — fs_write_text
      2-인자 미러 클래스(드라이버 14사이트: declare 프리앰블/front
      허용목록/direct 인식기·디스패치·방출·타입추론/직접 C 프로토타입/
      호스트 런타임 impl). **fixpoint_full·core 무변경 적중**(Int-반환
      호스트 호출은 일반 호출 로워링 — Str-반환만 테이블). POSIX 덮어
      쓰기·소스부재 nonzero 잠금: e391 양 엔진 42, parity 등록,
      PRELUDE/LANGUAGE 반영.
- [x] 2. vaisdb 원자화 ✅ — write_text_atomic(temp-then-rename)으로
      remove_doc 샤드+docs.txt 재작성 전환(torn-file 부류 근절),
      index_reset이 .tmp 잔재 스윕, 크래시 모델 헤더 문서화(레지스트리
      append = 커밋 포인트).
- [x] 3. 스캔 dedup(last-wins) ✅ — scan_term_scores 스캔별 contrib 맵
      → fold(크래시-재시도 중복 포스팅에도 점수 정확), term_doc_count
      last-match로 why 정합. stats/top 진단 카운트는 창 내 과대 가능
      (문서화).
- [x] 4. 부분 실패 불변 게이트 ✅ — 워크플로 게이트 +10 케이스(중복
      포스팅 점수 불변 — 구 코드면 a1=4로 실패하는 실검증/why 정합/
      고아 포스팅 불가시+랭킹 불변/reindex 수렴/remove 무-tmp/스테일
      tmp 무시). 성능 회귀 없음(search 17ms/similar 56ms/cold 1.4s).
진행률: 4/4 (100%) — **제품 트랙 후보 P1~P4 전량 종결.** 게이트: front/
direct/fixpoint-full/test.sh 410(e391 편입)/parity native=410/self-host/
fmt/release 전부 GREEN. 트랩: 신규 예제는 첫 줄 `# expect: 42` 마커
필수(없으면 test.sh skip + parity 실패).

## 직전 완료 (2026-08-07) — VaisDB 읽기 경로 스케일 (벤치 잔여 헤비 소탕)
모드: 개별선택 (2026-08-04 벤치의 search x31 초선형·similar 19.8s·top 10s)
진단 실측: search 44→522→1441ms(5x docs→33x), why 16→86→230ms(N²).
격리 프로브(1000행 39ms→3000행 283ms, split 유무 무관)로 **진짜 근본 =
`__vais_str_slice`가 호출마다 소스 전체 strlen 스캔**(양 엔진 동형:
fixpoint_full.vais 21463 IR / vaisc_native.c 36582 C). 라인 추출 루프
전부가 파일 크기 2차 — lines()/split 빌트인 내부 슬라이스(레지스트리
경로), index.vais 수동 스캐너의 라인 슬라이스(샤드 경로: similar=95스캔
×O(n²), top=64샤드 전량)가 이 하나로 환원. 정렬 데수가 N²(선택정렬)은
부차 원인으로 실측 확인(교체해도 곡선 불변).
- [x] 1. 정렬 데수가 셸 정렬화 ✅ 2026-08-07 — `.sort()`(삽입)·
      `.sort_by(_desc)`(선택) 둘 다 3x+1 갭 셸 정렬(기존 %G-플래그 삽입
      패턴 동형, 캡 4095에서 최악 ~26만 스텝, 비교자 매핑 반전 주의:
      선택 best-pick desc=`>` → 삽입 shift-test desc=`<`). e335/e336/
      e340/e332 양 엔진 42. 부차 원인이지만 항상-N² 낭비 제거.
- [x] 2. str_slice strlen root-fix ✅ 2026-08-08 — `__vais_str_slice_raw`
      (malloc+copy만) 신설, 빌트인 내부 호출 **9+9곳**(full IR 프리루드 /
      direct C 런타임 미러) 전환. 사용자-레벨 trap 계약(kind 4, 134)
      프로브로 불변 확인. core는 emit-ir 세대 루프(세대 사이 rm -f
      build/vaisc)로 재생성, **gen2==gen3 fixpoint**. 격리 프로브:
      3000행 .lines() 283→9ms. **정본 재생성 경로 확정: self-probe가
      아니라 드라이버 emit-ir 루프**(gen1은 구 프리루드 탑재가 정상,
      and/or 커밋의 gen1!=gen2·gen2==gen3 패턴 재현).
- [x] 3. vaisdb 스캐너 in-place화 + 스냅샷 ✅ — field_copy/field_digits
      헬퍼, scan_term_scores/term_doc_count(프리픽스 비교, 불일치 무할당
      skip)/term_totals_into(탭 2개 워크, term 키만 복사)/remove_doc
      (doc 필드 in-place 매치+바이트 복사) 재작성, run_search 히트당
      doc_src_path→스냅샷 1회, run_similar 3독→1독. 구/신 출력 6연산
      완전 일치 + remove 기능 프로브 green. 경유 발견: 미정의 로컬
      (`tab`)이 front를 통과해 clang IR 타입 오류로 늦게 표면 —
      unknown-variable front 검사 후보 등록.
- [x] 4. 재실측+게이트+환류 ✅ — 공식 벤치(200/1000): search 1075→14ms
      (x77)/similar 19842→59ms(x336)/msearch x24/top x3.4, 5x 스케일
      팩터 search x1.75·similar x3.9·top x5.1(전부 선형 이하).
      PERF-BASELINE/CHANGELOG 갱신. 게이트: fmt/front/direct/
      fixpoint-full/test.sh 409/parity 409/fixpoint-full-self/
      vaisdb-workflow/vaisdb-scale 전부 GREEN. 트랩 노트: vaisdb
      self-test 스크래치 exit 2는 구/신 동일(기존 동작), zsh 루프
      미인용 변수 함정 재발(명시 커맨드로 재검증). top 잔여 상수
      (~31µs/라인 빌더 사이클)는 후보로 기록.
진행률: 4/4 (100%) — **읽기 경로 전 연산 선형화**

## 직전 완료 (2026-08-04) — front 진단 정밀도: map/list 테이블 fn-스코프
모드: 개별선택
- [x] 1. 근본수정 ✅ 2026-08-04 — check_front_contract_text의 map_locals/
      map_types·list_locals/list_types **파일 누적 → `fn ` 행 리셋**
      (front_fn_scope_reset 신설, front_rebind_reset 패턴 미러). 시그니처
      테이블(map_fns/struct_names/callable_names)은 파일 스코프 유지.
      HEAD로 빌드한 pre-fix 드라이버 오거부 ↔ post-fix 42 양방향 격리
      검증, 진짜 위반(로컬 `= 5`/자기-fn Map 파라미터 `= 3`)은 여전히
      LOUD 거부.
- [x] 2. 회귀 가드 ✅ — 기존 map/list accept·reject 픽스처 전수 감사
      (파일-스코프 의존 0건, `let x: Map = call()`형 자체 등록 확인) +
      map_shadow_name_cross_fn accept 픽스처(타 fn `pos: Map<Str,Int>`
      파라미터 + main Int `pos` `+=` 루프, exit 42).
- [x] 3. 검증·머지 ✅ — front 333/direct/fixpoint-full/test.sh 409/parity
      native=409/release GREEN. 541af7f3 → main 머지 f11ee3dd(5979c3f6
      레지스트리 배칭과 파일 겹침 0, 머지 후 main에서 front/test.sh/
      parity 재검증 green). 워크트리·브랜치 정리 완결.
진행률: 3/3 (100%) — **2026-07-26m 이후 개명 회피(`at`/`counts`) 불필요**

## 직전 완료 (2026-07-26m) — 증분 재인덱스: fs_mtime 승격 + vaisdb reindex
모드: 개별선택
- [x] 1. fs_mtime 승격 ✅ 2026-07-26 — 미러 배선(native.c 18참조, 런타임
      2종, **core 무변경 적중**), e373 잠금(존재>0/미존재 0/재작성
      비감소) 양 엔진 첫 시도 42.
- [x] 2. 인덱스 v3 + reindex ✅ — `id<TAB>path<TAB>mtime`(v2/bare 하위
      호환 = 스탬프 0 → always-stale), doc_mtime, reindex(added/updated/
      skipped). **경유 발견 2건**: ① 체커 Map-대입 검증의 로컬 테이블
      함수경계 누수(바인더명 `counts`가 타 함수 Map 로컬과 충돌해 LOUD
      오거부 — 후보 등록, 개명 회피) ② self-test 데이터 오염 자기트랩
      (legacy 라인이 후행 카운트 검증 오염 — 정리 추가).
- [x] 3. 게이트 ✅ — self-test(스탬프 3세대 파싱/add·skip 무sleep 결정)
      + workflow +5(backdated touch로 updated 경로 결정적/갱신 내용 즉시
      검색/stats 정확). 기존 게이트 무변경 GREEN.
- [x] 4. 환류 + 문서 ✅ — HOST_IO/PRELUDE/LANGUAGE/README/CHANGELOG,
      체커 스코프 누수 후보 등록, parity 392 실측(native=392). 래더
      (fmt+release) GREEN(LADDER-EXIT 0). 커밋·머지·푸시 완결.
진행률: 4/4 (100%) — **검색기 일상 루프(ingest→search→reindex) 완성**

## 직전 완료 (2026-07-26l) — VaisDB 검색 UX: search 서브커맨드 + 스니펫
모드: 개별선택 (제품화 2기 — 실사용 검색기)
- [x] 1. 인덱스 v2 ✅ 2026-07-26 — docs.txt `id<TAB>절대경로`(stdin `-`),
      docs_list_into가 id-only 반환 유지로 호출부 무수정, doc_src_path/
      abs_src_path 신규, remove 라인 보존 재작성. **구 형식 하위 호환
      공짜 성립**(bare-id 라인 = 스니펫만 생략).
- [x] 2. search ✅ — rank + 첫 매칭 라인 스니펫(str_trim + UTF-8 연속
      바이트 백오프 80바이트 트림), 0점 문서 숨김(재검토 반영: 경고
      대신 조용한 생략), 순번은 노출 기준. **경유 발견→root-fix: full
      silent 오값** — arr_elem_end가 괄호-비인지 콤마 정지라 리터럴 필드
      의 다중 인자 call(`m.get(i, 0)`)이 분리돼 orphan이 플랫 오프셋
      -1에 store(앞 원소 마지막 필드를 0으로 덮음, 마지막 push만 생존).
      IR 실측으로 확정, arg_comma_end(3중 깊이) 위임으로 워커 5사이트
      일괄 치유, e372 잠금(+배열 원소 call). .ll 강제-리빌드 수렴.
- [x] 3. 확장자 확대 ✅ — 화이트리스트 7종, 게이트 픽스처를 신 계약으로
      갱신(skip.md→skip.bin — 구 ".txt만" 계약을 잠그던 케이스).
- [x] 4. 게이트 + 실사용 ✅ — workflow +5 search 케이스 전부 GREEN,
      repo 실검색 데모(docs/design 인덱스 → 한글 스니펫 UTF-8 정확 출력
      실증). fs_mtime host API 수요 후보 등록. parity 391 실측(native=
      391), 래더(fmt+release) GREEN(LADDER-EXIT 0). 커밋·머지·푸시 완결.
진행률: 4/4 (100%)

### 후보 추가 (검색 스프린트 환류)
- ~~fs_mtime(path) host API~~ (완결 2026-07-26m: 승격 + vaisdb reindex,
  e373).
- ~~체커 Map-대입 검증 로컬 테이블의 함수 경계 누수(2026-07-26m 실측, 회피=바인더 개명)~~
  (완결 2026-08-04: `fn ` 행 리셋 근본수정 541af7f3, 머지 f11ee3dd,
  map_shadow_name_cross_fn 픽스처 — 개명 회피 불필요).

## 직전 완료 (2026-07-26k) — VaisDB P4: 스케일 게이트 (제품 래더 종결)
모드: 개별선택
- [x] 1. scripts/vaisdb-scale-gate.sh ✅ 2026-07-26 — 결정적 합성 코퍼스
      (60문서×203고유 = 포스팅 12,180)로 ingest-dir(예산 20s, 실측
      287ms)/rank 읽기 경로(예산 5s) vaisbench 예산 검사 + 정확성 스팟
      (top score/stats 캡처 비교 — exit-as-count와 pipefail 충돌 회피).
- [x] 2. 래더 편입 ✅ — gates.tasks에 vaisdb-scale 태스크(16태스크),
      ladder 체인 fmt+perf+vaisdb-scale+release, workflow parse 케이스
      15→16, 스테일 주석 정정.
- [x] 3. 문서 + 마감 ✅ — 리포트 P4 절(**제품 래더 P1~P4 종결 선언**),
      CHANGELOG, gates.tasks 주석 정정. 래더 GREEN(LADDER-EXIT 0 —
      vaisdb-scale 태스크 자기 체인 포함 통과 확인). 커밋·머지·푸시 완결.
진행률: 3/3 (100%) — **VaisDB 제품 래더 P1~P4 전체 종결**

## 직전 완료 (2026-07-26j) — VaisDB P3: ingest 견고성 (trap 제거 + 배치 계속)
모드: 개별선택 (스코프 정련: 실측 실패 모드는 "초대형 문서가 trap으로
프로세스+배치를 죽임" — temp-rename 전면 도입 대신 trap 제거가 본질.
문서-단위 쓰기 순서는 이미 안전(docs.txt가 postings 성공 후 마지막)).
- [x] 1. 스트리밍 term 카운터 ✅ 2026-07-26 — doc_terms_guarded_into
      (바이트 스캔 화이트스페이스-런 토크나이즈 + fresh insert 전 len
      가드 → -2). 내장 동일성 self-test(총계·카운트) + 질의 경로 통일.
- [x] 2. 스킵·보고 ✅ — 단일 ingest "error: document too large" 3,
      ingest-dir stderr "skipped (too large): <id>" 후 계속, 초과 합성
      4100-고유 문서가 인덱스 불변(self-test 잠금).
- [x] 3. 게이트 + 완료 기준 ✅ — workflow +4케이스(배치 스킵 stdout
      계약/stderr 보고/단일 거부 3/생존자 조회) 포함 전부 GREEN.
      **corpus_s 실문서 10 ingest-dir exit 0 실측**(8 인제스트 + 초대형
      2 스킵, 포스팅 11,877, rank 정상 — 베이스라인 즉사 완전 해소).
- [x] 4. 환류 + 문서 ✅ — 리포트 P3 절(temp-rename 미채택 근거 포함),
      CHANGELOG. 래더(fmt+release) GREEN(LADDER-EXIT 0). 커밋·머지·푸시
      완결.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-26i) — VaisDB P2: 디스크-우선 인덱스 (제품 재설계)
모드: 개별선택
- [x] 1. 레이아웃 + ingest ✅ 2026-07-26 — 인덱스=디렉토리(docs.txt +
      terms/s<N>.txt 64샤드), 샤드별 str_builder 배치→fs_append_text.
      self-test 42 양 엔진(신 레이아웃 전면 재작성).
- [x] 2. 조회 경로 ✅ — query/rank/report가 term별 샤드 바이트 스트리밍
      스캔 + per-query Map(4096 구간), remove는 샤드 재작성 필터.
      **기존 workflow 게이트 케이스 무변경 전부 GREEN**(회귀 기준 충족).
      경유 트랩 2건: ① bare str_builder_append 문장(기록된 "할당형만"
      재범 — 10지점 let 교정) ② **주석 속 `'`가 스캐너 오염**(lowering
      조용히 스킵→오도성 헤더 거부 — 신규 후보 등록, 진단 왕복 소모).
- [x] 3. 스케일 기준 ✅ — **corpus_l 374파일 ingest 0.9초, 포스팅
      23,114(구 계약 5.6배), rank 정상** — 인덱스 어휘 무제한 실증.
      잔존 정직 기록: 고유>4096 초대형 문서(per-doc Map)/부분 실패
      잔존(P3 대상).
- [x] 4. 환류 + 문서 ✅ — 리포트 P2 절/CHANGELOG/후보 2건(주석 `'`
      스캐너·초대형 문서). 래더(fmt+release) GREEN(LADDER-EXIT 0 —
      e337 릴리스 코퍼스 엔트리 포함 전 게이트). 커밋·머지·푸시 완결.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-26h) — VaisDB P1: Map 용량 계약 256→4096 (수요 주도 1호)
모드: 개별선택
- [x] 1. Recon ✅ 2026-07-26 — core는 map_cap() 파생 3함수 + IR 텍스트
      하드코드 39지점(값-plane 오프셋 256/len 슬롯 512/클리어 루프 513),
      direct는 typedef·insert 12지점. 스택 비용 맵당 ~64KB 허용 판정.
      내부 버퍼 256들(env/argv)과 from_byte 255 경계는 불변 확인.
- [x] 2. 상향 적용 ✅ — 정밀 패턴 치환(치환 수 = 분류 수 정합), .ll
      강제-리빌드 진짜 수렴(gen2==gen3). 경계 재실측 4096 OK/4097 trap
      134. **실전 문서(고유 1,588) ingest green + rank 동작**(P1 성공
      기준). 인덱스 flat 합산은 예측대로 P2 잔존. **경유 silent 발견·
      즉시 root-fix**: direct __vais_int_to_str 8-순환 버퍼가 map 키
      저장 시 붕괴(Str(i) 20개 → len 8) → malloc 사본(+OOM kind 5 trap).
- [x] 3. 게이트·문서 ✅ — workflow +8케이스(4096 holds/4097 traps ×
      양 엔진), e371 잠금(구 256 경계 통과 + 변환-키 사본 생존), PRELUDE
      map 계약 신규 문서화, CHANGELOG, 베이스라인 리포트 P1 결과 갱신.
      parity 390 실측(native=390). 래더(fmt+release) GREEN(LADDER-EXIT
      0). 커밋·머지·푸시 완결.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-26g) — VaisDB 제품화 1: 스케일 실측 (제품 모드 진입)
모드: 개별선택
- [x] 1. 코퍼스 3단(소10/중62/대374) + 기준선 ✅ 2026-07-26 — **소형
      코퍼스조차 ingest-dir 즉사**(v1.1.0 메시지 trap 134 발화). 성능은
      무결(문서당 ingest 중앙값 10ms — vaisbench 실측).
- [x] 2. 한계 실측 ✅ — 합성 문서로 차원 분리: **Map 256 계약이 이중
      즉사점**(문서당 고유 term 257 trap — 실전 문서 ~1,600 / 인덱스
      flat key 문서 2개째 400키 trap). 단어·라인은 스트리밍이라 5,000
      OK(무제한). 부수 발견: 부분 실패 시 인덱스 원자성 부재.
- [x] 3. 리포트 + 제품 래더 ✅ — docs/design/VAISDB-SCALE-BASELINE.md
      (경계 표/아키텍처 판정/P1~P4 래더). **수요 주도 컴파일러 1호 확정:
      Map 용량 계약 256→4096 상향**(P1). 제품 방향: 디스크-우선 인덱스
      (P2, term-샤딩 포스팅 + fs_append_text) + 원자성(P3) + 스케일
      게이트(P4).
진행률: 3/3 (100%)

### VaisDB 제품 트랙 다음 후보
- ~~P1. Map 계약 상향(컴파일러): cap 256→4096~~ (완결: PRELUDE 4096 계약).
- ~~P2. 디스크-우선 인덱스: term-샤딩 포스팅 파일, corpus_l(374) green~~
  (완결 a2d27dc4: 64샤드 terms/s<N>.txt, corpus_l 374파일 0.9s/23,114
  포스팅 — VAISDB-SCALE-BASELINE.md:68).
- ~~P3. ingest 원자성: temp-then-rename, 부분 실패 불변 게이트~~ (완결
  2026-08-08b: fs_rename 승격 + write_text_atomic + 스캔 dedup + 워크플로
  게이트 +10 — 제품 트랙 후보 전량 종결).
- ~~P4. 스케일 게이트: vaisbench 예산 모드 래더 편입~~ (완결 2026-07-26k).
- ~~미정의 로컬 변수 front 검사~~ (완결 2026-08-08c: fn-스코프
  defined-names + bare 단일-식별자 인자 검사, unknown_variable 픽스처).
- ~~top 잔여 상수~~ (완결 2026-08-08c: 정체는 빌더가 아니라 **Map 선형
  탐색** — per-shard 로컬 집계로 2,945→154ms, 19배).

## 직전 완료 (2026-07-26f) — fuzzing 라운드 8 + 값-정확성 사이클 마감
모드: 개별선택 (완료까지 자동 진행 — 사용자 지시)
- [x] 1. 라운드 8 프로브 ✅ 2026-07-26 — 오프셋 민감 표면 8종 × 선행-필드
      변형 × 양 엔진(격리 포함 22런): call().field 비-0 오프셋/struct-복사
      read·write/원소 필드 read/중첩 체인/sort_by 키/모듈 경계 struct-
      return/Result payload 회수 전부 값 정확. **"우연히 정답" 클래스
      silent 무발견** — e369 가드가 유일 사례였음을 전수로 확증.
- [x] 2. 발견 처리 ✅ — LOUD 인접 2건(bind-first 패밀리): ① 원소 Str
      필드 let(`docs[1].title`) → **root-fix**(rhs_los_field_chain_is_str,
      수집기 2곳 — e368/e369에 이은 3번째 리시버, e370 잠금, 강제-리빌드
      수렴) ② 인라인 Ok(Struct{Str..}) 리터럴 → 후보 등록(staged e306
      형태가 verified). 프로브 자기트랩 1건: 멀티라인 match는 front가
      거부하는 문서화된 경계(문법 오류였음).
- [x] 3. 사이클 마감 ✅ — **값-정확성 사이클 공식 종결**(silent 무발견
      전수 확증 + 잔여 전부 LOUD/휴면), v1.1.0 릴리스 컷(CHANGELOG +
      드라이버 버전), parity 389 실측, 래더 GREEN(LADDER-EXIT 0).
      제품 전환 핸드오프(WORKLOG). 커밋·태그·머지·푸시 완결.
진행률: 3/3 (100%) — **사이클 종결, 이후 컴파일러는 수요 주도 모드**

## 직전 완료 (2026-07-26e) — 필드-Str 패밀리 소탕 (e368 인접 완결)
모드: 개별선택
- [x] 1. 매트릭스 프로브 ✅ 2026-07-26 — 1단 .len 양 엔진 OK/2단은
      direct만 실패/mut 재대입 양 엔진 OK. 확장 프로브가 **full silent
      오값 발견**: `let a = make().num`이 struct-복사 분기로 오라우팅
      (꼬리 무시) → slot 0 로드(단일 필드 struct에선 우연히 정답,
      중첩 필드 선행 시 포인터 로드 — f5/f6 대조로 확정).
- [x] 2. Root-fix 2건 ✅ — ① direct nested 분기 2곳(로컬/call-결과)에
      Str 터미널+트레일링 len 래핑. ② full 수집기·let-emit의 struct-복사
      분기에 rhs_is_bare_call_stmt 가드(꼬리 있으면 스칼라 슬롯 →
      기존 emit_struct_return_field_call 플랫-오프셋 경로가 정확 emit).
      .ll 강제-리빌드 수렴(gen1==gen2). **silent 부류 1건 근절**(잔여
      Str-바인딩 형태는 silent→LOUD 전환, 후보 등록).
- [x] 3. e369 잠금 ✅ — 2단 로컬 체인/call-결과 스칼라·중첩 Str len/
      mut 재대입 양 엔진 42, f6·구조-복사 회귀 0. parity 388 실측
      (native=388), 래더(fmt+release) GREEN(LADDER-EXIT 0).
진행률: 3/3 (100%)

## 직전 완료 (2026-07-26d) — 값-정확성 fuzzing 라운드 7: 데수가 교차 조합
모드: 개별선택
- [x] 1. 프로브 8종 + 격리 2종 × 양 엔진 ✅ 2026-07-26 — 교차 조합
      (리터럴 필드 속 체인·grid 읽기/체인식 행 인덱스/5중 리터럴/while
      반복 생성/한 라인 3중 dynamic-row/문자열 필드 교차). **교차 자체는
      전부 값 정확** — 데수가 3종 상호작용 무결.
- [x] 2. **발견 1건 root-fix** ✅ — x8이 노출한 건 교차가 아니라 선재
      full 갭: `let t = m.tag`(struct Str 필드 → let, 깊이 1부터) 슬롯이
      i64로 오타이핑(LOUD store 충돌). 기존 rhs_struct_field_chain_sty가
      Str 터미널(-2)을 의도적으로 거부(struct 복사 전용) — 자매 프레디킷
      rhs_struct_field_chain_is_str 신설, 수집기 2곳 배선. .ll 재생성
      **강제 리빌드 절차로 진짜 수렴**. e368 잠금(1·2단 체인 + len/eq/
      concat 조합). 기존 verified는 match-arm 회수·직접 .len 체인뿐이었음.
- [x] 3. 판정·기록 ✅ — 라운드 수확: 새 데수가층 무결 + 인접 기본형 갭
      1건 승격(프로브가 자연스럽게 쓴 형태가 갭 — e358 패턴 재현).
      parity 387. 누적 157프로브.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-26c) — dynamic-row 중첩 리스트 읽기 root-fix (마지막 구갭)
모드: 개별선택
- [x] 1. Recon ✅ 2026-07-26 — NestedListInfo가 행별 플랫 리스트
      (vais_nested_<name>_rowN, List<Int>)로 lowering, 리터럴-행만 치환.
      dynamic-row 실패: full %v-1 / direct 미해결 ident(grid 이름 소거
      후). **주의: 주석(`: List<List<Int>>`) 있는 바인딩만 등록** — 무주석
      프로브는 별개 거부(잘못된 프로브였음). 동적-열은 기존 verified.
- [x] 2. Root-fix ✅ — lower_nested_list_dynamic_row_line: 행 식 임시
      호이스트 → 범위 가드 2개(음수/초과 시 row0[0-1] 경유 = 승격된
      메시지 trap 재사용) → let mut 결과 + 행별 멀티라인 단문 if 선택
      (direct의 단일라인 다문장 한계 회피 설계). 최종/생성 라인 전부
      occurrences 재작성 통과(리터럴-행 혼합 지원). core 무변경.
- [x] 3. e367 잠금 ✅ — 루프 누적(210)/이중 변수 인덱스/조합식/call-인자
      양 엔진 42, 범위 밖 행 134+"index out of range" 메시지 양 엔진,
      리터럴-행 회귀 0(d4/e348).
- [x] 4. 환류 + 문서 ✅ — 후보 취소선(**구갭 목록 완전 소진**), README/
      CHANGELOG, parity 386 실측(native=386). 래더 GREEN(LADDER-EXIT 0).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-26b) — 3중 인라인 struct 리터럴 root-fix
모드: 개별선택
- [x] 1. 매트릭스 프로브 ✅ 2026-07-26 — **후보 노트 정정**: 갭은 full
      전용(core 리터럴 파서 2단 한정 — 3단째 생성자명 @VAIS_UNRESOLVED_
      IDENT로 LOUD), direct는 3·4중 원래 지원. 2중 다중필드는 양 엔진
      verified 확인.
- [x] 2. Root-fix ✅ — 드라이버 공유 데수가 lower_nested_struct_literal_
      line: 3단+ 감지 시 innermost 리터럴부터 __vais_slit 합성 let으로
      호이스트(문자열/블록 브레이스 인지 스캐너), 완전 단계화 출력.
      2중 무접촉(트리거 깊이 3+). core 무변경(.ll 재생성 불필요 —
      데수가가 core 도달 전 처리).
- [x] 3. e366 잠금 ✅ — 3중 혼합 필드(전 레이어 형제 필드) + 4중 체인
      양 엔진 42. 경계 프로브 6종: 문자열 내 중괄호/단일라인 if 분리
      판정(데수가 회귀 0 — 실패 2건은 선재 direct 한계로 후보 기록).
      기존 중첩 struct 코퍼스(e01/e190/e197) 회귀 0.
- [x] 4. 환류 + 문서 ✅ — 후보 취소선 + 신규 direct 후보 2건(중첩 필드
      .len 체인/단일라인 if 다문장), README/CHANGELOG, parity 385 실측
      (native=385). 래더 GREEN(LADDER-EXIT 0 — 23k줄 core 소스가 새
      데수가 통과, 오발화 0 실증).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-26) — 무메시지 trap 잔여 메시지화 (진단 패밀리 완결)
모드: 개별선택
- [x] 1. Recon ✅ 2026-07-26 — **후보 노트 정정 실측**: direct는 진짜
      무메시지 7지점(__builtin_trap: slice/from_byte 범위 2 + OOM 5 —
      fs_list_files·dirs malloc ×4, token calloc ×1), core는 무메시지가
      아니라 **오라벨**(slice/from_byte trap이 kind 3 capacity 메시지
      차용). core llvm.trap 22지점 분류: kind3 차용 2 외 나머지는 정당한
      capacity(맵/split) 또는 컴파일가드·고정배열 부류(스코프 밖 기록).
- [x] 2. 메시지 trap 배선 ✅ — vais_list_trap kind 4(str 범위)/5(OOM,
      direct만 — core는 malloc NULL 무보호가 별개 사실로 기록) 추가.
      direct 정의를 첫 호출자(str_slice) 앞으로 이동(2026-07-18 플레이스
      먼트 트랩 회피). **경유 발견: 공허 수렴 함정** — cp 직후 mtime
      초단위 -nt 비교로 드라이버 리빌드 스킵 → gen1==gen2가 구 바이너리
      2회 emit의 공허 수렴이었음(stale-binary 교훈의 신종). 세대 간
      rm -f build/vaisc 강제 리빌드 절차로 **진짜 수렴(gen3==gen4)** 확보.
- [x] 3. 게이트 ✅ — workflow +6케이스(양 엔진 빌드 0/실행 134/메시지
      grep), 기존 list trap 4케이스 회귀 GREEN. 프로브 4/4(엔진×slice·
      byte) 메시지 정확.
- [x] 4. 환류 + 문서 ✅ — PRELUDE 진단 섹션(범위/OOM 메시지 + full=stdout
      puts 특성 명시), CHANGELOG, 후보 취소선 처리. 신규 후보 기록:
      core malloc NULL 무보호(별개 부류). 래더(fmt+release) GREEN
      (LADDER-EXIT 0 — fixpoint 수렴 게이트가 강제 리빌드 절차 산출물
      독립 재검증).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-25g) — 도그푸딩 26: vaiscut (열두 번째 도구)
모드: 개별선택 (완료까지 자동 진행 — 사용자 지시)
- [x] 1. e365 vaiscut 패키지 ✅ 2026-07-25 — `-f N`/`-d SEP`(기본 탭),
      파일/`-`/무파일=stdin. str_split_into 라인 단위 제품 첫 실사용.
      양 엔진 첫 시도 42(4번째 무갭 도구 스프린트).
- [x] 2. cut 관례 계약 ✅ — 무구분자=전체 라인/초과=빈 라인/빈 필드
      보존/멀티바이트 SEP 전부 실측 정확. 누락 파일 3 계속, usage 2,
      stdout 순수성.
- [x] 3. 게이트 + vaisbox 11애플릿 ✅ — workflow +11케이스(byte-cmp
      기대값/초과 grep -c/stdin/usage stdout-빈/**grep→cut→sort 3단
      체인**), vaisbox list 11/dispatch(42 산술 count*4-2). 전부 GREEN.
- [x] 4. 환류 + 문서 ✅ — **컴파일러 갭 0건**. parity 384 실측(native=
      384), PRELUDE/README/CHANGELOG. 래더(fmt+release) GREEN(LADDER-EXIT
      0 직접 확인) 후 커밋·main 머지·푸시 완결.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-25f) — 도그푸딩 25: direct 파리티 소탕
모드: 개별선택
- [x] 1. fs_remove direct 배선 ✅ 2026-07-25 — 재작성 사이트 일괄(프레디킷/
      게이트/emit/타입추론 Int/프로토타입 — 임베디드 impl·declare는 기존)
      + **bare 문장 원인 별도 발견**: 문장 분류기의 bare call 수용이
      user-fn 조회 전용 → fs_remove 호스트-디스카드 슬라이스
      (call_stmt_host_discard) 추가. e363 잠금(멱등 미존재 0 포함) 양 엔진
      42.
- [x] 2. direct 사용자-fn Str 체인 ✅ — 원인 2중: ① parse_builtin 패스의
      helper-call opaque-skip(도그푸딩 6 유산)이 `.len()`을 C로 누출 →
      Str-반환 user-fn이면 __vais_str_len 래핑 ② let 타입 추론의 user-fn
      조회가 체인 무시하고 Str 반환 → 조기 체인 게이트(Int) 추가. e364
      잠금(ret/let/if/산술 × 0·1-인자) 양 엔진 42. **체인 패밀리 완전
      종결**(e360+e364).
- [x] 3. 게이트 + 환류 + 문서 ✅ — 후보 목록 2건 취소선, PRELUDE/HOST_IO/
      README/CHANGELOG, e360 스테일 주석 정정, parity 383 실측(native=
      383). workflow 게이트 회귀 GREEN, 래더(fmt+release) GREEN
      (LADDER-EXIT 0 직접 확인).
진행률: 3/3 (100%)

## 직전 완료 (2026-07-25e) — 도그푸딩 24: fs_append_text 승격 + vaistee
모드: 개별선택
- [x] 1. `fs_append_text(path, text) -> Int` 승격 ✅ 2026-07-25 —
      fs_write_text 완전 미러(native.c 23=23, "ab" 모드, 런타임 2종).
      **core 무변경 실측 확정**(Int-반환 제네릭 call 경로, fs_list_files
      선례 적중 — .ll 재생성 불필요). e361 잠금(누적→트렁케이트) 양 엔진
      첫 시도 42. 경유 발견: fs_remove direct 미배선(후보 등록).
- [x] 2. e362 vaistee ✅ — stdout_write 바이트 정확 패스스루 + 파일들
      truncate/-a append, 실패 stderr+3(잔여 계속). self-test는
      write_files(파일 절반)만 사용해 stdout 무오염. 양 엔진 42.
- [x] 3. 게이트 + vaisbox ✅ — workflow +11케이스(passthrough/파일 내용/
      멀티/-a 누적/미존재 생성/실패 3/stdout 유지/grep→tee→wc 체인+파일
      사본) + vaisbox 10애플릿(list 10/dispatch, 42 산술 count*4+2).
      전부 GREEN.
- [x] 4. 환류 + 문서 ✅ — fs_remove direct 미배선 후보 등록, HOST_IO/
      PRELUDE/LANGUAGE/README/CHANGELOG, parity 381 실측(native=381).
      래더(fmt+release) GREEN, LADDER-EXIT 0 직접 확인.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-25d) — 값-정확성 fuzzing 라운드 6: 체인 인접 영역
모드: 개별선택
- [x] 1. 프로브 11종 × 양 엔진(22런) ✅ 2026-07-25 — 체인 위치 확장
      (while-조건/call-인자/이항 양변/Str() 변환/and 논리 조합), **call==
      call·call!=call equality 첫 탐침**, mut×host-call 재바인딩(빈→PATH→
      빈), mut Int 체인 재대입, 0-인자 4종(proc_self/fs_cwd/fs_temp_dir/
      stdin_read_all) 체인 전수, Str-call의 사용자-fn 인자 직접 전달.
- [x] 2. 발견 처리 ✅ — **발견 0건**(silent/LOUD 모두 없음). 처리 대상 없음.
- [x] 3. 판정 ✅ — 체인 수정(2026-07-25c)의 blast radius 값 정확 실측.
      코드 무변경(기록만, 라운드 3 선례). 누적 147프로브 기준선.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-25c) — 체인 갭 root-fix: host Str-call 리시버 `.len()`
모드: 개별선택
- [x] 1. 원인 recon ✅ 2026-07-25 — 매트릭스 8프로브 완성(위치 3×인자수 2
      ×양 엔진 + 사용자-fn/산술). **full 원인 2중**: ① gen_factor call-emit
      이 raw i8* 반환(skip_factor는 `.len()`을 포지션만 스킵 — "스킵하되
      emit 누락" 형태) ② 슬롯 수집기 공유 프레디킷(rhs_is_host_str_call)이
      체인 바인딩을 Str 슬롯으로 오타이핑. **direct 원인**: 0-인자 헬퍼
      분기가 트레일링-len 래핑(23252) 도달 전 continue → `.len()` C 누출.
      경유 트랩: zsh 루프 `$eng` 미인용 워드스플리팅으로 direct 전멸 오판
      1회(실제 회귀 아님 — bash/zsh 차이).
- [x] 2. full core fix ✅ — call-emit retstr 반환에 trailing_len_call_end
      peek(+emit_strlen_from_ptr, 기존 10사이트 관례) + rhs_is_host_str_call
      에 체인 시 0 반환(호출자 add_local_slots/collect_top_slots 2곳 공유
      fix). .ll 재생성 2세대 수렴 ×2회.
- [x] 3. direct fix ✅ — 0-인자 분기에 Str-반환 서브셋(fs_cwd/fs_temp_dir/
      stdin_read_all/proc_self)의 __vais_str_len 래핑(Int-반환 3종 제외).
- [x] 4. e360 잠금 + 환류 ✅ — 매트릭스 예제(ret/let/if/산술 × 0·1-인자)
      양 엔진 42, e358 스테일 주석 정정, 후보 갱신(완결 취소선 + 잔여:
      사용자-fn 체인 direct 미지원 LOUD), CHANGELOG/README. parity 379
      실측(native=379). 래더(fmt+release) GREEN, LADDER-EXIT 0 직접 확인.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-25b) — 도그푸딩 23: env_get 승격 + vaisenv (열 번째 도구)
모드: 개별선택
- [x] 1. `env_get(name) -> Str` host API 승격 ✅ 2026-07-25 — path_dirname
      완전 미러(native.c 21사이트 = 21, core 2, 런타임 2종 — copy_str/
      fs_host_copy 트랩 회피). .ll 재생성 gen1==gen2 수렴. e358 양 엔진 42
      + 네거티브(변수 설정 시 비-42)로 실환경 읽기 실증. **경유 발견:
      선재 LOUD 갭** — host Str-call 리시버 `.len()` 체인이 full 전 형태/
      direct 0-인자에서 clang 실패(env_get 특이 아님, fs_temp_dir 동일 —
      격리 프로브 5종 실측). 후보 등록, e358은 bind-then-len verified form.
- [x] 2. e359 vaisenv 패키지 ✅ — 값 라인 출력, 미설정 stderr+exit 3(잔여
      계속), stdout 순수성. env_get 총계약(미설정/빈 이름→"") 문서화.
      양 엔진 첫 시도 42.
- [x] 3. 게이트 + vaisbox 등록 ✅ — workflow +8케이스(build/self-test/
      set/unset 3/mixed 2/env→grep 체인) + vaisbox 9애플릿(list 9/dispatch
      vaisenv, 42 산술 count*4+6). 전부 GREEN.
- [x] 4. 환류 + 문서 ✅ — 선재 체인 갭(host Str-call 리시버 .len())을
      프로브 매트릭스와 함께 후보 등록. HOST_IO/PRELUDE/LANGUAGE/README/
      CHANGELOG, parity 378 실측(native=378). 래더(fmt+release) GREEN —
      재생성 core의 fixpoint 수렴 게이트 포함.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-25) — 도그푸딩 22: vaissort (아홉 번째 도구)
모드: 개별선택
- [x] 1. e357 vaissort 패키지 ✅ 2026-07-25 — 파일/`-`(stdin) 입력, 라인
      분해 후 `List<Str>.sort`(str_cmp 사전순) 정렬 출력. **sort 표면 제품
      첫 실사용**. 무인자 self-test 42 양 엔진(첫 시도).
- [x] 2. `-u`/`-r` ✅ — 정렬 후 인접 dedupe(전역 unique — 런타임 Str
      equality e352/e353 수정 경로 제품 검증) + 역순 출력, 조합(-u -r)
      포함 실측 정확. 에러/usage stderr(stdout-빈 검증), 누락 파일 exit 3
      후 잔여 소스 계속 정렬.
- [x] 3. 게이트 + vaisbox 등록 ✅ — workflow +12케이스(build/self-test/
      기본/-u/-r/-u -r/stdin/빈 stdin/누락 3/누락 stdout-빈/usage stdout-빈/
      grep→sort -r 파이프, 전부 cmp 바이트 대조) + vaisbox 8도구
      (list 8/dispatch vaissort). 주의: vaisgrep stdin 출력은 `N: line`
      프리픽스라 체인 기대값에 반영.
- [x] 4. 환류 + 문서 ✅ — **컴파일러 갭 0건**(vaiswc에 이어 2연속 무갭 —
      sort/이웃 equality/다중 소스 누적 전부 첫 시도 통과). parity 376
      (native=376 실측), PRELUDE/README/CHANGELOG, 4095 계약 명시.
      래더(fmt+release) GREEN.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-24g) — 도그푸딩 21: vaisbox (멀티콜 디스패처)
모드: 개별선택
- [x] 1. proc_self() 승격 + vaisbox ✅ 2026-07-24 — argv[0] 노출 표면 부재
      발견(proc_arg(0)=첫 사용자 인자 규약) → proc_self host API 승격
      (host runtime argv0 저장 + direct 9그룹 + core Str-return). vaisbox
      basename 디스패치 + 형제 재실행.
- [x] 2. list/unknown ✅ + **재실행 가드**(자기참조/형제부재 exit 3 —
      무한 재귀 위험 실측 후 추가).
- [x] 3. 게이트 ✅ — workflow +6(self-test/list/dispatch/pipe/unknown/
      guard). 명시 `<tool>`이 1급, argv0 디스패치는 형제 실존 시만(문서화).
- [x] 4. **갭 노출→root-fix: full `Str == 문자열반환호출`** ✅ — proc_self
      미등록으로 반환 슬롯이 i64→icmp 타입 충돌. is_host_str_return 등록,
      e356 잠금. parity 375.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-24f) — 도그푸딩 20: vaiswc (일곱 번째 도구)
모드: 개별선택
- [x] 1. e354 vaiswc ✅ 2026-07-24 — count.tally 모듈(struct Counts 반환,
      모듈 경계 struct-return 실증) + str_split_ws_into 제품 첫 실사용.
- [x] 2. 다중 소스 누적 total ✅ — 파일/stdin/혼합 집계, 누락 시 stderr+계속.
- [x] 3. 게이트 ✅ — workflow +7(단일/total 형태 grep -qx/stdin/누락 3/
      grep→wc 파이프). self-test 42 양 엔진.
- [x] 4. 환류 + 문서 ✅ — **컴파일러 갭 0건(첫 시도 42)**. parity 373.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-24e) — 도그푸딩 19: 상수-접힘 패밀리 정밀사냥
모드: 개별선택 (e352 인접 위험)
- [x] 1~2. 프로브 10종 × 양 엔진(20런) + fold 감사 ✅ 2026-07-24 —
      파라미터/슬라이스/argv len·인덱싱·빈검사/헬퍼 경유 리스트 메타/리터럴
      로컬 identity 전부 값 정확. 접힘 소스는 string_slot_eq 리터럴 키뿐임
      확정(18에서 파라미터, 여기서 mut).
- [x] 3. **발견 1건 root-fix**: `mut` Str 로컬 재대입 시 최초 리터럴 키가
      남아 stale 값으로 접힘(동일길이 재대입=여전히 같음, 다른길이=절대
      다름) → mut 바인딩은 리터럴 키/길이 폐기(3함수 add_local_slots×2+
      collect_top_slots). e353 잠금. 최상위 전역 mut 재대입은 별개 미지원
      문법(스코프 밖).
- [x] 4. 문서 ✅ — CHANGELOG/parity 372. 로컬 fold 패밀리 종결.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-24d) — 도그푸딩 18: vaisdiff (여섯 번째 도구)
모드: 개별선택
- [x] 1. e351 vaisdiff ✅ 2026-07-24 — 바이트 트림+중간 블록 라인 리포트,
      6000줄 동일 바이트 경로 self-test. exit 0/1/3(cmp 관례).
- [x] 2. `-` 한쪽 stdin ✅ (양쪽 - 는 2).
- [x] 3. 게이트 ✅ — workflow +7케이스.
- [x] 4. **갭 노출→root-fix: full Str 파라미터 equality 상수 접힘** ✅ —
      string_slot_eq가 리터럴 키(sty=오프셋) 없는 슬롯(파라미터/argv/slice,
      sty=-1)까지 접어 동일-길이 서로 다른 파라미터가 "같음"(icmp 1,0로
      상수화), 길이 다르면 "다름" 접힘. ident-vs-ident에서만 발화(리터럴
      피연산자는 토큰 종류가 달라 무사 — 기존 fuzzing이 놓친 이유).
      키 유효성 가드 2줄(.ll 재생성), e352 잠금, parity 371. 스프린트 17
      CHANGELOG/README 누락 백필 포함.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-24c) — 도그푸딩 17: stderr_write + vaisdb 파이프 + stdout 순수성
모드: 개별선택
- [x] 1. `stderr_write(text) -> Int` 승격 ✅ 2026-07-24 — stdout_write
      미러(fd 2).
- [x] 2. vaisdb `ingest-stdin <index> <doc-id>` ✅ — 파이프 인제스트,
      vaisgrep→vaisdb 체인 게이트.
- [x] 3. 필터 도구 stdout 순수성 ✅ — vaisgrep/vaisfmt의 에러 메시지를
      stderr로(eprint 헬퍼), stdout-빈 검증 케이스.
- [x] 4. 환류 + 문서 ✅.
진행률: 4/4 (100%) — 체크박스 백필 2026-07-25(완료 커밋 629f277e, 기록만 누락)

## 직전 완료 (2026-07-24b) — 도그푸딩 16: stdout_write 승격 + vaisfmt 필터
모드: 개별선택
- [x] 1. stdout_write 승격 ✅ 2026-07-24 — host fwrite+flush, direct 9그룹,
      core는 기본 i64 반환이라 무변경. 양 엔진 raw 출력·바이트 수 정확.
- [x] 2. vaisfmt `-` ✅ — 필터 모드(od로 바이트 정확 검증), `-c -` dirty 1.
- [x] 3. 게이트 ✅ — workflow +4(cmp 바이트 대조, **3-도구 파이프
      grep→fmt→grep** exit 2).
- [x] 4. 환류 + 문서 ✅ — HOST_IO/PRELUDE/LANGUAGE/README. 갭 0건.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-24) — 도그푸딩 15: stdin 표면 승격 + vaisgrep 파이프
모드: 개별선택
- [x] 1. stdin_read_all 승격 ✅ 2026-07-24 — host runtime(동적 버퍼) + direct
      10그룹 + **core is_host_str_return 프레디킷**(.ll 재생성 — 미등록 시
      full이 슬롯을 i64로 오타이핑). 경유 트랩: 기록된 C 선언 순서
      (fs_host_copy_n 뒤 배치) 재확인.
- [x] 2. vaisgrep `-` ✅ — grep_body 추출 후 stdin 분기. 라인/-c/빈 입력/
      **체인 파이프(vaisgrep|vaisgrep)** 실측 정확.
- [x] 3. 게이트 ✅ — workflow +3(stdin 라인 2/카운트 2/빈 0). 예제는
      corpus의 stdin 상속 행 위험으로 제품 코드+게이트로 커버(기록).
- [x] 4. 환류 + 문서 ✅ — HOST_IO/PRELUDE/LANGUAGE/README. 갭 0건.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-23b) — 도그푸딩 14: vaisbench 예산 모드 + perf 감시 태스크
모드: 개별선택
- [x] 1. `-b <budget_ms>` ✅ 2026-07-23 — runs/median/budget 리포트 후
      median>예산이면 exit 3. self-test +2단계(관대/불가능 예산).
- [x] 2. vaisbench-gate.sh ✅ — 패키지→`-b` 실행 래퍼.
- [x] 3. perf 태스크 ✅ — ladder = fmt+perf+release(15태스크). 실측:
      native 게이트 median 16.7s < 60s 예산 통과.
- [x] 4. 게이트/문서 ✅ — workflow +2(-b 통과 0/초과 3), parse 15,
      PERF-BASELINE "재개 트리거의 자동화" 기록. **갭 0건.**
진행률: 4/4 (100%)

## 직전 완료 (2026-07-23) — 도그푸딩 13: vaisbench (다섯 번째 도구)
모드: 개별선택
- [x] 1. e350 vaisbench ✅ 2026-07-23 — bench.stats 모듈(min/max/avg/median
      — 파라미터 리스트 in-place sort) + 가변 인자 패스스루 + time_millis
      제품 첫 실사용.
- [x] 2. 실패 전파 ✅ — 비0 즉시 전파(false→1), n<1/비숫자→usage 2.
- [x] 3. 게이트 ✅ — self-test 42 양 엔진, workflow +5케이스, parity 369.
- [x] 4. 환류 + 문서 ✅ — **컴파일러 갭 0건(첫 시도 42)**. 실전 증명:
      네이티브 스모크 게이트 3회 벤치가 기준선 17s 재현(16.1~17.9s).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-22e) — 성능 사이클 종결
모드: 기록만 (코드 무변경)
- [x] 사이클 결산 ✅ 2026-07-22 — 4유닛(래더 중복 제거→fixpoint 샤딩→
      value/parity 샤딩→selfhost 페이즈 병렬화)으로 **래더 ~69분→~22분
      (3.1×)**. 정본 카운트/케이스 집합 전 유닛에서 불변 실증.
- 잔여 비용(수확 체감으로 종결): selfhost 최중량 프로브 2개(각 ~150s,
  진짜 직렬 — 1세대 컴파일러 빌드+4.4MB emit), release 전용 게이트들,
  엔트리당 scripts/vaisc 프로세스 스폰 하한. 재개 트리거: 래더 시간이
  다시 병목이 되거나 게이트 수가 크게 늘 때.
진행률: 1/1 (100%)

## 직전 완료 (2026-07-22d) — selfhost 게이트 페이즈 병렬화
모드: 개별선택
- [x] 1. 독립성 판정 ✅ — 5개 프로브는 각자 embed→1세대 컴파일러 빌드로
      상호 독립, compare만 self_probe/retarget_fixpoint_full 산출물 소비.
- [x] 2. 도구 페이즈 인자 ✅ — argc 3=단일 프로브/4=compare(명시 stage
      경로)/2=기존 직렬 전체. 경유 이슈 1건(compare argc 오프바이원) 정정.
- [x] 3. 실측 ✅ — **272s→177s(1.5×)**, rc=0, 케이스-레벨 PASS 집합 직렬
      동일(diff 검증). VAIS_SELFHOST_PHASES=serial 보존. 래더 ~22분 누적.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-22c) — value/parity 게이트 샤딩
모드: 개별선택
- [x] 1. 도구 샤딩 ✅ — value/parity 체크 도구에 (shard_index, shard_count)
      선택 인자 + 매니페스트 엔트리 이름-해시 버킷 필터(단일-이름 경로와
      argc 4 직렬 경로 보존).
- [x] 2. 셸 팬아웃+집계 ✅ — VAIS_VALUE_SHARDS/VAIS_PARITY_SHARDS(기본 8),
      샤드 RESULT 카운터 합산으로 정본 RESULT 라인 재구성.
- [x] 3. 실측 ✅ — **206s→129s / 205s→129s**, 합산 pass=368·native=368
      불변. 래더(fmt+release) ~69분→~24분 누적.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-22b) — fixpoint-full 게이트 샤딩 병렬화
모드: 개별선택
- [x] 1. 원인 실측 ✅ — 케이스마다 23k줄 core를 embed→컴파일러 빌드→emit→
      clang→run (직렬 863s의 정체).
- [x] 2. 무상태 해시 샤딩 ✅ — 도구가 (shard_index, shard_count) 선택 인자
      수용, 6개 케이스 진입 함수에 이름-해시 버킷 조기 반환(케이스 간
      상태 없음 → 분할=구성상 전수).
- [x] 3. 게이트 8-워커 팬아웃 ✅ — VAIS_FIXPOINT_SHARDS(기본 8, 1=직렬
      동작 보존), 샤드 로그 집계·단일 RESULT. **863s→320s(2.7×)**,
      케이스 중복 0 실측. 차기 후보: test.sh/parity 동일 패턴.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-22) — 성능 기준선 측정 + 래더 중복 제거
모드: 개별선택 (우선순위 자체 판단: 래더 ~10분×상시 실행이 실측된 최대 고통)
- [x] 1. 단위 빌드 기준선 ✅ 2026-07-22 — hello 174/193ms, 패키지 ~140ms,
      core emit 444ms, 드라이버 리빌드 11.9s. 단위 비용은 문제 아님.
- [x] 2. 게이트 분해 ✅ — **직렬 래더 실측 ~69분**(체감 10분은 오판정).
      release 2153s(내부 재실행 1967s), fixpoint-full 863s 최대 단일.
- [x] 3. 중복 제거 ✅ — ladder = fmt + release(엄격 상위집합, ~36분,
      48% 단축). quick(~6분)/개별 태스크 유지, 14태스크 불변.
- [x] 4. 문서 ✅ — docs/PERF-BASELINE.md 신설(표+분석+차기 타깃
      fixpoint-full 기록).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-21b) — 값-정확성 fuzzing 라운드 5 (수렴 재확인)
모드: 개별선택
- [x] 1~2. 프로브 27종 × 양 엔진(54런) ✅ 2026-07-21 — 메서드명 필드 21종
      전수(잔여 1건 발견), 빈 리스트 trap 계약, clear 재사용, 동명 필드
      2-struct, 깊은 else-if, 메서드-혼합 루프 가드.
- [x] 발견 2건 = **라운드 4 패밀리 잔여 소탕(구조적 소진)** ✅ —
      ① remove_at 디스패치 무가드 1지점(감사 패턴 밖 형태) → `(` 가드.
      ② direct 잔여 무메시지 계약 trap 12지점(인덱스/삽입/용량/빈 pop·
      max·min/Map key_at·value_at·256 insert) → 메시지 trap 일괄.
      grep 잔여 = str_slice·from_byte 범위 2 + OOM 5뿐(별개 부류, 노트).
- [x] 3. 판정 ✅ — **신규-영역 무발견은 라운드 3·4·5 연속 3회**(4~5의
      발견은 전부 기지 패밀리 잔여, 전수 감사로 소진). 값-정확성 fuzzing
      사이클 종결. 누적 136프로브 기준선.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-21) — 값-정확성 fuzzing 라운드 4 (수렴 확인 → 리셋)
모드: 개별선택
- [x] 1. 프로브 12종 × 양 엔진(24런) ✅ 2026-07-21 — extend 자기 앨리어싱/
      insert_at 경계(0·len)/parse_int 경계(-13·007)/겹침 replace/
      Map<Int,Bool>·<Str,Char>/first·last 변이 후/음수 mod 루프/
      proc_capture stderr 텍스트/count·index_of/sort_by 순서 값 정확.
- [x] 2. **발견 1건 root-fix** ✅ 2026-07-21 — 메서드명과 같은 struct 필드
      (count/contains/index_of)를 **읽기만 해도 full 컴파일러가 무메시지
      abort**(gen_factor 메서드 디스패치가 `(` 확인 없이 paren_end 돌진).
      전수 감사로 무가드 3지점 확정 → `toks[i+3].kind == 9` 가드(.ll 재생성).
      e349(parity 368).
- [x] 3. 수렴 판정 ✅ — 라운드 3(0건)→4(1건)로 연속 무발견 리셋. 수렴
      확정에는 연속 2회 무발견 필요 → 라운드 5 권고.
진행률: 3/3 (100%)

## 직전 완료 (2026-07-20b) — 값-정확성 fuzzing 라운드 3 (흐름·스트레스)
모드: 개별선택
- [x] 1~3. 프로브 19종 × 양 엔진(38런) ✅ 2026-07-20 — Result<Str,Int>/
      <Str,Str> `?`·match 메시지 전파, for-each(Int/Str/누적 push 구조체),
      filter/map/sum·first 파이프라인, sort_by→filter 조합, 5KB builder→
      slice→replace 체인, Map 200키 덮어쓰기·remove, 10k 루프 누적,
      proc exit 255 왕복, 10단 call 체인, doc_term_counts, Bool/Char.
- [x] 4. **발견 0건 — 첫 무결함 라운드** ✅ 2026-07-20. 라운드 1~2의 수정
      (discard 데수가/중첩 리스트 일반화/brace if-식 거부) 이후 신규 영역
      전반 값 정확 실측. 코드 무변경(문서 기록만).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-20) — 값-정확성 fuzzing 라운드 2 (경계·흐름 확장)
모드: 개별선택
- [x] 1~3. 프로브 46종 × 양 엔진(92런) ✅ 2026-07-20 — i64 경계 일치성/
      빈 문자열 × str 빌트인 전체/char 합산/Result `?` 체인/Option match/
      break·continue/중첩 while/깊은 struct 체인/Map 순서 값 정확.
- [x] 4. 발견 3건 처리 ✅ 2026-07-20 —
      ① **중첩 리스트 조합식 읽기**: bare return만 동작(그 외 %v-1 clang
      에러) → 출현 치환 일반화(`nested_list_rewrite_occurrences`) + direct
      파이프라인에 lowering 편입(e77이 direct에서도 동작, 패리티 확장).
      ② **brace if-식 value 위치**: full이 silent 0 → front 거부
      ("then/else" help, front 게이트 reject 케이스). e348(parity 367).
      ③ 3중 인라인 struct 리터럴: LOUD clang 실패 — 단계 조립이 verified,
      아래 후보 등록.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-18c) — 값-정확성 fuzzing 라운드 (네이티브 양 엔진)
모드: 개별선택
- [x] 1~3. 프로브 32종 × 양 엔진 ✅ 2026-07-18 — 산술/우선순위/음수
      div·mod(C 트렁케이션 확인), 컨테이너 인터리빙, struct 흐름, 최근 승격
      표면 경계(str_cmp/sort/@ 100깊이/builder/앨리어싱) 값 정확.
- [x] 4. 발견 4건 전부 root-fix ✅ 2026-07-18 —
      ① direct: bare remove_at/pop 문장 미지원(문장 화이트리스트 +
      (void) 표현식 경로 재사용). ②~④ **full silent 오컴파일 3건**: bare
      remove_at/pop 문장이 미데수가로 core 도달(len 미조정·struct 리스트
      오합산·pop이 len 증가) — 공유 lowering에 discard-데수가
      (`let __vais_discardN = ...`) 추가로 검증된 할당형 경로 라우팅.
      e347(parity 366).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-18b) — 리스트 계약 trap 진단 승격
모드: 개별선택
- [x] 1. vais_list_trap(kind) ✅ 2026-07-18 — kind 0/1=index, 2=empty,
      3=capacity 메시지 후 abort. **최종 설계: core가 모든 emit 모듈
      프리앰블에 internal define(문자열 글로벌+puts+abort)을 자급 포함** —
      bare-libc 링크(codegen 게이트)까지 무의존. 드라이버/fixpoint 게이트
      런타임에는 부트스트랩 안전판 실심볼(구세대 .ll의 declare 대비) 유지.
      경유 이슈 3건: 링크 미해결(실심볼)→이중 declare→bare-libc 링크 —
      internal define 자급으로 종결, .ll 2세대 수렴.
- [x] 2. core 15지점 ✅ 2026-07-18 — 4개 라벨 trap 헬퍼(bounds/insert/
      empty/capacity) + 러너타임 헬퍼 trap 라벨 11곳의 llvm.trap을
      vais_list_trap 호출로 교체(.ll 재생성, 위생 0/0).
- [x] 3. direct 12지점 ✅ 2026-07-18 — checked_index + 용량 검사 11곳의
      __builtin_trap을 메시지 trap으로(플레이스먼트 트랩 2회: 매크로 정의
      순서/emit 순서 — 헬퍼를 checked_index 직전 배치로 해결).
- [x] 4. 게이트+문서 ✅ 2026-07-18 — workflow +4케이스(양 엔진 overflow
      빌드 0/실행 134), PRELUDE 진단 문서화. 정상 경로 무영향(100줄=100).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-18) — 도그푸딩 12: vaisfmt 위생 게이트 편입
모드: 개별선택
- [x] 1. scripts/vaisfmt-check.sh ✅ 2026-07-18 — 패키지 빌드 후 std/
      examples/compiler/tools 4트리 -c 검사, 실패 트리 보고.
- [x] 2. 갭 노출→도구 재구조화 ✅ 2026-07-18 — **str_split_lines_into가
      4095 슬롯 리스트 계약 초과(23k줄 core 소스)에서 무메시지 trap** →
      vaisfmt를 바이트 오프셋 스트리밍(리스트 미사용)으로 재작성, 5000줄
      생성 self-test 케이스로 보호. 4트리 위생 OK 실측.
- [x] 3. 게이트 편입 ✅ 2026-07-18 — gates.tasks에 fmt 태스크(quick/ladder
      체인 선두), workflow parse 케이스 14.
- [x] 4. 환류 + 문서 ✅ 2026-07-18 — 리스트 계약 상한 PRELUDE 명시, *_into
      초과 trap의 진단 메시지 부재를 아래 후보 등록.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-14g) — 도그푸딩 11: vaisfmt (네 번째 도구, str_builder)
모드: 개별선택
- [x] 1. direct str_builder 승격 ✅ 2026-07-14 — new/push/append/finish 4종
      (predicate/parse_builtin 그룹/skip/추론/host-helper 체인 argc·인자타입
      per-position Int·Str/prototype). full은 기존 검증. 수정 중 SEGV 1건
      (0-인자 분기 누락)·오염 치환 1건(expected argc) 즉시 정정.
- [x] 2. e346 vaisfmt 패키지 ✅ 2026-07-14 — 후행 공백 제거+최종 개행 보장
      (빈 줄 보존), `-c` 체크/in-place fix, 재귀 트리 워크(@). str_builder
      로 본문 재구축. self-test 42 양 엔진.
- [x] 3. 실측 ✅ 2026-07-14 — repo 3트리(examples/compiler/std) clean 0
      확인. workflow +7케이스(check 1/fix 1/recheck 0/missing 3/std clean 0).
- [x] 4. 환류 + 문서 ✅ 2026-07-14 — parity 365. **휴면 후보 재평가**:
      generic Result<T,E>·richer layout 모두 도그푸딩 3~11 동안 신규 요구
      0건 — 트리거 미충족 유지 확인(아래 후보 항목에 재평가일 기록).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-14f) — 도그푸딩 10: vaismake로 자기 게이트 래더 실행
모드: 개별선택
- [x] 1. tools/gates.tasks ✅ 2026-07-14 — 게이트 11개 + quick/ladder 체인
      (13 태스크), 주석/의존 문법 실사용.
- [x] 2. scripts/vaismake-ladder.sh ✅ 2026-07-14 — 패키지 빌드 후 ladder
      태스크 실행(첫 실패 중단·exit 전파 = 기존 래더 시맨틱 그대로).
- [x] 3. 실측 ✅ 2026-07-14 — 이번 스프린트 래더 검증을 vaismake로 수행.
      workflow 게이트에 파싱 케이스(list=13).
- [x] 4. 환류 + 문서 ✅ 2026-07-14 — 갭 0건. README/PRELUDE/CHANGELOG.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-14e) — 도그푸딩 9: vaismake 태스크 의존성
모드: 개별선택
- [x] 1. `!needs` 파서 ✅ 2026-07-14 — task_needs_into(이름 매칭 라인들의
      의존 누적).
- [x] 2. 의존 해석 실행 ✅ 2026-07-14 — run_task_tree: **@ 재귀 + Map 상태
      (1=visiting/2=done) 제품 실사용**, 의존 실패 전파, 순환 exit 4.
- [x] 3. 게이트 ✅ 2026-07-14 — workflow +3케이스(deps-first 0/실패중단 1/
      순환 4), self-test +7단계(42 유지).
- [x] 4. 환류 + 문서 ✅ 2026-07-14 — **컴파일러 갭 0건, 첫 시도 양 엔진 42**
      (@재귀+Map 파라미터+리스트 인자 조합 성숙 실측). PRELUDE/README/
      CHANGELOG.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-14d) — 도그푸딩 8: vaismake !env + 도구 체인
모드: 개별선택
- [x] 1. direct proc_run_env 승격 ✅ 2026-07-14 — proc_run 미러 5지점
      (2 List<Str> 인자 분기 + setenv 오버레이 헬퍼). full 무변경.
- [x] 2. vaismake `!env` ✅ 2026-07-14 — task_env_into 파서(!접두는 엔트리
      제외), run 모드 proc_run_env 적용, usage에 no-env(-o) 계약 표기.
- [x] 3. 도구 체인 게이트 ✅ 2026-07-14 — vaismake 태스크가 packaged
      vaisgrep 실행, exit 2 전파 확인(3-도구 조합 첫 실전).
- [x] 4. 환류 + 문서 ✅ 2026-07-14 — **컴파일러 갭 0건**(proc_run_env는
      계획된 승격). e345/parity 364, workflow +2케이스, PRELUDE/README/
      CHANGELOG.
진행률: 4/4 (100%)

## 직전 완료 (2026-07-14c) — 도그푸딩 7: vaismake (세 번째 배포 도구, proc 표면)
모드: 개별선택
- [x] 1. e344 vaismake 패키지 ✅ 2026-07-14 — make.tasks 모듈(엔트리 필터/
      이름·커맨드 추출/argv 분리), self-test 42.
- [x] 2. 실행 ✅ 2026-07-14 — **갭 노출→즉시 승격: direct proc_run 미배선**
      (proc_capture만 존재). lean fork/execvp/waitpid 헬퍼(비게이트) +
      expression/statement/추론 5지점. run ok 0/bad 1/목록 3.
- [x] 3. 캡처 ✅ 2026-07-14 — `-o`가 proc_capture stdout 출력(trim), exit=
      자식 code. proc 표면 제품 첫 실사용.
- [x] 4. 환류 + 문서 ✅ 2026-07-14 — parity 363(e344 main), workflow +8
      케이스, PRELUDE/README/CHANGELOG. 트랩: argv는 공백 분리(따옴표
      미지원 — 문서화된 no-shell 계약).
진행률: 4/4 (100%)

## 직전 완료 (2026-07-14b) — @(...) self-recursion 전 위치 승격
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
- [x] 5. 갭 환류 + 문서 정리 (Opus 직접) ✅ 2026-07-10 — 환류 갭 1호
      (built-in List sort)는 "다음 후보 작업"에 등록 후 e335/e336으로 완결.
진행률: 5/5 (100%) — 체크박스 백필 2026-07-25

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
- (재평가 2026-07-14: 도그푸딩 3~11 아홉 스프린트 동안 generic Result·중첩
  layout 신규 요구 0건 — 두 휴면 후보 모두 트리거 미충족 유지.)
- ~~host Str-call 리시버 위 메서드 체인~~ (완결 2026-07-25c): host call
  `.len()` 체인은 양 엔진 승격 완료(e360, ret/let/if/산술 × 0·1-인자).
  ~~잔여: 사용자 정의 Str 함수 체인~~ (완결 2026-07-25f): direct의
  opaque-skip에 __vais_str_len 래핑 + 타입 추론 체인 게이트 추가로 양
  엔진 완결(e364) — 체인 패밀리 전체 종료.
- ~~fs_remove direct 미배선~~ (완결 2026-07-25f): direct 재작성 사이트
  일괄 배선 + bare 문장 슬라이스(call_stmt_host_discard) — 양 엔진 e363
  잠금(idempotent 미존재 0 포함).
- ~~3중 이상 인라인 중첩 struct 리터럴~~ (완결 2026-07-26b): 드라이버
  데수가가 3단+ 리터럴을 단계 let(__vais_slit)로 재작성 — 양 엔진 e366
  잠금(3중 혼합 필드 + 4중). 원인은 full 전용(core 리터럴 파서 2단 한정,
  3단째 생성자명 미해결 → LOUD)이었고 direct는 원래 지원. 신규 후보 2건
  기록: ① ~~중첩 필드 경로 위 Str `.len()` 체인 direct~~ (완결
  2026-07-26e: nested 분기 2곳에 __vais_str_len 래핑, e369 잠금 — full
  silent call().field 오값도 같은 스프린트에서 bare-call 가드로 근절)
  ② direct는 단일라인 if 블록의 다문장(let+return)을 미지원(LOUD,
  full은 지원 — 스타일상 비권장 형태).
- call-결과 Str 필드의 `.len()` 없는 바인딩(`let s = make().mid.tag`)은
  양 엔진 LOUD(2026-07-26e에서 full은 silent→LOUD로 전환됨). verified
  form = `let o = make()` 후 필드 접근. 수요 시 슬롯 Str 타이핑 +
  inttoptr 배선으로 승격.
- 인라인 `Ok(Struct { .. })` 리터럴에 Str 필드 포함 시 full LOUD
  (@VAIS_UNRESOLVED_IDENT — 라운드 8 실측, all-Int 리터럴은 동작).
  verified form = e306의 staged(`let doc = ...` 후 `Ok(doc)`).
  수요 시 Result 래퍼 lowering에 Str-필드 리터럴 지원 확장.
- **주석 속 아포스트로피가 드라이버 스캐너를 오염**(P2 실측): contains_
  generic_type 등이 주석을 코드로 스캔해 `'`를 미종결 char 리터럴로
  판정하면 전체 텍스트 스캔이 0 반환 → Result lowering이 조용히 스킵되고
  하류에서 오도성 헤더 거부("supports ... function headers")로 표면화.
  수요 시 스캐너에 주석 스킵 추가. 당장은 .vais 주석에 `'` 회피.
- 문서당 고유 term > 4096(초대형 기술문서)은 per-doc 카운팅 Map 한도
  잔존 — VaisDB 측 후속(분할 ingest/스트리밍 카운팅) 후보.
- ~~중첩 리스트 dynamic-row 읽기~~ (완결 2026-07-26c): 행 스위치 데수가
  (행 식 임시 호이스트 + 행별 멀티라인 if 선택 + 범위 밖은 음수-인덱스
  경유 메시지 trap 재사용). e367 잠금(루프 누적/이중 변수 인덱스/조합식/
  call-인자, 양 엔진). 리터럴-행·동적-열 기존 경로 무접촉. **구갭 목록
  소진** — 잔여는 direct 후보 2건(중첩 필드 .len 체인/단일라인 if 다문장)
  과 core malloc NULL 무보호, 휴면 2종뿐.
- ~~잔여 무메시지 trap 2부류~~ (완결 2026-07-26): str 범위(kind 4 "vais
  str trap: slice or byte out of range")·direct OOM(kind 5) 메시지화 —
  양 엔진 게이트 잠금. 신규 기록: **core(full) 런타임의 malloc NULL은
  무보호**(trap조차 없음, null-deref) — 전수 NULL 검사 배선은 별개
  스프린트 규모, 수요 시 검토.
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
