#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
API_KEY="${API_KEY:-your_api_key_here}"
AGENT_FILE="${AGENT_FILE:-agent.tar.gz}"

echo "Testing upload agent endpoint..."
response=$(curl -s -w "%{http_code}" -X POST \
  -H "X-API-Key: $API_KEY" \
  -F "file=@$AGENT_FILE" \
  "$API_BASE_URL/api/v1/agents/upload")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
    echo "✓ Upload agent successful"
    echo "Response: $body"
else
    echo "✗ Upload agent failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi