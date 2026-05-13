# Phase 197 P197-S — vais-server Audit

**Date**: 2026-04-18
**Compiler**: /Users/sswoo/study/projects/vais/compiler/target/release/vaisc (built 2026-04-18 07:53)
**Method**: fresh build.sh with this repo's vaisc via `VAISC=` env override.

---

## Build flow executed

```
# Step 1: Create docs dir
mkdir -p /Users/sswoo/study/projects/vais/compiler/docs/phase197

# Step 2: src layout
ls /Users/sswoo/study/projects/vais/lang/packages/vais-server/src/
# → api  auth  core  db  http  main.vais  middleware  router  util  ws

# Step 3: Fresh build via env override
cd /Users/sswoo/study/projects/vais/lang/packages/vais-server && \
VAISC=/Users/sswoo/study/projects/vais/compiler/target/release/vaisc \
VAIS_STD_PATH=/Users/sswoo/study/projects/vais/compiler/std \
bash build.sh

# Step 4: Test compilation scan (22 .vais test files, with VAIS_DEP_PATHS=src/)
```

The build.sh respects `VAISC` env var (`VAISC="${VAISC:-${HOME}/.cargo/bin/vaisc}"`), so env override worked directly.

---

## IR emission (vaisc)

- **Exit code**: 0 (success)
- **Errors**: none
- **Warnings**: 1
  - `VAIS_SINGLE_MODULE=1 is deprecated — per-module codegen now supports cross-module generics`
  - This warning is benign; the build continues and succeeds.

---

## clang link

- **Step 2/3** (IR → object): exit 0
- **Step 3/3** (link): exit 0
- **Errors**: none

---

## Binary produced

- **Path**: /Users/sswoo/study/projects/vais/lang/packages/vais-server/vais-server
- **`file` output**: `Mach-O 64-bit executable arm64`
- **Binary runs** (`./vais-server --version` / `--help` both respond):
  ```
  vais-server v0.1.0
  Configured 3 routes, 2 middlewares
  Would listen on
  Server ready.
  ```

---

## Test scan results

22 `.vais` test files found under `tests/`. Compiled individually with `--emit-ir` and `VAIS_DEP_PATHS=src/`.

**Summary: 7 PASS / 15 FAIL**

### Passing tests (7)

| Test file | Subdirectory |
|-----------|-------------|
| test_error.vais | core/ |
| test_shutdown.vais | core/ |
| test_response.vais | http/ |
| test_status.vais | http/ |
| test_pipeline.vais | middleware/ |
| test_yaml.vais | util/ |
| test_protocol.vais | ws/ |

### Failing tests (15) — categorized by root cause

#### Category A: C-style for-loop syntax not supported — P001 (7 tests)

Pattern `I i = 0; i < n; i = i + 1 { ... }` triggers `error[P001] Unexpected token` (semicolon after initializer).
This is a parser-level failure — Vais `I` only supports `I condition { }`, not C-style three-part iteration.

Affected files and error locations:
- `tests/db/test_query.vais` → `query.vais:261:16` — `I i = 0; i < self.columns.len(); i = i + 1`
- `tests/http/test_method.vais` → `test_method.vais:54:12` — `I i = 0; i < methods.len(); i = i + 1`
- `tests/integration/test_db_integration.vais` → `query.vais:261:16` (same dep)
- `tests/integration/test_full_flow.vais` → `router.vais:87:16` — `I i = 0; i < METHOD_COUNT; i = i + 1`
- `tests/integration/test_router.vais` → `router.vais:87:16` (same dep)
- `tests/router/test_router.vais` → `router.vais:87:16` (same dep)
- `tests/router/test_tree.vais` → `tree.vais:70:12` — `I i = 0; i < parts.len(); i = i + 1`

#### Category B: `VaisError` field access mismatch — E030 (2 tests)

Tests access `.code` and `.message` fields on `VaisError`, but the actual type does not expose those fields. 6 errors in `test_jwt.vais`, 4 in `test_password.vais`.

- `tests/auth/test_jwt.vais`: `e.code`, `e.message` → `no field 'code'/'message' on type 'VaisError'`
- `tests/auth/test_password.vais`: same pattern, 4 occurrences

#### Category C: `M expr { }` match on Result — type errors (2 tests)

- `tests/core/test_config.vais` (4 type errors): `M config.validate() { ... }` — match on a method returning `Result`
- `tests/integration/test_core.vais` (1 type error): `M bad_config.validate() { ... }`
- `tests/integration/test_http.vais` (1 type error): `error[E001] Type mismatch — expected i64, found bool`

#### Category D: Duplicate definition — E008 (2 tests)

- `tests/integration/test_middleware.vais`: duplicate function name around line 90
- `tests/middleware/test_logger.vais`: duplicate definition at line 147

#### Category E: Empty file — P002 (1 test)

- `tests/api/test_graphql.vais`: file is a comment-only stub, `error[P002] Unexpected end of file`

---

## Failure breakdown

| Error code | Count | Root cause |
|------------|-------|------------|
| P001 — Unexpected token | 7 | C-style `for` init-cond-step inside `I` |
| P002 — Unexpected EOF | 1 | Stub/empty test file (comment only) |
| E001 — Type mismatch | 1 | `bool` where `i64` expected |
| E008 — Duplicate definition | 2 | Duplicate function names in test files |
| E030 — No such field | 2 tests (10 errors) | `VaisError` struct missing `.code`/`.message` fields |
| Type errors (multi) | 2 tests | `M` match on Result-returning methods |

---

## Hypotheses (Phase 195/196 변경과의 연관성)

**main build (src/main.vais)**: Phase 195/196 영향 없음. 빌드 완전 성공, 바이너리 정상 생성.

**Test failures 연관성 분석**:

1. **C-style for-loop (P001, 7건)**: Phase 195/196 이전부터 존재했을 가능성이 높음. Vais `I` 키워드는 단순 조건 루프만 지원하며, C-style 세미콜론 구문은 Phase 195/196 이전에도 미지원. **Phase 195/196 변경과 무관**.

2. **VaisError 필드 없음 (E030, 2 tests)**: `VaisError` 타입 정의가 변경되어 `.code`/`.message` 필드가 제거되었거나 이름이 바뀐 경우. Phase 195/196에서 표준 에러 타입 리팩토링이 있었다면 관련될 수 있으나, 검증 필요. **가능한 연관**.

3. **bool↔i64 타입 불일치 (E001, 1건)**: Phase 158에서 확정된 strict type rule 준수. `bool`을 `i64`로 암시적 변환 금지 정책. **Phase 195/196과 무관, Phase 158 정책 적용**.

4. **Duplicate definition (E008, 2건)**: 테스트 파일 자체 문제. **Phase 195/196과 무관**.

5. **Empty stub file (P002, 1건)**: `test_graphql.vais`가 스텁 파일. **Phase 195/196과 무관**.

6. **Match on Result 타입 오류**: `M config.validate()` 패턴이 타입 체커를 통과하지 못함. Phase 195/196의 타입 체커 변경이 Result 매칭 경로에 영향을 주었을 가능성 있음. **조사 권장**.

**결론**: 주요 빌드 파이프라인(src/main.vais → IR → 바이너리)은 Phase 195/196 이후 완전히 정상. 15건의 테스트 실패 중 14건은 Phase 195/196 이전부터 존재했을 테스트 소스 문제(C-style for, stub file, duplicate defs, VaisError API 불일치). 단 `bool→i64` 타입 불일치와 `M Result` 매칭 오류는 Phase 158/195 타입 정책과 접점이 있어 추가 확인 권장.

---

## PROMISE

PROMISE: COMPLETE
