# AOEL Extension Guide

**Version:** 1.0.0
**Date:** 2026-01-12

**"누구나 AOEL을 확장할 수 있습니다"**

---

## Overview

이 가이드는 AOEL 생태계에 기여하고자 하는 **개발자를 위한 문서**입니다.

```
확장 유형:
1. 패키지 개발 - 라이브러리/도구 만들기
2. FFI 바인딩 - 기존 라이브러리 연동
3. 도구 개발 - IDE, 린터, 포매터 등
4. 코어 기여 - 언어 자체 개선
```

---

## Part 1: 패키지 개발

### 1.1 첫 번째 패키지 만들기

```bash
# 프로젝트 생성
aoel new my-first-lib --lib
cd my-first-lib

# 구조
my-first-lib/
├── aoel.toml
├── src/
│   └── lib.aoel
└── tests/
    └── test_lib.aoel
```

**aoel.toml:**
```toml
[package]
name = "my-first-lib"
version = "0.1.0"
edition = "2026"
description = "My first AOEL library"
license = "MIT"
```

**src/lib.aoel:**
```aoel
# 모듈 정의
mod my_first_lib

# Public 함수
pub fn greet(name: s) -> s = "Hello, " + name + "!"

# Public 타입
pub type User = {
    name: s,
    age: i
}

# Public 함수 with 타입
pub fn create_user(name: s, age: i) -> User = {
    User { name: name, age: age }
}
```

**tests/test_lib.aoel:**
```aoel
use std.test
use my_first_lib.{greet, create_user}

#[test]
fn test_greet() {
    assert_eq(greet("World"), "Hello, World!")
}

#[test]
fn test_create_user() {
    user = create_user("John", 30)
    assert_eq(user.name, "John")
    assert_eq(user.age, 30)
}
```

```bash
# 테스트 실행
aoel test

# 빌드
aoel build
```

### 1.2 API 설계 베스트 프랙티스

#### 일관된 네이밍

```aoel
# Good - 동사로 시작하는 액션
pub fn get_user(id: i) -> ?User
pub fn create_user(data: UserInput) -> !User
pub fn update_user(id: i, data: UserInput) -> !User
pub fn delete_user(id: i) -> !void

# Good - is/has로 시작하는 불리언
pub fn is_valid(data) -> b
pub fn has_permission(user, action) -> b

# Bad - 불명확한 이름
pub fn user(id: i)        # get? create? delete?
pub fn check(data)        # 뭘 체크?
```

#### 에러 처리

```aoel
# Result 타입으로 실패 가능성 명시
pub fn parse_config(path: s) -> !Config = {
    content = fs.read(path)?           # 파일 에러 전파
    config = json.parse(content)?      # 파싱 에러 전파
    validate(config)?                  # 검증 에러 전파
    ok(config)
}

# 명확한 에러 타입
pub type ConfigError =
    | FileNotFound(path: s)
    | ParseError(line: i, msg: s)
    | ValidationError(field: s, msg: s)

pub fn parse_config(path: s) -> Result<Config, ConfigError>
```

#### Option 사용

```aoel
# 없을 수 있는 값은 Option으로
pub fn find_user(id: i) -> ?User

# 기본값 제공
pub fn get_user_or_default(id: i, default: User) -> User = {
    find_user(id) ?? default
}
```

#### 빌더 패턴

```aoel
pub type RequestBuilder = {
    url: s,
    method: s,
    headers: {s: s},
    body: ?s,
    timeout: ?Duration,
}

pub fn request(url: s) -> RequestBuilder = {
    RequestBuilder {
        url: url,
        method: "GET",
        headers: {},
        body: nil,
        timeout: nil,
    }
}

impl RequestBuilder {
    pub fn method(self, m: s) -> Self = { ...self, method: m }
    pub fn header(self, k: s, v: s) -> Self = {
        ...self,
        headers: { ...self.headers, [k]: v }
    }
    pub fn body(self, b: s) -> Self = { ...self, body: some(b) }
    pub fn timeout(self, t: Duration) -> Self = { ...self, timeout: some(t) }
    pub fn send(self) -> !Response = { ... }
}

# 사용
response = request("https://api.example.com")
    .method("POST")
    .header("Content-Type", "application/json")
    .body('{"name": "John"}')
    .timeout(30.seconds)
    .send()?
```

### 1.3 문서화

