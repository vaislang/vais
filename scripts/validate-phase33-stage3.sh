#!/bin/bash
# Validation script for Phase 33 Stage 3 implementation
# Checks that all required files exist and compile correctly

set -e

echo "=========================================="
echo "Phase 33 Stage 3 Validation"
echo "=========================================="
echo ""

FAILED=0

# Function to check if file exists
check_file() {
    if [ -f "$1" ]; then
        echo "✓ $1"
    else
        echo "✗ $1 (missing)"
        FAILED=1
    fi
}

# Function to check if directory exists
check_dir() {
    if [ -d "$1" ]; then
        echo "✓ $1/"
    else
        echo "✗ $1/ (missing)"
        FAILED=1
    fi
}

echo "1. Checking created files..."
echo ""

check_file "Dockerfile.registry"
check_file "docker-compose.registry.yml"
check_file "REGISTRY_DEPLOYMENT.md"
check_file "docs/phase33-stage3-summary.md"
check_file "docs/phase33-stage3-files.md"
check_file "scripts/deploy-registry.sh"
check_dir "examples/package"
check_file "examples/package/vais.toml"
check_file "examples/package/README.md"
check_file "examples/package/src/lib.vais"

echo ""
echo "2. Checking modified files..."
echo ""

check_file "crates/vaisc/src/main.rs"

echo ""
echo "3. Checking existing implementation files..."
echo ""

check_file "crates/vaisc/src/registry/mod.rs"
check_file "crates/vaisc/src/registry/version.rs"
check_file "crates/vaisc/src/registry/resolver.rs"
check_file "crates/vaisc/src/registry/client.rs"
check_file "crates/vaisc/src/registry/archive.rs"
check_file "crates/vaisc/src/package.rs"

echo ""
echo "4. Checking registry server..."
echo ""

check_dir "crates/vais-registry-server"
check_file "crates/vais-registry-server/src/main.rs"
check_file "crates/vais-registry-server/Cargo.toml"

echo ""
echo "5. Compilation checks..."
echo ""

echo "Checking vaisc compilation..."
if cargo check -p vaisc --quiet 2>&1 | grep -q "error"; then
    echo "✗ vaisc failed to compile"
    FAILED=1
else
    echo "✓ vaisc compiles successfully"
fi

echo "Checking registry server compilation..."
if cargo check -p vais-registry-server --quiet 2>&1 | grep -q "error"; then
    echo "✗ vais-registry-server failed to compile"
    FAILED=1
else
    echo "✓ vais-registry-server compiles successfully"
fi

echo ""
echo "6. Docker file validation..."
echo ""

if command -v docker-compose &> /dev/null; then
    echo "Validating docker-compose.registry.yml..."
    if docker-compose -f docker-compose.registry.yml config > /dev/null 2>&1; then
        echo "✓ docker-compose.registry.yml is valid"
    else
        echo "✗ docker-compose.registry.yml has errors"
        FAILED=1
    fi
else
    echo "⚠ docker-compose not available, skipping validation"
fi

echo ""
echo "7. Script permissions..."
echo ""

if [ -x "scripts/deploy-registry.sh" ]; then
    echo "✓ deploy-registry.sh is executable"
else
    echo "✗ deploy-registry.sh is not executable"
    FAILED=1
fi

if [ -x "scripts/validate-phase33-stage3.sh" ]; then
    echo "✓ validate-phase33-stage3.sh is executable"
else
    echo "⚠ validate-phase33-stage3.sh is not executable (run: chmod +x)"
fi

echo ""
echo "8. Documentation completeness..."
echo ""

DEPLOYMENT_LINES=$(wc -l < REGISTRY_DEPLOYMENT.md)
SUMMARY_LINES=$(wc -l < docs/phase33-stage3-summary.md)
FILES_LINES=$(wc -l < docs/phase33-stage3-files.md)

if [ "$DEPLOYMENT_LINES" -gt 400 ]; then
    echo "✓ REGISTRY_DEPLOYMENT.md is comprehensive ($DEPLOYMENT_LINES lines)"
else
    echo "⚠ REGISTRY_DEPLOYMENT.md seems incomplete ($DEPLOYMENT_LINES lines)"
fi

if [ "$SUMMARY_LINES" -gt 500 ]; then
    echo "✓ phase33-stage3-summary.md is comprehensive ($SUMMARY_LINES lines)"
else
    echo "⚠ phase33-stage3-summary.md seems incomplete ($SUMMARY_LINES lines)"
fi

if [ "$FILES_LINES" -gt 200 ]; then
    echo "✓ phase33-stage3-files.md is comprehensive ($FILES_LINES lines)"
else
    echo "⚠ phase33-stage3-files.md seems incomplete ($FILES_LINES lines)"
fi

echo ""
echo "9. Feature checklist..."
echo ""

# Check for key features in the code
if grep -q "Verifying checksum" crates/vaisc/src/main.rs; then
    echo "✓ Post-upload checksum verification implemented"
else
    echo "✗ Post-upload checksum verification not found"
    FAILED=1
fi

if grep -q "sha256_hex" crates/vaisc/src/registry/archive.rs; then
    echo "✓ SHA-256 checksum function exists"
else
    echo "✗ SHA-256 checksum function not found"
    FAILED=1
fi

if grep -q "VersionReq" crates/vaisc/src/registry/version.rs; then
    echo "✓ Semver version requirements implemented"
else
    echo "✗ Semver version requirements not found"
    FAILED=1
fi

if grep -q "DependencyResolver" crates/vaisc/src/registry/resolver.rs; then
    echo "✓ Dependency resolver implemented"
else
    echo "✗ Dependency resolver not found"
    FAILED=1
fi

echo ""
echo "=========================================="
if [ $FAILED -eq 0 ]; then
    echo "✓ All checks passed!"
    echo "=========================================="
    echo ""
    echo "Phase 33 Stage 3 is complete and ready."
    echo ""
    echo "Next steps:"
    echo "  1. Review the changes"
    echo "  2. Test deployment: ./scripts/deploy-registry.sh"
    echo "  3. Commit with: git add ... && git commit -m 'feat: ...'"
    echo ""
    exit 0
else
    echo "✗ Some checks failed"
    echo "=========================================="
    echo ""
    echo "Please review the failures above and fix them."
    echo ""
    exit 1
fi
