-- Vais Language Server Configuration for Neovim
-- This file provides an example configuration for using the Vais Language Server with nvim-lspconfig

-- Prerequisites:
-- 1. Install nvim-lspconfig: https://github.com/neovim/nvim-lspconfig
-- 2. Build the Vais language server (vais-lsp) and ensure it's in your PATH

-- Add this to your Neovim configuration (e.g., ~/.config/nvim/init.lua or ~/.config/nvim/lua/lsp.lua)

local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define Vais LSP configuration if not already defined
if not configs.vais_lsp then
  configs.vais_lsp = {
    default_config = {
      -- Command to start the Vais language server
      -- Adjust the path if vais-lsp is not in your PATH
      cmd = { 'vais-lsp' },

      -- File types that trigger the LSP
      filetypes = { 'vais' },

      -- Root directory patterns (where the LSP should start)
      -- Looks for common project markers
      root_dir = lspconfig.util.root_pattern(
        'Cargo.toml',
        'vais.toml',
        '.git',
        'vais.json'
      ),

      -- LSP server settings
      settings = {
        vais = {
          -- Enable/disable diagnostics
          diagnostics = {
            enable = true,
          },
          -- Enable/disable completion
          completion = {
            enable = true,
          },
          -- Enable/disable hover information
          hover = {
            enable = true,
          },
        },
      },

      -- Initial options passed to the server
      init_options = {
        -- Add any initialization options here
      },
    },
  }
end

-- Setup the Vais LSP with default configuration
lspconfig.vais_lsp.setup({
  -- Optional: Add custom on_attach function for key mappings
  on_attach = function(client, bufnr)
    -- Enable completion triggered by <c-x><c-o>
    vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

    -- Buffer-local keymaps
    local bufopts = { noremap = true, silent = true, buffer = bufnr }

    -- Go to definition
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, bufopts)

    -- Go to declaration
    vim.keymap.set('n', 'gD', vim.lsp.buf.declaration, bufopts)

    -- Show hover information
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, bufopts)

    -- Go to implementation
    vim.keymap.set('n', 'gi', vim.lsp.buf.implementation, bufopts)

    -- Show signature help
    vim.keymap.set('n', '<C-k>', vim.lsp.buf.signature_help, bufopts)

    -- Add workspace folder
    vim.keymap.set('n', '<space>wa', vim.lsp.buf.add_workspace_folder, bufopts)

    -- Remove workspace folder
    vim.keymap.set('n', '<space>wr', vim.lsp.buf.remove_workspace_folder, bufopts)

    -- List workspace folders
    vim.keymap.set('n', '<space>wl', function()
      print(vim.inspect(vim.lsp.buf.list_workspace_folders()))
    end, bufopts)

    -- Go to type definition
    vim.keymap.set('n', '<space>D', vim.lsp.buf.type_definition, bufopts)

    -- Rename symbol
    vim.keymap.set('n', '<space>rn', vim.lsp.buf.rename, bufopts)

    -- Code action
    vim.keymap.set('n', '<space>ca', vim.lsp.buf.code_action, bufopts)

    -- Find references
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, bufopts)

    -- Format buffer
    vim.keymap.set('n', '<space>f', function()
      vim.lsp.buf.format({ async = true })
    end, bufopts)
  end,

  -- Optional: Add custom capabilities (e.g., for nvim-cmp)
  capabilities = require('cmp_nvim_lsp').default_capabilities(
    vim.lsp.protocol.make_client_capabilities()
  ),

  -- Optional: Custom flags
  flags = {
    -- Debounce text changes (in ms)
    debounce_text_changes = 150,
  },
})

-- Optional: Configure diagnostic display
vim.diagnostic.config({
  virtual_text = true,
  signs = true,
  underline = true,
  update_in_insert = false,
  severity_sort = true,
})

-- Optional: Define diagnostic signs
local signs = { Error = "✘", Warn = "▲", Hint = "⚑", Info = "»" }
for type, icon in pairs(signs) do
  local hl = "DiagnosticSign" .. type
  vim.fn.sign_define(hl, { text = icon, texthl = hl, numhl = hl })
end

-- Example: Auto-format on save (optional)
-- vim.api.nvim_create_autocmd("BufWritePre", {
--   pattern = "*.vais",
--   callback = function()
--     vim.lsp.buf.format({ async = false })
--   end,
-- })

-- Note: Make sure to load this configuration in your init.lua or init.vim
-- For init.lua: require('lsp')
-- For init.vim: lua require('lsp')
