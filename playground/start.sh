#!/bin/bash

# Vais Playground Quick Start Script

set -e

echo "🚀 Vais Playground Setup"
echo "======================="
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+ first."
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

# Check Node version
NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js version must be 18 or higher. Current: $(node -v)"
    exit 1
fi

echo "✅ Node.js $(node -v) detected"
echo ""

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed."
    exit 1
fi

echo "✅ npm $(npm -v) detected"
echo ""

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo ""
else
    echo "✅ Dependencies already installed"
    echo ""
fi

# Check if we want to build WASM (optional)
if [ "$1" == "--with-wasm" ]; then
    echo "🔨 Building WASM compiler..."
    echo "   (This requires Rust and wasm-pack)"
    echo ""

    if ! command -v cargo &> /dev/null; then
        echo "⚠️  Rust not found. Skipping WASM build."
        echo "   Install Rust from: https://rustup.rs/"
        echo ""
    else
        # Check if wasm32 target is installed
        if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
            echo "📦 Installing wasm32 target..."
            rustup target add wasm32-unknown-unknown
        fi

        # Build WASM (this will fail if vais-wasm crate doesn't exist yet)
        if [ -d "../crates/vais-wasm" ]; then
            cd ../crates/vais-wasm
            wasm-pack build --target web --out-dir ../../playground/public/wasm
            cd ../../playground
            echo "✅ WASM build complete"
        else
            echo "⚠️  vais-wasm crate not found. Using mock compiler."
            echo "   See INTEGRATION.md for WASM setup instructions."
        fi
        echo ""
    fi
fi

# Start development server
echo "🎯 Starting development server..."
echo ""
echo "The playground will open at: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

npm run dev
