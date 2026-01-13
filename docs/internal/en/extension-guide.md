# Vais Extension Guide

**Version:** 1.0.0
**Date:** 2026-01-12

**"Anyone can extend Vais"**

---

## Overview

This guide is for **developers who want to contribute** to the Vais ecosystem.

```
Extension Types:
1. Package Development - Create libraries/tools
2. FFI Bindings - Integrate existing libraries
3. Tool Development - IDE, linter, formatter, etc.
4. Core Contribution - Improve the language itself
```

---

## Part 1: Package Development

### 1.1 Creating Your First Package

```bash
# Create project
vais new my-first-lib --lib
cd my-first-lib

# Structure
my-first-lib/
├── vais.toml
├── src/
│   └── lib.vais
└── tests/
    └── test_lib.vais
```

**vais.toml:**
```toml
[package]
name = "my-first-lib"
version = "0.1.0"
edition = "2026"
description = "My first Vais library"
license = "MIT"
```

**src/lib.vais:**
```vais
# Module definition
mod my_first_lib

# Public function
pub greet(name: s) -> s = "Hello, " + name + "!"

# Public type
pub type User = {
    name: s,
    age: i
}

# Public function with type
pub create_user(name: s, age: i) -> User = {
    User { name: name, age: age }
}
```

**tests/test_lib.vais:**
```vais
use std.test
use my_first_lib.{greet, create_user}

#[test]
test_greet() = {
    assert_eq(greet("World"), "Hello, World!")
}

#[test]
test_create_user() = {
    user = create_user("John", 30)
    assert_eq(user.name, "John")
    assert_eq(user.age, 30)
}
```

```bash
# Run tests
vais test

# Build
vais build
```

### 1.2 API Design Best Practices

#### Consistent Naming

```vais
# Good - Actions starting with verb
pub get_user(id: i) -> ?User
pub create_user(data: UserInput) -> !User
pub update_user(id: i, data: UserInput) -> !User
pub delete_user(id: i) -> !void

# Good - Booleans starting with is/has
pub is_valid(data) -> b
pub has_permission(user, action) -> b

# Bad - Unclear names
pub user(id: i)        # get? create? delete?
pub check(data)        # check what?
```

#### Error Handling

```vais
# Use Result type to indicate failure possibility
pub parse_config(path: s) -> !Config = {
    content = fs.read(path)?           # Propagate file error
    config = json.parse(content)?      # Propagate parse error
    validate(config)?                  # Propagate validation error
    ok(config)
}

# Clear error types
pub type ConfigError =
    | FileNotFound(path: s)
    | ParseError(line: i, msg: s)
    | ValidationError(field: s, msg: s)

pub parse_config(path: s) -> Result<Config, ConfigError>
```

#### Using Option

```vais
# Use Option for values that may not exist
pub find_user(id: i) -> ?User

# Provide defaults
pub get_user_or_default(id: i, default: User) -> User = {
    find_user(id) ?? default
}
```

#### Builder Pattern

```vais
pub type RequestBuilder = {
    url: s,
    method: s,
    headers: {s: s},
    body: ?s,
    timeout: ?Duration,
}

pub request(url: s) -> RequestBuilder = {
    RequestBuilder {
        url: url,
        method: "GET",
        headers: {},
        body: nil,
        timeout: nil,
    }
}

impl RequestBuilder {
    pub method(self, m: s) -> Self = { ...self, method: m }
    pub header(self, k: s, v: s) -> Self = {
        ...self,
        headers: { ...self.headers, [k]: v }
    }
    pub body(self, b: s) -> Self = { ...self, body: some(b) }
    pub timeout(self, t: Duration) -> Self = { ...self, timeout: some(t) }
    pub send(self) -> !Response = { ... }
}

# Usage
response = request("https://api.example.com")
    .method("POST")
    .header("Content-Type", "application/json")
    .body('{"name": "John"}')
    .timeout(30.seconds)
    .send()?
```

### 1.3 Documentation

```vais
/// Creates a user.
///
/// # Arguments
/// * `name` - User's name
/// * `age` - User's age (must be >= 0)
///
/// # Returns
/// The created user object
///
/// # Errors
/// * `ValidationError` - If age is negative
///
/// # Examples
/// ```
/// user = create_user("John", 30)?
/// assert_eq(user.name, "John")
/// ```
pub create_user(name: s, age: i) -> !User = {
    if age < 0 {
        err(ValidationError("age", "must be non-negative"))
    } else {
        ok(User { name, age })
    }
}
```

### 1.4 Publishing

```bash
# 1. Write README.md
# 2. Add LICENSE file
# 3. Ensure tests pass
vais test

# 4. Ensure lint passes
vais lint

# 5. Dry-run check
vais publish --dry-run

# 6. Publish
vais publish
```

---

## Part 2: FFI Binding Development

### 2.1 C Library Bindings

**Example: SQLite Binding**

```vais
# src/lib.vais
mod sqlite

