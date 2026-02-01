#!/bin/bash
set -e

# Vais .deb Package Builder
# Builds a Debian package for the Vais programming language

VERSION="1.0.0"
ARCH="amd64"
PACKAGE_NAME="vais_${VERSION}_${ARCH}"
BUILD_DIR="build/${PACKAGE_NAME}"

echo "Building Vais .deb package v${VERSION}..."

# Clean and create build directory structure
rm -rf build
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/vais/std"

# Create DEBIAN control file
cat > "${BUILD_DIR}/DEBIAN/control" << EOF
Package: vais
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: Steve <steve@vais-lang.org>
Depends: clang
Section: devel
Priority: optional
Homepage: https://github.com/vaislang/vais
Description: AI-optimized systems programming language
 Vais is a modern systems programming language with an LLVM backend,
 designed for AI-assisted development. It combines low-level control
 with high-level abstractions, featuring:
 .
  - Static typing with type inference
  - Memory safety with ownership tracking
  - Native LLVM code generation
  - Rich standard library
  - LSP and DAP support for IDE integration
EOF

# Build the release binary
echo "Building release binary..."
cd ../..
cargo build --release --bin vaisc

# Copy binary
echo "Copying binary..."
cp target/release/vaisc "packaging/deb/${BUILD_DIR}/usr/bin/"
chmod +x "packaging/deb/${BUILD_DIR}/usr/bin/vaisc"

# Copy standard library
echo "Copying standard library..."
cp -r std/* "packaging/deb/${BUILD_DIR}/usr/share/vais/std/"

# Build the .deb package
cd packaging/deb
echo "Building .deb package..."
dpkg-deb --build "${BUILD_DIR}"

# Move to output directory
mv "${BUILD_DIR}.deb" .

echo "Package built successfully: ${PACKAGE_NAME}.deb"
echo ""
echo "Install with: sudo dpkg -i ${PACKAGE_NAME}.deb"
echo "Remove with:  sudo dpkg -r vais"
