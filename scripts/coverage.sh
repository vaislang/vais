#!/bin/bash
# Coverage reporting script
# Usage: ./scripts/coverage.sh [command]
# Commands: html, lcov, all, clean, view

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COVERAGE_DIR="$PROJECT_ROOT/target/coverage"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Ensure cargo-tarpaulin is installed
ensure_tarpaulin() {
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
        cargo install cargo-tarpaulin
    fi
}

# Generate HTML coverage
generate_html() {
    echo -e "${BLUE}Generating HTML coverage report...${NC}"
    cd "$PROJECT_ROOT"
    cargo tarpaulin --config tarpaulin.toml --out Html --output-dir "$COVERAGE_DIR"
    echo -e "${GREEN}HTML report generated at: $COVERAGE_DIR/index.html${NC}"
}

# Generate Lcov coverage
generate_lcov() {
    echo -e "${BLUE}Generating Lcov coverage report...${NC}"
    cd "$PROJECT_ROOT"
    cargo tarpaulin --config tarpaulin.toml --out Lcov --output-dir "$COVERAGE_DIR"
    echo -e "${GREEN}Lcov report generated at: $COVERAGE_DIR/lcov.info${NC}"
}

# Generate all coverage reports
generate_all() {
    echo -e "${BLUE}Generating all coverage reports...${NC}"
    cd "$PROJECT_ROOT"
    cargo tarpaulin --config tarpaulin.toml --out Html --out Lcov --output-dir "$COVERAGE_DIR"
    echo -e "${GREEN}All reports generated in: $COVERAGE_DIR${NC}"
}

# Clean coverage reports
clean_coverage() {
    echo -e "${YELLOW}Cleaning coverage reports...${NC}"
    rm -rf "$COVERAGE_DIR"
    echo -e "${GREEN}Coverage directory cleaned${NC}"
}

# View HTML coverage report
view_coverage() {
    if [ ! -f "$COVERAGE_DIR/index.html" ]; then
        echo -e "${RED}No HTML coverage report found at $COVERAGE_DIR/index.html${NC}"
        echo "Run 'generate_html' first"
        exit 1
    fi

    echo -e "${BLUE}Opening coverage report...${NC}"

    # Try different ways to open the file based on OS
    if command -v open &> /dev/null; then
        open "$COVERAGE_DIR/index.html"
    elif command -v xdg-open &> /dev/null; then
        xdg-open "$COVERAGE_DIR/index.html"
    elif command -v start &> /dev/null; then
        start "$COVERAGE_DIR/index.html"
    else
        echo -e "${YELLOW}Please open this file in your browser: $COVERAGE_DIR/index.html${NC}"
    fi
}

# Print help
print_help() {
    cat << EOF
${BLUE}Coverage Reporting Script${NC}

Usage: $0 [command]

Commands:
    html    Generate HTML coverage report
    lcov    Generate Lcov format coverage report
    all     Generate all coverage reports (HTML + Lcov)
    view    Open HTML coverage report in browser
    clean   Remove all generated coverage reports
    help    Show this help message

Examples:
    $0 html        # Generate HTML report
    $0 all         # Generate all reports
    $0 view        # View HTML report in browser
    $0 clean       # Clean up coverage files

Coverage reports are saved to: $COVERAGE_DIR
EOF
}

# Main script logic
main() {
    local command="${1:-all}"

    case "$command" in
        html)
            ensure_tarpaulin
            generate_html
            ;;
        lcov)
            ensure_tarpaulin
            generate_lcov
            ;;
        all)
            ensure_tarpaulin
            generate_all
            ;;
        view)
            view_coverage
            ;;
        clean)
            clean_coverage
            ;;
        help)
            print_help
            ;;
        *)
            echo -e "${RED}Unknown command: $command${NC}"
            print_help
            exit 1
            ;;
    esac
}

main "$@"
