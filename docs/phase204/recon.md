# Phase 204 Recon-204 — vaisdb E-계열 sub-pattern

## 측정 (compiler dir에서 실행, Phase 203 fix 적용)

| 코드 | 파일 수 | Top sub-pattern |
|------|---------|-----------------|
| P001 | 0 ✅ | — |
| E030 | 3 | page_size/RagConfig, m/HnswConfig, catalog/ExecContext (각 1건) |
| E003 | 6 | 5x `trait Drop` is not defined (X S: Drop 에서 Drop이 import 안 됨), 1x trait RowSource |
| E002 | 26 | **20x `__strlen`** (extern declaration 누락) + 6 기타 (store_i16, from_utf8_lossy, err_internal, ErrorCategory, 등) |
| E004 | 47 | **21x `put_u8`** (→ write_u8로 rename) + 8x `len` + 4x `to_vec` + 3x `to_string`/`put_f32_le` + 나머지 분산 |
| E001 | ~70 | 3x i64/bool mismatch, 2x &[u8]/&mut, 많은 `Option, found Option<?N>` (type inference 실패) |

## 처리 가능성 분석

### 높은 가치 (대량 일괄 해소)
- **E002 `__strlen`**: 20 파일 → 공통 extern 파일 import 또는 각 파일 `X F __strlen(s: str) -> i64` 추가
- **E004 `put_u8` → `write_u8` rename**: 21 파일 → sed 가능
- **E004 `put_f32_le` → `write_f32_le`**: 3 파일
- **E003 trait Drop import**: 5 파일 → `U std/?` import 추가 또는 trait 정의 확인

### 중간 가치 (per-file)
- E030 3건 — 각 struct에 필드 추가 또는 caller 수정
- E003 RowSource 1건
- E004 `len`/`to_vec`/`to_string` — 각 파일 컨텍스트 확인

### 난이도 높음 (Phase 205+)
- **E001 Option, found Option<?N>**: type inference 실패 — compiler side 이슈 가능성 (Option<T>의 T를 못 푸는 경우). 먼저 몇 건 샘플 확인 필요
- E001 bool/i64, &[u8]/&mut: 각 per-file 판정

## 권고 (Phase 204 작업 재배치)

- **Task #31 E030+E003**: E030 3 + E003 6 (trait Drop 5 + RowSource 1) — Opus direct
- **Task #32 E002**: `__strlen` 20건 일괄 + 기타 6건 — impl-sonnet
- **Task #33 E001/E004 top**: `put_u8` 21 rename + `put_f32_le` 3 + `len`/`to_vec` 일부 — impl-sonnet
- **Task #34 Gate**: 재측정

예상 해소:
- E002: 26 → ~6 (77%)
- E004: 47 → ~20 (57%)
- E030: 3 → 0
- E003: 6 → ≤2
- E001: 70 → ~50 (Phase 204 일부, 대부분 Phase 205)

PROMISE: COMPLETE
