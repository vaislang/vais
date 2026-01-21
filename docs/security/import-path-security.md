# Import Path Security

## Overview

The Vais compiler implements robust security measures to protect against malicious import paths that could be used to access unauthorized files or directories.

## Security Threats Addressed

### 1. Path Traversal Attacks

Path traversal attacks attempt to escape allowed directories using sequences like `../` to access files outside the project:

```vais
// BLOCKED: Attempting to access system files
U ../../../etc/passwd
```

**Protection**: All import paths are canonicalized and validated to ensure they remain within:
- The current project directory
- The Vais standard library directory

### 2. Symlink Attacks

Symlinks could be used to create a seemingly safe path that actually points to sensitive system files:

```bash
# Attacker creates symlink
ln -s /etc/passwd myproject/safe_module.vais
```

```vais
// BLOCKED: Even though path looks safe, symlink target is validated
U safe_module
```

**Protection**: The compiler uses `canonicalize()` to resolve the real path after following all symlinks, then validates the actual target location.

### 3. Absolute Path Access

Direct absolute paths to system files:

```vais
// BLOCKED: Absolute paths to system locations
U /var/log/system.vais
```

**Protection**: Import paths are validated against allowed base directories.

## Implementation Details

### Security Validation Pipeline

1. **Path Construction**: Module path segments are joined to form a file path
2. **Canonicalization**: `std::fs::canonicalize()` resolves symlinks and normalizes the path
3. **Validation**: The canonical path is checked against allowed directories:
   - Project root (current working directory)
   - Standard library path (VAIS_STD_PATH or auto-detected)
4. **Extension Check**: Only `.vais` files are allowed

### Code Location

The security implementation is in `crates/vaisc/src/main.rs`:

- `resolve_import_path()`: Main entry point for import resolution
- `validate_and_canonicalize_import()`: Security validation logic

## Allowed Import Patterns

### Local Module Imports
```vais
// ✓ Import from same directory
U utils

// ✓ Import from subdirectory
U modules::auth
```

### Standard Library Imports
```vais
// ✓ Import from std library
U std/vec
U std/option
```

### Relative Imports (Within Project)
```vais
// ✓ Import from parent directory (if within project root)
U ../common
```

## Blocked Import Patterns

```vais
// ✗ Path traversal outside project
U ../../../etc/passwd

// ✗ Absolute system paths
U /etc/hosts

// ✗ Symlinks pointing outside allowed directories
U symlink_to_system_file

// ✗ Non-.vais files (even if accessible)
U malicious.txt
```

## Error Messages

The compiler provides clear but security-conscious error messages:

```
error: Import path 'malicious' is outside allowed directories
```

```
error: Invalid import file type: 'config.txt' (only .vais files allowed)
```

```
error: Cannot find module 'nonexistent': tried 'nonexistent.vais' and 'nonexistent/mod.vais'
```

## Testing

Comprehensive security tests are located in `crates/vaisc/tests/import_security_tests.rs`:

```bash
# Run security tests
cargo test --package vaisc --test import_security_tests

# Run all compiler tests
cargo test --package vaisc
```

## Configuration

### Standard Library Path

The standard library path can be configured via:

1. `VAIS_STD_PATH` environment variable
2. Relative to executable (for installed compiler)
3. Auto-detection from project root (for development)

Example:
```bash
export VAIS_STD_PATH=/usr/local/lib/vais/std
vaisc build myprogram.vais
```

## Security Considerations

### Defense in Depth

The import security system implements multiple layers of protection:

1. **Parser Level**: Restricts syntactically valid import paths
2. **Resolution Level**: Validates constructed file paths
3. **Canonicalization**: Resolves true file locations
4. **Access Control**: Enforces directory boundaries
5. **Type Checking**: Only allows `.vais` source files

### Known Limitations

1. **Symbolic Links Within Project**: Symlinks that point to locations within the allowed directories are permitted. This is intentional to support common development practices.

2. **TOCTOU Considerations**: There's a theoretical time-of-check-time-of-use gap between validation and file reading. This is mitigated by:
   - Using the canonical path for both validation and loading
   - File system operations failing on permission errors
   - The compiler running with user privileges (not elevated)

## Best Practices

### For Users

1. Only compile code from trusted sources
2. Review import statements in third-party modules
3. Use relative imports within your project
4. Keep the standard library in a protected location

### For Plugin Developers

When developing compiler plugins:

1. Use the standard import resolution API
2. Don't bypass security checks
3. Validate any paths received from user input
4. Follow the principle of least privilege

## Related Documentation

- [Module System](../language/modules.md)
- [Standard Library](../stdlib/README.md)
- [Compiler Architecture](../compiler/architecture.md)

## Changelog

### Version 0.0.1 (2026-01-21)
- Initial implementation of import path security
- Added canonicalization-based validation
- Implemented directory boundary checks
- Added comprehensive security test suite
