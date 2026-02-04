#!/bin/bash
# Build script for vais-datapipe

set -e

echo "=== Building Vais Data Pipeline ==="

# Check if vaisc is available
if ! command -v vaisc &> /dev/null; then
    echo "Error: vaisc not found in PATH"
    echo "Please build and install vaisc first:"
    echo "  cd /path/to/vais"
    echo "  cargo build --release --bin vaisc"
    echo "  export PATH=\$PATH:\$(pwd)/target/release"
    exit 1
fi

# Create output directory
mkdir -p build

# Compile all source files
echo "Compiling CSV reader..."
vaisc src/csv_reader.vais -o build/csv_reader.ll || exit 1

echo "Compiling transformer..."
vaisc src/transformer.vais -o build/transformer.ll || exit 1

echo "Compiling JSON writer..."
vaisc src/json_writer.vais -o build/json_writer.ll || exit 1

echo "Compiling pipeline..."
vaisc src/pipeline.vais -o build/pipeline.ll || exit 1

echo "Compiling main..."
vaisc src/main.vais -o build/main.ll || exit 1

# Link all modules (if linker is available)
echo "Linking modules..."
# This would require a proper linker implementation
# For now, individual modules can be tested

echo ""
echo "=== Build Complete ==="
echo "Output files are in build/"
echo ""
echo "Note: Full linking requires linker support."
echo "Individual modules can be tested separately."
