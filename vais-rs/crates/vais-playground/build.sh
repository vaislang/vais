#!/bin/bash
# Vais Playground Build Script

set -e

echo "Building Vais Playground WASM..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build WASM
cd "$(dirname "$0")"
wasm-pack build --target web --out-dir www/pkg

echo ""
echo "Build complete! To run the playground:"
echo ""
echo "  cd www"
echo "  python3 -m http.server 8080"
echo ""
echo "Then open http://localhost:8080 in your browser."
