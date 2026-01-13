# Vais Architecture: Extensibility-First Design

**Version:** 1.0.0
**Date:** 2026-01-12
**Status:** Draft

---

## Vision

**"The Python of the AI Era"**

Just as Python succeeded with human-friendly syntax,
Vais succeeds with **AI-friendly syntax + extensible ecosystem**.

```
Vais = Small Core + Powerful Extensibility + Community Ecosystem
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

**Principles:**
- Keep the core minimal (hard to change)
- Implement most features as libraries
- Enable language extension without core changes

### 2. Everything is a Package

```vais
# Language features can also be provided as packages
use std.async      # async/await support
use std.macro      # macro system
use std.typing     # advanced type features
```

### 3. Zero-Cost Abstraction

```vais
# Unused features have zero cost
# Only imported items are optimized
```

### 4. FFI First-Class

```vais
# Leverage existing ecosystems
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

pub sin(x) = ...
pub cos(x) = ...

# Private helper (not exported)
taylor_series(x, n) = ...
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

### Package Registry (vais Package Manager)

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

1. **Leverage existing ecosystems** - Call Python, Rust, C libraries
2. **Bidirectional communication** - Call from Vais & call Vais from outside
3. **Type safety** - Automatic type conversion and validation
4. **Minimal overhead** - Efficient data transfer

### C FFI

```vais
# C function declaration
ffi c {
    # libc
    fn malloc(size: usize) -> *void
    fn free(ptr: *void)

    # Custom library
    @link("mylib")
    fn my_function(a: i32, b: i32) -> i32
}

# Usage
result = c.my_function(10, 20)
```

### Rust FFI

```vais
# Rust crate integration
ffi rust {
    @crate("tokio", version = "1.0")
    mod async_runtime {
        fn spawn(future: Future) -> JoinHandle
        fn block_on(future: Future) -> T
    }
}

# Usage
handle = rust.async_runtime.spawn(my_async_fn())
```

### Python FFI

```vais
# Python library integration
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

# Usage
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
# Automatic memory management
ffi c {
    @managed  # Managed by Vais GC
    fn create_buffer(size: i) -> *Buffer

    @manual   # Manual release required
    fn raw_alloc(size: i) -> *void
}

# managed is auto-released
buf = c.create_buffer(1024)
# auto-released when scope ends

# manual requires explicit release
ptr = c.raw_alloc(1024)
defer c.free(ptr)  # explicit release
```

---

## Standard Library Structure

### Core Modules (std.*)

```
std/
├── core/              # Language basics (auto-imported)
│   ├── types.vais     # Basic type definitions
│   ├── ops.vais       # Operator traits
│   └── prelude.vais   # Basic functions
│
├── io/                # Input/Output
│   ├── file.vais      # File read/write
│   ├── stdin.vais     # Standard input
│   ├── stdout.vais    # Standard output
│   └── path.vais      # Path handling
│
├── net/               # Networking
│   ├── http.vais      # HTTP client/server
│   ├── tcp.vais       # TCP socket
│   ├── udp.vais       # UDP socket
│   └── url.vais       # URL parsing
│
├── data/              # Data formats
│   ├── json.vais      # JSON
│   ├── csv.vais       # CSV
│   ├── toml.vais      # TOML
│   └── xml.vais       # XML
│
├── text/              # Text processing
│   ├── regex.vais     # Regular expressions
│   ├── fmt.vais       # Formatting
│   └── encoding.vais  # Encoding
│
├── time/              # Time
│   ├── datetime.vais  # Date/Time
│   ├── duration.vais  # Duration
│   └── timezone.vais  # Timezone
│
├── math/              # Mathematics
│   ├── basic.vais     # Basic math
│   ├── random.vais    # Random numbers
│   └── stats.vais     # Statistics
│
├── collections/       # Data structures
│   ├── list.vais      # List extensions
│   ├── set.vais       # Set
│   ├── map.vais       # Map extensions
│   ├── queue.vais     # Queue
│   └── heap.vais      # Heap
│
├── async/             # Asynchronous
│   ├── future.vais    # Future/Promise
│   ├── channel.vais   # Channel
│   └── spawn.vais     # Task creation
│
├── test/              # Testing
│   ├── assert.vais    # Assertions
│   ├── mock.vais      # Mocking
│   └── bench.vais     # Benchmarking
│
└── sys/               # System
    ├── env.vais       # Environment variables
    ├── process.vais   # Process
    └── os.vais        # OS info
```

### Usage Examples

