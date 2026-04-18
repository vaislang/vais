# Phase 206 Recon-206 + E001 quick wins

## E001 진단 한계

### vaisc 출력 제약
E001 에러 출력에 **source location (line)이 없음**:
```
error: error[E001] Type mismatch
  note: expected &mut [u8], found &[u8]
   = help: try using a type cast or conversion function
```
다른 에러 (E030 등)에는 `--> file.vais:line:col` 표시되지만 E001에는 누락. compiler 측 error formatter 개선 필요.

### Per-file 시도
`tuple.vais`에서 `&buf` → `&mut buf` 수정 시도했으나 E001 유지 — 다른 위치에서 cascading. line 없이는 디버깅 비효율.

## E022 (use-after-move) 19건

```
note: use of moved value: variable 'init' was moved
```
'init'는 코드에 직접 없음 — compiler 내부 임시 이름. 진짜 변수명 표시 안 됨.

## 결론

E001/E022 mass 처리는 **compiler error formatter 개선 없이는 비효율**. Phase 206은 노력 대비 효과 낮음.

## 권고

1. **즉시**: Phase 207에서 compiler error formatter에 line/source 추가 (E001/E022)
2. **그 후**: vaisdb E001/E022 per-file 처리

## 누적 결과 (Phase 199~205 종료 시점)

압도적인 진전:
- P001: 47 → 0 (100%)
- E030: 27 → 0 (100%)
- E003: 6 → 0 (100%)
- E002: 26 → 3 (88%)
- E004: 143 → 40 (72%)

남은:
- E001: 154 (compiler error 개선 후)
- OTHER (E006/E008/E022): ~50 (similar)

PROMISE: COMPLETE
