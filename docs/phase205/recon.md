# Phase 205 Recon-205

## Phase 204 종료 측정 후 추가 분석

### E001 top sub-patterns (154 파일)
- `expected &mut [u8], found &[u8]` — 4건 (mutable ref 누락)
- `expected i64, found bool` — 3건 (boolean → integer cast)
- `expected &[u8], found &mut` — 3건 (반대 방향)
- `expected indexable type, found &str` — 2건 (str indexing)
- `expected RwLock<()>, found RwLock` — 1건
- `expected Option, found Option<?N>` — 30+건 (type inference 실패, generic 파라미터 추정 못함)

### OTHER (78건) → 진짜 분류
- **E006 = 23** (새 코드 — 추후 분류)
- **E022 = 19** (use-after-move)
- **E008 = 6** (다른 종류)

### E004 잔여 (40건)
- **`put_u16_le` 21건** (Phase 204 sed 누락 — write_u16_le rename)
- `len` 7건
- `to_vec` 4건
- `write_f32_le` 3건 (이미 fix되었지만 다른 위치)
- `to_string` 3건

### 추가 발견 — put_* 가족
- `put_u64_le` 69 occurrences (write_u64_le 미존재 — i64 cast 필요)
- `put_u32_le` 63 occurrences (write_u32_le 미존재 — i32 cast 필요)
- `put_string` 25 occurrences (write_str 사용)
- `put_i64_le` 17 occurrences (write_i64_le rename)
- `put_f64_le` 3 occurrences (write_f64_le rename)

## 처리 가능성

### 즉시 처리 (sed 가능)
- `put_u16_le` → `write_u16_le` ✅ (Recon에서 즉시 적용)
- `put_i64_le` → `write_i64_le` ✅
- `put_f64_le` → `write_f64_le` ✅

### 처리 가능 (cast 필요)
- `put_u64_le(x)` → `write_i64_le(x as i64)` (overflow 가능 — 신중)
- `put_u32_le(x)` → `write_i32_le(x as i32)`
- `put_string(s)` → `write_str(s)`

### Per-file judgment
- E001 ref mutability — 호출 site 확인 필요
- E001 Option<?N> — type inference로 풀어야

## 권고 (Phase 205 작업 재배치)

iter1에서 put_u16_le 이미 fix. iter2에서:
- Task #36 E001 top: bool/i64, ref mutability (10 파일 정도)
- Task #37 E004: put_i64_le/put_f64_le rename + put_u64_le/u32_le cast + put_string
- Task #38 Gate

PROMISE: COMPLETE
