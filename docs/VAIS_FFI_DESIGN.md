# Vais FFI (Foreign Function Interface) Design

**Version:** 1.0.0
**Date:** 2026-01-12

---

## Overview

FFI는 Vais의 **핵심 경쟁력**입니다.

기존 생태계(Python, Rust, C)를 활용할 수 있어야 Vais이 실용적인 언어가 됩니다.

```
┌─────────────────────────────────────────────────────────────┐
│                      Vais Code                               │
├─────────────────────────────────────────────────────────────┤
│                      FFI Layer                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  C ABI   │  │   Rust   │  │  Python  │  │   WASM   │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
├─────────────────────────────────────────────────────────────┤
│                   Native Libraries                           │
│  libc, OpenSSL, SQLite, NumPy, Pandas, tokio, ...          │
└─────────────────────────────────────────────────────────────┘
```

---

## Design Goals

| Goal | Description | Priority |
|------|-------------|----------|
| **Safety** | 메모리 안전성 보장 | 최상 |
| **Ergonomic** | 쉬운 사용법 | 상 |
| **Performance** | 최소 오버헤드 | 상 |
| **Bidirectional** | Vais ↔ 외부 양방향 | 중 |
| **Auto-binding** | 자동 바인딩 생성 | 중 |

---

## Type Mapping

### Universal Type Correspondence

```
Vais          C              Rust           Python
────────────────────────────────────────────────────
i             int64_t        i64            int
i32           int32_t        i32            int
i64           int64_t        i64            int
i8            int8_t         i8             int
u8            uint8_t        u8             int
f             double         f64            float
f32           float          f32            float
b             bool           bool           bool
s             char*          String         str
[u8]          uint8_t*       Vec<u8>        bytes
[T]           T*+len         Vec<T>         list
{s:V}         -              HashMap        dict
?T            T* (nullable)  Option<T>      Optional
!T            -              Result<T,E>    raise/return
*T            T*             *mut T         ctypes.pointer
fn(A)->B      fn_ptr         fn(A)->B       Callable
```

---

## C FFI

### Basic Declaration

```vais
ffi c {
    # 단순 함수
    fn abs(x: i32) -> i32
    fn strlen(s: *i8) -> usize

    # 라이브러리 링크
    @link("m")  # libm
    fn sin(x: f) -> f
    fn cos(x: f) -> f
    fn sqrt(x: f) -> f

    # 커스텀 라이브러리
    @link("mylib")
    @header("mylib.h")
    fn my_function(a: i32, b: i32) -> i32
}
```

### Struct Mapping

```vais
ffi c {
    # C struct 매핑
    @repr(C)
    type Point = {
        x: f64,
        y: f64
    }

    @repr(C)
    type Rect = {
        origin: Point,
        size: Point
    }

    fn create_rect(x: f64, y: f64, w: f64, h: f64) -> Rect
    fn rect_area(r: *Rect) -> f64
}
```

### Memory Management

```vais
ffi c {
    # 메모리 할당
    fn malloc(size: usize) -> *void
    fn free(ptr: *void)
    fn realloc(ptr: *void, size: usize) -> *void

    # Vais 관리 (자동 해제)
    @managed
    fn create_buffer(size: usize) -> *Buffer

    # 수동 관리 (명시적 해제 필요)
    @manual
    fn raw_alloc(size: usize) -> *void
}

# 사용
buf = c.create_buffer(1024)
# scope 끝나면 자동 해제

ptr = c.raw_alloc(1024)
defer c.free(ptr)  # 명시적 해제
```

### Callback

```vais
ffi c {
    # 콜백 타입
    type Comparator = fn(*void, *void) -> i32

    fn qsort(
        base: *void,
        num: usize,
        size: usize,
        cmp: Comparator
    )
}

# Vais에서 콜백 제공
my_compare = (a, b) => {
    va = *(a as *i32)
    vb = *(b as *i32)
    va - vb
}

c.qsort(arr.ptr, arr.len, 4, my_compare)
```

---

## Rust FFI

### Crate Integration

```vais
ffi rust {
    # Crate 선언
    @crate("serde_json", version = "1.0")
    mod json {
        type Value
        fn from_str(s: s) -> !Value
        fn to_string(v: Value) -> !s
    }

    @crate("tokio", version = "1.0", features = ["full"])
    mod tokio {
        fn spawn<F: Future>(f: F) -> JoinHandle
        fn block_on<F: Future>(f: F) -> F::Output
    }

    @crate("reqwest", version = "0.11")
    mod http {
        async fn get(url: s) -> !Response
        type Response {
            fn status(self) -> u16
            async fn text(self) -> !s
            async fn json<T>(self) -> !T
        }
    }
}
```

### Direct Usage

```vais
use ffi.rust.json
use ffi.rust.http

# JSON 파싱
data = json.from_str('{"name": "John", "age": 30}')?
name = data["name"].as_str()?

# HTTP 요청
response = http.get("https://api.example.com/data").await?
body = response.text().await?
```

### Trait Mapping

```vais
ffi rust {
    # Rust trait을 Vais trait으로
    @trait_map(Iterator)
    trait RustIterator<T> {
        fn next(self) -> ?T
    }

    @trait_map(IntoIterator)
    trait IntoRustIterator<T> {
        fn into_iter(self) -> RustIterator<T>
    }
}
```

---

## Python FFI

### Module Import

```vais
ffi python {
    # NumPy
    @module("numpy")
    mod np {
        type ndarray

        fn array(data: [f]) -> ndarray
        fn zeros(shape: (i, i)) -> ndarray
        fn ones(shape: (i, i)) -> ndarray
        fn dot(a: ndarray, b: ndarray) -> ndarray
        fn transpose(a: ndarray) -> ndarray

        # ndarray 메서드
        impl ndarray {
            fn shape(self) -> (i, i)
            fn reshape(self, shape: (i, i)) -> ndarray
            fn sum(self) -> f
            fn mean(self) -> f
        }
    }

    # Pandas
    @module("pandas")
    mod pd {
        type DataFrame
        type Series

        fn DataFrame(data: {s: [any]}) -> DataFrame
        fn read_csv(path: s) -> DataFrame
        fn read_json(path: s) -> DataFrame

        impl DataFrame {
            fn head(self, n: i = 5) -> DataFrame
            fn tail(self, n: i = 5) -> DataFrame
            fn describe(self) -> DataFrame
            fn groupby(self, col: s) -> GroupBy
            fn to_csv(self, path: s)
        }
    }

    # Scikit-learn
    @module("sklearn.linear_model")
    mod sklearn {
        type LinearRegression {
            fn new() -> Self
            fn fit(self, X: ndarray, y: ndarray) -> Self
            fn predict(self, X: ndarray) -> ndarray
            fn score(self, X: ndarray, y: ndarray) -> f
        }
    }
}
```

### Usage Example

```vais
use ffi.python.{np, pd, sklearn}

# 데이터 로드
df = pd.read_csv("data.csv")
print(df.head())

# NumPy 배열로 변환
X = np.array(df["features"].to_list())
y = np.array(df["target"].to_list())

# 모델 학습
model = sklearn.LinearRegression.new()
model.fit(X, y)

# 예측
predictions = model.predict(X)
score = model.score(X, y)
print("R² Score: " + score.str)
```

### Type Conversion

```vais
# 자동 변환
vais_list = [1.0, 2.0, 3.0]
np_array = np.array(vais_list)      # Vais list → numpy array
back = np_array.to_list()           # numpy array → Vais list

vais_dict = {"a": [1,2], "b": [3,4]}
df = pd.DataFrame(vais_dict)        # Vais dict → DataFrame
back = df.to_dict()                 # DataFrame → Vais dict
```

### Error Handling

```vais
# Python 예외를 Vais Result로
result = try {
    df = pd.read_csv("nonexistent.csv")
    ok(df)
} catch (e: PythonError) {
    err("Failed to read CSV: " + e.message)
}

# 또는 ? 연산자
df = pd.read_csv("data.csv")?  # 실패시 에러 전파
```

---

## WASM FFI (Future)

### Import WASM Module

```vais
ffi wasm {
    @module("calculator.wasm")
    mod calc {
        fn add(a: i32, b: i32) -> i32
        fn multiply(a: i32, b: i32) -> i32
    }

    @module("image.wasm")
    mod image {
        fn resize(data: [u8], w: i32, h: i32) -> [u8]
        fn blur(data: [u8], radius: f32) -> [u8]
    }
}
```

### Export Vais to WASM

```vais
# Vais 함수를 WASM으로 컴파일
@wasm_export
pub fn process_data(input: [u8]) -> [u8] = {
    # 처리 로직
}

# 컴파일
# vais build --target wasm32
```

---

## Auto-Binding Generation

### From C Header

```bash
# C 헤더에서 자동 바인딩 생성
vais bindgen c sqlite3.h --output sqlite.vais
```

```vais
# 생성된 파일 (sqlite.vais)
ffi c {
    @link("sqlite3")

    type sqlite3
    type sqlite3_stmt

    fn sqlite3_open(filename: *i8, db: **sqlite3) -> i32
    fn sqlite3_close(db: *sqlite3) -> i32
    fn sqlite3_exec(
        db: *sqlite3,
        sql: *i8,
        callback: fn(*void, i32, **i8, **i8) -> i32,
        arg: *void,
        errmsg: **i8
    ) -> i32
    # ...
}
```

### From Rust Crate

```bash
# Cargo.toml에서 바인딩 생성
vais bindgen rust serde_json --output json.vais
```

