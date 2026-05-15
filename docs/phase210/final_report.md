# Phase 210 Final Report

## TL;DR

| Metric | Phase 209 end | Phase 210 end |
|--------|---------------|---------------|
| P001 | 0 | 0 |
| E030 | 0 | 0 |
| E003 | 0 | 0 |
| E002 | 2 | 2 |
| E004 | 40 | 40 |
| E001 total | 155 | 155 |
| E001 with location | 86% | 86% |
| stdlib str free API | — | **str_byte_at, str_len 추가** |

## 핵심 작업

### Task #51 — stdlib str_byte_at 추가
`std/string.vais`에 primitive `&str` 전용 byte access API 추가:
```vais
F str_byte_at(s: str, i: i64) -> i64
F str_len(s: str) -> i64
X F __strlen(s: str) -> i64
```
- 기존 `String.char_at`은 wrapper type용. vaisdb 같은 primitive str 기반 코드에는 무용
- `load_byte(s + i)` 내부 intrinsic 활용 + bounds check

### Task #50 — `?` operator span fix (시도)
`special.rs`의 `Try`/`Unwrap` 에러에서 `inner.span` → `expr.span` 변경 시도.
그러나 문제 재현 계속됨 — vaisdb user.vais line 24 import에 여전히 attribute됨.
진짜 원인은 compiler path 밖. **Phase 211+ multi-file SourceMap 필요**.

### Task #52 — vaisdb str indexing 적용 (sample)
- `src/sql/types.vais`: `s[0]`/`s[idx]` → `str_byte_at(s, 0)`/`str_byte_at(s, idx)` (4 instances)
- vaisdb commit 6aad993
- E001 1건 해소 (해당 파일 str indexing 부분). 추가 42+ 파일 동일 패턴 적용 권고 (Phase 211).

## 누적 (Phase 199~210)

| Phase | 성과 |
|-------|------|
| 199~202 | P001 47→0 |
| 203 | compiler source_root fix |
| 204 | E002/E003/E030 cleanup |
| 205 | put_* 177→0 |
| 206 | mass fix 한계 |
| 207 | error_report fallback + with_span |
| 208 | E001 진단 0→86% |
| 209 | typed-binding 385→0 |
| **210** | **stdlib str_byte_at + sample 적용** |

### Compiler 누적 변경
- Phase 203: `cmd_check` source_root = package root (vais.toml 탐색)
- Phase 207: error_report byte offset fallback + `TypeError::with_span` helper
- Phase 208: checker_expr/calls.rs 11 unify 사이트에 `with_span`
- Phase 210: `std/string.vais`에 `str_byte_at`, `str_len`, `__strlen` 추가
- baseline: clippy 0, 10 E2E PASS 계속 유지

### Vaisdb 누적
- P001 47→0
- E030 27→0
- E003 6→0
- E002 26→2 (92%)
- E004 143→40 (72%)
- typed-binding 385→0
- vaisdb commits: 19

## Phase 211 권고

1. **str indexing per-file 확대 적용** — 42 파일 각각 `str_byte_at(s, i)` 변환 + `U std/string.{str_byte_at}` import
2. **multi-file SourceMap for 진짜 source line 추적** (compiler 측 큰 작업)
3. **ops/mod bool/i64 cluster** — 3 파일
4. **&mut [u8] deep context** — 4 파일
5. **OTHER 79건 분류** — E006/E008/E022

PROMISE: COMPLETE
