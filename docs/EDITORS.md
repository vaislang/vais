# Vais Editor Integration Guide

This document provides setup instructions for using Vais with various text editors and IDEs.

## Overview

Vais provides editor support through:
- **LSP Server** (`vais-lsp`) - Language Server Protocol for IDE features
- **Syntax Highlighting** - TextMate grammar and editor-specific highlighting
- **Extensions/Plugins** - Editor-specific integration packages

## Quick Start

### Build the LSP Server

First, build the Vais LSP server:

```bash
cd /path/to/vais
cargo build --release --bin vais-lsp

# Add to PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

## Supported Editors

| Editor | Status | LSP | Syntax | Setup Complexity |
|--------|--------|-----|--------|------------------|
| [VS Code](#visual-studio-code) | Full Support | Yes | Yes | Easy |
| [Neovim](#neovim) | Full Support | Yes | Yes | Medium |
| [Helix](#helix) | Full Support | Yes | Yes | Easy |
| [Emacs](#emacs) | Full Support | Yes | Yes | Medium |

---

## Visual Studio Code

VS Code has the most complete integration through the official extension.

### Installation

1. **From Marketplace** (coming soon):
   ```
   ext install vais-vscode
   ```

2. **Manual Installation**:
   ```bash
   cd vscode-vais
   npm install
   npm run compile
   code --install-extension vais-vscode-0.2.0.vsix
   ```

### Features
- Full syntax highlighting
- LSP integration (completion, hover, go-to-definition)
- Real-time diagnostics
- Semantic tokens
- Code actions and quick fixes
- Inlay hints
- Call hierarchy

### Configuration

Open VS Code settings and configure:

```json
{
  "vais.languageServer.path": "/path/to/vais-lsp",
  "vais.trace.server": "verbose"
}
```

---

## Neovim

Neovim integration uses native Vim syntax and nvim-lspconfig.

### Installation

**Option 1: Automated Installation**

```bash
cd editors/neovim
chmod +x install.sh
./install.sh
```

**Option 2: Manual Installation**

```bash
# Copy syntax files
mkdir -p ~/.config/nvim/syntax
mkdir -p ~/.config/nvim/ftdetect
mkdir -p ~/.config/nvim/ftplugin

cp editors/neovim/syntax/vais.vim ~/.config/nvim/syntax/
cp editors/neovim/ftdetect/vais.vim ~/.config/nvim/ftdetect/
cp editors/neovim/ftplugin/vais.vim ~/.config/nvim/ftplugin/
```

**Option 3: Plugin Manager (lazy.nvim)**

```lua
{
  dir = "/path/to/vais/editors/neovim",
  ft = "vais",
}
```

### LSP Setup

Add to your Neovim config (init.lua):

```lua
-- Basic LSP setup
local lspconfig = require('lspconfig')

local configs = require('lspconfig.configs')
if not configs.vais_lsp then
  configs.vais_lsp = {
    default_config = {
      cmd = { 'vais-lsp' },
      filetypes = { 'vais' },
      root_dir = lspconfig.util.root_pattern('.git', 'Cargo.toml'),
      settings = {},
    },
  }
end