### From Python Type Hints

```bash
# Python stub 파일에서 바인딩 생성
vais bindgen python numpy.pyi --output numpy.vais
```

---

## Safety Guarantees

### Memory Safety

```vais
ffi c {
    # 안전하지 않은 함수는 unsafe 블록 필요
    @unsafe
    fn raw_pointer_op(ptr: *void) -> *void
}

# 사용시
unsafe {
    result = c.raw_pointer_op(ptr)
}
```

### Null Safety

```vais
ffi c {
    # nullable 반환값
    fn find(arr: *i32, len: usize, val: i32) -> ?*i32
}

# 사용시 nil 체크 강제
ptr = c.find(arr, len, target)
if ptr != nil {
    value = *ptr
}

# 또는
value = c.find(arr, len, target)? # nil이면 early return
```

### Thread Safety

```vais
ffi c {
    # thread-safe 표시
    @thread_safe
    fn atomic_add(ptr: *i32, val: i32) -> i32

    # not thread-safe (single thread only)
    @single_thread
    fn global_state_modify()
}
```

---

## Performance Optimization

### Zero-Copy

```vais
ffi c {
    # 데이터 복사 없이 포인터 전달
    @zero_copy
    fn process_buffer(data: &[u8], len: usize)
}

# Vais 배열이 직접 전달됨 (복사 없음)
c.process_buffer(my_data, my_data.len)
```

### Batch Calls

```vais
# 여러 FFI 호출을 배치로
ffi.batch {
    a = c.compute1(x)
    b = c.compute2(y)
    c = c.compute3(z)
}
# 세 호출이 최적화되어 실행
```

### Caching

```vais
ffi python {
    # 결과 캐싱
    @cached(ttl = 60)
    fn expensive_computation(x: f) -> f
}
```

---

## Error Handling

### C Errors

```vais
ffi c {
    # errno 기반 에러
    @errno
    fn open(path: *i8, flags: i32) -> !i32

    # 반환값 기반 에러
    @error_code(negative)
    fn write(fd: i32, buf: *void, count: usize) -> !isize
}

# 사용
fd = c.open("/tmp/test", O_RDONLY)?
bytes = c.write(fd, buf, len)?
```

### Rust Errors

```vais
ffi rust {
    # Result 타입 자동 변환
    @crate("std")
    mod fs {
        fn read_to_string(path: s) -> !s  # Result<String, io::Error>
    }
}

content = rust.fs.read_to_string("file.txt")?
```

### Python Exceptions

```vais
ffi python {
    # 예외를 Result로 변환
    @module("json")
    mod json {
        fn loads(s: s) -> !any  # JSONDecodeError → err
    }
}

data = python.json.loads(json_str)?
```

---

## FFI Builder API

라이브러리 개발자를 위한 FFI 래퍼 작성 도구:

```vais
# 고수준 래퍼 생성
mod sqlite

use ffi.c.sqlite3_raw as raw

pub type Database = {
    handle: *raw.sqlite3
}

pub fn open(path: s) -> !Database = {
    mut db: *raw.sqlite3 = nil
    result = raw.sqlite3_open(path.c_str(), &db)
    if result != 0 {
        err("Failed to open database")
    } else {
        ok(Database { handle: db })
    }
}

pub fn close(db: Database) = {
    raw.sqlite3_close(db.handle)
}

pub fn execute(db: Database, sql: s) -> !void = {
    mut errmsg: *i8 = nil
    result = raw.sqlite3_exec(db.handle, sql.c_str(), nil, nil, &errmsg)
    if result != 0 {
        msg = s.from_c_str(errmsg)
        raw.sqlite3_free(errmsg)
        err(msg)
    } else {
        ok(())
    }
}
```

---

## Implementation Plan

### Phase 1: C FFI (Core)
```
- Basic function calls
- Primitive type mapping
- Struct mapping
- Callback support
- Memory management (@managed/@manual)
```

### Phase 2: Rust FFI
```
- Crate linking
- Type mapping
- Trait support
- Async integration
```

### Phase 3: Python FFI
```
- Module import
- Type conversion
- Exception handling
- NumPy/Pandas integration
```

### Phase 4: Tooling
```
- Auto-binding generator
- Performance profiler
- Safety analyzer
```

---

## Summary

FFI 설계 원칙:

| 원칙 | 설명 |
|------|------|
| **Safety First** | 안전하지 않은 코드는 명시적 `unsafe` |
| **Ergonomic** | 고수준 API로 쉽게 사용 |
| **Zero-Cost** | 가능한 오버헤드 최소화 |
| **Bidirectional** | Vais ↔ 외부 양방향 지원 |
| **Auto-Binding** | 바인딩 자동 생성 도구 제공 |

**기존 생태계를 활용할 수 있어야 새 언어가 성공합니다.**
