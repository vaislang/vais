#!/bin/sh
# Vais Programming Language — One-Click Installer
# Usage: curl -fsSL https://vais.dev/install.sh | sh
#        wget -qO- https://vais.dev/install.sh | sh
#
# Environment variables:
#   VAIS_VERSION   - Specific version to install (default: latest)
#   VAIS_INSTALL   - Installation directory (default: /usr/local/bin)

set -e

REPO="vaislang/vais"
INSTALL_DIR="${VAIS_INSTALL:-/usr/local/bin}"

# Validate INSTALL_DIR is an absolute path
case "$INSTALL_DIR" in
    /*) ;; # absolute path — ok
    *)  printf "error VAIS_INSTALL must be an absolute path (got: %s)\n" "$INSTALL_DIR" >&2; exit 1 ;;
esac

# Colors (disabled if not a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' BOLD='' RESET=''
fi

info()  { printf "${BLUE}info${RESET}  %s\n" "$1"; }
ok()    { printf "${GREEN}ok${RESET}    %s\n" "$1"; }
warn()  { printf "${YELLOW}warn${RESET}  %s\n" "$1"; }
error() { printf "${RED}error${RESET} %s\n" "$1" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS_NAME="linux" ;;
        Darwin) OS_NAME="macos" ;;
        *)      error "Unsupported operating system: $OS. Use Windows installer (install.ps1) for Windows." ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_NAME="x86_64" ;;
        aarch64|arm64)   ARCH_NAME="aarch64" ;;
        *)               error "Unsupported architecture: $ARCH" ;;
    esac

    # Map to release target triple
    case "${OS_NAME}-${ARCH_NAME}" in
        linux-x86_64)    TARGET="x86_64-unknown-linux-gnu" ;;
        linux-aarch64)   TARGET="aarch64-unknown-linux-gnu" ;;
        macos-x86_64)    TARGET="x86_64-apple-darwin" ;;
        macos-aarch64)   TARGET="aarch64-apple-darwin" ;;
        *)               error "Unsupported platform: ${OS_NAME}-${ARCH_NAME}" ;;
    esac

    info "Detected platform: ${OS_NAME} ${ARCH_NAME} (${TARGET})"
}

# Check for required commands
check_deps() {
    if command -v curl >/dev/null 2>&1; then
        DOWNLOAD="curl -fsSL"
        DOWNLOAD_TO="curl -fsSL -o"
    elif command -v wget >/dev/null 2>&1; then
        DOWNLOAD="wget -qO-"
        DOWNLOAD_TO="wget -qO"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    if ! command -v tar >/dev/null 2>&1; then
        error "tar is required but not found."
    fi
}

# Get latest version from GitHub API
get_version() {
    if [ -n "$VAIS_VERSION" ]; then
        # Validate semver-like format (v0.0.0 or 0.0.0, optional pre-release)
        if ! echo "$VAIS_VERSION" | grep -qE '^v?[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
            error "Invalid version format: ${VAIS_VERSION}. Expected: v1.2.3 or 1.2.3"
        fi
        VERSION="$VAIS_VERSION"
        info "Installing specified version: ${VERSION}"
    else
        info "Fetching latest version..."
        VERSION="$($DOWNLOAD "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')"
        if [ -z "$VERSION" ]; then
            error "Failed to fetch latest version. Set VAIS_VERSION manually or check your network."
        fi
        info "Latest version: ${VERSION}"
    fi
}

# Download and install
install_vais() {
    ARCHIVE="vais-${VERSION}-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

    info "Downloading ${ARCHIVE}..."

    # Create temp directory
    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf "$TMP_DIR"' EXIT

    $DOWNLOAD_TO "${TMP_DIR}/${ARCHIVE}" "$URL" 2>/dev/null \
        || error "Download failed. Check that version ${VERSION} exists for ${TARGET}.\n  URL: ${URL}"

    info "Extracting..."
    tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

    # Find the binary
    BINARY=""
    if [ -f "${TMP_DIR}/vais/vaisc" ]; then
        BINARY="${TMP_DIR}/vais/vaisc"
    elif [ -f "${TMP_DIR}/vaisc" ]; then
        BINARY="${TMP_DIR}/vaisc"
    else
        error "Binary 'vaisc' not found in archive."
    fi

    # Install binary
    info "Installing to ${INSTALL_DIR}/vaisc..."
    if [ -w "$INSTALL_DIR" ]; then
        cp "$BINARY" "${INSTALL_DIR}/vaisc"
        chmod +x "${INSTALL_DIR}/vaisc"
    else
        warn "Permission denied for ${INSTALL_DIR}. Using sudo..."
        sudo cp "$BINARY" "${INSTALL_DIR}/vaisc"
        sudo chmod +x "${INSTALL_DIR}/vaisc"
    fi

    # Install std library if present
    if [ -d "${TMP_DIR}/vais/std" ]; then
        STD_DIR="${INSTALL_DIR}/../lib/vais/std"
        info "Installing standard library to ${STD_DIR}..."
        if [ -w "$(dirname "$STD_DIR")" ] 2>/dev/null; then
            mkdir -p "$STD_DIR"
            cp -r "${TMP_DIR}/vais/std/"* "$STD_DIR/"
        else
            sudo mkdir -p "$STD_DIR"
            sudo cp -r "${TMP_DIR}/vais/std/"* "$STD_DIR/"
        fi
    fi
}

# Verify installation
verify() {
    if command -v vaisc >/dev/null 2>&1; then
        INSTALLED_VERSION="$(vaisc --version 2>/dev/null || echo "unknown")"
        ok "Vais installed successfully! (${INSTALLED_VERSION})"
    elif [ -x "${INSTALL_DIR}/vaisc" ]; then
        ok "Vais installed to ${INSTALL_DIR}/vaisc"
        warn "${INSTALL_DIR} may not be in your PATH. Add it with:"
        printf "  export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
    else
        error "Installation verification failed."
    fi
}

# Print next steps
print_next_steps() {
    printf "\n${BOLD}Getting started:${RESET}\n"
    printf "  ${GREEN}\$${RESET} echo 'F main() { puts(\"Hello, Vais!\") }' > hello.vais\n"
    printf "  ${GREEN}\$${RESET} vaisc run hello.vais\n"
    printf "\n"
    printf "  Docs:       https://vais.dev/docs/\n"
    printf "  Playground:  https://vais.dev/playground/\n"
    printf "  GitHub:      https://github.com/${REPO}\n"
    printf "\n"
}

# Main
main() {
    printf "\n${BOLD}  Vais Installer${RESET}\n\n"
    detect_platform
    check_deps
    get_version
    install_vais
    verify
    print_next_steps
}

main