```vais
use std.io.{read_file, write_file}
use std.net.http.{get, post}
use std.data.json.{parse, stringify}

# File reading
content = read_file("data.txt")?

# HTTP request
response = get("https://api.example.com/data")?
data = parse(response.body)?

# File writing
write_file("output.json", stringify(data))?
```

---

## Extension Points

### 1. Custom Operators

```vais
# Operator definition (in package)
mod matrix

pub type Matrix = [[f]]

# Matrix multiplication operator
pub op (a: Matrix) ** (b: Matrix) -> Matrix {
    # implementation
}

# Usage
use matrix.{Matrix, **}
result = mat_a ** mat_b
```

### 2. Custom Syntax (Macros)

```vais
# Macro definition
mod html

pub macro html! {
    # HTML DSL
    (<$tag $attrs*>$children*</$tag>) => {
        Element.new($tag, $attrs, $children)
    }
}

# Usage
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
# Trait definition
mod iter

pub trait Iterable<T> {
    fn iter(self) -> Iterator<T>
}

pub trait Mappable<T> {
    fn map<U>(self, f: T -> U) -> Self<U>
}

# Implement for custom type
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
lint = "vais-clippy"           # Code lint
optimize = "vais-optimize"     # Additional optimization
codegen = "vais-native"        # Native compilation
```

---

## Governance Model

### Project Structure

```
Vais Organization
├── Core Team              # Core language development
│   ├── Language Design    # Syntax, type system
│   ├── Runtime            # VM, GC
│   └── Tooling            # CLI, LSP
│
├── Library Team           # Standard library
│   ├── std.io
│   ├── std.net
│   └── ...
│
├── Community Team         # Community management
│   ├── Documentation
│   ├── Education
│   └── Events
│
└── Package Registry       # Package registry operations
```

### RFC Process (Request for Comments)

```
1. Idea          → GitHub Discussion
2. Pre-RFC       → Initial discussion
3. RFC           → Formal proposal
4. Review        → Community review
5. FCP           → Final Comment Period
6. Accepted      → Implementation begins
7. Implemented   → Release
```

### Versioning

```
Vais version scheme:
- Major: Breaking changes (rare)
- Minor: New features (backward compatible)
- Patch: Bug fixes

Edition system (inspired by Rust):
- Edition 2026: Initial version
- Edition 2027: Improved version
- Previous edition code continues to work
```

---

## Community Package Examples

### Web Framework (vais-web)

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

### Data Science (vais-data)

```vais
use data.{DataFrame, Series}
use data.plot

# Load data
df = DataFrame.read_csv("sales.csv")

# Data processing
result = df
    .?(_["region"] == "Asia")
    .@{ _["revenue"] * _["quantity"] }
    .groupby("product")
    .sum()

# Visualization
plot.bar(result, x="product", y="total")
    .title("Asia Revenue by Product")
    .save("chart.png")
```

### Machine Learning (vais-ml)

```vais
use ml.{Model, train, predict}
use ml.nn.{Dense, Sequential}

# Model definition
model = Sequential([
    Dense(128, activation="relu"),
    Dense(64, activation="relu"),
    Dense(10, activation="softmax")
])

# Training
model.compile(optimizer="adam", loss="cross_entropy")
model.fit(x_train, y_train, epochs=10)

# Prediction
predictions = model.predict(x_test)
```

---

## Implementation Priority

### Phase 1: Core
```
[x] Language Spec (v6b)
[ ] Lexer
[ ] Parser
[ ] AST
[ ] Type System
[ ] VM
[ ] Basic CLI
```

### Phase 2: Foundation
```
[ ] Module System
[ ] Package Manager (basic)
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

### Phase 4: Ecosystem
```
[ ] Package Registry
[ ] Documentation Site
[ ] std.net
[ ] std.data.json
[ ] std.async
```

### Phase 5: Community
```
[ ] Community Guidelines
[ ] RFC Process
[ ] First Community Packages
[ ] Tutorials & Examples
```

---

## Success Metrics

| Metric | Target (1 year) | Target (3 years) |
|--------|-----------------|------------------|
| GitHub Stars | 1,000+ | 10,000+ |
| Packages | 50+ | 500+ |
| Contributors | 20+ | 100+ |
| Production Users | 10+ | 1,000+ |
| Documentation Pages | 100+ | 500+ |

---

## Summary

Vais success formula:

```
Small Core (stable, hard to change)
    +
Powerful FFI (leverage existing ecosystems)
    +
Easy Package System (anyone can extend)
    +
Good Documentation (lower entry barrier)
    +
Community Governance (grow together)
    =
Sustainable Ecosystem
```

**What Python took 30 years to achieve, we can accomplish faster in the AI era.**
