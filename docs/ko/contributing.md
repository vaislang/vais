# Vais 기여 가이드

Vais 프로젝트에 기여해 주셔서 감사합니다! 이 문서에서는 프로젝트에 기여하는 방법을 안내합니다.

## 목차

- [기여 방법](#기여-방법)
- [개발 환경 설정](#개발-환경-설정)
- [프로젝트 구조](#프로젝트-구조)
- [코딩 스타일](#코딩-스타일)
- [테스트](#테스트)
- [Pull Request 가이드](#pull-request-가이드)
- [이슈 보고](#이슈-보고)
- [문서 기여](#문서-기여)

---

## 기여 방법

Vais에 기여하는 방법은 여러 가지가 있습니다:

1. **버그 보고** - 문제를 발견하면 이슈를 열어주세요
2. **기능 제안** - 새로운 기능 아이디어를 제안해주세요
3. **코드 기여** - 버그 수정이나 새 기능 구현
4. **문서 개선** - 문서의 오류 수정이나 내용 추가
5. **예제 작성** - 유용한 예제 코드 추가
6. **테스트 추가** - 테스트 커버리지 향상

---

## 개발 환경 설정

### 요구 사항

- **Rust** 1.70 이상
- **Cargo** (Rust 패키지 매니저)
- **Git**

### 저장소 클론 및 빌드

```bash
# 저장소 포크 후 클론
git clone https://github.com/YOUR_USERNAME/vais.git
cd vais/vais-rs

# 의존성 설치 및 빌드
cargo build

# 테스트 실행
cargo test

# 릴리스 빌드
cargo build --release
```

### JIT 기능 활성화 (선택사항)

```bash
cargo build --release --features jit
```

### 개발 도구

- **cargo fmt** - 코드 포맷팅
- **cargo clippy** - 린팅(Linting)
- **cargo test** - 테스트 실행
- **cargo doc** - 문서 생성

---

## 프로젝트 구조

```
vais-rs/
├── crates/
│   ├── vais-lexer/      # 렉서(Lexer) - 토큰화
│   ├── vais-ast/        # AST 정의
│   ├── vais-parser/     # 파서(Parser) + 모듈 시스템
│   ├── vais-typeck/     # 타입 체커(Type Checker) - Hindley-Milner
│   ├── vais-ir/         # IR + 최적화
│   ├── vais-lowering/   # AST → IR 변환
│   ├── vais-vm/         # 스택 기반 가상 머신
│   ├── vais-jit/        # 적응형 JIT (Cranelift)
│   ├── vais-codegen/    # 코드 생성 (C/WASM/LLVM)
│   ├── vais-tools/      # 포맷터, 프로파일러, 디버거
│   ├── vais-lsp/        # 언어 서버(Language Server)
│   ├── vais-playground/ # 웹 플레이그라운드 (WASM)
│   └── vais-cli/        # CLI 인터페이스
├── examples/            # 예제 파일들
├── tests/               # 통합 테스트
├── benches/             # 벤치마크
└── docs/                # 문서
```

### 주요 크레이트(Crate) 설명

| 크레이트 | 역할 |
|----------|------|
| `vais-lexer` | 소스 코드를 토큰으로 변환 |
| `vais-parser` | 토큰을 AST로 파싱 |
| `vais-typeck` | 타입 검사 및 추론 |
| `vais-ir` | 중간 표현(IR) 및 최적화 |
| `vais-vm` | 바이트코드 실행 |
| `vais-jit` | JIT 컴파일 (Cranelift 기반) |
| `vais-codegen` | 네이티브 코드 생성 |

---

## 코딩 스타일

### Rust 코딩 규칙

1. **포맷팅**: `cargo fmt`로 코드 포맷팅
2. **린팅**: `cargo clippy`로 경고 없이 통과
3. **문서화**: 공개 API에 문서 주석 추가

```rust
/// 두 수를 더합니다.
///
/// # 예제
///
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

### 명명 규칙

- **타입/구조체**: `PascalCase` (예: `TokenKind`, `AstNode`)
- **함수/변수**: `snake_case` (예: `parse_expression`, `token_count`)
- **상수**: `SCREAMING_SNAKE_CASE` (예: `MAX_RECURSION_DEPTH`)
- **모듈**: `snake_case` (예: `type_checker`, `code_gen`)

### 커밋 메시지 규칙

```
<type>(<scope>): <subject>

<body>

<footer>
```

**타입**:
- `feat`: 새 기능
- `fix`: 버그 수정
- `docs`: 문서 변경
- `style`: 포맷팅 변경
- `refactor`: 리팩토링
- `test`: 테스트 추가/수정
- `chore`: 빌드/도구 변경

**예시**:
```
feat(parser): add support for pattern matching

Implement pattern matching syntax with guard clauses.

Closes #123
```

---

## 테스트

### 테스트 실행

```bash
# 모든 테스트 실행
cargo test

# 특정 크레이트 테스트
cargo test -p vais-parser

# 특정 테스트 실행
cargo test test_factorial

# 상세 출력
cargo test -- --nocapture
```

### 테스트 작성 가이드

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let result = eval("1 + 2");
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_factorial() {
        let code = "factorial(n) = n < 2 ? 1 : n * $(n - 1); factorial(5)";
        let result = eval(code);
        assert_eq!(result, Value::Int(120));
    }
}
```

### 벤치마크 실행

```bash
cargo bench
```

---

## Pull Request 가이드

### PR 프로세스

1. **포크(Fork)**: 저장소를 포크합니다
2. **브랜치 생성**: 기능/수정을 위한 브랜치를 생성합니다
   ```bash
   git checkout -b feature/my-feature
   ```
3. **변경 사항 커밋**: 의미 있는 커밋 메시지로 커밋합니다
4. **테스트 통과**: 모든 테스트가 통과하는지 확인합니다
5. **PR 생성**: Pull Request를 생성합니다

### PR 체크리스트

- [ ] 코드가 `cargo fmt`로 포맷됨
- [ ] `cargo clippy`에서 경고 없음
- [ ] 모든 테스트 통과 (`cargo test`)
- [ ] 새 기능에 대한 테스트 추가
- [ ] 문서 업데이트 (필요시)
- [ ] 커밋 메시지가 규칙을 따름

### PR 템플릿

```markdown
## 설명
<!-- 변경 사항에 대한 간단한 설명 -->

## 변경 유형
- [ ] 버그 수정
- [ ] 새 기능
- [ ] 문서 업데이트
- [ ] 리팩토링
- [ ] 기타

## 테스트
<!-- 테스트 방법 설명 -->

## 관련 이슈
<!-- 관련 이슈 번호 (예: Closes #123) -->
```

---

## 이슈 보고

### 버그 보고

버그를 보고할 때는 다음 정보를 포함해주세요:

1. **환경**
   - OS 및 버전
   - Rust 버전
   - Vais 버전

2. **재현 단계**
   - 버그를 재현하는 정확한 단계

3. **예상 동작**
   - 기대했던 동작

4. **실제 동작**
   - 실제로 발생한 동작

5. **코드 예제**
   - 문제를 재현하는 최소한의 코드

### 버그 보고 템플릿

```markdown
## 버그 설명
<!-- 버그에 대한 명확한 설명 -->

## 재현 단계
1.
2.
3.

## 예상 동작
<!-- 기대했던 동작 -->

## 실제 동작
<!-- 실제로 발생한 동작 -->

## 환경
- OS:
- Rust 버전:
- Vais 버전:

## 추가 정보
<!-- 스크린샷, 로그 등 -->
```

### 기능 요청

새 기능을 제안할 때는 다음을 포함해주세요:

1. **사용 사례** - 이 기능이 필요한 이유
2. **제안하는 솔루션** - 원하는 기능의 동작 방식
3. **대안** - 고려한 다른 방안
4. **예제** - 기능 사용 예제 코드

---

## 문서 기여

### 문서 구조

```
docs/
├── api.md          # API 레퍼런스
├── syntax.md       # 문법 가이드
├── examples.md     # 예제
└── ko/             # 한국어 문서
    ├── README.md
    ├── getting-started.md
    ├── syntax.md
    ├── api.md
    ├── examples.md
    └── contributing.md
```

### 문서 작성 가이드

1. **명확성**: 명확하고 이해하기 쉽게 작성
2. **예제**: 가능한 한 코드 예제 포함
3. **일관성**: 기존 문서 스타일과 일관되게 작성
4. **링크**: 관련 문서로의 링크 포함

### 문서 미리보기

문서를 작성한 후 마크다운 미리보기로 확인하세요.

---

## 라이선스

Vais는 [MIT 라이선스](../../LICENSE)로 배포됩니다. 기여하는 코드도 동일한 라이선스를 따릅니다.

---

## 연락처

- **GitHub Issues**: [github.com/sswoo88/vais/issues](https://github.com/sswoo88/vais/issues)
- **GitHub Discussions**: [github.com/sswoo88/vais/discussions](https://github.com/sswoo88/vais/discussions)

기여에 대해 궁금한 점이 있으면 이슈나 토론에서 질문해주세요!

---

## 관련 문서

- [시작 가이드](getting-started.md)
- [문법 가이드](syntax.md)
- [API 레퍼런스](api.md)
- [예제](examples.md)
