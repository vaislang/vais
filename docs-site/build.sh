#!/usr/bin/env bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building Vais Documentation Site${NC}"
echo "======================================"

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo -e "${YELLOW}mdBook not found. Installing...${NC}"
    cargo install mdbook
fi

# Check if mdbook-linkcheck is installed (optional but recommended)
if ! command -v mdbook-linkcheck &> /dev/null; then
    echo -e "${YELLOW}mdbook-linkcheck not found. Installing...${NC}"
    cargo install mdbook-linkcheck
fi

# Navigate to docs-site directory
cd "$(dirname "$0")"

# Clean previous build
echo -e "${GREEN}Cleaning previous build...${NC}"
rm -rf book/

# Build the documentation
echo -e "${GREEN}Building documentation...${NC}"
mdbook build

# Check if build was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Documentation built successfully!${NC}"
    echo -e "${GREEN}Output directory: $(pwd)/book${NC}"
    echo ""
    echo -e "To view the documentation locally, run:"
    echo -e "  ${YELLOW}mdbook serve${NC}"
    echo ""
    echo -e "Or open: ${YELLOW}$(pwd)/book/index.html${NC}"
else
    echo -e "${RED}✗ Build failed!${NC}"
    exit 1
fi
