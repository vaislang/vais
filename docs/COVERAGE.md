# Test Coverage Guide

This document describes how to measure and track test coverage for the Vais project.

## Overview

The Vais project uses **cargo-tarpaulin** for measuring test coverage. Tarpaulin is a code coverage tool that works with Rust projects and provides multiple output formats:

- **HTML** - Interactive web-based coverage report
- **Lcov** - Standard coverage format compatible with codecov and other CI/CD tools
- **Stdout** - Console output for quick checks

## Configuration

### tarpaulin.toml

The main configuration file is `/tarpaulin.toml` at the project root. Key settings:

```toml
# Output formats
out = ["Stdout", "Html", "Lcov"]

# Output directory
output-dir = "target/coverage"

# Excluded files/packages
exclude-files = ["benches/*", "examples/*", "tests/*"]
exclude = ["vais-benches"]

# Enable parallel execution for faster runs
parallel = false

# Fail if coverage falls below threshold (0 = no minimum)
fail-under = 0
```

### .cargo/config.toml

Convenient cargo aliases are configured in `.cargo/config.toml`:

```toml
# Generate coverage with all formats
cargo coverage

# Generate HTML only
cargo coverage-html

# Generate Lcov only
cargo coverage-lcov
```

## Local Coverage Measurement

### Prerequisites

Install cargo-tarpaulin:

```bash
cargo install cargo-tarpaulin
```

### Generate Coverage Reports

Using the convenience script:

```bash
# Generate all reports (HTML + Lcov)
./scripts/coverage.sh all

# Generate HTML only
./scripts/coverage.sh html

# Generate Lcov only
./scripts/coverage.sh lcov

# View HTML report in browser
./scripts/coverage.sh view

# Clean up coverage files
./scripts/coverage.sh clean
```

Using cargo aliases directly:

```bash
# All formats (configured via tarpaulin.toml)
cargo coverage

# HTML only
cargo coverage-html

# Lcov only
cargo coverage-lcov
```

Using cargo-tarpaulin directly:

```bash
# Basic usage
cargo tarpaulin --config tarpaulin.toml

# HTML output
cargo tarpaulin --config tarpaulin.toml --out Html --output-dir target/coverage

# Lcov output
cargo tarpaulin --config tarpaulin.toml --out Lcov --output-dir target/coverage

# Multiple formats
cargo tarpaulin --config tarpaulin.toml --out Html --out Lcov --output-dir target/coverage

# Verbose output
cargo tarpaulin --config tarpaulin.toml --verbose

# Specific package
cargo tarpaulin -p vais-lexer --config tarpaulin.toml
```

## Report Interpretation

### HTML Report

Open `target/coverage/index.html` in your browser to view:

- **Summary** - Overall coverage percentages
- **File List** - Individual file coverage statistics
- **Code View** - Line-by-line coverage visualization
  - Green - Lines covered by tests
  - Red - Lines not covered
  - Orange - Lines partially covered (conditionals)

### Coverage Metrics

- **Lines** - Percentage of executable lines covered
- **Branches** - Percentage of branch conditions covered
- **Functions** - Percentage of functions called in tests

## CI/CD Integration

### GitHub Actions

Coverage is automatically measured in the CI pipeline:

```yaml
coverage:
  name: Test Coverage
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo install cargo-tarpaulin
    - name: Generate reports
      run: cargo tarpaulin --config tarpaulin.toml --out Html --out Lcov --output-dir target/coverage
    - uses: actions/upload-artifact@v4
      with:
        name: coverage-reports
        path: target/coverage/
    - uses: codecov/codecov-action@v3
      with:
        files: ./target/coverage/lcov.info
```

### Codecov Integration

For codecov.io integration:

1. Get your repository token from [codecov.io](https://codecov.io)
2. Add to GitHub secrets as `CODECOV_TOKEN` (optional for public repos)
3. The CI workflow automatically uploads Lcov reports
4. View trends at: `https://codecov.io/gh/sswoo88/vais`

## Setting Coverage Thresholds

To enforce minimum coverage requirements:

```toml
# In tarpaulin.toml
fail-under = 70  # Fail if coverage drops below 70%
```

Or via command line:

```bash
cargo tarpaulin --fail-under 70
```

## Performance Considerations

### Measurement Overhead

Coverage measurement adds overhead:
- **Time**: 2-3x slower than regular test runs
- **Memory**: Slightly higher memory usage
- **I/O**: Writes coverage data during execution

### Optimization Tips

1. **Parallel execution** (when supported):
   ```bash
   cargo tarpaulin --parallel
   ```

2. **Specific packages**:
   ```bash
   cargo tarpaulin -p vais-lexer
   ```

3. **Exclude unnecessary code**:
   - Examples: `exclude-files = ["examples/*"]`
   - Benchmarks: `exclude = ["vais-benches"]`

4. **Release builds** (faster):
   ```bash
   cargo tarpaulin --release
   ```

## Troubleshooting

### "No coverage data generated"

**Problem**: Tarpaulin runs but generates no coverage files.

**Solutions**:
- Ensure LLVM is installed: `llvm-config --version`
- Check tarpaulin compatibility: `cargo install cargo-tarpaulin --force`
- Try with specific package: `cargo tarpaulin -p vais-lexer`

### "Tests fail during coverage measurement"

**Problem**: Tests pass normally but fail under coverage.

**Solutions**:
- Set `ignore-panics = true` in tarpaulin.toml
- Use `ignore-tests = false` to verify test detection
- Check for tests that depend on specific timing or environment

### "Coverage report is incomplete"

**Problem**: Some files missing from coverage report.

**Solutions**:
- Verify files contain tests or are called by tests
- Check exclude patterns in tarpaulin.toml
- Use `--all` flag to include all code

## Best Practices

1. **Measure regularly** - Run coverage before major releases
2. **Track trends** - Use codecov to monitor coverage over time
3. **Set goals** - Aim for >80% line coverage for core modules
4. **Focus on critical paths** - Prioritize testing business logic
5. **Document uncovered code** - Use comments to explain intentional gaps

## Related Documentation

- [cargo-tarpaulin GitHub](https://github.com/xd009642/tarpaulin)
- [codecov Documentation](https://docs.codecov.io/)
- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Project contribution guidelines
