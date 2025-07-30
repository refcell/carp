#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
API_KEY="${API_KEY:-your_api_key_here}"

echo "Testing publish agent endpoint..."
response=$(curl -s -w "%{http_code}" -X POST \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"test-agent\",\"version\":\"1.0.0\"}" \
  "$API_BASE_URL/api/v1/agents/publish")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
    echo "✓ Publish agent successful"
    echo "Response: $body"
else
    echo "✗ Publish agent failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi