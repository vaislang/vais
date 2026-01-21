# Security Examples

This directory contains examples demonstrating the Vais compiler's security features.

## Import Path Security

The Vais compiler implements robust protection against malicious import paths.

### Safe Example

`safe_import.vais` demonstrates proper import usage:
- Importing from the standard library
- Importing from local project modules

```bash
# This will work correctly
vaisc check safe_import.vais
```

### Blocked Patterns

The following import patterns are automatically blocked by the compiler:

1. **Path Traversal**:
   ```vais
   U ../../../etc/passwd
   ```

2. **Symlink Attacks**:
   Creating a symlink to system files and trying to import it will be detected and blocked.

3. **Non-.vais Files**:
   ```vais
   U config.txt
   ```

## Testing Security

Run the security test suite:
```bash
cargo test --package vaisc --test import_security_tests
```

## Documentation

For detailed information about import path security, see:
- [Import Path Security Documentation](../../docs/security/import-path-security.md)
