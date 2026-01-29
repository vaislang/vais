# vais-security

Static security analysis for Vais programs. This crate analyzes the Abstract Syntax Tree (AST) of Vais code to detect potential security vulnerabilities.

## Features

### Buffer Overflow Detection
- Manual memory operations (`malloc`, `free`, `load_byte`, `store_byte`)
- Array indexing without bounds checking
- Unsafe C functions (`strcpy`, `strcat`, `gets`, `sprintf`)
- Memory operations (`memcpy`, `memmove`, `memset`)

### Pointer Safety
- Raw pointer arithmetic
- Use-after-free vulnerabilities
- Dereferencing potentially invalid pointers

### Injection Vulnerabilities
- Command injection (string concatenation in `system`, `exec`, `popen`)
- SQL injection (string concatenation in database queries)

### Hardcoded Secrets
- API keys and tokens (e.g., `sk-`, `pk-` prefixes)
- Passwords in string literals
- High-entropy strings that may be secrets
- Variable names suggesting sensitive data

### Integer Overflow
- Arithmetic operations on unchecked user input
- Potentially unsafe integer operations

### Error Handling
- Unchecked operations (`unwrap` without proper error handling)

## Usage

```rust
use vais_security::{SecurityAnalyzer, Severity};
use vais_parser::Parser;
use vais_lexer::tokenize;

let source = r#"
    F main() -> i64 {
        ptr := malloc(100)
        store_i64(ptr, 42)
        free(ptr)
        0
    }
"#;

// Parse the code
let tokens = tokenize(source).unwrap();
let mut parser = Parser::new(tokens);
let module = parser.parse_module().unwrap();

// Analyze for security issues
let mut analyzer = SecurityAnalyzer::new();
let findings = analyzer.analyze(&module);

// Filter by severity
for finding in findings {
    if finding.severity >= Severity::High {
        println!("{}", finding);
    }
}
```

## Example

Run the example analyzer:

```bash
cargo run --example analyze
```

## Security Findings

Each finding includes:
- **Severity**: Critical, High, Medium, Low, or Info
- **Category**: Type of security issue
- **Description**: Details about the issue
- **Location**: Source code position (span)
- **Recommendation**: How to fix the issue

## Severity Levels

- **Critical**: Severe vulnerabilities requiring immediate attention (e.g., command injection, hardcoded tokens)
- **High**: Serious security issues (e.g., use-after-free, unsafe pointers, hardcoded passwords)
- **Medium**: Notable security concerns (e.g., integer overflow, memory leaks)
- **Low**: Minor issues (e.g., unchecked error handling)
- **Info**: Best practices and recommendations

## Conservative Analysis

The analyzer is intentionally conservative and may report potential issues even if they're not confirmed vulnerabilities. This "fail-safe" approach helps catch security issues early in development.

## Testing

Run the test suite:

```bash
cargo test -p vais-security
```

The test suite includes 22+ tests covering various security scenarios.
