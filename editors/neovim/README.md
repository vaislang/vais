# Vais Language Support for Neovim

This directory contains Neovim integration files for the Vais programming language, providing syntax highlighting, filetype detection, and LSP configuration.

## Files

- `syntax/vais.vim` - Vim syntax highlighting for Vais
- `ftdetect/vais.vim` - Filetype detection for `.vais` files
- `ftplugin/vais.vim` - Filetype-specific settings (indentation, comments, etc.)
- `lsp.lua` - LSP configuration example for nvim-lspconfig

## Installation

### Method 1: Manual Installation

Copy the files to your Neovim runtime path:

```bash
# Create directories if they don't exist
mkdir -p ~/.config/nvim/syntax
mkdir -p ~/.config/nvim/ftdetect
mkdir -p ~/.config/nvim/ftplugin

# Copy the files
cp syntax/vais.vim ~/.config/nvim/syntax/
cp ftdetect/vais.vim ~/.config/nvim/ftdetect/
cp ftplugin/vais.vim ~/.config/nvim/ftplugin/
```

### Method 2: Using a Plugin Manager

#### Using lazy.nvim

```lua
{
  dir = "/path/to/vais/editors/neovim",
  ft = "vais",
}
```

#### Using packer.nvim

```lua
use {
  "/path/to/vais/editors/neovim",
  ft = "vais",
}
```

#### Using vim-plug

```vim
Plug '/path/to/vais/editors/neovim'
```

## Language Server Setup

### Prerequisites

1. Build the Vais language server:
   ```bash
   cd /path/to/vais
   cargo build --release --bin vais-lsp
   ```

2. Add the language server to your PATH:
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="/path/to/vais/target/release:$PATH"
   ```

3. Install [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig):
   ```lua
   -- Using lazy.nvim
   { "neovim/nvim-lspconfig" }
   ```

### LSP Configuration

Copy the LSP configuration to your Neovim config:

```bash
cp lsp.lua ~/.config/nvim/lua/vais_lsp.lua
```

Then load it in your `init.lua`:

```lua
require('vais_lsp')
```

Or, if you prefer to integrate it directly into your existing LSP configuration, see the contents of `lsp.lua` for the configuration details.

## Features

### Syntax Highlighting

The syntax file highlights:
- **Keywords**: F, S, E, I, L, M, W, T, X, V, C, R, B, N, A (single character keywords)
- **Control flow**: if, else, match, loop, while, for, break, continue, return
- **Types**: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, str
- **Operators**: @, :=, ->, =>, ?, and standard operators
- **Comments**: # line comments
- **Strings**: Double and single quoted strings with escape sequences
- **Numbers**: Decimal, hex (0x), binary (0b), octal (0o), and floating point

### Filetype Settings

The ftplugin provides:
- 2-space indentation (tabs expanded to spaces)
- Comment string configuration for easy commenting
- Smart indentation for nested structures
- Format options optimized for code editing
- Fold settings based on indentation

### LSP Features (when configured)

- Code completion
- Go to definition
- Find references
- Hover documentation
- Signature help
- Rename refactoring
- Code actions
- Diagnostics

## Key Mappings (LSP)

When the LSP is active, the following key mappings are available:

| Key          | Action                    |
|--------------|---------------------------|
| `gd`         | Go to definition          |
| `gD`         | Go to declaration         |
| `K`          | Show hover information    |
| `gi`         | Go to implementation      |
| `<C-k>`      | Signature help            |
| `gr`         | Find references           |
| `<space>rn`  | Rename symbol             |
| `<space>ca`  | Code actions              |
| `<space>f`   | Format buffer             |
| `<space>D`   | Go to type definition     |

## Customization

### Changing Indentation

Edit `ftplugin/vais.vim` and modify these lines:

```vim
setlocal shiftwidth=4     " Change from 2 to 4 spaces
setlocal softtabstop=4
setlocal tabstop=4
```

### Disabling Auto-formatting

Comment out the auto-format on save section in `lsp.lua`:

```lua
-- vim.api.nvim_create_autocmd("BufWritePre", {
--   pattern = "*.vais",
--   callback = function()
--     vim.lsp.buf.format({ async = false })
--   end,
-- })
```

## Testing

To test the installation:

1. Open a `.vais` file in Neovim
2. Check that syntax highlighting is working
3. Run `:set filetype?` - it should show `filetype=vais`
4. Run `:LspInfo` - it should show the Vais LSP server (if configured)

## Troubleshooting

### Syntax highlighting not working

- Ensure files are in the correct runtime path
- Run `:scriptnames` to verify the syntax file is loaded
- Try `:set filetype=vais` manually

### LSP not starting

- Check that `vais-lsp` is in your PATH: `which vais-lsp`
- View LSP logs: `:LspLog`
- Check LSP status: `:LspInfo`
- Verify the language server binary works: `vais-lsp --version`

### Wrong indentation

- Check your global settings don't override ftplugin settings
- Verify the ftplugin is loaded: `:verbose set shiftwidth?`

## Contributing

If you find issues or want to improve the Neovim integration, please submit issues or pull requests to the Vais repository.

## License

This integration follows the same license as the Vais language project.
