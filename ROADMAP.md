# AOEL Development Roadmap

## Project Overview

**AOEL (AI-Optimized Executable Language)**
AI가 가장 효율적으로 생성, 수정, 실행할 수 있는 프로그래밍 언어

---

## Current Status

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Prototype (Python) | **DONE** | 100% |
| Phase 1: Foundation (Rust) | **DONE** | 100% |
| Phase 2: Execution | **DONE** | 100% |
| Phase 3: Optimization | **DONE** | 100% |
| Phase 4: Native Compile | **DONE** | 100% |
| Phase 5: Ecosystem | **DONE** | 100% |

**Last Updated:** 2026-01-13

---

## Quick Start

```bash
# 빌드
cd aoel-rs
cargo build --release

# 실행
./target/release/aoel run examples/factorial.aoel

# 네이티브 컴파일
./target/release/aoel build examples/factorial.aoel --target llvm

# JIT 실행 (Cranelift)
cargo build --release --features cranelift
./target/release/aoel jit examples/simple.aoel

# 패키지 매니저
./target/release/aoel init my-project    # 새 프로젝트
./target/release/aoel add utils          # 의존성 추가
./target/release/aoel publish            # 레지스트리에 게시

# 개발 도구
./target/release/aoel format file.aoel   # 코드 포맷팅
./target/release/aoel profile file.aoel  # 성능 프로파일링
```

---

## Language Syntax

```aoel
// 함수 정의
add(a, b) = a + b
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// 컬렉션 연산
[1,2,3].@(_ * 2)      // map: [2,4,6]
[1,2,3].?(_ > 1)      // filter: [2,3]
[1,2,3]./(0, _ + _)   // reduce: 6

// 모듈
use math.{sin, cos}
```

---

## Completed Phases

### Phase 0-1: Foundation
- Python 프로토타입 → Rust 재구현
- Lexer, Parser, AST, Type Checker (Hindley-Milner)
- 76개 테스트 통과

### Phase 2: Execution
- 스택 기반 VM
- 50+ 빌트인 함수
- 클로저, 컬렉션 연산, 자기 재귀 ($)

### Phase 3: Optimization
- 상수 폴딩/전파, DCE, CSE
- 명령어 융합, 꼬리 재귀 최적화 (TCO)

### Phase 4: Native Compile
| Backend | Command | 특징 |
|---------|---------|------|
| C | `aoel build file.aoel` | 가장 호환성 좋음 |
| WASM | `aoel build file.aoel --target wasm` | 웹 지원 |
| LLVM | `aoel build file.aoel --target llvm` | 최적화 우수 |
| Cranelift | `aoel jit file.aoel` | 빠른 컴파일 |
| **JIT (Adaptive)** | `aoel run file.aoel --jit` | **50-75배 성능 향상** |

---

## Phase 5: Ecosystem (완료)

### 완료
- [x] 50+ 빌트인 함수
- [x] 모듈 시스템 (`use`, `pub`)
- [x] REPL
- [x] **std.io** - 파일 I/O, 경로 처리, 디렉토리 연산
- [x] **std.json** - JSON 파싱/생성/조작
- [x] **std.net** - HTTP 클라이언트 (GET/POST/PUT/DELETE)
- [x] **LSP 완전 구현** - 자동완성, Hover, Go to Definition, Find References, Rename, Signature Help
- [x] **패키지 매니저 (APM)** - aoel.toml, init/add/remove/install/publish
- [x] **FFI (C 바인딩)** - libc 함수 호출 (abs, sqrt, pow, sin, cos, etc.)
- [x] **개발 도구 (aoel-tools)** - Formatter, Profiler, Debugger
- [x] **Adaptive JIT (aoel-jit)** - Cranelift 기반 적응형 JIT 컴파일러
- [x] **Online REPL/Playground** - 웹 기반 AOEL 실행 환경 (WASM)

