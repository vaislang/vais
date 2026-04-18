# Phase 208 Final Report — with_span 광범위 적용 + E001 분석

## TL;DR

**E001 진단 가시성 대폭 개선**: 0% → **86%** (134/155 E001 에러가 line/col 표시).

| Metric | Phase 207 end | Phase 208 end |
|--------|---------------|---------------|
| E001 total | 154 | 155 |
| E001 with --> location | 0 | **134 (86%)** |
| compiler baseline | green | green |

## 핵심 작업

### Task #42 Fix-Span — with_span 적용 사이트
`crates/vais-types/src/checker_expr/calls.rs` 내 unify 호출 사이트:
- 일반 함수 호출 인자 (line 70, 198, 254, 349, 423, 697, 761)
- str 메서드 호출 (charAt, contains, indexOf, substring, push_str)

각 site 패턴:
```rust
self.unify(expected, &arg_type).map_err(|e| e.with_span(arg.span))?;
```

총 **11 sites** 업데이트. baseline green 유지.

### Task #43 E001 재측정
- 134/155 E001에 source location 표시
- 나머지 21건은 unify 호출이 다른 파일에서 발생 (checker_expr 외)

### Task #44 E001 top 파일 분석
134 E001에 line/col 확보 후 top patterns:
- **85건 "expected Optional or Result, found ()"** — `?` operator 오용
  - **주의**: span이 대부분 U import 라인으로 잘못 attribute됨 (compiler 측 span 추적 bug 의심)
- **34건 "expected indexable type, found &str"** — str[i] 인덱싱 시도
- **8건 "expected indexable type, found str"** — 동
- **4건 "&mut [u8], found &[u8]"** — mutable ref 필요
- **3건 i64/bool** mismatch

### E001 per-file fix은 Phase 209+로 연기
- top 2 sub-pattern (?operator span bug, str indexing) 모두 **이슈의 성격이 다양**해 자동화 어려움
- span bug는 compiler 측 추가 조사 필요
- str indexing은 `.char_at(i)` / `.as_bytes()[i]` 등 per-file 판정

## 누적 (Phase 199~208)

| Phase | 성과 |
|-------|------|
| 199~202 | P001 47→0 (vaisdb mechanical) |
| 203 | compiler source_root fix |
| 204 | E002/E003/E030 대량 해소 |
| 205 | put_* 177→0 internal cleanup |
| 206 | mass fix 한계 진단 |
| 207 | error_report fallback + with_span helper |
| 208 | **E001 진단 가시성 0→86%** |

### compiler 변경 (누적)
- Phase 203: cmd_check source_root = package root (find_package_source_root)
- Phase 207: error_report get_source_context fail 시 byte offset 표시 + TypeError::with_span helper
- Phase 208: checker_expr/calls.rs 11 sites에 with_span 적용

### 실질 결과
- vaisdb P001: 47 → 0 (100%)
- vaisdb E030: 27 → 0 (100%)
- vaisdb E003: 6 → 0 (100%)
- vaisdb E002: 26 → 3 (88%)
- vaisdb E004: 143 → 40 (72%)
- vaisdb E001 진단: 불가 → 86% line/col 확보
- compiler baseline: 계속 green

## Phase 209+ 권고

1. **compiler span attribution bug** 조사: `?` operator span이 import 라인으로 잘못 표시되는 현상
2. str indexing 34건 per-file (sonnet batch 5-7 파일)
3. i: u64 := mut 0 잔여 (Phase 199 missed some) — 각 grep으로 찾아 수정
4. 나머지 &mut [u8] 4건 per-file

PROMISE: COMPLETE