# Low-level FFI declaration
ffi c {
    @link("sqlite3")

    type sqlite3
    type sqlite3_stmt

    fn sqlite3_open(filename: *i8, db: **sqlite3) -> i32
    fn sqlite3_close(db: *sqlite3) -> i32
    fn sqlite3_exec(
        db: *sqlite3,
        sql: *i8,
        callback: ?fn(*void, i32, **i8, **i8) -> i32,
        arg: *void,
        errmsg: **i8
    ) -> i32
    fn sqlite3_prepare_v2(
        db: *sqlite3,
        sql: *i8,
        nbyte: i32,
        stmt: **sqlite3_stmt,
        tail: **i8
    ) -> i32
    fn sqlite3_step(stmt: *sqlite3_stmt) -> i32
    fn sqlite3_finalize(stmt: *sqlite3_stmt) -> i32
    fn sqlite3_column_text(stmt: *sqlite3_stmt, col: i32) -> *i8
    fn sqlite3_column_int(stmt: *sqlite3_stmt, col: i32) -> i32
    fn sqlite3_free(ptr: *void)
}

# High-level API
pub type Database = {
    handle: *c.sqlite3,
}

pub type Error =
    | OpenError(msg: s)
    | QueryError(msg: s)

pub open(path: s) -> !Database = {
    mut db: *c.sqlite3 = nil
    result = c.sqlite3_open(path.c_str(), &db)
    if result != 0 {
        err(OpenError("Failed to open database: " + path))
    } else {
        ok(Database { handle: db })
    }
}

impl Database {
    pub close(self) = {
        c.sqlite3_close(self.handle)
    }

    pub execute(self, sql: s) -> !void = {
        mut errmsg: *i8 = nil
        result = c.sqlite3_exec(self.handle, sql.c_str(), nil, nil, &errmsg)
        if result != 0 {
            msg = s.from_c_str(errmsg)
            c.sqlite3_free(errmsg)
            err(QueryError(msg))
        } else {
            ok(())
        }
    }

    pub query(self, sql: s) -> ![[s]] = {
        # implementation...
    }
}
```

**Usage:**
```vais
use sqlite

db = sqlite.open("test.db")?
db.execute("CREATE TABLE IF NOT EXISTS users (name TEXT, age INT)")?
db.execute("INSERT INTO users VALUES ('John', 30)")?
rows = db.query("SELECT * FROM users")?
db.close()
```

### 2.2 Python Library Bindings

**Example: NumPy Binding**

```vais
# src/lib.vais
mod numpy

ffi python {
    @module("numpy")
    mod np {
        type ndarray

        # Creation
        fn array(data: [f]) -> ndarray
        fn zeros(shape: (i, i)) -> ndarray
        fn ones(shape: (i, i)) -> ndarray
        fn arange(start: f, stop: f, step: f = 1.0) -> ndarray
        fn linspace(start: f, stop: f, num: i) -> ndarray

        # Operations
        fn add(a: ndarray, b: ndarray) -> ndarray
        fn subtract(a: ndarray, b: ndarray) -> ndarray
        fn multiply(a: ndarray, b: ndarray) -> ndarray
        fn divide(a: ndarray, b: ndarray) -> ndarray
        fn dot(a: ndarray, b: ndarray) -> ndarray
        fn matmul(a: ndarray, b: ndarray) -> ndarray

        # Statistics
        fn sum(a: ndarray) -> f
        fn mean(a: ndarray) -> f
        fn std(a: ndarray) -> f
        fn min(a: ndarray) -> f
        fn max(a: ndarray) -> f

        # Transformation
        fn reshape(a: ndarray, shape: (i, i)) -> ndarray
        fn transpose(a: ndarray) -> ndarray
        fn flatten(a: ndarray) -> ndarray
    }
}

# High-level wrapper
pub type Array = {
    inner: python.np.ndarray,
}

pub array(data: [f]) -> Array = {
    Array { inner: python.np.array(data) }
}

pub zeros(rows: i, cols: i) -> Array = {
    Array { inner: python.np.zeros((rows, cols)) }
}

impl Array {
    pub shape(self) -> (i, i) = self.inner.shape

    pub sum(self) -> f = python.np.sum(self.inner)
    pub mean(self) -> f = python.np.mean(self.inner)

    pub reshape(self, rows: i, cols: i) -> Array = {
        Array { inner: python.np.reshape(self.inner, (rows, cols)) }
    }

    pub T(self) -> Array = {
        Array { inner: python.np.transpose(self.inner) }
    }

    # Operator overloading
    pub op (self) + (other: Array) -> Array = {
        Array { inner: python.np.add(self.inner, other.inner) }
    }

    pub op (self) * (other: Array) -> Array = {
        Array { inner: python.np.multiply(self.inner, other.inner) }
    }

    pub op (self) @ (other: Array) -> Array = {  # Matrix multiplication
        Array { inner: python.np.matmul(self.inner, other.inner) }
    }
}
```

**Usage:**
```vais
use numpy.{array, zeros, Array}

a = array([1.0, 2.0, 3.0])
b = array([4.0, 5.0, 6.0])

