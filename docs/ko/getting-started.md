# Vais 시작 가이드

이 문서에서는 Vais를 설치하고 첫 프로그램을 작성하는 방법을 안내합니다.

## 목차

- [요구 사항](#요구-사항)
- [설치](#설치)
- [첫 번째 프로그램](#첫-번째-프로그램)
- [실행 방법](#실행-방법)
- [REPL 사용하기](#repl-사용하기)
- [개발 도구](#개발-도구)
- [다음 단계](#다음-단계)

---

## 요구 사항

Vais를 빌드하고 실행하려면 다음이 필요합니다:

- **Rust** 1.70 이상
- **Cargo** (Rust 패키지 매니저)

Rust가 설치되어 있지 않다면:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 설치

### 소스에서 빌드

```bash
# 저장소 클론
git clone https://github.com/sswoo88/vais.git
cd vais/vais-rs

# 릴리스 모드로 빌드
cargo build --release

# 실행 파일 확인
./target/release/vais --version
```

### PATH에 추가 (선택사항)

어디서든 `vais` 명령어를 사용하려면 PATH에 추가합니다:

```bash
# bash/zsh
echo 'export PATH="$PATH:/path/to/vais/vais-rs/target/release"' >> ~/.bashrc
source ~/.bashrc

# 또는 심볼릭 링크 생성
sudo ln -s /path/to/vais/vais-rs/target/release/vais /usr/local/bin/vais
```

### JIT 지원으로 빌드 (선택사항)

JIT(Just-In-Time) 컴파일을 사용하면 50-75배 빠른 실행이 가능합니다:

```bash
cargo build --release --features jit
```

---

## 첫 번째 프로그램

### Hello World

`hello.vais` 파일을 생성합니다:

```vais
// 첫 번째 Vais 프로그램
print("안녕, Vais!")
```

실행:

```bash
vais run hello.vais
```

출력:

```
안녕, Vais!
```

### 변수와 함수

`basics.vais` 파일을 생성합니다:

```vais
// 변수 선언
name = "Vais"
version = 1.0

// 함수 정의
greet(who) = "안녕, " ++ who ++ "!"

// 함수 호출
message = greet(name)
print(message)
print("버전:", version)
```

### 컬렉션 연산

`collections.vais` 파일을 생성합니다:

```vais
// 배열 생성
numbers = [1, 2, 3, 4, 5]

// Map: 각 요소를 2배
doubled = numbers.@(_ * 2)
print("2배:", doubled)

// Filter: 짝수만 선택
evens = numbers.?(_ % 2 == 0)
print("짝수:", evens)

// Reduce: 합계 계산
sum = numbers./+(0, _ + _)
print("합계:", sum)

// 체이닝: 필터 -> 맵 -> 리듀스
result = [1..11].?(_ % 2 == 0).@(_ * _)./+(0, _ + _)
print("짝수의 제곱 합:", result)
```

### 재귀 함수

`recursion.vais` 파일을 생성합니다:

```vais
// 자기 재귀 연산자 $를 사용한 팩토리얼
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// 피보나치
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)

// 테스트
print("5! =", factorial(5))
print("10번째 피보나치:", fib(10))

// 1-10의 팩토리얼
print("팩토리얼 1-10:", [1..11].@(factorial(_)))
```

---

## 실행 방법

### 인터프리터 실행 (기본)

```bash
vais run program.vais
```

### JIT 컴파일 실행 (빠름)

```bash
vais run program.vais --jit
```

### 네이티브 컴파일

다양한 타겟으로 컴파일할 수 있습니다:

```bash
# C 코드로 컴파일
vais build program.vais --target c

# WebAssembly로 컴파일
vais build program.vais --target wasm

# LLVM IR로 컴파일
vais build program.vais --target llvm
```

---

## REPL 사용하기

REPL(Read-Eval-Print Loop)을 사용하면 대화형으로 코드를 실행할 수 있습니다:

```bash
vais repl
```

REPL 세션 예제:

```
vais> x = 10
10
vais> y = 20
20
vais> x + y
30
vais> factorial(n) = n < 2 ? 1 : n * $(n - 1)
<function factorial>
vais> factorial(5)
120
vais> [1, 2, 3].@(_ * 2)
[2, 4, 6]
```

종료하려면 `Ctrl+D` 또는 `exit`를 입력합니다.

---

## 개발 도구

### 언어 서버(LSP)

IDE 지원을 위한 언어 서버를 실행합니다:

```bash
vais lsp
```

지원 기능:
- 자동 완성(Autocomplete)
- 호버 문서(Hover Documentation)
- 정의로 이동(Go to Definition)
- 참조 찾기(Find References)
- 이름 변경(Rename)
- 시그니처 도움말(Signature Help)

### 코드 포맷팅

```bash
# stdout으로 포맷된 코드 출력
vais format program.vais

# 파일에 직접 쓰기
vais format program.vais --write

# 포맷 검사 (CI용)
vais format program.vais --check
```

### 프로파일링

```bash
# 실행 프로파일
vais profile program.vais

# JSON 형식으로 출력
vais profile program.vais --format json
```

### 패키지 매니저

```bash
# 새 프로젝트 초기화
vais init my-project

# 의존성 추가
vais add utils

# 의존성 설치
vais install

# 레지스트리에 게시
vais publish
```

---

## 프로젝트 구조

Vais 프로젝트의 권장 구조:

```
my-project/
├── vais.toml           # 프로젝트 설정 파일
├── src/
│   ├── main.vais       # 메인 진입점
│   └── utils.vais      # 유틸리티 모듈
└── lib/
    └── math.vais       # 수학 라이브러리
```

### vais.toml 예제

```toml
[package]
name = "my-project"
version = "0.1.0"
description = "My Vais project"

[dependencies]
utils = "1.0.0"
```

---

## 다음 단계

Vais를 더 깊이 배우려면 다음 문서를 참조하세요:

- [문법 가이드](syntax.md) - 전체 문법 레퍼런스
- [API 레퍼런스](api.md) - 내장 함수 및 표준 라이브러리
- [예제](examples.md) - 다양한 실용 예제
- [기여 가이드](contributing.md) - 프로젝트에 기여하는 방법

---

## 도움말 및 지원

- **GitHub Issues**: [github.com/sswoo88/vais/issues](https://github.com/sswoo88/vais/issues)
- **명령어 도움말**: `vais --help`
