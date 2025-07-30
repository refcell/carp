#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
AGENT_NAME="${AGENT_NAME:-test-agent}"
AGENT_VERSION="${AGENT_VERSION:-latest}"

echo "Testing download agent endpoint..."
response=$(curl -s -w "%{http_code}" -X GET \
  "$API_BASE_URL/api/v1/agents/$AGENT_NAME/$AGENT_VERSION/download")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ]; then
    echo "✓ Download agent successful"
    echo "Response: $body"
else
    echo "✗ Download agent failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi
