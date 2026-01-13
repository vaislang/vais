# Vais 아키텍처: 확장성 우선 설계

**버전:** 1.0.0
**날짜:** 2026-01-12
**상태:** 초안

---

## 비전

**"AI 시대의 Python"**

Python이 인간 친화적인 구문으로 성공했듯이,
Vais는 **AI 친화적 구문 + 확장 가능한 생태계**로 성공합니다.

```
Vais = 작은 코어 + 강력한 확장성 + 커뮤니티 생태계
```

---

## 설계 원칙

### 1. 작은 코어, 큰 생태계

```
┌─────────────────────────────────────────────────────┐
│                   커뮤니티 패키지                      │
│  (vais-numpy, vais-web, vais-ml, vais-db, ...)     │
├─────────────────────────────────────────────────────┤
│                   표준 라이브러리                      │
│  (std.io, std.net, std.json, std.test, ...)        │
├─────────────────────────────────────────────────────┤
│                     FFI 레이어                        │
│  (C, Rust, Python 연동)                              │
├─────────────────────────────────────────────────────┤
│                     코어 언어                         │
│  (Lexer, Parser, VM, 타입 시스템)                    │
└─────────────────────────────────────────────────────┘
```

**원칙:**
- 코어는 최소화 (변경하기 어려움)
- 대부분의 기능은 라이브러리로 구현
- 코어 변경 없이 언어 확장 가능

### 2. 모든 것은 패키지

```vais
# 언어 기능도 패키지로 제공 가능
use std.async      # async/await 지원
use std.macro      # 매크로 시스템
use std.typing     # 고급 타입 기능
```

### 3. 제로 코스트 추상화

```vais
# 사용하지 않는 기능은 비용 없음
# 임포트된 항목만 최적화됨
```

### 4. FFI 일급 지원

```vais
# 기존 생태계 활용
use ffi.python.numpy as np
use ffi.rust.tokio as async_rt
use ffi.c.sqlite as db
```

---

## 아키텍처 개요

```
┌──────────────────────────────────────────────────────────────┐
│                        Vais 생태계                             │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   패키지     │  │   패키지     │  │   패키지     │   ...    │
│  │  레지스트리   │  │   매니저    │  │   빌더      │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│                        표준 라이브러리                          │
├──────────┬──────────┬──────────┬──────────┬─────────────────┤
│  std.io  │ std.net  │ std.json │ std.test │ std.async  ...  │
├──────────┴──────────┴──────────┴──────────┴─────────────────┤
│                         FFI 레이어                             │
├──────────┬──────────┬──────────┬────────────────────────────┤
│    C     │   Rust   │  Python  │   WASM    (향후)            │
├──────────┴──────────┴──────────┴────────────────────────────┤
│                       코어 런타임                               │
├──────────┬──────────┬──────────┬────────────────────────────┤
│    VM    │   GC     │  스레드   │   스케줄러                   │
├──────────┴──────────┴──────────┴────────────────────────────┤
│                        코어 언어                                │
├──────────┬──────────┬──────────┬────────────────────────────┤
│  Lexer   │  Parser  │   AST    │   타입 시스템                 │
└──────────┴──────────┴──────────┴────────────────────────────┘
```

---

## 모듈 시스템

### 모듈 정의

```vais
# math.vais
mod math

# Public 내보내기 (기본은 private)
pub pi = 3.14159265359
pub e = 2.71828182846

pub sin(x) = ...
pub cos(x) = ...

# Private 헬퍼 (내보내지 않음)
taylor_series(x, n) = ...
```

### 모듈 사용

```vais
# main.vais
use math                    # 모든 public 임포트
use math.{sin, cos}         # 특정 항목 임포트
use math.sin as sine        # 별칭
use math.*                  # 글로브 임포트 (비권장)
```

### 모듈 경로 해석

```
project/
├── vais.toml              # 프로젝트 설정
├── src/
│   ├── main.vais          # 진입점
│   ├── utils.vais         # use utils
│   └── helpers/
│       └── string.vais    # use helpers.string
└── deps/                  # 다운로드된 패키지
    └── http/
        └── src/
            └── lib.vais   # use http
```

---

## 패키지 시스템

### 패키지 정의 (vais.toml)

```toml
[package]
name = "my-project"
version = "1.0.0"
description = "나의 멋진 Vais 프로젝트"
authors = ["Your Name <you@example.com>"]
license = "MIT"
repository = "https://github.com/you/my-project"

[dependencies]
std = "1.0"                    # 표준 라이브러리
http = "2.1"                   # 커뮤니티 패키지
json = { version = "1.0", optional = true }

[dev-dependencies]
test = "1.0"
benchmark = "0.5"

[features]
default = ["json"]
full = ["json", "xml", "yaml"]

[ffi]
python = ["numpy", "pandas"]   # Python 연동
rust = ["tokio"]               # Rust 연동
```

### 패키지 레지스트리 (vais Package Manager)

```bash
# 프로젝트 초기화
vais init my-project

# 의존성 추가
vais add http
vais add json --optional

# 의존성 설치
vais install

# 패키지 퍼블리시
vais publish

# 패키지 검색
vais search "http client"
```

### 레지스트리 구조

```
registry.vais.dev/
├── api/
│   ├── packages/           # 패키지 메타데이터
│   ├── versions/           # 버전 정보
│   └── downloads/          # 다운로드 통계
├── storage/
│   └── packages/           # 실제 패키지 파일
└── index/
    └── search/             # 검색 인덱스
```

---

## FFI (Foreign Function Interface)

### 설계 목표

1. **기존 생태계 활용** - Python, Rust, C 라이브러리 호출
2. **양방향 통신** - Vais에서 호출 & 외부에서 Vais 호출
3. **타입 안전성** - 자동 타입 변환 및 검증
4. **최소 오버헤드** - 효율적인 데이터 전송

### C FFI

```vais
# C 함수 선언
ffi c {
    # libc
    fn malloc(size: usize) -> *void
    fn free(ptr: *void)

    # 커스텀 라이브러리
    @link("mylib")
    fn my_function(a: i32, b: i32) -> i32
}

# 사용
result = c.my_function(10, 20)
```

### Rust FFI

```vais
# Rust crate 통합
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
# Python 라이브러리 통합
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

### 타입 매핑

```
Vais 타입    <->    C 타입       <->    Rust 타입    <->    Python 타입
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

### 메모리 안전성

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
# 스코프 끝에서 자동 해제

# manual은 명시적 해제 필요
ptr = c.raw_alloc(1024)
defer c.free(ptr)  # 명시적 해제
```

---

## 표준 라이브러리 구조

### 코어 모듈 (std.*)

```
std/
├── core/              # 언어 기본 (자동 임포트)
│   ├── types.vais     # 기본 타입 정의
│   ├── ops.vais       # 연산자 트레이트
│   └── prelude.vais   # 기본 함수
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
│   ├── regex.vais     # 정규 표현식
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
│   ├── set.vais       # 셋
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
│   └── bench.vais     # 벤치마킹
│
└── sys/               # 시스템
    ├── env.vais       # 환경 변수
    ├── process.vais   # 프로세스
    └── os.vais        # OS 정보
```

### 사용 예제

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

## 확장 포인트

### 1. 커스텀 연산자

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

### 2. 커스텀 구문 (매크로)

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

### 3. 트레이트를 이용한 커스텀 타입

```vais
# 트레이트 정의
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

### 4. 컴파일러 플러그인 (향후)

```toml
# vais.toml
[plugins]
lint = "vais-clippy"           # 코드 린트
optimize = "vais-optimize"     # 추가 최적화
codegen = "vais-native"        # 네이티브 컴파일
```

---

## 거버넌스 모델

### 프로젝트 구조

```
Vais 조직
├── 코어 팀              # 핵심 언어 개발
│   ├── 언어 설계        # 구문, 타입 시스템
│   ├── 런타임           # VM, GC
│   └── 도구             # CLI, LSP
│
├── 라이브러리 팀        # 표준 라이브러리
│   ├── std.io
│   ├── std.net
│   └── ...
│
├── 커뮤니티 팀          # 커뮤니티 관리
│   ├── 문서화
│   ├── 교육
│   └── 이벤트
│
└── 패키지 레지스트리    # 패키지 레지스트리 운영
```

### RFC 프로세스 (Request for Comments)

```
1. 아이디어        → GitHub Discussion
2. Pre-RFC        → 초기 논의
3. RFC            → 공식 제안
4. 리뷰           → 커뮤니티 리뷰
5. FCP            → 최종 코멘트 기간
6. 승인           → 구현 시작
7. 구현 완료      → 릴리스
```

### 버전 관리

```
Vais 버전 체계:
- Major: 브레이킹 체인지 (드물게)
- Minor: 새 기능 (하위 호환)
- Patch: 버그 수정

에디션 시스템 (Rust 참고):
- Edition 2026: 초기 버전
- Edition 2027: 개선 버전
- 이전 에디션 코드는 계속 동작
```

---

## 커뮤니티 패키지 예시

### 웹 프레임워크 (vais-web)

```vais
use web.{App, route, get, post}

app = App.new()

@get("/")
index(req) = "Hello, Vais!"

@get("/users/:id")
get_user(req) = {
    id = req.params.id
    User.find(id)?
}

@post("/users")
create_user(req) = {
    data = req.json()?
    User.create(data)?
}

app.run(port=8080)
```

### 데이터 사이언스 (vais-data)

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
    .title("아시아 지역 제품별 매출")
    .save("chart.png")
```

### 머신러닝 (vais-ml)

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

## 구현 우선순위

### Phase 1: 코어
```
[x] 언어 스펙 (v6b)
[ ] Lexer
[ ] Parser
[ ] AST
[ ] 타입 시스템
[ ] VM
[ ] 기본 CLI
```

### Phase 2: 기반
```
[ ] 모듈 시스템
[ ] 패키지 매니저 (기본)
[ ] std.core
[ ] std.io
[ ] std.collections
```

### Phase 3: FFI
```
[ ] C FFI
[ ] Rust FFI
[ ] Python FFI
```

### Phase 4: 생태계
```
[ ] 패키지 레지스트리
[ ] 문서 사이트
[ ] std.net
[ ] std.data.json
[ ] std.async
```

### Phase 5: 커뮤니티
```
[ ] 커뮤니티 가이드라인
[ ] RFC 프로세스
[ ] 첫 커뮤니티 패키지
[ ] 튜토리얼 & 예제
```

---

## 성공 지표

| 지표 | 목표 (1년) | 목표 (3년) |
|------|-----------|-----------|
| GitHub Stars | 1,000+ | 10,000+ |
| 패키지 수 | 50+ | 500+ |
| 기여자 수 | 20+ | 100+ |
| 프로덕션 사용자 | 10+ | 1,000+ |
| 문서 페이지 | 100+ | 500+ |

---

## 요약

Vais 성공 공식:

```
작은 코어 (안정적, 변경 어려움)
    +
강력한 FFI (기존 생태계 활용)
    +
쉬운 패키지 시스템 (누구나 확장 가능)
    +
좋은 문서화 (진입 장벽 낮춤)
    +
커뮤니티 거버넌스 (함께 성장)
    =
지속 가능한 생태계
```

**Python이 30년에 걸쳐 이룬 것을, AI 시대에는 더 빠르게 달성할 수 있습니다.**
