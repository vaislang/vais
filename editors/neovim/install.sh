#!/bin/bash
# Installation script for Vais Neovim integration
# Supports: nvim-lspconfig, coc.nvim, UltiSnips

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
VAIS_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

# Default Neovim config directory
NVIM_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/nvim"

echo -e "${CYAN}Vais Neovim Integration Installer${NC}"
echo "=================================="
echo ""

# Check if vais-lsp is available
if command -v vais-lsp &> /dev/null; then
    LSP_PATH="vais-lsp"
    echo -e "${GREEN}✓${NC} Found vais-lsp in PATH"
elif [ -f "$VAIS_ROOT/target/release/vais-lsp" ]; then
    LSP_PATH="$VAIS_ROOT/target/release/vais-lsp"
    echo -e "${GREEN}✓${NC} Found vais-lsp at $LSP_PATH"
else
    echo -e "${YELLOW}⚠${NC} vais-lsp not found in PATH or target/release/"
    LSP_PATH="vais-lsp"
    echo "  Will use 'vais-lsp' - make sure to build and install it"
fi

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

# Ask about LSP framework
echo ""
echo "Select your completion framework:"
echo "  1) nvim-lspconfig (recommended)"
echo "  2) coc.nvim"
echo "  3) Both"
echo "  4) Skip LSP configuration"
read -p "Enter choice [1-4]: " lsp_choice

case $lsp_choice in
    1)
        install_lspconfig=true
        install_coc=false
        ;;
    2)
        install_lspconfig=false
        install_coc=true
        ;;
    3)
        install_lspconfig=true
        install_coc=true
        ;;
    *)
        install_lspconfig=false
        install_coc=false
        ;;
esac

# Install nvim-lspconfig configuration
if [ "$install_lspconfig" = true ]; then
    cp "$SCRIPT_DIR/lsp.lua" "$NVIM_CONFIG/lua/vais_lsp.lua"
    echo -e "${GREEN}✓${NC} Installed lua/vais_lsp.lua"
    echo ""
    echo -e "${YELLOW}Note:${NC} Add the following line to your init.lua to enable LSP:"
    echo "  require('vais_lsp')"
fi

# Install coc.nvim configuration
if [ "$install_coc" = true ]; then
    echo ""
    echo -e "${CYAN}Configuring coc.nvim...${NC}"

    COC_SETTINGS="$NVIM_CONFIG/coc-settings.json"
    VAIS_COC_CONFIG=$(cat << EOF
{
  "languageserver": {
    "vais": {
      "command": "$LSP_PATH",
      "filetypes": ["vais"],
      "rootPatterns": ["vais.toml", ".git/"]
    }
  }
}
EOF
)

    if [ -f "$COC_SETTINGS" ]; then
        # Backup existing settings
        cp "$COC_SETTINGS" "$COC_SETTINGS.bak"
        echo -e "${YELLOW}⚠${NC} Backed up existing coc-settings.json to coc-settings.json.bak"

        # Try to merge using jq if available
        if command -v jq &> /dev/null; then
            jq -s '.[0] * .[1]' "$COC_SETTINGS.bak" <(echo "$VAIS_COC_CONFIG") > "$COC_SETTINGS"
            echo -e "${GREEN}✓${NC} Merged vais configuration into coc-settings.json"
        else
            echo -e "${YELLOW}⚠${NC} jq not found. Please manually add to $COC_SETTINGS:"
            echo "$VAIS_COC_CONFIG"
        fi
    else
        echo "$VAIS_COC_CONFIG" > "$COC_SETTINGS"
        echo -e "${GREEN}✓${NC} Created coc-settings.json with vais configuration"
    fi
fi

# Ask about UltiSnips
echo ""
read -p "Install UltiSnips snippets? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    ULTISNIPS_DIR="$NVIM_CONFIG/UltiSnips"
    mkdir -p "$ULTISNIPS_DIR"

    cat > "$ULTISNIPS_DIR/vais.snippets" << 'SNIPPETS_EOF'
# Vais snippets for UltiSnips

snippet F "Function definition" b
F ${1:name}(${2:params}) -> ${3:ReturnType} {
	$0
}
endsnippet

snippet Fe "Function expression" b
F ${1:name}(${2:params}) -> ${3:ReturnType} = $0
endsnippet

snippet S "Struct definition" b
S ${1:Name} {
	${2:field}: ${3:Type},
}
endsnippet

snippet E "Enum definition" b
E ${1:Name} {
	${2:Variant1},
	${3:Variant2},
}
endsnippet

snippet T "Trait definition" b
T ${1:TraitName} {
	F ${2:method}(&self) -> ${3:ReturnType};
}
endsnippet

snippet X "Impl block" b
X ${1:Type} {
	F ${2:method}(&self) -> ${3:ReturnType} {
		$0
	}
}
endsnippet

snippet M "Match expression" b
M ${1:expr} {
	${2:pattern} => ${3:result},
	_ => $0,
}
endsnippet

snippet main "Main function" b
F main() -> i64 {
	$0
	0
}
endsnippet

snippet test "Test function" b
#[test]
F test_${1:name}() {
	$0
}
endsnippet

snippet I "If expression" b
I ${1:condition} {
	$0
}
endsnippet

snippet Ie "If-else expression" b
I ${1:condition} {
	$2
} else {
	$0
}
endsnippet

snippet L "Loop" b
L {
	$0
}
endsnippet

snippet W "While loop" b
W ${1:condition} {
	$0
}
endsnippet

snippet for "For loop" b
for ${1:item} in ${2:iter} {
	$0
}
endsnippet
SNIPPETS_EOF

    echo -e "${GREEN}✓${NC} Installed UltiSnips snippets to $ULTISNIPS_DIR/vais.snippets"
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Files installed to: $NVIM_CONFIG"
echo ""
echo "Next steps:"
echo "1. Restart Neovim"
if [ "$install_lspconfig" = true ]; then
    echo "2. Make sure nvim-lspconfig is installed"
    echo "3. Add 'require(\"vais_lsp\")' to your init.lua"
fi
if [ "$install_coc" = true ]; then
    echo "2. Make sure coc.nvim is installed"
    echo "3. Run :CocRestart in Neovim"
fi
echo ""
echo "For more information, see: $SCRIPT_DIR/README.md"
