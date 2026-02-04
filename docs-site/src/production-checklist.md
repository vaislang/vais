# Production Readiness Checklist

This comprehensive checklist ensures your Vais application is production-ready. It covers all Phase 33 features and critical infrastructure components.

## 1. TLS/HTTPS Configuration

### Certificate Setup
- [ ] Obtain valid TLS certificate (PEM format)
- [ ] Configure certificate chain correctly
- [ ] Store private key securely with proper file permissions (0600)
- [ ] Set up certificate renewal process before expiration
- [ ] Test certificate validity with OpenSSL: `openssl x509 -in cert.pem -text -noout`

### CA Bundle Configuration
- [ ] Specify system CA bundle path or include custom CA certificates
- [ ] Verify CA chain completeness with intermediate certificates
- [ ] Test chain validation: `openssl verify -CAfile ca-bundle.pem cert.pem`
- [ ] Document CA bundle source and update schedule

### Server-Side TLS (HTTPS Server)
- [ ] Configure TLS version (minimum TLS 1.2)
- [ ] Enable certificate-based authentication
- [ ] Bind HTTPS listener to correct port (typically 443)
- [ ] Load certificates in application startup
- [ ] Implement certificate hot-reloading for zero-downtime updates

Example TLS server configuration:
```vais
F create_tls_server(cert_path: Str, key_path: Str, port: i32) -> Server {
  L server := Server::with_tls(cert_path, key_path)
  L _ := server.bind("0.0.0.0", port)
  R server
}
```

### Client-Side TLS (HTTPS Client)
- [ ] Configure client certificate verification
- [ ] Set appropriate certificate verification mode
- [ ] Handle certificate validation errors gracefully
- [ ] Implement custom CA bundle for client requests
- [ ] Test HTTPS client with self-signed certificates in development

Example HTTPS client configuration:
```vais
F fetch_secure(url: Str, ca_bundle: Str) -> Result<Str> {
  L client := Client::with_ca_bundle(ca_bundle)
  L response := client.get(url)?
  R response.body()
}
```

### Certificate Verification Modes
- [ ] Production: Strict verification (verify both hostname and certificate chain)
- [ ] Staging: Standard verification with proper error logging
- [ ] Development: Allow custom CA or self-signed (document security implications)
- [ ] Monitoring: Log all certificate validation failures

### Dependencies
- [ ] Verify OpenSSL or LibreSSL is installed and accessible
- [ ] Check system library paths: `pkg-config --cflags openssl`
- [ ] Pin dependency versions in Cargo.lock
- [ ] Document native dependency installation instructions

---

## 2. Cross-Platform Async I/O

### Platform Support Matrix
| Platform | Reactor | Status | Notes |
|----------|---------|--------|-------|
| macOS | kqueue | Recommended | Efficient, well-tested |
| Linux | epoll | Recommended | High-performance, standard |
| Windows | IOCP | Supported | I/O Completion Ports |
| BSD | kqueue | Supported | Similar to macOS |

- [ ] Test async I/O on all target platforms
- [ ] Verify platform detection works correctly
- [ ] Document platform-specific performance characteristics

### Reactor Configuration
- [ ] Call `async_platform()` to detect and initialize correct reactor
- [ ] Set appropriate reactor timeout values
- [ ] Configure event queue sizes for workload
- [ ] Tune reactor thread pool size (typically CPU count)

Example reactor initialization:
```vais
F setup_async_runtime() -> Result<()> {
  L platform := async_platform()
  I platform == "kqueue" {
    L _ := setup_kqueue_reactor()
  } E I platform == "epoll" {
    L _ := setup_epoll_reactor()
  } E {
    R Err("Unsupported platform")
  }
  R Ok(())
}
```

### Timer Setup
- [ ] Configure timer precision (millisecond vs microsecond)
- [ ] Test timeout accuracy under load
- [ ] Implement timer sweep mechanism for cleanup
- [ ] Document timer resolution limitations per platform

