# AOEL - AI 최적화 실행 언어

[![Build Status](https://github.com/sswoo88/aoel/actions/workflows/ci.yml/badge.svg)](https://github.com/sswoo88/aoel/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**AOEL (AI-Optimized Executable Language)**은 AI가 코드를 가장 효율적으로 생성, 수정, 실행할 수 있도록 설계된 프로그래밍 언어입니다. 토큰 효율성에 최적화된 간결한 문법을 제공하면서도 완전한 표현력을 유지합니다.

[English](README.md) | [문법 가이드](docs/syntax.md) | [API 레퍼런스](docs/api.md) | [예제](docs/examples.md)

## 특징

- **토큰 효율적 문법** - Python 대비 30-60% 적은 토큰
- **함수형 우선** - 일급 함수, 클로저, 컬렉션 연산
- **자기 재귀** - 우아한 재귀 정의를 위한 `$` 연산자
- **컬렉션 연산자** - `.@` (map), `.?` (filter), `./` (reduce)
- **다중 백엔드** - 인터프리터, JIT (50-75배 빠름), C, WASM, LLVM
- **풍부한 생태계** - LSP, 패키지 매니저, 디버거, 포맷터, 프로파일러
- **웹 플레이그라운드** - WebAssembly로 브라우저에서 AOEL 실행

## 빠른 시작

### 설치

```bash
# 저장소 클론
git clone https://github.com/sswoo88/aoel.git
cd aoel/aoel-rs

# 빌드
cargo build --release

# PATH에 추가 (선택사항)
export PATH="$PATH:$(pwd)/target/release"
```

### Hello World

```bash
echo 'print("안녕, AOEL!")' > hello.aoel
./target/release/aoel run hello.aoel
```

### REPL

```bash
./target/release/aoel repl
```

## 언어 개요

### 함수

```aoel
// 간단한 함수
add(a, b) = a + b

// 자기 재귀 ($)를 사용한 재귀
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// 피보나치
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)
```

### 컬렉션 연산

```aoel
numbers = [1, 2, 3, 4, 5]

// Map: 각 요소를 2배
doubled = numbers.@(_ * 2)        // [2, 4, 6, 8, 10]

// Filter: 짝수만 남기기
evens = numbers.?(_ % 2 == 0)     // [2, 4]

// Reduce: 합계
sum = numbers./+(0, _ + _)        // 15

// 연산 체이닝
result = [1..10].?(_ % 2 == 0).@(_ * _)  // [4, 16, 36, 64]
```

### 삼항 표현식

```aoel
// 삼항 연산자
max(a, b) = a > b ? a : b

// 중첩 삼항
grade(score) = score >= 90 ? "A" : score >= 80 ? "B" : score >= 70 ? "C" : "F"
```

### 모듈

```aoel
// 특정 함수 가져오기
use math.{sin, cos, pi}

// 공개 함수 (내보내기 가능)
pub calculate(x) = sin(x) * cos(x)
```

## 실행 모드

### 인터프리터 (기본)

```bash
aoel run program.aoel
```

### JIT 컴파일 (50-75배 빠름)

```bash
# JIT 지원으로 빌드
cargo build --release --features jit

# JIT로 실행
aoel run program.aoel --jit
```

### 네이티브 컴파일

```bash
# C로 컴파일
aoel build program.aoel --target c

# WebAssembly로 컴파일
aoel build program.aoel --target wasm

# LLVM IR로 컴파일
aoel build program.aoel --target llvm
```

## 내장 함수

### 코어 (20+)
`print`, `println`, `len`, `type`, `str`, `int`, `float`, `range`, `abs`, `sqrt`, `pow`, `sin`, `cos`, `tan`, `log`, `exp`, `floor`, `ceil`, `round`, `min`, `max`

### 컬렉션 (15+)
`head`, `tail`, `init`, `last`, `reverse`, `sort`, `unique`, `concat`, `flatten`, `zip`, `enumerate`, `take`, `drop`, `slice`, `sum`, `product`

### 문자열 (10+)
`split`, `join`, `trim`, `upper`, `lower`, `contains`, `replace`, `starts_with`, `ends_with`, `substring`

### 파일 I/O - std.io (25+)
`read_file`, `write_file`, `append_file`, `read_lines`, `path_join`, `path_exists`, `list_dir`, `create_dir`, `remove_file`, `cwd`, `env_get`

### JSON - std.json (14)
`json_parse`, `json_stringify`, `json_get`, `json_set`, `json_keys`, `json_values`, `json_has`, `json_remove`, `json_merge`, `json_type`

### HTTP - std.net (10)
`http_get`, `http_post`, `http_put`, `http_delete`, `http_get_json`, `http_post_json`, `url_encode`, `url_decode`

## 개발 도구

### 언어 서버 (LSP)

완전한 IDE 지원:
- 자동 완성
- 호버 문서
- 정의로 이동
- 참조 찾기
- 이름 변경
- 시그니처 도움말

```bash
# LSP 서버 시작
aoel lsp
```

### 패키지 매니저

```bash
# 새 프로젝트 초기화
aoel init my-project

# 의존성 추가
aoel add utils

# 의존성 설치
aoel install

# 레지스트리에 게시
aoel publish
```

### 코드 포맷팅

```bash
# stdout으로 포맷
aoel format program.aoel

# 파일에 직접 쓰기
aoel format program.aoel --write

# 포맷 검사
aoel format program.aoel --check
```

### 프로파일러

```bash
# 실행 프로파일
aoel profile program.aoel

# JSON 출력
aoel profile program.aoel --format json
```

## 웹 플레이그라운드

설치 없이 브라우저에서 AOEL을 사용해보세요:

```bash
cd aoel-rs/crates/aoel-playground
wasm-pack build --target web --out-dir www/pkg
cd www && python3 -m http.server 8080
# http://localhost:8080 열기
```

## 프로젝트 구조

```
aoel-rs/crates/
├── aoel-lexer/      # 토크나이저
├── aoel-ast/        # AST 정의
├── aoel-parser/     # 파서 + 모듈
├── aoel-typeck/     # 타입 체커 (Hindley-Milner)
├── aoel-ir/         # IR + 최적화
├── aoel-lowering/   # AST → IR
├── aoel-vm/         # 스택 기반 VM
├── aoel-jit/        # 적응형 JIT (Cranelift)
├── aoel-codegen/    # C/WASM/LLVM 백엔드
├── aoel-tools/      # 포맷터, 프로파일러, 디버거
├── aoel-lsp/        # 언어 서버
├── aoel-playground/ # 웹 플레이그라운드 (WASM)
└── aoel-cli/        # CLI 인터페이스
```

## 성능

| 벤치마크 | 인터프리터 | JIT | 속도 향상 |
|----------|------------|-----|-----------|
| add(100, 200) | 769 ns | 15 ns | 51배 |
| calc(50, 30) | 875 ns | 14 ns | 62배 |
| math(100) | 961 ns | 13 ns | 74배 |

## 기여하기

기여를 환영합니다! Pull Request를 자유롭게 제출해 주세요.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다 - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 감사의 말

- Rust로 구축
- JIT는 [Cranelift](https://cranelift.dev/)로 구동
- 함수형 프로그래밍 언어에서 영감을 받음
