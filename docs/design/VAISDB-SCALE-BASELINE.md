# VaisDB Scale Baseline (2026-07-26)

제품 모드 첫 실측: 도그푸딩 프로토타입(e337 vaisdb CLI, v1.1.0 컴파일러)을
실전급 코퍼스에 노출해 한계 차원을 분리했다. 결론 — **성능은 병목이
아니고, Map 256 계약이 두 지점에서 즉사를 만든다.**

## 실측 경계 (합성 문서로 차원 분리)

| 차원 | 지배 계약 | 실측 경계 | 실전 요구 | 판정 |
|------|----------|-----------|-----------|------|
| 문서당 고유 term | Map insert 256 | 256 OK / **257 trap(134)** | 실전 문서 ~1,600 (examples/README 실측) | **병목 1** |
| 인덱스 flat key (docs×terms) | Map insert 256 | 문서 1개(200키) OK / **2개째(400키) trap** | 수만+ | **병목 1 동근원 — 치명** |
| 문서당 총 단어 | (스트리밍) | 5,000 OK | — | 무제한 |
| 문서당 라인 | (스트리밍, ingest 경로) | 5,000 OK | — | 무제한 |
| 코퍼스 파일 수 | List 4095 | 374 OK (미도달) | 수천 | 후순위 |
| 단일 연산 성능 | — | ingest 중앙값 10ms/문서(200 term) | — | 무결 |

- trap은 v1.1.0에서 승격된 메시지 진단으로 발화("vais list trap: capacity
  exceeded", exit 134) — 무증상 아님.
- **부수 발견**: 다중 문서 ingest 중 trap 시 인덱스가 부분 상태로 잔존
  (doc-a만 남음, stats=docs 1) — **ingest 원자성 부재**.
- 실전 코퍼스 재현: repo 문서 10개 소형 코퍼스조차 ingest-dir 즉사
  (첫 문서의 고유 term이 256 초과).

## 아키텍처 판정

어휘는 코퍼스와 함께 무한 성장하므로 인메모리 Map cap을 아무리 올려도
검색엔진의 정본이 될 수 없다. 제품 방향은 두 갈래 병행:

1. **컴파일러(수요 주도 1호): Map 용량 계약 상향** — 256 → 4096(리스트
   계약과 정합). 실용 구간(문서당 고유 term, 세션 내 질의 Map)을 확보
   한다. insert/key_at/value_at trap 경계·양 엔진·게이트 정합 필수.
2. **VaisDB: 디스크-우선 인덱스 재설계** — 스냅샷(디스크)이 정본,
   메모리 Map은 per-doc/per-query 작업 버퍼로만. term-샤딩된 포스팅
   파일(term → doc:count 라인들)로 ingest는 append(fs_append_text),
   query는 필요한 term 파일만 로드. vaisfmt/vaisdiff의 스트리밍 선례를
   인덱스에 적용하는 것.

## 제품 스프린트 래더 (제안)

- **P1. Map 계약 상향(컴파일러)**: cap 256→4096, trap 메시지·게이트
  (양 엔진), 회귀(기존 Map 코퍼스). → 문서당 고유 term 실용 구간 확보.
- **P2. 디스크-우선 인덱스**: term-샤딩 포스팅 파일 레이아웃 설계 +
  ingest/query 재작성(문서당 Map은 P1 구간 내, 인덱스는 무제한).
  ingest-dir로 corpus_l(374파일) green이 완료 기준.
- **P3. ingest 원자성**: temp-then-rename 스냅샷/포스팅 쓰기, 부분 실패
  시 인덱스 불변 게이트.
- **P4. 스케일 게이트**: vaisbench 예산 모드로 ingest/query 스케일 곡선
  상시 감시(래더 편입).

## P1 결과 (2026-07-26h)

Map 계약 256→4096 상향 적용 완료. 재실측: 문서당 고유 term 경계 4096
OK/4097 trap(양 엔진), **실전 문서(고유 1,588) 단일 ingest green** +
stats/rank 정상. 인덱스 flat key는 소형 코퍼스(10문서 합산 어휘 >4096)
에서 여전히 trap — 예측대로 P2(디스크-우선) 영역으로 잔존. 경유 발견:
direct의 `__vais_int_to_str` 8-순환 버퍼가 map 키 저장 시 silent 붕괴
(20개 insert → len 8) — malloc 사본으로 root-fix(e371이 잠금).

## P2 결과 (2026-07-26i)

디스크-우선 인덱스 적용 완료(e337 재작성): 인덱스=디렉토리(docs.txt +
terms/s<N>.txt 64샤드 포스팅), ingest는 샤드별 fs_append_text 배치,
query/rank는 쿼리 term의 샤드만 바이트 스트리밍 스캔 + per-query Map
스코어(4096 구간). **기존 workflow 게이트 케이스 무변경 전부 GREEN**
(stdout/exit 계약 보존) — 회귀 기준 충족.

스케일 실측: **corpus_l 374파일 ingest 0.9초, 포스팅 23,114**(구 인덱스
계약 4096의 5.6배 — 인덱스 어휘 무제한 실증), rank 정상. 잔존 한계:
① 고유 term > 4096인 초대형 단일 문서(ROADMAP/WORKLOG급 기술문서)는
per-doc 카운팅 Map 한도에 걸림 — 후속(문서 분할 ingest 또는 스트리밍
카운팅) ② 부분 실패 시 docs.txt 부분 잔존 — P3 원자성 스프린트 대상.

## P3 결과 (2026-07-26j)

ingest 견고성 완결: 스트리밍 term 카운터(바이트 스캔 + Map len 가드)가
내장 카운터를 대체 — 어휘가 4096 창을 넘는 문서는 **trap 대신 -2 보고**.
단일 ingest는 "error: document too large" exit 3, ingest-dir은 해당
문서를 stderr로 보고하고 계속(Ok=성공 수). 질의 경로도 동일 카운터.

완료 기준 실측: **corpus_s(실문서 10, ROADMAP·WORKLOG급 초대형 2 포함)
ingest-dir exit 0** — 8개 인제스트(포스팅 11,877) + 2개 스킵 보고,
rank 정상. 베이스라인의 원 즉사 케이스가 완전 해소됐다. 문서-단위 쓰기
순서(성공한 postings 후 docs.txt)는 P2부터 안전 — temp-rename 전면
도입은 실측 실패 모드(대형 문서 trap)와 무관해 미채택(근거 기록).

## 재현 커맨드

합성 경계 문서는 고유 N개 단어를 총 M개로 반복한 1~5,000라인 텍스트.
경계 실측: `vaisdb ingest <idx> d <doc>` — 고유 256=0/257=134,
두 번째 문서(합산 400키)=134. 타이밍: `vaisbench 3 vaisdb ingest ...`.