### TODO (Future Enhancements)
- [ ] JIT 제어 흐름 지원 (if/else, loops)
- [ ] JIT 재귀 함수 컴파일
- [ ] JIT Float 전용 최적화 경로
- [ ] SIMD 벡터화 (배열 연산)
- [x] 문서화 (README, 사용자 가이드)
- [ ] GitHub Pages 배포

---

## Project Structure

```
aoel-rs/crates/
├── aoel-lexer/      # 토큰화
├── aoel-ast/        # AST 정의
├── aoel-parser/     # 파서 + 모듈
├── aoel-typeck/     # 타입 체커
├── aoel-ir/         # IR + 최적화
├── aoel-lowering/   # AST → IR
├── aoel-vm/         # 스택 VM
├── aoel-jit/        # Adaptive JIT (Cranelift)
├── aoel-codegen/    # C/WASM/LLVM/Cranelift
├── aoel-tools/      # Formatter, Profiler, Debugger
├── aoel-lsp/        # Language Server
├── aoel-playground/ # Web Playground (WASM)
└── aoel-cli/        # CLI
```

---

## Test Summary

| Crate | Tests |
|-------|-------|
| aoel-lexer | 11 |
| aoel-parser | 10 |
| aoel-typeck | 11 |
| aoel-ir | 20 |
| aoel-lowering | 3 |
| aoel-vm | 30 |
| aoel-codegen | 14 |
| aoel-tools | 7 |
| aoel-cli | 3 |
| **Total** | **113** |

---

## Change Log (Recent)

### 2026-01-13 - Online REPL/Playground 구현
**aoel-playground 크레이트 추가 (WASM)**
- 웹 브라우저에서 AOEL 코드 실행
- WASM으로 컴파일된 경량 VM 사용
- 코드 에디터 (라인 번호, 커서 위치 표시)
- 실시간 실행 및 결과 출력
- 실행 시간 측정

**기능**
- **예제 코드** - Hello World, Factorial, Fibonacci, Map/Filter, Reduce, Prime Numbers, Math Functions, String Operations
- **코드 공유** - URL 기반 코드 공유 (Base64 인코딩)
- **코드 포맷팅** - AST 기반 코드 정리
- **키보드 단축키** - Ctrl+Enter (실행), Ctrl+Shift+F (포맷)

**WASM API**
- `execute(source)` - 코드 실행 (JSON 결과)
- `check(source)` - 파싱 + 타입체크
- `format_code(source)` - 코드 포맷팅
- `get_ast(source)` - AST 출력
- `get_tokens(source)` - 토큰 목록

**빌드 방법**
```bash
cd aoel-rs/crates/aoel-playground
wasm-pack build --target web --out-dir www/pkg
cd www && python3 -m http.server 8080
```

**제한사항** (브라우저 보안)
- 파일 I/O 미지원 (std.io)
- 네트워크 접근 미지원 (std.net)
- FFI 미지원
- 최대 재귀 깊이: 500

### 2026-01-13 - FFI 동적 라이브러리 로딩 구현
**libloading 기반 동적 FFI**
- `FfiLoader` 클래스 추가 - 동적 라이브러리 로드/함수 호출
- 플랫폼별 라이브러리 이름 자동 해석 (macOS: .dylib, Linux: .so, Windows: .dll)
- 검색 경로 설정 가능 (`add_ffi_search_path`)
- 함수 시그니처 등록 (`register_ffi_function`)

**지원 함수 시그니처**
- 인자 0-3개 함수 (i32, i64, f64, cstr)
- 반환 타입: void, int, float, string

**사용 예시**
```rust
let mut vm = Vm::new();
// FFI 함수 등록
vm.register_ffi_function("mylib", "compute",
    vec![FfiType::F64, FfiType::F64],
    FfiType::F64);
// 라이브러리 검색 경로 추가
vm.add_ffi_search_path("/opt/mylib");
```

**테스트**: 6개 신규 추가 (총 113개)

### 2026-01-13 - 개발 도구 (aoel-tools) 구현
**aoel-tools 크레이트 추가**
- **Formatter** - AST 기반 코드 포맷터
  - `aoel format <file>` - 코드 포맷 (stdout)
  - `aoel format <file> --write` - 파일에 덮어쓰기
  - `aoel format <file> --check` - 포맷 검사
  - 설정: `--indent`, `--max-width`