### Event Loop Best Practices
- [ ] Never block in async code (use `spawn_blocking` for CPU-bound work)
- [ ] Keep event loop iterations short
- [ ] Monitor event loop latency with tracing
- [ ] Implement fair scheduling between tasks
- [ ] Use structured concurrency (limit concurrent tasks)

### Platform Detection
- [ ] Use `async_platform()` function to detect runtime environment
- [ ] Cache platform detection result at startup
- [ ] Document all supported reactor names

```vais
F main() {
  L platform := async_platform()
  L _ := println("Running on: {}", platform)
}
```

---

## 3. Package Registry

### Registry Server Deployment

#### Docker Deployment
- [ ] Build registry server Docker image
- [ ] Configure Docker Compose with persistent storage
- [ ] Set up SQLite database volume for package metadata
- [ ] Configure registry environment variables

Example Docker Compose configuration:
```yaml
version: '3.8'
services:
  registry:
    image: vais-registry:latest
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: sqlite:///data/registry.db
      JWT_SECRET: ${JWT_SECRET}
      TLS_ENABLED: "true"
      TLS_CERT: /etc/registry/cert.pem
      TLS_KEY: /etc/registry/key.pem
    volumes:
      - registry_data:/data
      - ./certs:/etc/registry
    restart: always

volumes:
  registry_data:
```

#### Production Configuration
- [ ] Enable HTTPS with valid certificates
- [ ] Set up reverse proxy (nginx/Caddy) for load balancing
- [ ] Configure authentication backend
- [ ] Set up monitoring and alerting
- [ ] Implement log aggregation

### Publishing Packages
- [ ] Create `Vais.toml` metadata file with package information
- [ ] Set semantic version correctly
- [ ] Document all public APIs
- [ ] Run tests before publishing: `cargo test`

Example Vais.toml:
```toml
[package]
name = "my-lib"
version = "1.0.0"
authors = ["Your Name"]
description = "Brief description"
repository = "https://github.com/user/my-lib"

[dependencies]
# List dependencies here
```

Publishing command:
```bash
vais publish --registry https://registry.vais.dev
```

- [ ] Sign package with key: `vais publish --sign-key ./key.pem`
- [ ] Verify successful publication on registry
- [ ] Test dependency resolution

### Installing Dependencies
- [ ] Configure `.vais/config.toml` with registry URL
- [ ] Set up authentication credentials (username/password or token)
- [ ] Test `vais install` with various package sources

Example installing with credentials:
```bash
vais install --registry https://registry.vais.dev \
  --username user \
  --token ${REGISTRY_TOKEN}
```

- [ ] Implement lock file (`Vais.lock`) for reproducible builds
- [ ] Document dependency update process
- [ ] Set up dependency security scanning

### Semantic Versioning Constraints
Supported version constraint operators:

| Operator | Example | Meaning |
|----------|---------|---------|
| `^` | `^1.2.3` | Compatible release: >=1.2.3, <2.0.0 |
| `~` | `~1.2.3` | Patch release: >=1.2.3, <1.3.0 |
| `>=` | `>=1.2.3` | At least this version |
| `<=` | `<=2.0.0` | At most this version |
| `=` | `=1.2.3` | Exact version |
| `*` | `1.2.*` | Any patch version |

- [ ] Document version constraint strategy
- [ ] Test resolution with conflicting constraints
- [ ] Configure version preference (latest stable vs specific)

### Package Signing and Verification
- [ ] Generate signing key pair: `vais-registry generate-keys`
- [ ] Store private key securely (encrypted at rest)
- [ ] Configure registry to verify SHA-256 hashes
- [ ] Implement signature verification in package manager

SHA-256 verification:
```bash
sha256sum package.tar.gz > package.tar.gz.sha256
vais publish --signature-file package.tar.gz.sha256
```

- [ ] Document key rotation procedures
- [ ] Test signature verification on client side

