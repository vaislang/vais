# Frequently Asked Questions (FAQ)

Vais에 대해 자주 묻는 질문들입니다.

## 일반 질문

### Vais는 무엇인가요?

Vais (Vibe AI Language for Systems)는 AI 코드 생성에 최적화된 시스템 프로그래밍 언어입니다. Rust와 같은 안전성을 제공하면서도 단일 문자 키워드로 토큰을 절감합니다.

### Vais의 주요 특징은 무엇인가요?

- **AI 최적화**: 단일 문자 키워드로 토큰 수 50% 감소
- **타입 안전성**: 완전한 타입 추론과 검사
- **시스템 프로그래밍**: C 수준의 성능과 제어
- **LLVM 백엔드**: 최신 최적화 기술 활용
- **현대적 문법**: 함수형, 객체지향, 절차형 프로그래밍 지원

### Vais는 어디에 사용할 수 있나요?

- 시스템 소프트웨어 (OS, 커널 모듈)
- 고성능 서버 (웹 서버, 게임 서버)
- 데이터 처리 (데이터베이스, 빅데이터)
- 임베디드 시스템 (IoT, 펌웨어)
- 컴파일러와 도구
- 암호화폐와 블록체인

### 현재 Vais의 성숙도는 어떻게 되나요?

Vais는 현재 **알파 단계**입니다:
- 기본 언어 기능: 완성
- 표준 라이브러리: 진행 중 (50% 완성)
- 컴파일러 안정성: 높음
- 프로덕션 준비: 아직 미흡

프로덕션 사용을 위해서는 Phase 34를 기다려주세요.

## 언어 설계 질문

### 왜 단일 문자 키워드를 사용하나요?

단일 문자 키워드의 장점:

1. **토큰 절감**: AI 모델의 컨텍스트 윈도우 활용도 증가
   - `if`, `else`, `while` (3 토큰) → `I`, `E`, `L` (1 토큰)
   - 평균 토큰 수 50% 감소

2. **입력 속도**: 사람이 타이핑할 때 빠름
   - `function` (1 토큰) → `F` (1 토큰)

3. **가독성**: 일단 익숙해지면 명확함
   ```vais
   # 전통 문법
   if condition {
       for i = 0; i < 10; i++ {
           return i * 2
       }
   }

   # Vais
   I condition {
       L i := 0; i < 10; i = i + 1 {
           R i * 2
       }
   }
   ```

### AI 코드 생성에 어떻게 최적화되나요?

```
전통 언어 vs Vais (같은 코드):

# Python: 156 토큰
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

# Vais: 78 토큰
F fibonacci(n: i64) -> i64 {
    I n <= 1 { R n }
    R fibonacci(n - 1) + fibonacci(n - 2)
}

절감 효과:
- 토큰 수: 50% 감소
- 생성 속도: 2배 빠름
- 비용: 50% 절감
```

### Vais는 Rust와 다른 점이 무엇인가요?

| 항목 | Vais | Rust |
|------|------|------|
| 문법 | 간결 (단일 문자) | 상세 (명시적) |
| 학습곡선 | 가파름 | 매우 가파름 |
| AI 최적화 | 예 | 아니오 |
| 토큰 수 | 50% 감소 | 기준 |
| 성능 | 동급 | 동급 |
| 안전성 | 높음 | 매우 높음 |
| 커뮤니티 | 작음 | 매우 큼 |
| 생태계 | 초기 | 성숙 |

### Vais는 Go와 다른 점이 무엇인가요?

| 항목 | Vais | Go |
|------|------|-----|
| 문법 | 간결 | 간단 |
| 타입 추론 | 완전 | 부분 |
| 제네릭 | 완전 | 완전 (1.18+) |
| 메모리 관리 | 수동 + 선택적 GC | GC (필수) |
| 성능 | C 수준 | Go 수준 |
| 컴파일 속도 | 중간 | 빠름 |
| 바이너리 크기 | 작음 | 중간 |
| AI 최적화 | 예 | 아니오 |

### Vais는 Zig과 다른 점이 무엇인가요?

| 항목 | Vais | Zig |
|------|------|-----|
| 문법 | 간결 | 상세 |
| 학습곡선 | 가파름 | 가파름 |
| 메타프로그래밍 | 선택적 | 강력 |
| 컴파일 속도 | 중간 | 빠름 |
| AI 최적화 | 예 | 아니오 |
| 커뮤니티 | 초기 | 초기 |
| C 호환성 | 부분 | 전체 |

