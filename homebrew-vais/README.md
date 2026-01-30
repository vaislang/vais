# Homebrew Tap for Vais

This is the official Homebrew tap for the Vais programming language compiler.

## What is Vais?

Vais is an AI-optimized systems programming language with an LLVM backend. It combines modern language features with performance and safety, designed to be easy for AI systems to understand and generate.

## Installation

### Prerequisites

- macOS with Homebrew installed
- LLVM 17 (automatically installed as a dependency)

### Install from this tap

```bash
# Add the tap
brew tap sswoo88/vais

# Install Vais
brew install vais
```

### Install from source (HEAD)

To install the latest development version:

```bash
brew install --HEAD vais
```

## Usage

After installation, you can use the `vaisc` compiler:

```bash
# Show help
vaisc --help

# Start the REPL
vaisc repl

# Compile a Vais program
vaisc compile program.vais

# Run a Vais program
vaisc run program.vais
```

## Standard Library

The Vais standard library is automatically installed to:
```
$(brew --prefix)/share/vais/std/
```

The compiler will automatically find the standard library location.

## Updating

To update to the latest version:

```bash
brew update
brew upgrade vais
```

## Uninstalling

```bash
brew uninstall vais
brew untap sswoo88/vais
```

## Links

- [GitHub Repository](https://github.com/sswoo88/vais)
- [Documentation](https://github.com/sswoo88/vais/tree/main/docs)
- [Examples](https://github.com/sswoo88/vais/tree/main/examples)

## Support

For issues, feature requests, or questions:
- [Issue Tracker](https://github.com/sswoo88/vais/issues)
- [Discussions](https://github.com/sswoo88/vais/discussions)

## License

The Vais programming language is licensed under the MIT License.
