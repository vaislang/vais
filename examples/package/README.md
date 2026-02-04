# Example Vais Package

This is an example package demonstrating the structure and requirements for publishing to the Vais package registry.

## Package Structure

```
example-package/
├── vais.toml          # Package manifest (required)
├── src/               # Source code directory (required)
│   └── lib.vais      # Library entry point
└── README.md          # Documentation (optional)
```

## vais.toml

The `vais.toml` file defines package metadata and dependencies:

```toml
[package]
name = "example-package"
version = "0.1.0"
authors = ["Your Name <email@example.com>"]
description = "Package description"
license = "MIT"

[dependencies]
# Semver version constraints
dependency-name = "^1.0.0"

[dev-dependencies]
# Development-only dependencies
test-framework = ">=0.3.0"
```

## Publishing

To publish this package to a registry:

```bash
# Login to the registry
vaisc pkg login --registry https://registry.vais.dev

# Publish the package
vaisc pkg publish --registry https://registry.vais.dev

# Or with verbose output
vaisc pkg publish --registry https://registry.vais.dev --verbose
```

## Installing

Other users can install your package:

```bash
# Install latest version
vaisc pkg install example-package

# Install specific version
vaisc pkg install example-package@0.1.0

# Install with version constraint
vaisc pkg install example-package@^0.1.0
```

## Using in Other Packages

Add to your `vais.toml`:

```toml
[dependencies]
example-package = "^0.1.0"
```

Then use in your code:

```vais
import example_package::{greet, Person}

F main() -> i64 {
    puts(greet("World"))

    let person := Person::new("Alice", 30)
    puts(person.introduce())

    0
}
```

## Version Constraints

- `^1.2.3` - Caret: >=1.2.3, <2.0.0 (recommended)
- `~1.2.3` - Tilde: >=1.2.3, <1.3.0
- `>=1.0.0` - Greater than or equal
- `=1.2.3` - Exact version
- `1.*` - Wildcard
- `>=1.0.0, <2.0.0` - Multiple constraints

## Best Practices

1. **Semantic Versioning**: Follow semver (major.minor.patch)
   - Increment major for breaking changes
   - Increment minor for new features
   - Increment patch for bug fixes

2. **License**: Always specify a license

3. **Description**: Write clear, concise descriptions

4. **Dependencies**: Specify version constraints for stability

5. **Documentation**: Include README and inline comments

6. **Testing**: Use dev-dependencies for test frameworks

## License

MIT