lspconfig.vais_lsp.setup({
  on_attach = function(client, bufnr)
    -- Key mappings
    local opts = { noremap = true, silent = true, buffer = bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
  end,
})
```

See `editors/neovim/lsp.lua` for complete configuration with all features.

### Key Bindings (with LSP)

| Key | Action |
|-----|--------|
| `gd` | Go to definition |
| `K` | Show hover info |
| `gr` | Find references |
| `<leader>rn` | Rename symbol |
| `<leader>ca` | Code actions |
| `[d` / `]d` | Previous/next diagnostic |

---

## Helix

Helix provides built-in LSP support with minimal configuration.

### Installation

```bash
# Copy language configuration
cp editors/helix/languages.toml ~/.config/helix/languages.toml

# Copy syntax queries (optional, for enhanced highlighting)
mkdir -p ~/.config/helix/runtime/queries/vais
cp editors/helix/queries/vais/highlights.scm ~/.config/helix/runtime/queries/vais/
```

Or merge with existing `languages.toml`:

```toml
[[language]]
name = "vais"
scope = "source.vais"
injection-regex = "vais"
file-types = ["vais"]
comment-tokens = "#"
indent = { tab-width = 4, unit = "    " }
language-servers = ["vais-lsp"]

[language-server.vais-lsp]
command = "vais-lsp"
```

### Features

All features work automatically after setup:
- Syntax highlighting
- Auto-completion (Ctrl+Space)
- Hover documentation (Space+k)
- Go to definition (gd)
- Find references (gr)
- Rename (Space+r)
- Code actions (Space+a)

### Key Bindings

| Key | Action |
|-----|--------|
| `gd` | Go to definition |
| `gr` | Go to references |
| `Space+k` | Hover documentation |
| `Space+r` | Rename |
| `Space+a` | Code actions |
| `Ctrl+Space` | Completion |

---

## Emacs

Emacs integration provides a full major mode with LSP support.

### Installation

**Option 1: use-package**

```elisp
(use-package vais-mode
  :load-path "/path/to/vais/editors/emacs"
  :mode "\\.vais\\'"
  :custom
  (vais-indent-offset 4))

(use-package vais-lsp
  :load-path "/path/to/vais/editors/emacs"
  :after (vais-mode lsp-mode)
  :hook (vais-mode . lsp-deferred)
  :custom
  (vais-lsp-server-path "/path/to/vais-lsp"))
```

**Option 2: straight.el**

```elisp
(straight-use-package
 '(vais-mode :type git :local-repo "/path/to/vais/editors/emacs"))
```

**Option 3: Manual**

```elisp
(add-to-list 'load-path "/path/to/vais/editors/emacs")
(require 'vais-mode)
(require 'vais-lsp)
```

### LSP Setup

**With lsp-mode:**

```elisp
(use-package lsp-mode
  :hook (vais-mode . lsp-deferred)
  :commands lsp)

;; vais-lsp.el automatically registers the server
```

**With eglot (Emacs 29+):**

```elisp
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(vais-mode . ("vais-lsp"))))

(add-hook 'vais-mode-hook 'eglot-ensure)
```

### Features

- Full syntax highlighting (font-lock)
- Smart indentation
- Comment handling (`#` single-line)
- Imenu integration (functions, structs, enums, traits)
- LSP features via lsp-mode or eglot

### Key Bindings

| Key | Action |
|-----|--------|
| `C-c C-c` | Comment region |
| `C-c C-u` | Uncomment region |
| `C-M-a` | Beginning of function |
| `C-M-e` | End of function |
| `M-.` | Go to definition (LSP) |
| `M-?` | Find references (LSP) |

---

## LSP Features Reference

The Vais LSP server (`vais-lsp`) provides these capabilities:

| Feature | Description |
|---------|-------------|
| **Diagnostics** | Real-time error detection |
| **Completion** | Keywords, types, functions, methods |
| **Hover** | Function signatures, documentation |
| **Definition** | Jump to symbol definition |
| **References** | Find all symbol usages |
| **Rename** | Rename symbols across files |
| **Code Actions** | Quick fixes and refactorings |
| **Semantic Tokens** | Enhanced syntax highlighting |
| **Inlay Hints** | Type inference annotations |
| **Folding** | Code folding for functions/blocks |
| **Call Hierarchy** | Function call relationships |
| **Document Symbols** | Outline view |

### Code Actions

Available quick fixes:
1. **Create variable** - For undefined variables
2. **Import module** - Auto-import standard library
3. **Type cast** - Fix type mismatches
4. **Extract variable** - Extract expression to variable
5. **Extract function** - Extract selection to function

---

## Troubleshooting

### LSP Server Not Starting

1. **Check if server is built:**
   ```bash
   ls -la target/release/vais-lsp
   ```

2. **Check if server is in PATH:**
   ```bash
   which vais-lsp
   ```

3. **Run server manually to check for errors:**
   ```bash
   vais-lsp 2>&1 | head -20
   ```

### Syntax Highlighting Not Working

1. **Verify file extension:** Must be `.vais`
2. **Check filetype detection:**
   - Neovim: `:set ft?` should show `vais`
   - Emacs: `M-x describe-mode` should show `Vais`
3. **Reload syntax files after installation**

### Completion Not Triggering

1. **Verify LSP is connected:**
   - Neovim: `:LspInfo`
   - VS Code: Check status bar
   - Emacs: `M-x lsp-describe-session`
2. **Check trigger characters:** `.` and `:` trigger completion
3. **Manual trigger:** Use Ctrl+Space or your editor's completion key

### Performance Issues

1. **Large files:** LSP may be slow on files >10,000 lines
2. **Enable incremental sync** if available
3. **Check for excessive logging:** Disable trace/verbose modes

---

## Debug Adapter Protocol (DAP)

Vais includes a Debug Adapter Protocol server (`vais-dap`) for IDE-level debugging support.

### Building the DAP Server

```bash
cargo build --release --bin vais-dap

# Add to PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

### Features

- Source-level debugging with breakpoints
- Step over, step into, step out
- Local variables and arguments inspection
- Register inspection
- Memory read/write
- Disassembly view
- Conditional breakpoints
- Function breakpoints
- Exception breakpoints (panic)
- Expression evaluation

### VS Code Debugging

1. Install the Vais extension
2. Create a launch configuration in `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "vais",
      "request": "launch",
      "name": "Debug Vais Program",
      "program": "${workspaceFolder}/main.vais",
      "stopOnEntry": true,
      "autoCompile": true,
      "optLevel": 0
    }
  ]
}
```

3. Set breakpoints by clicking in the gutter
4. Press F5 to start debugging

### Neovim Debugging

With nvim-dap:

```lua
local dap = require('dap')

dap.adapters.vais = {
  type = 'executable',
  command = 'vais-dap',
}

dap.configurations.vais = {
  {
    type = 'vais',
    request = 'launch',
    name = 'Debug Vais Program',
    program = '${file}',
    stopOnEntry = true,
    autoCompile = true,
  }
}
```

### Emacs Debugging

With dap-mode:

```elisp
(require 'dap-mode)

(dap-register-debug-template
 "Vais Debug"
 (list :type "vais"
       :request "launch"
       :name "Debug Vais"
       :program nil  ; Will prompt for file
       :stopOnEntry t
       :autoCompile t))

;; Or use dap-debug directly
(defun vais-debug ()
  "Debug current Vais file."
  (interactive)
  (dap-debug
   (list :type "vais"
         :request "launch"
         :name "Debug"
         :program (buffer-file-name)
         :stopOnEntry t
         :autoCompile t)))
```

### CLI Usage

```bash
# Start DAP server (stdio mode, for IDE integration)
vais-dap

# Start DAP server on TCP port (for remote debugging)
vais-dap --port 4711

# With verbose logging
vais-dap --log-level debug
```

### Compile with Debug Info

To enable source-level debugging, compile with the `-g` flag:

```bash
vaisc build main.vais -g -O0
```

---

## Contributing

Found an issue or want to add support for another editor?

1. Open an issue at https://github.com/vais-lang/vais/issues
2. PRs welcome for new editor integrations
3. Follow existing patterns in `editors/` directory

---

## See Also

- [Language Specification](LANGUAGE_SPEC.md)
- [Standard Library Reference](STDLIB.md)
- [Tutorial](TUTORIAL.md)
