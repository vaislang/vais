#!/bin/bash
# Build script for vais-todo

set -e

echo "Building vais-todo..."

# Check if vaisc compiler exists
if ! command -v vaisc &> /dev/null; then
    echo "Error: vaisc compiler not found"
    echo "Please build the Vais compiler first:"
    echo "  cd /Users/sswoo/study/projects/vais"
    echo "  cargo build --release"
    exit 1
fi

# Compile the project
echo "Compiling src/main.vais..."
vaisc src/main.vais -o vais-todo

echo "Build complete! Binary: ./vais-todo"
echo ""
echo "Run with: ./vais-todo help"
