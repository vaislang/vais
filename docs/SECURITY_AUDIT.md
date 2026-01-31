# Vais Compiler Security Audit Report

**Date:** January 31, 2026
**Auditor:** Security Audit Team
**Version:** 0.1.0
**Scope:** Full security review of Vais compiler codebase

---

## Executive Summary

This security audit was conducted on the Vais programming language compiler, focusing on potential vulnerabilities in code parsing, generation, plugin systems, and server components. The audit examined approximately 95,691 lines of Rust code across 30+ crates.

### Overall Risk Assessment: **MEDIUM**

While the compiler demonstrates good security practices in several areas (path traversal protection, FFI validation, security analyzer crate), there are several areas requiring immediate attention, particularly around:

1. **Resource exhaustion vulnerabilities** (no recursion depth limits in parser)
2. **Plugin system security** (arbitrary code execution by design)
3. **Playground server** (untrusted code execution without sandboxing)
4. **Missing timeout protections** in compilation
5. **Excessive use of `.unwrap()`** (1,337 instances across codebase)

---

## Methodology

The audit employed the following techniques:

1. **Static Code Analysis**: Automated scanning for security patterns
   - Searched for `unsafe` blocks (77 occurrences)
   - Identified `.unwrap()` usage (1,337 instances)
   - Detected `.expect()` calls (255 instances)
   - Found `panic!` usage (86 instances)

2. **Manual Code Review**: In-depth examination of security-critical components
   - Parser input validation
   - Code generation and LLVM IR injection
   - FFI boundary safety
   - Path traversal protection
   - Plugin loading mechanisms
   - Server-side code execution

3. **Architecture Analysis**: Review of overall system security design
   - Trust boundaries
   - Privilege separation
   - Input sanitization

4. **Test Coverage Review**: Examination of existing security tests
   - Import security tests present and comprehensive
   - Memory safety tests exist
   - Fuzz testing infrastructure present

---

## Findings

### CRITICAL Severity

#### C-1: Playground Server - Remote Code Execution Without Sandboxing
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-playground-server/src/main.rs:423-465`

**Description:**
The playground server compiles and executes arbitrary user-submitted Vais code without proper sandboxing. While it does clear environment variables (`env_clear()`) and restricts `PATH`, this is insufficient protection against malicious code.

**Risk:**
- Arbitrary file system access (read/write within process permissions)
- Network access (can establish outbound connections)
- Resource exhaustion (CPU, memory, disk)
- Process spawning
- No timeout on execution (only compilation has timeout via semaphore)

**Evidence:**
```rust
let exec_output = Command::new(bin_path.to_str().unwrap())
    .env_clear()
    .env("PATH", "/usr/bin:/bin")
    .output();
```

**Recommendation:**
- **Immediate:** Implement timeout for execution (using tokio timeout or platform-specific mechanisms)
- **High Priority:** Add proper sandboxing:
  - Linux: Use seccomp-bpf, namespaces, or Landlock LSM
  - macOS: Use sandbox-exec or App Sandbox
  - Consider using WASM compilation target for playground (no syscall access)
- **Additional:** Implement resource limits (CPU time, memory, file descriptors)
- **Network:** Disable network access via firewall rules or namespace isolation

---

#### C-2: Plugin System - Arbitrary Code Execution by Design
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-plugin/src/loader.rs:131-228`

**Description:**
The plugin system loads and executes arbitrary native code from shared libraries (.so, .dylib, .dll). This is by design but creates significant security risks if users load malicious plugins.

**Risk:**
- Complete system compromise if malicious plugin is loaded
- No signature verification on plugins
- No permission model for plugin capabilities
- Plugins run with full compiler privileges

**Evidence:**
```rust
let library = unsafe {
    Library::new(path).map_err(|e| format!("Failed to load plugin '{}': {}", path.display(), e))?
};
// Creates plugin instances from raw pointers without validation
Box::from_raw(raw)
```

**Recommendation:**
- **Immediate:** Add prominent security warnings in documentation
- **High Priority:**
  - Implement plugin signature verification (code signing)
  - Add permission model (declare capabilities: filesystem, network, etc.)
  - Consider WASM-based plugin system for sandboxing
- **Additional:**
  - Maintain official plugin registry with security vetting
  - Implement plugin isolation (separate processes)
  - Add `--allow-plugins` flag (deny by default)

---

### HIGH Severity

