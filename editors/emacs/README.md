# Vais Emacs Integration

Emacs major mode and LSP integration for the Vais programming language.

## Features

- **Syntax Highlighting**: Full support for Vais single-character keywords and language constructs
- **Automatic Indentation**: Smart indentation based on language structure
- **Comment Support**: Single-line comments using `#`
- **LSP Integration**: Code completion, go-to-definition, diagnostics via `lsp-mode` or `eglot`
- **Code Navigation**: Function and type navigation with imenu
- **Syntax Table**: Proper handling of strings, operators, and delimiters

## Installation

### Prerequisites

- Emacs 25.1 or higher (26.1+ recommended for LSP support)
- Vais compiler installed in your PATH
- Vais LSP server (`vais-lsp`) built and accessible

### Method 1: Manual Installation

1. Clone or copy the `vais-mode.el` and `vais-lsp.el` files to your Emacs configuration directory:

```bash
cp editors/emacs/vais-mode.el ~/.emacs.d/lisp/
cp editors/emacs/vais-lsp.el ~/.emacs.d/lisp/
```

2. Add to your `~/.emacs` or `~/.emacs.d/init.el`:

```elisp
(add-to-list 'load-path "~/.emacs.d/lisp/")
(require 'vais-mode)
(require 'vais-lsp)
```

### Method 2: Using `use-package`

Add to your Emacs configuration:

```elisp
(use-package vais-mode
  :load-path "path/to/vais/editors/emacs"
  :mode "\\.vais\\'"
  :config
  ;; Optional: customize indentation
  (setq vais-indent-offset 4))

(use-package vais-lsp
  :load-path "path/to/vais/editors/emacs"
  :after vais-mode
  :hook (vais-mode . vais-lsp-mode)
  :config
  ;; Optional: customize LSP server path
  (setq vais-lsp-server-path "/path/to/vais-lsp"))
```

### Method 3: Using `straight.el`

Add to your Emacs configuration:

```elisp
(straight-use-package
 '(vais-mode :type git
             :host github
             :repo "your-username/vais"
             :files ("editors/emacs/vais-mode.el"
                     "editors/emacs/vais-lsp.el")))

(use-package vais-mode
  :straight t
  :mode "\\.vais\\'")

(use-package vais-lsp
  :straight t
  :after vais-mode
  :hook (vais-mode . vais-lsp-mode))
```

## LSP Setup

### Using `lsp-mode`

Install `lsp-mode` and configure it for Vais:

```elisp
(use-package lsp-mode
  :ensure t
  :commands (lsp lsp-deferred)
  :hook (vais-mode . lsp-deferred)
  :init
  (setq lsp-keymap-prefix "C-c l")
  :config
  (setq lsp-enable-snippet t
        lsp-enable-symbol-highlighting t))

(use-package lsp-ui
  :ensure t
  :commands lsp-ui-mode
  :config
  (setq lsp-ui-doc-enable t
        lsp-ui-doc-position 'at-point
        lsp-ui-sideline-show-hover t))

(use-package company
  :ensure t
  :hook (vais-mode . company-mode))

(use-package flycheck
  :ensure t
  :hook (vais-mode . flycheck-mode))

;; Load Vais LSP integration
(require 'vais-lsp)
```

### Using `eglot`

`eglot` is simpler and built into Emacs 29+:

```elisp
(use-package eglot
  :ensure t
  :hook (vais-mode . eglot-ensure)
  :config
  ;; Vais LSP is automatically configured via vais-lsp.el
  (add-to-list 'eglot-server-programs
               '(vais-mode . ("vais-lsp"))))

;; Load Vais LSP integration
(require 'vais-lsp)
```

### Custom LSP Server Path

If your LSP server is not in PATH:

```elisp
(setq vais-lsp-server-path "/usr/local/bin/vais-lsp")
;; or
(setq vais-lsp-server-command '("/path/to/vais-lsp" "--stdio"))
```

## Language Features

### Vais Syntax Elements

The mode recognizes and highlights:

#### Keywords (Single-Character)
- `F` - Function definition
- `S` - Struct definition
- `E` - Enum definition
- `I` - If statement
- `L` - Loop
- `M` - Match expression
- `W` - Trait (was While, now Trait)
- `T` - Trait (alternative)
- `X` - Impl block
- `V` - Let binding
- `C` - Const definition
- `R` - Return statement
- `B` - Break
- `N` - Continue (Next)
- `A` - Async

#### Types
- Integers: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- Floats: `f32`, `f64`
- Others: `bool`, `str`, `char`, `void`

#### Special Operators
- `@` - Tail recursion operator
- `:=` - Variable assignment
- `->` - Return type indicator
- `=>` - Match arm separator

#### Comments
- `#` - Single-line comment

### Example Code