- **Profiler** - 함수별 실행 시간 측정
  - `aoel profile <file>` - 텍스트 출력
  - `aoel profile <file> --format json` - JSON 출력
  - 호출 횟수, 평균/최소/최대 시간 측정
- **Debugger** - 브레이크포인트, 스텝 실행
  - 브레이크포인트 설정/해제/토글
  - step, step_into, step_out, continue
  - 변수 검사, 콜 스택 조회
  - 감시 표현식 (watch)

**테스트**: 7개 신규 추가 (총 103개)

### 2026-01-13 - FFI (Foreign Function Interface) 구현
**FFI 문법**
```aoel
ffi "c" {
    fn abs(n: i32) -> i32
    fn sqrt(x: f64) -> f64
    fn pow(base: f64, exp: f64) -> f64
    fn getenv(key: cstr) -> cstr
}

// 사용
print(abs(-42))      // 42
print(sqrt(16.0))    // 4.0
```

**지원 함수 (libc)**
- 수학: abs, sqrt, pow, sin, cos, tan, log, exp, floor, ceil
- 문자열: strlen, atoi, atof
- 시스템: getenv, time, rand

**FFI 타입**
- 정수: i8, i16, i32, i64, u8, u16, u32, u64
- 실수: f32, f64
- 기타: bool, cstr, ptr, void

### 2026-01-13 - 패키지 매니저 (APM) 구현
**APM (AOEL Package Manager)**
- `aoel init [path]` - 새 프로젝트 초기화
- `aoel add <pkg>` - 의존성 추가
- `aoel remove <pkg>` - 의존성 제거
- `aoel install` - 의존성 설치
- `aoel list` - 의존성 목록
- `aoel publish` - 로컬 레지스트리에 게시

**aoel.toml 지원**
```toml
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
utils = "1.0.0"
```

**테스트**: 3개 신규 추가 (총 96개)

### 2026-01-13 - std.io, std.json, std.net 및 LSP 개선
**std.io (27개 함수)**
- 파일 I/O: read_file, write_file, append_file, read_lines, read_file_bytes
- 경로: path_join, path_exists, path_is_file, path_is_dir, path_parent, path_filename, path_extension, path_stem, path_absolute
- 디렉토리: list_dir, create_dir, create_dir_all, remove_file, remove_dir, remove_dir_all, copy_file, rename
- 환경: cwd, chdir, env_get, env_set, readline

**std.json (14개 함수)**
- 파싱/생성: json_parse, json_stringify, json_stringify_pretty
- 접근/조작: json_get, json_set, json_keys, json_values, json_has, json_remove, json_merge
- 타입 검사: json_type, json_is_null, json_is_object, json_is_array

**std.net (10개 함수)**
- HTTP: http_get, http_get_json, http_post, http_post_json, http_put, http_delete, http_head, http_request
- URL: url_encode, url_decode

**LSP v0.2.0 개선**
- 자동완성: 90+ 빌트인 함수 (std.io, std.json, std.net 포함)
- Signature Help: 함수 인자 도움말
- Find References: 심볼 참조 찾기
- Rename: 심볼 이름 변경
- 문서화: 모든 빌트인 함수에 상세 설명 추가

**테스트**: 17개 신규 추가 (총 93개)

### 2026-01-12 - Phase 4 완료
- Cranelift JIT 백엔드 추가
- LLVM Jump/JumpIfNot 완전 구현
- 4가지 백엔드 지원 (C, WASM, LLVM, Cranelift)

### 2026-01-12 - Phase 3 완료
- 최적화 패스: 상수 전파, CSE, 명령어 융합, TCO
- v6b 문법을 AOEL 메인으로 통합

### 2026-01-11 - Phase 0-2 완료
- Python 프로토타입
- Rust 컴파일러 프론트엔드
- VM 및 IR 구현