### Authentication (JWT Tokens)
- [ ] Configure JWT token generation in registry
- [ ] Set token expiration (recommended: 24-48 hours)
- [ ] Implement token refresh mechanism
- [ ] Store JWT secret securely

JWT configuration:
```vais
F authenticate(username: Str, password: Str) -> Result<JwtToken> {
  L validated := validate_credentials(username, password)?
  L token := JwtToken::new(username, 24h)
  R Ok(token)
}
```

- [ ] Use HTTPS-only for authentication endpoints
- [ ] Implement rate limiting on auth endpoints
- [ ] Log all authentication attempts
- [ ] Monitor for token theft/misuse

---

## 4. Debugging (DAP)

### VSCode Launch Configuration
Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Vais Program",
      "type": "vais",
      "request": "launch",
      "program": "${workspaceFolder}/bin/myapp",
      "args": ["--verbose"],
      "stopOnEntry": false,
      "console": "integratedTerminal",
      "cwd": "${workspaceFolder}"
    },
    {
      "name": "Attach to Process",
      "type": "vais",
      "request": "attach",
      "processId": "${command:pickProcess}"
    }
  ]
}
```

Configuration checklist:
- [ ] Set correct `program` path to compiled binary
- [ ] Configure command-line arguments if needed
- [ ] Choose appropriate console (integratedTerminal, internalConsole, externalTerminal)
- [ ] Set working directory correctly
- [ ] Test launch configuration with simple program first

### Breakpoint Management
- [ ] Set line breakpoints by clicking gutter in VSCode
- [ ] Set conditional breakpoints: right-click line → Add Conditional Breakpoint

Example conditional breakpoint expression:
```
i > 100 && error_code == 42
```

- [ ] Set logpoints for non-intrusive debugging: `Iteration {i}: value={value}`
- [ ] Manage breakpoints in Debug panel (enable/disable/delete)
- [ ] Test breakpoint persistence across debug sessions
- [ ] Clear breakpoints before production code review

### Variable Inspection
- [ ] Inspect local variables in debug panel
- [ ] Expand complex types (structs, enums) to view fields
- [ ] Use Watch panel to track specific variables across execution
- [ ] Format variables: hover over variable for tooltip
- [ ] View memory representation if needed

Watch expression examples:
```
my_struct.field
array[index]
map.get("key")
```

### Step Debugging
- [ ] **Step Over** (F10): Execute current line, skip function bodies
- [ ] **Step Into** (F11): Enter function calls to debug inner logic
- [ ] **Step Out** (Shift+F11): Return from current function
- [ ] **Continue** (F5): Resume execution to next breakpoint
- [ ] **Pause** (F6): Interrupt execution to inspect state

Step debugging best practices:
- [ ] Use step-over for high-level control flow
- [ ] Use step-into when investigating specific functions
- [ ] Combine with variable watches for efficiency
- [ ] Document steps needed to reproduce issues

### Call Stack Inspection
- [ ] View full call stack in Debug panel
- [ ] Click stack frames to navigate function calls
- [ ] Identify the call path that led to current location
- [ ] Check for infinite recursion patterns
- [ ] Monitor for stack overflow conditions

Call stack analysis:
- [ ] Verify expected function call order
- [ ] Check parameter values at each stack level
- [ ] Look for unexpected function calls
- [ ] Document complex call paths for code review

---

## 5. Structured Logging

### Log Levels
Configure logging with appropriate levels:

| Level | Usage | Example |
|-------|-------|---------|
| TRACE | Extremely detailed debugging | Individual field values, loop iterations |
| DEBUG | Development and debugging | Function entry/exit, state changes |
| INFO | Important application events | Server startup, configuration loaded |
| WARN | Warning conditions | Deprecated API usage, fallbacks triggered |
| ERROR | Error conditions | Failed requests, exceptions |

Example log level configuration:
```vais
F setup_logging() -> Result<()> {
  L logger := Logger::new()
  L _ := logger.set_level(LogLevel::INFO)
  L _ := logger.set_format(LogFormat::Json)
  R Ok(())
}
```

- [ ] Set appropriate default log level (INFO for production)
- [ ] Allow runtime log level adjustment via environment variable
- [ ] Document log level meanings for your application

### JSON Output Format
Enable JSON output for log aggregation systems:

```json
{
  "timestamp": "2025-02-04T12:34:56.789Z",
  "level": "INFO",
  "message": "Request processed",
  "target": "vais::server",
  "span": "request_id=abc123",
  "request_method": "GET",
  "request_path": "/api/users",
  "response_status": 200,
  "duration_ms": 42
}
```

Configuration:
```vais
L _ := logger.set_format(LogFormat::Json)
```

- [ ] Enable JSON format for production deployments
- [ ] Configure log aggregation (ELK, Datadog, CloudWatch)
- [ ] Test log parsing with aggregation system
- [ ] Monitor JSON formatting correctness

### File Output Configuration
- [ ] Configure log file path: `/var/log/myapp/vais.log`
- [ ] Set up log rotation (daily, by size)
- [ ] Configure retention policy (keep last 30 days)
- [ ] Ensure log directory has correct permissions
- [ ] Set up separate error log file if needed

Example file configuration:
```vais
F setup_file_logging() -> Result<()> {
  L logger := Logger::new()
  L _ := logger.set_output_file("/var/log/myapp/vais.log")
  L _ := logger.set_rotation_policy(RotationPolicy::Daily)
  L _ := logger.set_retention_days(30)
  R Ok(())
}
```

- [ ] Test log file creation and rotation
- [ ] Monitor disk space usage
- [ ] Implement automated archival of old logs

### Span-Based Tracing
Use spans to track request flow across components:

```vais
F handle_request(req_id: Str) {
  L _ := span!("handle_request", request_id = req_id, {
    L _ := authenticate_user()
    L _ := process_data()
    L _ := send_response()
  })
}
```

- [ ] Create spans for major operations
- [ ] Use unique request IDs for tracing
- [ ] Include context in span fields
- [ ] Test span correlation across services
- [ ] Document span naming convention

### Structured Fields
Include structured key=value fields in logs:

```vais
F log_request(method: Str, path: Str, status: i32, duration_ms: i64) {
  L _ := info!("Request completed";
    request_method = method,
    request_path = path,
    response_status = status,
    duration_ms = duration_ms
  )
}
```

Benefits:
- [ ] Easier log searching and filtering
- [ ] Better integration with log aggregation systems
- [ ] More maintainable than string formatting
- [ ] Performance benefits from pre-structured data

Structured field examples:
- User context: `user_id`, `username`, `organization_id`
- Request context: `request_id`, `session_id`, `trace_id`
- Performance: `duration_ms`, `memory_bytes`, `cpu_percent`
- Errors: `error_type`, `error_code`, `error_message`

---

## 6. Compression

### Gzip Compression
- [ ] Enable gzip compression for HTTP responses
- [ ] Configure minimum content size to compress (default 1KB)
- [ ] Test compression with various content types
- [ ] Verify client accepts gzip (Accept-Encoding header)

Example gzip configuration:
```vais
F create_gzip_middleware(min_size: usize) -> Middleware {
  R Middleware::gzip()
    .min_compress_size(min_size)
    .quality(GzipQuality::Default)
}
```

Configuration checklist:
- [ ] Enable gzip for text content (HTML, CSS, JSON, XML)
- [ ] Disable gzip for already-compressed content (JPEG, PNG, MP4)
- [ ] Test compression ratio and performance impact
- [ ] Monitor CPU usage under compression load

### Deflate Compression
- [ ] Consider deflate for compatibility with older clients
- [ ] Test deflate compression if required by clients
- [ ] Document compression algorithm preference

Deflate is less common but may be needed for legacy systems:
```vais
I request.accepts_deflate() {
  L _ := response.set_compression(CompressionType::Deflate)
}
```

### Streaming Compression
For large responses:
- [ ] Implement chunked transfer encoding
- [ ] Compress data in streaming fashion
- [ ] Monitor memory usage with streaming
- [ ] Test timeout handling for long-running streams

Streaming example:
```vais
F stream_large_file(path: Str) -> Stream<u8> {
  L file := File::open(path)?
  L compressed := GzipStream::new(file)
  R Stream::from(compressed)
}
```

### Compression Levels
Configure compression trade-offs:

| Level | Speed | Ratio | Use Case |
|-------|-------|-------|----------|
| Fast (1-3) | Very fast | 60-70% | Real-time APIs, low latency |
| Default (6) | Balanced | 75-80% | General purpose (recommended) |
| Best (9) | Slow | 85-90% | Static assets, batch processing |

- [ ] Use default compression for general APIs
- [ ] Use fast compression for real-time data
- [ ] Use best compression for static assets
- [ ] Benchmark compression levels for workload
- [ ] Document compression strategy decision

### Dependencies
- [ ] Ensure zlib is installed: `pkg-config --cflags zlib`
- [ ] Verify zlib library paths
- [ ] Test compression on all platforms
- [ ] Document zlib version requirements

---

## 7. Performance Tuning

### Compilation Optimization Flags
Configure release build optimizations:

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Enable Link Time Optimization
codegen-units = 1       # Slower build, faster binary
strip = true            # Strip debug symbols
panic = "abort"         # Smaller binary (no panic unwinding)
```

Optimization checklist:
- [ ] Enable LTO for production builds
- [ ] Set codegen-units to 1 for maximum optimization
- [ ] Test binary size impact
- [ ] Benchmark performance improvements
- [ ] Document build time impact

### Inkwell Backend (Default)
The default code generator provides ~36% faster compilation:

- [ ] Verify Inkwell is enabled: `cargo build --features inkwell`
- [ ] Test compilation time: `time cargo build --release`
- [ ] Compare with alternative backends if available
- [ ] Monitor memory usage during compilation
- [ ] Document compilation performance expectations

### Tail Call Optimization
- [ ] Identify tail-recursive functions in code
- [ ] Ensure functions end with direct recursive call
- [ ] Verify tail calls are optimized: check LLVM IR
- [ ] Test for stack overflow prevention

Tail call example:
```vais
F factorial_tail(n: i32, acc: i32) -> i32 {
  I n <= 1 {
    R acc
  }
  R factorial_tail(n - 1, acc * n)  # Tail call - optimized
}
```

### Inlining Thresholds
- [ ] Set appropriate inline threshold in compiler flags
- [ ] Profile hot functions for inlining candidates
- [ ] Avoid excessive inlining (code bloat)
- [ ] Test binary size impact
- [ ] Document inlining strategy

### MIR Optimization Pipeline
The Mid-level IR optimizer applies multiple passes:

- [ ] Verify MIR optimization is enabled
- [ ] Monitor optimization pass performance
- [ ] Test optimization impact on code correctness
- [ ] Profile specific optimization passes if slow
- [ ] Document any optimization-related issues

MIR optimizations include:
- Constant folding
- Dead code elimination
- Common subexpression elimination
- Function inlining
- Loop unrolling (with limits)

---

## 8. Security Checklist

### TLS Minimum Version
- [ ] Set minimum TLS version to 1.2 or higher
- [ ] Disable older protocols (SSL 3.0, TLS 1.0, TLS 1.1)
- [ ] Document TLS version policy
- [ ] Test with SSL Labs or similar tool

Configuration example:
```vais
F create_secure_server() -> Server {
  L server := Server::new()
  L _ := server.set_min_tls_version("1.2")
  L _ := server.set_max_tls_version("1.3")
  R server
}
```

### Certificate Validation
- [ ] Validate certificate chain against CA bundles
- [ ] Verify certificate expiration date
- [ ] Check certificate issuer validity
- [ ] Validate certificate subject matches domain
- [ ] Implement certificate pinning for sensitive connections

Validation checklist:
- [ ] Test valid certificates pass validation
- [ ] Test expired certificates are rejected
- [ ] Test certificates with wrong domain are rejected
- [ ] Test self-signed certificates (in development only)
- [ ] Test certificate chain validation

### SQL Injection Prevention (ORM Auto-Escaping)
The Vais ORM automatically escapes all parameters:

```vais
# Safe - parameter automatically escaped
L users := User::find_by_name(user_input)

# Safe - parameterized query
L users := db.query("SELECT * FROM users WHERE name = ?", user_input)

# Unsafe - string concatenation (NEVER do this)
# L users := db.query("SELECT * FROM users WHERE name = '" + user_input + "'")
```

ORM Security measures:
- [ ] Use ORM query builder methods (preferred)
- [ ] Use parameterized queries if raw SQL needed
- [ ] Never concatenate user input into SQL strings
- [ ] Audit code for string concatenation in queries
- [ ] Enable SQL query logging during development
- [ ] Test with SQL injection payloads: `'; DROP TABLE users; --`

### Input Validation
- [ ] Define validation rules for all user inputs
- [ ] Implement server-side validation (never trust client-side only)
- [ ] Validate input type, length, and format
- [ ] Sanitize inputs before database or external API calls
- [ ] Implement rate limiting on input endpoints

Input validation example:
```vais
F validate_user_input(email: Str, age: i32) -> Result<()> {
  I !email.contains("@") {
    R Err("Invalid email format")
  }
  I age < 0 || age > 150 {
    R Err("Invalid age")
  }
  R Ok(())
}
```

Validation checklist:
- [ ] Email format validation
- [ ] URL validation for user-provided links
- [ ] File type validation (check magic bytes, not extension)
- [ ] Length limits (prevent buffer overflow)
- [ ] Range validation (numeric values)
- [ ] Pattern validation (regex where appropriate)

### Package Signature Verification
- [ ] Verify package signatures before installation
- [ ] Maintain list of trusted package authors
- [ ] Implement signature revocation mechanism
- [ ] Document key compromise procedures
- [ ] Audit all installed packages regularly

Package security procedures:
- [ ] `vais package verify <package.tar.gz>`
- [ ] Check package signature against registry
- [ ] Monitor for unsigned packages in dependencies
- [ ] Implement dependency scanning in CI/CD

### General Security Practices
- [ ] Run security audit: `cargo audit`
- [ ] Keep dependencies updated: `cargo update`
- [ ] Review dependency changes before updating
- [ ] Implement secret management (environment variables, secret vaults)
- [ ] Use HTTPS everywhere (never plain HTTP in production)
- [ ] Implement CORS headers correctly
- [ ] Set security headers: CSP, HSTS, X-Frame-Options
- [ ] Implement proper error handling (don't leak sensitive info)
- [ ] Use constant-time comparisons for security-sensitive operations
- [ ] Document security assumptions and threat model

---

## Final Verification Checklist

Before deploying to production:

- [ ] All tests passing: `cargo test`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Security audit passed: `cargo audit`
- [ ] Performance benchmarks reviewed
- [ ] Logging configured and tested
- [ ] Monitoring and alerting configured
- [ ] Disaster recovery plan documented
- [ ] Runbooks created for common issues
- [ ] Load testing completed
- [ ] Security review completed
- [ ] Code review completed
- [ ] Documentation updated
- [ ] Deployment procedure tested
- [ ] Rollback procedure tested
- [ ] Staging environment mirrors production

---

## Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/) - Security best practices
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework) - Security guidelines
- [OpenSSL Documentation](https://www.openssl.org/docs/) - TLS/SSL configuration
- [LLVM Optimization Reference](https://llvm.org/docs/Passes/) - Compiler optimizations
- [Structured Logging Best Practices](https://www.kartar.net/2015/12/structured-logging/) - Logging patterns

