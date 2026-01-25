# Vais Compiler Fuzzing Infrastructure

This directory contains fuzzing infrastructure for the Vais compiler using cargo-fuzz (libFuzzer).

## Prerequisites

1. **Rust nightly toolchain**:
   ```bash
   rustup install nightly
   ```

2. **cargo-fuzz**:
   ```bash
   cargo install cargo-fuzz
   ```

## Fuzz Targets

| Target | Description | Input |
|--------|-------------|-------|
| `fuzz_lexer` | Lexer/tokenizer | Raw source code |
| `fuzz_parser` | Parser | Raw source code |
| `fuzz_type_checker` | Type checker | Valid parsed AST |
| `fuzz_codegen` | Code generator | Valid typed AST |
| `fuzz_full_pipeline` | Full compiler pipeline | Structured program |

## Quick Start

Run a specific fuzzer:
```bash
cd fuzz
cargo +nightly fuzz run fuzz_lexer
```

Run with a time limit (300 seconds):
```bash
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=300
```

Run with dictionary:
```bash
cargo +nightly fuzz run fuzz_lexer -- -dict=dictionaries/vais.dict
```

## Corpus Management

Seed corpus is located in `corpus/<target>/`. Add interesting inputs to improve coverage:

```bash
# Add a new seed file
echo 'F hello() -> str = "world"' > corpus/fuzz_lexer/new_seed.vais

# Minimize corpus after fuzzing
cargo +nightly fuzz cmin fuzz_lexer
```

## Viewing Crashes

Crash inputs are saved to `artifacts/<target>/`:
```bash
# List crashes
ls artifacts/fuzz_lexer/

# Reproduce a crash
cargo +nightly fuzz run fuzz_lexer artifacts/fuzz_lexer/crash-*
```

## Sanitizer Testing

Run tests with Address Sanitizer:
```bash
../scripts/run-sanitizers.sh address
```

Run tests with Undefined Behavior Sanitizer:
```bash
../scripts/run-sanitizers.sh undefined
```

Run both ASAN and UBSAN:
```bash
../scripts/run-sanitizers.sh all
```

## Coverage

Generate coverage report:
```bash
cargo +nightly fuzz coverage fuzz_lexer
```

## OSS-Fuzz Integration

Files for OSS-Fuzz integration are in `oss-fuzz/`:
- `project.yaml` - Project configuration
- `Dockerfile` - Build environment
- `build.sh` - Build script

To submit to OSS-Fuzz:
1. Fork https://github.com/google/oss-fuzz
2. Copy `oss-fuzz/` contents to `projects/vais/`
3. Submit a pull request

## Directory Structure

```
fuzz/
├── Cargo.toml              # Fuzz workspace
├── README.md               # This file
├── fuzz_targets/           # Fuzz target source code
│   ├── fuzz_lexer.rs
│   ├── fuzz_parser.rs
│   ├── fuzz_type_checker.rs
│   ├── fuzz_codegen.rs
│   └── fuzz_full_pipeline.rs
├── corpus/                 # Seed corpus
│   ├── fuzz_lexer/
│   ├── fuzz_parser/
│   ├── fuzz_type_checker/
│   └── fuzz_full_pipeline/
├── artifacts/              # Crash inputs (auto-generated)
├── dictionaries/           # Fuzzing dictionaries
│   └── vais.dict
└── oss-fuzz/               # OSS-Fuzz integration
    ├── project.yaml
    ├── Dockerfile
    └── build.sh
```

## Troubleshooting

### "error: linker `cc` not found"
Install build essentials:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install
```

### "error: could not compile `libfuzzer-sys`"
Ensure nightly toolchain is being used:
```bash
rustup default nightly
# or
cargo +nightly fuzz run ...
```

### Memory issues
Limit memory usage:
```bash
cargo +nightly fuzz run fuzz_lexer -- -rss_limit_mb=2048
```
