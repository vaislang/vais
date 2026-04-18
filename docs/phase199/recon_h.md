# Phase 199 Recon-H — vaisdb 47 P001 분류

## 측정 환경
- 일자: 2026-04-18
- vaisc: `target/release/vaisc check` (compiler HEAD 23f3df9f)
- vaisdb: 외부 git repo, /Users/sswoo/study/projects/vais/lang/packages/vaisdb
- 방식: 276 .vais 파일 전수 vaisc check → 47 P001 → 라인+토큰 분류
- 원본 raw 데이터: /tmp/p001_full.txt (376 lines, 47 entries)

## 전체 통계
- vaisdb 총 .vais: 276
- P001 파일: 47 (17%)
- 식별된 sub-pattern: **18개** (예상보다 분산 — 단순 mechanical fix만으론 47% 정도가 한계)

## Sub-pattern별 분류

### G1 — Mechanical-safe (sed 가능, 단일 라인 패턴)

#### C1. `var: type := mut value` 또는 `LF/M ... type.METHOD()` (Rust-style typed binding) — **13건**
- 토큰: `found U64/U32/F32/I64/U8, expected expression`
- 패턴: `i: u64 := mut 0` → Vais는 `i := mut 0u64` 또는 타입 추론
- 또는 `f32.MAX`, `i64.parse(x)` 같은 primitive 메서드 콜 — Vais는 다른 호출 방식 필요
- 파일:
  - src/security/role.vais:325 — `j: u64 := mut 0;`
  - src/security/policy.vais:108 — `i: u64 := mut 0;`
  - src/security/rls.vais:286 — `i: u64 := mut 0;`
  - src/rag/memory/session.vais:338 — `i: u32 := mut 0;`
  - src/graph/wal.vais:226 — `LF i: u64 = 0`
  - src/server/copy.vais:290 — `LF empty: Vec<u8> = Vec.with_capacity(0)`
  - src/fulltext/index/deletion_bitmap.vais:289 — `max_doc_id := u64.from_le_bytes(...)`
  - src/vector/quantize/scalar.vais:241 — `R f32.INFINITY`
  - src/vector/quantize/pq.vais:218 — `best_dist := mut f32.MAX`
  - src/vector/quantize/mod.vais:312 — `val := mut f32.from_le_bytes(bytes)`
  - src/vector/hnsw/wal.vais:37 — `LF neighbors: Vec<(u64, f32)> := mut Vec.with_capacity(0)`
  - src/sql/catalog/constraints.vais:353 — `M i64.parse(trimmed) { ... }`
- mechanical-safe: **partial** — `i: u64 := mut 0;` → `i := mut 0u64` 일괄 가능. `f32.MAX/INFINITY` 같은 const 호출은 stdlib 매핑 필요 (별도 판정).

#### A1. Match-arm comma 누락 (`} Err(e) =>`, `} None =>`) — **4건**
- 토큰: `found Ident("Err"|"None"|"RAG_*"), expected ','`
- 파일:
  - src/fulltext/ddl.vais:109
  - src/rag/memory/storage.vais:100
  - src/rag/concurrency.vais:104
  - src/rag/wal.vais:381
- mechanical-safe: **yes** — 직전 arm 끝에 `,` 추가

#### A2. Trait impl `X T for S { }` → `X S: T { }` — **2건**
- 토큰: `found Ident("for"), expected '{'`
- 파일:
  - src/fulltext/concurrency.vais:204 — `X Drop for FullTextReadGuard`
  - src/vector/concurrency.vais:204 — `X Drop for HnswReadGuard`
- mechanical-safe: **yes** — 단일 라인 swap

#### C4. Line-continuation leading operator (`+` at line start) — **4건**
- 토큰: `found Plus, expected expression` 또는 `found Comma, expected expression`
- 파일:
  - src/rag/mod.vais:250 — `+ chunk_infos.get(...)`
  - src/ops/profiling.vais:237 — `+ (current_ns - self.sql_start_ns) / 1000`
  - src/ops/types.vais:258 — `+ self.graph_file_size + ...`
  - src/fulltext/search/phrase.vais:114 — `match_count,` (trailing comma in expr)
- mechanical-safe: **yes** — 이전 줄 끝에 붙이거나 괄호 묶기

### G2 — Per-file judgment (per-file rewrite 필요)

#### C2. `mut` in patterns/types (`Some((mut x, ..))`, `Term(mut tq)`, `field: mut Type`) — **5건**
- 토큰: `found Mut, expected pattern` 또는 `found Mut, expected type name`
- 파일:
  - src/graph/index/label.vais:120 — `Some((mut decoded_label_id, mut node_id))`
  - src/fulltext/mod.vais:665 — `Term(mut tq) =>`
  - src/server/tcp.vais:103 — `proc: mut ConnectionProcessor`
  - src/sql/executor/window.vais:188 — `partition: mut Partition`
  - src/sql/executor/sort_agg.vais:383 — `group: mut AggGroup`
- mechanical-safe: **no** — Vais는 mut binding 위치 다름. 함수 인자 mut, 패턴 내 mut 모두 수용 안 함 → 함수 본문에서 재바인딩 필요

#### C3. `vec![0u8; self.X]` macro에 `self.X` count — **2건**
- 토큰: `found SelfLower, expected macro token`
- 파일:
  - src/graph/edge/storage.vais:106 — `vec![0u8; self.page_size as u64]`
  - src/vector/storage.vais:215 — `vec![0u8; self.page_data.len()]`
- mechanical-safe: **no** — `vec!` 매크로가 self 표현식 인자를 안 받음. `Vec.repeat(0u8, self.X)` 같은 호출로 변환

#### C5. `LW key, _: &collection` (destructuring) — **2건**
- 토큰: `found Comma|Colon, expected '{'`
- 파일:
  - src/security/user.vais:503 — `LW key, _: &self.users {`
  - src/storage/recovery/undo.vais:73 — `LW &(txn_id, last_lsn): &active_txns {`
- mechanical-safe: **no** — Vais `LW` 문법은 destructuring iteration 미지원으로 보임. 명시적 인덱스 또는 `for (k, v) in` 형태 변환

#### C7. Path-style `mod/path.Type.Variant` in match arm — **2건**
- 토큰: `found Slash, expected '=>'`
- 파일:
  - src/planner/graph_plan.vais:97 — `sql/types.SqlValue.StringVal { v }`
  - src/planner/fulltext_plan.vais:50 — `sql/types.SqlValue.StringVal { v }`
- mechanical-safe: **partial** — `U sql/types as ...` import 후 짧은 이름 사용

#### C8. Struct field pattern `{ field: binding }` — **2건**
- 토큰: `found Colon, expected ','`
- 파일:
  - src/planner/analyzer.vais:485 — `SqlValue.StringVal { v: ref s }`
  - src/planner/optimizer.vais:100 — `HybridPlanNode.VectorScan { params, alias, cost: scan_cost }`
- mechanical-safe: **partial** — Vais는 field rebind 문법이 다름 (`{ field as alias }` 또는 동명 사용). per-file rewrite

#### C16. Unbalanced/Wrong-context `}` (recovery 후 라인 어긋남) — **2건**
- 토큰: `found RBrace, expected ':'`
- 파일:
  - src/storage/recovery/redo.vais:109
  - src/vector/filter.vais:243
- mechanical-safe: **no** — 진짜 구조 문제, 이전 라인 회복 후 cascading. 수동 inspect

### G3 — One-off (단일 사례, 별도 처리)

| ID | 파일 | 라인 | 패턴 | 처리 방향 |
|----|------|------|------|-----------|
| C6 | src/graph/concurrency.vais:272 | `I opt_lock := mut self.X` | if-let 문법, Vais는 `IL` 또는 다름 | per-file |
| C9 | src/storage/io/mmap.vais:109 | `extern "C" {` | Rust FFI 블록, Vais는 `N` 키워드 + 다른 grammar | rewrite |
| C10 | src/vector/hnsw/cow.vais:270 | `I<'a> Drop for EpochGuard<'a>` | Lifetime + trait impl | rewrite (lifetime 제거 또는 Vais 형식) |
| C11 | src/fulltext/search/boolean.vais:15 | `term: str,` (top-level) | struct decl 누락 | 컨텍스트 확인 필요 |
| C12 | src/storage/recovery/truncation.vais:105 | `Some(&seg) => seg` | ref pattern | `Some(seg)` + `&seg` 사용으로 변환 |
| C13 | src/fulltext/maintenance/compaction.vais:71 | `self.lists_compacted = ...` | top-level statement | fn 내부로 이동 |
| C14 | src/sql/parser/token.vais:186 | `b'(' =>` | byte literal in match | byte 값 비교로 변환 |
| C15 | src/fulltext/search/doc_freq.vais:67 | `Some(idx) -> {` | arrow misuse | `=>` 로 교체 |
| C17 | src/graph/stats.vais:265 | `vec![entries[j]]` | 매크로 인덱스 | Vec.from(slice) |
| C18 | src/ops/dump.vais:38 | `get_create_table_sql: F(str) -> Result<...>` | fn-type field | type alias 또는 closure type |

## 해소 추정 합산

| Group | 파일 수 | 적합 모델 | 예상 해소 |
|-------|---------|-----------|-----------|
| G1 mechanical (C1+A1+A2+C4) | 23 | Opus direct (per-file but obvious) | 23/23 |
| G2 per-file judgment | 13 | Opus direct (deliberate) | 8~13 |
| G3 one-off (10건) | 10 | 사례별 — 일부 Phase 200 미루기 | 4~7 |
| 잔여/cascading | 1 | Phase 200 | - |

**Phase 199 Tier 1 목표 (47 → ≤20)**: G1 23건 처리 + G2 일부 + G3 쉬운 4건 ≈ **30건 해소 가능** → 47 → ~17 ✅ 목표 달성 가능.

## 권고 (Phase 199 작업 재정렬)

기존 ROADMAP B199-A1~A4/B 분류와 매핑:

| ROADMAP task | 실측 매핑 | 파일 수 | 비고 |
|--------------|-----------|---------|------|
| B199-A1 (match-arm comma) | A1 | 4 | 추정 15였으나 실측 4 |
| B199-A2 (trait impl X T for S) | A2 | 2 | 추정 10이었으나 실측 2 (Drop 패턴만) |
| B199-A3 (global G X := mut) | — | 0 | vaisdb에 없음 (Phase 195 마이그레이션 완료) |
| B199-A4 (leading op) | C4 | 4 | 추정 2-5 정확 |
| B199-B (C-style for) | C1 일부 | 13 | 실측 13 (typed binding이 진짜 패턴) |
| **신규 B199-C (mut in pat/type)** | C2 | 5 | 추가 task 필요 |
| **신규 B199-D (vec! self)** | C3 | 2 | 작은 일괄 |
| **신규 B199-E (one-off 10건)** | G3 | 10 | per-file, 일부 Phase 200 |

**처리 순서 제안 (의존도 낮음 → 높음)**:
1. A1 (4) — 가장 안전 (comma 추가만)
2. A2 (2) — 단일 라인 swap
3. C4 (4) — 라인 join, 의미 보존 쉬움
4. C1 (13) — typed binding 변환, sub-pattern 균일
5. C2 (5) — mut 위치 변경, body 재바인딩
6. C3 (2) — vec! → Vec.repeat
7. G3 → per-file 판정. 쉬운 4건만 (C12, C13, C14, C15) Phase 199, 나머지 6건 Phase 200으로 deferred

## 예상하지 못한 발견

1. **G X := mut (A3)는 vaisdb에 0건**: Phase 195 lesson 적용 완료. ROADMAP B199-A3 task 불필요 → **삭제 또는 noop 처리**
2. **A1/A2 추정치 과대평가**: ROADMAP은 A1=15/A2=10이었으나 실측 4/2. 실제 비중은 C1 (typed binding, 13건)이 압도적
3. **47건 중 1/4 (10건)이 one-off**: vaisdb는 단일 패턴 반복이 아니라 **Rust 잔재 다양 패턴**. Tier 1 mechanical만으론 70% 해소가 한계

PROMISE: COMPLETE
