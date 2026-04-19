# Phase 6.27c — Structural Compiler Work for vaisdb 229 → 261

**현재**: vaisdb 229/261 (32 fails), Phase 6.27b 포화. 추가 기계적 수정 불가.
**목표**: 261/261. 5개 구조적 compiler 과제로 분리. 각 과제는 독립 실행 가능 (blockedBy 없음).
**전략**: 개별 파일 패치 금지. compiler-side 해결만 인정.

---

## Task A — MutexGuard<T> Deref (unlock 5+ files)

**영향 파일** (5개):
- `fulltext/concurrency.vais`: `queue.push(txn_id)` where queue is `MutexGuard<Vec<u64>>`
- `vector/concurrency.vais`: 동일 패턴
- `vector/hnsw/cow.vais`: `neighbors.get(&node_id).ok_or(...)` where neighbors is `MutexGuard<HashMap>`
- `fulltext/mod.vais`: 여러 lock.write_lock() 후 guard 사용
- `storage/txn/deadlock.vais`: `self.edges.get_opt(...)` 후 mutate

**증상**: `MutexGuard<T>` 타입에 `.push` / `.get` / `.ok_or` 등 inner `T`의 메소드 호출 시 "function not defined".

**근본 원인**: `std/sync.vais`의 `MutexGuard<T>` 가 stub (actual `mutex_ptr: i64`). `Deref`/method forwarding 미구현. TC는 `guard.method()` 호출을 MutexGuard 자체 method로 해석.

**구현 경로**:
1. `crates/vais-types/src/checker_expr/calls.rs`의 MethodCall 체크에서 receiver가 `Named{MutexGuard, [T]}` 일 때:
   - 먼저 T의 method 탐색 → 있으면 T에게 dispatch
   - 없으면 MutexGuard 자체 method
2. `crates/vais-codegen/src/inkwell/gen_method.rs` (또는 유사)에서 같은 Auto-deref: Named{MutexGuard,[T]} → GEP inner ptr → call T method
3. 또는 (simpler): stdlib `sync.vais`에 MutexGuard에 `deref`/`deref_mut` 반환 추가 + Auto `*guard` rule

**완료 기준**:
- `./scripts/check-integrity.sh` vaisdb ≥ 234 (+5)
- std 82/82, phase158 18/18 유지
- 위 5 파일 중 최소 3개가 TC + codegen pass

**추정 작업량**: 200-400줄 compiler + std

---

## Task B — Bidirectional TC for Variant Name Disambiguation (unlock 3+ files)

**영향 파일** (3개):
- `sql/parser/parser_expr.vais`: `UnaryOp { op: Not }` — Not이 TokenKind.Not과 UnaryOp.Not 둘 다
- `sql/parser/mod.vais`: 유사 TokenKind vs enum variant 충돌
- `sql/parser/parser_select.vais`: `match_token(Star)` 맥락 + parse_expr cross-file

**증상**: bare identifier가 여러 enum의 variant 이름일 때 TC가 첫 enum으로 resolve → 잘못된 context에서 "expected X, found Y".

**근본 원인**: lookup_var은 variant 이름 1번 히트면 종료. Expected type context를 모름.

**구현 경로**:
1. `crates/vais-types/src/checker_expr/mod.rs`에 `check_expr_bidirectional(expr, hint: Option<&ResolvedType>)` 확장 — 이미 있으면 활용
2. Struct-lit field checking에서 `expected_fields.get(field_name)` 을 hint로 넘김
3. lookup_var에서 hint가 enum Named{E}면 그 enum의 variant 우선
4. 안전 장치: hint 없을 땐 현재 로직 (iter-42 sort) 유지

**완료 기준**:
- vaisdb ≥ 232 (+3)
- parser_expr.vais + parser/mod.vais 최소 2개 TC pass

**추정 작업량**: 150-300줄 compiler

---

## Task C — Cross-file `X Parser` Method Resolution (unlock 4 files)

**영향 파일** (4개):
- `sql/parser/parser.vais`: `self.parse_select()` (parser_select.vais)
- `sql/parser/parser_expr.vais`: `self.parse_select()`
- `sql/parser/parser_select.vais`: `self.parse_expr()` (parser_expr.vais)
- `sql/parser/mod.vais`: 종합

**증상**: 단일 파일 컴파일 시 다른 파일에 정의된 `X Parser { F method(...) }`가 보이지 않아 "function not defined".

**근본 원인**: integrity test는 파일 단독 build. X 블록은 파일 경계를 넘지만 TC/codegen이 import된 파일의 X 블록만 import.

**구현 경로**:
Option 1 (compiler): `crates/vais-types/src/` 에서 모든 import된 파일의 X 블록 메소드를 해당 struct scope에 등록.
Option 2 (test runner): `crates/vaisc/tests/integrity/ecosystem_health.rs`에서 parser_* 파일들을 하나의 그룹으로 빌드 (같은 디렉토리의 모든 .vais 묶음).
Option 3 (vaisdb): sql/parser/parser.vais가 모든 parser_*.vais를 U import — 개별 테스트 시 transitive import가 다른 X 블록도 가져오게.

**권장**: Option 3이 가장 빠르고 risk low. parser.vais 하나에 `U sql/parser/parser_expr` 등 추가 + cycle 발생 확인.

**완료 기준**:
- vaisdb ≥ 233 (+4)
- 4개 parser_* 파일 모두 TC pass

**추정 작업량**: 20-50줄 vaisdb (Option 3) 또는 200-400줄 compiler (Option 1/2)

---

## Task D — HashMap<K,V> Value Type Propagation (unlock 5+ files)

**영향 파일** (5+개):
- `sql/executor/mod.vais`: `expected TableInfo, found i64`
- `sql/catalog/manager.vais`: 동일
- `security/policy.vais`: `expected i64, found Vec<PolicyEntry>`
- `rag/chunking/graph.vais`: `Vec<u64>` erasure
- `storage/txn/deadlock.vais`: `Vec<u64>` erasure

**증상**: `HashMap.get(&key)` 반환 `Option<V>` 에서 V가 i64로 erase. Pattern `Some(v)` 로 bind한 v가 잘못된 타입.

**근본 원인**: TC level에서는 V를 제대로 추적하지만 codegen side에서 HashMap ops가 value-opaque (i64 erased). 특히 Some(v) 패턴 bind가 값을 i64로 읽어들임. iter 33/34의 Never-promotion는 let-assign 경로만 커버 — Match-arm은 miss.

**구현 경로**:
1. codegen pattern.rs의 `resolve_variant_field_types`에서 HashMap.get 반환 `Option<V>` 추적 강화 (TC substitution 적용)
2. 또는 codegen Ident/Field 경로에서 `get/get_opt` 결과에 대한 타입 upgrade (infer_expr_type_inner에 HashMap method return types를 inline resolve)
3. iter 33 deref patch가 HashMap<K,V>.get 때 match_type에 Optional(V-concrete)을 보게 되면 자동 해결됨 — 확인 필요

**완료 기준**:
- vaisdb ≥ 234 (+5)
- 5개 파일 중 최소 3개 TC pass

**추정 작업량**: 100-300줄 compiler

---

## Task E — Trait `&dyn T` Method Dispatch (unlock 6+ files)

**영향 파일** (6개):
- `sql/executor/sort_agg.vais`: `self.input.next()` where input: `Box<dyn Executor>`
- `sql/executor/subquery.vais`, `window.vais`, `join.vais`, `dml.vais`: 동일
- `vector/hnsw/{bulk,delete,insert,wal}.vais`: `&mut dyn NodeStore`

**증상**: 트레잇 object method 호출시 "function not defined".

**근본 원인**: Vais codegen에 vtable-based dynamic dispatch 미구현. TC는 trait method를 알지만 codegen은 concrete struct만 처리.

**구현 경로**:
1. `crates/vais-codegen/src/inkwell/gen_method.rs` 에 dyn dispatch 추가
   - `&dyn T` 의 LLVM layout을 fat pointer (data_ptr, vtable_ptr) 로 표준화
   - Trait method call → vtable[method_idx] dispatch
2. `crates/vais-types/src/` 에서 vtable 레이아웃 + method index 메타데이터 생성
3. 트레잇 구현체 등록 시 vtable 생성

**완료 기준**:
- vaisdb ≥ 235 (+6)
- 6 파일 중 최소 3개 pass

**추정 작업량**: 500-1000줄 compiler (큰 feature)

---

## 실행 계획 (harness용)

### 우선순위
1. **Task C** (cross-file X Parser) — risk lowest, 가장 작은 compiler 변경, 4 파일 빠른 회수
2. **Task A** (MutexGuard Deref) — 5 파일, 중간 난이도
3. **Task B** (Bidirectional TC) — 3 파일, iter-50/52 fix 기반 연장
4. **Task D** (HashMap V) — 5 파일, iter-33 기반 연장
5. **Task E** (trait dyn) — 6 파일, 가장 큰 feature, 마지막

### Harness ROADMAP 작업 항목

```
Phase 6.27c:
- [ ] 6.27c.1 Cross-file X Parser method resolution (Opus direct)
  target: sql/parser/{parser,parser_expr,parser_select,mod}.vais
  done when: vaisdb ≥ 233
- [ ] 6.27c.2 MutexGuard<T> Deref / method forwarding (Opus direct)
  target: fulltext/concurrency, vector/concurrency, vector/hnsw/cow, fulltext/mod, storage/txn/deadlock
  done when: vaisdb ≥ 238
- [ ] 6.27c.3 Bidirectional TC for variant disambiguation (Opus direct)
  target: checker_expr/ + lookup_var hint propagation
  done when: vaisdb ≥ 241
- [ ] 6.27c.4 HashMap<K,V>.get Option<V> codegen type propagation (Opus direct)
  target: codegen pattern.rs + type_inference.rs
  done when: vaisdb ≥ 246
- [ ] 6.27c.5 Trait &dyn T dispatch (vtable codegen) (Opus direct)
  target: vais-codegen gen_method + vtable infra
  done when: vaisdb ≥ 252
- [ ] 6.27c.6 remaining residual files (mechanical cleanup after C.1-5)
  done when: vaisdb 261/261
```

### 각 task별 Opus direct 이유

모두 Opus direct:
- compiler cross-module 작업, design intent 중심
- impl-sonnet에게 위임할 만한 반복 작업 없음
- Phase 2.10 요요 패턴 경험: compiler 수정은 baseline 확인 필수

### 중단 조건

- 개별 task가 baseline regression 유발 → 즉시 revert, 해당 task를 deferred
- 한 task가 예상 작업량 2배 초과 → stop, re-plan

---

## 메모리에 기록해야 할 최종 상태

- Phase 6.27b 완료 기준 확장: Tier 2 203/261 완료, 추가로 229/261 stable 달성 (229가 새 base)
- Phase 6.27c 시작 전제: 229 floor 유지
- 5개 task는 blockedBy 없음 — 어떤 것부터 해도 OK. 위 순서는 risk/reward 기반 권장.
