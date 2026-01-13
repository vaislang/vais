# Vais Architecture: Extensibility-First Design

**Version:** 1.0.0
**Date:** 2026-01-12
**Status:** Draft

---

## Vision

**"AI 시대의 Python"**

Python이 인간 친화적 문법으로 성공했듯이,
Vais은 **AI 친화적 문법 + 확장 가능한 생태계**로 성공한다.

```
Vais = 작은 코어 + 강력한 확장성 + 커뮤니티 생태계
```

---

## Design Principles

### 1. Small Core, Big Ecosystem

```
┌─────────────────────────────────────────────────────┐
│                   Community Packages                 │
│  (vais-numpy, vais-web, vais-ml, vais-db, ...)     │
├─────────────────────────────────────────────────────┤
│                  Standard Library                    │
│  (std.io, std.net, std.json, std.test, ...)        │
├─────────────────────────────────────────────────────┤
│                   FFI Layer                          │
│  (C, Rust, Python interop)                          │
├─────────────────────────────────────────────────────┤
│                   Core Language                      │
│  (Lexer, Parser, VM, Type System)                   │
└─────────────────────────────────────────────────────┘
```

**원칙:**
- 코어는 최소한으로 유지 (변경 어려움)
- 대부분의 기능은 라이브러리로 구현
- 코어 변경 없이 언어 확장 가능

### 2. Everything is a Package

```
# 언어 기능도 패키지로 제공 가능
use std.async      # async/await 지원
use std.macro      # 매크로 시스템
use std.typing     # 고급 타입 기능
```

### 3. Zero-Cost Abstraction

```
# 사용하지 않는 기능은 비용 0
# 필요한 것만 import하면 최적화됨
```

### 4. FFI First-Class

```
# 기존 생태계 활용 가능
use ffi.python.numpy as np
use ffi.rust.tokio as async_rt
use ffi.c.sqlite as db
```

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                        Vais Ecosystem                         │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   Package   │  │   Package   │  │   Package   │   ...    │
│  │  Registry   │  │   Manager   │  │   Builder   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│                        Standard Library                       │
├──────────┬──────────┬──────────┬──────────┬─────────────────┤
│  std.io  │ std.net  │ std.json │ std.test │ std.async  ...  │
├──────────┴──────────┴──────────┴──────────┴─────────────────┤
│                         FFI Layer                             │
├──────────┬──────────┬──────────┬────────────────────────────┤
│    C     │   Rust   │  Python  │   WASM    (future)         │
├──────────┴──────────┴──────────┴────────────────────────────┤
│                       Core Runtime                            │
├──────────┬──────────┬──────────┬────────────────────────────┤
│    VM    │   GC     │  Thread  │   Scheduler                │
├──────────┴──────────┴──────────┴────────────────────────────┤
│                      Core Language                            │
├──────────┬──────────┬──────────┬────────────────────────────┤
│  Lexer   │  Parser  │   AST    │   Type System              │
└──────────┴──────────┴──────────┴────────────────────────────┘
```

---

## Module System

### Module Definition

```vais
# math.vais
mod math

# Public exports (default private)
pub pi = 3.14159265359
pub e = 2.71828182846

pub fn sin(x) = ...
pub fn cos(x) = ...

# Private helper (not exported)
fn taylor_series(x, n) = ...
```

### Module Usage

```vais
# main.vais
use math                    # import all public
use math.{sin, cos}         # import specific
use math.sin as sine        # alias
use math.*                  # glob import (discouraged)
```

### Module Path Resolution

```
project/
├── vais.toml              # Project config
├── src/
│   ├── main.vais          # Entry point
│   ├── utils.vais         # use utils
│   └── helpers/
│       └── string.vais    # use helpers.string
└── deps/                  # Downloaded packages
    └── http/
        └── src/
            └── lib.vais   # use http
```

---

## Package System

### Package Definition (vais.toml)

```toml
[package]
name = "my-project"
version = "1.0.0"
description = "My awesome Vais project"
authors = ["Your Name <you@example.com>"]
license = "MIT"
repository = "https://github.com/you/my-project"

[dependencies]
std = "1.0"                    # Standard library
http = "2.1"                   # Community package
json = { version = "1.0", optional = true }

[dev-dependencies]
test = "1.0"
benchmark = "0.5"

[features]
default = ["json"]
full = ["json", "xml", "yaml"]

[ffi]
python = ["numpy", "pandas"]   # Python interop
rust = ["tokio"]               # Rust interop
```

### Package Registry (apm - Vais Package Manager)

```bash
# Initialize project
vais init my-project

# Add dependency
vais add http
vais add json --optional

# Install dependencies
vais install

# Publish package
vais publish

# Search packages
vais search "http client"
```

### Registry Structure

```
registry.vais.dev/
├── api/
│   ├── packages/           # Package metadata
│   ├── versions/           # Version info
│   └── downloads/          # Download stats
├── storage/
│   └── packages/           # Actual package files
└── index/
    └── search/             # Search index
```

---

## FFI (Foreign Function Interface)

### Design Goals

1. **기존 생태계 활용** - Python, Rust, C 라이브러리 호출
2. **양방향 통신** - Vais에서 호출 & 외부에서 Vais 호출
3. **타입 안전성** - 자동 타입 변환 및 검증
4. **최소 오버헤드** - 효율적인 데이터 전달

### C FFI

```vais
# C 함수 선언
ffi c {
    # libc
    fn malloc(size: usize) -> *void
    fn free(ptr: *void)

    # Custom library
    @link("mylib")
    fn my_function(a: i32, b: i32) -> i32
}

# 사용
result = c.my_function(10, 20)
```

### Rust FFI

```vais
# Rust crate 연동
ffi rust {
    @crate("tokio", version = "1.0")
    mod async_runtime {
        fn spawn(future: Future) -> JoinHandle
        fn block_on(future: Future) -> T
    }
}

# 사용
handle = rust.async_runtime.spawn(my_async_fn())
```

### Python FFI

```vais
# Python 라이브러리 연동
ffi python {
    @module("numpy")
    mod np {
        fn array(data: [f]) -> NDArray
        fn zeros(shape: (i, i)) -> NDArray
        fn dot(a: NDArray, b: NDArray) -> NDArray
    }

    @module("pandas")
    mod pd {
        fn DataFrame(data: {s: [any]}) -> DataFrame
        fn read_csv(path: s) -> DataFrame
    }
}

# 사용
arr = python.np.array([1.0, 2.0, 3.0])
df = python.pd.read_csv("data.csv")
```

### Type Mapping

```
Vais Type    <->    C Type       <->    Rust Type    <->    Python Type
─────────────────────────────────────────────────────────────────────────
i / i64            int64_t             i64                 int
i32                int32_t             i32                 int
f / f64            double              f64                 float
f32                float               f32                 float
b                  bool                bool                bool
s                  char*               String              str
[T]                T*                  Vec<T>              list
{K:V}              -                   HashMap<K,V>        dict
?T                 T* (nullable)       Option<T>           Optional[T]
```

### Memory Safety

```vais
# 자동 메모리 관리
ffi c {
    @managed  # Vais GC가 관리
    fn create_buffer(size: i) -> *Buffer

    @manual   # 수동 해제 필요
    fn raw_alloc(size: i) -> *void
}

# managed는 자동 해제
buf = c.create_buffer(1024)
# scope 끝나면 자동 해제

