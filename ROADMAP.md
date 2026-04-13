# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **현재 버전**: 0.1.0 (Phase 190 완료 → Phase 190.5 준비)
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-04-13 (Phase 190 완료: 5/6 + 신규 #7, E2E 2563 passed / 0 fail)

---

## Current Tasks — Phase 190.5: 문자열 메모리 안정화 (drop-tracking)

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

- [ ] 1. 문자열 concat drop-tracking 리팩토링 (Opus direct)
  [대상 파일]:
    - crates/vais-codegen/src/string_ops.rs (concat/append/push_str 경로)
    - crates/vais-codegen/src/vtable.rs (destructor 엔트리, str에 대한 drop impl)
    - crates/vais-codegen/src/inkwell/gen_expr/ (alloc_slot 라이프사이클 참고, 수정 최소화)
  [완료 기준]:
    - `a + b + c + d` 체인 컴파일 후 valgrind/ASan 기준 누수 0
    - vais-monitor 장기 실행 시 RSS 증가가 입력량에 비례(상수 시간 내 수렴)
    - E2E 2563 passed / 0 fail 유지 (특히 phase134_strings, phase90_strings, phase183_option_result_struct)
    - 새 E2E 테스트 추가: phase190_str_concat_drop.rs (누수 시나리오 assert_exit_code)

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

progress: 0/1

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
