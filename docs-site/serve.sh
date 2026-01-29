#!/usr/bin/env bash

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Vais Documentation Server${NC}"
echo "====================================="

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo -e "${YELLOW}mdBook not found. Installing...${NC}"
    cargo install mdbook
fi

# Navigate to docs-site directory
cd "$(dirname "$0")"

# Start the server
echo -e "${GREEN}Starting development server...${NC}"
echo -e "Documentation will be available at: ${YELLOW}http://localhost:3000${NC}"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

mdbook serve --open
