# Phase 213 Final Report — compiler imported_item_count fix

## TL;DR (massive breakthrough)

**OK files 30 → 78 (160% increase)**. cmd_check가 `set_imported_item_count`를 호출하지 않아 ownership check가 std/* 같은 imported items에도 적용되던 버그 수정.

| Metric | Phase 212 end | Phase 213 end |
|--------|---------------|---------------|
| **OK files** | 30 (11%) | **78 (28%)** ⬆⬆ |
| P001 | 0 | 0 |
| E001 | 197 | **72** (-63%) |
| E022 | 17 | **2** (-88%) |
| E008 | 2 | 2 |
| E006 | 15 | 13 |
| E004 | 14 | 89 (revealed deeper) |
| E002 | 1 | 10 (revealed) |
| E003 | 0 | 3 (revealed) |
| E030 | 0 | 5 (revealed) |
| ELSE | 0 | 2 |

## 핵심 작업

### Task #59 — E022 reporter → 진짜 root cause
- 'init' 변수명은 std/vec.vais의 `fold(init: T, ...)` 파라미터
- `acc := mut init` → ownership tracker가 init 이동으로 표시
- 하지만 fold는 stdlib code, 사용자 코드는 use 안 함
- 실제 문제: cmd_check가 `set_imported_item_count`를 호출 안 함 → ownership check가 imported items 검사
- build/core.rs는 이미 호출 — simple.rs 누락

### Fix (crates/vaisc/src/commands/simple.rs)
```rust
let single_file_ast = parse(&source).unwrap_or_else(...);
let original_non_use_count = single_file_ast.items.iter()
    .filter(|item| !matches!(item.node, vais_ast::Item::Use(_)))
    .count();
let imported_count = ast.items.len().saturating_sub(original_non_use_count);
if imported_count > 0 {
    checker.set_imported_item_count(imported_count);
}
```

## 누적 (Phase 199~213)

전체:
- vaisdb P001: 47 → 0 (100%)
- vaisdb E022: 17 → 2 (88%)
- **vaisdb OK files: 30 → 78 (160% increase)**
- compiler 변경: 5건 (source_root, error fallback, with_span, str_byte_at stdlib, imported_item_count)
- vaisdb commits: 21
- compiler commits: 30+

## Phase 214 권고

남은: E004 89, E001 72, E006 13, E002 10, E030 5, E003 3, E022 2, E008 2.

다음 phase: E004 89건 (Phase 211 cascading 다음 layer) 일괄.

PROMISE: COMPLETE
