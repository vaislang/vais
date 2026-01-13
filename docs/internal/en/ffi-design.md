# Vais FFI (Foreign Function Interface) Design

**Version:** 1.0.0
**Date:** 2026-01-12

---

## Overview

FFI is Vais's **core competitive advantage**.

Vais must be able to leverage existing ecosystems (Python, Rust, C) to be a practical language.

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
| **Safety** | Guarantee memory safety | Highest |
| **Ergonomic** | Easy to use | High |
| **Performance** | Minimal overhead | High |
| **Bidirectional** | Vais ↔ External both ways | Medium |
| **Auto-binding** | Automatic binding generation | Medium |

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
    # Simple functions
    fn abs(x: i32) -> i32
    fn strlen(s: *i8) -> usize

    # Library linking
    @link("m")  # libm
    fn sin(x: f) -> f
    fn cos(x: f) -> f
    fn sqrt(x: f) -> f

    # Custom library
    @link("mylib")
    @header("mylib.h")
    fn my_function(a: i32, b: i32) -> i32
}
```

### Struct Mapping

```vais
ffi c {
    # C struct mapping
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
    # Memory allocation
    fn malloc(size: usize) -> *void
    fn free(ptr: *void)
    fn realloc(ptr: *void, size: usize) -> *void

    # Vais managed (auto-release)
    @managed
    fn create_buffer(size: usize) -> *Buffer

    # Manual management (explicit release required)
    @manual
    fn raw_alloc(size: usize) -> *void
}

# Usage
buf = c.create_buffer(1024)
# auto-released when scope ends

ptr = c.raw_alloc(1024)
defer c.free(ptr)  # explicit release
```

### Callback

```vais
ffi c {
    # Callback type
    type Comparator = fn(*void, *void) -> i32

    fn qsort(
        base: *void,
        num: usize,
        size: usize,
        cmp: Comparator
    )
}

# Provide callback from Vais
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
    # Crate declaration
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

# JSON parsing
data = json.from_str('{"name": "John", "age": 30}')?
name = data["name"].as_str()?

# HTTP request
response = http.get("https://api.example.com/data").await?
body = response.text().await?
```

### Trait Mapping

```vais
ffi rust {
    # Rust trait to Vais trait
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

        # ndarray methods
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

# Load data
df = pd.read_csv("data.csv")
print(df.head())

# Convert to NumPy array
X = np.array(df["features"].to_list())
y = np.array(df["target"].to_list())

# Train model
model = sklearn.LinearRegression.new()
model.fit(X, y)

# Predict
predictions = model.predict(X)
score = model.score(X, y)
print("R² Score: " + score.str)
```

### Type Conversion

```vais
# Automatic conversion
vais_list = [1.0, 2.0, 3.0]
np_array = np.array(vais_list)      # Vais list → numpy array
back = np_array.to_list()           # numpy array → Vais list

vais_dict = {"a": [1,2], "b": [3,4]}
df = pd.DataFrame(vais_dict)        # Vais dict → DataFrame
back = df.to_dict()                 # DataFrame → Vais dict
```

### Error Handling

```vais
# Python exception to Vais Result
result = try {
    df = pd.read_csv("nonexistent.csv")
    ok(df)
} catch (e: PythonError) {
    err("Failed to read CSV: " + e.message)
}

# Or use ? operator
df = pd.read_csv("data.csv")?  # propagate error on failure
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
# Compile Vais function to WASM
@wasm_export
pub process_data(input: [u8]) -> [u8] = {
    # processing logic
}

# Compile
# vais build --target wasm32
```

---

## Auto-Binding Generation

### From C Header

```bash
# Generate auto-binding from C header
vais bindgen c sqlite3.h --output sqlite.vais
```

```vais
# Generated file (sqlite.vais)
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
# Generate binding from Cargo.toml
vais bindgen rust serde_json --output json.vais
```

### From Python Type Hints

```bash
# Generate binding from Python stub file
vais bindgen python numpy.pyi --output numpy.vais
```

---

## Safety Guarantees

### Memory Safety

```vais
ffi c {
    # Unsafe functions require unsafe block
    @unsafe
    fn raw_pointer_op(ptr: *void) -> *void
}

# Usage
unsafe {
    result = c.raw_pointer_op(ptr)
}
```

### Null Safety

```vais
ffi c {
    # nullable return value
    fn find(arr: *i32, len: usize, val: i32) -> ?*i32
}

# nil check is enforced
ptr = c.find(arr, len, target)
if ptr != nil {
    value = *ptr
}

# Or
value = c.find(arr, len, target)? # early return if nil
```

### Thread Safety

```vais
ffi c {
    # thread-safe marker
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
    # Pass pointer without data copy
    @zero_copy
    fn process_buffer(data: &[u8], len: usize)
}

# Vais array is passed directly (no copy)
c.process_buffer(my_data, my_data.len)
```

### Batch Calls

```vais
# Batch multiple FFI calls
ffi.batch {
    a = c.compute1(x)
    b = c.compute2(y)
    c = c.compute3(z)
}
# Three calls are optimized
```

### Caching

```vais
ffi python {
    # Result caching
    @cached(ttl = 60)
    fn expensive_computation(x: f) -> f
}
```

---

## Error Handling

### C Errors

```vais
ffi c {
    # errno-based error
    @errno
    fn open(path: *i8, flags: i32) -> !i32

    # Return value-based error
    @error_code(negative)
    fn write(fd: i32, buf: *void, count: usize) -> !isize
}

# Usage
fd = c.open("/tmp/test", O_RDONLY)?
bytes = c.write(fd, buf, len)?
```

### Rust Errors

```vais
ffi rust {
    # Result type auto-conversion
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
    # Convert exception to Result
    @module("json")
    mod json {
        fn loads(s: s) -> !any  # JSONDecodeError → err
    }
}

data = python.json.loads(json_str)?
```

---

## FFI Builder API

FFI wrapper authoring tool for library developers:

```vais
# Generate high-level wrapper
mod sqlite

use ffi.c.sqlite3_raw as raw

pub type Database = {
    handle: *raw.sqlite3
}

pub open(path: s) -> !Database = {
    mut db: *raw.sqlite3 = nil
    result = raw.sqlite3_open(path.c_str(), &db)
    if result != 0 {
        err("Failed to open database")
    } else {
        ok(Database { handle: db })
    }
}

pub close(db: Database) = {
    raw.sqlite3_close(db.handle)
}

pub execute(db: Database, sql: s) -> !void = {
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

FFI design principles:

| Principle | Description |
|-----------|-------------|
| **Safety First** | Unsafe code requires explicit `unsafe` |
| **Ergonomic** | Easy to use with high-level API |
| **Zero-Cost** | Minimize overhead where possible |
| **Bidirectional** | Support Vais ↔ External both ways |
| **Auto-Binding** | Provide auto-binding generation tools |

**New languages succeed when they can leverage existing ecosystems.**