c = a + b
println(c.sum())  # 21.0

m1 = zeros(3, 3)
m2 = m1.reshape(9, 1)
```

### 2.3 Binding Best Practices

```
1. Hide low-level FFI internally
   - Users only interact with high-level API

2. Convert errors to Vais style
   - C error codes → Result type
   - Python exceptions → Result type

3. Clear memory management
   - Document who is responsible for freeing
   - Use RAII pattern

4. Automate type conversion
   - Vais types ↔ External types
   - Document conversion costs

5. Thorough testing
   - Memory leak tests
   - Error case tests
```

---

## Part 3: Tool Development

### 3.1 LSP (Language Server Protocol)

Building a Vais LSP server enables support in all editors.

```rust
// LSP server implementation example in Rust
use tower_lsp::{LspService, Server};
use tower_lsp::lsp_types::*;

#[derive(Debug)]
struct VaisLanguageServer {
    // ...
}

#[tower_lsp::async_trait]
impl LanguageServer for VaisLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                // ...
            },
            ..Default::default()
        })
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Implement auto-completion
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // Implement hover info
    }
}
```

### 3.2 Linter Plugins

```vais
# Custom lint rule definition
mod my_lints

use std.lint.{Rule, Diagnostic, Severity}

pub struct NoTodoComments

impl Rule for NoTodoComments {
    fn name(self) -> s = "no-todo-comments"

    fn check(self, ast: AST) -> [Diagnostic] = {
        mut diagnostics = []

        for comment in ast.comments {
            if comment.text.has("TODO") || comment.text.has("FIXME") {
                diagnostics.push(Diagnostic {
                    severity: Severity.Warning,
                    message: "TODO/FIXME comment found",
                    span: comment.span,
                    suggestion: nil,
                })
            }
        }

        diagnostics
    }
}
```

### 3.3 Code Formatter

```vais
# Formatter config
# .vais-fmt.toml
[format]
indent = 4
max_line_length = 100
trailing_comma = true
single_quotes = false
```

---

## Part 4: Core Contribution

### 4.1 RFC Process

Procedure for proposing new features:

```
1. Discuss idea on GitHub Discussion
2. Write RFC document (use template)
3. Submit RFC as PR
4. Community review (minimum 2 weeks)
5. Core team decision
6. Implementation upon approval
```

**RFC Template:**

```markdown
# RFC: [Title]

## Summary
One paragraph summary

## Motivation
Why is this feature needed?

## Detailed Design
Concrete design details

## Alternatives
Other approaches considered

## Unresolved Questions
Parts not yet decided
```

### 4.2 Code Contribution

```bash
# 1. Fork & Clone
git clone https://github.com/YOUR_USERNAME/vais
cd vais

# 2. Create branch
git checkout -b feature/my-feature

# 3. Develop
# ...

# 4. Test
cargo test

# 5. Lint
cargo clippy

# 6. Commit
git commit -m "Add: my feature description"

# 7. Push & PR
git push origin feature/my-feature
# Create PR on GitHub
```

### 4.3 Coding Style

```rust
// Rust code style (for core development)

// Good
fn process_tokens(tokens: &[Token]) -> Result<AST, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// Good - Clear error handling
fn read_file(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

// Good - Documentation
/// Parses the given source code into an AST.
///
/// # Arguments
/// * `source` - The source code to parse
///
/// # Returns
/// * `Ok(AST)` - Successfully parsed AST
/// * `Err(ParseError)` - Parse error with location
pub fn parse(source: &str) -> Result<AST, ParseError> {
    // ...
}
```

---

## Part 5: Community Guidelines

### 5.1 Code of Conduct

```
1. Respect: Respect all contributors
2. Inclusion: Welcome people from diverse backgrounds
3. Collaboration: Build better things together
4. Constructive: Keep criticism constructive
5. Patience: Be patient with new contributors
```

### 5.2 Getting Help

```
- GitHub Issues: Bug reports, feature requests
- GitHub Discussions: Questions, idea discussions
- Discord: Real-time chat
- Stack Overflow: Questions with vais tag
```

### 5.3 Recognizing Contributions

```
All contributors are recorded in CONTRIBUTORS.md.

Contribution types:
- Code contributions
- Documentation writing
- Bug reports
- Reviews
- Community help
```

---

## Quick Reference

### Creating a Package
```bash
vais new my-lib --lib
cd my-lib
# Write code
vais test
vais publish
```

### FFI Bindings
```vais
ffi c { @link("lib") fn func() -> i32 }
ffi python { @module("numpy") mod np { ... } }
ffi rust { @crate("tokio") mod async { ... } }
```

### Core Contribution
```bash
git clone https://github.com/vais-lang/vais
git checkout -b feature/my-feature
# Develop & test
# Create PR
```

---

## Conclusion

The Vais ecosystem grows through **your contributions**.

```
Even small contributions are welcome:
- Typo fixes
- Documentation improvements
- Bug reports
- Answering questions
- Package development
```

**Let's build the language of the AI era together!**
