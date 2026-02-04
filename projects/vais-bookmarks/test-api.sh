#!/bin/bash
# test-api.sh - Automated API testing script for Vais Bookmarks

set -e  # Exit on error

API_URL="http://localhost:8080"
PASSED=0
FAILED=0

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "  Vais Bookmarks API Test Suite"
echo "=========================================="
echo ""

# Helper function to test an endpoint
test_endpoint() {
    local name="$1"
    local method="$2"
    local path="$3"
    local data="$4"
    local expected_status="$5"

    echo -n "Testing: $name ... "

    if [ -z "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$API_URL$path")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$API_URL$path" \
            -H "Content-Type: application/json" \
            -d "$data")
    fi

    status=$(echo "$response" | tail -1)
    body=$(echo "$response" | head -n -1)

    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✓ PASS${NC} (Status: $status)"
        PASSED=$((PASSED + 1))
        echo "  Response: $body"
    else
        echo -e "${RED}✗ FAIL${NC} (Expected: $expected_status, Got: $status)"
        FAILED=$((FAILED + 1))
        echo "  Response: $body"
    fi
    echo ""
}

# Check if server is running
echo "Checking server connection..."
if ! curl -s "$API_URL/api/health" > /dev/null 2>&1; then
    echo -e "${RED}Error: Server is not running at $API_URL${NC}"
    echo "Please start the server first:"
    echo "  ./bookmark-server"
    exit 1
fi
echo -e "${GREEN}✓ Server is running${NC}"
echo ""

# Run tests
echo "=========================================="
echo "  Running API Tests"
echo "=========================================="
echo ""

# Test 1: Health Check
test_endpoint "Health Check" "GET" "/api/health" "" "200"

# Test 2: List Empty Bookmarks
test_endpoint "List Bookmarks (Empty)" "GET" "/api/bookmarks" "" "200"

# Test 3: Create Bookmark #1
test_endpoint "Create Bookmark #1" "POST" "/api/bookmarks" \
    '{"title":"Vais Language","url":"https://vais-lang.org","tags":"programming,compiler"}' \
    "201"

# Test 4: Create Bookmark #2
test_endpoint "Create Bookmark #2" "POST" "/api/bookmarks" \
    '{"title":"GitHub","url":"https://github.com","tags":"code,git"}' \
    "201"

# Test 5: Create Bookmark #3
test_endpoint "Create Bookmark #3" "POST" "/api/bookmarks" \
    '{"title":"Rust Lang","url":"https://rust-lang.org","tags":"programming,systems"}' \
    "201"

# Test 6: List All Bookmarks
test_endpoint "List All Bookmarks" "GET" "/api/bookmarks" "" "200"

# Test 7: Get Single Bookmark
test_endpoint "Get Bookmark #1" "GET" "/api/bookmarks/1" "" "200"

# Test 8: Update Bookmark
test_endpoint "Update Bookmark #1" "PUT" "/api/bookmarks/1" \
    '{"title":"Vais - AI-Optimized Language","url":"https://vais-lang.org","tags":"programming,compiler,ai"}' \
    "200"

# Test 9: Search Bookmarks
test_endpoint "Search 'vais'" "GET" "/api/search?q=vais" "" "200"

# Test 10: Delete Bookmark
test_endpoint "Delete Bookmark #2" "DELETE" "/api/bookmarks/2" "" "200"

# Test 11: Get Deleted Bookmark (should fail)
test_endpoint "Get Deleted Bookmark" "GET" "/api/bookmarks/2" "" "404"

# Test 12: Create with Missing Fields
test_endpoint "Create with Missing Title" "POST" "/api/bookmarks" \
    '{"url":"https://example.com"}' \
    "400"

# Test 13: Update Non-Existent Bookmark
test_endpoint "Update Non-Existent" "PUT" "/api/bookmarks/999" \
    '{"title":"Test","url":"https://test.com","tags":"test"}' \
    "404"

# Test 14: Delete Non-Existent Bookmark
test_endpoint "Delete Non-Existent" "DELETE" "/api/bookmarks/999" "" "404"

# Summary
echo "=========================================="
echo "  Test Summary"
echo "=========================================="
echo -e "Total Tests:  $((PASSED + FAILED))"
echo -e "${GREEN}Passed:       $PASSED${NC}"
echo -e "${RED}Failed:       $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed! ✗${NC}"
    exit 1
fi
