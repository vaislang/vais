# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **현재 버전**: 0.1.0 (Phase 190 완료 → Phase 190.5 준비)
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-04-14 (Phase 190.5 + 190.6 완료 → Phase 191 follow-up 5건 등록)

---

## Current Tasks — Phase 191: 문자열 소유권 모델 확장 (RFC-001 follow-ups)

mode: auto
iteration: 19
max_iterations: 30
session_checkpoint: 2026-04-15 세션 4 — #10/#2a'/#9 3건 연속 완료.
  commits: b61f6e7a (#10), 7561b3dc (#2a'), c57943e1 (#9).
  E2E 진행: 2583 → 2586 → 2587 (ignored 1→0).
  세션 4 완료 작업:
    #10 — method_call.rs:188 fallback에 try_generate_vec_specialization 추가
          + fn_ctx snapshot/restore + pending_specialized_ir flush.
    #2a' — Vec_push$str owned-bitmap wrapping + Vec_drop$str shallow-free
           prelude. RFC-002 §9.8 4/6 완료 (return-transfer 미완).
    #9 — match arm Str PHI을 fat-pointer로 통일 + ownership transfer.
  남은 작업 (모두 design-heavy + 사용자 리뷰 또는 RFC 필수):
    #2b — RFC-002 §4.2 Option D 구현. ownership_mask field (ABI 변경) +
          struct shallow-drop + take_field! builtin. 광범위 회귀 위험,
          fresh session + 사용자 리뷰 권장.
    #2c — Nested container recursion. blockedBy: #2b.
    #3 — Trait object str 반환. RFC-002-trait-object-string.md 작성 필요.
    #4 — 클로저 캡처된 str. RFC-003-closure-string-capture.md 작성 필요.
  재개 권장: fresh session에서 #2b 착수 또는 RFC 작업 #3/#4 시작.
session_checkpoint: 2026-04-15 세션 5 iter 18 — #2b Iter A 완료 + Iter B/C/D 분할.
  commits: bd087e58 (Iter A survey + plan).
  user_gate: #2b 착수 선택 (RFC-002 §4.2 Option D approved).
  세션 5 완료: Iter A (survey + sub-iter plan) + #2b를 #2b-B/#2b-C/#2b-D 3개 
    독립 sub-task로 분할하여 등록. 구현 0. E2E 2587 baseline 유지.
  scope_decision: 세션 4 "광범위 회귀 위험 + fresh session 권장" warning 존중.
    #2a/#2a' 분할 패턴 재사용. 각 Iter 한 세션 1개씩 — auto mode가 자연스럽게
    blockedBy 체인으로 한 iter만 픽업하도록 #2b-B → #2b-C → #2b-D 체인 설정.
  **NEXT SESSION (fresh)**: `/clear` + `/harness` → auto mode가 미완료 `- [ ]` 
    재복원. blockedBy 체인에 의해 **#2b-B만 unblocked 상태**로 진입 → 자동 픽업.
    Iter B 완료 후 `/loop` 재진입 시 #2b-C가 unblock, 그 다음 #2b-D 순.
    각 Iter 1 세션. Iter 종료 후 사용자가 `/clear` 결정.
  재개 권장: fresh session에서 #2b-B (layout amendment) 착수 — 자동 진입 예정.
    저위험 스위치 옵션: RFC 작업 #3/#4는 blockedBy 없으므로 auto가 우선 #2b-B 선택.
    사용자가 다른 순서 원하면 `/harness` 진입 후 작업 선택 단계에서 명시 가능.
session_checkpoint: 2026-04-14 세션 3 — #2a-rfc + RFC §9.8 진단 완료.
  commits: 9c616289 (RFC §9), 456f12d4 (세션 2 체크포인트), 6728b481 (§9.7 blocker).
  세션 3 최종 상태:
    - §9.7 CRITICAL은 IR probe로 misdiagnosis 확정 → §9.8에 진짜 진단 기재.
      %Vec와 %Vec$T는 structural equivalence (LLVM 타입 동등) 관계,
      같은 body `{i64,i64,i64,i64}` 유지가 유일 invariant.
    - RFC-003 (Phase 192) withdrawn — 불필요.
    - #2a 구현 계획 확정: std/vec.vais에 owned:i64 5th field 추가 (uniform).
    - #2a blockedBy 해제. 구현 착수 가능.
  재개 권장: /clear + /harness → #2a 구현 (§9.8 revised plan 6단계).
  세션 3에서 구현 시작하지 않은 이유: 컨텍스트 보호. #2a는 large, fresh
  session에서 단일 집중 iteration으로 완료 권장.

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

- [ ] 2. Container-owned strings: Vec<str> / 사용자 struct str 필드 (Opus direct) — parent tracker
  note: 상위 tracker. 실행은 #2a(완료)/#2a-rfc(완료)/#2a'(완료)/#2b(tracker)/#2b-B/#2b-C/#2b-D/#2c로 분할.
    이 항목은 auto pickup 대상 아님. 모든 sub-task 완료 시 `- [x] 2`로 close.
  blockedBy: #2b-D, #2c.
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

- [x] 2a. Vec<str> 레이아웃 + owned bitmap + __drop_Vec_str — **scope reduced** (Opus direct) ✅ 2026-04-15
  scope_decision: RFC-002 §9.8 6단계 중 stdlib 레이아웃(step 1) + codegen helpers IR 정의(step 2)만 이 작업에 포함.
    push 호출-부 wrapping / scope-exit prelude / return-transfer / e2e 검증은 #2a' (신규)로 이관.
    이유: Iter B 진행 중 발견된 stdlib-Vec-generic-grow 미특수화 버그가 Vec<str> 컴파일 자체를 막아서
    wiring 검증 경로 부재. 별도 선결 작업 필요. RFC-002 §9.8은 실제 구현 2/6만 커버하므로 §9.9 업데이트 권장.
  changes:
    std/vec.vais: Vec<T> 5필드 `owned:i64` 추가 + with_capacity init=0.
    crates/vais-codegen/src/string_ops.rs:
      generate_vec_str_container_helpers (~140 LOC LLVM IR) —
        __vais_vec_str_owned_ensure(%Vec*, i64), __vais_vec_str_owned_set(%Vec*, i64),
        __vais_vec_str_shallow_free(%Vec*). ABI: `void @free(i8*)` + `i8* @malloc(i64)` 기반.
      generate_vec_str_container_declarations — per-module extern 선언.
    crates/vais-codegen/src/module_gen/{mod,instantiations,subset}.rs:
      `generated_structs.contains_key("Vec$str")` 가드 하에 emission.
  verify:
    cargo build --workspace --exclude vais-python --exclude vais-node: green.
    cargo test -p vaisc --test e2e: 2582 passed / 0 failed / 1 ignored (baseline 유지).
    조건부 emission이므로 기존 e2e에서 dormant. Vec<str> specialize trigger 테스트 부재.
  invariant_confirmed: RFC-002 §9.8 structural equivalence — stdlib 단독 변경이 모든
    Vec<T> specialization에 자동 전파 실증. 5필드 → 모든 %Vec$T 5필드.
  follow-up: #2a' (새 작업) + 그 전에 #10 (Vec_grow 특수화 버그 선행 수정).
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

  **2026-04-14 세션 3 추가 진단 (§9.8, RFC-002 업데이트)**:
  IR runtime probe로 CRITICAL이 **misdiagnosis**였음 확정.
  실제: %Vec과 %Vec$T는 같은 body `{i64,i64,i64,i64}`로 structurally
  equivalent (LLVM 타입 동등성) — 모든 %Vec GEP는 %Vec$T에 대해
  interchangeably 동작. "fallback vs specialized" 충돌은 존재하지 않음.
  진짜 invariant: 두 타입의 field layout이 identical이어야 함.
  → 해결: std/vec.vais에 owned:i64 5th field 추가 (base+모든 specialized 자동 전파).
  → RFC-003 불필요, withdrawn.
  → #2a 새 구현 계획 (RFC-002 §9.8):
    1. std/vec.vais: Vec<T>에 owned:i64 추가 + new/with_capacity/grow/drop 수정.
    2. Vec_push$str call-site wrapping (codegen).
    3. __vais_vec_str_shallow_free prelude splice + helpers emission.
    4. pending_return_skip_container 추가 (return transfer).
    5. e2e tests (§6 cases 1, 2, 6).

- [ ] 2b. struct shallow-drop + ownership_mask + user-Drop sequencing (Opus direct, tracker)
  status: Iter A 완료 (survey + design plan, 세션 5 iter 18 commit bd087e58).
    남은 작업은 #2b-B/#2b-C/#2b-D 3개 독립 sub-task로 분할 — 각 fresh session 1개씩.
    이 항목은 auto pickup 대상 아님 (blockedBy로 gating). 모두 완료 시 `- [x] 2b` close.
  blockedBy: #2b-D.
  [참조]: RFC-002 §4.2 Option D, §4.3 struct 경로, §4.4 post-emission.
  [대상 파일]:
    - crates/vais-codegen/src/generate_expr_struct.rs:87-130 (struct literal 필드 저장 루프
      — ownership_mask bit set hook 위치, #2a' method_call.rs:706-768 패턴 대칭).
    - crates/vais-codegen/src/stmt.rs:828-902 `generate_scope_drop_cleanup` (struct 타입
      scope-exit — user drop 호출 직후 __vais_struct_shallow_free_{Name} splice).
    - crates/vais-codegen/src/stmt.rs:1001-1080 `generate_drop_cleanup` (function-exit 대칭).
    - crates/vais-codegen/src/string_ops.rs (신규 헬퍼 emission — __vais_vec_str_*와 동형).
    - crates/vais-codegen/src/types/mod.rs:60-68 `StructInfo` (heap-owned 필드 인덱스 집계).
    - crates/vais-codegen/src/trait_dispatch.rs:105-129 `register_trait_impl` (Drop impl
      등록 지점 확인 — shallow-drop은 drop_registry 항목에 엮지 말고 별도 emission
      경로로 갈 것 — 변경 최소).
    - docs/rfcs/RFC-002-container-string-ownership.md §4.2 업데이트 (take_field! ABI 확정).
  [설계]:
    - ownership_mask i64 필드: 구조체에 heap-owned 후보 필드(str/Vec<str>/사용자 Drop) 존재 시
      레이아웃 끝에 i64 append. 64 필드 cap, overflow = 컴파일 에러. Vec<T>의 owned:i64
      (§4.1 §9.8)와 동일한 "field가 있으면 있는 대로 전파" 원리.
    - struct literal codegen: 각 필드 rvalue 분류 — heap-owned(string_value_slot 보유)면
      ownership_mask 비트 set + slot transfer (Phase 191 #5 null-store 패턴 유지).
      literal/borrowed은 no-op (비트 0 유지).
    - user F drop 호출 후 shallow-drop 무조건 emission. 사용자는 free API 없음 →
      double-free 구조적 불가.
    - `take_field!` macro/builtin 스펙 작성 (구현은 별도 follow-up 가능). 없을 땐
      사용자는 into_parts() 패턴 사용.
  [완료 기준]: RFC-002 §6 tests (3) struct_str_field_drop, (4) struct_user_drop_takes_ownership.
    E2E baseline 2587 유지 + new 2 tests.
  [복잡도]: 높음 — 레이아웃 변경 + drop sequencing + bitmap helper + take_field! 스펙.
  blockedBy: #2a (completed).

  [Iter A 완료 2026-04-15 세션 5 iter 18 (Opus direct)]:
    survey_results:
      1. struct drop emission 경로: stmt.rs:828 `generate_scope_drop_cleanup`가
         drop_registry 기반으로 user drop 호출. stmt.rs:1001 `generate_drop_cleanup`이
         function-exit 대칭 경로 (양쪽 대응 필요).
      2. struct literal codegen: generate_expr_struct.rs:87-130 — 필드별 GEP+store 루프
         (line 106에서 field rvalue 생성 → line 110의 GEP → coercion → store).
         Hook 지점: line 106 후 val (rvalue) 확정 시점에 string_value_slot 조회
         → 보유 시 ownership_mask 비트 계산 + transfer (method_call.rs:730-780 패턴).
      3. StructInfo: types/mod.rs:60-68 `StructInfo { fields: Vec<(String, ResolvedType)> }`.
         heap-owned 판정: ResolvedType::Str 직접 비교 + Vec<str> / Named(with owned field)
         재귀 판정 (#2c 경로). ownership_mask 부착 여부는 field에 ::Str 포함 시에 한함.
      4. trait_dispatch.rs:105-129 `register_trait_impl`: "Drop" trait impl 시
         drop_registry[type] = drop_fn_name 등록. shallow-drop은 drop_registry에 엮지
         않음 (그러면 user drop과 경합) — stmt.rs:862 drop 호출 직후 추가 명령만 splice.
      5. vtable.rs: 현재 `__drop_{type_name}` 경로는 dyn Trait용 generic drop이라 관계 없음.
         user-defined Drop은 trait_dispatch.rs 경로가 드라이브. shallow-drop은 codegen이
         직접 `__vais_struct_shallow_free_{Name}` LLVM 함수를 emit.
      6. 백엔드 단일화: RFC-001 §5.4 "single implementation path" — generate_expr_struct.rs,
         stmt.rs, string_ops.rs는 inkwell/text-IR 공유. inkwell/gen_aggregate.rs는 tuple
         literal용이라 struct literal과 별도 경로 (확인 필요). #2a'에서도 단일 경로 유지.
    design_decisions_tentative (user gate 필요):
      D1. Layout 변경 조건: `StructInfo.fields`에 `ResolvedType::Str` 또는 "Vec$str"
          Named 또는 owned-containing Named가 포함되면 ownership_mask i64 append.
          그 외는 기존 레이아웃 유지 (비-str struct ABI 무변경).
      D2. Helper emission: string_ops.rs에 `generate_struct_shallow_free_helpers(struct_name,
          heap_field_indices: Vec<usize>)` 신규. module 수준에서 per-struct emit.
          시그니처: `void __vais_struct_shallow_free_{Name}(%Name*)`.
          IR: mask 로드 → 각 heap_field_index i마다 bit i 검사 → set이면 해당 필드 로드
          (field가 fat-ptr이면 .0 extractvalue) → free. bit clear는 불필요 (struct는
          consume-once).
      D3. Scope-exit sequence: stmt.rs:862 `droppable.iter().rev()` loop 안에서
          `drop_fn` call **직후** shallow-drop available 체크하여 call 추가.
          Shallow-drop available 조건: `needs_struct_shallow[type_name]` (신규 set).
      D4. Function-exit 대칭: stmt.rs:1038 동일 블록에서 대칭 처리.
      D5. Struct literal wrapping: generate_expr_struct.rs:106-107 val 확정 직후,
          effective_fields[field_idx].1이 Str이고 val이 string_value_slot에 있으면:
            (a) mask 비트 (1 << field_idx) OR로 ownership_mask에 set
            (b) string_value_slot remove + scope_str_stack entry remove
            (c) slot에 null store (Phase 191 #5 패턴)
            (d) alloc_tracker entry 유지
      D6. take_field! 스펙: 별도 follow-up (#2b-takef) 권장. Iter D에서 구현 여부 결정.
    sub_iter_plan:
      Iter B (fresh session): D1 layout amendment —
        types/mod.rs StructInfo에 `pub has_owned_mask: bool, pub heap_fields: Vec<usize>`
        파생 집계 메서드 추가. monomorphization 시점 정확한 hook 좌표 확인 필요
        (generate_expr_struct.rs line 17의 resolve_struct_name 직후 possible).
        검증: struct literal 정의 시 E2E 2587 유지.
      Iter C (fresh session): D2 + D3 + D4 emission —
        string_ops.rs에 generate_struct_shallow_free_helpers 추가 (Vec$str
        helpers의 방법과 동일). stmt.rs 두 drop cleanup 경로에 splice. module_gen/
        subset.rs + instantiations.rs에도 helper declarations 추가 (Phase 191 #2a 패턴).
        검증: user Drop + str 필드 struct 로컬 drop 시 leaks 0.
      Iter D (fresh session): D5 wrapping + e2e —
        generate_expr_struct.rs:106 hook + RFC-002 §6 tests (3)(4) 추가.
        E2E 2587 + 2 new tests.
        take_field! 스펙은 이 iter에서는 RFC 문구만 확정 (구현은 별도 작업).
    blocker_check:
      - 현재 `#[ignore]` 0건, #2a'의 함정(structural equivalence %Vec vs %Vec$T)은
        struct에는 없음 — 모든 struct monomorphization은 고유 이름. 안전.
      - user drop이 pre-drop hook로 `&self`만 받으므로 필드 무결성 유지 (RFC-002 §4.2).
    verify: ROADMAP update only, Rust 코드 변경 0 — E2E 2587 baseline 유지.

- [x] 2b-B. #2b Iter B — StructInfo 파생 + layout amendment (Opus direct) ✅ 2026-04-16
  strategy: sequential → Opus direct. opus_direct: ABI 레이아웃 조건부 변경 +
    monomorphization 대칭성 (text-IR + inkwell 두 백엔드) → 설계-구현 inseparable.
  session_iter: 19 (auto).
  changes:
    crates/vais-codegen/src/types/mod.rs:
      StructInfo에 `has_owned_mask: bool, heap_fields: Vec<usize>` 필드 추가.
      `StructInfo::field_is_heap_owned(ty)` — Str / Named{"Vec$str"} / Named{"Vec", [Str]} 판정.
      `StructInfo::derive_ownership_mask(fields)` — 두 registration site 동기화용 헬퍼.
      heap_fields에 #[allow(dead_code)] (Iter C/D에서 소비).
    crates/vais-codegen/src/registration.rs:
      register_struct에서 derive_ownership_mask 호출 → StructInfo 초기화.
    crates/vais-codegen/src/function_gen/generics.rs:
      generate_specialized_struct_type에서 post-substitution fields 기반 derive +
      llvm_fields에 "i64" 조건부 append. StructInfo 등록 시 동일한 값 사용.
    crates/vais-codegen/src/types/type_gen.rs:
      generate_struct_type이 info.has_owned_mask → "i64" append.
    crates/vais-codegen/src/inkwell/gen_declaration.rs:
      define_struct 대칭 분기 (vais_types::ResolvedType import 추가,
      resolved_fields 수집 → derive_ownership_mask → i64_type() 추가).
    crates/vais-codegen/src/inkwell/gen_types.rs:
      define_specialized_struct 대칭 (substituted_fields 기반 derive).
    crates/vais-codegen/src/generate_expr_struct.rs:
      struct literal alloca 직후 ownership_mask 필드 zero-init 프리루드 추가
      (effective_type_name StructInfo 조회 → has_owned_mask 시 GEP+store i64 0).
    crates/vais-codegen/src/types/tests.rs:
      4개 StructInfo literal 생성 site에 has_owned_mask/heap_fields 필드 채움.
  verify:
    cargo check --workspace --exclude vais-python --exclude vais-node: green.
    cargo clippy --workspace --exclude vais-python --exclude vais-node: green.
    cargo test -p vais-codegen --lib types::tests: 73/0 passed.
    cargo test -p vaisc --test e2e phase191: 16/0 passed (baseline 16 유지).
    cargo test -p vaisc --test e2e: 2587/0/0 passed (baseline 2587 유지, 742s).
  invariant_preserved:
    - 기존 non-str struct 레이아웃 무변경 (heap_fields 없으면 i64 append 하지 않음).
    - Vec$str 레이아웃 무변경 (post-substitution fields에 직접 Str 없음 →
      has_owned_mask=false). #2a에서 이미 owned:i64 추가됨.
    - User struct `S P { name: str }` → has_owned_mask=true, heap_fields=[0],
      레이아웃 { {ptr,i64}, i64 } (fat-ptr + mask).
    - 텍스트-IR과 inkwell 두 백엔드 모두 대칭 분기.
  scope:
    Iter B는 infrastructure 단계 — mask 필드 할당/제로초기화만 구현.
    실제 비트 OR (struct literal wrapping) 은 Iter D.
    shallow-drop helper emission은 Iter C.
  [상속]: #2b Iter A 완료 (commit bd087e58, ROADMAP #2b 본문 참조).
  [sub-steps]:
    1. crates/vais-codegen/src/types/mod.rs:60-68 `StructInfo`에 집계 필드 추가:
       `pub has_owned_mask: bool, pub heap_fields: Vec<usize>`. 계산 시점은
       struct 등록(register_struct 혹은 등가 경로)에서 fields 훑어 ResolvedType::Str
       또는 Named{"Vec$str"} 포함 시 true + 인덱스 수집.
    2. generate_expr_struct.rs:17 `resolve_struct_name` 직후 `struct_info.has_owned_mask`
       조회 → true면 `effective_fields`에 ("__ownership_mask", I64) append +
       `effective_type_name` layout에도 i64 append. 해당 alloca는 기존 struct_ptr가
       포괄 (같은 %Type 레이아웃이므로).
    3. monomorphization 경로(generate_specialized_struct_type — #2a 좌표: 
       function_gen/generics.rs:57-85, inkwell/gen_types.rs:162-200)에도 대칭 분기.
       base struct가 has_owned_mask면 specialized layout에도 i64 append.
    4. with_capacity/new 같은 constructor 경로 확인 — 새 필드가 0으로 초기화되도록.
       struct literal이 아닌 필드 누락 케이스 감지.
  [검증]: E2E 2587 baseline 유지 (Iter B만으론 기능 추가 0, layout 변경이 기존 struct에
    영향 없음 확인). test-case: str 필드 없는 struct은 layout 무변경.
  [완료 기준]: cargo test -p vaisc --test e2e 2587/0/0. `- [x] 2b-B` + changes: log.
    Iter C를 위한 unblock 신호.
  [대상 파일]:
    - crates/vais-codegen/src/types/mod.rs
    - crates/vais-codegen/src/generate_expr_struct.rs (line 17-79 영역)
    - crates/vais-codegen/src/function_gen/generics.rs (line 57-85 근처)
    - crates/vais-codegen/src/inkwell/gen_types.rs (line 162-200 근처, 대칭)
    - crates/vais-codegen/src/registration.rs (StructInfo 생성 지점 — 재확인)
  [복잡도]: 중. ABI 조건부 변경 + monomorphization 대칭성이 핵심.
  blockedBy: 없음 (#2a 완료).

- [ ] 2b-C. #2b Iter C — shallow-drop helper emission + scope/function-exit splice (Opus direct)
  [상속]: #2b-B 완료 (StructInfo.has_owned_mask + layout).
  [sub-steps]:
    1. crates/vais-codegen/src/string_ops.rs에 `generate_struct_shallow_free_helpers(
       struct_name: &str, fields: &[(String, ResolvedType)], heap_field_indices: &[usize])` 
       추가. 패턴: #2a'의 `generate_vec_str_container_helpers` 대응.
       시그니처: `void @__vais_struct_shallow_free_{Name}(%{Name}*)`.
       IR: ownership_mask 필드 GEP+load → 각 heap_field_index i마다:
         (a) bit (1 << i) 마스크 & mask → ne 0 체크
         (b) set이면 field i GEP → fat-ptr이면 .0 extractvalue → free
         (c) 비트 clear 불필요 (struct consume-once).
    2. crates/vais-codegen/src/lib.rs + init.rs에 `needs_struct_shallow: HashSet<String>`
       신규 필드 (#2a'의 needs_vec_str_helpers 패턴).
    3. module_gen/{mod,instantiations,subset}.rs에 declarations + helper body emission:
       needs_struct_shallow의 각 type마다 generate_struct_shallow_free_helpers + extern 선언.
       #2a 패턴(generated_structs.contains_key 가드) 재사용.
    4. crates/vais-codegen/src/stmt.rs:862 scope-exit drop loop — drop_fn call 직후:
       `if self.types.structs[type_name].has_owned_mask` 검사 후 
       `call void @__vais_struct_shallow_free_{Name}(%Name* %ptr_tmp)` splice +
       `self.needs_struct_shallow.insert(type_name)`.
    5. crates/vais-codegen/src/stmt.rs:1038 function-exit drop loop — 동일 splice.
    6. 사전 체크: user drop이 정의 없는 struct(drop_registry 미등록)인데 heap 필드는
       있는 경우 — shallow-drop만 호출해야 함. stmt.rs:842 guard가 drop_registry 있을
       때만 진입이므로, 추가로 `has_owned_mask만 true` 분기 필요. 
  [검증]: 
    - cargo test -p vaisc --test e2e 2587/0/0 (helper emission이 dormant 상태일 때).
    - 수동 테스트: S Person {name: str}; p := Person {name: "a"+"b"}; ← Iter D에서 
      literal wrapping까지 연결되어야 leak 0 확인 가능. Iter C 단독은 infrastructure만.
  [완료 기준]: cargo clippy green + E2E 2587/0/0. Iter D unblock.
  [대상 파일]:
    - crates/vais-codegen/src/string_ops.rs
    - crates/vais-codegen/src/lib.rs + init.rs
    - crates/vais-codegen/src/module_gen/{mod,instantiations,subset}.rs
    - crates/vais-codegen/src/stmt.rs (line 828-900 + 1001-1080)
  [복잡도]: 높음. shallow-drop sequencing + helper lifecycle + user-drop-없는 경로 분기.
  blockedBy: #2b-B.

- [ ] 2b-D. #2b Iter D — struct literal wrapping + E2E RFC-002 §6 (3)(4) (Opus direct)
  [상속]: #2b-C 완료 (shallow-drop helpers + splice wired, dormant).
  [sub-steps]:
    1. crates/vais-codegen/src/generate_expr_struct.rs:106-107 hook — val (field rvalue)
       확정 직후:
         - effective_fields[field_idx].1 == ResolvedType::Str
         - struct_info.has_owned_mask == true
         - fn_ctx.string_value_slot에 val의 SSA token 존재
       3가지 조건 AND 시:
         (a) ownership_mask 필드 GEP → load → OR with (1 << field_idx) → store.
         (b) string_value_slot remove + scope_str_stack 최상위 frame에서 해당 slot 제거.
         (c) `store i8* null, i8** {slot}` emit (Phase 191 #5 패턴).
         (d) alloc_tracker entry 유지.
       모든 struct 필드 저장 전 ownership_mask를 0으로 초기화하는 prelude 1회 필요.
    2. 새 e2e: crates/vaisc/tests/e2e/phase191_struct_str_drop.rs (신규, 2+ tests)
       - (3) struct_str_field_drop: S P {n: str}; p := P {n: "a"+"b"}; drop → leaks 0
       - (4) struct_user_drop_takes_ownership: S P {n: str} + X P: Drop {F drop...};
             user drop이 도메인 로직만, shallow-drop이 n 정리. leaks 0.
       선택: (5) struct_literal_only — literal name → bitmap 0, free 호출 없음 확인.
    3. crates/vaisc/tests/e2e/main.rs 모듈 등록.
    4. docs/rfcs/RFC-002-container-string-ownership.md §4.2 문구 업데이트 —
       구현 완료 상태 반영 + take_field! 스펙은 별도 follow-up으로 명시.
  [검증]:
    - cargo test -p vaisc --test e2e: 2589/0/0 (baseline 2587 + 2~3 new).
    - macOS leaks --atExit 스모크 (Iter D 변경 확인).
    - cargo clippy green.
  [완료 기준]: E2E baseline+new 통과. #2b parent tracker close (`- [x] 2b`) + 
    RFC-002 §8 check item 1개 close.
  [대상 파일]:
    - crates/vais-codegen/src/generate_expr_struct.rs (line 81-150 영역 hook 추가)
    - crates/vaisc/tests/e2e/phase191_struct_str_drop.rs (신규)
    - crates/vaisc/tests/e2e/main.rs
    - docs/rfcs/RFC-002-container-string-ownership.md (§4.2 마감 주석)
  [복잡도]: 중간. hook 1개 + e2e 2~3개 + RFC 문구.
  blockedBy: #2b-C.

- [ ] 2c. Nested container recursion (Vec<Vec<str>>, Vec<struct{str}>) (Opus direct)
  [참조]: RFC-002 §5 Q3
  [대상 파일]: vtable.rs (모노모피제이션 recursion), drop_registry
  [완료 기준]: RFC-002 §6 test (5) nested_vec_of_struct_str. 외곽 Vec drop이 모든 내부 str 정리.
  [복잡도]: 중간.
  blockedBy: #2a (completed), #2b-D.

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

- [x] 10. stdlib Vec<T> Vec_grow 특수화 버그 수정 (Opus direct) ✅ 2026-04-15
  strategy: sequential research-first — research-haiku 진단 시도 → truncated → Opus 직접 조사로 **근본 원인 확정**.
  diagnosis_2026-04-15 (Opus):
    버그 지점: crates/vais-codegen/src/expr_helpers_call/method_call.rs:164-193.
    메커니즘: `Vec_push$i64` specialize 중 내부 `@.grow()` = `MethodCall{receiver: SelfCall, method: "grow"}`.
      self의 recv_type = `Vec<i64>` (concrete). Line 166: base = "Vec_grow".
      Line 188: mangled = vais_types::mangle_name("Vec_grow", [i64]) = "Vec_grow$i64".
      Line 189 guard: `self.types.functions.contains_key(&mangled)` → **FALSE** (아직 스케줄 안 됨).
      Line 192: fallback to unmangled `base` = "Vec_grow" → LLVM에 `@Vec_grow` 미정의 symbol 참조 → 링크 에러.
    근본 문제: **on-demand specialization이 "Vec_push$i64 body 안의 Vec_grow$i64 호출"을 감지하지 못함**.
      `Vec_push$i64`는 user 최상위 호출부에서 스케줄됨.
      그 body 안의 `@.grow()` (같은 impl block의 generic method)는 별도 스케줄 엔트리가 없음.
      따라서 `Vec_grow$i64` specialization은 영원히 호출/emit되지 않음.
    **fix path**: method_call.rs:189 guard 수정 —
      `contains_key` false일 때 fallback하지 말고 **specialization 스케줄링**.
      즉, generic method base(`Vec_grow`)가 fn_instantiations에 있고 자기 struct의 impl method면,
      resolve_generic_call() 호출 + generate_specialized_function 트리거 + 이후 mangled 사용.
    구현 규모: 중간 (monomorphization worklist 재진입 패턴 확인 필요, Vec<T> 외 다른
      generic impl의 self-method-call도 동일 영향 — 광범위 회귀 위험).
    model: Opus direct — dispatch 설계 이슈 + 잠재적 광범위 영향 + 설계-구현 inseparable.
    deferred: 이 세션에서 context 소진. fresh session에서 재개 권장.
  [출처]: Phase 191 #2a Iter B 진행 중 2026-04-15 발견.
  [증상]: `U std/vec` 후 `Vec.with_capacity(N).push(x).drop()` 최소 예제에서
    LLVM `error: use of undefined value '@Vec_grow'` — Vec_push<T> specialize는
    emit되지만 호출하는 Vec_grow는 base name으로만 참조되고 specialize 미emit.
  [재현]:
    examples/simple_vec_test.vais 또는
    `U std/vec\nF main() -> i64 { v := Vec.with_capacity(4); v.push(42); v.drop(); 0 }`
  [예상 근본 원인]:
    - Vec_push 내부의 `@.grow()` self-method 호출이 specialize instantiate하는 경로 누락.
    - 또는 Vec_grow$T가 monomorphization worklist에 추가되지 않음.
  [대상 파일]:
    - crates/vais-codegen/src/function_gen/generics.rs (specialization worklist)
    - crates/vais-codegen/src/expr_helpers_call/method_call.rs (@ self-recursion dispatch)
    - 관련 확인: 2582 e2e가 모두 local Vec<T>를 재정의해서 우회 — stdlib Vec는
      production에서 실사용 경로 부재. 역사적으로 가려져 있었음.
  [완료 기준]:
    - `U std/vec` + `Vec.with_capacity`/push/drop 최소 예제 컴파일 + 실행 성공
    - 2582 baseline 유지
    - 신규 e2e (Vec<i64>, Vec<str> 양쪽) 추가
  [복잡도]: 중간. monomorphization 경로 1개 점 수정일 것으로 예상.
  [블록 해제]: #2a' 착수 가능해짐.

  [완료 2026-04-15 Opus direct]:
    changes:
      crates/vais-codegen/src/expr_helpers_call/method_call.rs:188-206 —
        all-concrete 경로에서 mangled 미등록 시 generated_functions 재검사 +
        try_generate_vec_specialization fallback 추가. 무한 재진입은
        generate_specialized_function_inner:241의 generated_functions.insert
        先 guard로 차단.
      crates/vais-codegen/src/expr_helpers_call/method_call.rs:1128-1140 —
        try_generate_vec_specialization의 {Vec, HashMap, Option} whitelist 제거,
        struct_defs/generic_method_bodies 보유 여부로 일반화.
      crates/vais-codegen/src/expr_helpers_call/method_call.rs:1149-1226 —
        method signature pre-register (types.functions.insert) — recursive
        body 내 자기 참조가 return/param 타입 조회에 걸리도록. 이후 fn_ctx
        snapshot/restore로 `initialize_function_state` clobber 방지 (locals/
        scope_stack/alloc_tracker/entry_allocas 등 19개 필드 저장).
      crates/vais-codegen/src/module_gen/instantiations.rs:794-803 +
        crates/vais-codegen/src/module_gen/subset.rs:773-781 —
        pending_specialized_ir flush 추가 (기존 generate_module만 flush하던 것을
        with_instantiations + subset 양쪽에 미러).
    verify:
      cargo clippy --workspace --exclude vais-python --exclude vais-node: green.
      cargo test -p vaisc --test e2e: 2583 passed / 0 failed / 1 ignored
        (baseline 2582 + 1 new e2e_phase191_vec_grow_spec_from_push).
      새 테스트: Local Vec<T> 구조(std/vec와 동일 5-field 레이아웃) + push(1,2,3)
        + drop. 이전 동작: link error `@Vec_grow` undefined. 현재 동작: exit 0.
    scope_decision:
      원래 계획(U std/vec + Vec.with_capacity/push/drop 실행)은 달성 못함 —
      stdlib `vec_new() -> Vec<i64>` 내 `Vec.with_capacity(8)` 경로가 별도
      타입 추론 버그 (infer_expr_type이 Vec<i64>가 아니라 Vec<> 반환 → alloca
      `%Vec` vs call 결과 `%Vec$i64` LLVM 타입 불일치) 때문.
      static method dispatch 분기(method_call.rs:917+)에 on-demand 확장을
      넣어봤으나, 이 경로가 활성화되면 stdlib 자체 vec_new 컴파일이 깨져 스택
      전체가 막힘. 따라서 이번 작업은 ROADMAP 진단 원문의 `method_call.rs:189`
      범위(=`@.grow()` self-method 호출 경로)에만 한정.
    follow_up:
      별도 작업 `#12. Vec static method + user-code Vec.with_capacity<T>
      specialization`으로 분리 권장. 필요한 부수 fix:
      (a) infer_expr_type에서 generic struct 정적 메서드 호출의 반환 타입에
          컨텍스트로부터 T를 전파.
      (b) generate_expr_struct_lit에서 `has_generic_fields=false`지만 struct
          자체는 generic인 경우(Vec<T>의 전부-i64 필드)도 specialized 이름 사용.
      (c) 또는 `%Vec`와 `%Vec$T`를 LLVM opaque struct로 전환해 structural
          equivalence를 언어 레벨로 관철.

- [x] 2a'. Vec<str> call-site wiring + e2e (Opus direct) ✅ 2026-04-15
  [상속]: RFC-002 §9.8 6단계 중 stdlib(#2a 완료) 제외한 나머지.
  [sub-steps]:
    3. Vec_push$str call-site wrapping —
       crates/vais-codegen/src/expr_helpers_call/method_call.rs:620-691 부근.
       조건: full_method_name.starts_with("Vec_push$str"). push 호출 직전에
       heap-owned rvalue 판정 (string_value_slot lookup) → owned_ensure(cap) +
       owned_set(len) inject + alloc_slot transfer (Phase 191 #5 패턴).
    4. Scope-exit drop prelude splice —
       crates/vais-codegen/src/stmt.rs:828 `generate_scope_drop_cleanup`에
       type_name == "Vec$str" (또는 "owned" 필드 보유 struct)면 user drop 직전
       `__vais_vec_str_shallow_free` 호출 inject.
    5. pending_return_skip_container (state.rs) + return-transfer 플러밍.
    6. e2e phase191_container_str_drop.rs:
       (1) vec_str_push_drop_no_leak, (2) mixed_literal, (6) return_transfers.
  [완료 기준]: RFC-002 §6 tests 통과 + E2E baseline + leaks 0 (macOS `leaks --atExit`).
  [복잡도]: 높음.
  blockedBy: #10 (Vec_grow 수정).

  [완료 2026-04-15 Opus direct]:
    changes:
      crates/vais-codegen/src/expr_helpers_call/method_call.rs:706-768 —
        Vec_push$str call-site wrapping: `__vais_vec_str_owned_ensure(v, len+1)` +
        `__vais_vec_str_owned_set(v, len)` when rvalue is heap-owned (tracked via
        string_value_slot). Transfer 후 slot에 null store + string_value_slot /
        scope_str_stack entry 제거 (Phase 191 #5 ownership-transfer 패턴).
      crates/vais-codegen/src/expr_helpers_call/method_call.rs:686-703 —
        Vec_drop$str prelude splice: user Vec.drop() 직전
        `__vais_vec_str_shallow_free(v)` inject. 이 helper가 owned bitmap을
        순회하며 heap-owned element 문자열 버퍼를 free → user drop이 data 블록
        free하면서 전체 소유권 정리 완료.
      crates/vais-codegen/src/lib.rs:203-205 + init.rs:99 —
        `needs_vec_str_helpers` flag 추가 (현재는 helper emission이 이미
        `generated_structs["Vec$str"]` 조건으로 gated되어 있어 flag는 future
        use; struct registration 없이 helper만 필요한 edge case 대비).
      crates/vaisc/tests/e2e/phase191_vec_str_sandbox.rs (신규, 3 tests):
        (a) push_literal_only — literal str만 push, bitmap 비활성 경로.
        (b) push_concat_drop — 100-iter concat 결과 push, drop 후 heap 정리.
        (c) push_mixed_literal_heap — literal + heap 혼합, bitmap 정확도 확인.
      crates/vaisc/tests/e2e/main.rs: module registration.
    verify:
      cargo build -p vais-codegen: green.
      cargo clippy --workspace --exclude vais-python --exclude vais-node: green.
      cargo test -p vaisc --test e2e: 2586 passed / 0 failed / 1 ignored
        (baseline 2583 + 3 new phase191_vec_str_sandbox tests).
    scope_reduced:
      RFC-002 §9.8 6단계 중 완료: #2a 기존(1,2) + 이번 (3,4).
      미완료: (5) pending_return_skip_container — Vec<str> 반환 시 function-exit
      청소 스킵. 현재 구현된 경로는 user가 명시적 drop()을 호출하는 flow만
      커버. `F make() -> Vec<str> { v := ...; v.push(...); v }` 같은 return-by-
      value 경로는 drop이 누락되면 leak, 중복 호출되면 double-free 가능. 별도
      follow-up 권장 (#2a'' 또는 #2b와 묶음).
    known_limit:
      - vaisc CLI로 직접 `.vais` 파일 컴파일 시 TC가 store_typed를 인식하지
        못해 "Undefined function" 에러 발생. 그러나 TypeChecker API를 직접
        사용하는 e2e 테스트는 정상 동작. CLI-specific TC 초기화 이슈로 판단,
        #2a' 범위 밖.
      - stdlib std/vec import 경로는 `Vec.with_capacity` 타입 추론 버그
        (#10 scope 제외) 미해결 상태 지속.

- [x] 9. Match-arm string PHI fat-ptr unification (text-IR) (Opus direct) ✅ 2026-04-15
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

  [완료 2026-04-15 Opus direct]:
    changes:
      crates/vais-codegen/src/control_flow/match_gen.rs:404 —
        Str arm의 phi_type을 `i8*` → `{ i8*, i64 }`로 통일. 모든 arm body가
        fat-pointer를 emit하므로 PHI도 fat-pointer로 받음 (LLVM 검증 통과).
      crates/vais-codegen/src/control_flow/match_gen.rs:317-355 —
        default fallthrough block의 default_val: Str 타입일 때
        `insertvalue { i8*, i64 } { i8* null, i64 0 }, i64 0, 1` 명령어를
        emit하여 fat-pointer zero 값 생성. 또한 default_label 진입 시
        fn_ctx.current_block 갱신 누락을 수정 (이전엔 arm body는 갱신했으나
        default 분기는 누락 → 후속 IR이 잘못된 block으로 emit).
      crates/vais-codegen/src/control_flow/match_gen.rs:460-484 —
        Str PHI ownership merge 로직 추가. if-expr PHI(expr_helpers_control.rs
        :344-371)와 동형: 각 arm value의 string_value_slot 조회 → PHI 결과
        SSA에 첫 slot 등록 + 추가 slot은 phi_extra_slots에 stash. 후속
        return / let-binding hook이 ownership transfer 처리.
      crates/vaisc/tests/e2e/phase191_text_ir_scope_drop.rs:271-278 —
        e2e_phase191_match_arm_concat_phi의 `#[ignore]` 제거 + 주석 갱신.
        테스트는 `M n { 1 => "aa"+"bb", _ => "cc"+"dd" }`를 println으로 출력
        검증 — UAF 없이 "aabb"가 출력되어야 통과.
    verify:
      cargo build -p vais-codegen: green.
      cargo clippy --workspace --exclude vais-python --exclude vais-node: green.
      cargo test -p vaisc --test e2e: 2587 passed / 0 failed / 0 ignored
        (baseline 2586 + 1 unignored test, ignored count 1→0).
    rfc: RFC-001 §4.6 PHI merge UAF fix를 match에도 적용 — if/match가 동일
      ownership-transfer 규약을 따르도록 통일.

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
    #2a: Opus direct foreground (iter 10, fresh session 4) — RFC-002 §9.8
        revised plan 6단계. strategy: design-impl inseparable (stdlib Vec<T> 레이아웃
        변경 + codegen call-site wrap + helper emission + scope-exit prelude +
        return-transfer + e2e tests는 모두 §9.8 invariant "%Vec 与 %Vec$T body
        동일"에 묶여있어 단일 브레인 필요). 파일 범위 넓음(std/vec.vais +
        vais-codegen 5+ files) + ABI 영향 → impl-sonnet 위임 시 설계 의도 상실 위험.
        opus_direct: RFC-002 §9.8 invariant-preserving stdlib amendment —
        layout 변경 + 모든 specialization 전파 + helper ABI 동시 설계 필요.
        user_gate: 2-3 sub-iterations 합의 (2026-04-15).
          Iter A: std/vec.vais owned:i64 5th field + with_capacity init ✅ 2026-04-15
            (E2E 2582/0, baseline 유지 — structural equivalence 실증).
          Iter B: codegen helpers emission (3 functions) ✅ 2026-04-15
            (string_ops.rs: __vais_vec_str_owned_ensure/set/shallow_free +
             module_gen/{mod,instantiations,subset}.rs: emit when Vec$str exists.
             E2E 2582/0 유지, dormant until call-site wiring).
          **BLOCKER 발견 2026-04-15**: stdlib Vec<T>는 **어떤 T에 대해서도
            현재 빌드 실패** — Vec_push 특수화가 generic @Vec_grow를 호출하는데
            해당 symbol 미정의. /tmp/test_vec_i64.vais, examples/simple_vec_test.vais
            모두 `use of undefined value '@Vec_grow'`. 모든 e2e 테스트는 inline
            local Vec<T> 정의를 사용하므로 이 버그가 가려져 있었음 (U std/vec
            임포트한 e2e 0개 확인).
            영향: #2a 원래 계획(stdlib Vec<str> call-site wrap)은 stdlib Vec
            자체가 작동해야 검증 가능. Iter C 착수 전 결정 필요.
            옵션 A: Vec_grow 특수화 버그 선행 수정 (scope creep, 별도 작업).
            옵션 B: e2e를 local Vec<T> + 직접 __vais_vec_str_* helper 호출로
              작성 (stdlib 우회, #2a 핵심 의미 검증 가능).
            옵션 C: #2a를 helpers emission 완료까지만 범위 축소, #2a' 신설.
          Iter B-next: 사용자 결정 대기.

progress: 7/11 resolved (#1, #5, #6, #7, #8, #2a-rfc, #2a scope-reduced complete;
  #2b/#2c still blocked by #2a' + #10; #9 new surfaced by #8; #10 new surfaced by #2a Iter B;
  #2a' new — carries remaining #2a wiring post-#10);
  RFC-002 §9.8 partial — structural equivalence + stdlib amendment 실증됨,
  helpers IR 완비, 하지만 stdlib Vec generic specialization 경로 자체가 깨져있어
  wiring 진입점 막힘. #10 선결 필요. RFC-002 §9.9 업데이트 권장(scope split 반영).

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
