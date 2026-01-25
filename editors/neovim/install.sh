#!/bin/bash
# Installation script for Vais Neovim integration

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Default Neovim config directory
NVIM_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/nvim"

echo -e "${GREEN}Vais Neovim Integration Installer${NC}"
echo "=================================="
echo ""

# Check if Neovim config directory exists
if [ ! -d "$NVIM_CONFIG" ]; then
    echo -e "${YELLOW}Warning: Neovim config directory not found at $NVIM_CONFIG${NC}"
    read -p "Create it? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        mkdir -p "$NVIM_CONFIG"
        echo -e "${GREEN}Created $NVIM_CONFIG${NC}"
    else
        echo -e "${RED}Installation cancelled.${NC}"
        exit 1
    fi
fi

# Create necessary directories
echo "Creating directories..."
mkdir -p "$NVIM_CONFIG/syntax"
mkdir -p "$NVIM_CONFIG/ftdetect"
mkdir -p "$NVIM_CONFIG/ftplugin"
mkdir -p "$NVIM_CONFIG/lua"

# Copy syntax file
echo "Installing syntax highlighting..."
cp "$SCRIPT_DIR/syntax/vais.vim" "$NVIM_CONFIG/syntax/"
echo -e "${GREEN}✓${NC} Installed syntax/vais.vim"

# Copy ftdetect file
echo "Installing filetype detection..."
cp "$SCRIPT_DIR/ftdetect/vais.vim" "$NVIM_CONFIG/ftdetect/"
echo -e "${GREEN}✓${NC} Installed ftdetect/vais.vim"

# Copy ftplugin file
echo "Installing filetype plugin..."
cp "$SCRIPT_DIR/ftplugin/vais.vim" "$NVIM_CONFIG/ftplugin/"
echo -e "${GREEN}✓${NC} Installed ftplugin/vais.vim"

# Ask about LSP configuration
echo ""
read -p "Install LSP configuration? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cp "$SCRIPT_DIR/lsp.lua" "$NVIM_CONFIG/lua/vais_lsp.lua"
    echo -e "${GREEN}✓${NC} Installed lua/vais_lsp.lua"
    echo ""
    echo -e "${YELLOW}Note:${NC} Add the following line to your init.lua to enable LSP:"
    echo "  require('vais_lsp')"
    echo ""
    echo "Or for init.vim:"
    echo "  lua require('vais_lsp')"
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Files installed to: $NVIM_CONFIG"
echo ""
echo "Next steps:"
echo "1. Make sure vais-lsp is built and in your PATH"
echo "2. Install nvim-lspconfig if you haven't already"
echo "3. Open a .vais file in Neovim to test"
echo ""
echo "For more information, see: $SCRIPT_DIR/README.md"
