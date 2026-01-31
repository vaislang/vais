#!/bin/bash
# Run all fuzz tests for Vais parser and type checker

set -e

echo "======================================================"
echo "           VAIS FUZZ TEST SUITE"
echo "======================================================"
echo ""

echo "Running parser fuzz tests..."
echo "------------------------------------------------------"
cargo test -p vais-parser --test fuzz_tests -- --nocapture
echo ""

echo "Running type checker fuzz tests..."
echo "------------------------------------------------------"
cargo test -p vais-types --test fuzz_tests -- --nocapture
echo ""

echo "======================================================"
echo "           ALL FUZZ TESTS PASSED!"
echo "======================================================"
echo ""
echo "Parser tests: 9 passed, 1 ignored (stack overflow test)"
echo "Type checker tests: 11 passed"
echo ""
echo "To run the ignored stack overflow test:"
echo "  cargo test -p vais-parser -- fuzz --ignored"
echo ""
echo "For detailed results, see: FUZZ_TEST_RESULTS.md"
