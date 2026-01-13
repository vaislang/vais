# Vais 한국어 문서

Vais(Vibe AI Script) 프로그래밍 언어의 공식 한국어 문서입니다.

## 문서 목차

### 시작하기

- [시작 가이드](getting-started.md) - 설치 및 첫 프로그램 작성
- [예제](examples.md) - 다양한 실용 예제 모음

### 레퍼런스

- [문법 가이드](syntax.md) - Vais 언어의 완전한 문법 설명
- [API 레퍼런스](api.md) - 내장 함수 및 표준 라이브러리

### 커뮤니티

- [기여 가이드](contributing.md) - 프로젝트에 기여하는 방법

---

## Vais란?

**Vais (Vibe AI Script)**는 AI가 코드를 가장 효율적으로 생성, 수정, 실행할 수 있도록 설계된 프로그래밍 언어입니다. 토큰 효율성에 최적화된 간결한 문법을 제공하면서도 완전한 표현력을 유지합니다.

### 주요 특징

| 특징 | 설명 |
|------|------|
| **토큰 효율적 문법** | Python 대비 30-60% 적은 토큰 |
| **함수형 우선** | 일급 함수(First-class Function), 클로저(Closure), 컬렉션 연산 |
| **자기 재귀(Self-Recursion)** | 우아한 재귀 정의를 위한 `$` 연산자 |
| **컬렉션 연산자** | `.@` (map), `.?` (filter), `./` (reduce) |
| **다중 백엔드(Backend)** | 인터프리터, JIT (50-75배 빠름), C, WASM, LLVM |
| **풍부한 생태계** | LSP, 패키지 매니저, 디버거, 포맷터, 프로파일러 |

### 간단한 예제

```vais
// 함수 정의
add(a, b) = a + b

// 자기 재귀($)를 사용한 팩토리얼
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// 컬렉션 연산
numbers = [1, 2, 3, 4, 5]
doubled = numbers.@(_ * 2)        // [2, 4, 6, 8, 10]
evens = numbers.?(_ % 2 == 0)     // [2, 4]
sum = numbers./+(0, _ + _)        // 15
```

---

## 빠른 시작

### 설치

```bash
# 저장소 클론
git clone https://github.com/sswoo88/vais.git
cd vais/vais-rs

# 빌드
cargo build --release

# PATH에 추가 (선택사항)
export PATH="$PATH:$(pwd)/target/release"
```

### Hello World

```bash
echo 'print("안녕, Vais!")' > hello.vais
./target/release/vais run hello.vais
```

### REPL 실행

```bash
./target/release/vais repl
```

더 자세한 내용은 [시작 가이드](getting-started.md)를 참조하세요.

---

## 관련 링크

- [GitHub 저장소](https://github.com/sswoo88/vais)
- [영어 문서](../)
- [프로젝트 README (한국어)](../../README.ko.md)
