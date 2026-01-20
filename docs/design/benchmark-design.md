# Vais Benchmark Suite Design

## 개요

Vais 컴파일러의 성능을 측정하고 회귀를 감지하는 벤치마크 스위트입니다.

## 목표

1. **컴파일 성능 측정**: 렉싱, 파싱, 타입체킹, 코드젠 각 단계별 시간 측정
2. **런타임 성능 측정**: 생성된 코드의 실행 성능 측정
3. **회귀 감지**: CI에서 자동으로 성능 저하 감지
4. **비교 분석**: 버전 간, 최적화 레벨 간 비교

## 아키텍처

```
benches/
├── Cargo.toml              # Criterion 의존성
├── compile_bench.rs        # 컴파일 벤치마크
├── runtime_bench.rs        # 런타임 벤치마크
├── fixtures/               # 테스트용 Vais 소스 파일
│   ├── fibonacci.vais      # 재귀 함수
│   ├── sort.vais           # 정렬 알고리즘
│   ├── string_ops.vais     # 문자열 연산
│   ├── struct_heavy.vais   # 구조체 연산
│   └── complex.vais        # 복합 벤치마크
└── results/                # 벤치마크 결과 저장
```

## 벤치마크 종류

### 1. 컴파일 벤치마크

| 벤치마크 | 측정 대상 | 입력 |
|----------|----------|------|
| lexer_throughput | 초당 토큰 처리량 | 다양한 크기 소스 |
| parser_throughput | 초당 AST 노드 생성 | 복잡도별 소스 |
| typecheck_time | 타입 검사 시간 | 제네릭, 트레이트 포함 |
| codegen_time | IR 생성 시간 | 함수/구조체 수별 |
| full_compile | 전체 컴파일 시간 | 실제 프로젝트 규모 |

### 2. 런타임 벤치마크

| 벤치마크 | 측정 대상 | 비교 대상 |
|----------|----------|----------|
| fibonacci | 재귀 성능 | C, Rust |
| quicksort | 메모리 접근 패턴 | C, Rust |
| string_concat | 문자열 처리 | C, Rust |
| struct_alloc | 힙 할당 성능 | C, Rust |
| closure_call | 클로저 호출 오버헤드 | Rust |

### 3. 최적화 벤치마크

| 최적화 레벨 | 측정 항목 |
|------------|----------|
| O0 | 기준선 (최적화 없음) |
| O1 | 기본 최적화 효과 |
| O2 | 표준 최적화 효과 |
| O3 | 공격적 최적화 효과 |

## 구현 세부사항

### Criterion 설정

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn compile_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");

    for fixture in ["fibonacci", "sort", "complex"] {
        let source = fs::read_to_string(format!("benches/fixtures/{}.vais", fixture)).unwrap();

        group.bench_with_input(
            BenchmarkId::new("lex", fixture),
            &source,
            |b, s| b.iter(|| tokenize(s)),
        );

        group.bench_with_input(
            BenchmarkId::new("parse", fixture),
            &source,
            |b, s| b.iter(|| parse(s)),
        );
    }

    group.finish();
}
```

### 메모리 벤치마크

```rust
// 메모리 사용량 측정 (선택적)
#[cfg(feature = "memory-bench")]
fn memory_benchmark(c: &mut Criterion) {
    // jemalloc 또는 custom allocator 사용
}
```

### CI 통합

```yaml
# .github/workflows/bench.yml
benchmark:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run benchmarks
      run: cargo bench --bench compile_bench -- --save-baseline main
    - name: Compare with baseline
      run: cargo bench --bench compile_bench -- --baseline main
```

## 결과 형식

### JSON 출력

```json
{
  "version": "0.0.1",
  "timestamp": "2026-01-20T12:00:00Z",
  "benchmarks": {
    "compile/lex/fibonacci": {
      "mean": 125.3,
      "std_dev": 2.1,
      "unit": "us"
    }
  }
}
```

### Markdown 리포트

```markdown
## Benchmark Results (v0.0.1)

| Benchmark | Mean | Std Dev | Change |
|-----------|------|---------|--------|
| compile/lex | 125μs | ±2.1μs | - |
| compile/parse | 340μs | ±5.2μs | - |
```

## 회귀 감지 기준

| 변화율 | 판정 | 액션 |
|--------|------|------|
| < -5% | 개선 | 로그만 |
| -5% ~ +5% | 정상 | 무시 |
| +5% ~ +10% | 경고 | PR 코멘트 |
| > +10% | 실패 | CI 실패 |

## 의존성

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

## 실행 방법

```bash
# 전체 벤치마크 실행
cargo bench

# 특정 벤치마크만 실행
cargo bench --bench compile_bench

# HTML 리포트 생성
cargo bench -- --save-baseline current

# 베이스라인과 비교
cargo bench -- --baseline main
```