```aoel
/// 사용자를 생성합니다.
///
/// # Arguments
/// * `name` - 사용자 이름
/// * `age` - 사용자 나이 (0 이상)
///
/// # Returns
/// 생성된 사용자 객체
///
/// # Errors
/// * `ValidationError` - 나이가 음수인 경우
///
/// # Examples
/// ```
/// user = create_user("John", 30)?
/// assert_eq(user.name, "John")
/// ```
pub fn create_user(name: s, age: i) -> !User = {
    if age < 0 {
        err(ValidationError("age", "must be non-negative"))
    } else {
        ok(User { name, age })
    }
}
```

### 1.4 퍼블리싱

```bash
# 1. README.md 작성
# 2. LICENSE 파일 추가
# 3. 테스트 통과 확인
aoel test

# 4. 린트 통과 확인
aoel lint

# 5. dry-run으로 확인
aoel publish --dry-run

# 6. 퍼블리시
aoel publish
```

---

## Part 2: FFI 바인딩 개발

### 2.1 C 라이브러리 바인딩

**예: SQLite 바인딩**

```aoel
# src/lib.aoel
mod sqlite

# Low-level FFI 선언
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

pub fn open(path: s) -> !Database = {
    mut db: *c.sqlite3 = nil
    result = c.sqlite3_open(path.c_str(), &db)
    if result != 0 {
        err(OpenError("Failed to open database: " + path))
    } else {
        ok(Database { handle: db })
    }
}

impl Database {
    pub fn close(self) = {
        c.sqlite3_close(self.handle)
    }

    pub fn execute(self, sql: s) -> !void = {
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

    pub fn query(self, sql: s) -> ![[s]] = {
        # 구현...
    }
}
```

**사용:**
```aoel
use sqlite

db = sqlite.open("test.db")?
db.execute("CREATE TABLE IF NOT EXISTS users (name TEXT, age INT)")?
db.execute("INSERT INTO users VALUES ('John', 30)")?
rows = db.query("SELECT * FROM users")?
db.close()
```

### 2.2 Python 라이브러리 바인딩

**예: NumPy 바인딩**

```aoel
# src/lib.aoel
mod numpy

ffi python {
    @module("numpy")
    mod np {
        type ndarray

        # 생성
        fn array(data: [f]) -> ndarray
        fn zeros(shape: (i, i)) -> ndarray
        fn ones(shape: (i, i)) -> ndarray
        fn arange(start: f, stop: f, step: f = 1.0) -> ndarray
        fn linspace(start: f, stop: f, num: i) -> ndarray

        # 연산
        fn add(a: ndarray, b: ndarray) -> ndarray
        fn subtract(a: ndarray, b: ndarray) -> ndarray
        fn multiply(a: ndarray, b: ndarray) -> ndarray
        fn divide(a: ndarray, b: ndarray) -> ndarray
        fn dot(a: ndarray, b: ndarray) -> ndarray
        fn matmul(a: ndarray, b: ndarray) -> ndarray

        # 통계
        fn sum(a: ndarray) -> f
        fn mean(a: ndarray) -> f
        fn std(a: ndarray) -> f
        fn min(a: ndarray) -> f
        fn max(a: ndarray) -> f

        # 변형
        fn reshape(a: ndarray, shape: (i, i)) -> ndarray
        fn transpose(a: ndarray) -> ndarray
        fn flatten(a: ndarray) -> ndarray
    }
}

# High-level wrapper
pub type Array = {
    inner: python.np.ndarray,
}

pub fn array(data: [f]) -> Array = {
    Array { inner: python.np.array(data) }
}

pub fn zeros(rows: i, cols: i) -> Array = {
    Array { inner: python.np.zeros((rows, cols)) }
}

impl Array {
    pub fn shape(self) -> (i, i) = self.inner.shape

    pub fn sum(self) -> f = python.np.sum(self.inner)
    pub fn mean(self) -> f = python.np.mean(self.inner)

    pub fn reshape(self, rows: i, cols: i) -> Array = {
        Array { inner: python.np.reshape(self.inner, (rows, cols)) }
    }

    pub fn T(self) -> Array = {
        Array { inner: python.np.transpose(self.inner) }
    }

    # 연산자 오버로딩
    pub op (self) + (other: Array) -> Array = {
        Array { inner: python.np.add(self.inner, other.inner) }
    }

    pub op (self) * (other: Array) -> Array = {
        Array { inner: python.np.multiply(self.inner, other.inner) }
    }

    pub op (self) @ (other: Array) -> Array = {  # 행렬곱
        Array { inner: python.np.matmul(self.inner, other.inner) }
    }
}
```

**사용:**
```aoel
use numpy.{array, zeros, Array}

a = array([1.0, 2.0, 3.0])
b = array([4.0, 5.0, 6.0])

c = a + b
println(c.sum())  # 21.0

m1 = zeros(3, 3)
m2 = m1.reshape(9, 1)
```

### 2.3 바인딩 베스트 프랙티스

```
1. Low-level FFI는 내부에 숨기기
   - 사용자는 High-level API만 사용

2. 에러 처리를 AOEL 스타일로
   - C 에러 코드 → Result 타입
   - Python 예외 → Result 타입

3. 메모리 관리 명확히
   - 누가 해제하는지 문서화
   - RAII 패턴 사용

4. 타입 변환 자동화
   - AOEL 타입 ↔ 외부 타입
   - 변환 비용 문서화

5. 테스트 철저히
   - 메모리 누수 테스트
   - 에러 케이스 테스트
```

---

## Part 3: 도구 개발

### 3.1 LSP (Language Server Protocol)

AOEL LSP 서버를 만들면 모든 에디터에서 지원 가능합니다.

```rust
// Rust로 LSP 서버 구현 예시
use tower_lsp::{LspService, Server};
use tower_lsp::lsp_types::*;

#[derive(Debug)]
struct AoelLanguageServer {
    // ...
}

#[tower_lsp::async_trait]
impl LanguageServer for AoelLanguageServer {
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
        // 자동완성 구현
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // 호버 정보 구현
    }
}
```

### 3.2 린터 플러그인

```aoel
# 커스텀 린트 규칙 정의
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

### 3.3 코드 포매터

```aoel
# 포매터 설정
# .aoel-fmt.toml
[format]
indent = 4
max_line_length = 100
trailing_comma = true
single_quotes = false
```

---

## Part 4: 코어 기여

### 4.1 RFC 프로세스

새 기능 제안 절차:

```
1. GitHub Discussion에서 아이디어 논의
2. RFC 문서 작성 (템플릿 사용)
3. PR로 RFC 제출
4. 커뮤니티 리뷰 (최소 2주)
5. 코어 팀 결정
6. 승인 시 구현
```

**RFC 템플릿:**

```markdown
# RFC: [제목]

## 요약
한 문단으로 요약

## 동기
왜 이 기능이 필요한가?

## 상세 설계
구체적인 설계

## 대안
고려한 다른 방법들

## 미해결 질문
결정되지 않은 부분들
```

### 4.2 코드 기여

```bash
# 1. Fork & Clone
git clone https://github.com/YOUR_USERNAME/aoel
cd aoel

# 2. 브랜치 생성
git checkout -b feature/my-feature

# 3. 개발
# ...

# 4. 테스트
cargo test

# 5. 린트
cargo clippy

# 6. 커밋
git commit -m "Add: my feature description"

# 7. Push & PR
git push origin feature/my-feature
# GitHub에서 PR 생성
```

### 4.3 코딩 스타일

```rust
// Rust 코드 스타일 (코어 개발용)

// Good
fn process_tokens(tokens: &[Token]) -> Result<AST, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// Good - 에러 처리 명확히
fn read_file(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

// Good - 문서화
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

## Part 5: 커뮤니티 가이드라인

### 5.1 행동 강령

```
1. 존중: 모든 기여자를 존중합니다
2. 포용: 다양한 배경의 사람들을 환영합니다
3. 협력: 함께 더 좋은 것을 만듭니다
4. 건설적: 비판은 건설적으로 합니다
5. 인내: 새로운 기여자에게 인내심을 갖습니다
```

### 5.2 도움 받기

```
- GitHub Issues: 버그 리포트, 기능 요청
- GitHub Discussions: 질문, 아이디어 논의
- Discord: 실시간 채팅
- Stack Overflow: aoel 태그로 질문
```

### 5.3 기여 인정

```
모든 기여자는 CONTRIBUTORS.md에 기록됩니다.

기여 유형:
- 코드 기여
- 문서 작성
- 버그 리포트
- 리뷰
- 커뮤니티 도움
```

---

## Quick Reference

### 패키지 만들기
```bash
aoel new my-lib --lib
cd my-lib
# 코드 작성
aoel test
aoel publish
```

### FFI 바인딩
```aoel
ffi c { @link("lib") fn func() -> i32 }
ffi python { @module("numpy") mod np { ... } }
ffi rust { @crate("tokio") mod async { ... } }
```

### 코어 기여
```bash
git clone https://github.com/aoel-lang/aoel
git checkout -b feature/my-feature
# 개발 & 테스트
# PR 생성
```

---

## 결론

AOEL 생태계는 **여러분의 기여**로 성장합니다.

```
작은 기여도 환영합니다:
- 오타 수정
- 문서 개선
- 버그 리포트
- 질문 답변
- 패키지 개발
```

**함께 AI 시대의 새로운 언어를 만들어갑시다!**
