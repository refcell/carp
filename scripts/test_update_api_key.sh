#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
API_KEY="${API_KEY:-your_api_key_here}"
KEY_ID="${KEY_ID:-key_uuid_here}"

echo "Testing update API key endpoint..."
response=$(curl -s -w "%{http_code}" -X PUT \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"updated-key\"}" \
  "$API_BASE_URL/api/v1/auth/api-keys?id=$KEY_ID")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ]; then
    echo "✓ Update API key successful"
    echo "Response: $body"
else
    echo "✗ Update API key failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi