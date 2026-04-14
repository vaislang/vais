# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **현재 버전**: 0.1.0 (Phase 190 완료 → Phase 190.5 준비)
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-04-14 (Phase 190.5 + 190.6 완료 → Phase 191 follow-up 5건 등록)

---

## Current Tasks — Phase 191: 문자열 소유권 모델 확장 (RFC-001 follow-ups)

mode: pending
iteration: 8
max_iterations: 30
session_checkpoint: 2026-04-14 세션 3 — #2a-rfc 완료 + #2a 구현 survey 완료.
  commits: 9c616289 (RFC §9), 456f12d4 (session 2 checkpoint).
  세션 3 업데이트:
    - 사용자 "전체 그냥 계속" 지시로 §9 re-sign-off 자동 기재.
    - #2a 구현 시작 시도 → code survey에서 CRITICAL 복잡 요인 발견:
      `%Vec`(generic) vs `%Vec$str`(specialized) 공존 레이아웃 ABI 충돌.
      구현 전 설계 결정 3안(i/ii/iii, ROADMAP #2a CRITICAL 섹션 참조) 중 선택 필요.
    - #2a는 pending 유지, ROADMAP에 정밀 hook 좌표 + 옵션 3안 기록.
    - auto-progress 중단, mode: pending 전환 (사용자 개입 필요).
  재개 권장:
    A) RFC-002에 레이아웃 ABI 결정 §9.7 추가 + 사용자 sign-off → #2a 구현.
    B) 승인 전 병렬 진행 가능: #9 (match-PHI fix, 중간 규모),
       #3 (trait-object str RFC), #4 (closure str RFC).

> Phase 190.5/190.6에서 RFC-001 §8 "Future work"로 명시한 범위 밖 항목들.
> 각 작업은 **독립적으로 진행 가능**하며 blockedBy 없음. 난이도/위험도 기준으로
> 순서를 제안(#1 → #2 → #3 → #4 → #5)했으나 사용자가 임의 순서 가능.
>
> **공통 참조 자료**: `docs/rfcs/RFC-001-string-ownership.md` (전체 소유권 모델),
> `crates/vais-codegen/src/inkwell/gen_stmt.rs` (scope_str_stack + var_string_slot
> 패턴 — 이 파일의 구현을 다른 소유 경로에도 적용).

### 작업

- [x] 1. vais-apps/monitor RSS plateau 자동화 스크립트 (Opus direct) ✅ 2026-04-14
  changes:
    vais-apps/monitor/bench/rss_plateau.sh (신규, 실행 가능) — ps -o rss= 1초 샘플링,
      warmup 제외 max-min delta 계산, threshold 기반 PASS/FAIL, trap으로 cleanup.
    vais-apps/monitor/docs/PERFORMANCE_TESTING.md (신규) — 사용법/exit codes/CI 가이드.
  verify:
    45s 스모크 (warmup 10s, threshold 500MB, keep_csv): monitor-server RSS 2.38MB,
    delta 0KB → PASS. 46행 CSV(헤더+45 샘플) 정상 수집, 종료 시 바이너리 + 임시파일 정리.
  skipped: 실전 300s 구동은 사용자 CI에서 실행 권장. Rust 코드 변경 없음 → E2E 영향 0.

- [ ] 2. Container-owned strings: Vec<str> / 사용자 struct str 필드 (Opus direct) — 분할 진행
  RFC: docs/rfcs/RFC-002-container-string-ownership.md ✅ Approved (user sign-off 2026-04-14, commit e1edb7bb).
  결정 요약:
    Q1: struct ownership_mask = 고정 i64 (64 필드 cap, overflow = 컴파일 에러).
    Q2: Option D — codegen shallow-drop만 heap 필드 free 가능. user Drop은 도메인 로직 전용.
         `take_field!` primitive로 명시적 ownership 이전만 허용. 구조적으로 double-free/leak 불가.
    Q5: 전체 재컴파일 허용 (pre-1.0).
  대체안 4건 기각 기록 (§4.6): tag-bit / always-clone / runtime provenance / wrapper.
  하위 작업: #2a (Vec<str> 레이아웃 + drop), #2b (struct shallow-drop + user Drop sequencing),
            #2c (nested container recursion). 각 단계 e2e + team-review. blockedBy 체이닝.
  블록: #9 (2a), #10 (2b), #11 (2c) 모두 완료 시 이 작업 close.
  [목표]: 컨테이너에 소유된 heap string이 컨테이너 destructor에서 free되도록 연결.
  [현재 상태]: Vec<str> push된 heap 문자열은 컨테이너가 drop돼도 문자열 버퍼는 leak.
  [대상 파일]:
    - crates/vais-codegen/src/vtable.rs (Vec/struct destructor emission)
    - crates/vais-codegen/src/inkwell/gen_aggregate.rs (Vec의 str push 시 소유권 이전)
    - crates/vais-codegen/src/string_ops.rs (concat 결과가 Vec.push() 인자로 갈 때 scope-drop 제외)
    - (필요 시) state.rs: Vec<PointerValue> 멤버 tracking
  [설계 질문]:
    - Vec<str>의 각 요소마다 free가 필요한지(heap) 아닌지(literal) 구분 — tag bit 없이 어떻게?
      → 후보 A: Vec<str> 전용 destructor(각 요소에 대해 free 호출) + 리터럴은 push 자체를 "clone to heap"으로 항상 승격
      → 후보 B: Vec 내부에 소유 플래그 추가 (ABI 변경)
      → 후보 C: push 시점에 heap 여부 확인 — 소유권 있으면 Vec로 transfer, 리터럴이면 strdup
    - 사용자 struct에 `name: str` 필드가 있을 때 struct destructor의 표준 drop 순서 확인
  [완료 기준]:
    - 새 e2e 테스트 phase191_container_str_drop.rs 3개 이상:
      (a) Vec<str> push + drop → leaks 0
      (b) struct { name: str } 로컬 → drop 시 name free
      (c) Vec<struct { s: str }> 중첩 — 외곽 Vec drop 시 내부 전부 정리
    - E2E baseline 유지 (2571 passed + new)
    - RFC-001 §8에서 이 항목 check 처리
  [복잡도]: 높음 — RFC 설계 결정(§4.4 수준) 필요. 사전 RFC 초안 권장.

- [ ] 3. Trait object str 반환 (Opus direct)
  [목표]: `dyn Trait` 메서드가 str을 반환할 때 소유권 규약 정립 + 구현.
  [현재 상태]: RFC-001 §5.3에서 "out of scope"로 명시, 현재 호출 시 동작 불확정.
  [대상 파일]:
    - crates/vais-codegen/src/trait_dispatch.rs (vtable 메서드 호출)
    - crates/vais-codegen/src/vtable.rs (메서드 시그니처 정규화)
    - (RFC 업데이트 필요): docs/rfcs/RFC-002-trait-object-string.md (신규)
  [설계 질문]:
    - trait 메서드의 str 반환은 callee owns(Owned) vs callee lends(Borrowed) 중 어디? Rust의 `-> String` vs `-> &str` 대응.
    - vtable 호출 후 caller가 받은 pointer의 drop 책임자 명확화
  [완료 기준]:
    - RFC-002 작성 + 사용자 리뷰 + 구현
    - e2e 테스트: trait 메서드가 concat 결과 반환 후 호출자가 2번 사용 → 내용 동일 + 종료 시 leaks 0
    - E2E baseline 유지
  [복잡도]: 중~높음 — trait 시스템과 얽힘. pre-RFC 필수.

- [ ] 4. 클로저 캡처된 str 소유권 (Opus direct)
  [목표]: `||` 클로저가 str을 캡처할 때 소유권/수명 규약 + UAF 방지.
  [현재 상태]: RFC-001 §7에서 "closures + long-running concat 안전 문제" 명시, 현재 alias-by-copy로 UAF 잠재 위험.
  [대상 파일]:
    - crates/vais-codegen/src/inkwell/gen_expr/lambda.rs / crates/vais-codegen/src/lambda_codegen.rs
    - RFC 업데이트: docs/rfcs/RFC-003-closure-string-capture.md (신규)
  [설계 질문]:
    - 캡처: move(소유권 이전) vs by-ref(& 수명)
    - Rust `move` 키워드 대응 검토 (Vais에 키워드 없음 — 기본 동작 결정 필요)
    - FnOnce/FnMut/Fn 분류가 Vais에 존재하는지 확인 필요
  [완료 기준]:
    - RFC-003 작성 + 사용자 리뷰 + 구현
    - e2e: let s = "a"+"b"; let f = || println(s); f(); f() → 동일 출력 2번 + 종료 시 leaks 0
    - E2E baseline 유지
  [복잡도]: 높음 — 클로저 런타임(captures struct)과 얽힘. pre-RFC 필수.

- [x] 5. Text-IR backend scope-drop parity (impl-sonnet + Opus fix) ✅ 2026-04-14
  changes:
    state.rs: scope_str_stack (Vec<Vec<String>>), scope_drop_label_counter fields.
    init.rs: init new fields on FunctionContext construction.
    stmt.rs: enter_scope also pushes scope_str_stack; new exit_scope_str;
      new generate_string_scope_cleanup (null-check+free IR with __sd_* labels);
      clear_alloc_tracker resets the new fields.
    stmt_visitor.rs: visit_block_stmts resolves last_value's transfer_slot,
      emits string-scope cleanup before Named-type drops, transfers slot to
      outer frame on natural block exit. Terminated paths discard the frame
      (already cleaned by Return/Break/Continue).
    string_ops.rs: concat/substring/push_str register each new slot in the
      topmost scope_str_stack frame alongside string_value_slot.
    crates/vaisc/tests/e2e/phase191_text_ir_scope_drop.rs (신규, 5 tests).
    crates/vaisc/tests/e2e/main.rs: module registration.
  impl-sonnet + Opus fix note:
    Initial impl removed freed slots from `alloc_tracker` inside scope
    cleanup. `track_alloc_with_slot` numbers new slots by
    `alloc_tracker.len()`, so removal caused post-loop concat to reuse
    `%__alloc_slot_0`, producing `multiple definition of local value`
    LLVM errors. Fix (Opus): leave `alloc_tracker` untouched, only remove
    from `string_value_slot`; the `store i8* null` lets function-exit
    cleanup skip safely. `still_tracked` also switched from `alloc_tracker`
    to `string_value_slot` to mirror inkwell's generate_block.
  verify:
    cargo build --workspace --exclude vais-python --exclude vais-node: green.
    cargo test -p vaisc --test e2e: 2576 passed / 0 failed (676s),
      baseline 2571 + 5 phase191 tests. No regressions.
    phase190_str_concat_drop: all green.
  rfc: RFC-001 §5.4 "single implementation path" — text-IR and inkwell now
    share the block-scope drop structural pattern.
  team-review_2026-04-14: Approve, 0 Critical, 4 Warnings, 3 Info.
    W3 (doc drift on `exit_scope`) fixed inline. W1/W2/W4 → follow-up items
    below (#6/#7/#8). Quote paths kept for traceability.

### Phase 191 #2 하위 구현 작업 (RFC-002 Approved 2026-04-14, **§9 re-review required 2026-04-14**)

- [x] 2a-rfc. RFC-002 보정 (§9 scope correction, Opus direct) ✅ 2026-04-14
  drift_found:
    std/vec.vais:51 Vec<T>는 4필드 {data,len,cap,elem_size} (RFC §2.1 3필드 가정과 불일치).
    std/vec.vais:238 이미 user F drop(&self) 존재 (RFC §2.1 "no Drop" 가정과 불일치).
    push는 user method (std/vec.vais:186), codegen intrinsic 아님 (RFC §2.2 가정과 불일치).
  changes:
    docs/rfcs/RFC-002-container-string-ownership.md (+211 lines):
      §2: "corrected 2026-04-14" — 실제 4필드 + user drop 상태 반영.
      §4.1: Vec<str> 레이아웃 4필드 → 5필드 (`+owned: i64`), 비-str Vec 불변.
      §4.3: push는 call-site wrapping (path α) — stdlib 미수정, codegen이
        call 주변에 owned-bit set + slot transfer IR inject.
      §4.4: user drop과 충돌 없는 prelude helper `__vais_vec_str_shallow_free`
        (Vec<str>에만, user drop 전 실행). struct는 기존대로 postlude 유지.
      §9 (신규): scope correction 상세, drift table, path β 기각, drop-ordering
        asymmetry 해설, monomorphization 주석, re-sign-off 요청.
  verify:
    원문 §1/§3/§5/§6/§7/§8 무변경. 구조: 9개 ## 섹션 (wc -l: 369 → 580).
    Rust 코드 변경 0 — 빌드/테스트 영향 없음.
  gate: **user re-sign-off 필요** (§9.6). 2a 구현은 approval 후 착수.
  iter: 7

- [ ] 2a. Vec<str> 레이아웃 + owned bitmap + __drop_Vec_str (Opus direct)
  [참조]: RFC-002 §4.1, §4.4 **(§9 corrected)**, §9 integration notes
  [대상 파일]:
    - crates/vais-codegen/src/vtable.rs (synthesize __vais_vec_str_shallow_free + splice into drop sequence, NOT replace Vec.drop)
    - crates/vais-codegen/src/inkwell/gen_aggregate.rs (Vec<str>.push call-site wrapping)
    - crates/vais-codegen/src/string_ops.rs (alloc_slot transfer on push)
    - crates/vais-codegen/src/state.rs (pending_return_skip_container)
    - (monomorphization) Vec<str> 5필드 레이아웃 선택 지점 (구현 시 정확한 hook 확정)
  [완료 기준]: RFC-002 §6 tests (1) vec_str_push_drop_no_leak, (2) mixed_literal, (6) return_transfers.
    E2E baseline 유지 (2582 + 3 new).
  [복잡도]: 높음 — 모노모피제이션별 레이아웃 변경, ABI 동일.
  blockedBy: #5 완료 (done), #2a-rfc user re-sign-off.

  [구현 survey 2026-04-14 세션 3 (Explore agent + 수동 확인)]
  — 다음 세션이 바로 구현 착수할 수 있도록 hook 좌표 확정:

  (A) **Layout hook (text-IR backend)**:
      crates/vais-codegen/src/function_gen/generics.rs:57-85 —
      `generate_specialized_struct_type`가 generic struct monomorphization마다
      호출되며, fields 벡터에 ("owned", ResolvedType::I64) append, llvm_fields에
      "i64" push하면 `%Vec$str = type { i64, i64, i64, i64, i64 }` emit됨.
      조건: `generic_struct.name.node == "Vec"` AND `inst.type_args[0] ==
      ResolvedType::Str` (아래 CRITICAL 주의).
  (A') **Layout hook (inkwell backend)**:
      crates/vais-codegen/src/inkwell/gen_types.rs:162-200
      (`define_specialized_struct`) — 같은 조건으로 5필드 분기.

  (B) **Push-site wrapping**:
      crates/vais-codegen/src/expr_helpers_call/method_call.rs:620-691 의
      Vec_ elem_size patch 로직과 **같은 블록**에 추가 가능.
      조건: `full_method_name.starts_with("Vec_push$str")`.
      삽입 위치: line 691 직후 (call emission 직전).
      필요 IR:
        1) owned bitmap-grow: helper fn `__vais_vec_str_owned_grow(%Vec*, i64 new_cap)` 호출.
        2) 현재 self.len 로드 → index i.
        3) rvalue의 alloc_slot을 string_value_slot에서 lookup:
           — found (heap-owned): `__vais_vec_str_owned_set(%Vec*, i64 i)` 호출,
                                  string_value_slot + scope_str_stack 제거,
                                  alloc_tracker entry에 null store.
           — not found (literal/borrowed): no-op.

  (C) **Scope-exit drop prelude splice**:
      crates/vais-codegen/src/stmt.rs:828-902 `generate_scope_drop_cleanup`.
      line 862 for-loop **이전**에:
        droppable iter하면서 `type_name == "Vec$str"` 또는 (더 robust하게)
        StructInfo 조회하여 fields에 "owned" field가 있는 Vec 파생 타입이면
        `call void @__vais_vec_str_shallow_free(%Vec$str* %{llvm_name})` emit.
      순서: prelude → 기존 user Vec.drop → 완료.

  (D) **Helper function emission**:
      vtable.rs 또는 전용 신규 파일 (예: crates/vais-codegen/src/container_drop.rs).
      Module-level emit (generate_module_with_instantiations에서 한 번):
        define void @__vais_vec_str_owned_set(%Vec$str*, i64)
        define void @__vais_vec_str_owned_grow(%Vec$str*, i64)
        define void @__vais_vec_str_shallow_free(%Vec$str*)
      Vec<str> 인스턴스화가 존재할 때만 emit (instantiations 스캔).

  (E) **Return-transfer 확장**:
      stmt.rs의 `pending_return_skip_slot` 옆에 `pending_return_skip_container`
      추가 (state.rs). Return value가 %Vec$str* 이면 그 Vec의 data/owned 버퍼를
      function-exit 청소 대상에서 제외. Test case (6) vec_str_return_transfers.

  (F) **Alloc_tracker index 충돌 회피**:
      Phase 191 #5에서 검증된 패턴 — transfer 시 entry는 유지, 포인터만 null store.
      owned bitmap transfer도 같은 규약 준수.

  ⚠️ **CRITICAL 복잡 요인 — 다음 세션 시작 시 먼저 확인**:
      현재 text-IR codegen은 `%Vec`(generic fallback)과 `%Vec$T`(specialized)를
      **공존**시킨다 (method_call.rs:650, helpers.rs:438, loops.rs:289 등이 모두
      `%Vec` GEP를 emit). 5필드 `%Vec$str`는 `%Vec` 레이아웃(4필드)과 **binary-
      incompatible**이므로, Vec<str> self-pointer가 `%Vec*`로 bitcast되는 경로에서
      GEP field-index 3 (elem_size) 이후로 field-index 4 (owned)를 읽으면 OOB 발생.
      옵션:
        (i) 모든 `%Vec` GEP 사이트를 `%Vec$str` 전용 경로로 분기 — 침습적.
        (ii) 5필드 공통 `%Vec` 레이아웃으로 통일하고 비-str Vec은 owned를 dead
             field로 방치 — ABI 변경(§4.1 "non-str Vec 불변" 약속 위배).
        (iii) Vec<str>만 별도 alias `%Vec_str_repr`로 관리하고 self-pointer를
             use site마다 bitcast — 복잡하지만 가장 invariant-preserving.
      결정 필요 → 다음 세션 첫 번째 작업.

  iter_note: 세션 3(iter 8)에서 survey + 정밀 hook 좌표 확보 후 구현 착수 전에
    CRITICAL 복잡 요인 발견 → 사용자 의사결정 필요하여 fresh session으로 연기.

- [ ] 2b. struct shallow-drop + ownership_mask + user-Drop sequencing (Opus direct)
  [참조]: RFC-002 §4.2 Option D
  [대상 파일]:
    - crates/vais-codegen/src/vtable.rs (auto-emit __drop_shallow_{Struct})
    - crates/vais-codegen/src/inkwell/gen_aggregate.rs (struct literal ownership transfer)
    - crates/vais-codegen/src/trait_dispatch.rs (drop sequence: user drop() → shallow-drop)
    - crates/vais-codegen/src/stmt.rs (struct drop emission path)
  [설계]:
    - user drop() 호출 후 shallow-drop 무조건 emission
    - ownership_mask i64 필드: 비트 i = 필드 i가 heap-owned
    - `take_field!` macro/builtin 스펙 작성 (구현은 별도 follow-up 가능)
  [완료 기준]: RFC-002 §6 tests (3) struct_str_field_drop, (4) struct_user_drop_takes_ownership.
    double-free 구조적 불가 증명: user가 free를 호출할 API 없음 검증.
  [복잡도]: 높음 — drop sequencing + bitmap + take_field! 스펙.
  blockedBy: #2a.

- [ ] 2c. Nested container recursion (Vec<Vec<str>>, Vec<struct{str}>) (Opus direct)
  [참조]: RFC-002 §5 Q3
  [대상 파일]: vtable.rs (모노모피제이션 recursion), drop_registry
  [완료 기준]: RFC-002 §6 test (5) nested_vec_of_struct_str. 외곽 Vec drop이 모든 내부 str 정리.
  [복잡도]: 중간.
  blockedBy: #2a, #2b.

### Phase 191 follow-ups (team-review 2026-04-14 발견)

- [x] 6. Break/Continue 경로 string scope cleanup (Opus direct) ✅ 2026-04-14
  design:
    LoopLabels/LoopContext에 scope_str_depth 스냅샷 추가. loop 진입 시
    scope_str_stack.len() 저장 → break/continue 시 [loop_depth..top) 프레임을
    null-check + free IR로 해제. 프레임은 pop하지 않음 — block-exit의
    terminated=true 경로가 discard하고, continue 재진입 시 frame.clear()로
    빈 프레임만 보이므로 redundant free 없음.
  changes:
    crates/vais-codegen/src/types/mod.rs: LoopLabels.scope_str_depth 필드 추가.
    crates/vais-codegen/src/generate_expr_loop.rs,
      generate_expr/loops.rs (×3 sites), expr_helpers_control.rs (×2 sites):
      6개 LoopLabels push 지점에 scope_str_depth 스냅샷 전달.
    crates/vais-codegen/src/stmt.rs: generate_loop_scope_cleanup 헬퍼 추가.
      Stmt::Break/Stmt::Continue 경로에 cleanup emission 삽입.
    crates/vais-codegen/src/stmt_visitor.rs: generate_break_stmt/
      generate_continue_stmt에 cleanup emission 삽입 (visitor 경로).
    crates/vais-codegen/src/inkwell/generator.rs: LoopContext.scope_str_depth.
    crates/vais-codegen/src/inkwell/gen_stmt.rs: 3개 LoopContext push 사이트에
      스냅샷 전달. generate_break/generate_continue에 emit_loop_scope_cleanup
      호출 삽입 + 신규 헬퍼 (emit_free_slot + string_value_slot 스크럽 +
      frame.clear()).
    crates/vaisc/tests/e2e/phase191_text_ir_scope_drop.rs: 2 new regression
      tests (e2e_phase191_break_frees_scope_strings, e2e_phase191_continue_
      frees_scope_strings) — 100k 이터 L 루프 + B/C 경로 leak-free 확인.
  verify:
    cargo clippy --workspace --exclude vais-python --exclude vais-node: green.
    cargo test -p vaisc --test e2e phase191: 8/8 (baseline 6 + 2 new).
    cargo test -p vaisc --test e2e: 2579/0 (baseline 2577 + 2 new, 690s).
  rfc: RFC-001 §5.4 단일 경로. out-of-scope: break-with-value 소유권 전이
    (Return의 pending_return_skip_slot과 유사한 메커니즘 필요 — follow-up).

- [x] 7. transfer_slot lookup Ident fallback (impl-sonnet) ✅ 2026-04-14
  changes:
    crates/vais-codegen/src/stmt_visitor.rs (visit_block_stmts 87-112):
      two-step lookup — SSA key → fallback to var_string_slot by Ident name
      when last non-terminator Stmt::Expr(Expr::Ident(name)).
    crates/vais-codegen/src/inkwell/gen_stmt.rs (generate_block 44-71):
      symmetric fallback. struct-value key → var_string_slot.get(name).copied().
    crates/vaisc/tests/e2e/phase191_text_ir_scope_drop.rs:
      new transfer_slot_ident_fallback_no_uaf test (bare Ident tail referring to
      heap concat local). Guards future alloca-backed Str representation.
  verify:
    cargo clippy --workspace --exclude vais-python --exclude vais-node: green.
    cargo test -p vaisc --test e2e phase191: 6/6 (baseline 5 + 1 new).
    cargo test -p vaisc --test e2e: 2577/0 (baseline 2576 + 1 new, 681s).
  rfc: future-proofs RFC-001 §5.4 block-scope drop against SSA representation
    drift (alloca-backed `let mut s`).

- [x] 8. Phase 191 #5 보강 E2E — substring/push_str/match/break coverage (impl-sonnet) ✅ 2026-04-14
  changes:
    crates/vaisc/tests/e2e/phase191_text_ir_scope_drop.rs: +4 tests
      (a) e2e_phase191_loop_body_substring_no_leak — 100k L + .substring(2,7)
      (b) e2e_phase191_loop_body_push_str_no_leak — 100k L + .push_str
      (c) e2e_phase191_match_arm_concat_phi — #[ignore], 선결 #9 필요
      (d) e2e_phase191_break_before_concat_no_leak — #6 후 break 경로 회귀.
  surfaced_bug:
    (c) 매치 arm 경로에서 text-IR PHI가 {ptr,i64} fat-ptr과 raw i8*를 섞어
      emission → `'%t6' defined with type '{ ptr, i64 }' but expected 'ptr'`.
      Phase 190.6 if-expr PHI 통합과 유사한 수정 필요. 신규 작업 #9 등록.
  verify:
    cargo clippy --workspace --exclude vais-python --exclude vais-node: green.
    cargo test -p vaisc --test e2e phase191: 11/0, 1 ignored.
    cargo test -p vaisc --test e2e: 2582/0, 1 ignored (baseline 2579 + 3 new).

- [ ] 9. Match-arm string PHI fat-ptr unification (text-IR) (Opus direct)
  [출처]: #8의 e2e_phase191_match_arm_concat_phi (#[ignore] 상태).
  [상태]: 매치 식이 str을 반환할 때 text-IR이 arm별로 다른 형태를 emit.
    arm 1의 let-bound concat은 fat ptr `{ptr, i64}`로, default arm은 raw `i8*`로
    PHI의 입력이 되어 LLVM 검증 실패.
    Phase 190.6 if-expr PHI 통합(RFC-001 §4.6)과 동형 문제.
  [대상 파일]:
    - crates/vais-codegen/src/control_flow/ (match 경로)
    - crates/vais-types/src/checker_expr.rs (match arm 타입 단일화)
    - crates/vais-codegen/src/inkwell/gen_expr/ (inkwell match — 상태 확인 필요)
  [완료 기준]:
    - e2e_phase191_match_arm_concat_phi에서 #[ignore] 제거 후 pass
    - 기존 if-expr PHI 테스트 무회귀
  [복잡도]: 중~높음. arm별 fat-ptr widening 설계 필요.

### 전략

  strategy: 독립 작업 — 파일 중첩 없음 (각각 다른 경로)
  execution: 난이도/위험도 기준 제안 순서 #1(쉘, 가장 안전) → #5(text-IR, 참조 구현 있음) →
             #2/3/4(RFC 필요, Opus direct). RFC 작업은 사전 설계 초안 + 사용자 리뷰 필수.
  blockedBy: 없음 (모두 병렬 가능하지만 충돌 방지 및 리뷰 부담 고려해 순차 권장)
  mode_log: auto 선택 (사용자), 순차 실행 (#1→#5→#2→#3→#4). #2/#3/#4 RFC 단계에서 리뷰 대기.
  strategy_log:
    #1: Direct delegate fast-path — 쉘 + docs, Rust 변경 없음, foreground impl-sonnet. ✅ iter 1
    #5: Sequential impl-sonnet background — 참조 구현(inkwell) 존재. ✅ iter 2
         (impl-sonnet hit tool budget after core impl; Opus finished test file +
         fixed alloc_tracker slot-id collision regression caught by new e2e.)
    #2/#3/#4: Opus direct — RFC + design 의사결정 inseparable, 사용자 리뷰 gating.
    #7: Sequential impl-sonnet background (iter 4) — 낮은 복잡도 패턴 매칭 추가.
        양 백엔드(text-IR + inkwell) transfer_slot 계산에 Ident fallback 삽입. ✅ 6a47c582
    #6: Sequential Opus direct (iter 5) — loop_stack 프레임 집계 설계 필요.
        opus_direct: Return/Break/Continue ownership-transfer 불변성 설계-구현 inseparable.
        파일 #7과 겹침(stmt_visitor.rs + inkwell/gen_stmt.rs) → 다른 작업과 병렬 불가. ✅ 5bbf8a9a
    #8: Sequential impl-sonnet background (iter 6) — 기존 테스트 패턴 복제. ✅
        3/4 pass, 1 test hit pre-existing match-PHI codegen bug → #[ignore],
        follow-up #9 등록. Agent는 "PROMISE: COMPLETE" 빠뜨리고 bug 분석 중 반환 —
        lead가 scope 조정 + 테스트 #[ignore] 처리로 마무리.
        #2a(Opus direct, large RFC work)는 #8 후 체크포인트 → fresh session 권장.
    #2a-rfc: Opus direct foreground (iter 7, 신규 sub-task). ✅ 완료.
        사전 verify에서 std/vec.vais Vec<T>=4필드{data,len,cap,elem_size} +
        user drop() 이미 존재 → RFC-002 §2/§4 "3필드" 가정 drift 발견.
        RFC §2/§4.1/§4.3/§4.4 보정 + §9 (scope correction, 211줄 추가) 신설.
        구현(#2a)은 사용자 re-sign-off gating — §9.6 확인 필요.
        auto-progress 여기서 일시 정지 (user gate).

progress: 6/10 resolved (#1, #5, #6, #7, #8, #2a-rfc complete; #2a/#2b/#2c pending — blocked on #2a-rfc re-sign-off; #9 new surfaced by #8); RFC-002 **re-review pending §9**

---

## Phase 190.5 + 190.6 완료 기록 (2026-04-14, commit 57697a74)

> 이 섹션은 이력 참조용. 신규 작업은 위 "Current Tasks" 참조.
> RFC-001 sign-off 완료. E2E 2571/0, 회귀 테스트 8개 (phase190_str_concat_drop.rs).

---

## Phase 190.5: 문자열 메모리 안정화 (drop-tracking) — 완료 상세

mode: pending
iteration: 1
max_iterations: 30

> Phase 190의 나머지 작업 #6만 독립 Phase로 분리. `/clear` 후 `/harness`로 이어받을 수 있도록 아래 컨텍스트를 그대로 사용.
> 나머지 Phase 190 완료 이력은 "## Phase 190 완료 기록" 섹션 참조.

### 배경

Phase 190 작업 중 문자열 연산 경로(push_str/as_bytes, concat)가 런타임에서 동작함을 확인했으나, `+` concat 체인이 중간 결과 버퍼를 해제하지 않아 장기 실행 서버(vais-monitor 등)에서 메모리가 단조 증가한다. 현재 `__vais_str_concat`은 새 버퍼를 할당하고 반환하지만, 이전 피연산자의 소유권(drop) 추적이 codegen 레벨에서 일관적이지 않아 이중 해제 위험 때문에 해제 자체가 보류되어 있다.

**왜 위험도 높음**:
- `vtable.rs`의 destructor 발행 경로와 `string_ops.rs`의 concat/슬라이스 코드가 결합돼 있음.
- 현재 alloc_slot 패턴은 가변 재할당(`store i8* %tNN, i8** %__alloc_slot_X`)으로 최신 값만 보존 — 중간 값들은 "소유되지 않음"으로 간주되어 drop이 누락됨. 단순히 drop을 추가하면 여전히 참조되는 버퍼를 해제해 UAF 발생.
- ownership tracking을 드물게 잘못 건드리면 trait object / Vec / 사용자 struct destructor가 연쇄 깨짐. E2E 2563건 중 수십 건이 회귀 가능.

### 작업

- [x] 1. 문자열 concat drop-tracking 리팩토링 (Opus direct) — 2026-04-14
  RFC: docs/rfcs/RFC-001-string-ownership.md (pre-implementation, §4.4 = scope-exit 보존 선택)
  3-layer fix:
    (a) Return-value ownership transfer: `pending_return_skip_slot`로 반환 슬롯 exclude.
    (b) Concat-chain intermediate free: LHS가 tracked concat 결과면 즉시 free(소비 증명).
    (c) Block-scope drop: `scope_str_stack`으로 루프 body 종료마다 잔여 concat 결과 free.
  changes:
    - state.rs / init.rs: string_value_slot + pending_return_skip_slot 필드
    - stmt.rs: generate_alloc_cleanup 재활성화 (skip-slot 지원) + track_alloc_with_slot
    - string_ops.rs: concat/substring/push_str 슬롯 기록, emit_intermediate_free
    - stmt_visitor.rs + function_gen/codegen.rs: 반환 경로 4곳 소유권 전이
    - inkwell/generator.rs: string_value_slot + pending_return_skip_slot + scope_str_stack
    - inkwell/gen_stmt.rs: emit_free_slot, mark_return_ownership_transfer, generate_block 래핑
    - inkwell/gen_expr/binary.rs: 반환 fat_ptr → slot 매핑 + intermediate free
    - inkwell/gen_function.rs + gen_special.rs: 구현·메서드 반환 경로 6곳 전이
  verification:
    - 루프 100k iter `a+b+c+d` 테스트: 299,997 → 99,999 → **0 leaks**
    - `leaks --atExit`: `Process: 188 nodes malloced / 0 leaks for 0 total leaked bytes`
    - E2E: 2563 passed / 0 failed (623.53s, baseline와 동일)
  out of scope (follow-up):
    - 컨테이너(Vec<str>, struct<str>) 내부 문자열 소유권
    - 클로저 캡처된 str
    - N-ary concat intrinsic / SSO / interning
    - call site str 반환값 caller-side tracking (Case 3 RFC §4.6)

### 접근 제안 (1차 분석 산물)

1. 현재 구조 매핑 — `string_ops.rs`의 concat 진입점에서 반환 값이 `alloc_slot`에 저장될 때 이전 슬롯 포인터를 drop 대상 리스트에 등록하는 경로가 있는지 확인. 없으면 `fn_ctx.drop_list`류 상태를 신설할지 기존 RAII 경로 재사용할지 결정.
2. 안전 경계 — literal (`@str.N` globals)과 heap-allocated 구분자 필요. `build_str_fat_ptr`가 literal로부터 생성한 fat pointer와 `strdup`/concat 경로로 생성한 포인터가 같은 표현을 쓰므로, tag bit 또는 별도 타입 스코프 도입 고민.
3. destructor 쓰기 순서 — 블록 종료 시점에 "이 스코프에서 생성된 중간 concat 결과"만 drop. `inkwell` 백엔드의 기존 RAII 인프라(이미 Vec/struct에 대해 있음)를 재사용할 것인지, text-IR 백엔드에 독자 drop emit을 추가할 것인지 결정.
4. RFC 초안 (구현 전 필수) — CLAUDE.md Phase 158 엄격 타입 전환 규칙과 동일한 수준으로 "문자열 소유권 모델"을 문서화. 이후 구현은 이 RFC 기준으로만 변경.

### 선결 조건

- 이 Phase를 시작하기 전에: `git pull` + `cargo test -p vaisc --test e2e 2>&1 | tail -3`으로 2563 baseline 재확인.
- valgrind/ASan 사용 가능한지 확인 (`cargo install cargo-valgrind` 혹은 macOS의 `leaks` 커맨드).
- `vais-apps/monitor` 장기 실행 harness(5분 이상 구동 + RSS 측정 스크립트) 준비 필요.

### 전략

  strategy: Opus direct sequential + pre-RFC 단계 (구현 전 RFC 초안 작성 → 사용자 리뷰 → 구현).
  opus_direct: design-impl inseparable — ownership 모델 전체 재정의, 단순 delegate 위험.

progress: 0.7/1 (팀 리뷰 Request Changes — let-bound return UAF, Phase 190.6으로 이어짐)

---

## Current Tasks — Phase 190.6: let-bound str return UAF fix

mode: pending
iteration: 1
max_iterations: 10

> Phase 190.5의 1차 fix(return-transfer + intermediate-free + scope-drop)는 루프 누수는 해결했으나,
> team-review(2026-04-14)에서 Critical UAF가 재현됨:
>   F build() -> str { msg := "a" + "b"; R msg } ← msg 로드 시 새 SSA → skip_slot 매칭 실패
>   → 반환 직전 slot 해제 → 반환 fat ptr의 i8* dangling
> 누수 → UAF로 악화. 즉시 해결 필요.

### 작업

- [x] 1. let-bound/assign/PHI 경로 소유권 전이 + 회귀 테스트 (Opus direct) — 2026-04-14
  [대상 파일 — inkwell]:
    - crates/vais-codegen/src/inkwell/generator.rs: var_string_slot 필드 추가
    - crates/vais-codegen/src/inkwell/gen_stmt.rs: Stmt::Let / Assign 훅, Return(Ident) 매칭
    - crates/vais-codegen/src/inkwell/gen_function.rs: implicit-return Ident 매칭 (6곳)
    - crates/vais-codegen/src/inkwell/gen_special.rs: method 반환 경로
    - crates/vais-codegen/src/inkwell/gen_expr/ (phi / if-as-expr 소유권 merge)
  [대상 파일 — text-IR (해당 시)]:
    - crates/vais-codegen/src/state.rs: var_string_slot
    - crates/vais-codegen/src/stmt_visitor.rs + stmt.rs: 동일 훅
    - text-IR 백엔드 전체에 scope-drop 부재 — 별도 TODO (deprecation 공지 고려)
  [회귀 테스트 (필수)]:
    - crates/vaisc/tests/e2e/phase190_str_concat_drop.rs:
      (a) let_bound_return: F build() -> str { msg := "a"+"b"; R msg } + assert stdout="ab"
      (b) callee_reused_twice: s := build(); println(s); println(s) + assert 동일 출력 2번
      (c) if_expr_return: I c { "a"+"b" } E { "c"+"d" } 반환 + assert 올바른 값
      (d) loop_concat_100k: 100k iter a+b+c+d + assert_exit_code 0
      (e) assign_rebind: s = "a"+"b"; s = "c"+"d"; assert 최종 "cd"
  [완료 검증]:
    - phase190_str_concat_drop 8개 케이스 모두 통과 (direct return / let-bound return /
      caller uses twice / if-expr true/false / concat chain / loop crash-free / push_str)
    - E2E 전체 baseline 유지 (2563 passed / 0 failed)
    - RFC-001 sign-off 업데이트 완료
  changes:
    - state.rs / init.rs: pending_return_skip_slot → Vec<String>, var_string_slot,
      var_string_slots_multi, phi_extra_slots
    - stmt.rs: generate_alloc_cleanup Vec 기반 skip, clear 확장
    - stmt_visitor.rs: let hook + return Ident fallback (multi-slot 우선)
    - function_gen/codegen.rs: 4개 implicit-return 경로 PHI extras 지원
    - control_flow/if_else.rs + expr_helpers_control.rs: PHI에서 ownership merge
      (string_value_slot + phi_extra_slots)
    - inkwell/generator.rs + gen_stmt.rs + gen_function.rs + gen_special.rs:
      text-IR과 병행 (Vec<PointerValue> skip_slots, var_string_slot,
      var_string_slots_multi, phi_extra_slots + resolve_return_owning_slots
      helper, Ident-fallback으로 mark_return_ownership_transfer_expr/_block)
    - inkwell/gen_expr/binary.rs: 기존과 동일(intermediate-free 유지)
    - docs/rfcs/RFC-001-string-ownership.md: sign-off 섹션 업데이트
    - tests/e2e/phase190_str_concat_drop.rs: 8개 회귀 케이스 추가
    - tests/e2e/main.rs: 새 모듈 등록

### 전략

  strategy: Opus direct sequential (design-impl inseparable — 소유권 추적 구조 확장)
  opus_direct: 팀 리뷰가 Critical UAF를 입증했고, 변수-슬롯 매핑 설계가 SSA/PHI/assign과 얽힘 — 단순 delegate 위험.

progress: 1/1 ✅

---

## Phase 190 완료 기록 (2026-04-13)

> 이 섹션은 이력 참조용. 신규 작업은 위 "Current Tasks" 참조.

### DX Quick Wins
- [x] 1. str.push_str() 메서드 추가 (impl-sonnet) — 커밋 f5729869
- [x] 2. str.as_bytes() 메서드 추가 (impl-sonnet) — 커밋 f5729869
- [x] 3. Vec[i].field 직접 접근 지원 (Opus direct) — 커밋 e37af621
  changes: gen_match.rs `infer_struct_name` — `Expr::Index` 케이스 추가. Vec<S>/&[S]/[S;N] 모두 지원.
  tests: phase190_vec_field_access.rs 3건 추가.

### 런타임 라이브러리 (vais-monitor 링크 성공 목표)
- [x] 4. vais-monitor 런타임 stub 라이브러리 생성 (impl-sonnet) — monitor_runtime.c (1660 LOC, iter 15)
  verification: 37/37 clang OK (#7 수정 후).
- [x] 5. vais-monitor ICE "await on non-Future" 잔여 해결 (Opus direct) — 커밋 258618d2
  changes: type_inference.rs — generic-resolved Call 경로에서 is_async → Future<T> 래핑 적용.
  verification: vais-monitor 37/37 OK, 링크/실행 OK. E2E 2563 passed / 0 fail (+2 phase190_generic_async).

### 신규 분리 작업
- [x] 7. vais-monitor main_anomaly.ll codegen 시그니처 불일치 해결 (Opus direct) — 커밋 50e00e70
  changes: generate_expr_call.rs — i1 인자 coercion 추가 (icmp ne 트렁케이션). i1↔wider-int 양방향 확장.
  verification: vais-monitor 37/37 clang + 링크 + 실행 OK. E2E 2561 → 2563 (+5 테스트 누적).

---

## 🗺️ 중장기 발전 로드맵 (2026-04-10 수립)

> **현재 위치**: Phase 190 (DX Quick Wins + 런타임 라이브러리)
> **목표**: v0.2.0 안정 릴리스 (다중 파일 프로젝트가 안정적으로 컴파일됨)

### 기존 히스토리에서 배운 것

Phase 141~188에 걸쳐 동일 근본 원인("i64 erasure")을 점진적으로 수정해옴. 핵심 교훈:

| 이미 해결된 것 | Phase | 상태 |
|---------------|-------|------|
| R1 Monomorphization 기본 구조 (specialized 함수 생성, `$` mangling) | 141~146 | ✅ 동작 |
| R2 IR Postprocessor → 컴파일러 자체 생성 전환 | 142~148 | ✅ 전환 완료 |
| compute_sizeof Named type 해석 | 150 | ✅ struct 필드 합산 |
| TC expr_types → codegen 연결 | 150 | ✅ 타입 정보 전달 |
| match phi value/pointer 통일 | 150 | ✅ alloca+store 변환 |
| Bool↔I64 coercion 제거 (TC) | 151 | ✅ unification.rs |
| str fat pointer `{i8*,i64}` 전환 시작 | 77~78 | ✅ C ABI 자동 변환 |
| cross-module struct 필드 resolution | 187 | ✅ load_module_with_imports |
| 서브디렉토리 import fallback | 187 | ✅ source_root |
| Vec<f32> 제네릭 타입 보존 | 182 | ✅ substitution 조회 우선 |
| VaisDB codegen deeper 에러 42→6→0 (표면 레이어) | 172~181 | ✅ 각 Phase별 해소 |

| 반복되는 패턴 (양파 깊이) | 교훈 |
|--------------------------|------|
| 매 Phase마다 "해결" → deeper 에러 노출 (172→173→177→180→181→182→188) | 점진적 coercion 추가는 끝이 없음 |
| i64 fallback 부분 제거 시도 (Phase 17~19, 141~146) → 특정 경로만 수정 | 전체 codegen 경로 통합이 안 됨 |
| coercion 토글 (Phase 151 제거 → 이후 재추가 필요) | 근본 해결 없이 제거하면 다시 필요해짐 |
| ir_fix.py 500+ iterations → bus error (Phase 150) | 후처리는 근본 해결이 아님 |

**핵심 진단**: Monomorphization 기본 구조는 Phase 141~146에서 완성. specialized 함수가 생성되지만, **generic body 내부에서 i64로 erased된 값이 specialized 함수에 전달되는 불일치가 잔존**. Phase 172~188의 deeper 에러들은 모두 이 불일치의 변형.

### 의존 관계

```
Phase 190: DX Quick Wins + 런타임 라이브러리 (현재)
    ↓
Phase 191: i64 fallback 잔존 경로 전량 제거
    ├─ generic body의 i64 erased 값 → concrete type 변환 경로 통합
    ├─ str 표현 `{i8*, i64}` 통일 (Phase 77~78 전환 미완료 경로)
    └─ TC coercion 잔여분 제거 (Phase 151 이후 재추가된 것들)
    ↓
Phase 192~193: 안정화 & 실전 검증
    ├─ vais-monitor 전체 컴파일 + 실행
    ├─ VaisDB 95%+ 테스트 통과
    └─ Cross-module 해킹 H5~H10 제거
    ↓
Phase 194: v0.2.0 릴리스
    ↓
장기: 생태계 & 확장 (195+)
```

---

### Phase 191 (예정): i64 fallback 잔존 경로 전량 제거

> **목적**: Phase 141~146 Monomorphization + Phase 150 TC expr_types + Phase 182 substitution 조회 인프라를 활용하여, codegen 전체에서 i64 fallback을 제거. Phase 172~188의 "양파 깊이" 반복을 근본적으로 종료.

**접근 방식**: `type_to_llvm`의 i64 fallback을 `InternalError`로 바꾼 후, E2E 테스트에서 실패하는 경로를 TC expr_types 또는 substitution으로 수정.

**이미 갖춰진 인프라** (재구현 불필요):
- TC expr_types: `HashMap<Span, ResolvedType>` (Phase 150)
- substitution 조회: generic param → concrete type (Phase 182에서 i64 fallback 전 우선 조회)
- specialized 함수 생성: `$` mangling (Phase 141~146)
- compute_sizeof: Named type struct 필드 합산 (Phase 150)
- Vec 런타임 stride: elem_size 기반 인덱싱 (Phase 150)
- `&Vec<T>` → `&[T]` deref coercion (Phase 150)

**대상 파일**:
- `crates/vais-codegen/src/types/conversion.rs` — `type_to_llvm` i64 fallback 제거
- `crates/vais-codegen/src/inkwell/gen_expr/` — call arg, store, load, ret, phi의 type coercion 통합
- `crates/vais-codegen/src/type_inference.rs` — TC expr_types 우선 참조 확대 (Phase 150 기반)

**str 표현 통일** (병행):
- Phase 77~78에서 시작한 `{i8*, i64}` 전환을 모든 codegen 경로에서 완성
- Phase 177 inttoptr 워크어라운드 제거 → 정상 extractvalue로 교체
- extern 함수 호출 시 자동 extractvalue(0) 삽입

**TC coercion 정리**:
- Phase 151에서 제거 후 재추가된 coercion 확인 & 최종 제거
- CLAUDE.md Phase 158 규칙 100% 준수 검증 (**`VAIS_TC_NONFATAL=1` 사용 금지**)
- `phase158_type_strict.rs` E2E 보호 테스트 통과

**완료 기준**:
- `type_to_llvm`에서 Generic/Named → i64 fallback 경로 0개
- str 관련 codegen 경로에서 i64 표현 0곳
- unification.rs에 금지된 coercion (Bool↔I64, Str↔I64, Unit↔I64) 0건
- E2E 2555+ passed / 0 fail (regression 0)

---

### Phase 192~193 (예정): 안정화 — 실전 검증 & 해킹 제거

#### Phase 192: 실전 프로젝트 전체 컴파일

**검증 프로젝트**:
- vais-monitor: 37/37 모듈 clang 통과 후 실행 가능 바이너리
- VaisDB: 13/13 테스트 스위트 (2026-04-10 strict 빌드 0 errors 달성 기준 유지)

**완료 기준**:
- vais-monitor 37/37 모듈 OK + 실행 성공
- VaisDB 303+ 테스트 중 95%+ 통과
- E2E 테스트 0 fail

#### Phase 193: Cross-module 해킹 H5~H10 제거

**현재 상태**: Phase 187에서 cross-module struct 필드 resolution, 서브디렉토리 import fallback 해결. 그러나 H5~H10 hardcoded method/constant fallback (300줄+)은 잔존.
- i64 fallback 제거(Phase 191) 후 해킹 대부분 불필요해질 것으로 예상

**완료 기준**:
- H5~H10 해킹 코드 전량 제거
- multi-file 프로젝트의 cross-module 제네릭이 정상 동작
- vais-monitor + VaisDB 재검증

---

### Phase 194 (예정): v0.2.0 릴리스

**체크리스트**:
- 보안 감사 (cargo audit)
- 문서 업데이트 (LANGUAGE_SPEC, STDLIB, FFI_GUIDE)
- 성능 벤치마크 갱신 (현재: 50K LOC → 58.8ms, Fib35 C 대비 1.06x)
- GitHub Release + Homebrew + Docker 배포
- CHANGELOG 작성 (Phase 141~194 변경 요약)

---

### 장기 로드맵 (Phase 195+)

> v0.2.0 안정화 이후 검토. 우선순위는 커뮤니티 피드백에 따라 조정.

| 방향 | 내용 | 근거 | 예상 Phase |
|------|------|------|-----------|
| **가독성 개선** | `fn`/`struct` 등 긴 키워드 별칭(alias) 허용 | 단일 문자 키워드의 진입장벽 낮춤 | 195~196 |
| **패키지 생태계** | HTTP 서버, SQL 클라이언트 등 핵심 라이브러리 | 현재 9개 → 30+, 실용성 확보 | 197~200 |
| **킬러 유스케이스** | "AI가 VAIS로 WASM 플러그인 생성" 시나리오 | VAIS의 강점(다중 백엔드 + AI 토큰 효율)을 살리는 데모 | 201 |
| **증분 컴파일** | 변경 파일만 재컴파일, `vaisc check` 빠른 검증 | 대규모 프로젝트 지원 (현재 vaisc incremental.rs 존재) | 202~204 |
| **셀프호스팅 LLVM 백엔드** | Rust Inkwell 의존 제거, VAIS로 LLVM IR 생성 | 진정한 bootstrap. 현재 selfhost 50K+ LOC 기반 | 205~210 |
| **Dynamic Dispatch** | vtable 기반 `&dyn Trait` 다형성 | R5에서 static dispatch만 구현 (Phase 141~146) | 211~212 |
| **공식 벤치마크** | C/Rust 대비 성능 데이터 공개 | 공식 사이트 게시, 채택 촉진 | 213 |

---

## 📋 프로젝트 개요

### 핵심 특징
- **단일 문자 키워드**: `F` (function), `S` (struct), `E` (enum), `I` (if), `L` (loop), `M` (match)
- **자재귀 연산자** `@`: 현재 함수 재귀 호출
- **표현식 지향**: 모든 것이 표현식
- **LLVM 백엔드**: 네이티브 성능
- **타입 추론**: 최소한의 타입 어노테이션

### 기술 스택
- **언어**: Rust
- **파서**: Recursive Descent (logos 기반 Lexer)
- **백엔드**: LLVM IR (clang 컴파일)
- **테스트**: cargo test

---

## 📦 프로젝트 구조

```
crates/
├── vais-ast/          # 추상 구문 트리 ✅
├── vais-lexer/        # 토크나이저 (logos) ✅
├── vais-parser/       # Recursive descent 파서 ✅
├── vais-types/        # 타입 체커 ✅
├── vais-codegen/      # LLVM IR 생성기 ✅
├── vais-codegen-js/   # JavaScript (ESM) 코드 생성기 ✅
├── vais-mir/          # Middle IR ✅
├── vais-lsp/          # Language Server ✅
├── vais-dap/          # Debug Adapter Protocol ✅
├── vais-i18n/         # 다국어 에러 메시지 ✅
├── vais-plugin/       # 플러그인 시스템 ✅
├── vais-macro/        # 선언적 매크로 시스템 ✅
├── vais-jit/          # Cranelift JIT 컴파일러 ✅
├── vais-gc/           # 세대별 가비지 컬렉터 ✅
├── vais-gpu/          # GPU 코드젠 (CUDA/Metal/OpenCL/WebGPU) ✅
├── vais-hotreload/    # 핫 리로딩 ✅
├── vais-dynload/      # 동적 모듈 로딩 & WASM 샌드박스 ✅
├── vais-bindgen/      # FFI 바인딩 생성기 ✅
├── vais-query/        # Salsa-style 쿼리 데이터베이스 ✅
├── vais-profiler/     # 컴파일러 프로파일러 ✅
├── vais-security/     # 보안 분석 & 감사 ✅
├── vais-supply-chain/ # SBOM & 의존성 감사 ✅
├── vais-testgen/      # 속성 기반 테스트 생성 ✅
├── vais-tutorial/     # 인터랙티브 튜토리얼 ✅
├── vais-registry-server/    # 패키지 레지스트리 (Axum/SQLite) ✅
├── vais-playground-server/  # 웹 플레이그라운드 백엔드 ✅
├── vais-python/       # Python 바인딩 (PyO3) ✅
├── vais-node/         # Node.js 바인딩 (NAPI) ✅
└── vaisc/             # CLI 컴파일러 & REPL ✅

std/               # 표준 라이브러리 (.vais + C 런타임) ✅
examples/          # 예제 코드 (208 파일) ✅
selfhost/          # Self-hosting 컴파일러 ✅
benches/           # 벤치마크 스위트 (criterion) ✅
playground/        # 웹 플레이그라운드 프론트엔드 ✅
docs-site/         # mdBook 문서 사이트 ✅
vscode-vais/       # VSCode Extension ✅
intellij-vais/     # IntelliJ Plugin ✅
community/         # 브랜드/홍보/커뮤니티 자료 ✅
```

---

## 📊 프로젝트 현황

### 핵심 수치

| 지표 | 값 |
|------|-----|
| 전체 테스트 | 10,400+ (E2E 2,555+, 단위 8,400+) |
| 표준 라이브러리 | 74개 .vais + 19개 C 런타임 |
| 셀프호스트 코드 | 50,000+ LOC (컴파일러 + MIR + LSP + Formatter + Doc + Stdlib) |
| 컴파일 성능 | 50K lines → 58.8ms (850K lines/s) |
| 토큰 절감 | 시스템 코드에서 Rust 대비 57%, C 대비 60% 절감 |
| 컴파일 속도 비교 | C 대비 8.5x, Go 대비 8x, Rust 대비 19x faster (단일 파일 IR 생성) |
| 실전 프로젝트 | 3개 (CLI, HTTP API, 데이터 파이프라인) |

### 코드 건강도 (2026-03-29 감사)

| 지표 | 값 | 상태 |
|------|-----|------|
| TODO/FIXME | 0개 | ✅ |
| Clippy 경고 | 0건 | ✅ |
| 프로덕션 panic/expect | 0개 | ✅ |
| 에러 처리 | Result 패턴 일관, bare unwrap 없음 | ✅ |
| 대형 파일 (>1000줄) | 13개 (R14에서 comptime/concurrent 분할) | ✅ |
| unsafe SAFETY 주석 | 44/44 문서화 (100%) | ✅ |
| 의존성 버전 | 전부 최신 안정 버전 | ✅ |
| 보안 (입력 검증/인젝션/시크릿) | 이슈 없음 | ✅ |
| pre-existing 테스트 실패 | 0건 (Phase 159에서 전수 해결) | ✅ |

### 릴리즈 상태: v0.1.0 (프리릴리스)

> **버전 정책**: 현재는 0.x.x 프리릴리스 단계입니다. 언어 문법이 완전히 확정되어 더 이상 수정이 필요 없을 때 v1.0.0 정식 릴리스를 배포합니다.

| 항목 | 상태 |
|------|------|
| 빌드 안정성 / Clippy 0건 | ✅ |
| 테스트 전체 통과 (9,500+) | ✅ |
| E2E 2,555+ 통과 (0 fail, 0 ignored) | ✅ |
| 보안 감사 (cargo audit 통과) | ✅ |
| 배포 (Homebrew, cargo install, Docker, GitHub Releases) | ✅ |
| 문서 (mdBook, API 문서 65개 모듈) | ✅ |
| CI/CD (3-OS 매트릭스, 코드 커버리지) | ✅ |
| 패키지 레지스트리 (10개 패키지) | ✅ |
| 셀프호스팅 (부트스트랩 + MIR + LSP + Formatter) | ✅ |

---

## 🔒 언어 문법 스펙 기준선 (Phase 39 기준 — 동결)

> **원칙**: 아래 문법이 현재 구현된 Vais 언어의 전체 범위입니다. 이후 Phase에서는 **기존 문법의 완성도를 높이는 것**에 집중하며, 새로운 키워드/문법을 추가하지 않습니다. 문법 변경이 필요한 경우 별도 RFC로 진행합니다.

### 키워드 (확정)

| 분류 | 키워드 |
|------|--------|
| **단일 문자** | `F`(함수) `S`(구조체) `E`(열거형/else) `I`(if) `L`(루프) `M`(매치) `R`(리턴) `B`(break) `C`(continue/const) `T`(타입별칭) `U`(import) `P`(pub) `W`(trait) `X`(impl) `D`(defer) `O`(union) `N`(extern) `G`(global) `A`(async) `Y`(await) |
| **다중 문자** | `mut` `self` `Self` `true` `false` `spawn` `await` `yield` `where` `dyn` `macro` `as` `const` `comptime` `lazy` `force` `linear` `affine` `move` `consume` `pure` `effect` `io` `unsafe` `weak` `clone` |

### 연산자 (확정)

| 분류 | 연산자 |
|------|--------|
| **산술** | `+` `-` `*` `/` `%` |
| **비교** | `<` `<=` `>` `>=` `==` `!=` |
| **비트** | `&` `\|` `^` `~` `<<` `>>` |
| **논리** | `&&` `\|\|` `!` |
| **대입** | `=` `:=` `+=` `-=` `*=` `/=` |
| **특수** | `\|>` (파이프) `?` (삼항/try) `!` (unwrap) `@` (자재귀) `$` (매크로) `..` `..=` `...` (범위/가변인자) `->` `=>` (화살표) |

### 선언 (확정)

| 구문 | 상태 | 비고 |
|------|------|------|
| `F name(params) -> T { body }` | ✅ 완전 | 제네릭, where, async, default param |
| `S Name { fields }` | ✅ 완전 | 제네릭, 메서드, where |
| `E Name { Variants }` | ✅ 완전 | 유닛/튜플/구조체 variant |
| `W Name { methods }` | ✅ 완전 | super traits, GAT, where |
| `X Type: Trait { }` | ✅ 완전 | associated types |
| `T Name = Type` | ✅ 완전 | 타입 별칭 + trait 별칭 |
| `O Name { fields }` | ✅ 완전 | C-style 비태그 union |
| `N "C" { F ... }` | ✅ 완전 | extern, WASM import |
| `C NAME: T = expr` | ✅ 완전 | 상수 |
| `G name := expr` | ✅ 완전 | 전역 변수 |
| `macro name! { }` | ✅ 완전 | 선언적 매크로 |

### 타입 시스템 (확정)

| 타입 | 상태 |
|------|------|
| `i8~i128`, `u8~u128`, `f32`, `f64`, `bool`, `str` | ✅ 완전 |
| `Vec<T>`, `HashMap<K,V>`, `Option<T>`, `Result<T,E>` | ✅ 완전 |
| `[T]`, `[T; N]`, `&[T]`, `&mut [T]` | ✅ 완전 |
| `(T1, T2)`, `fn(A)->B`, `*T`, `&T`, `&mut T` | ✅ 완전 |
| `'a`, `&'a T` (라이프타임) | ✅ 완전 |
| `dyn Trait`, `X Trait` (impl Trait) | ✅ 완전 |
| `linear T`, `affine T` | ✅ 완전 |
| Dependent types `{x: T \| pred}` | ✅ 완전 (컴파일타임+런타임 검증) |
| SIMD `Vec4f32` 등 | ✅ 완전 |

### 패턴 매칭 (확정)

`_`, 리터럴, 변수, 튜플, 구조체, enum variant, 범위, or(`\|`), guard(`I cond`), alias(`x @ pat`)

### 어트리뷰트 (확정)

`#[cfg(...)]`, `#[wasm_import(...)]`, `#[wasm_export(...)]`, `#[requires(...)]`, `#[ensures(...)]`, `#[invariant(...)]`

---

## 📜 Phase 히스토리

> 상세 체크리스트는 git log를 참조하세요. Phase 번호는 누적 연번입니다.

### Phase 1~7: 기반 구축 (E2E — → 392)

핵심 컴파일러 파이프라인 (Lexer/Parser/TC/Codegen), Generics, Traits, Closures, Async/Await, Stdlib, LSP/REPL/Debugger 구현. inkwell/JIT/WASM/Python/Node 백엔드 확장. Effect/Dependent/Linear Types, MIR, Query-based 아키텍처. **부트스트랩 달성** (SHA256 일치). CI/CD, i18n, Homebrew/Docker 배포.

### Phase 8~21: 확장 · 안정화 (E2E 392 → 637)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 8 | 언어 진화 · Stdlib | 에러복구, Incremental TC, cfg 조건부 컴파일, 패키지매니저 | 392 |
| 9~10 | WASM · JS Codegen · 타입 추론 | wasm32 codegen, WASI, codegen-js (ESM), InferFailed E032 | 467 |
| 11~12 | CI · Lifetime · 성능 | Windows CI, CFG/NLL, 병렬 TC/CG (4.14x), Slice fat pointer | 498 |
| 13~14 | 에코시스템 · 토큰 최적화 | 9개 패키지, AES-256, JIT Result, 토큰 -31%, auto-return | 520 |
| 15~16 | 언어 확장 · 타입 건전성 | where/pattern alias/trait alias/impl Trait/HKT/GAT, Incremental, Tarjan SCC | 589 |
| 17~19 | Codegen · Selfhost · 보안 | Range struct, i64 fallback 제거, lazy/spawn, 보안 20건 수정, Docs 다국어 | 655 |
| 20~21 | 정리 · 복구 | Codegen 버그 수정 +44 E2E, ROADMAP 통합, 중복 제거 | 637 |

### Phase 22~52: Codegen 완성 · 품질 강화 (E2E 637 → 900)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 22~24 | 모듈 분할 R6 · 성능 | formatter/expr/function_gen 분할, Vec::with_capacity, codegen -8.3% | 647 |
| 25~27 | Codegen · 타입 건전성 | indirect call, i64 fallback→InternalError, TC pre-codegen 검증 | 713 |
| 28~31 | 코드 정리 · Selfhost · 모듈 분할 R7 | dead_code 정리, monomorphization 3-pass, tiered/item/doc_gen 분할 | 723 |
| 32~36 | E2E 확장 · assert_compiles 전환 | 136개 assert_compiles→assert_exit_code, type alias 버그 수정, 모듈 분할 R8 | 755 |
| 37~40 | E2E 800+ · Codegen 강화 | Spawn/Lazy 수정, Generic/Slice/Bool/Where, AST 15서브모듈, 모듈 분할 R9 | 811 |
| 41~44 | 건전성 · Pre-existing 전수 수정 | 135건 이슈 수정, pre-existing 14→0, var_resolved_types 도입 | 862 |
| 45~47 | 테스트 정리 · 900 달성 | 40개 중복 제거, 모듈 분할 R10, +78 E2E | 900 |
| 48~51 | Codegen 완성 | Spawn/Async 상태 머신, assert_compiles 7→4, E2E 900 전체 통과(0 fail) | 900 |
| 52 | ROADMAP 정리 | 완료 체크리스트 삭제, 638→~240줄 (-62%) | 900 |

### Phase 53~76: 성숙 · 릴리스 (E2E 900 → 967)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 53~54 | 외부 정합성 · CI | VSCode/IntelliJ 문법, Docs 4개 신규, codecov 60% | 900 |
| 55~62 | 코드 커버리지 | +2,948 단위 테스트, llvm-cov 전환, 68.7% 달성 | 900 |
| 63~64 | 버전 리셋 · EBNF 스펙 | 0.0.5 프리릴리스, vais.ebnf 154 rules, grammar_coverage 275개 | 900 |
| 65~66 | Pre-existing 검증 · Unify 완성 | 전수 수정 확인, unify 6 variant + apply_substitutions 13 compound | 900 |
| 67~70 | Codegen 확장 · 안전성 | Monomorphization 전이, Map literal, compound assign, panic 0개 | 919 |
| 71~73 | Object Safety · ABI · 릴리스 | v0.0.5, E034 중복함수 검출, assert_compiles 0개 | 931 |
| 74~76 | Stdlib · 온보딩 · 파일럿 | TOML/YAML 파서, 학습 경로 3단계, 실전 프로젝트 2개, **v0.1.0 릴리스** | 967 |

### Phase 77~143: 프로덕션 품질 · Monomorphization (E2E 967 → 2,383)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 77~78 | Codecov · str fat pointer | +515 tests, str `{i8*,i64}` 전환, C ABI 자동 변환 | 1,040 |
| 79~81 | 에러 위치 · 직렬화 · E2E 확장 | SpannedCodegenError, MessagePack/Protobuf, E2E 1,150 | 1,150 |
| 82~83 | 성능 · Stdlib | 50K 61.6ms (-4.6%), regex/http_client/sqlite 확충 | 1,185 |
| 84~86 | Selfhost · WASM · IDE | MIR Lowering, WASI P2/Component Model, LSP/DAP/IntelliJ +590 tests | 1,250 |
| 87~89 | 문서 · 위생 · 기술부채 | API Ref +16모듈, gitignore, Dependent Types 검증, Unicode, Codecov +203 | 1,291 |
| 90~91 | 최적화 · E2E 1,500 | GVN-CSE/DCE/Loop Unswitch, +218 E2E, MIR 4패스, Method mono, Lifetime 실장 | 1,540 |
| 92~94 | 안정성 · 성능 · 생태계 | Panic-free 180+ expect→Result, proptest, 2-level 캐시, Ed25519, vaisc fix, lint 7종 | 1,540 |
| 95~98 | 검증 · 기술부채 · CI | IR 검증 게이트, cargo fmt 65파일, clang-17 명시, LSP 모듈화 | 1,620 |
| 99~108 | 안정성 · 감사 · 분할 R11-R12 | expect→Result 전수, 9파일 분할, Codecov, E2E 0 ignored | 1,620 |
| 109~117 | v1.0 블로커 · 완성도 | Slice bounds check, scope-based auto free, Monomorphization 경고, WASM E2E 44 | 1,723 |
| 118~130 | 성능 · 타입 강화 · 분할 R13 | strict_type_mode 기본화, Lexer -29.8%, Parser -9.9%, +235 E2E | 2,036 |
| 131~140 | 감사 · E2E 2,345 · Stdlib 강화 | SAFETY 44건, R14 분할, Result 표준화, Vec/String/HashMap 메서드 | 2,345 |
| 141~143 | R1 Monomorphization · IR Type Tracking | specialized 함수, temp_var_types 레지스트리, store/load/ret 타입 추적, Drop auto-call | 2,383 |

### Phase 144~190: Codegen 근본 · 실전 검증 (E2E 2,383 → 2,555+)

Phase 141~148 근본 수정 (R1~R6) 후 VaisDB/vais-monitor 실전 컴파일에서 드러난 "양파 깊이" 버그를 Phase 172~190에 걸쳐 해소. 2026-04-10 Phase 189 완료 시점에 vais-monitor 37/37 모듈, vaisdb 13/13 테스트 strict 빌드 0 errors, E2E 2,555+ 0 fail 달성. 상세는 git log 및 "중장기 발전 로드맵" 섹션 참조.

---

## 🔴 Codegen 근본 문제 (VaisDB 실전 컴파일에서 발견, 2026-03-20)

> **배경**: VaisDB (RAG-native hybrid DB, ~200파일 순수 Vais) 컴파일 과정에서 발견된 컴파일러 한계.
> C1-C8 근본 수정 완료 (커밋 bcf1be5), TC 에러 674→5 (-99%), test_graph 37/45 통과 (82%).
> **모든 근본 문제 해결 완료** (Phase 141-148, 2026-03-23 확인)

| 이슈 | 상태 | 해결 Phase | E2E 테스트 |
|------|------|-----------|-----------|
| R1: Generic Monomorphization | ✅ 해결 | 141-146 | 23개 (phase145_r1) |
| R2: IR Postprocessor 제거 | ✅ 해결 | 142-148 | 14개 (phase145_r2) |
| R3: Per-Module Codegen | ✅ 해결 | 147 | 10개 (phase147) |
| R4: RAII/Drop | ✅ 해결 | 145-146 | 13개 (phase145_r4) |
| R5: Trait Dispatch | ✅ Static dispatch 동작 | 기존 | vtable 생성 + name mangling |
| R6: TC NONFATAL 제거 | ✅ 제거 | 145 | 4개 (phase145_r6) |

> R5 dynamic dispatch (vtable 기반 &dyn Trait 다형성)는 향후 확장 가능. 현재 static dispatch로 실전 코드 동작.

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |
