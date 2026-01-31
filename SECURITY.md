# Security Policy

## Supported Versions

We take security seriously for the Vais programming language compiler. The following versions are currently supported with security updates:

| Version | Supported          | Status        |
| ------- | ------------------ | ------------- |
| 0.1.x   | :white_check_mark: | Development   |
| < 0.1.0 | :x:                | Pre-release   |

**Note:** Vais is currently in active development (version 0.1.x). Security updates will be applied to the latest development version. Once we reach 1.0.0, we will maintain security support for the current major version and the previous major version.

## Security Model

### Trust Boundaries

1. **Compiler Inputs:**
   - Source code files (`.vais`)
   - Import/module paths
   - CLI arguments
   - Configuration files

2. **Generated Code:**
   - LLVM IR
   - Native binaries
   - FFI declarations

3. **Runtime Components:**
   - Plugin system (native shared libraries)
   - Playground server (remote code execution)
   - Package registry

### What We Protect Against

- **Code Injection:** Malicious LLVM IR generation
- **Path Traversal:** Unauthorized file system access via imports
- **Resource Exhaustion:** DoS via compiler bombs (deep recursion, macro expansion)
- **FFI Misuse:** Type safety violations at FFI boundaries
- **Supply Chain:** Compromised dependencies or packages

### Out of Scope

The following are **not** considered security vulnerabilities:

- Bugs in user-written Vais code (use our security analyzer: `vais-security`)
- Performance issues that don't lead to DoS
- Compilation errors on malformed input (expected behavior)
- Behaviors explicitly documented as unsafe

## Reporting a Vulnerability

### Where to Report

**DO NOT** report security vulnerabilities through public GitHub issues.

Instead, please report security vulnerabilities to:

**Email:** security@vais.dev (if not available, create a GitHub Security Advisory)

**GitHub Security Advisory:**
1. Go to https://github.com/sswoo88/vais/security/advisories
2. Click "New draft security advisory"
3. Fill out the form with details

### What to Include

Please include the following information in your report:

1. **Description:** Clear description of the vulnerability
2. **Impact:** What an attacker could achieve
3. **Reproduction Steps:** Minimal steps to reproduce
4. **Proof of Concept:** Example code or commands (if safe to include)
5. **Environment:** OS, compiler version, relevant configuration
6. **Suggested Fix:** If you have ideas (optional)

### Example Report

```
Subject: [SECURITY] Stack overflow in parser via deep recursion

Description:
The Vais parser can be crashed with deeply nested expressions, causing
a stack overflow and denial of service.

Impact:
An attacker can crash the compiler or any service using it (including
the playground server) by submitting crafted Vais source code.

Reproduction:
1. Create a file with 10,000 nested parentheses: (((((...))))
2. Run: vaisc input.vais
3. Observe: Stack overflow crash

Environment:
- Vais version: 0.1.0
- OS: macOS 14.0 (also tested on Linux)
- Command: vaisc build deep.vais

Suggested Fix:
Add MAX_RECURSION_DEPTH constant and track depth in Parser struct.
```

## Response Timeline

We are committed to addressing security vulnerabilities promptly:

| Severity  | First Response | Fix Target    | Disclosure    |
|-----------|----------------|---------------|---------------|
| Critical  | 24 hours       | 7 days        | After fix     |
| High      | 48 hours       | 14 days       | After fix     |
| Medium    | 1 week         | 30 days       | After fix     |
| Low       | 2 weeks        | 90 days       | With release  |

**Severity Levels:**

- **Critical:** Remote code execution, arbitrary file access, complete system compromise
- **High:** DoS, privilege escalation, information disclosure of sensitive data
- **Medium:** Limited DoS, minor information disclosure, bypassing security features
- **Low:** Low-impact issues, theoretical vulnerabilities

## Disclosure Policy

We follow **coordinated disclosure**:

1. **Private Disclosure:** You report the vulnerability privately
2. **Investigation:** We confirm and investigate (target: 48-72 hours)
3. **Development:** We develop and test a fix
4. **Notification:** We notify you when fix is ready
5. **Public Disclosure:** We publish advisory after fix is released (or after 90 days, whichever comes first)

### Credit

We believe in giving credit where it's due:

- Security researchers will be credited in release notes and security advisories (unless you prefer to remain anonymous)
- We may offer recognition on our website/documentation
- For significant vulnerabilities, we may offer rewards (program TBD)

## Security Updates

### How to Stay Informed

- **GitHub Security Advisories:** https://github.com/sswoo88/vais/security/advisories
- **Release Notes:** Check CHANGELOG.md for security fixes
- **Mailing List:** (TBD) Subscribe to security-announce@vais.dev

