#\!/bin/bash

# Simple script to list agents from a Carp API server
# Usage: ./scripts/list-agents.sh [BASE_URL]

set -e

BASE_URL="${1:-http://localhost:3307}"
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Testing Carp API at $BASE_URL${NC}"
echo ""

# Test the search endpoint (should not require auth)
echo "Testing search endpoint..."
response=$(curl -s -w "HTTP_STATUS:%{http_code}" "$BASE_URL/api/v1/agents/search?limit=5")
http_status=$(echo "$response" | grep -o "HTTP_STATUS:[0-9]*" | cut -d: -f2)
body=$(echo "$response" | sed 's/HTTP_STATUS:[0-9]*$//')

if [ "$http_status" -eq 200 ]; then
    echo -e "${GREEN}✅ Search endpoint working (Status: $http_status)${NC}"
    
    # Try to parse and display results
    if command -v jq &> /dev/null; then
        agent_count=$(echo "$body" | jq -r '.total // 0')
        echo "Found $agent_count total agents"
        
        echo ""
        echo "First 5 agents:"
        echo "$body" | jq -r '.agents[]? | "  • \(.name) v\(.version) by \(.author)"'
    else
        echo "Response: $body"
    fi
else
    echo -e "${RED}❌ Search endpoint failed (Status: $http_status)${NC}"
    echo "Response: $body"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 API is working correctly\!${NC}"
EOF < /dev/null