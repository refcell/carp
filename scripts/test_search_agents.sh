#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
SEARCH_QUERY="${SEARCH_QUERY:-test}"

echo "Testing search agents endpoint..."
response=$(curl -s -w "%{http_code}" -X GET \
  "$API_BASE_URL/api/v1/agents/search?q=$SEARCH_QUERY")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ]; then
    echo "✓ Search agents successful"
    echo "Response: $body"
else
    echo "✗ Search agents failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi
