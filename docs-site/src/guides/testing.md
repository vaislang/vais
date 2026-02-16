# 테스팅 가이드

Vais 프로그램의 품질을 보장하기 위한 테스트 작성 방법을 안내합니다.

## 개요

Vais는 다양한 수준의 테스트를 지원합니다:

- **단위 테스트**: 개별 함수와 모듈의 동작 검증
- **통합 테스트**: 여러 모듈 간 상호작용 검증
- **속성 기반 테스트**: 무작위 입력으로 속성 검증
- **E2E 테스트**: 컴파일부터 실행까지 전체 파이프라인 검증
- **벤치마킹**: 성능 측정 및 회귀 감지

## 단위 테스트

### 기본 테스트 함수

Vais에서 테스트 함수는 `test_` 접두사로 시작하는 함수로 작성합니다:

```vais
# math.vais
F add(a: i64, b: i64) -> i64 {
    a + b
}

F test_add_positive() {
    assert_eq(add(2, 3), 5)
}

F test_add_negative() {
    assert_eq(add(-5, 3), -2)
}

F test_add_zero() {
    assert_eq(add(0, 0), 0)
}
```

### Assert 빌트인

Vais는 다음 assert 빌트인을 제공합니다:

```vais
# 조건 검증
assert(condition)
assert(x > 0)

# 값 동등성 검증
assert_eq(actual, expected)
assert_eq(result, 42)

# 부동소수점 근사 비교 (epsilon: 1e-10)
assert_approx(3.14159, pi(), 0.0001)
```

### 구조체 테스트

```vais
S Point {
    x: i64,
    y: i64
}

F Point::distance_squared(self) -> i64 {
    self.x * self.x + self.y * self.y
}

F test_point_distance() {
    p := Point { x: 3, y: 4 }
    assert_eq(p.distance_squared(), 25)
}

F test_point_origin() {
    origin := Point { x: 0, y: 0 }
    assert_eq(origin.distance_squared(), 0)
}
```

### Enum 및 패턴 매칭 테스트

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 {
        R Result::Err("division by zero")
    }
    Result::Ok(a / b)
}

F test_divide_success() {
    result := divide(10, 2)
    M result {
        Result::Ok(val) => assert_eq(val, 5),
        Result::Err(_) => assert(false)
    }
}

F test_divide_by_zero() {
    result := divide(10, 0)
    M result {
        Result::Ok(_) => assert(false),
        Result::Err(msg) => assert_eq(msg, "division by zero")
    }
}
```

## 통합 테스트

### 모듈 간 테스트

여러 모듈을 조합하여 테스트합니다:

```vais
# tests/integration_test.vais
U std/vec
U std/string

F test_vec_operations() {
    v := Vec::new()
    v.push(10)
    v.push(20)
    v.push(30)

    assert_eq(v.len(), 3)
    assert_eq(v.get(0), 10)
    assert_eq(v.get(2), 30)
}

F test_string_manipulation() {
    s := String::from("hello")
    s.push_str(" world")

    assert_eq(s.len(), 11)
    assert_eq(s.as_str(), "hello world")
}
```

### 파일 I/O 테스트

```vais
U std/fs

F test_file_roundtrip() {
    path := "/tmp/test.txt"
    content := "test data"

    # 파일 쓰기
    fs::write_file(path, content)

    # 파일 읽기
    read_content := fs::read_file(path)
    assert_eq(read_content, content)

    # 정리
    fs::remove_file(path)
}
```

## 속성 기반 테스트

`vais-testgen` 크레이트를 사용하여 속성 기반 테스트를 작성할 수 있습니다:

```vais
U vais_testgen/property

# 속성: 리스트를 정렬한 후 뒤집으면 내림차순 정렬과 같다
F prop_sort_reverse() {
    gen := property::list_i64(100)

    property::check(gen, |xs| {
        sorted := xs.sort()
        reversed := sorted.reverse()
        descending := xs.sort_by(|a, b| b - a)

        assert_eq(reversed, descending)
    })
}

# 속성: 문자열 연결은 결합법칙을 만족한다
F prop_string_concat_associative() {
    gen := property::string_gen(50)

    property::check_triple(gen, |a, b, c| {
        left := a.concat(b).concat(c)
        right := a.concat(b.concat(c))

        assert_eq(left, right)
    })
}
```

## E2E 테스트

### 컴파일 및 실행 테스트

```bash
# 컴파일 성공 테스트
vaisc examples/hello.vais -o hello
./hello

# 출력 검증
output=$(./hello)
test "$output" = "Hello, Vais!"

# 종료 코드 검증
./hello
test $? -eq 0
```

### 에러 케이스 테스트

```bash
# 컴파일 에러 테스트
vaisc examples/invalid.vais 2>&1 | grep "E032: Type inference failed"

# 런타임 에러 테스트 (예상된 panic)
vaisc examples/bounds_check.vais -o bounds_check
! ./bounds_check  # 실패가 예상되므로 ! 사용
```

### Rust 기반 E2E 테스트

```rust
// crates/vaisc/tests/e2e_tests.rs
#[test]
fn test_fibonacci() {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "vaisc", "--", "examples/fibonacci.vais"])
        .output()
        .expect("Failed to execute vaisc");

    assert!(output.status.success());

    // 컴파일된 바이너리 실행
    let result = std::process::Command::new("./fibonacci")
        .output()
        .expect("Failed to execute fibonacci");

    assert_eq!(String::from_utf8_lossy(&result.stdout), "55\n");
}
```

## 벤치마킹

### Criterion 기반 벤치마크 (Rust)

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vais_parser::Parser;
use vais_lexer::tokenize;

fn bench_parser(c: &mut Criterion) {
    let source = std::fs::read_to_string("examples/large.vais").unwrap();
    let tokens = tokenize(&source).unwrap();

    c.bench_function("parse_large_file", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            parser.parse_module()
        })
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
```

### 런타임 벤치마크 (Vais)

```vais
# benches/bench_sorting.vais
U std/vec
U std/time

F benchmark_bubble_sort(size: i64) -> i64 {
    v := Vec::with_capacity(size)

    # 무작위 데이터 생성
    L i:0..size {
        v.push(size - i)
    }

    start := time::now_micros()
    bubble_sort(&v)
    end := time::now_micros()

    end - start  # 마이크로초 단위 반환
}

F bubble_sort(arr: &mut Vec<i64>) {
    n := arr.len()
    L i:0..n {
        L j:0..(n - i - 1) {
            I arr.get(j) > arr.get(j + 1) {
                swap(arr, j, j + 1)
            }
        }
    }
}

F main() -> i64 {
    time_1k := benchmark_bubble_sort(1000)
    time_5k := benchmark_bubble_sort(5000)

    print_i64(time_1k)
    print_i64(time_5k)

    0
}
```

컴파일 후 실행:

```bash
vaisc benches/bench_sorting.vais --emit-ir -o bench_sorting.ll
clang bench_sorting.ll -o bench_sorting -lm
./bench_sorting
```

## 코드 커버리지

### LLVM Source-Based Coverage

Vais 컴파일러는 `--coverage` 플래그로 LLVM 커버리지를 지원합니다:

```bash
# 커버리지 활성화하여 컴파일
vaisc --coverage src/main.vais -o main

# 실행하여 프로파일 생성
./main  # default.profraw 생성됨

# HTML 리포트 생성
llvm-profdata merge -sparse default.profraw -o default.profdata
llvm-cov show ./main -instr-profile=default.profdata -format=html > coverage.html
```

### Rust 컴파일러 커버리지

```bash
# Rust 컴파일러 테스트 커버리지
cargo install cargo-llvm-cov
cargo llvm-cov --html --open
```

## CI 통합

### GitHub Actions 워크플로우

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install LLVM 17
        run: |
          wget https://apt.llvm.org/llvm.sh
          chmod +x llvm.sh
          sudo ./llvm.sh 17

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Run tests
        run: cargo test --workspace

      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings

      - name: E2E tests
        run: |
          cargo build --release
          ./target/release/vaisc examples/hello.vais -o hello
          ./hello
```

### 커버리지 리포팅

```yaml
# codecov 통합
- name: Generate coverage
  run: cargo llvm-cov --lcov --output-path lcov.info

- name: Upload to codecov
  uses: codecov/codecov-action@v3
  with:
    files: lcov.info
```

## 테스트 조직화

### 디렉토리 구조

```
my_project/
├── src/
│   ├── main.vais
│   ├── lib.vais
│   └── utils.vais
├── tests/
│   ├── unit/
│   │   ├── test_lib.vais
│   │   └── test_utils.vais
│   ├── integration/
│   │   └── test_full_pipeline.vais
│   └── e2e/
│       └── test_cli.sh
└── benches/
    └── bench_performance.vais
```

### 테스트 명명 규칙

- 단위 테스트: `test_<function_name>_<scenario>`
- 통합 테스트: `test_<module>_<interaction>`
- E2E 테스트: `test_e2e_<feature>`
- 벤치마크: `bench_<operation>_<size>`

## 모범 사례

1. **독립성**: 각 테스트는 독립적으로 실행 가능해야 합니다
2. **결정성**: 테스트 결과는 항상 동일해야 합니다
3. **빠른 실행**: 단위 테스트는 밀리초 단위로 실행되어야 합니다
4. **명확한 실패 메시지**: assert 시 의미 있는 메시지를 포함합니다
5. **경계 조건**: 0, 음수, 최대값 등 경계 케이스를 테스트합니다
6. **에러 경로**: 정상 경로뿐만 아니라 에러 경로도 테스트합니다

## 추가 자료

- [Vais 표준 라이브러리](../stdlib.md)
- [컴파일러 아키텍처](../architecture.md)
- [CI/CD 가이드](https://github.com/vaislang/vais/blob/main/.github/workflows/)
- [벤치마크 결과](https://github.com/vaislang/vais/tree/main/benches)