# manual은 명시적 해제
ptr = c.raw_alloc(1024)
defer c.free(ptr)  # 명시적 해제
```

---

## Standard Library Structure

### Core Modules (std.*)

```
std/
├── core/              # 언어 기본 (자동 import)
│   ├── types.vais     # 기본 타입 정의
│   ├── ops.vais       # 연산자 트레잇
│   └── prelude.vais   # 기본 함수들
│
├── io/                # 입출력
│   ├── file.vais      # 파일 읽기/쓰기
│   ├── stdin.vais     # 표준 입력
│   ├── stdout.vais    # 표준 출력
│   └── path.vais      # 경로 처리
│
├── net/               # 네트워킹
│   ├── http.vais      # HTTP 클라이언트/서버
│   ├── tcp.vais       # TCP 소켓
│   ├── udp.vais       # UDP 소켓
│   └── url.vais       # URL 파싱
│
├── data/              # 데이터 포맷
│   ├── json.vais      # JSON
│   ├── csv.vais       # CSV
│   ├── toml.vais      # TOML
│   └── xml.vais       # XML
│
├── text/              # 텍스트 처리
│   ├── regex.vais     # 정규표현식
│   ├── fmt.vais       # 포매팅
│   └── encoding.vais  # 인코딩
│
├── time/              # 시간
│   ├── datetime.vais  # 날짜/시간
│   ├── duration.vais  # 기간
│   └── timezone.vais  # 타임존
│
├── math/              # 수학
│   ├── basic.vais     # 기본 수학
│   ├── random.vais    # 난수
│   └── stats.vais     # 통계
│
├── collections/       # 자료구조
│   ├── list.vais      # 리스트 확장
│   ├── set.vais       # 집합
│   ├── map.vais       # 맵 확장
│   ├── queue.vais     # 큐
│   └── heap.vais      # 힙
│
├── async/             # 비동기
│   ├── future.vais    # Future/Promise
│   ├── channel.vais   # 채널
│   └── spawn.vais     # 태스크 생성
│
├── test/              # 테스팅
│   ├── assert.vais    # 어설션
│   ├── mock.vais      # 모킹
│   └── bench.vais     # 벤치마크
│
└── sys/               # 시스템
    ├── env.vais       # 환경변수
    ├── process.vais   # 프로세스
    └── os.vais        # OS 정보
```

### Usage Examples

```vais
use std.io.{read_file, write_file}
use std.net.http.{get, post}
use std.data.json.{parse, stringify}

# 파일 읽기
content = read_file("data.txt")?

# HTTP 요청
response = get("https://api.example.com/data")?
data = parse(response.body)?

# 파일 쓰기
write_file("output.json", stringify(data))?
```

---

## Extension Points

### 1. Custom Operators

```vais
# 연산자 정의 (패키지에서)
mod matrix

pub type Matrix = [[f]]

# 행렬 곱셈 연산자
pub op (a: Matrix) ** (b: Matrix) -> Matrix {
    # 구현
}

# 사용
use matrix.{Matrix, **}
result = mat_a ** mat_b
```

### 2. Custom Syntax (Macros)

```vais
# 매크로 정의
mod html

pub macro html! {
    # HTML DSL
    (<$tag $attrs*>$children*</$tag>) => {
        Element.new($tag, $attrs, $children)
    }
}

# 사용
use html.html!

page = html! {
    <div class="container">
        <h1>"Hello"</h1>
        <p>"World"</p>
    </div>
}
```

### 3. Custom Types with Traits

```vais
# 트레잇 정의
mod iter

pub trait Iterable<T> {
    fn iter(self) -> Iterator<T>
}

pub trait Mappable<T> {
    fn map<U>(self, f: T -> U) -> Self<U>
}

# 커스텀 타입에 구현
mod my_collection

use iter.{Iterable, Mappable}

pub type MyList<T> = ...

impl Iterable<T> for MyList<T> {
    fn iter(self) = ...
}

impl Mappable<T> for MyList<T> {
    fn map(self, f) = ...
}
```

### 4. Compiler Plugins (Future)

```toml
# vais.toml
[plugins]
lint = "vais-clippy"           # 코드 린트
optimize = "vais-optimize"     # 추가 최적화
codegen = "vais-native"        # 네이티브 컴파일
```

---

## Governance Model

### Project Structure

```
Vais Organization
├── Core Team              # 언어 핵심 개발
│   ├── Language Design    # 문법, 타입 시스템
│   ├── Runtime            # VM, GC
│   └── Tooling            # CLI, LSP
│
├── Library Team           # 표준 라이브러리
│   ├── std.io
│   ├── std.net
│   └── ...
│
├── Community Team         # 커뮤니티 관리
│   ├── Documentation
│   ├── Education
│   └── Events
│
└── Package Registry       # 패키지 레지스트리 운영
```

### RFC Process (Request for Comments)

```
1. Idea          → GitHub Discussion
2. Pre-RFC       → 초기 논의
3. RFC           → 정식 제안서
4. Review        → 커뮤니티 리뷰
5. FCP           → Final Comment Period
6. Accepted      → 구현 시작
7. Implemented   → 릴리즈
```

### Versioning

```
Vais 버전 체계:
- Major: 호환성 깨지는 변경 (드묾)
- Minor: 새 기능 추가 (하위 호환)
- Patch: 버그 수정

Edition 시스템 (Rust 참고):
- Edition 2026: 초기 버전
- Edition 2027: 개선 버전
- 이전 Edition 코드도 계속 동작
```

---

## Community Package Examples

### Web Framework (vais-web)

```vais
use web.{App, route, get, post}

app = App.new()

@get("/")
fn index(req) = "Hello, Vais!"

@get("/users/:id")
fn get_user(req) = {
    id = req.params.id
    User.find(id)?
}

@post("/users")
fn create_user(req) = {
    data = req.json()?
    User.create(data)?
}

app.run(port=8080)
```

### Data Science (vais-data)

```vais
use data.{DataFrame, Series}
use data.plot

# 데이터 로드
df = DataFrame.read_csv("sales.csv")

# 데이터 처리
result = df
    .?(_["region"] == "Asia")
    .@{ _["revenue"] * _["quantity"] }
    .groupby("product")
    .sum()

# 시각화
plot.bar(result, x="product", y="total")
    .title("Asia Revenue by Product")
    .save("chart.png")
```

### Machine Learning (vais-ml)

```vais
use ml.{Model, train, predict}
use ml.nn.{Dense, Sequential}

# 모델 정의
model = Sequential([
    Dense(128, activation="relu"),
    Dense(64, activation="relu"),
    Dense(10, activation="softmax")
])

# 학습
model.compile(optimizer="adam", loss="cross_entropy")
model.fit(x_train, y_train, epochs=10)

# 예측
predictions = model.predict(x_test)
```

---

## Implementation Priority

### Phase 1: Core (Month 1-2)
```
[x] Language Spec (v6b)
[ ] Lexer
[ ] Parser
[ ] AST
[ ] Type System
[ ] VM
[ ] Basic CLI
```

### Phase 2: Foundation (Month 2-3)
```
[ ] Module System
[ ] Package Manager (basic)
[ ] std.core
[ ] std.io
[ ] std.collections
```

### Phase 3: FFI (Month 3-4)
```
[ ] C FFI
[ ] Rust FFI
[ ] Python FFI
```

### Phase 4: Ecosystem (Month 4-6)
```
[ ] Package Registry
[ ] Documentation Site
[ ] std.net
[ ] std.data.json
[ ] std.async
```

### Phase 5: Community (Month 6+)
```
[ ] Community Guidelines
[ ] RFC Process
[ ] First Community Packages
[ ] Tutorials & Examples
```

---

## Success Metrics

| Metric | Target (1년) | Target (3년) |
|--------|-------------|--------------|
| GitHub Stars | 1,000+ | 10,000+ |
| Packages | 50+ | 500+ |
| Contributors | 20+ | 100+ |
| Production Users | 10+ | 1,000+ |
| Documentation Pages | 100+ | 500+ |

---

## Summary

Vais의 성공 공식:

```
작은 코어 (안정적, 변경 어려움)
    +
강력한 FFI (기존 생태계 활용)
    +
쉬운 패키지 시스템 (누구나 확장)
    +
좋은 문서 (진입장벽 낮춤)
    +
커뮤니티 거버넌스 (함께 성장)
    =
지속 가능한 생태계
```

**Python이 30년 걸린 것을 AI 시대에는 더 빠르게 달성할 수 있습니다.**