## 마이그레이션 질문

### C 개발자가 Vais를 배우려면 어떻게 하나요?

#### 1단계: 기본 문법 학습 (1-2일)

```vais
# C 코드
int add(int a, int b) {
    return a + b;
}

# Vais 코드 (동일한 기능)
F add(a: i64, b: i64) -> i64 = a + b
```

#### 2단계: 고급 기능 학습 (3-5일)

```vais
# 제네릭 (C에는 없음)
F max<T>(a: T, b: T) -> T {
    I a > b { R a } E { R b }
}

# 패턴 매칭 (C에는 없음)
M result {
    Result.Ok(val) => puts("Success: {val}"),
    Result.Err(err) => puts("Error: {err}")
}
```

#### 3단계: 프로젝트 작성 (1주)

기본 프로젝트 작성하기:
- 파일 I/O
- 네트워크 통신
- 데이터 구조

#### 학습 리소스

- [Getting Started](./getting-started.md)
- [언어 사양](../language/language-spec.md)
- [예제 코드](https://github.com/vaislang/vais/tree/main/examples)

### Rust 개발자가 Vais를 배우려면 어떻게 하나요?

#### 주요 차이점

```vais
# Rust 코드
fn main() {
    let x = 5;
    println!("x = {}", x);
}

# Vais 코드 (동일)
F main() {
    x := 5
    puts("x = {x}")
}
```

#### 익숙한 부분

- 소유권 개념은 유사 (GC 옵션은 추가)
- 패턴 매칭: 유사하지만 간단함
- 제네릭: 유사하지만 더 간단함
- 에러 처리: Result 타입 유사

#### 새로운 부분

- 단일 문자 키워드: 적응 필요
- 토큰 최적화: Rust와 다름
- 파이프 연산자: 문법이 다름 (`|>` vs `|`)

### Go 개발자가 Vais로 마이그레이션할 수 있을까요?

Go와 Vais의 주요 차이:

```go
// Go
func main() {
    for i := 0; i < 10; i++ {
        fmt.Println(i)
    }
}
```

```vais
// Vais (유사한 구조)
F main() {
    L i := 0; i < 10; i = i + 1 {
        puts("{i}")
    }
}
```

#### 적응 방법

1. **동시성**: Go의 goroutine은 없음, 대신 비동기 함수 사용
2. **인터페이스**: Go의 implicit interface와 유사
3. **에러 처리**: Result 타입 사용 (더 명시적)

## 기술 질문

### Vais가 컴파일되는 방식은?

```
Vais 소스 코드
     ↓
렉서 (Lexer)
     ↓
파서 (Parser) → AST
     ↓
타입 검사 (Type Checker)
     ↓
코드 생성 (Codegen) → LLVM IR
     ↓
LLVM 최적화
     ↓
clang/LLVM
     ↓
머신 코드 → 바이너리
```

### Vais 프로그램의 성능은 어느 정도인가요?

성능 비교 (벤치마크):

```
작업                    Vais    Rust    C       Go
------------------------------------------------------
Hello World         1ms     2ms     1ms     5ms
Fibonacci(30)       400ms   400ms   400ms   450ms
문자열 처리          10ms    10ms    12ms    15ms
네트워크 요청       50ms    50ms    55ms    60ms

결론: Vais는 Rust/C와 동일한 성능
```

### GC가 필수인가요?

아니오, GC는 선택사항입니다:

```toml
# Vais.toml
[profile.release]
gc = false          # GC 비활성화 (수동 관리)
```

```toml
# GC 활성화 (선택적)
[profile.release]
gc = true
gc-heap-size = "1GB"
gc-threads = 4
```

### 어떤 플랫폼을 지원하나요?

- **Linux**: x86_64, ARM64
- **macOS**: x86_64, Apple Silicon (ARM64)
- **Windows**: x86_64
- **웹어셈블리**: WASM
- **embedded**: ARM 기반 microcontroller (experimental)

## 협업 질문

### 팀 프로젝트에서 Vais를 사용해도 될까요?

**준비도 확인**:

```
□ 팀원들이 Vais 학습에 동의했나요? (1-2주 필요)
□ 프로덕션 배포 요구사항이 있나요? (아직 알파)
□ IDE 지원이 충분한가요? (VSCode, IntelliJ 지원)
□ 라이브러리 생태계가 충분한가요? (부분적)
□ 커뮤니티 지원이 중요한가요? (Rust/Go보다 작음)
```

**권장사항**:

- 신규 프로젝트: 가능 (작은 팀, 실험적 성격)
- 기존 프로젝트 전환: 아직 권장 안 함
- 프로덕션: 기다려주세요 (Phase 34 예정)

### Vais 코드를 C/Rust로 변환할 수 있나요?

완벽한 자동 변환은 어렵지만:

1. **Vais → LLVM IR**: 가능 (중간 표현)
2. **LLVM IR → C**: 가능하지만 가독성 낮음
3. **수동 변환**: 권장 (기능별로)

Vais의 목표는 지속적인 사용이므로, 변환보다는 **Vais 개선**을 고려하세요.

## 배포 질문

### 프로덕션 환경에서 Vais를 사용할 수 있나요?

현재 상태:

```
■■■■■■■■■■ 문법 및 기본 기능 (95% 완성)
■■■■■■■■□□ 표준 라이브러리 (60% 완성)
■■■■■■■■□□ 컴파일러 안정성 (80% 완성)
■■■■□□□□□□ 프로덕션 준비 (40% 완성)
```

**권장 용도**:

- ✓ 실험적 프로젝트
- ✓ 개인 도구
- ✓ 학습 프로젝트
- ✗ 중요한 비즈니스 로직 (아직)
- ✗ 높은 가용성 요구 시스템 (아직)

### Vais로 만든 바이너리를 배포하는 방법은?

```bash
# 릴리스 빌드
vaisc build --release myapp.vais -o myapp

# 최적화된 빌드
vaisc build --release --lto myapp.vais -o myapp

# 크기 최소화
vaisc build --release --strip myapp.vais -o myapp

# 정적 링킹
vaisc build --release --static myapp.vais -o myapp
```

배포 체크리스트:

```
□ 릴리스 빌드 (-O2 또는 -O3)
□ LTO 활성화
□ 불필요한 심볼 제거 (--strip)
□ 테스트 실행
□ 성능 벤치마크
□ 메모리 사용량 검증
□ 보안 감시
```

## 성능 질문

### Vais로 작성한 프로그램이 Rust와 같은 성능인가요?

예, 대부분의 경우 동일한 성능입니다:

```
같은 알고리즘 → 유사한 LLVM IR → 유사한 성능
```

**예외**:

- 복잡한 제네릭: Vais가 약간 더 빠를 수 있음
- 타입 추론: Vais가 약간 느릴 수 있음

**권장**: 벤치마크로 확인하세요.

### 작은 바이너리를 만들 수 있나요?

```bash
# 최소 크기 빌드
vaisc build --release --lto --strip -Copt-level=z myapp.vais -o myapp

# 결과 (일반적)
Hello World: ~50KB
간단한 도구: ~200KB
복잡한 앱: ~1-5MB
```

## 커뮤니티 질문

### Vais 커뮤니티는 어떻게 구성되어 있나요?

- **GitHub**: https://github.com/vaislang/vais
- **Discord**: (준비 중)
- **포럼**: (준비 중)
- **IRC**: (준비 중)

### 버그를 발견했어요, 어디에 보고하나요?

GitHub Issues: https://github.com/vaislang/vais/issues

보고 방법:

1. 문제 재현 방법 명확하게
2. 예상 동작과 실제 동작
3. 환경 정보 (OS, Rust 버전)
4. 최소 예제 제공

### Vais에 기여할 수 있나요?

환영합니다! 기여 방법:

1. **이슈 찾기**: https://github.com/vaislang/vais/issues
2. **포크 후 수정**: 기능 추가 또는 버그 수정
3. **PR 제출**: 설명과 함께 변경사항 제시
4. **리뷰 대기**: 커뮤니티 피드백 수렴

## 다음 단계

- [Getting Started](./getting-started.md): 시작하기
- [코딩 스타일 가이드](./style-guide.md): 커뮤니티 스타일
- [에러 처리 가이드](./error-handling.md): 안전한 코딩