### Update Recommendations

1. **Always use the latest version** of the Vais compiler
2. **Enable security checks:** Use `vaisc check` with security warnings
3. **Review dependencies:** Run `vaisc pkg audit` regularly
4. **Monitor advisories:** Subscribe to security notifications

## Secure Usage Guidelines

### For Compiler Users

1. **Don't compile untrusted code without sandboxing:**
   ```bash
   # Bad: Compiling untrusted code directly
   vaisc build untrusted.vais

   # Better: Use a container or VM
   docker run --rm -v $(pwd):/work vais:latest vaisc build untrusted.vais
   ```

2. **Validate import paths:**
   - Never use user-controlled import paths without validation
   - Stick to relative imports within your project

3. **Use the security analyzer:**
   ```bash
   vaisc build --enable-security-analysis main.vais
   ```

4. **Keep dependencies updated:**
   ```bash
   vaisc pkg update
   vaisc pkg audit
   ```

### For Plugin Developers

1. **Sign your plugins** (when signing is implemented)
2. **Minimize permissions** required
3. **Validate all inputs** from the compiler
4. **Document security implications** of your plugin

### For Server/CI Operators

1. **Sandbox compilation:**
   - Use containers (Docker, Podman)
   - Apply resource limits (CPU, memory, time)
   - Disable network access during compilation

2. **Limit concurrent compilations:**
   ```bash
   vaisc build -j 4  # Limit to 4 parallel jobs
   ```

3. **Set timeouts:**
   - Use process supervisors (systemd, supervisord)
   - Apply job timeouts in CI/CD

4. **Restrict file system access:**
   - Read-only mounts for source code
   - Separate output directory with size limits

## Known Security Features

### Built-in Security Mechanisms

1. **Path Canonicalization:** Import paths are canonicalized to prevent traversal
2. **FFI Type Validation:** FFI boundaries are type-checked for safety
3. **Memory Safety:** Rust-based compiler is memory-safe by default
4. **Static Analysis:** Built-in security analyzer (`vais-security` crate)

### Security Tools

- `vais-security`: Static security analyzer for Vais code
- `vaisc pkg audit`: Dependency vulnerability scanner
- `vais-supply-chain`: Supply chain security tools

## Bug Bounty Program

**Status:** Not currently available

We are considering establishing a bug bounty program in the future. Until then, we offer:

- Public recognition and credit
- Direct communication with the development team
- Potential job/collaboration opportunities for significant findings

## Frequently Asked Questions

### Q: Is it safe to run the Vais Playground?

**A:** The playground should only be run with proper sandboxing. See security audit report for recommendations. We do NOT recommend running the playground server in production without:
- Containerization (Docker/Podman)
- Resource limits (CPU, memory, timeout)
- Network isolation
- Rate limiting

### Q: Can I trust third-party plugins?

**A:** Currently, plugins run with full compiler privileges and can execute arbitrary code. Only load plugins from sources you trust completely. We are working on a permission model for plugins.

### Q: How do I report a vulnerability in a Vais package (not the compiler)?

**A:** Contact the package maintainer directly. If the package is in the official registry and the maintainer is unresponsive, you can report to security@vais.dev.

### Q: What about dependencies (Rust crates)?

**A:** We regularly audit our dependencies using `cargo audit`. If you find a vulnerability in one of our dependencies, please report it to the respective project AND notify us so we can update.

### Q: Is Vais safe for production use?

**A:** Vais is currently in development (0.1.x). While we take security seriously, we recommend:
- Thorough testing before production deployment
- Containerization and sandboxing
- Regular updates
- Security review of your specific use case

## Security Roadmap

Planned security improvements:

### Short-term (0.2.0)
- [ ] Parser recursion depth limits
- [ ] Compilation timeouts
- [ ] Playground sandboxing
- [ ] Enhanced fuzzing coverage

### Medium-term (0.3.0)
- [ ] Plugin signature verification
- [ ] Plugin permission model
- [ ] WASM-based plugin sandboxing
- [ ] Dependency lock file signing

### Long-term (1.0.0)
- [ ] Security bug bounty program
- [ ] Third-party security audit
- [ ] Formal security certification
- [ ] Security-focused documentation

## Contact

- **Security Issues:** security@vais.dev (or GitHub Security Advisory)
- **General Security Questions:** GitHub Discussions (tag: security)
- **Documentation:** docs/SECURITY_AUDIT.md

## Acknowledgments

We thank the security researchers and community members who help keep Vais secure. Notable contributors:

- (This section will be updated as security researchers contribute)

---

**Last Updated:** January 31, 2026
**Next Review:** April 30, 2026