#### H-1: Parser Stack Overflow - No Recursion Depth Limit
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-parser/src/expr.rs` (entire file)

**Description:**
The recursive descent parser has no explicit depth limit for nested expressions. Deeply nested expressions (e.g., 10,000 levels of parentheses or nested function calls) could cause stack overflow, leading to denial of service.

**Risk:**
- Denial of service via crafted input
- Stack overflow crashes
- Potential for exploitation if stack overflow is controlled

**Evidence:**
- No `MAX_RECURSION_DEPTH` constant found in parser
- Recursive functions like `parse_expr()`, `parse_assignment()`, `parse_ternary()` have no depth tracking
- Parser file `/Users/sswoo/study/projects/vais/crates/vais-parser/src/lib.rs` shows synchronization but no depth limits

**Recommendation:**
- **Immediate:** Add recursion depth tracking and limit (suggested: 256-512)
```rust
const MAX_PARSE_DEPTH: usize = 256;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    depth: usize,  // Add this field
    // ...
}

fn parse_expr(&mut self) -> ParseResult<Spanned<Expr>> {
    if self.depth > MAX_PARSE_DEPTH {
        return Err(ParseError::TooDeep);
    }
    self.depth += 1;
    let result = self.parse_assignment();
    self.depth -= 1;
    result
}
```

---

#### H-2: Codegen - Potential LLVM IR Injection
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-codegen/src/lib.rs`

**Description:**
The code generator creates LLVM IR as text strings. While type checking should prevent malicious constructs, improper escaping of identifiers or string literals could potentially allow injection of arbitrary LLVM IR.

**Risk:**
- Malicious LLVM IR generation
- Bypassing safety checks
- Potential for generating code with unintended behavior

**Investigation:**
- Text-based IR generation found in default backend
- FFI validation exists (good!) but limited to FFI boundaries
- String handling in codegen needs escaping verification

**Recommendation:**
- **Immediate:** Audit all string interpolation in IR generation
- **High Priority:**
  - Ensure all user-controlled identifiers are properly escaped/validated
  - Consider switching to inkwell backend (typed LLVM API) as default
  - Add IR validation pass before passing to clang
- **Testing:** Add fuzzing for identifier edge cases (special chars, LLVM keywords)

---

#### H-3: Excessive Use of `.unwrap()` Leading to Panic Risk
**Location:** Throughout codebase (1,337 instances)

**Description:**
The codebase contains 1,337 uses of `.unwrap()` which will panic if the value is None/Err. While many may be in test code or provably safe contexts, unwraps on user-controlled input create denial of service vulnerabilities.

**Critical Files:**
- `/Users/sswoo/study/projects/vais/crates/vais-playground-server/src/main.rs:433-434` - unwrap on path conversion
- `/Users/sswoo/study/projects/vais/crates/vais-codegen/src/lib.rs` - 167 unwraps
- Parser and type checker files

**Risk:**
- Denial of service via crafted input causing panics
- Poor error messages for users
- Compiler crashes on malformed input

**Recommendation:**
- **Immediate:** Audit all unwraps in user-facing input handling code
- **High Priority:**
  - Replace unwraps with proper error handling in:
    - Parser (public APIs)
    - Type checker
    - Code generator (user input paths)
    - CLI argument processing
- **Long-term:** Add clippy lint to deny unwrap in production code
```toml
[workspace.lints.clippy]
unwrap_used = "deny"  # For non-test code
```

---

#### H-4: No Compilation Timeout Protection
**Location:** `/Users/sswoo/study/projects/vais/crates/vaisc/src/main.rs`

**Description:**
The compiler has no timeout for compilation operations. A crafted input with excessive complexity (e.g., type inference loops, macro expansion bombs) could cause indefinite compilation.

**Risk:**
- Denial of service in CI/CD pipelines
- Resource exhaustion in shared environments
- Playground server hangs (despite semaphore limiting concurrency)

**Recommendation:**
- **Immediate:** Add compilation timeout (suggest 30-60 seconds default, configurable)
```rust
tokio::time::timeout(
    Duration::from_secs(timeout),
    compile_task
)
```
- **Additional:**
  - Add phase-specific timeouts (parsing, type checking, codegen)
  - Implement complexity metrics and early abort

---

### MEDIUM Severity

#### M-1: FFI Function Validation Has Warning-Only Mode
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-codegen/src/ffi.rs:36-128`

**Description:**
The FFI validation properly detects unsafe patterns (generics in FFI, trait objects) but some checks only emit warnings via `eprintln!` instead of hard errors.

**Evidence:**
```rust
// Warning only - does not prevent compilation
eprintln!("Warning: FFI parameter '{}' uses unknown type '{}'", param_name, name);
Ok(())
```

**Risk:**
- Users may miss warnings and ship unsafe FFI code
- Undefined behavior if FFI types are mismatched
- ABI incompatibility

**Recommendation:**
- **Immediate:** Make FFI validation errors hard failures by default
- **Add flag:** `--allow-unsafe-ffi` for intentional unsafe FFI (with explicit opt-in)
- **Improve:** Better error messages explaining why FFI type is unsafe

---

#### M-2: Import Path Validation Present But Limited Testing
**Location:** `/Users/sswoo/study/projects/vais/crates/vaisc/tests/import_security_tests.rs`

**Description:**
Good security tests exist for path traversal prevention, but the actual implementation of import resolution needs verification.

**Positive Findings:**
- Tests for `../` path traversal attacks
- Tests for absolute path imports (`/etc/passwd`)
- Tests for symlink attacks
- Tests for non-.vais file rejection

**Gaps:**
- No verification of canonicalization in actual import code
- Tests are integration tests (good) but no unit tests of path validation
- No tests for Windows-specific path issues (`C:\`, UNC paths)

**Recommendation:**
- **Immediate:** Verify canonicalization is actually implemented in import resolver
- **Add:** Unit tests for path validation logic
- **Add:** Windows-specific path security tests
- **Document:** Security guarantees of import system

---

#### M-3: Standard Library Uses Unsafe Memory Operations
**Location:** `/Users/sswoo/study/projects/vais/std/io.vais:39-100`

**Description:**
The standard library's I/O module uses low-level memory operations (`malloc`, `free`, `load_byte`, `store_byte`) without bounds checking.

**Evidence:**
```vais
buffer := malloc(max_len)
// ...
c := load_byte(buffer + i)  // No bounds checking
```

**Risk:**
- Buffer overflows if max_len exceeds buffer
- Use-after-free if buffer used after free
- Users may copy unsafe patterns

**Recommendation:**
- **Immediate:** Add bounds checking wrappers in std lib
- **High Priority:**
  - Provide safe abstraction layer over raw memory ops
  - Add runtime bounds checking (at least in debug mode)
  - Document unsafe operations prominently
- **Consider:** Making raw memory ops require `unsafe` context in language

---

#### M-4: No Rate Limiting on Playground Server
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-playground-server/src/main.rs`

**Description:**
The playground server limits concurrent compilations (semaphore) but has no rate limiting per IP or user.

**Risk:**
- DoS via rapid requests from single IP
- Resource exhaustion
- Abuse by malicious actors

**Recommendation:**
- **Immediate:** Add rate limiting middleware
```rust
use tower_governor::{GovernorLayer, GovernorConfig};

let governor_conf = Box::new(
    GovernorConfig::default()
        .per_millisecond(100)  // 10 requests per second
        .burst_size(5)
);
app.layer(GovernorLayer { config: Box::leak(governor_conf) })
```
- **Additional:**
  - CAPTCHA for repeated failures
  - IP-based banning for abuse
  - CloudFlare or similar protection

---

#### M-5: Dependency Vulnerabilities Unknown
**Location:** `/Users/sswoo/study/projects/vais/Cargo.toml`

**Description:**
No evidence of regular dependency security audits. `cargo audit` is not installed/used in CI.

**Recommendation:**
- **Immediate:** Install and run `cargo audit`
```bash
cargo install cargo-audit
cargo audit
```
- **High Priority:**
  - Add cargo-audit to CI pipeline
  - Set up automated dependency updates (Dependabot/Renovate)
  - Subscribe to RustSec advisory notifications
- **Additional:**
  - Pin dependency versions for reproducible builds
  - Review transitive dependencies for supply chain risks

---

### LOW Severity

#### L-1: Unsafe Blocks Present But Necessary
**Location:** Multiple files (77 instances)

**Description:**
The codebase contains 77 `unsafe` blocks, primarily in:
- Plugin loading (necessary for FFI)
- GC implementation (necessary for memory management)
- Profiler FFI bindings
- Hot reload dynamic loading

**Assessment:**
Most unsafe blocks appear necessary and well-documented. No obvious misuse detected in manual review.

**Recommendation:**
- **Low Priority:** Audit each unsafe block with `// SAFETY:` comments
- **Document:** Why each unsafe block is necessary
- **Review:** Security team review of all unsafe code

---

#### L-2: Environment Variable Usage
**Location:** 10+ files use `std::env::var`

**Description:**
Environment variables are used for configuration (VAIS_STD_PATH, PLAYGROUND_*, etc.). This is generally acceptable but should be validated.