```vais
# Function with tail recursion
F factorial(n: i64) -> i64 =
    n < 2 ? 1 : n * @(n - 1)

# Struct definition
S Point {
    x: i64,
    y: i64
}

# Trait definition
W Printable {
    F print(&self) -> i64
}

# Impl block
X Point: Printable {
    F print(&self) -> i64 {
        puts("Point coordinates")
        0
    }
}

# Main function
F main() -> i64 {
    p := Point { x: 10, y: 20 }
    p.print()
    factorial(5)
}
```

## Key Bindings

### Basic Editing
- `RET` - New line and indent
- `TAB` - Indent line
- `C-c C-c` - Comment region
- `C-c C-u` - Uncomment region

### Navigation
- `C-M-a` - Beginning of function
- `C-M-e` - End of function
- `M-g i` or `C-c C-j` - Jump to function/struct with imenu

### LSP (when enabled)
- `C-c l r` - Restart LSP server
- `C-c l v` - Show LSP server version

#### With lsp-mode:
- `C-c l g g` - Go to definition
- `C-c l g r` - Find references
- `C-c l r r` - Rename symbol
- `C-c l h h` - Show hover documentation
- `C-c l =` - Format buffer

#### With eglot:
- `M-.` - Go to definition
- `M-?` - Find references
- `C-c r` - Rename symbol
- `C-c h` - Show hover documentation
- `C-c f` - Format buffer

## Configuration Options

### Customization Variables

```elisp
;; Indentation width (default: 4)
(setq vais-indent-offset 2)

;; LSP server command (default: '("vais-lsp"))
(setq vais-lsp-server-command '("vais-lsp" "--stdio"))

;; LSP server path (default: nil, uses PATH)
(setq vais-lsp-server-path "/custom/path/to/vais-lsp")
```

### Complete Configuration Example

```elisp
;; ~/.emacs.d/init.el

;; Add Vais mode to load path
(add-to-list 'load-path "~/projects/vais/editors/emacs")

;; Load and configure Vais mode
(use-package vais-mode
  :mode "\\.vais\\'"
  :config
  (setq vais-indent-offset 4))

;; LSP with lsp-mode
(use-package lsp-mode
  :ensure t
  :hook (vais-mode . lsp-deferred)
  :commands (lsp lsp-deferred)
  :init
  (setq lsp-keymap-prefix "C-c l")
  :config
  (setq lsp-enable-snippet t
        lsp-enable-symbol-highlighting t
        lsp-signature-auto-activate t
        lsp-signature-render-documentation t))

(use-package lsp-ui
  :ensure t
  :after lsp-mode
  :commands lsp-ui-mode
  :config
  (setq lsp-ui-doc-enable t
        lsp-ui-doc-show-with-cursor t
        lsp-ui-doc-position 'at-point
        lsp-ui-sideline-show-hover t
        lsp-ui-sideline-show-diagnostics t))

;; Code completion
(use-package company
  :ensure t
  :hook (vais-mode . company-mode)
  :config
  (setq company-minimum-prefix-length 1
        company-idle-delay 0.0))

;; Syntax checking
(use-package flycheck
  :ensure t
  :hook (vais-mode . flycheck-mode))

;; Load Vais LSP integration
(require 'vais-lsp)
(add-hook 'vais-mode-hook #'vais-lsp-mode)
```

## Building the LSP Server

If you haven't built the Vais LSP server yet:

```bash
cd /path/to/vais
cargo build --release --bin vais-lsp
# The binary will be at: target/release/vais-lsp
```

Add it to your PATH or configure the path in Emacs:

```elisp
(setq vais-lsp-server-path "/path/to/vais/target/release/vais-lsp")
```

## Troubleshooting

### LSP Server Not Starting

1. Check if the server is in PATH:
   ```bash
   which vais-lsp
   ```

2. Test the server manually:
   ```bash
   vais-lsp --version
   ```

3. Check Emacs logs:
   - For lsp-mode: `M-x lsp-workspace-show-log`
   - For eglot: `M-x eglot-events-buffer`

### Syntax Highlighting Issues

1. Ensure `vais-mode` is loaded:
   ```elisp
   M-x describe-mode
   ```

2. Force reload the mode:
   ```elisp
   M-x revert-buffer
   ```

3. Check font-lock is enabled:
   ```elisp
   M-x font-lock-mode
   ```

### Indentation Problems

1. Check current indentation offset:
   ```elisp
   M-x describe-variable RET vais-indent-offset
   ```

2. Manually adjust indentation:
   ```elisp
   M-x set-variable RET vais-indent-offset RET 2
   ```

## Contributing

Contributions are welcome! Please submit issues and pull requests to the main Vais repository.

## License

This Emacs mode is part of the Vais project and is licensed under the same terms as Vais itself.

## Resources

- [Vais Language Documentation](../../README.md)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [Emacs LSP Mode](https://emacs-lsp.github.io/lsp-mode/)
- [Eglot Documentation](https://joaotavora.github.io/eglot/)
