# Phase 200 Recon-200 — 28 P001 정확 분류 (cascading 포함)

## 측정
- 일자: 2026-04-18
- 방식: vaisdb 전수 vaisc check (28 P001 파일) + grep 기반 sub-pattern 전수 카운트
- vaisdb HEAD: bb2877d (Phase 199 종료)
- compiler HEAD: bb16584b
- 출처 raw: /tmp/p200_full.txt (28 first-error entries)

## 핵심 발견 — Cascading underestimate

Phase 199 final_report에서는 "28 파일 = 28 P001"로 가정. **실제 cascading 포함 건수는 훨씬 큼**.

### 가장 큰 cascading: graph/wal.vais

`grep '^\s+LF \w+: \w+ = 0' src` 결과:
- **graph/wal.vais 단독으로 27개 instance**의 `LF x: type = 0` 패턴
- vaisc check는 첫 1건 (line 410)만 보고 → 26개 더 cascading 대기
- Phase 200 P0-A에서 graph/wal.vais 한 파일 fix 시 P001 카운트는 1만 줄지만, 후속 cascading errors도 함께 노출됨

### 작은 cascading
- `1e-8` (scientific notation): vector/quantize/scalar.vais:140 (1건, scalar는 현재 E004 단계로 진입했음)
- 기타 파일 내 추가 instance 가능성 — fix 시 전수 grep 확인 필요

## Sub-pattern별 정확 잔여 (P001 origin)

| Pattern | files | first-error 건수 | cascading 추정 (grep) | 적합 처리 |
|---------|-------|------------------|----------------------|-----------|
| C1a `var: type := mut` | ~3 (security 잔여 cascade) | 0 (Phase 199 처리 완료) | 0 | - |
| C1b `LF x: T = primitive` | 1 (graph/wal) | 1 | **27** | P0-A Opus direct |
| C1c `*.from_le_bytes` | 2 (deletion_bitmap, vector/quantize/mod) | 2 | 4 (deletion_bitmap 3 + mod 1) | P0-A (ByteBuffer 헬퍼) |
| C1d scientific notation `1e-N` | 1 (scalar) | (E004 차단) | 1 | P2 trivial |
| C5 LW destructure | 5 | 5 | 5 (security/{user,role,policy} + recovery/{undo,truncation}) | P0-B Opus direct |
| C7 path-style match arm | 2 | 2 | 2 (planner/{graph_plan, fulltext_plan}) | P0-C impl-sonnet |
| C8 struct field rebind | 2 | 2 | 2 (planner/{analyzer, optimizer}) | P2 |
| C9 `b'literal'` byte literal | 1 | 1 | 1 (sql/parser/token.vais) | P2 |
| C10 lifetime `<'a>` trait impl | 1 | 1 | 1 (vector/hnsw/cow.vais) | P2 |
| C11 top-level field decl | 1 | 1 | 1 (fulltext/search/boolean.vais) | P2 |
| C12 unbalanced `}` (cascading recovery) | 2 | 2 | 2 (storage/recovery/redo.vais, vector/filter.vais) | P2 — 다른 fix 후 자연 해소 가능 |
| C13 `self.X = ...` 위치 | 1 | 1 | 1 (fulltext/maintenance/compaction.vais) | P2 |
| C14 `vec![..; self.X]` macro | 2 | 2 | 2 (graph/edge/storage, vector/storage) | P1 grammar 결정 |
| C15 `mut` in pattern | 2 | 2 | 2 (graph/index/label, fulltext/mod) | P1 grammar 결정 |
| C16 if-let `I opt := mut self.X {` | 1 | 1 | 1 (graph/concurrency.vais) | P2 |
| C17 `Fn(T)` callable type | 2 | 2 | 2 (fulltext/concurrency, vector/concurrency) | P1 grammar 결정 |
| C18 dump struct fn-type field | 1 | 1 | 1 (ops/dump.vais) | P1 (closure type) |
| **합계** | **28** | **28** | **~54 cascading 포함** | - |

## 핵심 결론

1. **first-error 측정 underestimate ratio ≈ 2x**. 28 파일 → 54+ 실제 instance
2. **P0 작업 우선순위 변경**:
   - **P0-A (수정)**: graph/wal.vais 한 파일이 27 cascading instance 보유 → 단독 fix로 큰 가치
   - **P0-A 추가**: deletion_bitmap, vector/quantize/mod의 from_le_bytes 4 instance — ByteBuffer 헬퍼 필요
   - **P0-B (수정)**: C5 LW destructure 5 — 기존 그대로
   - **P0-C (수정)**: C7 path-style 2 — 기존 그대로
3. **P1 grammar 보류건**: C14 vec!-self, C15 mut-pattern, C17 Fn(T), C18 fn-type field — 4 sub-pattern 결정 필요
4. **P2 one-off 8건**: 단순 mechanical (b'(', `1e-8`, top-level decl, redo cascade 등). 일괄 또는 다음 phase

## P0 처리 순서 권고 (Phase 200 task #10/11/12)

| 순서 | task | 파일/instance | 위임 |
|------|------|--------------|------|
| 1 | P0-A graph/wal.vais 27 cascade | 1 파일, 27 LF i:type=0 fix | impl-sonnet (반복 패턴 균일) |
| 2 | P0-A from_le_bytes 4 instance | 2 파일 (deletion_bitmap, mod) — ByteBuffer 활용 | Opus direct (per-file judgment) |
| 3 | P0-B LW destructure 5 | 5 파일 | Opus direct (의미 보존 중요) |
| 4 | P0-C path-style 2 | 2 파일, U as alias 추가 | impl-sonnet |
| 5 | P1-Decision 4 sub-pattern | docs only | Opus direct |

## 예상 Phase 200 결과

- 28 → 13 P001 (P0 작업 15 해소 추정 — first-error 기준)
- Cascading 포함 실제 해소: ~30+ instance (graph/wal 27 + LW 5 + path 2 + bytes 4)
- Exit target ≤10에는 3 미달 — P2 일부 추가 처리 필요

PROMISE: COMPLETE
