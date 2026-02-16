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

**출력:**
```
Hello, Vais!
```

## 기본 문법

### 변수

```vais
F main() {
    x := 42              # i64 타입으로 추론됨
    y := 3.14            # f64 타입으로 추론됨
    name := "Alice"      # str 타입으로 추론됨
    flag := true         # bool 타입으로 추론됨

    puts("Variables declared!")
}
```

### 함수

```vais
F add(a: i64, b: i64) -> i64 {
    a + b  # 마지막 표현식이 반환값
}

F main() {
    result := add(10, 20)
    print_i64(result)  # 출력: 30
}
```

### 제어 흐름

```vais
F main() {
    x := 10

    # if 표현식
    msg := I x > 5 { "big" } E { "small" }
    puts(msg)

    # 반복문
    L i:0..5 {
        print_i64(i)
    }
}
```

### 자기 재귀

`@`를 사용하여 현재 함수를 호출합니다:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}

F main() {
    print_i64(factorial(5))  # 출력: 120
}
```

## 다음 단계

- [튜토리얼](./tutorial.md) - Vais를 심도 있게 배우기
- [언어 사양](../language/language-spec.md) - 전체 문법 참조
- [예제 프로그램](https://github.com/vaislang/vais/tree/main/examples) - 코드 샘플 둘러보기
