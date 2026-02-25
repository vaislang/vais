# Vais Language Support for IntelliJ IDEA

IntelliJ IDEA plugin providing language support for the Vais programming language.

## Features

- **Syntax Highlighting**: Full syntax highlighting for Vais source files
- **LSP Integration**: Leverages vais-lsp for intelligent features:
  - Code completion
  - Go to definition
  - Find references
  - Hover information
  - Error diagnostics
  - Code formatting

## Requirements

- IntelliJ IDEA 2023.3 or later
- `vais-lsp` binary in PATH or one of the following locations:
  - `~/.cargo/bin/vais-lsp`
  - `/usr/local/bin/vais-lsp`
  - `/opt/homebrew/bin/vais-lsp`
  - Project's `target/release/vais-lsp` or `target/debug/vais-lsp`

## Installation

### From JetBrains Marketplace (Coming Soon)

1. Open IntelliJ IDEA
2. Go to Settings/Preferences → Plugins → Marketplace
3. Search for "Vais Language Support"
4. Click Install

### From Source

```bash
# Clone the repository
git clone https://github.com/vaislang/vais.git
cd vais/intellij-vais

# Build the plugin
./gradlew buildPlugin

# The plugin ZIP will be in build/distributions/
```

Install the plugin ZIP via Settings → Plugins → ⚙️ → Install Plugin from Disk...

## Building vais-lsp

The LSP server is required for intelligent features:

```bash
cd ../crates/vais-lsp
cargo build --release

# Install to cargo bin
cargo install --path .
```

## Development

### Prerequisites

- JDK 17 or later
- Gradle 8.5 or later

### Building

```bash
./gradlew build
```

### Running in Development IDE

```bash
./gradlew runIde
```

### Publishing

```bash
# Set environment variables
export PUBLISH_TOKEN="your-jetbrains-token"

./gradlew publishPlugin
```

## Vais Language Overview

Vais uses single-character keywords for conciseness:

| Keyword | Meaning |
|---------|---------|
| `F` | Function definition |
| `S` | Struct definition |
| `E` | Enum / Else |
| `I` | If statement |
| `L` | Loop |
| `M` | Match |
| `W` | Trait (interface) |
| `T` | Type alias |
| `X` | Implementation block |
| `P` | Pub (public visibility) |
| `C` | Continue |
| `R` | Return |
| `B` | Break |
| `N` | Extern (foreign function) |
| `U` | Use (import) |
| `D` | Defer |
| `O` | Union |
| `G` | Global variable |
| `A` | Async |
| `Y` | Await |

Example:
```vais
# Function definition
F add(a: i64, b: i64) -> i64 = a + b

# Struct with methods
S Point {
    x: f64,
    y: f64
}

# Main function
F main() -> i64 {
    p := Point { x: 1.0, y: 2.0 }
    puts("Hello, Vais!")
    0
}
```

## License

MIT License - see the main Vais repository for details.
