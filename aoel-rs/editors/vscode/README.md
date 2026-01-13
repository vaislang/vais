# AOEL Language Support for VS Code

Syntax highlighting, LSP support, snippets, and tools for the AOEL programming language.

## Features

- **Syntax Highlighting**: Full TextMate grammar for AOEL syntax
- **Language Server**: Autocompletion, diagnostics, hover information
- **Code Snippets**: Common AOEL patterns and constructs
- **Run & Debug**: Execute AOEL files directly from VS Code
- **Format**: Automatic code formatting

## Installation

### From VSIX (Local)

1. Build the extension:
   ```bash
   cd editors/vscode
   npm install
   npm run compile
   npm run package
   ```

2. Install in VS Code:
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type "Install from VSIX"
   - Select the generated `.vsix` file

### Prerequisites

- [AOEL CLI](../../README.md) installed and in PATH
- Node.js 18+ (for development)

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `aoel.lsp.enabled` | `true` | Enable AOEL Language Server |
| `aoel.lsp.path` | `aoel-lsp` | Path to LSP executable |
| `aoel.format.onSave` | `false` | Format on save |
| `aoel.format.indentWidth` | `4` | Indentation width |
| `aoel.run.jit` | `false` | Use JIT compilation |

## Commands

| Command | Keybinding | Description |
|---------|------------|-------------|
| AOEL: Run File | `Cmd+Shift+R` | Execute current file |
| AOEL: Format File | `Cmd+Shift+F` | Format current file |
| AOEL: Check File | - | Type check current file |
| AOEL: Start REPL | - | Open AOEL REPL |
| AOEL: Show AST | - | Display AST tree |

## Snippets

| Prefix | Description |
|--------|-------------|
| `fn` | Function definition |
| `fnb` | Function with block body |
| `async` | Async function |
| `if` | If-then-else expression |
| `match` | Pattern matching |
| `for` | For loop |
| `map` | Map operation |
| `filter` | Filter operation |
| `reduce` | Reduce operation |
| `pipe` | Pipeline |
| `try` | Try-catch block |
| `type` | Type definition |
| `enum` | Enum definition |

## Development

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch mode
npm run watch

# Package extension
npm run package
```

## License

MIT
