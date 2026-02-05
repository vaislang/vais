#!/bin/bash
# Publish seed packages to Vais Registry
# Usage: ./scripts/publish-packages.sh [registry_url] [api_token]

set -e

REGISTRY_URL="${1:-http://localhost:3000}"
API_TOKEN="${2:-}"
PACKAGES_DIR="packages"

echo "========================================"
echo "Vais Package Publisher"
echo "========================================"
echo "Registry: $REGISTRY_URL"
echo ""

# Check registry is accessible
if ! curl -s "${REGISTRY_URL}/health" | grep -q "ok"; then
    echo "Error: Registry at $REGISTRY_URL is not accessible"
    echo "Make sure the registry server is running."
    exit 1
fi
echo "Registry is accessible"

# Check if API token is provided
if [ -z "$API_TOKEN" ]; then
    echo ""
    echo "No API token provided."
    echo "You can either:"
    echo "  1. Register and get a token: curl -X POST ${REGISTRY_URL}/api/v1/auth/register -H 'Content-Type: application/json' -d '{\"username\":\"admin\",\"password\":\"yourpassword\",\"email\":\"admin@example.com\"}'"
    echo "  2. Login to get a token: curl -X POST ${REGISTRY_URL}/api/v1/auth/login -H 'Content-Type: application/json' -d '{\"username\":\"admin\",\"password\":\"yourpassword\"}'"
    echo ""
    read -p "Enter API token: " API_TOKEN
    if [ -z "$API_TOKEN" ]; then
        echo "Error: API token is required"
        exit 1
    fi
fi

# List of packages to publish
PACKAGES=(
    "cli-args"
    "color"
    "csv"
    "dotenv"
    "env"
    "math-ext"
    "retry"
    "toml-parser"
    "validate"
    "cache"
)

TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

published=0
failed=0

for pkg in "${PACKAGES[@]}"; do
    PKG_DIR="${PACKAGES_DIR}/${pkg}"

    if [ ! -d "$PKG_DIR" ]; then
        echo "Warning: Package directory not found: $PKG_DIR"
        ((failed++))
        continue
    fi

    if [ ! -f "$PKG_DIR/vais.toml" ]; then
        echo "Warning: vais.toml not found in $PKG_DIR"
        ((failed++))
        continue
    fi

    echo ""
    echo "Publishing: $pkg"
    echo "----------------------------------------"

    # Read package info from vais.toml
    PKG_NAME=$(grep '^name' "$PKG_DIR/vais.toml" | sed 's/.*=.*"\(.*\)"/\1/')
    PKG_VERSION=$(grep '^version' "$PKG_DIR/vais.toml" | sed 's/.*=.*"\(.*\)"/\1/')
    PKG_DESC=$(grep '^description' "$PKG_DIR/vais.toml" | sed 's/.*=.*"\(.*\)"/\1/')

    echo "  Name: $PKG_NAME"
    echo "  Version: $PKG_VERSION"
    echo "  Description: $PKG_DESC"

    # Create archive
    ARCHIVE_PATH="${TEMP_DIR}/${pkg}-${PKG_VERSION}.tar.gz"
    (cd "$PKG_DIR" && tar -czf "$ARCHIVE_PATH" .)

    ARCHIVE_SIZE=$(stat -f%z "$ARCHIVE_PATH" 2>/dev/null || stat -c%s "$ARCHIVE_PATH")
    echo "  Archive size: ${ARCHIVE_SIZE} bytes"

    # Create metadata JSON (dependencies/dev_dependencies must be objects, not arrays)
    METADATA_FILE="${TEMP_DIR}/${pkg}-metadata.json"
    cat > "$METADATA_FILE" <<EOF
{"name":"$PKG_NAME","version":"$PKG_VERSION","description":"$PKG_DESC","license":"MIT","authors":["Vais Team"],"keywords":[],"categories":[],"dependencies":{},"dev_dependencies":{}}
EOF

    # Publish using multipart form
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST "${REGISTRY_URL}/api/v1/packages/publish" \
        -H "Authorization: Bearer ${API_TOKEN}" \
        -F "metadata=@${METADATA_FILE};type=application/json" \
        -F "archive=@${ARCHIVE_PATH};type=application/gzip")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
        echo "  Published successfully"
        ((published++))
    elif [ "$HTTP_CODE" = "409" ]; then
        echo "  Already exists (version conflict)"
        ((published++))
    else
        echo "  Failed (HTTP $HTTP_CODE): $BODY"
        ((failed++))
    fi
done

echo ""
echo "========================================"
echo "Summary"
echo "========================================"
echo "Published: $published"
echo "Failed:    $failed"
echo ""

if [ $failed -eq 0 ]; then
    echo "All packages published successfully!"
    echo ""
    echo "Verify with:"
    echo "  curl ${REGISTRY_URL}/api/v1/search"
else
    echo "Some packages failed to publish."
    exit 1
fi
