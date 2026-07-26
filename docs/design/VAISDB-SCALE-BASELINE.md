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

## 재현 커맨드

합성 경계 문서는 고유 N개 단어를 총 M개로 반복한 1~5,000라인 텍스트.
경계 실측: `vaisdb ingest <idx> d <doc>` — 고유 256=0/257=134,
두 번째 문서(합산 400키)=134. 타이밍: `vaisbench 3 vaisdb ingest ...`.