**Recommendation:**
- **Validate:** All environment variable inputs
- **Document:** Security implications of each environment variable
- **Consider:** Privilege separation (don't run compiler as root)

---

#### L-3: Debug Information in Release Builds
**Location:** Cargo.toml profile configurations

**Description:**
Debug information should be stripped from release builds to reduce binary size and information leakage.

**Recommendation:**
- **Add:** Explicit profile configurations
```toml
[profile.release]
debug = false
strip = true
```

---

## Security Strengths

The following positive security practices were observed:

### 1. Dedicated Security Analyzer Crate
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-security/`

Excellent proactive security analysis including:
- Buffer overflow detection
- Hardcoded secret scanning (with entropy analysis)
- SQL/command injection detection
- Use-after-free tracking
- Integer overflow detection

**Recommendation:** Integrate into default compilation pipeline with warnings.

### 2. Comprehensive Import Security Tests
**Location:** `/Users/sswoo/study/projects/vais/crates/vaisc/tests/import_security_tests.rs`

Well-designed test suite covering:
- Path traversal attacks
- Symlink attacks
- Absolute path prevention
- File extension validation

### 3. FFI Type Validation
**Location:** `/Users/sswoo/study/projects/vais/crates/vais-codegen/src/ffi.rs`

Good validation preventing:
- Generic types in FFI (not C-compatible)
- Trait objects in FFI
- Platform-dependent type warnings

### 4. Type Safety
The compiler uses Rust, providing:
- Memory safety by default
- No null pointer dereferences
- Thread safety guarantees

### 5. Fuzz Testing Infrastructure
**Location:** `/Users/sswoo/study/projects/vais/fuzz/`

Fuzzing targets exist for:
- Lexer
- Parser
- Full compilation pipeline

---

## Recommendations Summary

### Immediate Actions (Within 1 Week)

1. **Add parser recursion depth limit** (H-1)
2. **Implement playground execution timeout** (C-1)
3. **Add rate limiting to playground server** (M-4)
4. **Audit unwrap() usage in user input paths** (H-3)
5. **Run cargo audit and fix critical vulnerabilities** (M-5)

### High Priority (Within 1 Month)

1. **Implement playground sandboxing** (C-1)
2. **Add plugin signature verification** (C-2)
3. **Add compilation timeout** (H-4)
4. **Review and fix LLVM IR generation escaping** (H-2)
5. **Make FFI validation errors hard failures** (M-1)

### Medium Priority (Within 3 Months)

1. **Implement plugin permission model** (C-2)
2. **Add bounds checking to std lib memory ops** (M-3)
3. **Improve import path validation testing** (M-2)
4. **Add dependency security scanning to CI** (M-5)
5. **Document security architecture**

### Long-term (6+ Months)

1. **Consider WASM-based plugin system** (C-2)
2. **Migrate to inkwell backend for type safety** (H-2)
3. **Add clippy lints to prevent unsafe patterns**
4. **Establish security bug bounty program**
5. **Third-party security audit**

---

## Testing Recommendations

1. **Add Security Test Suite:**
   - DoS tests (deep recursion, large inputs, compilation bombs)
   - Injection tests (LLVM IR, FFI)
   - Resource exhaustion tests

2. **Continuous Fuzzing:**
   - Run fuzzing in CI (OSS-Fuzz integration)
   - Increase corpus coverage
   - Add structure-aware fuzzing

3. **Penetration Testing:**
   - Red team exercise on playground server
   - Plugin security testing
   - Supply chain attack simulation

---

## Conclusion

The Vais compiler demonstrates good security awareness in several areas, particularly around path traversal prevention and FFI validation. However, critical vulnerabilities exist in the playground server's code execution environment and the plugin system's lack of sandboxing.

The highest priority items are:
1. Sandboxing playground code execution
2. Adding recursion depth limits to prevent stack overflow
3. Implementing timeouts for compilation and execution
4. Reducing unwrap() usage in critical paths

With these improvements, the overall risk rating could be reduced to **LOW-MEDIUM**.

---

## Appendix A: Vulnerability Statistics

| Severity | Count | Addressed | Open |
|----------|-------|-----------|------|
| Critical | 2     | 0         | 2    |
| High     | 4     | 0         | 4    |
| Medium   | 5     | 0         | 5    |
| Low      | 3     | 0         | 3    |
| **Total** | **14** | **0**   | **14** |

## Appendix B: Code Metrics

- **Total Lines of Rust Code:** ~95,691
- **Unsafe Blocks:** 77
- **Unwrap Calls:** 1,337
- **Expect Calls:** 255
- **Panic Calls:** 86
- **Security Tests:** 16 test functions in import_security_tests.rs
- **Crates:** 30+

---

**Report Generated:** 2026-01-31
**Next Audit Recommended:** 2026-04-31 (3 months)
