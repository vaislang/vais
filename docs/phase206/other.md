# Phase 206 OTHER 일괄 — 결과: 보류

## 결정

E022 use-after-move (19건) 및 E006/E008 처리는 **compiler 측 error formatter 한계로 비효율**:
- E022 메시지에 임시 변수명 'init'가 표시되어 진짜 변수 식별 불가
- E001/E022 모두 source line 누락

## 권고

Phase 207에서:
1. compiler error formatter 개선 (E001/E022/E006에 source location + 진짜 변수명 표시)
2. 그 후 vaisdb 측 mass fix

본 task 결과: **수정 0건, 분석만**.

PROMISE: COMPLETE
