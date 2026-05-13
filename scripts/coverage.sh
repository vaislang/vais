#!/bin/bash
# Coverage reporting script using cargo-llvm-cov
# Usage: ./scripts/coverage.sh [command]
# Commands: html, lcov, all, clean, view
#
# Requires: cargo-llvm-cov (install: cargo install cargo-llvm-cov)
#           llvm-tools-preview (install: rustup component add llvm-tools-preview)

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COVERAGE_DIR="$PROJECT_ROOT/target/coverage"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Excluded crates (synced with tarpaulin.toml and CI)
EXCLUDES="--exclude vais-python --exclude vais-node --exclude vais-dap --exclude vais-playground-server"

# Ensure cargo-llvm-cov is installed
ensure_llvm_cov() {
    if ! cargo llvm-cov --version &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-llvm-cov...${NC}"
        cargo install cargo-llvm-cov
    fi
    # Ensure llvm-tools-preview component is available
    if ! rustup component list --installed 2>/dev/null | grep -q llvm-tools; then
        echo -e "${YELLOW}Installing llvm-tools-preview...${NC}"
        rustup component add llvm-tools-preview
    fi
}

# Generate HTML coverage
generate_html() {
    echo -e "${BLUE}Generating HTML coverage report...${NC}"
    cd "$PROJECT_ROOT"
    mkdir -p "$COVERAGE_DIR"
    cargo llvm-cov --workspace $EXCLUDES --html --output-dir "$COVERAGE_DIR"
    echo -e "${GREEN}HTML report generated at: $COVERAGE_DIR/html/index.html${NC}"
}

# Generate Lcov coverage
generate_lcov() {
    echo -e "${BLUE}Generating Lcov coverage report...${NC}"
    cd "$PROJECT_ROOT"
    mkdir -p "$COVERAGE_DIR"
    cargo llvm-cov --workspace $EXCLUDES --lcov --output-path "$COVERAGE_DIR/lcov.info"
    echo -e "${GREEN}Lcov report generated at: $COVERAGE_DIR/lcov.info${NC}"
}

# Generate all coverage reports
generate_all() {
    echo -e "${BLUE}Generating all coverage reports...${NC}"
    cd "$PROJECT_ROOT"
    mkdir -p "$COVERAGE_DIR"

    # Generate lcov (primary format for CI/Codecov)
    cargo llvm-cov --workspace $EXCLUDES --lcov --output-path "$COVERAGE_DIR/lcov.info"

    # Generate HTML report
    cargo llvm-cov --workspace $EXCLUDES --html --output-dir "$COVERAGE_DIR"

    # Generate JSON summary
    cargo llvm-cov --workspace $EXCLUDES --json --output-path "$COVERAGE_DIR/coverage.json"

    echo -e "${GREEN}All reports generated in: $COVERAGE_DIR${NC}"

    # Display summary
    display_summary
}

# Display coverage summary from lcov.info
display_summary() {
    if [ -f "$COVERAGE_DIR/lcov.info" ]; then
        echo -e "\n${BLUE}Coverage Summary:${NC}"
        echo -e "${BLUE}==================${NC}"

        LINES=$(grep -c "^DA:" "$COVERAGE_DIR/lcov.info" 2>/dev/null || echo 0)
        COVERED=$(grep "^DA:" "$COVERAGE_DIR/lcov.info" 2>/dev/null | grep -cv ",0$" || echo 0)

        if [ "$LINES" -gt 0 ]; then
            COVERAGE=$(python3 -c "print(f'{$COVERED * 100 / $LINES:.1f}')")
            echo -e "Overall Coverage: ${GREEN}${COVERAGE}%${NC} ($COVERED/$LINES lines)"
            echo -e "Target: ${YELLOW}75%+${NC}"

            # Check if coverage meets target
            if (( $(echo "$COVERAGE >= 75" | bc -l 2>/dev/null || echo 0) )); then
                echo -e "Status: ${GREEN}PASSING${NC} (meets 75% target)"
            else
                echo -e "Status: ${YELLOW}NEEDS IMPROVEMENT${NC} (below 75% target)"
            fi
        else
            echo -e "${YELLOW}Coverage data not available${NC}"
        fi
        echo ""
    elif [ -f "$COVERAGE_DIR/coverage.json" ]; then
        echo -e "\n${BLUE}Coverage Summary:${NC}"
        if command -v jq &> /dev/null; then
            jq '.data[0].totals.lines' "$COVERAGE_DIR/coverage.json"
        else
            echo -e "${YELLOW}Install jq for JSON summary parsing${NC}"
        fi
    else
        echo -e "${YELLOW}No coverage reports found. Run 'all' first.${NC}"
    fi
}

# Clean coverage reports
clean_coverage() {
    echo -e "${YELLOW}Cleaning coverage reports...${NC}"
    rm -rf "$COVERAGE_DIR"
    cargo llvm-cov clean --workspace 2>/dev/null || true
    echo -e "${GREEN}Coverage data cleaned${NC}"
}

# View HTML coverage report
view_coverage() {
    local html_path="$COVERAGE_DIR/html/index.html"
    if [ ! -f "$html_path" ]; then
        echo -e "${RED}No HTML coverage report found at $html_path${NC}"
        echo "Run './scripts/coverage.sh html' or './scripts/coverage.sh all' first"
        exit 1
    fi

    echo -e "${BLUE}Opening coverage report...${NC}"

    # Try different ways to open the file based on OS
    if command -v open &> /dev/null; then
        open "$html_path"
    elif command -v xdg-open &> /dev/null; then
        xdg-open "$html_path"
    elif command -v start &> /dev/null; then
        start "$html_path"
    else
        echo -e "${YELLOW}Please open this file in your browser: $html_path${NC}"
    fi
}

# Print help
print_help() {
    cat << EOF
${BLUE}Coverage Reporting Script (cargo-llvm-cov)${NC}

Usage: $0 [command]

Commands:
    html      Generate HTML coverage report
    lcov      Generate Lcov format coverage report
    all       Generate all coverage reports (HTML + Lcov + JSON)
    summary   Display coverage summary from existing reports
    view      Open HTML coverage report in browser
    clean     Remove all generated coverage reports + profraw data
    help      Show this help message

Examples:
    $0 html        # Generate HTML report
    $0 all         # Generate all reports with summary
    $0 summary     # Display coverage summary
    $0 view        # View HTML report in browser
    $0 clean       # Clean up coverage files

Coverage Target: 75%+
Coverage reports are saved to: $COVERAGE_DIR
EOF
}

# Main script logic
main() {
    local command="${1:-all}"

    case "$command" in
        html)
            ensure_llvm_cov
            generate_html
            ;;
        lcov)
            ensure_llvm_cov
            generate_lcov
            ;;
        all)
            ensure_llvm_cov
            generate_all
            ;;
        summary)
            display_summary
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
