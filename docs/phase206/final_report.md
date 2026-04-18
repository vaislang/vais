# Phase 206 Final Report — vaisdb 측 mass fix 한계 도달

## TL;DR

Phase 206은 **수정 0건** — 노력 대비 효과 낮음을 확인하고 Phase 207을 compiler 작업으로 권고.

| Metric | Phase 205 end | Phase 206 end | Delta |
|--------|---------------|---------------|-------|
| P001 | 0 | 0 | 유지 |
| E001 | 154 | 154 | 유지 |
| E002 | 3 | 3 | 유지 |
| E003 | 0 | 0 | 유지 |
| E004 | 40 | 40 | 유지 |
| E030 | 0 | 0 | 유지 |
| OTHER | 49+30 | 49+30 | 유지 |

## 핵심 발견

### E001 진단 한계
vaisc 에러 출력이 E001에 **source line/column 누락**:
```
error: error[E001] Type mismatch
  note: expected &mut [u8], found &[u8]
   = help: try using a type cast or conversion function
```
대비:
```
error: error[E030] No such field
  --> mod.vais:124:18
   |
 124 |     ...
```

### E022 변수명 한계
```
note: use of moved value: variable 'init' was moved
```
'init'는 컴파일러 내부 임시 이름. 실제 코드에 없음.

### Mass fix 시도
`tuple.vais`에서 ref → mut ref 패턴 시도 — line 없이는 cascading 디버깅 어려움. 1 location 고쳐도 같은 에러 메시지가 다른 곳에서 발생.

## 권고 — Phase 207은 compiler 작업

### P0. compiler error formatter 개선
- 위치: `crates/vais-types/src/types/error.rs` 또는 error_formatter
- 변경: E001/E022/E006/E008 모두 `--> file:line:col` 출력 + E022 진짜 변수명 표시
- 예상 영향: vaisdb E001 154건의 디버깅 효율 10x 이상

### P1. (P0 완료 후) vaisdb E001/E022 per-file 처리
- E001 154 → 100~50 예상
- E022 19 → 5 예상

### P2. E004 잔여 (40건)
- len/to_vec/to_string 등 stdlib API 호출 패턴 정리

## 누적 (Phase 199~206)

| Phase | 성과 |
|-------|------|
| 199~202 | P001 47→0 |
| 203 | compiler source_root fix |
| 204 | E002 88%↓ E003 100%↓ E030 67%↓ |
| 205 | E030 100%↓, put_* 177→0 internal |
| **206** | **mass fix 한계 도달** — Phase 207 권고 |

전체:
- P001: 47 → 0 (100%)
- E030: 27 → 0 (100%)
- E003: 6 → 0 (100%)
- E002: 26 → 3 (88%)
- E004: 143 → 40 (72%)
- compiler 수정: 1 (Phase 203 source_root)
- vaisdb commits: 17

## 종합

Phase 199~206에 걸쳐 vaisdb migration의 P001/E030/E003은 **완전 해소**. E002/E004도 큰 진전 (88%/72%↓). 

E001/E022 잔여는 compiler error formatter 개선 없이는 효율적 처리 불가. **Phase 207 = compiler 작업** (formatter 개선) 후 vaisdb 마무리 작업이 자연스러운 순서.

PROMISE: COMPLETE
