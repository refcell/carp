#!/bin/bash

# Test Script for Carp API Search Functionality
# This script tests various search scenarios against the local API

set -e

# Configuration
BASE_URL="http://localhost:3307"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper function to make API calls and check responses
test_endpoint() {
    local endpoint="$1"
    local expected_status="$2"
    local description="$3"

    echo -e "${BLUE}Testing: $description${NC}"
    echo "GET $endpoint"

    response=$(curl -s -w "HTTP_STATUS:%{http_code}" "$endpoint")
    http_status=$(echo "$response" | grep -o "HTTP_STATUS:[0-9]*" | cut -d: -f2)
    body=$(echo "$response" | sed 's/HTTP_STATUS:[0-9]*$//')

    if [ "$http_status" -eq "$expected_status" ]; then
        echo -e "${GREEN}✅ Status: $http_status (Expected: $expected_status)${NC}"
        if [ "$expected_status" -eq 200 ]; then
            # Pretty print JSON response
            echo "$body" | python3 -m json.tool 2>/dev/null || echo "$body"
        fi
    else
        echo -e "${RED}❌ Status: $http_status (Expected: $expected_status)${NC}"
        echo "Response: $body"
        return 1
    fi
    echo ""
}

# Helper function to check if server is running
check_server() {
    echo -e "${BLUE}Checking if local server is running...${NC}"
    if curl -s "$BASE_URL/api/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Server is running at $BASE_URL${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}❌ Server not running at $BASE_URL${NC}"
        echo "Please start the server first with: ./scripts/start-local-api.sh"
        echo ""
        return 1
    fi
}

# Main test suite
main() {
    echo "🧪 Carp API Search Test Suite"
    echo "=============================="
    echo ""

    # Check if server is running
    if ! check_server; then
        exit 1
    fi

    echo "Running API tests..."
    echo ""

    # Test 1: Health endpoint
    test_endpoint "$BASE_URL/api/health" 200 "Health Check"

    # Test 2: Search all agents (empty query)
    test_endpoint "$BASE_URL/api/v1/agents/search" 200 "List All Agents (Empty Search)"

    # Test 3: Search with limit
    test_endpoint "$BASE_URL/api/v1/agents/search?limit=5" 200 "List Agents with Limit"

    # Test 4: Search for specific agent
    test_endpoint "$BASE_URL/api/v1/agents/search?q=test" 200 "Search for 'test' agents"

    # Test 5: Search with exact match
    test_endpoint "$BASE_URL/api/v1/agents/search?q=example&exact=true" 200 "Exact Search for 'example'"

    # Test 6: Search with pagination
    test_endpoint "$BASE_URL/api/v1/agents/search?page=1&limit=10" 200 "Search with Pagination"

    # Test 7: Search with invalid parameters (should still work)
    test_endpoint "$BASE_URL/api/v1/agents/search?limit=invalid" 200 "Search with Invalid Limit (Fallback)"

    echo -e "${GREEN}🎉 All tests completed!${NC}"
    echo ""
    echo "To test with the CLI:"
    echo "  cd cli && cargo run -- list --verbose"
    echo ""
    echo "To test specific searches:"
    echo "  curl '$BASE_URL/api/v1/agents/search?q=your-search-term'"
}

# Run the test suite
main "$@"
