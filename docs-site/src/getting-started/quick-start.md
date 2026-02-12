# 빠른 시작

Vais 프로그래밍 언어를 빠르게 시작하는 방법입니다.

## 설치

```bash
# macOS / Linux (Homebrew)
brew tap vaislang/tap && brew install vais

# 또는 Cargo
cargo install vaisc
```

> 소스에서 직접 빌드하려면 [Getting Started](../guide/getting-started.md#소스에서-빌드) 가이드를 참고하세요.

## 첫 번째 프로그램

다음 코드를 `hello.vais` 파일로 저장합니다:

```vais
F main() {
    puts("Hello, Vais!")
}
```

## 컴파일 및 실행

```bash
# 컴파일
vaisc build hello.vais -o hello
./hello

# 또는 직접 실행
vaisc run hello.vais
```

## 다음 단계

- [튜토리얼](./tutorial.md)에서 더 자세한 내용을 확인하세요
- [언어 사양](../language/language-spec.md)에서 전체 문법을 배우세요
- [예제 코드](https://github.com/vaislang/vais/tree/main/examples)를 살펴보세요
