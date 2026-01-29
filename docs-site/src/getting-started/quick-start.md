# 빠른 시작

Vais 프로그래밍 언어를 빠르게 시작하는 방법입니다.

## 설치

먼저 Vais 컴파일러를 설치합니다:

```bash
cargo build --release
```

## 첫 번째 프로그램

다음 코드를 `hello.vais` 파일로 저장합니다:

```vais
F main() {
    print("Hello, Vais!")
}
```

## 컴파일 및 실행

프로그램을 컴파일하고 실행합니다:

```bash
# 컴파일
./target/release/vaisc build hello.vais -o hello

# 실행
./hello
```

또는 직접 실행:

```bash
./target/release/vaisc run hello.vais
```

## 다음 단계

- [튜토리얼](./tutorial.md)에서 더 자세한 내용을 확인하세요
- [언어 사양](../language/language-spec.md)에서 전체 문법을 배우세요
- [예제 코드](https://github.com/sswoo88/vais/tree/main/examples)를 살펴보세요
