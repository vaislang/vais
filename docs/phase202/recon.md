# Phase 202 Recon-202 — vaisdb P001 + E-계열 전수 측정

## 측정
- 일자: 2026-04-18
- vaisdb HEAD: b1a0d012 (Phase 201 종료 후)
- compiler HEAD: 72b6d604 (Phase 202 시작)
- 방식: 276 .vais 파일 전수 vaisc check, first-error per file 기준

## 전수 통계

| Error | 파일 수 | 비고 |
|-------|---------|------|
| P001 | 2 (Phase 202 iter1 작업 중 → 1) | redo.vais + filter.vais cascading |
| E001 (Type mismatch) | 13 | Str/str, RwLock<T>, 타입 일치 불량 |
| E002 (Undefined var) | 44 | 주요: time_micros, NULL_PAGE/LSN constants |
| E003 (Undefined type) | 12 | TxnSnapshot, ReciprocalRankFusion 등 |
| **E004 (Undefined fn)** | **143** | 최대 도메인 |
| E022 (use-after-move) | 0 | 이미 clear |
| E030 (No such field) | 27 | ctes, max_depth 등 struct drift |
| **총 고유 파일** | **~190+** | 중복 가능 (한 파일이 여러 에러) |

## E004 (143 파일) — Top 10 missing symbols

```
27  push
23  len
11  lock
 7  resize
 6  write_u8
 5  write_i64_le
 4  write_u16_le
 4  with_severity
 3  insert
 2+ with_hint, validate, open, match_token, is_some, fetch_add
```

### 근본 원인 분석

**핵심 발견: Vec method dispatch 실패**

```vais
# doc_freq.vais:75
self.entries.push(entry)
```

- `entries: Vec<DocFreqEntry>` (struct 필드)
- `U std/vec` import 있음
- stdlib std/vec.vais 에 `F push(&self, value: T)` 정의됨
- 그럼에도 `function 'push' is not defined`

**진단**: compiler가 **struct field의 `Vec<T>` type annotation에 대해 stdlib impl method 해결 실패**. parameterized generic impl method dispatch 한계.

**영향**: top 10 missing symbols 중 `push`/`len`/`resize`/`insert`는 모두 **Vec/HashMap 메서드**. 따라서:

| 심볼 카테고리 | count | 처리 난이도 |
|--------------|-------|------------|
| Vec/HashMap 메서드 (push/len/resize/insert/to_vec) | ~60+ | **compiler fix 필요** — vaisdb 측 불가 |
| ByteBuffer write_X_le | ~20 | ByteBuffer import 누락 가능 |
| lock / open / validate 등 | ~20 | 각 domain 내부 정의 누락 |
| 기타 (with_severity, fetch_add 등) | ~40 | 개별 분석 |

### 결론: **Phase 202 범위에서 E004 대규모 처리 불가**

vaisdb 측에서 단순 symbol 추가로 해결되지 않는 이슈 ≥ 60건. compiler generic method dispatch 개선 없이는 Phase 202 exit criteria "15+ 파일 해소" 달성이 어려움.

## P001 structural (redo.vais, filter.vais)

### redo.vais
- **iter1에서 해소 완료** (Phase 202): tuple pattern `(Ok(x), Ok(y))` → nested M, `?` operator 분리
- 현재 E002 (lsn_segment, NULL_LSN 등) 단계로 진입

### filter.vais
- **iter1에서 부분 시도**: match arm comma 추가, `.ok_or()?` → nested M
- P001 여전히 cascading (line 243, 250 등 잔존)
- 구조적 parser context 오류 — 다음 iter에서 full-file rewrite 필요

## E030 (27 파일) — Struct field drift

샘플:
- graph_plan.vais:52 `params.max_depth` (GraphTraverseNodeParams에 없음)
- analyzer.vais:44 `query.ctes` (SelectQuery에 없음)
- (기타 샘플 수집 필요)

**처리 난이도**: per-file, struct 정의 확인 후 field 추가 또는 call site 수정. **mechanical 가능**.

## E002 (44 파일) — Undefined variable

샘플:
- header.vais `time_micros` (Phase 198 B4에서 확인 — import 있음에도 resolution 실패)
- 여러 constants: NULL_PAGE, NULL_LSN (import 누락 가능)

**처리 난이도**: per-file import 추가 또는 stdlib re-export 이슈.

## Phase 202 재설계 권고

iter1에서 P001 1건 해소, baseline green 확인. 남은 iter에서:

### 옵션 A (원안 유지) — E004 mass 처리
**권장하지 않음**: 60+ 파일이 compiler 한계로 고칠 수 없음. 노력 대비 효과 낮음.

### 옵션 B (pivot) — E030 + E002 focus
- E030 (27 파일): struct field drift — mechanical per-file
- E002 (44 파일): import 누락 — mechanical per-file
- Vec method dispatch는 **Phase 203+ compiler 개선**으로 이관

### 옵션 C (P001 완료만) — 보수
- filter.vais structural 완료까지만 (P001 = 0)
- E-계열은 분류만 하고 Phase 203으로 이관

## 권고: 옵션 B

Phase 202 남은 iter:
- iter2: filter.vais P001 완료 (P0-Struct 마무리)
- iter3: E030 top 5 파일 처리 (~5 파일 해소)
- iter4: Gate + Phase 203 seed (compiler 개선 필요 항목 명시)

## 예상 Phase 202 최종 결과

- vaisdb P001: 2 → 0 (완료)
- vaisdb E030: 27 → ~22 (5 해소)
- vaisdb E004: 143 (변동 없음) — Phase 203 compiler 개선 대기
- compiler baseline: green 유지 (iter1 확인 완료)

PROMISE: COMPLETE
